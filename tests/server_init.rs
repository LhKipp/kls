use kls::kserver::ClientI;
use kls::kserver::KServer;
use kls::stdx::new_amtx;
use kls::stdx::AMtx;
use tower_lsp::{lsp_types::InitializeParams, LanguageServer};

pub struct TestClientData {}

#[derive(Clone)]
pub struct TestClient {
    pub v: AMtx<TestClientData>,
}
impl TestClient {
    pub fn new() -> Self {
        TestClient {
            v: new_amtx(TestClientData {}),
        }
    }
}
impl ClientI for TestClient {}

struct ServerInitOptions {
    init: bool,
}
impl Default for ServerInitOptions {
    fn default() -> Self {
        Self { init: true }
    }
}

async fn server_init() -> (TestClient, KServer) {
    server_init_(ServerInitOptions::default()).await
}
async fn server_init_(init_opts: ServerInitOptions) -> (TestClient, KServer) {
    let client = TestClient::new();
    let server = KServer {
        client: Box::new(client.clone()),
    };
    if init_opts.init {
        server
            .initialize(InitializeParams::default())
            .await
            .expect("server.initialize returned err");
        server.initialized(InitializedParams::default())
    }

    (client, server)
}

#[tokio::test]
async fn initialize_should_return_lsp_info() {
    let (_, server) = server_init_(ServerInitOptions { init: false }).await;

    let init = server
        .initialize(InitializeParams::default())
        .await
        .expect("server.initialize failed");
    let server_info = init.server_info.expect("No ServerInfo returned");
    assert_eq!(server_info.name, "Kls");
    assert_eq!(server_info.version, None);
}

#[tokio::test]
async fn shutdown_should_return_ok() {
    let (_, server) = server_init().await;
    server.shutdown().await.expect("server.shutdown failed")
}
