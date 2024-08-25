mod file_scope;
mod project_scope;
mod source_set_scope;

pub use file_scope::SFile;
pub use project_scope::SProject;
pub use source_set_scope::SSourceSet;

use tokio::task::JoinHandle;
use tracing::{debug, error};

use crate::project::{PSourceSet, ProjectI};
use enum_as_inner::EnumAsInner;
use indextree::{Arena, NodeId};
use std::{collections::HashMap, fmt, path::PathBuf, sync::Arc};
use stdx::{new_arc_rw_lock, ARwLock};
use tree_sitter::{Node, Range};

#[derive(Clone)]
pub struct Scopes(pub ARwLock<ScopesData>);

impl Scopes {
    pub fn new() -> Self {
        Scopes(new_arc_rw_lock(ScopesData::new()))
    }

    /// Adds scopes
    pub async fn add_scopes_from_project_recursive(
        &self,
        project: Box<dyn ProjectI>,
    ) -> anyhow::Result<()> {
        let (project_node_id, s_project) = SProject::create_project_scope(self, &project)?;
        let source_sets = SSourceSet::create_source_set_scopes(self, project_node_id, &s_project)?;

        for (source_set_node_id, source_set) in source_sets {
            let scopes = self.clone();
            tokio::spawn(async move {
                if let Err(e) =
                    SFile::create_file_scopes(scopes, source_set_node_id, &source_set).await
                {
                    error!(
                        "Error while creating files of source_set {:?} - {}",
                        source_set.read().kind.as_source_set().unwrap(),
                        e
                    );
                }
            });
        }

        Ok(())
    }
}

pub struct ScopesData {
    pub(crate) scopes: indextree::Arena<ARwLock<Scope>>,
    /// root nodes in scopes
    pub project_nodes: Vec<NodeId>,
    pub file_nodes: HashMap<PathBuf, NodeId>,
}

impl ScopesData {
    pub fn debug_fmt_scopes(&self) -> anyhow::Result<String> {
        let result = String::with_capacity(1024);

        // let format_kind = |kind: &SKind, depth: usize| {
        //     result += &" ".repeat(depth);
        // };

        // for root in self.project_nodes {
        //     for n_id in root.descendants(&self.scopes) {
        //         format_kind()
        //         let value = &self.scopes.get(n_id).ok_or_else(||format!("Could not find node in arena"))?.get().read().kind
        //     }
        // }

        Ok(result)
    }

    pub fn new() -> Self {
        ScopesData {
            scopes: Arena::new(),
            project_nodes: vec![],
            file_nodes: HashMap::new(),
        }
    }
}

pub type ARwScope = ARwLock<Scope>;
#[derive(new, Debug)]
pub struct Scope {
    pub kind: SKind,
}

impl Scope {
    pub fn new_arw(kind: SKind) -> ARwScope {
        new_arc_rw_lock(Scope { kind })
    }
}

// pub type WScope<'a> = RwLockWriteGuard<'a, Scope>;
// pub type RScope<'a> = RwLockReadGuard<'a, Scope>;

#[derive(Debug, EnumAsInner)]
pub enum SKind {
    Project(SProject),
    SourceSet(SSourceSet),
    File(SFile),
    // Module { path: PathBuf, range: Range },
    // Class { name: String, range: Range },
    // Function(String /*name*/),
    // MemberFunction(String /*name*/),
}
