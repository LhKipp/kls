use std::sync::atomic::Ordering;

use crate::visit::*;

pub(crate) fn visit_class(builder: &mut ScopeBuilder, class_node: &Node) {
    let class_decl = node::ClassDeclaration::new(class_node.clone(), &builder.buffer.text);
    trace!(
        "Visiting class with name {:?}",
        class_decl.find_type_identifier()
    );

    let (class_name, name_range) = if let Some(class_name) = class_decl.find_type_identifier() {
        (class_name.text(), class_name.node.range())
    } else {
        builder.errors.push("Class is missing a name".into());
        (
            format!(
                "___{}",
                VISIT_UNIQUE_NUMBER_GENERATOR.fetch_add(1, Ordering::SeqCst),
            ),
            class_decl.node.range(),
        )
    };

    let cur_scope = &mut builder.current_mut().data;
    let class_tc_key = cur_scope.ty_table.new_key();

    cur_scope.items.insert(
        class_name.clone(),
        SItem::new(
            name_range,
            SItemKind::Class(SItemClass {
                name: class_name.clone(),
                tc_key: class_tc_key,
            }),
        ),
    );

    builder.push_scope(Scope::new(
        SKind::Class(class_name),
        class_decl.node.range(),
    ));
    builder.current_mut().data.items.insert(
        "this".into(),
        SItem::new(
            name_range,
            SItemKind::Var(SItemVar {
                name: "this".into(),
                tc_key: class_tc_key,
                mutable: false,
                static_: false,
            }),
        ),
    );

    visit_class_definition(builder, &class_decl);

    builder.finish_scope();
}

pub(crate) fn visit_class_definition(builder: &mut ScopeBuilder, node: &node::ClassDeclaration) {
    if let Some(primary_ctor) = node.find_primary_constructor() {
        for parameter in primary_ctor.find_all_class_parameter() {
            let Some(simple_identifier) = parameter.find_simple_identifier() else {
                continue;
            };
            let tc_key = builder.current_ty_table().new_key();
            builder.current_mut().data.items.insert(
                simple_identifier.text(),
                SItem::new(
                    simple_identifier.node.range(),
                    SItemKind::Var(
                        // TODO mutable
                        SItemVar {
                            name: simple_identifier.text(),
                            tc_key,
                            mutable: false,
                            static_: false,
                        },
                    ),
                ),
            );
        }
    }
    if let Some(enum_class_body) = node.find_enum_class_body() {
        visit_enum_class(builder, node, enum_class_body);
    }
}

pub(crate) fn visit_enum_class(
    builder: &mut ScopeBuilder,
    _class_decl: &node::ClassDeclaration,
    enum_body: node::EnumClassBody,
) {
    let class_tc_key = builder.find_var("this").unwrap().tc_key;
    for enum_entry in enum_body.find_all_enum_entry() {
        if let Some(enum_entry_name) = enum_entry.find_simple_identifier() {
            builder.current_mut().data.items.insert(
                enum_entry_name.text(),
                SItem::new(
                    enum_entry_name.node.range(),
                    SItemKind::Var(
                        // TODO mutable
                        SItemVar {
                            name: enum_entry_name.text(),
                            tc_key: class_tc_key,
                            mutable: false,
                            static_: true,
                        },
                    ),
                ),
            );
        }
    }
}
