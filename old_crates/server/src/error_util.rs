use tracing::error;

pub fn map_err<T>(err: anyhow::Error) -> tower_lsp::jsonrpc::Result<T> {
    error!("{}", err);
    return Err(tower_lsp::jsonrpc::Error::internal_error());
}
