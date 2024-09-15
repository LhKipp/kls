// #![feature(iter_intersperse)]
// #![feature(entry_insert)]
#![allow(dead_code)]
#![allow(unused_imports)]

use std::path::PathBuf;

use anyhow::{anyhow, bail, ensure};
use tower_lsp::lsp_types::Url;

#[macro_use]
extern crate derive_new;

pub mod kserver;
pub mod project;
pub mod request_handler;
mod scope;
pub mod range_util;

/// [Url::to_file_path] does not check, for the scheme, so we do manually
fn to_file_path(uri: &Url) -> anyhow::Result<PathBuf> {
    ensure!(uri.scheme() == "file", "Only file paths are supported");
    uri.to_file_path()
        .map_err(|_| anyhow!("Url::to_file_path failed"))
}
