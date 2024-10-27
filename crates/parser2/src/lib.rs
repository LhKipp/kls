#![feature(get_mut_unchecked)]
#![allow(unused)]
#![allow(dead_code)]

#[macro_use]
extern crate derive_new;

mod ast_util;
mod grammar;
mod lexer;
mod marker;
mod node;
mod parse_event;
mod parser;
mod parser_api;
mod tree_builder;
mod node_helper;

pub use lexer::Token;
pub use node::{Node, RcNode};

pub use parser::ChangedRange;
pub use parser_api::{parse_no_state, parse_with_state};
