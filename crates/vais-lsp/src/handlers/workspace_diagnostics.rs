//! Workspace-wide diagnostics collection
//!
//! Scans all `.vais` files reachable from any open document's directory tree
//! and publishes parse diagnostics for each file to the LSP client.

use std::path::Path;

use tower_lsp::lsp_types::*;
use vais_parser::parse;

use crate::backend::VaisBackend;
use crate::diagnostics::parse_error_to_diagnostic;

/// Collect all `.vais` files under `root`, recursively.
fn collect_vais_files(root: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    collect_vais_files_inner(root, &mut files);
    files
}

fn collect_vais_files_inner(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip hidden directories and common non-source dirs
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
            collect_vais_files_inner(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("vais") {
            out.push(path);
        }
    }
}

/// Derive a workspace root from the open document URIs.
///
/// Walks up from each open document's directory until we reach a directory
/// that contains a `Cargo.toml` or `.git` marker (project root). Falls back
/// to the document's directory if no marker is found.
fn find_workspace_root(backend: &VaisBackend) -> Option<std::path::PathBuf> {
    for entry in backend.documents.iter() {
        let uri = entry.key();
        if let Ok(file_path) = uri.to_file_path() {
            let mut candidate = file_path.parent()?.to_path_buf();
            loop {
                if candidate.join("Cargo.toml").exists()
                    || candidate.join(".git").exists()
                    || candidate.join("vais.toml").exists()
                {
                    return Some(candidate);
                }
                match candidate.parent() {
                    Some(p) => candidate = p.to_path_buf(),
                    None => break,
                }
            }
            // No project root marker found; use document directory
            return file_path.parent().map(|p| p.to_path_buf());
        }
    }
    None
}

/// Publish diagnostics for every `.vais` file in the workspace.
///
/// Files that are currently open in the editor already receive live
/// diagnostics via `did_change`; this function ensures that all other
/// `.vais` files in the project are also checked.
pub(crate) async fn publish_workspace_diagnostics(backend: &VaisBackend) {
    let root = match find_workspace_root(backend) {
        Some(r) => r,
        None => return,
    };

    let vais_files = collect_vais_files(&root);

    for path in vais_files {
        let uri = match Url::from_file_path(&path) {
            Ok(u) => u,
            Err(_) => continue,
        };

        // Skip files already tracked as open documents (they are handled live)
        if backend.documents.contains_key(&uri) {
            continue;
        }

        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let diagnostics = match parse(&source) {
            Ok(_) => vec![],
            Err(err) => vec![parse_error_to_diagnostic(&err, &source)],
        };

        backend
            .client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}
