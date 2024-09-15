#![allow(unused)]
#![allow(dead_code)]
use std::collections::VecDeque;

use crop::Rope;
use tracing::{debug, info};
use tree_sitter::{Node, Point};

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
    pub static ref SourceFileId: u16 = LANGUAGE.id_for_node_kind("source_file", true);
}

pub fn parse(rope: &Rope, old_tree: Option<&Tree>) -> Option<Tree> {
    // LANGUAGE.id_for_node_kind(kind, named)
    let mut parser = Parser::new();
    parser.set_language(*LANGUAGE).unwrap();
    let empty_bytes: &[u8] = &[];
    let newline_bytes: &[u8] = &[b'\n'];
    let mut all_text = String::new();
    let tree = parser.parse_with(
        &mut |_byte: usize, position: Point| -> &[u8] {
            debug!(
                "Got byte pos {}, and row/col pos {}/{}",
                _byte, position.row, position.column
            );

            if (_byte >= rope.byte_len()) {
                debug!("returning empty bytes");
                return empty_bytes;
            }
            // The rope does not store newlines. So
            // if (rope.byte(_byte) == b'\n') {
            //     info!("Returning newline");
            //     all_text += "\n";
            //     return newline_bytes;
            // };
            //
            // let line = rope.line(position.row);
            // let max_len = line.byte_len();
            // let text = line
            //     .byte_slice(position.column..max_len)
            //     .chunks()
            //     .next()
            //     .expect("no more text");
            // info!("returning text {}", text);
            // all_text += text;
            // return text.as_ref();
            // return line.chunks().next().expect("No more chunks").as_bytes();

            match rope.byte_slice(_byte..).chunks().next() {
                Some(text) => {
                    debug!("Returning text to parse {}", text);
                    all_text += text;
                    text.as_bytes()
                }
                None => {
                    debug!("returning empty slice");
                    empty_bytes
                }
            }
            //     rope.line(position.row).bytes()
            // let row = position.row as usize;
            // let column = position.column as usize;
            // if row < lines.len() {
            //     if column < lines[row].as_bytes().len() {
            //         &lines[row].as_bytes()[column..]
            //     } else {
            //         b"\n"
            //     }
            // } else {
            //     &[]
            // }
        },
        old_tree,
    );
    debug!("Parsed text is {}", all_text);
    debug!("tree is {}", tree.clone().unwrap().root_node().to_sexp());
    tree
}

pub fn text_of(node: &Node<'_>, rope: &Rope) -> String {
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
    queue.push_back(*node);

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

#[test]
fn parse_generates_tree_with_correct_offsets() {
    let text = r#"
package com.example
class A()
fun myfunc() { 
    val x = 3
}
"#;
    let result = parse(&Rope::from(text), None).expect("Parse should return a tree");

    let mut text = String::new();
    bfs_descend(&result.root_node(), |n| {
        text += &format!("{:#?}\n", n);
        true
    });

    let expected = r#"{Node source_file (1, 0) - (6, 0)}
{Node package_header (1, 0) - (1, 19)}
{Node class_declaration (2, 0) - (2, 9)}
{Node function_declaration (3, 0) - (5, 1)}
{Node package (1, 0) - (1, 7)}
{Node identifier (1, 8) - (1, 19)}
{Node class (2, 0) - (2, 5)}
{Node type_identifier (2, 6) - (2, 7)}
{Node primary_constructor (2, 7) - (2, 9)}
{Node fun (3, 0) - (3, 3)}
{Node simple_identifier (3, 4) - (3, 10)}
{Node function_value_parameters (3, 10) - (3, 12)}
{Node function_body (3, 13) - (5, 1)}
{Node simple_identifier (1, 8) - (1, 11)}
{Node . (1, 11) - (1, 12)}
{Node simple_identifier (1, 12) - (1, 19)}
{Node ( (2, 7) - (2, 8)}
{Node ) (2, 8) - (2, 9)}
{Node ( (3, 10) - (3, 11)}
{Node ) (3, 11) - (3, 12)}
{Node { (3, 13) - (3, 14)}
{Node statements (4, 4) - (4, 13)}
{Node } (5, 0) - (5, 1)}
{Node property_declaration (4, 4) - (4, 13)}
{Node binding_pattern_kind (4, 4) - (4, 7)}
{Node variable_declaration (4, 8) - (4, 9)}
{Node = (4, 10) - (4, 11)}
{Node integer_literal (4, 12) - (4, 13)}
{Node val (4, 4) - (4, 7)}
{Node simple_identifier (4, 8) - (4, 9)}
"#;
    assert_eq!(text, expected);
}
