#![allow(dead_code)]

use server::kserver::KServer;
use tower_lsp::{LspService, Server};
use tracing::debug;
use tracing_subscriber::EnvFilter;

// extern "C" {
//     fn tree_sitter_kotlin() -> Language;
// }

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| KServer::new(Box::new(client))).finish();

    debug!("KLS starting");
    Server::new(stdin, stdout, socket).serve(service).await;
}
