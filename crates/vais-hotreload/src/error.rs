//! Error types for hot reloading

use thiserror::Error;

pub type Result<T> = std::result::Result<T, HotReloadError>;

#[derive(Error, Debug)]
pub enum HotReloadError {
    #[error("File watcher error: {0}")]
    WatchError(#[from] notify::Error),

    #[error("Library loading error: {0}")]
    LoadError(#[from] libloading::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Compilation error: {0}")]
    CompilationError(String),

    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("Invalid dylib path: {0}")]
    InvalidPath(String),

    #[error("Hot reload not initialized")]
    NotInitialized,

    #[error("Reload already in progress")]
    ReloadInProgress,

    #[error("Timeout waiting for compilation")]
    CompilationTimeout,
}
