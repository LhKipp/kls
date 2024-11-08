use anyhow::{bail, ensure};
use indextree::NodeId;
use parser::node::PackageHeader;
use std::{cell::RefCell, thread::panicking};
use tracing::debug;
use tree_sitter::{Node, Tree, TreeCursor};

use crate::scope::{SKind, Scope};

use super::ScopeBuilder;

pub(super) fn create_package_header(
    self_: &mut ScopeBuilder<'_>,
    node: Node,
) -> anyhow::Result<Option<Scope>> {
    debug!("inserting package header scope");
    let package_header_node = PackageHeader::new(node, &self_.s_file.text);
    let Some(ident) = package_header_node.find_identifier() else {
        debug!("not inserting package header, as it has no package identifier");
        // Don't insert empty package
        return Ok(None);
    };
    let text = ident.text();

    Ok(Some(Scope::new(
        SKind::PackageHeader { ident: text },
        node.byte_range().try_into().unwrap(),
    )))
}

pub(super) fn update_package_header(
    self_: &mut ScopeBuilder<'_>,
    scope_node_id: &NodeId,
    _tree: &Tree,
    node: &Node,
) -> anyhow::Result<()> {
    debug!("Updating package header");
    if !node.kind_id() == *parser::node::PackageHeaderId {
        panic!("Node must be PackageHeader")
    }

    let mut cursor = node.walk();
    let mut package_header_text = "".to_string();

    if !cursor.goto_first_child() { // move to package
    }
    if !cursor.goto_next_sibling() { // no Identifier
    }
    if cursor.node().kind_id() == *parser::node::IdentifierId {
        package_header_text = parser::text_of(&cursor.node(), &self_.s_file.text);
    }

    let package_header_scope = self_
        .s_file
        .scopes
        .get_mut(*scope_node_id)
        .unwrap()
        .get_mut();
    let package_header = package_header_scope.kind.as_package_header_mut().unwrap();
    *package_header = package_header_text;
    debug!("new package header is {}", *package_header);

    Ok(())
}
