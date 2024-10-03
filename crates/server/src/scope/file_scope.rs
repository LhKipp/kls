use crate::project::{PProject, ProjectI};
use crate::range_util::TextRange;
use anyhow::bail;
use crop::Rope;
use indextree::*;
use itertools::Itertools;
use std::cell::RefCell;
use std::ops::Range;
use std::path::PathBuf;
use tap::Tap;
use tokio::fs;
use tracing::trace;

use super::Scope;

#[derive(Debug, new)]
pub struct GSFile {
    pub path: PathBuf,
    pub text: Rope,
    pub ast: tree_sitter::Tree,
    #[new(default)]
    pub scopes: indextree::Arena<Scope>,
    #[new(default)]
    pub root_nodes: Vec<NodeId>,
}

impl GSFile {
    pub fn scope_at_byte(&mut self, byte: u32) -> Option<NodeId> {
        self.scope_having_best_match(&|scope| scope.range.contains(byte))
    }

    /// iterates down the scope-tree, until condition is not satisfied
    /// If no scope satisfies `condition`, returns None
    pub fn scope_having_best_match(&self, condition: &dyn Fn(&Scope) -> bool) -> Option<NodeId> {
        let mut current = *self
            .root_nodes
            .clone() // CLONE
            .iter()
            .find(|n| {
                let root_scope = self.root_scope(n);
                return condition(root_scope.get());
            })?;

        'outer: loop {
            for child_scope in current.children(&self.scopes) {
                if condition(self.scopes.get(child_scope).unwrap().get()) {
                    current = child_scope;
                    continue 'outer;
                }
            }
            return Some(current);
        }
    }

    fn root_scope(&self, node_id: &NodeId) -> &Node<Scope> {
        return self
            .scopes
            .get(*node_id)
            .unwrap_or_else(|| panic!("No scope for node-id {}", node_id));
    }

    pub fn new_root_scope(&mut self, scope: Scope) {
        let id = self.scopes.new_node(scope);
        self.root_nodes.push(id);
    }
}
