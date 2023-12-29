use anyhow::Result;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::atomic::{AtomicI32, Ordering},
};
use tycheck::{TcKey, TyTable};

use atree::{Arena, Token};
use parser::node;
use theban_interval_tree::IntervalTree;
use tracing::{error, trace};
use tree_sitter::{Node, Range};

use crate::buffer::Buffer;

static UNIQUE_NUMBER_GENERATOR: AtomicI32 = AtomicI32::new(1);

struct Scope {
    pub kind: SKind,
    pub range: Range,
    pub ty_table: TyTable,
    pub items: HashMap<String /*item-name*/, SItem>,
}

// Used for debugging purposes
enum SKind {
    Project,
    Module,
    Class(String /*name*/),
    Function(String /*name*/),
    MemberFunction(String /*name*/),
}

#[derive(new)]
struct SItem {
    pub location: Range,
    pub item: SItemKind,
}

enum SItemKind {
    SourceFileMetadata(SItemSourceFileMetadata),
    PackageHeader(String),
    Class(SItemClass),
    Var(SItemVar),
}

struct SItemSourceFileMetadata {}

struct SItemClass {
    pub name: String,
    pub tc_key: TcKey,
}

struct SItemVar {
    pub name: String,
    pub tc_key: TcKey,
    pub mutable: bool,
}

impl Scope {
    pub fn new(kind: SKind, range: Range) -> Scope {
        Scope {
            kind,
            range,
            items: HashMap::new(),
            ty_table: TyTable::new(),
        }
    }
}

impl Scopes {
    pub fn new() -> Self {
        Self {
            scopes: Arena::new(),
            scopes_of_buffer: HashMap::new(),
        }
    }

    pub fn build_scopes_from(&mut self, buffer: &Buffer) -> Result<()> {
        let (_, errors) = ScopeBuilder::build_scopes_from(buffer, &buffer.tree.root_node());
        if !errors.is_empty() {
            error!(
                "Errors building symbols from file {}: {}",
                buffer.path.display(),
                errors.join(",")
            );
        }

        // for symbol in symbols {
        //     self.insert_symbol(w_lock, symbol);
        // }

        Ok(())
    }
}

struct ScopeBuilder<'a> {
    pub buffer: &'a Buffer,
    pub root: Option<Token>,
    pub all: Arena<Scope>,
    pub errors: Vec<String>,
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

    fn push_scope(&mut self, scope: Scope) -> Token {
        self.current = Some(self.current().token().append(&mut self.all, scope));
        self.current.unwrap()
    }

    fn finish_scope(&mut self) {
        self.current = self.current.unwrap().ancestors_tokens(&self.all).next();
        assert!(self.current.is_some()); // when finishing, a parent scope must be present
    }

    fn current_ty_table(&mut self) -> &mut TyTable {
        &mut self.current().data.ty_table
    }

    fn current(&mut self) -> &mut atree::Node<Scope> {
        self.all.get_mut(self.current.unwrap()).unwrap()
    }

    fn root(&self) -> Token {
        self.root.unwrap()
    }

    pub fn build_scopes_from(buffer: &Buffer, node: &Node) -> (Arena<Scope>, Vec<String>) {
        trace!("Creating scopes from node {}", node.to_sexp());
        let mut scope_builder = ScopeBuilder::new(buffer);
        scope_builder.build_for(node);
        (scope_builder.all, scope_builder.errors)
    }

    pub fn build_for(&mut self, node: &Node) {
        match node.kind() {
            "source_file" => self.visit_source_file(node),
            _ => panic!("not implement to start somewhere else than source_file"),
        }
    }

    fn visit(&mut self, node: &Node) {
        match node.kind() {
            "package_header" => self.visit_package_header(node),
            "class_declaration" => self.visit_class(node),
            _ => panic!("Not implemented visit for node kind {}", node.kind()),
        };
    }

    fn visit_source_file(&mut self, source_file_node: &Node) {
        let root_scope_token = self
            .all
            .new_node(Scope::new(SKind::Module, source_file_node.range()));
        self.root = Some(root_scope_token);
        self.current = self.root;

        let mut cursor = source_file_node.walk();
        for child in source_file_node.children(&mut cursor) {
            self.visit(&child);
        }
    }

    fn visit_package_header(&mut self, package_node: &Node) {
        let package_decl = node::PackageHeader::new(package_node.clone(), &self.buffer.text);
        trace!(
            "Visiting package_header {:?}",
            package_decl.find_identifier()
        );

        let package_ident = if let Some(package_ident) = package_decl.find_identifier() {
            package_ident.text()
        } else {
            self.errors
                .push("Package declaration missing package name".into());
            String::new()
        };

        self.current().data.items.insert(
            package_ident.clone(),
            SItem::new(
                package_node.range(),
                SItemKind::PackageHeader(package_ident),
            ),
        );
    }

    fn visit_class(&mut self, class_node: &Node) {
        let class_decl = node::ClassDeclaration::new(class_node.clone(), &self.buffer.text);
        trace!(
            "Visiting class with name {:?}",
            class_decl.find_type_identifier()
        );

        let (class_name, name_range) = if let Some(class_name) = class_decl.find_type_identifier() {
            (class_name.text(), class_name.node.range())
        } else {
            self.errors.push("Class is missing a name".into());
            (
                format!(
                    "___{}",
                    UNIQUE_NUMBER_GENERATOR.fetch_add(1, Ordering::SeqCst),
                ),
                class_decl.node.range(),
            )
        };

        let cur_scope = &mut self.current().data;

        cur_scope.items.insert(
            class_name.clone(),
            SItem::new(
                name_range,
                SItemKind::Class(SItemClass {
                    name: class_name.clone(),
                    tc_key: cur_scope.ty_table.new_key(),
                }),
            ),
        );

        self.push_scope(Scope::new(
            SKind::Class(class_name),
            class_decl.node.range(),
        ));

        self.visit_class_definition(&class_decl);

        self.finish_scope();
    }

    fn visit_class_definition(&mut self, node: &node::ClassDeclaration) {
        if let Some(primary_ctor) = node.find_primary_constructor() {
            for parameter in primary_ctor.find_all_class_parameter() {
                let Some(simple_identifier) = parameter.find_simple_identifier() else {
                    continue;
                };
                let tc_key = self.current_ty_table().new_key();
                self.current().data.items.insert(
                    simple_identifier.text(),
                    SItem::new(
                        simple_identifier.node.range(),
                        SItemKind::Var(
                            // TODO mutable
                            SItemVar {
                                name: simple_identifier.text(),
                                tc_key,
                                mutable: false,
                            },
                        ),
                    ),
                );
            }
        }
    }

    fn visit_enum_class(
        &mut self,
        _class_decl: &node::ClassDeclaration,
        _enum_body: node::EnumClassBody,
        _class_name: String,
    ) {
        // let entries = enum_body
        //     .entries()
        //     .iter()
        //     .map(|entry| {
        //         entry
        //             .name()
        //             .expect("Not handling complex enum entries for now")
        //     })
        //     .collect::<Vec<_>>();

        // self.add_symbol(
        //     class_decl.node.range(),
        //     class_name.clone(),
        //     SymbolKind::Enum(SymbolEnum {
        //         name: class_name,
        //         entries,
        //     }),
        //     true,
        // );
    }
}

impl Scope {}
