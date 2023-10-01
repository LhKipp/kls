use std::{path::PathBuf, sync::Arc};

use crate::buffer::Buffer;
use parser::{node, rec_descend, text_of};
use smallvec::SmallVec;

use parking_lot::RwLock;
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

    pub fn add_from_buffer(&self, buffer: &Buffer) {
        let mut package = Self::get_default_package_name();

        let mut indexes_w_lock = self.indexes.write();

        trace!("Visiting tree {}", buffer.tree.root_node().to_sexp());
        rec_descend(&buffer.tree.root_node(), |node: &Node| match node.kind() {
            "package_header" => {
                trace!("Visiting package_header {}", text_of(&node, &buffer.text));
                if let Some(identifier) = node.child_by_field_name("identifier") {
                    package = parser::text_of(&identifier, &buffer.text);
                }
                false
            }
            "class_declaration" => {
                let class_decl = node::ClassDecl::new(&node, &buffer.text);
                trace!("Visiting class with name {:?}", class_decl.name());
                if let Some(class_name) = class_decl.name() {
                    indexes_w_lock
                        .entry(class_name.clone().into())
                        .or_insert_with(|| smallvec::smallvec![])
                        .push(Symbol::new(
                            buffer.path.clone(),
                            class_decl.node.range(),
                            package.clone(),
                            SymbolKind::Class(SymbolClass { name: class_name }),
                        ));
                }
                true
            }
            _ => true,
        })
    }

    pub fn completions_for(&self, word: &str) -> Vec<CompletionItem> {
        trace!(
            "Getting completion for {} while indexes are \n{:?}",
            word,
            self.indexes.read()
        );
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
        }
        item
    }
}
