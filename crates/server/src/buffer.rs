use core::fmt;
use crop::Rope;
use parking_lot::lock_api::RwLockWriteGuard;
use parking_lot::RwLock;
use std::{collections::HashMap, path::PathBuf};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::TextDocumentContentChangeEvent;
use tower_lsp::{
    jsonrpc::Error,
    lsp_types::{Position, Url},
};
use tree_sitter::{InputEdit, Point};

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

    pub fn edit(
        &self,
        uri: &Url,
        changes: &[TextDocumentContentChangeEvent],
    ) -> Result<Vec<tree_sitter::Range>> {
        let mut w_lock = self.buffers.write();
        if let Some(buffer) = w_lock.get_mut(&uri.to_file_path().unwrap()) {
            return Ok(buffer.edit(changes));
        }
        Err(Error::invalid_params(format!("No such buffer: {}", uri)))
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

    pub fn edit(&mut self, changes: &[TextDocumentContentChangeEvent]) -> Vec<tree_sitter::Range> {
        for change in changes {
            if let Some(range) = &change.range {
                let old_byte_range = self.to_byte_range(range);
                let new_byte_range =
                    old_byte_range.start..(old_byte_range.start + change.text.len());
                self.text.replace(old_byte_range.clone(), &change.text);
                // if range.start == range.end {
                //     // insert at point
                //     trace!("Text before change: |{}|", self.text);
                //     self.text.insert(old_byte_range.start, &change.text);
                //     trace!("Text after change: |{}|", self.text);
                // } else {
                //     // replacement or deletion
                // }
                let new_end_point = self.point_of(new_byte_range.end);

                self.tree.edit(&InputEdit {
                    start_byte: new_byte_range.start,
                    old_end_byte: old_byte_range.end,
                    new_end_byte: new_byte_range.end,
                    start_position: Point::new(
                        range.start.line as usize,
                        range.start.character as usize,
                    ),
                    old_end_position: Point::new(
                        range.end.line as usize,
                        range.end.character as usize,
                    ),
                    new_end_position: new_end_point,
                });
            }
        }

        // TODO parse rope
        let new_tree = crate::parse_kotlin::reparse(&self.text.to_string(), &self.tree)
            .expect("Not handling no tree yet");

        let changed_ranges = self.tree.changed_ranges(&new_tree);
        self.tree = new_tree;
        changed_ranges.collect::<Vec<_>>()
    }

    // fn to_capped_byte_range(
    //     &self,
    //     range: &tower_lsp::lsp_types::Range,
    //     new_text_len: usize,
    // ) -> std::ops::Range<usize> {
    //     if self.text.line_len() == 0 {
    //         return 0..0;
    //     }

    //     let start_line = min(self.text.line_len(), range.start.line as usize);
    //     let end_line = min(self.text.line_len(), (range.end.line + 1) as usize);

    //     let lines = self.text.line_slice(start_line..end_line);
    //     let end_line_ = lines.line(lines.line_len() - 1);

    //     // let range_byte_count = lines.byte_len()
    //     //     + 1 // + 1 ==> commented out -1
    //     //     - (range.start.character as usize /*-1*/) // -1 to make start inclusive
    //     //     + (- (end_line_.byte_len() as isize - range.end.character as isize)) as usize;

    //     let mut start_byte = match start_line {
    //         0 => 0,
    //         1.. => self.text.line_slice(0..start_line).byte_len(),
    //         _ => unreachable!(),
    //     } + range.start.character as usize;

    //     start_byte = min(self.text.byte_len(), start_byte);
    //     let end_byte = min(self.text.byte_len(), start_byte + new_text_len);

    //     start_byte..(end_byte) // don't know why -1
    // }

    fn to_byte_range(&self, range: &tower_lsp::lsp_types::Range) -> std::ops::Range<usize> {
        (self.text.byte_of_line(range.start.line as usize) + range.start.character as usize)
            ..(self.text.byte_of_line(range.end.line as usize) + range.end.character as usize)
    }

    fn point_of(&self, byte_offset: usize) -> Point {
        let row = self.text.line_of_byte(byte_offset);
        let col = byte_offset - self.text.byte_of_line(row);

        Point::new(row, col)
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Buffer")
            .field("path", &self.path)
            .field("tree", &self.tree)
            .field("text", &self.text)
            .finish()
    }
}
