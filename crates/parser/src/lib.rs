#![allow(unused)]
#![allow(dead_code)]
use std::collections::VecDeque;

use crop::Rope;
use tree_sitter::Node;

#[macro_use]
extern crate derive_new;

pub mod node;

extern "C" {
    fn tree_sitter_kotlin() -> Language;
}
use std::path::Path;

use lazy_static::lazy_static;
use tree_sitter::{Language, Parser, Tree};

lazy_static! {
    static ref LANGUAGE: Language = unsafe { tree_sitter_kotlin() };
}

pub fn parse_file(path: &Path) -> (Option<Tree>, String) {
    // TODO possibly parse a rope instead of read to string and then convert to rope?
    let txt = std::fs::read_to_string(path).unwrap();
    parse(txt)
}
pub fn parse(text: String) -> (Option<Tree>, String) {
    let mut parser = Parser::new();
    parser.set_language(*LANGUAGE).unwrap();
    let tree = parser.parse(&text, None);

    (tree, text)
}
pub fn reparse(text: &str, old_tree: &Tree) -> Option<Tree> {
    let mut parser = Parser::new();
    parser.set_language(*LANGUAGE).unwrap();
    parser.parse(&text, Some(old_tree))
}

pub fn text_of(node: &Node, rope: &Rope) -> String {
    return rope.byte_slice(node.byte_range()).to_string();
}

pub fn is_utf8_multibyte_char(byte: u8) -> bool {
    // in utf8 everything with 10xx'xxxx pattern is
    // not the first byte
    (byte & 0xc0) == 0x80
}

pub fn bfs_descend<F>(node: &Node, mut f: F)
where
    F: FnMut(&Node) -> bool,
{
    let mut visit = |node: &Node| -> bool { f(node) };

    let mut queue = VecDeque::new();
    queue.push_back(node.clone());

    let mut cursor = node.walk();
    while let Some(node) = queue.pop_front() {
        if !visit(&node) {
            continue;
        }

        for child in node.children(&mut cursor) {
            queue.push_back(child);
        }
    }
}
