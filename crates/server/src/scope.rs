mod debug_print;
mod project_scope;

pub use debug_print::ScopeDebugPrettyPrint;
pub use project_scope::SProject;

use crate::project::PSourceSet;
use enum_as_inner::EnumAsInner;
use indextree::{Arena, NodeId};
use std::{fmt, path::PathBuf, sync::Arc};
use stdx::{new_arc_rw_lock, ARwLock};
use tree_sitter::{Node, Range};

pub type Scopes = ARwLock<ScopesData>;
pub fn new_scopes() -> Scopes {
    return new_arc_rw_lock(ScopesData::new());
}

pub struct ScopesData {
    pub(crate) scopes: indextree::Arena<ARwLock<Scope>>,
    /// root nodes in scopes
    pub project_nodes: Vec<NodeId>,
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

        return Ok(result);
    }

    pub fn new() -> Self {
        ScopesData {
            scopes: Arena::new(),
            project_nodes: vec![],
        }
    }
}

#[derive(new)]
pub struct Scope {
    pub kind: SKind,
}

impl Scope {
    pub fn new_arw(kind: SKind) -> ARwLock<Self> {
        return new_arc_rw_lock(Scope { kind });
    }
}

// pub type WScope<'a> = RwLockWriteGuard<'a, Scope>;
// pub type RScope<'a> = RwLockReadGuard<'a, Scope>;

#[derive(Debug, EnumAsInner)]
pub enum SKind {
    Project(SProject),
    SourceSet(SSourceSet),
    // Module { path: PathBuf, range: Range },
    // Class { name: String, range: Range },
    // Function(String /*name*/),
    // MemberFunction(String /*name*/),
}

#[derive(Debug)]
pub struct SSourceSet {
    data: PSourceSet,
}
