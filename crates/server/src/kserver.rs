use std::path::PathBuf;
use stdx::{new_arc_rw_lock, ARwLock};
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::request::{GotoTypeDefinitionParams, GotoTypeDefinitionResponse};
use tower_lsp::lsp_types::InitializeParams;
use tower_lsp::{async_trait, lsp_types::*, Client, LanguageServer};
use tracing::{debug, error, info, trace};
use walkdir::WalkDir;

use crate::buffer::Buffers;
use crate::project::Project;
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

pub struct Symbol {}

pub struct KServer {
    pub client: Box<dyn ClientI>,
    pub scopes: Scopes,
    pub buffers: Buffers,
    pub root_project: ARwLock<Project>,
}

impl KServer {
    pub fn new(client: Box<dyn ClientI>) -> Self {
        KServer {
            client,
            scopes: Scopes::new(),
            buffers: Buffers::new(),
            // After init phase, project will be set to something meaningful (panic otherwise)
            root_project: new_arc_rw_lock(Project::invalid_project()),
        }
    }

    async fn load_source_files_in_workspace(&self) -> Result<()> {
        debug!(
            "Loading source files in workspace {:?}",
            self.root_project.read()
        );
        let (source_dir, project_name) = {
            let r_project = self.root_project.read();
            (
                r_project.root_path().join("src/main/kotlin"),
                r_project.name(),
            )
        };

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
            let scopes = self
                .buffers
                .add_from_file(f.clone().into_path(), |buffer| {
                    let (scopes, errors) = Scope::build_scopes_from(buffer);
                    for e in errors {
                        println!("TODO handle error {}", e.msg);
                    }
                    scopes
                })
                .await;

            if let Err(e) = self.scopes.add_module(&project_name, scopes) {
                error!("Error loading source file {}: {:?}", f.path().display(), e);
                return Err(Error::internal_error());
            }
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
        let Some(project_path) = find_workspace_folder(&params).unwrap() else {
            panic!("No project passed to LSP via InitializeParams");
        };
        (*self.root_project.write()) = Project::new(project_path);

        self.scopes.add_project(self.root_project.read().name());
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

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        // let text = self
        //     .buffers
        //     .read(&params.text_document_position.text_document.uri, |buffer| {
        //         buffer.text_at(params.text_document_position.position)
        //     })?;

        // let completions = self.scopes.completions_for(&text);

        Ok(Some(CompletionResponse::Array(vec![])))
    }

    // async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
    //     todo!();
    // }

    async fn goto_type_definition(
        &self,
        _params: GotoTypeDefinitionParams,
    ) -> Result<Option<GotoTypeDefinitionResponse>> {
        todo!();
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

        let _edited_ranges = match self
            .buffers
            .edit(&params.text_document.uri, &params.content_changes)
        {
            Ok(r) => r,
            Err(e) => {
                handle_err(e);
                return;
            }
        };

        // TODO edit scopes

        // if let Err(e) = self.buffers.read(&params.text_document.uri, |buffer| {
        //     self.indexes.add_from_buffer_changes(buffer, &edited_ranges)
        // }) {
        //     handle_err(e);
        // }
    }
}
