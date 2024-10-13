use crate::scope::{SKind, Scope};
use indextree::NodeId;
use std::{
    cell::{Ref, RefCell},
    ops::Range,
};
use stdx::TextRange;
use tracing::{debug, warn};
use tree_sitter::{Node, Tree};

use crate::scope::GSFile;

mod function_declaration;
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
        if self.changed_range.0.is_empty() {
            debug!("Not updating scopes, because the changed range is empty");
            return Ok(());
        }

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
            .scope_having_best_match(&|scope| scope.range.overlaps_with(upsert_range))
        else {
            return self.insert_top_level_scopes(tree, upsert_range);
        };

        // CLONE
        let existing_scope = self
            .s_file
            .scopes
            .get(existing_scope_id)
            .unwrap()
            .get()
            .clone();

        // Check whether update replaces existing scope with a different scope kind
        if upsert_range.contains_range(existing_scope.range) {
            // just delete and reinsert
            debug!("removing existing scope as it got completely replaced during update");
            self.s_file.delete_scope(existing_scope_id);
            return self.insert_top_level_scopes(tree, upsert_range);
        }

        // Update existing scope

        let mut cursor = tree.walk();
        parser::first_descendant_for_byte(&mut cursor, upsert_range.start)?;

        loop {
            debug!("updating existing scope {:?}", existing_scope.kind);
            match &existing_scope.kind {
                SKind::PackageHeader { .. } => package_header::update_package_header(
                    self,
                    &existing_scope_id,
                    tree,
                    &cursor.node(),
                )?,
                SKind::FunDecl(_s_fun_decl) => function_declaration::update_function_declaration(
                    self,
                    existing_scope_id,
                    tree,
                    &mut cursor.clone(),
                    upsert_range,
                )?,
            };

            if parser::move_right(&mut cursor, parser::MoveMode::SkipUnnamed).is_ok()
                && cursor.node().start_byte() < upsert_range.end as usize
            {
                // TODO maybe update the existing_scope?
                // existing_scope = ;
                // another iteration => another update
            } else {
                break;
            };
        }

        Ok(())
    }

    pub fn insert_top_level_scopes(&mut self, tree: &Tree, r: TextRange) -> anyhow::Result<()> {
        debug!("Inserting top level scope for text at {}", r);
        let mut cursor = tree.walk();
        let node = parser::first_child_for_byte(&mut cursor, r.start)?;

        loop {
            debug!("inserting top level scope for range {r}");
            let node_kind_id = cursor.node().kind_id();

            let scope = if node_kind_id == *parser::node::PackageHeaderId {
                package_header::create_package_header(self, node)
            } else if node_kind_id == *parser::node::FunctionDeclarationId {
                function_declaration::create_fun_decl(self, node)
            } else {
                warn!("Unhandled to insert node of kind {}", node.kind());
                Ok(None)
            };

            if let Some(scope) = scope? {
                self.s_file.new_root_scope(scope);
            }

            if !cursor.goto_next_sibling() {
                debug!("No more ast nodes present. stopping to insert");
                return Ok(());
            }
            if cursor.node().byte_range().start >= r.end as usize {
                debug!(
                    "Not inserting {} as the node is after the passed range to insert",
                    cursor.node().kind_id()
                );
                return Ok(());
            }
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
            self.s_file.delete_scope(scope);
        } else {
            warn!("Got text-range to delete {r}, but found no single scope matching it")
        }
    }
}
