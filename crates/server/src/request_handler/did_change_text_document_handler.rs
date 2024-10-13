use crate::{
    range_util::*,
    scope::GSFile,
    scope_builder::{ChangedRange, ScopeBuilder, UpsertOrDelete},
};
use crop::Rope;
use stdx::TextRange;
use tap::Tap;
use tower_lsp::lsp_types::{DidChangeTextDocumentParams, Range};
use tracing::{debug, info, instrument::WithSubscriber, trace};
use tree_sitter::{InputEdit, Point, Tree};

use crate::{kserver::KServer, to_file_path};
use anyhow::{anyhow, bail};

#[derive(new)]
pub struct DidChangeTextDocumentHandler<'a> {
    server: &'a KServer,
    notification: &'a DidChangeTextDocumentParams,
}

/// Range in the text before edits
type OldRange = TextRange;
/// Range in the text after edits
type NewRange = TextRange;

#[derive(Debug)]
struct LspTextEdit {
    pub lsp_range: Range,
    pub byte_range_in_rope: TextRange,
    pub text: String,
    pub lsp_range_after_apply: Range,
}

#[derive(Debug)]
enum ByteEditOffsets {
    Insert {
        before_byte: u32,
        shift_right: u32,
    },
    Deletion {
        starting_at_byte: u32,
        shift_left: u32,
    },
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

        let (changed_ranges, new_ast) = self.edit_rope(s_file)?;
        // from now on everything is a NewRange

        trace!("Buffer after edits:\n{}", s_file.text.to_string());
        trace!("Tree after edits:\n{}", new_ast.root_node().to_sexp());

        for changed_range in changed_ranges {
            // TODO handle result
            ScopeBuilder::new(s_file, changed_range).update_scopes(&new_ast)?;
        }
        s_file.ast = new_ast;

