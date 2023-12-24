use std::collections::HashSet;

use crop::Rope;
use lazy_static::lazy_static;
use parser::{bfs_descend, node};
use std::path::{Path, PathBuf};
use std::sync::{atomic::AtomicI32, atomic::Ordering};
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};
use tracing::trace;
use tree_sitter::{Node, Range};

lazy_static! {
    pub static ref NODES_CONTAINING_SYMBOLS: HashSet<&'static str> =
        HashSet::from(["source_file", "package_header", "class_declaration"]);
}

static SYMBOL_ID_GENERATOR: AtomicI32 = AtomicI32::new(1);
#[derive(new, Debug, Clone)]
pub struct Symbol {
    #[new(value = "SYMBOL_ID_GENERATOR.fetch_add(1, Ordering::SeqCst)")]
    pub id: i32,
    pub file: PathBuf,
    pub package: String,
    pub location: Range,
    pub name: String,
    pub kind: SymbolKind,
    pub duplicate_behind_package: bool,
}

impl Symbol {
    pub fn name_behind_package(&self) -> Option<Vec<u8>> {
        if self.duplicate_behind_package {
            Some(format!("{}.{}", self.package, self.name).into())
        } else {
            None
        }
    }
    pub fn clone_as_behind_package(&self) -> Symbol {
        assert!(self.duplicate_behind_package);
        Symbol {
            id: self.id,
            file: self.file.clone(),
            package: self.package.clone(),
            location: self.location,
            name: format!("{}.{}", self.package, self.name),
            kind: self.kind.clone(),
            duplicate_behind_package: self.duplicate_behind_package,
        }
    }
    pub fn to_completion_item(&self) -> CompletionItem {
        let mut item = CompletionItem::default();
        match &self.kind {
            SymbolKind::Class(_) => {
                item.label = self.name.clone();
                item.kind = Some(CompletionItemKind::CLASS);
            }
            SymbolKind::Package => {
                item.label = self.name.clone();
                item.kind = Some(CompletionItemKind::MODULE)
            }
            SymbolKind::Enum(_) => {
                item.label = self.name.clone();
                item.kind = Some(CompletionItemKind::ENUM)
            }
        }
        // trace!("Mapped {:?} to {:?}", symbol, item);
        item
    }
}

#[derive(Debug, Clone)]
pub struct SymbolClass {
    pub name: String,
}
#[derive(Debug, Clone)]
pub struct SymbolEnum {
    pub name: String,
    pub entries: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Class(SymbolClass),
    Enum(SymbolEnum),
    Package,
}

struct PackageInfo {
    name: String,
    range: Range,
}

pub struct SymbolBuilder<'a> {
    file: &'a Path,
    source: &'a Rope,
    package: Option<PackageInfo>,

    symbols: Vec<Symbol>,
    errors: Vec<String>, // TODO better error
}

impl<'a> SymbolBuilder<'a> {
    pub fn new(file: &'a Path, source: &'a Rope) -> Self {
        SymbolBuilder {
            file,
            source,
            package: None,

            symbols: vec![],
            errors: vec![],
        }
    }

    pub fn finish(self) -> (Vec<Symbol>, Vec<String>) {
        (self.symbols, self.errors)
    }

    pub fn build_all_symbols_for(&mut self, root_node: Node) {
        bfs_descend(&root_node, |node: &Node| match node.kind() {
            "package_header" => self.symbols_of_package(node),
            "class_declaration" => self.symbols_of_class(node),
            _ => true,
        });
    }

    fn symbols_of_package(&mut self, package_node: &Node) -> bool {
        let package_decl = node::PackageHeader::new(package_node.clone(), &self.source);
        trace!("Visiting package_header {}", package_decl.text());

        if let Some(prev_package_decl) = &self.package {
            self.errors.push(format!(
                "Duplicate package declaration at {:?}. Previous package declaration found at {:?}",
                package_node.range(),
                prev_package_decl.range
            ));
            return false;
        }

        let mut package_info = PackageInfo {
            name: String::new(),
            range: package_node.range(),
        };
        if let Some(identifier) = package_decl.find_identifier() {
            package_info.name = identifier.text();

            self.add_symbol(
                package_info.range,
                package_info.name.clone(),
                SymbolKind::Package,
                false,
            )
        }
        self.package = Some(package_info);
        false
    }

    fn symbols_of_class(&mut self, class_node: &Node) -> bool {
        let class_decl = node::ClassDeclaration::new(class_node.clone(), &self.source);
        trace!(
            "Visiting class with name {:?}",
            class_decl.find_type_identifier()
        );

        if let Some(class_name) = class_decl.find_type_identifier() {
            if let Some(enum_body) = class_decl.find_enum_class_body() {
                self.symbols_of_enum_class(&class_decl, enum_body, class_name.text())
            } else {
                // normal class
                self.add_symbol(
                    class_decl.node.range(),
                    class_name.text(),
                    SymbolKind::Class(SymbolClass {
                        name: class_name.text(),
                    }),
                    true,
                );
            }
        }
        true
    }

    fn symbols_of_enum_class(
        &mut self,
        class_decl: &node::ClassDeclaration,
        enum_body: node::EnumClassBody,
        class_name: String,
    ) {
        let entries = enum_body
            .find_all_enum_entry()
            .iter()
            .map(|entry| {
                entry
                    .find_simple_identifier()
                    .expect("Not handling complex enum entries for now")
                    .text()
            })
            .collect::<Vec<_>>();

        self.add_symbol(
            class_decl.node.range(),
            class_name.clone(),
            SymbolKind::Enum(SymbolEnum {
                name: class_name,
                entries,
            }),
            true,
        );
    }

    fn add_symbol(
        &mut self,
        location: Range,
        symbol_name: String,
        symbol_kind: SymbolKind,
        searchable_behind_package: bool,
    ) {
        trace!("Adding symbol {} as {:?}", symbol_name, symbol_kind);

        let package = self
            .package
            .as_ref()
            .map(|p| p.name.as_str())
            .unwrap_or("")
            .to_string();
        let duplicate_behind_package = !package.is_empty() && searchable_behind_package;

        self.symbols.push(Symbol::new(
            self.file.to_path_buf(),
            package,
            location,
            symbol_name,
            symbol_kind,
            duplicate_behind_package,
        ))
    }
}
