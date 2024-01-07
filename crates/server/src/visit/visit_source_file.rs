use crate::visit::*;

pub(crate) fn visit_source_file(builder: &mut ScopeBuilder, source_file_node: &Node) {
    let root_scope_token = builder
        .all
        .new_node(Scope::new(SKind::Module, source_file_node.range()));
    builder.root = Some(root_scope_token);
    builder.current = builder.root;

    let mut cursor = source_file_node.walk();
    for child in source_file_node.children(&mut cursor) {
        builder.visit(&child);
    }
}
