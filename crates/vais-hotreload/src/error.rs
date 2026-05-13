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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compilation_error_display() {
        let err = HotReloadError::CompilationError("syntax error".to_string());
        assert_eq!(format!("{}", err), "Compilation error: syntax error");
    }

    #[test]
    fn test_symbol_not_found_display() {
        let err = HotReloadError::SymbolNotFound("my_function".to_string());
        assert_eq!(format!("{}", err), "Symbol not found: my_function");
    }

    #[test]
    fn test_invalid_path_display() {
        let err = HotReloadError::InvalidPath("/bad/path".to_string());
        assert_eq!(format!("{}", err), "Invalid dylib path: /bad/path");
    }

    #[test]
    fn test_not_initialized_display() {
        let err = HotReloadError::NotInitialized;
        assert_eq!(format!("{}", err), "Hot reload not initialized");
    }

    #[test]
    fn test_reload_in_progress_display() {
        let err = HotReloadError::ReloadInProgress;
        assert_eq!(format!("{}", err), "Reload already in progress");
    }

    #[test]
    fn test_compilation_timeout_display() {
        let err = HotReloadError::CompilationTimeout;
        assert_eq!(format!("{}", err), "Timeout waiting for compilation");
    }

    #[test]
    fn test_io_error_from() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: HotReloadError = io_err.into();
        assert!(matches!(err, HotReloadError::IoError(_)));
        assert!(format!("{}", err).contains("file not found"));
    }

    #[test]
    fn test_result_type_alias() {
        let ok_result: Result<i32> = Ok(42);
        assert!(ok_result.is_ok());
        assert_eq!(ok_result.ok(), Some(42));

        let err_result: Result<i32> = Err(HotReloadError::NotInitialized);
        assert!(err_result.is_err());
    }

    #[test]
    fn test_error_debug_impl() {
        let err = HotReloadError::CompilationError("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("CompilationError"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_compilation_error_with_multiline_message() {
        let err = HotReloadError::CompilationError("line 1: error\nline 2: details".to_string());
        let display = format!("{}", err);
        assert!(display.contains("line 1: error"));
        assert!(display.contains("line 2: details"));
    }

    #[test]
    fn test_symbol_not_found_empty_name() {
        let err = HotReloadError::SymbolNotFound(String::new());
        assert_eq!(format!("{}", err), "Symbol not found: ");
    }

    #[test]
    fn test_invalid_path_empty() {
        let err = HotReloadError::InvalidPath(String::new());
        assert_eq!(format!("{}", err), "Invalid dylib path: ");
    }
}
