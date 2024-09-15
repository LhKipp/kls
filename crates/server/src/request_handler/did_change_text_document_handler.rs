use crate::range_util::*;
use crop::Rope;
use stdx::{i32_as_usize, usize_as_i32};
use tower_lsp::lsp_types::{DidChangeTextDocumentParams, Range};
use tracing::{info, instrument::WithSubscriber, trace};
use tree_sitter::{InputEdit, Point};

use crate::{kserver::KServer, to_file_path};
use anyhow::{anyhow, bail};

#[derive(new)]
pub struct DidChangeTextDocumentHandler<'a> {
    server: &'a KServer,
    notification: &'a DidChangeTextDocumentParams,
}

/// Range in the text before edits
type OldRange = std::ops::Range<usize>;
/// Range in the text after edits
type NewRange = std::ops::Range<usize>;

struct EditOffset {
    pub applies_after_byte: usize,
    pub offset: i32,
}

impl<'a> DidChangeTextDocumentHandler<'a> {
    pub fn handle(&self) -> anyhow::Result<()> {
        let file_path = to_file_path(&self.notification.text_document.uri)?;

        let s_file = {
            let r_scopes = self.server.scopes.0.read();
            let s_f_node_id = r_scopes.file_nodes.get(&file_path).ok_or_else(|| {
                anyhow!(
                    "File {} is not registered in file_nodes. Not handling the change request",
                    file_path.display()
                )
            })?;

            r_scopes
                .scopes
                .get(*s_f_node_id)
                .ok_or_else(|| anyhow!("Registered file_node {} is not in scopes", s_f_node_id))?
                .get()
                .clone()
        };

        let mut w_s_file = s_file.write();
        let s_file = w_s_file.kind.as_file_mut().unwrap();
        trace!("Buffer before edits:\n{}", s_file.text.to_string());
        trace!("Tree before edits:\n{}", s_file.ast.root_node().to_sexp());

        // TODO handle changed ranges
        let _: Vec<NewRange> = self.edit_rope(&mut s_file.text, &mut s_file.ast)?;
        // from now on everything is a NewRange

        Ok(())
    }

    fn edit_rope(
        &self,
        rope: &mut Rope,
        ast: &mut tree_sitter::Tree,
    ) -> anyhow::Result<Vec<NewRange>> {
        let mut delete_edit_ranges: Vec<OldRange> = Vec::with_capacity(0);
        let mut edit_offsets: Vec<EditOffset> =
            Vec::with_capacity(self.notification.content_changes.len());

        for edit in &self.notification.content_changes {
            if edit.range_length.is_some() && edit.range.is_none() {
                bail!("editor sends deprecated DidChangeTextDocument notification. Expected field `range`, but only `range_length` has been provided");
            }
            if let Some(edit_range) = &edit.range {
                info!("range {:?}", edit_range);
                let old_byte_range = lsp_range_to_usize_range(rope, edit_range);

                if edit.text.is_empty() {
                    delete_edit_ranges.push(old_byte_range.clone());
                }

                edit_offsets.push(EditOffset {
                    applies_after_byte: old_byte_range.end,
                    offset: old_byte_range.len() as i32 - edit.text.len() as i32,
                });

                // old_client_changed_ranges.push(byte_range_from_usize_range(&old_byte_range));

                let new_byte_range = old_byte_range.start..(old_byte_range.start + edit.text.len());
                // new_client_changed_ranges.push(byte_range_from_usize_range(&new_byte_range));

                rope.replace(old_byte_range.clone(), &edit.text);
                let new_end_point = ts_point_of(rope, new_byte_range.end);

                ast.edit(&InputEdit {
                    start_byte: new_byte_range.start,
                    old_end_byte: old_byte_range.end,
                    new_end_byte: new_byte_range.end,
                    start_position: Point::new(
                        edit_range.start.line as usize,
                        edit_range.start.character as usize,
                    ),
                    old_end_position: Point::new(
                        edit_range.end.line as usize,
                        edit_range.end.character as usize,
                    ),
                    new_end_position: new_end_point,
                });
            }
        }

        // Update the tree
        let new_ast = parser::parse(rope, Some(ast)).expect("No tree returned");
        // store changed_ranges for later
        let mut changed_ranges: Vec<NewRange> = ast
            .changed_ranges(&new_ast)
            .map(ts_range_to_usize_range)
            .collect();
        *ast = new_ast;

        // Now we got to update all old stored ranges
        // TODO

        // special case: when an edit completely removes a node, without adding other text, ts will
        // not return it as a changed range
        for deletion in delete_edit_ranges {
            if changed_ranges.iter().any(|r| ranges_overlap(r, &deletion)) {
                // deletion did not completely remove a node, so it is already covered for
                continue;
            }
            changed_ranges.push(Self::old_range_to_new_range(&edit_offsets, &deletion));
        }

        Ok(changed_ranges)
    }

    fn old_range_to_new_range(edit_offsets: &Vec<EditOffset>, old_range: &OldRange) -> NewRange {
        let mut new_range = old_range.clone();
        for edit_offset in edit_offsets {
            if edit_offset.applies_after_byte <= new_range.start {
                new_range.start = i32_as_usize(usize_as_i32(new_range.start) + edit_offset.offset);
                new_range.end += i32_as_usize(usize_as_i32(new_range.end) + edit_offset.offset);
            }
        }
        new_range
    }
}
