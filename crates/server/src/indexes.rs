use crate::symbol::NODES_CONTAINING_SYMBOLS;
use crate::{
    range_util::{ChangedRanges, TextByteRange},
    symbol::{Symbol, SymbolBuilder},
};
use core::fmt;
use std::sync::Arc;
use tower_lsp::{jsonrpc::Result, lsp_types::CompletionItem};

use crate::{buffer::Buffer, range_util};
use smallvec::SmallVec;

use parking_lot::{
    lock_api::{RawRwLock, RwLockWriteGuard},
    RwLock,
};
use stdx::new_arc_rw_lock;

use qp_trie::Trie;
use theban_interval_tree::IntervalTree;
use tracing::{error, trace};
use tree_sitter::Node;

#[derive(Clone)]
struct AllIndexLookupData {
    name_key: Vec<u8>,
    name_behind_package_key: Option<Vec<u8>>,
    symbol_id: i32,
}

struct Data {
    pub indexes: Trie<Vec<u8>, SmallVec<[Symbol; 2]>>,
    pub roots: IntervalTree<AllIndexLookupData>,
}

pub struct Indexes {
    data: Arc<RwLock<Data>>,
}

impl Indexes {
    pub fn new() -> Self {
        Indexes {
            data: new_arc_rw_lock(Data {
                indexes: Trie::new(),
                roots: IntervalTree::new(),
            }),
        }
    }

    pub fn get_default_package_name() -> String {
        return "default".into();
        // info!("path {}", path.to_str().unwrap());
        // path.strip_prefix("src/main/kotlin")
        //     .unwrap()
        //     .to_str()
        //     .unwrap()
        //     .to_string()
        //     .replace("/", ".")
    }

    // insert symbol, updating existing symbol if one is already present
    fn insert_symbol<Lock: RawRwLock>(
        &self,
        w_lock: &mut RwLockWriteGuard<'_, Lock, Data>,
        symbol: Symbol,
    ) {
        trace!(
            "Inserting index for symbol {} ({:?})",
            symbol.name,
            symbol.kind
        );

        let roots_range = range_util::to_memrange(&symbol.location);

        // delete old entries
        if let Some(root_data) = w_lock.roots.get(roots_range).cloned() {
            w_lock.indexes.remove(&root_data.name_key);
            if let Some(name_behind_package) = &root_data.name_behind_package_key {
                w_lock.indexes.remove(name_behind_package);
            }
        }

        // insert new entry in roots
        w_lock.roots.insert(
            roots_range,
            AllIndexLookupData {
                name_key: symbol.name.clone().into(),
                name_behind_package_key: symbol.name_behind_package(),
                symbol_id: symbol.id,
            },
        );

        // insert new entries in indexes
        if symbol.duplicate_behind_package {
            let symbol_behind_package = symbol.clone_as_behind_package();
            w_lock
                .indexes
                .entry(symbol_behind_package.name.clone().into())
                .or_insert_with(|| smallvec::smallvec![])
                .push(symbol_behind_package);
        }
        w_lock
            .indexes
            .entry(symbol.name.clone().into())
            .or_insert_with(|| smallvec::smallvec![])
            .push(symbol);
    }

    fn delete_symbols<Lock: RawRwLock>(
        &self,
        w_lock: &mut RwLockWriteGuard<'_, Lock, Data>,
        edit_range: &TextByteRange,
    ) {
        let roots: Vec<_> = w_lock
            .roots
            .range(edit_range.start as u64, edit_range.end as u64)
            // TODO erase clone. Needs removing the Data struct?!
            .map(|(range, idx_data)| (range, idx_data.clone()))
            .collect();
        for (range, keys) in roots {
            trace!(
                "Removing indexes for range {:?}, because of edit_range {:?}",
                range,
                edit_range
            );
            if let Some(name_behind_package_key) = &keys.name_behind_package_key {
                w_lock.indexes.remove(name_behind_package_key);
            }
            w_lock.indexes.remove(&keys.name_key);
            w_lock.roots.delete(range)
        }
    }

    pub fn add_from_buffer(&self, buffer: &Buffer) -> Result<()> {
        let mut w_lock = self.data.write();
        self.add_from_node(&mut w_lock, &buffer, &buffer.tree.root_node())
    }

    pub fn add_from_buffer_changes(
        &self,
        buffer: &Buffer,
        edited_ranges: &ChangedRanges,
    ) -> Result<()> {
        let mut w_lock = self.data.write();

        // delete indexes for old ranges
        for change in &edited_ranges.old_ranges {
            self.delete_symbols(&mut w_lock, &change);
        }
        // Add indexes for new nodes
        for change in &edited_ranges.new_ranges {
            if let Some(new_node) = buffer
                .tree
                .root_node()
                .descendant_for_byte_range(change.start as usize, change.end as usize)
            {
                // The changed node might not directly map to a node, symbols are build from
                // To upate the node having the change, the ancestore mapping to a symbol must be
                // rebuilt
                if let Some(new_node) = find_ancestor_mapping_to_symbol(new_node) {
                    self.add_from_node(&mut w_lock, &buffer, &new_node)?;
                }
            }
        }
        Ok(())
    }

    fn add_from_node<Lock: RawRwLock>(
        &self,
        w_lock: &mut RwLockWriteGuard<'_, Lock, Data>,
        buffer: &Buffer,
        node: &Node,
    ) -> Result<()> {
        // TODO how to handle default package name?
        let _package = Self::get_default_package_name();

        trace!("Adding from node {}", node.to_sexp());
        let mut symbol_builder = SymbolBuilder::new(&buffer.path, &buffer.text);
        symbol_builder.build_all_symbols_for(*node);
        let (symbols, errors) = symbol_builder.finish();
        if !errors.is_empty() {
            error!(
                "Errors building symbols from file {}: {}",
                buffer.path.display(),
                errors.join(",")
            );
        }

        for symbol in symbols {
            self.insert_symbol(w_lock, symbol);
        }

        Ok(())
    }

    pub fn completions_for(&self, word: &str) -> Vec<CompletionItem> {
        trace!("Completing {} with indexes \n{:?}", word, self);
        self.data
            .read()
            .indexes
            .iter_prefix(word.as_bytes())
            .flat_map(|(_, symbols)| symbols)
            .map(|symbol| symbol.to_completion_item())
            .collect()
    }
}

impl fmt::Debug for Indexes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = self
            .data
            .read()
            .indexes
            .iter()
            .flat_map(|(k, symbols)| symbols.iter().map(|s| (k.clone(), s)))
            .map(|(k, symbol)| format!("{} -> {:?}\n", String::from_utf8(k).unwrap(), symbol))
            .collect();

        f.write_str(&s)
    }
}

fn find_ancestor_mapping_to_symbol(mut node: Node) -> Option<Node> {
    trace!("Finding node mapping to symbol for {}", node.kind());
    while !NODES_CONTAINING_SYMBOLS.contains(node.kind()) {
        node = node.parent()?;
    }
    Some(node)
}
