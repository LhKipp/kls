use std::path::PathBuf;

use crate::visit::*;

pub(crate) fn visit_source_file(
    builder: &mut ScopeBuilder,
    source_file_node: &Node,
    path: PathBuf,
) {
    let root_scope_token = builder
        .all
        // TODO module name
        .new_node(stdx::new_arc_rw_lock(Scope::new(SKind::Module {
            path,
            range: source_file_node.range(),
        })));
    builder.root = Some(root_scope_token);
    builder.current = builder.root;

    let mut cursor = source_file_node.walk();
    for child in source_file_node.children(&mut cursor) {
        builder.visit(&child);
    }
}
