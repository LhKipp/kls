mod file_scope;
mod file_scope_creation;
pub mod fun_decl_scope;
mod project_scope;
mod source_set_scope;

pub use file_scope::GSFile;
pub use fun_decl_scope::SFunDecl;
pub use project_scope::GSProject;
pub use source_set_scope::GSSourceSet;

use crate::project::{PSourceSet, ProjectI};
use enum_as_inner::EnumAsInner;
use indextree::{Arena, NodeId};
use std::{collections::HashMap, fmt, path::PathBuf, sync::Arc};
use stdx::{new_arc_rw_lock, ARwLock, TextRange};
use tokio::task::JoinHandle;
use tracing::{debug, error};
use tree_sitter::{Node, Range};

use self::file_scope_creation::create_file_scopes;

/// Global scopes are protected by a AMtx and operation on them can be concurrent. In comparison
/// normal [scope::Scope]'s, which are used on a file level and below, are not protected by a AMtx.
/// Once we are on a file level, we use non AMtx scopes to make life easier ...
#[derive(Clone)]
pub struct GScopes(pub ARwLock<GScopesData>);

impl GScopes {
    pub fn new() -> Self {
        GScopes(new_arc_rw_lock(GScopesData::new()))
    }

    /// Adds scopes
    pub async fn add_scopes_from_project_recursive(
        &self,
        project: Box<dyn ProjectI>,
    ) -> anyhow::Result<()> {
        let (project_node_id, s_project) = GSProject::create_project_scope(self, &project)?;
        let source_sets = GSSourceSet::create_source_set_scopes(self, project_node_id, &s_project)?;

        for (source_set_node_id, source_set) in source_sets {
            let scopes = self.clone();
            tokio::spawn(async move {
                if let Err(e) = create_file_scopes(scopes, source_set_node_id, &source_set).await {
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

impl Default for GScopes {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GScopesData {
    pub(crate) scopes: indextree::Arena<ARwLock<GScope>>,
    /// root nodes in scopes
    pub project_nodes: Vec<NodeId>,
    pub file_nodes: HashMap<PathBuf, NodeId>,
}

impl GScopesData {
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
        GScopesData {
            scopes: Arena::new(),
            project_nodes: vec![],
            file_nodes: HashMap::new(),
        }
    }
}

impl Default for GScopesData {
    fn default() -> Self {
        Self::new()
    }
}

pub type GARwScope = ARwLock<GScope>;
#[derive(new, Debug)]
pub struct GScope {
    pub kind: GSKind,
}

impl GScope {
    pub fn new_arw(kind: GSKind) -> GARwScope {
        new_arc_rw_lock(GScope { kind })
    }
}

// pub type WScope<'a> = RwLockWriteGuard<'a, Scope>;
// pub type RScope<'a> = RwLockReadGuard<'a, Scope>;

#[derive(Debug, EnumAsInner)]
pub enum GSKind {
    Project(GSProject),
    SourceSet(GSSourceSet),
    File(GSFile),
    // Module { path: PathBuf, range: Range },
    // Class { name: String, range: Range },
    // Function(String /*name*/),
    // MemberFunction(String /*name*/),
}

#[derive(Debug, new, Clone)]
pub struct Scope {
    pub kind: SKind,
    pub range: TextRange,
}

#[derive(Debug, EnumAsInner, Clone)]
pub enum SKind {
    PackageHeader { ident: String },
    FunDecl(SFunDecl),
    // Class { name: String, range: Range },
    // Function(String /*name*/),
    // MemberFunction(String /*name*/),
}

impl SKind {
    fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}
