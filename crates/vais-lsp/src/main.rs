//! Vais Language Server
//!
//! Implements the Language Server Protocol for Vais source files.

use tower_lsp::{LspService, Server};

mod backend;
mod diagnostics;
mod semantic;

use backend::VaisBackend;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| VaisBackend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
