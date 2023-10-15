use crate::range_util::{ChangedRanges, TextByteRange};
use core::fmt;
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{atomic::AtomicI32, atomic::Ordering, Arc},
};
use tower_lsp::jsonrpc::Result;

use crate::{buffer::Buffer, range_util};
use parser::{node, rec_descend};
use smallvec::SmallVec;

use parking_lot::{
    lock_api::{RawRwLock, RwLockWriteGuard},
    RwLock,
};
use stdx::new_arc_rw_lock;

use lazy_static::lazy_static;
use qp_trie::Trie;
use theban_interval_tree::IntervalTree;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};
use tracing::{error, trace};
use tree_sitter::{Node, Range};

lazy_static! {
    static ref NODES_CONTAINING_SYMBOLS: HashSet<&'static str> =
        HashSet::from(["source_file", "package_header", "class_declaration"]);
}

#[derive(Debug)]
pub struct SymbolClass {
    pub name: String,
}

#[derive(Debug)]
pub enum SymbolKind {
    Class(SymbolClass),
    Package,
}

static SYMBOL_ID_GENERATOR: AtomicI32 = AtomicI32::new(1);
#[derive(new, Debug)]
pub struct Symbol {
    #[new(value = "SYMBOL_ID_GENERATOR.fetch_add(1, Ordering::SeqCst)")]
    id: i32,
    pub file: PathBuf,
    pub location: Range,
    pub package: String,
    pub kind: SymbolKind,
}

#[derive(Clone)]
struct AllIndexLookupData {
    name_key: Vec<u8>,
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
        name_key: String,
        symbol: Symbol,
    ) {
        let roots_range = range_util::to_memrange(&symbol.location);
        let symbol_already_present = w_lock.roots.get(roots_range).cloned();

        // insert overwrites
        w_lock.roots.insert(
            roots_range,
            AllIndexLookupData {
                name_key: name_key.clone().into(),
                symbol_id: symbol.id,
            },
        );

        if let Some(root_data) = symbol_already_present {
            match w_lock.indexes.entry(root_data.name_key.clone()) {
                qp_trie::Entry::Vacant(_) => error!(
                    "Expected entry for {} is missing",
                    String::from_utf8(root_data.name_key).unwrap()
                ),
                qp_trie::Entry::Occupied(mut entry) => {
                    match entry
                        .get_mut()
                        .iter_mut()
                        .filter(|symbol| symbol.id == root_data.symbol_id)
                        .next()
                    {
                        Some(old_symbol) => *old_symbol = symbol,
                        None => error!(
                            "Expected symbol with name {} (id: {}) is missing",
                            String::from_utf8(root_data.name_key).unwrap(),
                            root_data.symbol_id
                        ),
                    }
                }
            }
        } else {
            w_lock
                .indexes
                .entry(name_key.into())
                .or_insert_with(|| smallvec::smallvec![])
                .push(symbol);
        }
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
        mut w_lock: &mut RwLockWriteGuard<'_, Lock, Data>,
        buffer: &Buffer,
        node: &Node,
    ) -> Result<()> {
        let mut package = Self::get_default_package_name();
        trace!("Adding from node {}", node.to_sexp());
        rec_descend(node, |node: &Node| match node.kind() {
            "package_header" => {
                let package_decl = node::PackageDecl::new(&node, &buffer.text);
                trace!("Visiting package_header {:?}", package_decl.package_ident());
                if let Some(identifier) = package_decl.package_ident() {
                    package = identifier.clone();
                    self.insert_symbol(
                        &mut w_lock,
                        package.clone(),
                        Symbol::new(
                            buffer.path.clone(),
                            package_decl.node.range(),
                            package.clone(),
                            SymbolKind::Package,
                        ),
                    );
                }
                false
            }
            "class_declaration" => {
                let class_decl = node::ClassDecl::new(&node, &buffer.text);
                trace!("Visiting class with name {:?}", class_decl.name());
                if let Some(class_name) = class_decl.name() {
                    self.insert_symbol(
                        &mut w_lock,
                        class_name.clone(),
                        Symbol::new(
                            buffer.path.clone(),
                            class_decl.node.range(),
                            package.clone(),
                            SymbolKind::Class(SymbolClass { name: class_name }),
                        ),
                    );
                }
                true
            }
            _ => true,
        });
        Ok(())
    }

    pub fn completions_for(&self, word: &str) -> Vec<CompletionItem> {
        trace!("Completing {} with indexes \n{:?}", word, self);
        self.data
            .read()
            .indexes
            .iter_prefix(word.as_bytes())
            .flat_map(|(_, symbols)| symbols)
            .map(|symbol| self.to_completion_item(symbol))
            .collect()
    }

    fn to_completion_item(&self, symbol: &Symbol) -> CompletionItem {
        let mut item = CompletionItem::default();
        match &symbol.kind {
            SymbolKind::Class(class_symbol) => {
                item.label = class_symbol.name.clone();
                item.kind = Some(CompletionItemKind::CLASS);
            }
            SymbolKind::Package => {
                item.label = symbol.package.clone();
                item.kind = Some(CompletionItemKind::MODULE)
            }
        }
        trace!("Mapped {:?} to {:?}", symbol, item);
        item
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
