extern "C" {
    fn tree_sitter_kotlin() -> Language;
}
use std::path::Path;

use lazy_static::lazy_static;
use tree_sitter::{Language, Parser, Tree};

lazy_static! {
    static ref LANGUAGE: Language = unsafe { tree_sitter_kotlin() };
}

pub(crate) fn parse_file(path: &Path) -> (Option<Tree>, String) {
    // TODO possibly parse a rope instead of read to string and then convert to rope?
    let txt = std::fs::read_to_string(path).unwrap();
    parse(txt)
}
pub(crate) fn parse(text: String) -> (Option<Tree>, String) {
    let mut parser = Parser::new();
    parser.set_language(*LANGUAGE).unwrap();
    let tree = parser.parse(&text, None);

    (tree, text)
}
pub(crate) fn reparse(text: &str, old_tree: &Tree) -> Option<Tree> {
    let mut parser = Parser::new();
    parser.set_language(*LANGUAGE).unwrap();
    parser.parse(&text, Some(old_tree))
}
