use crop::Rope;
use parking_lot::lock_api::RwLockWriteGuard;
use parking_lot::RwLock;
use std::{collections::HashMap, path::PathBuf};
use tower_lsp::jsonrpc::Result;
use tower_lsp::{
    jsonrpc::Error,
    lsp_types::{Position, Url},
};

pub struct Buffers {
    pub buffers: RwLock<HashMap<PathBuf, Buffer>>,
}

impl Buffers {
    pub fn new() -> Self {
        Buffers {
            buffers: RwLock::new(HashMap::new()),
        }
    }

    pub fn read<F, R>(&self, uri: &Url, mut f: F) -> Result<R>
    where
        F: FnMut(&Buffer) -> Result<R>,
    {
        let path = uri.to_file_path().unwrap();
        if let Some(buffer) = self.buffers.read().get(&path) {
            return f(buffer);
        }

        Err(Error::invalid_params(format!("No such buffer {}", uri)))
    }

    pub async fn add_from_file<F>(&self, path: PathBuf, mut and_then: F)
    where
        F: FnMut(&Buffer),
    {
        let (tree, source) = crate::parse_kotlin::parse_file(&path);
        let tree = tree.unwrap();

        let buffer = Buffer {
            path: path.clone(),
            text: source.into(),
            tree,
        };

        let mut w_lock = self.buffers.write();
        w_lock.entry(path.clone()).insert_entry(buffer).get();

        let r_lock = RwLockWriteGuard::downgrade(w_lock);
        and_then(r_lock.get(&path).unwrap());
    }
}

pub struct Buffer {
    pub path: PathBuf,
    pub tree: tree_sitter::Tree,
    pub text: Rope,
}

impl Buffer {
    pub fn text_at(&self, position: Position) -> Result<String> {
        // by position.character the protocol means the number of bytes
        let word: String = self
            .text
            .line(position.line as usize)
            .byte_slice((0 as usize)..(position.character as usize))
            // in utf8 a multibyte character later bytes can never equal b' '
            // those bytes have the pattern 10xx'xxxx
            .chars()
            .rev()
            .take_while(|byte| byte.clone() != ' ')
            .collect();
        let word = word.chars().rev().collect();

        Ok(word)
    }
}
