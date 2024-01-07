pub(crate) mod visit_class;
pub(crate) mod visit_package_header;
pub(crate) mod visit_source_file;

pub(crate) use visit_class::*;
pub(crate) use visit_package_header::*;
pub(crate) use visit_source_file::*;

pub(crate) use crate::scope::{SItem, SItemClass, SItemKind, SItemVar, SKind, Scope, ScopeBuilder};
pub(crate) use parser::node;
pub(crate) use tracing::trace;
#[allow(unused_imports)]
pub(crate) use tree_sitter::{Node, Range};
#[allow(unused_imports)]
pub(crate) use tycheck::{TcKey, TyTable};

use std::sync::atomic::AtomicI32;

pub(crate) static VISIT_UNIQUE_NUMBER_GENERATOR: AtomicI32 = AtomicI32::new(1);
