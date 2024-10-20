use tracing::level_filters::LevelFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

pub fn setup() {
    let subscriber = tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(fmt::Layer::default().with_writer(std::io::stdout));
    tracing::subscriber::set_global_default(subscriber).expect("unable to set test subscriber");
}
