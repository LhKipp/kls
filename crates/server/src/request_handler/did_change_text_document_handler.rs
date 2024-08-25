use crop::Rope;
use tower_lsp::lsp_types::DidChangeTextDocumentParams;

use crate::{kserver::KServer, to_file_path};
use anyhow::anyhow;

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
        let rope = &mut w_s_file.kind.as_file_mut().unwrap().text;
        self.edit_rope(rope)?;

        Ok(())
    }

    fn edit_rope(&self, rope: &mut Rope) -> anyhow::Result<()> {
        for change in &self.notification.content_changes {
            if let Some(range) = &change.range {
                let old_byte_range = self.to_byte_range(rope, range);
                rope.replace(old_byte_range.clone(), &change.text);
            }
        }
        Ok(())
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
