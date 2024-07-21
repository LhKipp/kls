use futures::future::join_all;
use parking_lot::RwLockWriteGuard;
use std::path::PathBuf;
use stdx::{new_arc_rw_lock, ARwLock};
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::request::{GotoTypeDefinitionParams, GotoTypeDefinitionResponse};
use tower_lsp::lsp_types::InitializeParams;
use tower_lsp::{async_trait, lsp_types::*, Client, LanguageServer};
use tracing::{debug, error, info, trace};
use walkdir::WalkDir;

#[async_trait]
pub trait ClientI: Send + Sync {
    async fn log_message(&self, ty: MessageType, msg: String);
}

#[async_trait]
impl ClientI for Client {
    async fn log_message(&self, ty: MessageType, msg: String) {
        self.log_message(ty, msg).await;
    }
}

pub struct KServer {
    pub client: Box<dyn ClientI>,
}

impl KServer {
    pub fn new(client: Box<dyn ClientI>) -> Self {
        KServer { client }
    }
}

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
                inlay_hint_provider: None,
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: None,
                // definition: Some(GotoCapability::default()),
                definition_provider: None,
                references_provider: None,
                rename_provider: None,
                ..ServerCapabilities::default()
            },
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, _: DidOpenTextDocumentParams) {}

    async fn did_change(&self, _: DidChangeTextDocumentParams) {
        // TODO synchronize notification. See
        // https://github.com/ebkalderon/tower-lsp/issues/284
    }
}
