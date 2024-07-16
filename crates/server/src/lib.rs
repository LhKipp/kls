// #![feature(iter_intersperse)]
// #![feature(entry_insert)]
#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate derive_new;

pub mod buffer;
mod completion;
mod error_util;
pub mod kserver;
pub mod project;
pub mod range_util;
pub mod scope;
pub mod scope_error;
pub mod scopes;
pub mod visit;
