//! Error types for the registry server

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Result type alias for server operations
pub type ServerResult<T> = Result<T, ServerError>;

/// Server error types
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Version not found: {0}@{1}")]
    VersionNotFound(String, String),

    #[error("Package already exists: {0}")]
    PackageExists(String),

    #[error("Version already exists: {0}@{1}")]
    VersionExists(String, String),

    #[error("Invalid package name: {0}")]
    InvalidPackageName(String),

    #[error("Invalid version: {0}")]
    InvalidVersion(String),

    #[error("Invalid checksum")]
    InvalidChecksum,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("User already exists: {0}")]
    UserExists(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Archive error: {0}")]
    Archive(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            ServerError::PackageNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ServerError::VersionNotFound(_, _) => (StatusCode::NOT_FOUND, self.to_string()),
            ServerError::PackageExists(_) => (StatusCode::CONFLICT, self.to_string()),
            ServerError::VersionExists(_, _) => (StatusCode::CONFLICT, self.to_string()),
            ServerError::InvalidPackageName(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServerError::InvalidVersion(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServerError::InvalidChecksum => (StatusCode::BAD_REQUEST, self.to_string()),
            ServerError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            ServerError::Forbidden(_) => (StatusCode::FORBIDDEN, self.to_string()),
            ServerError::InvalidToken => (StatusCode::UNAUTHORIZED, self.to_string()),
            ServerError::TokenExpired => (StatusCode::UNAUTHORIZED, self.to_string()),
            ServerError::UserNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ServerError::UserExists(_) => (StatusCode::CONFLICT, self.to_string()),
            ServerError::InvalidCredentials => (StatusCode::UNAUTHORIZED, self.to_string()),
            ServerError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
            ServerError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error".to_string()),
            ServerError::Json(_) => (StatusCode::BAD_REQUEST, "Invalid JSON".to_string()),
            ServerError::Archive(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServerError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServerError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            ),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}
