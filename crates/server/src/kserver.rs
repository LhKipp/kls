use anyhow::bail;
use closure::closure;
use futures::future::join_all;
use parking_lot::RwLockWriteGuard;
use std::path::PathBuf;
use std::sync::Arc;
use stdx::{new_arc_lock, new_arc_rw_lock, AMtx, ARwLock};
use tokio::runtime::Handle;
use tokio::task::JoinHandle;
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::request::{GotoTypeDefinitionParams, GotoTypeDefinitionResponse};
use tower_lsp::lsp_types::InitializeParams;
use tower_lsp::{async_trait, lsp_types::*, Client, LanguageServer};
use tracing::{debug, error, info, trace};
use walkdir::WalkDir;

use crate::project::ProjectI;
use crate::request_handler::did_change_text_document_handler::DidChangeTextDocumentHandler;
use crate::request_handler::print_scopes_handler::{PrintScopesHandler, PrintScopesRequest};
use crate::scope::*;

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
    pub client: Arc<dyn ClientI>,
    // The root_dir from initialize. Empty if initialize did not yet ran
    pub root_dir: ARwLock<Option<PathBuf>>,
    pub background_tasks: AMtx<Vec<JoinHandle<()>>>,

    pub scopes: Scopes,
}

impl KServer {
    pub fn new(client: Arc<dyn ClientI>) -> Self {
        KServer {
            client,
            root_dir: new_arc_rw_lock(None),
            background_tasks: new_arc_lock(vec![]),
            scopes: Scopes::new(),
        }
    }

    /// Custom request
    pub async fn print_scopes(&self, request: PrintScopesRequest) -> Result<String> {
        map_result(PrintScopesHandler::new(&self, &request).handle())
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for KServer {
    async fn did_open(&self, _: DidOpenTextDocumentParams) {}

    async fn did_change(&self, notification: DidChangeTextDocumentParams) {
        // TODO synchronize notification. See
        // https://github.com/ebkalderon/tower-lsp/issues/284
        if let Err(e) = DidChangeTextDocumentHandler::new(&self, &notification).handle() {
            error!("{}", e);
        }
    }

    async fn initialize(&self, init_params: InitializeParams) -> Result<InitializeResult> {
        {
            let mut w_root_dir = self.root_dir.write();
            match &*w_root_dir {
                Some(_) => {
                    return Err(tower_lsp::jsonrpc::Error::invalid_params(
                        "Already initialized",
                    ))
                }
                None => *w_root_dir = Some(root_dir_of(&init_params)?),
            }
        }

        let root_project =
            match <dyn ProjectI>::new(self.root_dir.read().as_ref().unwrap().as_ref()) {
                Err(e) => return map_err(e),
                Ok(p) => p,
            };

        {
            let scopes = self.scopes.clone();
            let client = self.client.clone();
            self.background_tasks.lock().push(tokio::spawn(async move {
                if let Err(e) = scopes.add_scopes_from_project_recursive(root_project).await {
                    client
                        .log_message(
                            MessageType::ERROR,
                            format!("Error while importing project: {}", e),
                        )
                        .await
                }
            }));
        }

        let _file_operation_registration_options = FileOperationRegistrationOptions {
            filters: vec![FileOperationFilter {
                pattern: FileOperationPattern {
                    glob: "**.kt".to_string(),
                    ..FileOperationPattern::default()
                },
                ..FileOperationFilter::default()
            }],
        };

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
                // Not working :(
                // text_document_sync: Some(TextDocumentSyncCapability::Options(
                //     TextDocumentSyncOptions {
                //         open_close: Some(true),
                //         change: Some(TextDocumentSyncKind::INCREMENTAL),
                //         ..Default::default()
                //     },
                // )),
                completion_provider: None,
                // definition: Some(GotoCapability::default()),
                definition_provider: None,
                references_provider: None,
                rename_provider: None,
                // workspace: Some(WorkspaceServerCapabilities {
                //     file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                //         did_create: Some(file_operation_registration_options.clone()),
                //         did_rename: Some(file_operation_registration_options.clone()),
                //         did_delete: Some(file_operation_registration_options.clone()),
                //         ..Default::default()
                //     }),
                //     ..Default::default()
                // }),
                ..Default::default()
            },
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

fn root_dir_of(init_params: &InitializeParams) -> Result<PathBuf> {
    if let Some(workspace_folders) = &init_params.workspace_folders {
        if workspace_folders.len() != 1 {
            return Err(tower_lsp::jsonrpc::Error::invalid_params(format!(
                "Exactly 1 workspace must be passed!. Got {}",
                workspace_folders.len()
            )));
        }
        if workspace_folders[0].uri.scheme() != "file" {
            return Err(tower_lsp::jsonrpc::Error::invalid_params(
                "0 workspace folders are passed",
            ));
        }

        Ok(workspace_folders[0].uri.to_file_path().unwrap().to_owned())
    } else {
        Err(tower_lsp::jsonrpc::Error::invalid_params(
            "No workspace folders are passed",
        ))
    }
}

fn map_err<T>(err: anyhow::Error) -> tower_lsp::jsonrpc::Result<T> {
    error!("KServer caught error: {}", err);
    Err(tower_lsp::jsonrpc::Error::invalid_params(err.to_string()))
}

fn map_result<T>(result: anyhow::Result<T>) -> tower_lsp::jsonrpc::Result<T> {
    result.map_err(|e| map_err::<T>(e).err().unwrap())
}
