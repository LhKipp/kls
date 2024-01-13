use std::collections::HashMap;

use crate::scope::*;
use anyhow::bail;
use atree::{Arena, Token};
use stdx::{new_arc_rw_lock, ARwLock};

pub struct Scopes {
    data: ARwLock<ScopesData>,
}

struct ScopesData {
    pub(crate) scopes: Arena<ARwLock<Scope>>,
    project_nodes: HashMap<String, Token>,
}

impl Scopes {
    pub fn new() -> Self {
        Self {
            data: stdx::new_arc_rw_lock(ScopesData {
                scopes: Arena::new(),
                project_nodes: HashMap::new(),
            }),
        }
    }
    pub fn add_project(&self, name: String) {
        let mut w_lock = self.data.write();
        let token = w_lock
            .scopes
            .new_node(new_arc_rw_lock(Scope::new(SKind::Project {
                name: name.clone(),
            })));
        w_lock.project_nodes.insert(name, token);
    }
    pub fn add_module(&self, project_name: &str, scopes: ScopeTree) -> anyhow::Result<()> {
        let mut w_data = self.data.write();
        let Some(project_token) = w_data.project_nodes.get(project_name).cloned() else {
            bail!("No project found with name {}", project_name)
        };
        w_data
            .scopes
            .copy_and_append_subtree(project_token, &scopes.tree, scopes.root);
        return Ok(());
    }
}
