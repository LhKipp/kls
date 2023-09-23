use std::{collections::HashMap, path::PathBuf, sync::Arc};

use parking_lot::RwLock;
use stdx::new_arc_rw_lock;

use qp_trie::Trie;
use tracing::info;

pub struct Symbol {}

pub struct Indexes {
    pub asts: Arc<RwLock<HashMap<PathBuf, tree_sitter::Tree>>>,
    pub indexes: Arc<RwLock<Trie<Vec<u8>, Symbol>>>,
}

impl Indexes {
    pub fn new() -> Self {
        Indexes {
            asts: new_arc_rw_lock(HashMap::new()),
            indexes: new_arc_rw_lock(Trie::new()),
        }
    }

    pub async fn add_from_file(&self, path: PathBuf) {
        let ast = crate::parse_kotlin::parse_file(&path).unwrap();

        self.asts.write().insert(path, ast.clone());

        let cursor = ast.walk();
        info!(
            "cursor field-name {}",
            cursor.field_name().unwrap_or("no field-name present")
        );
        // cursor.field_name();
    }
}
