use core::fmt;
use crop::Rope;
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use std::{collections::HashMap, path::PathBuf};
use stdx::ARwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::TextDocumentContentChangeEvent;
use tower_lsp::{
    jsonrpc::Error,
    lsp_types::{Position, Url},
};
use tracing::{trace, warn, Level};
use tree_sitter::{InputEdit, Node, Point};

use crate::range_util::{self, byte_range_from_usize_range, ChangedRanges, TextByteRange};

pub struct Buffers {
    pub buffers: ARwLock<HashMap<PathBuf, ARwLock<Buffer>>>,
}

impl<'a> Buffers {
    pub fn new() -> Self {
        Buffers {
            buffers: stdx::new_arc_rw_lock(HashMap::new()),
        }
    }

    pub fn shallow_clone(&self) -> Buffers {
        Buffers {
            buffers: self.buffers.clone(),
        }
    }

    pub fn at(&'a self, uri: &Url) -> Result<ARwLock<Buffer>> {
        let path = uri.to_file_path().unwrap();
        let r_lock = self.buffers.read();
        if let Some(buffer) = r_lock.get(&path).cloned() {
            return Ok(buffer);
        }

        Err(Error::invalid_params(format!("No such buffer {}", uri)))
    }

    pub fn add(&self, buffer: Buffer) {
        let old_val = self
            .buffers
            .write()
            .insert(buffer.path.clone(), stdx::new_arc_rw_lock(buffer));
        if let Some(old_val) = old_val {
            warn!("Logic error: Buffer at {} was set to be added, but already present! (Updated instead)", old_val.read().path.display());
        }
    }

    pub fn edit(
        &self,
        uri: &Url,
        changes: &[TextDocumentContentChangeEvent],
    ) -> Result<ChangedRanges> {
        let mut w_lock = self.buffers.write();
        if let Some(buffer) = w_lock.get_mut(&uri.to_file_path().unwrap()) {
            return Ok(buffer.write().edit(changes));
        }
        Err(Error::invalid_params(format!("No such buffer: {}", uri)))
    }
}

type WBuffer<'a> = RwLockWriteGuard<'a, Buffer>;
type RBuffer<'a> = RwLockReadGuard<'a, Buffer>;
pub struct Buffer {
    /// path to the buffer
    pub path: PathBuf,
    /// the project this buffer belongs to
    pub project: String,
    /// the source set this buffer belongs to
    pub source_set: String,
    pub tree: tree_sitter::Tree,
    pub text: Rope,
}

impl Buffer {
    pub fn from_file(project: String, source_set: String, path: PathBuf) -> Self {
        let (tree, text) = parser::parse_file(&path);
        let tree = tree.unwrap();

        Buffer {
            path: path.clone(),
            project,
            source_set,
            text: text.into(),
            tree,
        }
    }
    pub fn node_chain_to(&self, byte_pos: usize) -> Vec<Node> {
        let mut node_chain = vec![];
        while let Some(node) = self
            .tree
            .root_node()
            .descendant_for_byte_range(byte_pos, byte_pos)
        {
            node_chain.push(node)
        }
        if tracing::event_enabled!(Level::TRACE) {
            let nodes = node_chain
                .iter()
                .map(|n| parser::text_of(n, &self.text))
                .fold(String::new(), |a, b| a + &b + "->");
            trace!("Nodes to byte_pos {byte_pos}: {}", nodes);
        }

        node_chain
    }

    pub fn text_at(&self, position: &Position) -> Result<String> {
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

    pub fn edit(&mut self, changes: &[TextDocumentContentChangeEvent]) -> ChangedRanges {
        trace!("Buffer before edits:\n{}", self.text.to_string());
        trace!("Tree before edits:\n{}", self.tree.root_node().to_sexp());

        let mut new_client_changed_ranges: Vec<TextByteRange> = vec![];
        let mut old_client_changed_ranges: Vec<TextByteRange> = vec![];

        for change in changes {
            if let Some(range) = &change.range {
                let old_byte_range = self.to_byte_range(range);
                old_client_changed_ranges.push(byte_range_from_usize_range(&old_byte_range));

                let new_byte_range =
                    old_byte_range.start..(old_byte_range.start + change.text.len());
                new_client_changed_ranges.push(byte_range_from_usize_range(&new_byte_range));

                self.text.replace(old_byte_range.clone(), &change.text);
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
        trace!("Buffer after edits:\n{}", self.text.to_string());

        let new_tree = parser::reparse(&self.text.to_string() /*TODO pass rope*/, &self.tree)
            .expect("Not handling no tree yet");

        let new_ts_changed_ranges: Vec<TextByteRange> = self
            .tree
            .changed_ranges(&new_tree)
            .map(|range| range.start_byte as u32..range.end_byte as u32)
            .collect();

        let old_ts_changed_ranges = range_util::map_new_ranges_to_old_ranges(
            &new_ts_changed_ranges,
            &old_client_changed_ranges,
            changes.iter().map(|change| change.text.as_str()).collect(),
        );

        let changed_ranges = ChangedRanges {
            old_ranges: range_util::merge_ranges(
                &old_ts_changed_ranges,
                &old_client_changed_ranges,
            ),
            new_ranges: range_util::merge_ranges(
                &new_ts_changed_ranges,
                &new_client_changed_ranges,
            ),
        };

        trace!("Changed ranges: {:?}", changed_ranges);

        self.tree = new_tree;
        trace!("Tree after edits:\n{}", self.tree.root_node().to_sexp());

        changed_ranges
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

    pub fn to_byte_position(&self, point: &tower_lsp::lsp_types::Position) -> usize {
        self.text.byte_of_line(point.line as usize) + point.character as usize
    }

    fn to_byte_range(&self, range: &tower_lsp::lsp_types::Range) -> std::ops::Range<usize> {
        self.to_byte_position(&range.start)..self.to_byte_position(&range.end)
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
