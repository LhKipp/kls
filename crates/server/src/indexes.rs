use core::fmt;
use std::{path::PathBuf, sync::Arc};
use tower_lsp::jsonrpc::Result;

use crate::buffer::Buffer;
use parser::{node, rec_descend};
use smallvec::SmallVec;

use parking_lot::{
    lock_api::{RawRwLock, RwLockWriteGuard},
    RwLock,
};
use stdx::new_arc_rw_lock;

use qp_trie::Trie;
use theban_interval_tree::IntervalTree;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};
use tracing::trace;
use tree_sitter::{Node, Range};

#[derive(Debug)]
pub struct SymbolClass {
    pub name: String,
}

#[derive(Debug)]
pub enum SymbolKind {
    Class(SymbolClass),
    Package,
}

#[derive(new, Debug)]
pub struct Symbol {
    pub file: PathBuf,
    pub location: Range,
    pub package: String,
    pub kind: SymbolKind,
}

#[derive(Clone)]
struct AllIndexLookupData {
    name_key: Vec<u8>,
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

    fn add_symbol<Lock: RawRwLock>(
        &self,
        w_lock: &mut RwLockWriteGuard<'_, Lock, Data>,
        name_key: String,
        symbol: Symbol,
    ) {
        w_lock.roots.insert(
            to_memrange(&symbol.location),
            AllIndexLookupData {
                name_key: name_key.clone().into(),
            },
        );
        w_lock
            .indexes
            .entry(name_key.into())
            .or_insert_with(|| smallvec::smallvec![])
            .push(symbol);
    }

    fn delete_symbols<Lock: RawRwLock>(
        &self,
        w_lock: &mut RwLockWriteGuard<'_, Lock, Data>,
        range: &Range,
    ) {
        let roots: Vec<_> = w_lock
            .roots
            .range(range.start_byte as u64, range.end_byte as u64)
            // TODO erase clone. Needs removing the Data struct?!
            .map(|(range, idx_data)| (range, idx_data.clone()))
            .collect();
        for (range, keys) in roots {
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
        edited_ranges: &Vec<Range>,
    ) -> Result<()> {
        let mut w_lock = self.data.write();

        for change in edited_ranges {
            // delete indexes for range
            self.delete_symbols(&mut w_lock, &change);

            // Add indexes for new nodes
            if let Some(new_node) = buffer
                .tree
                .root_node()
                .descendant_for_byte_range(change.start_byte, change.end_byte)
            {
                self.add_from_node(&mut w_lock, &buffer, &new_node)?
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
                    self.add_symbol(
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
                    self.add_symbol(
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

fn to_memrange(range: &Range) -> memrange::Range {
    memrange::Range::new(range.start_byte as u64, range.end_byte as u64)
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
