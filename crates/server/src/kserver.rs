use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use stdx::new_arc_rw_lock;
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::InitializeParams;
use tower_lsp::{async_trait, lsp_types::*, Client, LanguageServer};
use tracing::{debug, error, info, trace};
use walkdir::WalkDir;

use crate::buffer::Buffers;
use crate::indexes::Indexes;

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

pub struct Symbol {}

pub struct KServer {
    pub client: Box<dyn ClientI>,
    pub indexes: Indexes,
    pub buffers: Buffers,
    pub workspace_root: Arc<RwLock<Option<PathBuf>>>,
}

impl KServer {
    pub fn new(client: Box<dyn ClientI>) -> Self {
        KServer {
            client,
            indexes: Indexes::new(),
            buffers: Buffers::new(),
            workspace_root: new_arc_rw_lock(None), // set in initialize
        }
    }

    async fn load_source_files_in_workspace(&self) -> Result<()> {
        trace!(
            "Loading source files in workspace {:?}",
            self.workspace_root.read()
        );
        let Some(workspace_root) = self.workspace_root.read().clone() else {
            return Ok(());
        };

        debug!("Loading all source files in {:?}", workspace_root);

        let source_dir = workspace_root.join("src/main/kotlin");

        if !source_dir.is_dir() {
            return Ok(());
        }

        for f in WalkDir::new(source_dir)
            .follow_links(true)
            .into_iter()
            .map(|e| e.unwrap())
            .filter(|f| f.file_type().is_file())
        {
            trace!("Visiting file {:?}", f.path());
            // TODO make individual buffer creation possible without interaction with buffers
            // then create indexes for buffer
            // then collect buffer and indexes
            self.buffers
                .add_from_file(f.into_path(), |buffer| {
                    if let Err(e) = self.indexes.add_from_buffer(buffer) {
                        panic!("Unhandled err {}", e);
                    }
                })
                .await;
        }

        Ok(())
    }

    pub fn text_of(&self, uri: &Url) -> String {
        self.buffers
            .read(uri, |buffer| Ok(buffer.text.to_string()))
            .unwrap()
    }
    pub fn tree_of(&self, uri: &Url) -> String {
        self.buffers
            .read(uri, |buffer| Ok(buffer.tree.root_node().to_sexp()))
            .unwrap()
    }
}

fn find_workspace_folder(params: &InitializeParams) -> Result<Option<PathBuf>> {
    let folders = params
        .workspace_folders
        .as_ref()
        .map(|v| Ok(v))
        .transpose()?;
    match folders {
        None => Ok(None),
        Some(folders) => {
            assert!(folders.len() == 1);
            assert!(folders[0].uri.scheme() == "file");
            Ok(Some(folders[0].uri.to_file_path().unwrap().to_owned()))
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for KServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        (*self.workspace_root.write()) = find_workspace_folder(&params).unwrap();

        self.load_source_files_in_workspace().await?;

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
                completion_provider: Some(CompletionOptions::default()),
                // definition: Some(GotoCapability::default()),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down");
        Ok(())
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let text = self
            .buffers
            .read(&params.text_document_position.text_document.uri, |buffer| {
                buffer.text_at(params.text_document_position.position)
            })?;

        let completions = self.indexes.completions_for(&text);

        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn did_open(&self, _: DidOpenTextDocumentParams) {
        // let doc = &params.text_document;
        // assert!(
        //     doc.language_id == "kotlin",
        //     "Unexpected language_id {}",
        //     doc.language_id
        // );
        // assert!(doc.uri.scheme() == "file");

        // let path = params
        //     .text_document
        //     .uri
        //     .to_file_path()
        //     .expect("URI::to_file_path failed")
        //     .strip_prefix(&self.workspace_root.read().as_path())
        //     .unwrap()
        //     .to_path_buf();
        // let tree = parse_kotlin::parse(&params.text_document.text).unwrap();

        // let mut wlock = self.asts.write();
        // wlock.insert(path, tree);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // TODO synchronize notification. See
        // https://github.com/ebkalderon/tower-lsp/issues/284
        let handle_err = |e: Error| {
            error!("Error on did_change: {}", e);
        };

        let edited_ranges = match self
            .buffers
            .edit(&params.text_document.uri, &params.content_changes)
        {
            Ok(r) => r,
            Err(e) => {
                handle_err(e);
                return;
            }
        };

        if let Err(e) = self.buffers.read(&params.text_document.uri, |buffer| {
            self.indexes.add_from_buffer_changes(buffer, &edited_ranges)
        }) {
            handle_err(e);
        }
    }
}
