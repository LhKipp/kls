use crate::scope::{
    fun_decl_scope::{Parameter, Type_},
    SFunDecl,
};
use anyhow::{bail, ensure};
use indextree::NodeId;
use parser::node::{FunctionDeclaration, FunctionValueParameters, UserType};
use std::{cell::RefCell, thread::panicking};
use tracing::{debug, trace};
use tree_sitter::{Node, Tree, TreeCursor};

use crate::scope::{SKind, Scope};

use super::ScopeBuilder;

pub(super) fn create_fun_decl(
    self_: &mut ScopeBuilder<'_>,
    node: Node,
) -> anyhow::Result<Option<Scope>> {
    debug!("creating function declaration");
    let fun_decl = FunctionDeclaration::new(node, &self_.s_file.text);
    let ident = fun_decl
        .find_simple_identifier()
        .map(|ident_node| ident_node.text());

    let parameters = fun_decl
        .find_function_value_parameters()
        .map_or(vec![], |params| get_parameters_of(params));

    let return_type = fun_decl.find_user_type().and_then(|n| get_type_of(n));

    Ok(Some(Scope::new(
        SKind::FunDecl(SFunDecl {
            ident,
            parameters,
            return_type,
        }),
        node.byte_range().try_into().unwrap(),
    )))
}

fn get_parameters_of(fun_value_parameters: FunctionValueParameters) -> Vec<Parameter> {
    return fun_value_parameters
        .find_all_parameter()
        .into_iter()
        .map(|parameter| {
            let ident = parameter.find_simple_identifier().map(|ident| ident.text());
            let type_ = parameter.find_user_type().and_then(get_type_of);
            Parameter { ident, type_ }
        })
        .collect::<Vec<_>>();
}

fn get_type_of(user_type: UserType) -> Option<Type_> {
    user_type.find_type_identifier().map(|t| {
        let t = t.text();
        if t == "Unit" {
            Type_::Unit
        } else {
            Type_::Simple(t)
        }
    })
}

pub(crate) fn update_function_declaration<'a>(
    self_: &mut ScopeBuilder<'a>,
    scope_node_id: NodeId,
    _tree: &Tree,
    cursor: &mut TreeCursor,
    _upsert_range: stdx::TextRange,
) -> anyhow::Result<()> {
    debug!(
        "updating function declaration. Cursor is at {}",
        cursor.node().kind()
    );
    move_cursor_to_mapable_node(cursor)?;

    let scope = self_
        .s_file
        .scopes
        .get_mut(scope_node_id)
        .unwrap()
        .get_mut();
    let scope_func_decl = scope.kind.as_fun_decl_mut().unwrap();

    if is_function_name_node(&cursor.node()) {
        let new_name = parser::text_of(&cursor.node(), &self_.s_file.text);
        debug!("Updated function name to {}", new_name);
        scope_func_decl.ident = Some(new_name);
    } else if cursor.node().kind_id() == *parser::node::FunctionValueParametersId {
        let new_params = get_parameters_of(FunctionValueParameters::new(
            cursor.node(),
            &self_.s_file.text,
        ));
        diff_and_assign_new_params(&mut scope_func_decl.parameters, new_params)?;
    } else if cursor.node().kind_id() == *parser::node::UserTypeId {
        let new_return_type = UserType::new(cursor.node(), &self_.s_file.text);
        scope_func_decl.return_type = get_type_of(new_return_type);
    }

    Ok(())
}

fn diff_and_assign_new_params(
    old_params: &mut Vec<Parameter>,
    new_params: Vec<Parameter>,
) -> anyhow::Result<()> {
    trace!("Updating params {:?} with {:?}", old_params, new_params);

    let mut i = 0;
    while i < old_params.len() {
        if i >= new_params.len() {
            // params got deleted after including i
            old_params.truncate(i);
            break;
        }
        let new_param = &new_params[i];
        let param = &old_params[i];

        if param.eq_no_ty(new_param) {
            // nothing changed. compare next parameters
        } else {
            // insert new_param. Diff `param` with next new_param
            old_params.insert(i, new_param.clone());
        }

        i += 1;
    }

    trace!("Updated params: {:?}", old_params);
    Ok(())
}

fn move_cursor_to_mapable_node(cursor: &mut TreeCursor) -> anyhow::Result<()> {
    loop {
        if is_function_name_node(&cursor.node()) {
            // update the function name
            return Ok(());
        } else if cursor.node().kind_id() == *parser::node::FunctionValueParametersId {
            // update the parameter
            return Ok(());
        } else if cursor.node().kind_id() == *parser::node::UserTypeId {
            // update the return type
            return Ok(());
        }

        ensure!(
            cursor.node().kind_id() != *parser::node::SourceFileId,
            "Don't know how to update the function"
        );

        cursor.goto_parent();
    }
}

fn is_function_name_node(node: &Node) -> bool {
    node.kind_id() == *parser::node::SimpleIdentifierId
        && node
            .parent()
            .is_some_and(|parent| parent.kind_id() == *parser::node::FunctionDeclarationId)
}
