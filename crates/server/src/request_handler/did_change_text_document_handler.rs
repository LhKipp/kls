use crop::Rope;
use tower_lsp::lsp_types::DidChangeTextDocumentParams;
use tracing::{info, instrument::WithSubscriber, trace};
use tree_sitter::{InputEdit, Point};

use crate::{kserver::KServer, to_file_path};
use anyhow::{anyhow, bail};

#[derive(new)]
pub struct DidChangeTextDocumentHandler<'a> {
    server: &'a KServer,
    notification: &'a DidChangeTextDocumentParams,
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
        // let rope = &mut w_s_file.kind.as_file_mut().unwrap().text;
        trace!("Buffer before edits:\n{}", s_file.text.to_string());
        trace!("Tree before edits:\n{}", s_file.tree.root_node().to_sexp());

        self.edit_rope(&mut s_file.text, &mut s_file.tree)?;

        Ok(())
    }

    fn edit_rope(&self, rope: &mut Rope, ast: &mut tree_sitter::Tree) -> anyhow::Result<()> {
        for change in &self.notification.content_changes {
            if change.range_length.is_some() && change.range.is_none() {
                bail!("editor sends deprecated DidChangeTextDocument notification. Expected field `range`, but only `range_length` has been provided");
            }
            if let Some(range) = &change.range {
                info!("range {:?}", range);
                let old_byte_range = self.to_byte_range(rope, range);
                // old_client_changed_ranges.push(byte_range_from_usize_range(&old_byte_range));

                let new_byte_range =
                    old_byte_range.start..(old_byte_range.start + change.text.len());
                // new_client_changed_ranges.push(byte_range_from_usize_range(&new_byte_range));

                rope.replace(old_byte_range.clone(), &change.text);
                let new_end_point = self.point_of(rope, new_byte_range.end);

                ast.edit(&InputEdit {
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

        let new_ast = parser::parse(rope, Some(ast)).expect("No tree returned");
        let changed_ranges = ast.changed_ranges(&new_ast);

        for r in changed_ranges {
            info!("Changed Range: {}-{}", r.start_point, r.end_point);
        }

        ast.root_node().edit(edit)

        *ast = new_ast;

        // iterate over all nodes of ast and populate
        // bi_map <own_node_i, node_range> (or insert if not yet present)
        // return tainted own_node_id's (node_ids changed or deleted)
        //
        // caller: iterte over scopes, if scopes node is tainted, update the scope, by looking up
        // ast-node 

        Ok(())
    }

    fn point_of(&self, rope: &Rope, byte_offset: usize) -> Point {
        let row = rope.line_of_byte(byte_offset);
        let col = byte_offset - rope.byte_of_line(row);

        Point::new(row, col)
    }

    fn to_byte_range(
        &self,
        rope: &Rope,
        range: &tower_lsp::lsp_types::Range,
    ) -> std::ops::Range<usize> {
        self.to_byte_position(rope, &range.start)..self.to_byte_position(rope, &range.end)
    }

    pub fn to_byte_position(&self, rope: &Rope, point: &tower_lsp::lsp_types::Position) -> usize {
        rope.byte_of_line(point.line as usize) + point.character as usize
    }
}
