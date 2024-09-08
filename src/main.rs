#![allow(dead_code)]

use std::sync::Arc;

use server::kserver::KServer;
use tower_lsp::{LspService, Server};
use tracing::debug;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

// extern "C" {
//     fn tree_sitter_kotlin() -> Language;
// }

#[tokio::main]
async fn main() {
    let args = AppArgs::from_env().expect("Parsing arguments failed");
    init_logging(&args);

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| KServer::new(Arc::new(client)))
        .custom_method("custom/printScopes", KServer::print_scopes)
        .finish();

    debug!("KLS starting");
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[derive(Debug)]
struct AppArgs {
    log_file: Option<String>,
    start_new_log_file: bool,
    log_timestamps: bool,
}
impl AppArgs {
    fn from_env() -> Result<AppArgs, pico_args::Error> {
        let mut pargs = pico_args::Arguments::from_env();
        Ok(AppArgs {
            log_file: pargs.opt_value_from_str("--log-file")?,
            start_new_log_file: pargs.contains("--start-new-log-file"),
            log_timestamps: pargs
                .opt_value_from_str("--log-timestamps")?
                .unwrap_or(true),
        })
    }
}

fn init_logging(args: &AppArgs) {
    let stderr_layer = fmt::Layer::default().with_writer(std::io::stderr);
    let file_layer = if let Some(log_file) = &args.log_file {
        if args.start_new_log_file && std::fs::metadata(log_file).is_ok_and(|f| f.is_file()) {
            std::fs::remove_file(log_file)
                .unwrap_or_else(|_| panic!("Could not remove log file {log_file}"));
        }

        let logfile = tracing_appender::rolling::never(".", log_file);
        let layer = fmt::Layer::default().with_writer(logfile).with_ansi(false);
        if !args.log_timestamps {
            Some(layer.without_time().boxed())
        } else {
            Some(layer.boxed())
        }
    } else {
        None
    };

    let subscriber = tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(file_layer)
        .with(stderr_layer);
    tracing::subscriber::set_global_default(subscriber).expect("unable to set global subscriber");

    debug!("Logging initialized");
}
