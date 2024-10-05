use crate::scope::{SKind, Scope};
use indextree::NodeId;
use std::{
    cell::{Ref, RefCell},
    ops::Range,
};
use tracing::{debug, warn};
use tree_sitter::{Node, Tree};

use crate::{range_util::TextRange, scope::GSFile};

mod package_header;

#[derive(Debug)]
pub enum UpsertOrDelete {
    Delete,
    Upsert,
}

#[derive(Debug)]
pub struct ChangedRange(pub TextRange, pub UpsertOrDelete);

#[derive(new)]
pub struct ScopeBuilder<'a> {
    pub s_file: &'a mut GSFile,
    pub changed_range: ChangedRange,
}

impl<'a> ScopeBuilder<'a> {
    pub fn update_scopes(&'a mut self, tree: &Tree) -> anyhow::Result<()> {
        debug!(
            "updating scopes for changed ranges {:?}",
            self.changed_range
        );
        let upsert_range = match self.changed_range {
            ChangedRange(r, UpsertOrDelete::Delete) => {
                self.delete_scope(r);
                return Ok(());
            }
            ChangedRange(r, UpsertOrDelete::Upsert) => r,
        };

        let Some(existing_scope_id) = self
            .s_file
            .scope_having_best_match(&|scope| scope.range.contains_range(upsert_range))
        else {
            self.insert_top_level_scope(tree, upsert_range);
            return Ok(());
        };

        let existing_scope = self.s_file.scopes.get(existing_scope_id).unwrap().get();

        // Check whether update replaces existing scope with a different scope kind
        if upsert_range.contains_range(existing_scope.range) {
            // just delete and reinsert
            debug!("removing existing scope as it got completely replaced during update");
            existing_scope_id.remove_subtree(&mut self.s_file.scopes);
            self.insert_top_level_scope(tree, upsert_range);
            return Ok(());
        }

        // Update existing scope

        let Some(ast_node) = Self::node_at(tree, existing_scope.range) else {
            warn!(
                "expected to update existing scope {:?}, but found no ast-node at {}",
                existing_scope, existing_scope.range
            );
            return Ok(());
        };
        debug!("updating existing scope {:?}", existing_scope.range);

        match &existing_scope.kind {
            SKind::PackageHeader { .. } => {
                package_header::update_package_header(self, &existing_scope_id, tree, &ast_node)
            }
        }
    }

    pub fn node_at(tree: &tree_sitter::Tree, r: TextRange) -> Option<Node> {
        let mut cursor = tree.walk();
        let Some(_) = cursor.goto_first_child_for_byte(r.start as usize) else {
            warn!(
                "expected to find a child in the ast at {}, but found none",
                r.start
            );
            return None;
        };
        Some(cursor.node())
    }

    pub fn insert_top_level_scope(&mut self, tree: &Tree, r: TextRange) {
        let Some(node) = Self::node_at(tree, r) else {
            warn!("expected to insert a top level scope, but found no ast-node at {r}");
            return;
        };

        debug!("inserting top level scope for range {r}");
        let node_kind_id = node.kind_id();
        if node_kind_id == *parser::PackageHeader {
            package_header::insert_package_header(self, node);
        } else {
            warn!("Unhandled to insert node of kind {}", node.kind());
        }
    }

    pub fn delete_scope(&mut self, r: TextRange) {
        if let Some(scope) = self
            .s_file
            .scope_having_best_match(&|scope| scope.range == r)
        {
            debug!(
                "deleting scope {:?} at {}",
                self.s_file.scopes.get(scope).unwrap().get().kind,
                r
            );
            if let Some(root_node_id) = self
                .s_file
                .root_nodes
                .iter()
                .position(|n| *n == scope)
            {
                self.s_file.root_nodes.remove(root_node_id);
            }
            scope.remove_subtree(&mut self.s_file.scopes)
        } else {
            warn!("Got text-range to delete {r}, but found no single scope matching it")
        }
    }
}
