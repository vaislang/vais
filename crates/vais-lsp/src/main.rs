//! Vais Language Server
//!
//! Implements the Language Server Protocol for Vais source files.

use tower_lsp::{LspService, Server};

mod ai_completion;
mod backend;
mod diagnostics;
mod semantic;

// Backend module extensions
mod analysis;
mod folding;
mod hints;
mod symbol_analysis;

use backend::VaisBackend;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(VaisBackend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
