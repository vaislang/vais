//! DAP Error Types

use thiserror::Error;

/// Result type for DAP operations
pub type DapResult<T> = Result<T, DapError>;

/// DAP-specific errors
#[derive(Debug, Error)]
pub enum DapError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Session not initialized")]
    NotInitialized,

    #[error("No active debug session")]
    NoActiveSession,

    #[error("Debugger error: {0}")]
    Debugger(String),

    #[error("Breakpoint error: {0}")]
    Breakpoint(String),

    #[error("Source mapping error: {0}")]
    SourceMapping(String),

    #[error("Thread {0} not found")]
    ThreadNotFound(i64),

    #[error("Frame {0} not found")]
    FrameNotFound(i64),

    #[error("Variable reference {0} not found")]
    VariableNotFound(i64),

    #[error("Process not running")]
    ProcessNotRunning,

    #[error("DWARF parsing error: {0}")]
    DwarfParsing(String),

    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    #[error("Timeout waiting for {0}")]
    Timeout(String),
}

impl DapError {
    /// Convert error to DAP error response format
    pub fn to_error_response(&self) -> serde_json::Value {
        serde_json::json!({
            "error": {
                "id": self.error_id(),
                "format": self.to_string(),
                "showUser": self.show_to_user(),
            }
        })
    }

    fn error_id(&self) -> i32 {
        match self {
            DapError::Io(_) => 1001,
            DapError::Json(_) => 1002,
            DapError::Protocol(_) => 1003,
            DapError::InvalidRequest(_) => 1004,
            DapError::NotInitialized => 1005,
            DapError::NoActiveSession => 1006,
            DapError::Debugger(_) => 2001,
            DapError::Breakpoint(_) => 2002,
            DapError::SourceMapping(_) => 2003,
            DapError::ThreadNotFound(_) => 2004,
            DapError::FrameNotFound(_) => 2005,
            DapError::VariableNotFound(_) => 2006,
            DapError::ProcessNotRunning => 2007,
            DapError::DwarfParsing(_) => 3001,
            DapError::Unsupported(_) => 4001,
            DapError::Timeout(_) => 5001,
        }
    }

    fn show_to_user(&self) -> bool {
        matches!(
            self,
            DapError::Debugger(_)
                | DapError::Breakpoint(_)
                | DapError::ProcessNotRunning
                | DapError::Unsupported(_)
        )
    }
}
