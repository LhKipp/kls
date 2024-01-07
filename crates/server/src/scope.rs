use enum_as_inner::EnumAsInner;
use std::{collections::HashMap, fmt};
use tycheck::{TcKey, TyTable};

use atree::{Arena, Token};
// use theban_interval_tree::IntervalTree;
use tracing::trace;
use tree_sitter::{Node, Range};

use crate::{buffer::Buffer, scope_error::ScopeErrors, visit};

pub(crate) use crate::scopes::Scopes;

pub(crate) type ScopeTree = Arena<Scope>;
pub(crate) struct Scope {
    pub kind: SKind,
    pub ty_table: TyTable,
    pub items: HashMap<String /*item-name*/, SItem>,
}

// Used for debugging purposes
#[derive(Debug)]
pub(crate) enum SKind {
    Project { name: String },
    Module(String /*name*/),
    Class { name: String, range: Range },
    Function(String /*name*/),
    MemberFunction(String /*name*/),
}

#[derive(new, Debug)]
pub(crate) struct SItem {
    pub location: Range,
    pub item: SItemKind,
}

#[derive(EnumAsInner, Debug)]
pub(crate) enum SItemKind {
    SourceFileMetadata(SItemSourceFileMetadata),
    PackageHeader(String),
    Class(SItemClass),
    Var(SItemVar),
}

#[derive(Debug)]
pub(crate) struct SItemSourceFileMetadata {}

#[derive(Debug)]
pub(crate) struct SItemClass {
    pub name: String,
    pub tc_key: TcKey,
}

#[derive(Debug)]
pub(crate) struct SItemVar {
    pub name: String,
    pub tc_key: TcKey,
    pub mutable: bool,
    pub static_: bool,
}

impl Scope {
    pub fn new(kind: SKind) -> Scope {
        Scope {
            kind,
            items: HashMap::new(),
            ty_table: TyTable::new(),
        }
    }

    pub fn build_scopes_from(buffer: &Buffer) -> (ScopeTree, ScopeErrors) {
        ScopeBuilder::build_scopes_from(buffer, &buffer.tree.root_node())
    }

    pub fn fmt_debug(&mut self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        let ws_prefix = || f.write_str(&" ".repeat(depth));
        let e1 = ws_prefix();
        let e5 = f.write_str(&format!("Kind: {:?}\n", self.kind));
        let errs = self
            .items
            .iter()
            .map(|(_, item)| format!("Item: {:?}", item))
            .map(|item| ws_prefix().or(f.write_str(&item)))
            .reduce(|a, b| a.or(b))
            .unwrap_or(Ok(()));

        e1.or(e5).or(errs)
    }
}

impl fmt::Debug for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_debug(f, 0)
    }
}

pub(crate) struct ScopeBuilder<'a> {
    pub buffer: &'a Buffer,
    pub root: Option<Token>,
    pub all: ScopeTree,
    pub errors: ScopeErrors,
    pub current: Option<Token>,
}

impl<'a> ScopeBuilder<'a> {
    fn new(buffer: &'a Buffer) -> Self {
        ScopeBuilder {
            buffer,
            root: None,
            all: Arena::new(),
            current: None,
            errors: vec![],
        }
    }

    pub fn push_scope(&mut self, scope: Scope) -> Token {
        self.current = Some(self.current_mut().token().append(&mut self.all, scope));
        self.current.unwrap()
    }

    pub fn finish_scope(&mut self) {
        self.current = self.current.unwrap().ancestors_tokens(&self.all).next();
        assert!(self.current.is_some()); // when finishing, a parent scope must be present
    }

    pub fn current_ty_table(&mut self) -> &mut TyTable {
        &mut self.current_mut().data.ty_table
    }

    pub fn current_mut(&mut self) -> &mut atree::Node<Scope> {
        self.all.get_mut(self.current.unwrap()).unwrap()
    }

    pub fn current(&mut self) -> &atree::Node<Scope> {
        self.all.get(self.current.unwrap()).unwrap()
    }

    pub fn root(&self) -> Token {
        self.root.unwrap()
    }

    pub fn find_var(&mut self, var_name: &str) -> Option<&SItemVar> {
        self.current()
            .data
            .items
            .get(var_name)
            .unwrap()
            .item
            .as_var()
    }

    pub fn find_var_mut(&mut self, var_name: &str) -> Option<&SItemVar> {
        self.current_mut()
            .data
            .items
            .get(var_name)
            .unwrap()
            .item
            .as_var()
    }

    pub fn build_scopes_from(buffer: &Buffer, node: &Node) -> (ScopeTree, ScopeErrors) {
        trace!("Creating scopes from node {}", node.to_sexp());
        let mut scope_builder = ScopeBuilder::new(buffer);
        scope_builder.build_for(node);
        (scope_builder.all, scope_builder.errors)
    }

    pub fn build_for(&mut self, node: &Node) {
        match node.kind() {
            "source_file" => visit::visit_source_file(self, node),
            _ => panic!("not implement to start somewhere else than source_file"),
        }
    }

    pub fn visit(&mut self, node: &Node) {
        match node.kind() {
            "package_header" => visit::visit_package_header(self, node),
            "class_declaration" => visit::visit_class(self, node),
            _ => panic!("Not implemented visit for node kind {}", node.kind()),
        };
    }
}

// pub fn add_from_buffer_changes(
//     &self,
//     buffer: &Buffer,
//     edited_ranges: &ChangedRanges,
// ) -> Result<()> {
//     let mut w_lock = self.data.write();

//     // delete indexes for old ranges
//     for change in &edited_ranges.old_ranges {
//         self.delete_symbols(&mut w_lock, &change);
//     }
//     // Add indexes for new nodes
//     for change in &edited_ranges.new_ranges {
//         if let Some(new_node) = buffer
//             .tree
//             .root_node()
//             .descendant_for_byte_range(change.start as usize, change.end as usize)
//         {
//             // The changed node might not directly map to a node, symbols are build from
//             // To upate the node having the change, the ancestore mapping to a symbol must be
//             // rebuilt
//             if let Some(new_node) = find_ancestor_mapping_to_symbol(new_node) {
//                 self.add_from_node(&mut w_lock, &buffer, &new_node)?;
//             }
//         }
//     }
//     Ok(())
// }
