use std::collections::HashMap;

use crate::scope::*;
use anyhow::bail;
use atree::{Arena, Token};
use stdx::{new_arc_rw_lock, ARwLock};

pub(crate) struct Scopes {
    pub(crate) scopes: ARwLock<Arena<ARwLock<Scope>>>,
    // protected by scopes lock
    project_nodes: HashMap<String, Token>,
}

impl Scopes {
    pub fn new() -> Self {
        Self {
            scopes: new_arc_rw_lock(Arena::new()),
            project_nodes: HashMap::new(),
        }
    }
    pub fn add_project(&self, name: String) {
        let mut w_lock = self.scopes.write();
        let token = w_lock.new_node(new_arc_rw_lock(Scope::new(SKind::Project {
            name: name.clone(),
        })));
        self.project_nodes.insert(name, token);
    }
    pub fn add_module(&self, project_name: &str, scopes: ScopeTree) -> anyhow::Result<()> {
        let w_scopes = self.scopes.write();
        let Some(project_token) = self.project_nodes.get(project_name) else {
            bail!("No project found with name {}", project_name)
        };
        // TODO move_and_append_subtree
        w_scopes.copy_and_append_subtree(project_token, scopes, )
        project_token.append(&mut *w_scopes, new_arc_rw_lock(scopes));
    }
}
