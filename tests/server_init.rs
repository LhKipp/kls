use testing::*;
use tower_lsp::{lsp_types::InitializeParams, LanguageServer};

#[tokio::test]
async fn initialize_should_return_lsp_info() {
    let (_, server) = server_init_(ServerInitOptions {
        init: false,
        workspace: None,
    })
    .await;

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
