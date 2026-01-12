//! AOEL Language Server
//!
//! LSP implementation for AOEL AOEL syntax

mod backend;
mod builtins;
mod diagnostics;
mod document;

use tower_lsp::{LspService, Server};
use backend::Backend;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
