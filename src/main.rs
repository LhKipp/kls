#![allow(dead_code)]

use kls::kserver::KServer;
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

    let (service, socket) = LspService::build(|client| KServer {
        client: Box::new(client),
    })
    .finish();

    debug!("KLS starting");
    Server::new(stdin, stdout, socket).serve(service).await;
}

// let language = unsafe { tree_sitter_kotlin() };
// parser.set_language(language).unwrap();

// let source_code = "fun test() {}";
// let tree = parser.parse(source_code, None).unwrap();
// let root_node = tree.root_node();

// assert_eq!(root_node.kind(), "source_file");
// assert_eq!(root_node.start_position().column, 0);
// assert_eq!(root_node.end_position().column, 13);
