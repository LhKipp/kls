use core::fmt;
use std::{path::PathBuf, sync::Arc};

use crate::buffer::Buffer;
use parser::{node, rec_descend};
use smallvec::SmallVec;

use parking_lot::{
    lock_api::{RawRwLock, RwLockWriteGuard},
    RwLock,
};
use stdx::new_arc_rw_lock;

use qp_trie::Trie;
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

pub struct Indexes {
    pub indexes: Arc<RwLock<Trie<Vec<u8>, SmallVec<[Symbol; 2]>>>>,
    // pub valid_locations: Arc<RwLock<HashMap<(Range, PathBuf), Vec<u8>>>>
}

impl Indexes {
    pub fn new() -> Self {
        Indexes {
            indexes: new_arc_rw_lock(Trie::new()),
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
        w_lock: &mut RwLockWriteGuard<'_, Lock, Trie<Vec<u8>, SmallVec<[Symbol; 2]>>>,
        package: String,
        symbol: Symbol,
    ) {
        w_lock
            .entry(package.clone().into())
            .or_insert_with(|| smallvec::smallvec![])
            .push(symbol)
    }

    pub fn add_from_buffer(&self, buffer: &Buffer) {
        let mut package = Self::get_default_package_name();

        let mut indexes_w_lock = self.indexes.write();

        trace!("Visiting tree {}", buffer.tree.root_node().to_sexp());
        rec_descend(&buffer.tree.root_node(), |node: &Node| match node.kind() {
            "package_header" => {
                let package_decl = node::PackageDecl::new(&node, &buffer.text);
                trace!("Visiting package_header {:?}", package_decl.package_ident());
                if let Some(identifier) = package_decl.package_ident() {
                    package = identifier.clone();
                    self.add_symbol(
                        &mut indexes_w_lock,
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
                        &mut indexes_w_lock,
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
        })
    }

    pub fn completions_for(&self, word: &str) -> Vec<CompletionItem> {
        trace!("Completing {} with indexes \n{:?}", word, self);
        self.indexes
            .read()
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
            .indexes
            .read()
            .iter()
            .flat_map(|(k, symbols)| symbols.iter().map(|s| (k.clone(), s)))
            .map(|(k, symbol)| format!("{} -> {:?}\n", String::from_utf8(k).unwrap(), symbol))
            .collect();

        f.write_str(&s)
    }
}
