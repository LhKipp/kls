use std::{
    collections::{HashMap, VecDeque},
    path::{Path, PathBuf},
    sync::Arc,
};

use parser::node;
use smallvec::SmallVec;

use parking_lot::RwLock;
use stdx::new_arc_rw_lock;

use qp_trie::Trie;
use tracing::trace;
use tree_sitter::{Node, Range};

pub struct SymbolClass {
    pub name: String,
}

pub enum SymbolKind {
    Class(SymbolClass),
}

#[derive(new)]
pub struct Symbol {
    pub file: PathBuf,
    pub location: Range,
    pub package: String,
    pub kind: SymbolKind,
}

pub struct Indexes {
    pub asts: Arc<RwLock<HashMap<PathBuf, (tree_sitter::Tree, Vec<u8>)>>>,
    pub indexes: Arc<RwLock<Trie<Vec<u8>, SmallVec<[Symbol; 2]>>>>,
    // pub valid_locations: Arc<RwLock<HashMap<(Range, PathBuf), Vec<u8>>>>
}

fn rec_descend<F>(node: &Node, tree_source: &[u8], mut f: F)
where
    F: FnMut(&Node) -> bool,
{
    let mut visit = |node: &Node| -> bool {
        trace!(
            "visiting {}: {}",
            node.kind(),
            node.utf8_text(tree_source).unwrap()
        );
        f(node)
    };

    let mut queue = VecDeque::new();
    queue.push_back(node.clone());

    let mut cursor = node.walk();
    while let Some(node) = queue.pop_front() {
        if !visit(&node) {
            continue;
        }

        for child in node.children(&mut cursor) {
            queue.push_back(child);
        }
    }
}

impl Indexes {
    pub fn new() -> Self {
        Indexes {
            asts: new_arc_rw_lock(HashMap::new()),
            indexes: new_arc_rw_lock(Trie::new()),
        }
    }

    pub fn get_default_package_name(_: &Path) -> String {
        return "default".into();
        // info!("path {}", path.to_str().unwrap());
        // path.strip_prefix("src/main/kotlin")
        //     .unwrap()
        //     .to_str()
        //     .unwrap()
        //     .to_string()
        //     .replace("/", ".")
    }

    pub async fn add_from_file(&self, path: PathBuf) {
        let (tree, source) = crate::parse_kotlin::parse_file(&path);
        let tree = tree.unwrap();

        {
            let mut w_lock = self.asts.write();
            w_lock
                .entry(path.clone())
                .insert_entry((tree, source.into()));
        }

        let mut package = Self::get_default_package_name(&path);

        let r_lock = self.asts.read();
        let (stored_tree, source) = r_lock.get(&path).unwrap();
        let mut indexes_w_lock = self.indexes.write();

        rec_descend(&stored_tree.root_node(), source, |node: &Node| {
            match node.kind() {
                "package_header" => {
                    if let Some(identifier) = node.child_by_field_name("identifier") {
                        package = identifier.utf8_text(source).unwrap().to_string();
                    }
                    false
                }
                "class_declaration" => {
                    let class_decl = node::ClassDecl::new(&node, &source);
                    if let Some(class_name) = class_decl.name() {
                        indexes_w_lock
                            .entry(class_name.clone().into())
                            .or_insert_with(|| smallvec::smallvec![])
                            .push(Symbol::new(
                                path.clone(),
                                class_decl.node.range(),
                                package.clone(),
                                SymbolKind::Class(SymbolClass { name: class_name }),
                            ));
                    }
                    true
                }
                _ => true,
            }
        })
    }
}