        Ok(())
    }

    fn edit_rope(&self, s_file: &mut GSFile) -> anyhow::Result<(Vec<ChangedRange>, Tree)> {
        let mut edit_offsets: Vec<ByteEditOffsets> =
            Vec::with_capacity(self.notification.content_changes.len());

        let mut i = 0usize;
        while let Some(edit) = &self.merged_content_changes(s_file, &mut i)? {
            trace!("applying merged_content_change: {:?}", edit);

            // old_range : val myValue
            // new_range1: val myV          -> applies_after_byte = byte_of(`V`), offset = -4
            // new_range2: val myValueHere  -> applies_after_byte = byte_of(`e`), offset = 4
            let offset = edit.text.len() as i32 - edit.byte_range_in_rope.len() as i32;
            if offset < 0 {
                edit_offsets.push(ByteEditOffsets::Deletion {
                    starting_at_byte: edit.byte_range_in_rope.start,
                    shift_left: offset.unsigned_abs(),
                });
            } else if offset > 0 {
                edit_offsets.push(ByteEditOffsets::Insert {
                    before_byte: edit.byte_range_in_rope.start,
                    shift_right: offset.unsigned_abs(),
                });
            } // no need to handle offset == 0, as exact replacements do not change range bytes

            let new_byte_range = edit.byte_range_in_rope.start
                ..(edit.byte_range_in_rope.start + edit.text.len() as u32);

            s_file
                .text
                .replace(edit.byte_range_in_rope.into_usize_range(), &edit.text);
            let new_end_point = ts_point_of(&s_file.text, new_byte_range.end as usize);

            s_file.ast.edit(&InputEdit {
                start_byte: new_byte_range.start as usize,
                old_end_byte: edit.byte_range_in_rope.end as usize,
                new_end_byte: new_byte_range.end as usize,
                start_position: Point::new(
                    edit.lsp_range.start.line as usize,
                    edit.lsp_range.start.character as usize,
                ),
                old_end_position: Point::new(
                    edit.lsp_range.end.line as usize,
                    edit.lsp_range.end.character as usize,
                ),
                new_end_position: new_end_point,
            });
        }

        // Update the tree
        let new_ast = parser::parse(&s_file.text, Some(&s_file.ast)).expect("No tree returned");
        // store changed_ranges for later
        let mut changed_ranges: Vec<_> = s_file
            .ast // the ranges are NewRange
            .changed_ranges(&new_ast)
            .map(ts_range_to_text_range)
            .map(|r| ChangedRange(r, UpsertOrDelete::Upsert))
            .collect();

        trace!(
            "Got changed ranges after updating the tree: {:?}",
            changed_ranges
        );

        // Now we got to update all old stored ranges
        for scope_node in s_file.scopes.iter_mut() {
            let scope = scope_node.get_mut();
            scope.range = Self::old_range_to_new_range(&edit_offsets, &scope.range)
                .tap_dbg(|r| trace!("Setting scope.range {} to {}", scope.range, r));
            if scope.range.is_empty() {
                debug!("Scope at {} got deleted. Adding as to delete", scope.range);
                changed_ranges.push(ChangedRange(scope.range, UpsertOrDelete::Delete));
            }
        }

        Ok((changed_ranges, new_ast))
    }

    fn old_range_to_new_range(
        edit_offsets: &Vec<ByteEditOffsets>,
        old_range: &OldRange,
    ) -> NewRange {
        trace!(
            "Updating old range {:?} based on edit_offsets {:?}",
            old_range,
            edit_offsets
        );
        let mut new_range = *old_range;
        for edit_offset in edit_offsets {
            trace!(
                "Applying edit_offset {:?} to new_range {}",
                edit_offset,
                new_range
            );
            match edit_offset {
                ByteEditOffsets::Insert {
                    before_byte,
                    shift_right,
                } => {
                    if new_range.start >= *before_byte {
                        new_range = new_range.shift_right_by(*shift_right);
                    } else if new_range.end > *before_byte {
                        new_range.end += *shift_right;
                    }
                }
                ByteEditOffsets::Deletion {
                    starting_at_byte,
                    shift_left,
                } => {
                    if *starting_at_byte <= new_range.start {
                        // case 1: Edit is before the range
                        // case 2: Edit is before the range && in the range
                        // case 3: Edit is in the range

                        // Make sure that when edit is a delete, the start is at most moved to the edit start
                        let distance_to_start = starting_at_byte.abs_diff(new_range.start);
                        if *shift_left >= distance_to_start {
                            new_range.start = *starting_at_byte;
                        } else {
                            new_range.start = new_range.start.strict_sub(*shift_left);
                        }

                        // Same for the end
                        let distance_to_end = starting_at_byte.abs_diff(new_range.end);
                        if *shift_left >= distance_to_end {
                            new_range.end = *starting_at_byte;
                        } else {
                            new_range.end = new_range.end.strict_sub(*shift_left);
                        }
                    } else {
                        // case 4: Edit is after the range => nothing to do
                    }
                }
            }
        }
        trace!("Returning {:?} after as the new_range", new_range);
        new_range
    }

    fn merged_content_changes(
        &self,
        s_file: &mut GSFile,
        i: &mut usize,
    ) -> anyhow::Result<Option<LspTextEdit>> {
        if *i == self.notification.content_changes.len() {
            return Ok(None);
        }

        let next_content_changes = &self.notification.content_changes[*i..];
        let mut edit: Option<LspTextEdit> = None;

        for content_change in next_content_changes {
            let Some(content_change_range_lsp) = content_change.range else {
                bail!("editor sends deprecated DidChangeTextDocument notification. Expected field `range` is not send");
            };
            if let Some(prior_edit) = &mut edit {
                if prior_edit.lsp_range_after_apply.end == content_change_range_lsp.start {
                    prior_edit.lsp_range.end = content_change_range_lsp.end;
                    prior_edit.byte_range_in_rope.end +=
                        lsp_range_byte_distance(&content_change_range_lsp);
                    prior_edit.text += &content_change.text;
                    prior_edit.lsp_range_after_apply =
                        lsp_range_apply_text_edit(&prior_edit.lsp_range, &prior_edit.text);
                    *i += 1
                } else {
                    return Ok(edit);
                }
            } else if edit.is_none() {
                edit = Some(LspTextEdit {
                    lsp_range: content_change_range_lsp,
                    byte_range_in_rope: lsp_range_to_text_range(
                        &s_file.text,
                        &content_change_range_lsp,
                    ),
                    text: content_change.text.clone(),
                    lsp_range_after_apply: lsp_range_apply_text_edit(
                        &content_change_range_lsp,
                        &content_change.text,
                    ),
                });
                *i += 1;
            } else {
                return Ok(edit);
            }
        }

        Ok(edit)
    }
}
