extern "C" {
    fn tree_sitter_kotlin() -> Language;
}
use std::path::Path;

use tree_sitter::{Language, Parser, Tree};

pub(crate) fn parse_file(path: &Path) -> Option<Tree> {
    let txt = std::fs::read_to_string(path).unwrap();
    parse(&txt)
}
pub(crate) fn parse(text: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let language = unsafe { tree_sitter_kotlin() };
    parser.set_language(language).unwrap();
    parser.parse(text, None)
}