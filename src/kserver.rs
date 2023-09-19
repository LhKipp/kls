use tower_lsp::jsonrpc::Result;
use tower_lsp::{lsp_types::InitializeParams, LanguageServer};
use tower_lsp::{lsp_types::*, Client};
use tracing::{info, warn};

pub trait ClientI: Send + Sync {}

impl ClientI for Client {}

pub struct KServer {
    pub client: Box<dyn ClientI>,
}

pub const LEGEND_TYPE: &[SemanticTokenType] = &[
    SemanticTokenType::FUNCTION,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::STRING,
    SemanticTokenType::COMMENT,
    SemanticTokenType::NUMBER,
    SemanticTokenType::KEYWORD,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::PARAMETER,
];

#[tower_lsp::async_trait]
impl LanguageServer for KServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "Kls".into(),
                version: None,
            }),
            offset_encoding: None,
            capabilities: ServerCapabilities {
                inlay_hint_provider: Some(OneOf::Left(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                // definition: Some(GotoCapability::default()),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let _ = params;
        warn!("Got a textDocument/didOpen notification, but it is not implemented");
    }
}
