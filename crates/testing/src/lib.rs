#[macro_use]
extern crate derive_builder;

use server::kserver::ClientI;
use server::kserver::KServer;
use stdx::new_arc_lock;
use stdx::AMtx;
use tower_lsp::async_trait;
use tower_lsp::lsp_types::InitializedParams;
use tower_lsp::lsp_types::MessageType;
use tower_lsp::lsp_types::Url;
use tower_lsp::lsp_types::WorkspaceFolder;
use tower_lsp::{lsp_types::InitializeParams, LanguageServer};
use tracing::info;
use tracing_subscriber::EnvFilter;

pub mod completion;
pub mod workspace;
pub use workspace::*;

pub struct TestClientData {}

#[derive(Clone)]
pub struct TestClient {
    pub v: AMtx<TestClientData>,
}
impl TestClient {
    pub fn new() -> Self {
        TestClient {
            v: new_arc_lock(TestClientData {}),
        }
    }
}

#[async_trait]
impl ClientI for TestClient {
    async fn log_message(&self, ty: MessageType, msg: String) {
        info!("ClientLogging: {:?} {}", ty, msg);
    }
}

#[derive(Builder)]
#[builder(setter(into), default)]
pub struct ServerInitOptions {
    pub init: bool,
    pub workspace: Option<Workspace>,
}

impl Default for ServerInitOptions {
    fn default() -> Self {
        Self {
            init: true,
            workspace: Some(Workspace::new()),
        }
    }
}

impl ServerInitOptions {
    pub fn workspace(&mut self) -> &mut Workspace {
        self.workspace.as_mut().unwrap()
    }
}

pub async fn server_init() -> (TestClient, KServer) {
    server_init_(ServerInitOptions::default()).await
}
pub async fn server_init_(init_opts: ServerInitOptions) -> (TestClient, KServer) {
    let client = TestClient::new();
    let server = KServer::new(Box::new(client.clone()));

    if init_opts.init {
        let mut params = InitializeParams::default();

        if let Some(workspace) = init_opts.workspace {
            params.workspace_folders = Some(vec![WorkspaceFolder {
                uri: Url::parse(&("file://".to_string() + workspace.root.to_str().unwrap()))
                    .unwrap(),
                name: "test_workspace_name".into(),
            }])
        }

        server
            .initialize(params)
            .await
            .expect("server.initialize returned err");
        server.initialized(InitializedParams {}).await;
    }

    (client, server)
}

pub fn init_test() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .without_time()
        .init();
}
