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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[test]
    fn test_error_display() {
        assert_eq!(
            ServerError::PackageNotFound("foo".to_string()).to_string(),
            "Package not found: foo"
        );
        assert_eq!(
            ServerError::VersionNotFound("foo".to_string(), "1.0.0".to_string()).to_string(),
            "Version not found: foo@1.0.0"
        );
        assert_eq!(
            ServerError::PackageExists("bar".to_string()).to_string(),
            "Package already exists: bar"
        );
        assert_eq!(
            ServerError::VersionExists("bar".to_string(), "2.0.0".to_string()).to_string(),
            "Version already exists: bar@2.0.0"
        );
        assert_eq!(
            ServerError::InvalidPackageName("bad name".to_string()).to_string(),
            "Invalid package name: bad name"
        );
        assert_eq!(
            ServerError::InvalidVersion("xyz".to_string()).to_string(),
            "Invalid version: xyz"
        );
        assert_eq!(ServerError::InvalidChecksum.to_string(), "Invalid checksum");
        assert_eq!(ServerError::Unauthorized.to_string(), "Unauthorized");
        assert_eq!(
            ServerError::Forbidden("not owner".to_string()).to_string(),
            "Forbidden: not owner"
        );
        assert_eq!(ServerError::InvalidToken.to_string(), "Invalid token");
        assert_eq!(ServerError::TokenExpired.to_string(), "Token expired");
        assert_eq!(
            ServerError::UserNotFound("alice".to_string()).to_string(),
            "User not found: alice"
        );
        assert_eq!(
            ServerError::UserExists("bob".to_string()).to_string(),
            "User already exists: bob"
        );
        assert_eq!(
            ServerError::InvalidCredentials.to_string(),
            "Invalid credentials"
        );
        assert_eq!(
            ServerError::Archive("corrupt".to_string()).to_string(),
            "Archive error: corrupt"
        );
        assert_eq!(
            ServerError::BadRequest("missing field".to_string()).to_string(),
            "Bad request: missing field"
        );
        assert_eq!(
            ServerError::Internal("oops".to_string()).to_string(),
            "Internal error: oops"
        );
    }

    #[test]
    fn test_error_into_response_not_found() {
        let err = ServerError::PackageNotFound("test".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_error_into_response_conflict() {
        let err = ServerError::PackageExists("test".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_error_into_response_version_conflict() {
        let err = ServerError::VersionExists("pkg".to_string(), "1.0".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_error_into_response_bad_request() {
        let err = ServerError::InvalidPackageName("x".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let err = ServerError::InvalidVersion("x".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let err = ServerError::InvalidChecksum;
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let err = ServerError::Archive("bad".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let err = ServerError::BadRequest("x".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_into_response_unauthorized() {
        let err = ServerError::Unauthorized;
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let err = ServerError::InvalidToken;
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let err = ServerError::TokenExpired;
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let err = ServerError::InvalidCredentials;
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_error_into_response_forbidden() {
        let err = ServerError::Forbidden("no access".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_error_into_response_user_not_found() {
        let err = ServerError::UserNotFound("x".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_error_into_response_user_exists() {
        let err = ServerError::UserExists("x".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_error_into_response_internal() {
        let err = ServerError::Internal("boom".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let err: ServerError = io_err.into();
        assert!(matches!(err, ServerError::Io(_)));
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_from_json() {
        let json_err = serde_json::from_str::<i32>("bad").unwrap_err();
        let err: ServerError = json_err.into();
        assert!(matches!(err, ServerError::Json(_)));
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_version_not_found_response() {
        let err = ServerError::VersionNotFound("pkg".to_string(), "3.0.0".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
