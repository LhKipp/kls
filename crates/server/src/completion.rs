use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, Position,
};

use crate::error_util::map_err;
use crate::scope::{SItem, SItemKind};
use crate::{buffer::Buffers, scope::Scopes};

#[derive(new)]
pub struct Completion {
    buffers: Buffers,
    scopes: Scopes,
}

impl Completion {
    pub async fn completions_for(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let buffer = self
            .buffers
            .at(&params.text_document_position.text_document.uri)?;
        let r_buffer = buffer.read();

        let cursor_byte_pos = r_buffer.to_byte_position(&params.text_document_position.position);
        let node_chain = r_buffer.node_chain_to(cursor_byte_pos);

        if node_chain.is_empty() {
            return Err(Error::invalid_params(format!(
                "Cursor is at byte position {}, but this byte is not contained in any ast node",
                cursor_byte_pos
            )));
        }

        // Infer position in ast
        let last_node = node_chain.last().unwrap();
        let completion_items = match last_node.kind() {
            "simple_identifier" => {
                let node_text = parser::text_of(last_node, &r_buffer.text);
                match self.scopes.find(
                    &r_buffer.project,
                    &r_buffer.source_set,
                    &r_buffer.path,
                    &node_text,
                ) {
                    Ok(items) => items.into_iter().map(|item| self.map(item)).collect(),
                    Err(e) => return map_err(e),
                }
            }

            _ => return Err(Error::invalid_params("The position is not yet handled")),
        };

        Ok(Some(CompletionResponse::Array(completion_items)))
    }

    fn map(&self, item: SItem) -> CompletionItem {
        match item.item {
            SItemKind::SourceFileMetadata(_) => todo!(),
            SItemKind::PackageHeader(_) => todo!(),
            SItemKind::Class(_) => todo!(),
            SItemKind::Var(var) => {
                return CompletionItem {
                    label: var.name,
                    kind: Some(CompletionItemKind::VARIABLE),
                    ..CompletionItem::default()
                }
            }
        }
    }
}
