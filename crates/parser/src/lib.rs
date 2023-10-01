use std::collections::VecDeque;

use crop::Rope;
use tree_sitter::Node;

#[macro_use]
extern crate derive_new;

pub mod node;

pub fn text_of(node: &Node, rope: &Rope) -> String {
    return rope.byte_slice(node.byte_range()).to_string();
}

pub fn is_utf8_multibyte_char(byte: u8) -> bool {
    // in utf8 everything with 10xx'xxxx pattern is
    // not the first byte
    (byte & 0xc0) == 0x80
}

pub fn rec_descend<F>(node: &Node, mut f: F)
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
