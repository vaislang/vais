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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert_eq!(
            DapError::Protocol("bad msg".to_string()).to_string(),
            "Protocol error: bad msg"
        );
        assert_eq!(
            DapError::InvalidRequest("missing field".to_string()).to_string(),
            "Invalid request: missing field"
        );
        assert_eq!(
            DapError::NotInitialized.to_string(),
            "Session not initialized"
        );
        assert_eq!(
            DapError::NoActiveSession.to_string(),
            "No active debug session"
        );
        assert_eq!(
            DapError::ProcessNotRunning.to_string(),
            "Process not running"
        );
        assert_eq!(
            DapError::ThreadNotFound(42).to_string(),
            "Thread 42 not found"
        );
        assert_eq!(DapError::FrameNotFound(7).to_string(), "Frame 7 not found");
        assert_eq!(
            DapError::VariableNotFound(99).to_string(),
            "Variable reference 99 not found"
        );
        assert_eq!(
            DapError::DwarfParsing("corrupt".to_string()).to_string(),
            "DWARF parsing error: corrupt"
        );
        assert_eq!(
            DapError::Unsupported("feature X".to_string()).to_string(),
            "Unsupported operation: feature X"
        );
        assert_eq!(
            DapError::Timeout("response".to_string()).to_string(),
            "Timeout waiting for response"
        );
    }

    #[test]
    fn test_error_ids_unique() {
        let errors: Vec<DapError> = vec![
            DapError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test")),
            DapError::Json(serde_json::from_str::<i32>("bad").unwrap_err()),
            DapError::Protocol("".to_string()),
            DapError::InvalidRequest("".to_string()),
            DapError::NotInitialized,
            DapError::NoActiveSession,
            DapError::Debugger("".to_string()),
            DapError::Breakpoint("".to_string()),
            DapError::SourceMapping("".to_string()),
            DapError::ThreadNotFound(0),
            DapError::FrameNotFound(0),
            DapError::VariableNotFound(0),
            DapError::ProcessNotRunning,
            DapError::DwarfParsing("".to_string()),
            DapError::Unsupported("".to_string()),
            DapError::Timeout("".to_string()),
        ];

        let ids: Vec<i32> = errors.iter().map(|e| e.error_id()).collect();
        let mut unique = ids.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(ids.len(), unique.len(), "Error IDs must be unique");
    }

    #[test]
    fn test_error_response_format() {
        let err = DapError::Debugger("crash".to_string());
        let resp = err.to_error_response();
        let error_obj = resp.get("error").unwrap();
        assert_eq!(error_obj.get("id").unwrap().as_i64().unwrap(), 2001);
        assert!(error_obj
            .get("format")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("crash"));
        assert!(error_obj.get("showUser").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_show_to_user() {
        // These should show to user
        assert!(DapError::Debugger("err".to_string()).show_to_user());
        assert!(DapError::Breakpoint("err".to_string()).show_to_user());
        assert!(DapError::ProcessNotRunning.show_to_user());
        assert!(DapError::Unsupported("x".to_string()).show_to_user());

        // These should NOT show to user
        assert!(!DapError::Protocol("err".to_string()).show_to_user());
        assert!(!DapError::NotInitialized.show_to_user());
        assert!(!DapError::NoActiveSession.show_to_user());
        assert!(!DapError::ThreadNotFound(1).show_to_user());
        assert!(!DapError::FrameNotFound(1).show_to_user());
        assert!(!DapError::VariableNotFound(1).show_to_user());
        assert!(!DapError::DwarfParsing("x".to_string()).show_to_user());
        assert!(!DapError::Timeout("x".to_string()).show_to_user());
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let dap_err: DapError = io_err.into();
        assert!(matches!(dap_err, DapError::Io(_)));
        assert!(dap_err.to_string().contains("file not found"));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let dap_err: DapError = json_err.into();
        assert!(matches!(dap_err, DapError::Json(_)));
    }
}
