//! Registry error types

use std::path::PathBuf;
use thiserror::Error;

/// Registry operation errors
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("invalid version string: {0}")]
    InvalidVersion(String),

    #[error("invalid version requirement: {0}")]
    InvalidVersionReq(String),

    #[error("package not found: {name}")]
    PackageNotFound { name: String },

    #[error("version not found: {name}@{version}")]
    VersionNotFound { name: String, version: String },

    #[error("no version of {name} satisfies requirement {req}")]
    NoMatchingVersion { name: String, req: String },

    #[error("registry not reachable: {url}")]
    RegistryUnreachable { url: String },

    #[error("HTTP error: {status} - {message}")]
    HttpError { status: u16, message: String },

    #[error("failed to read {path}: {source}")]
    ReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write {path}: {source}")]
    WriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to create directory {path}: {source}")]
    CreateDirError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("archive extraction failed: {message}")]
    ArchiveError { message: String },

    #[error("invalid package archive: {message}")]
    InvalidArchive { message: String },

    #[error("checksum mismatch for {name}@{version}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        name: String,
        version: String,
        expected: String,
        actual: String,
    },

    #[error("dependency resolution failed: {message}")]
    ResolutionError { message: String },

    #[error("cyclic dependency detected: {cycle}")]
    CyclicDependency { cycle: String },

    #[error("lock file parse error: {message}")]
    LockFileError { message: String },

    #[error("JSON parse error: {message}")]
    JsonError { message: String },

    #[error("TOML parse error: {message}")]
    TomlError { message: String },

    #[error("publish failed: {message}")]
    PublishFailed { message: String },

    #[error("authentication required: {message}")]
    AuthRequired { message: String },
}

pub type RegistryResult<T> = Result<T, RegistryError>;

impl From<serde_json::Error> for RegistryError {
    fn from(e: serde_json::Error) -> Self {
        RegistryError::JsonError {
            message: e.to_string(),
        }
    }
}

impl From<toml::de::Error> for RegistryError {
    fn from(e: toml::de::Error) -> Self {
        RegistryError::TomlError {
            message: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_version_error_message() {
        let err = RegistryError::InvalidVersion("bad version".to_string());
        let msg = err.to_string();
        assert!(msg.contains("invalid version string"));
        assert!(msg.contains("bad version"));
    }

    #[test]
    fn test_invalid_version_req_error_message() {
        let err = RegistryError::InvalidVersionReq("bad req".to_string());
        assert!(err.to_string().contains("invalid version requirement"));
    }

    #[test]
    fn test_package_not_found_error_message() {
        let err = RegistryError::PackageNotFound {
            name: "my-pkg".to_string(),
        };
        assert!(err.to_string().contains("package not found"));
        assert!(err.to_string().contains("my-pkg"));
    }

    #[test]
    fn test_version_not_found_error_message() {
        let err = RegistryError::VersionNotFound {
            name: "my-pkg".to_string(),
            version: "1.0.0".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("version not found"));
        assert!(msg.contains("my-pkg"));
        assert!(msg.contains("1.0.0"));
    }

    #[test]
    fn test_no_matching_version_error_message() {
        let err = RegistryError::NoMatchingVersion {
            name: "my-pkg".to_string(),
            req: "^2.0.0".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("no version"));
        assert!(msg.contains("my-pkg"));
        assert!(msg.contains("^2.0.0"));
    }

    #[test]
    fn test_registry_unreachable_error_message() {
        let err = RegistryError::RegistryUnreachable {
            url: "https://registry.vais.dev".to_string(),
        };
        assert!(err.to_string().contains("not reachable"));
    }

    #[test]
    fn test_http_error_message() {
        let err = RegistryError::HttpError {
            status: 404,
            message: "Not Found".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("404"));
        assert!(msg.contains("Not Found"));
    }

    #[test]
    fn test_archive_error_message() {
        let err = RegistryError::ArchiveError {
            message: "corrupt tar".to_string(),
        };
        assert!(err.to_string().contains("corrupt tar"));
    }

    #[test]
    fn test_invalid_archive_error_message() {
        let err = RegistryError::InvalidArchive {
            message: "missing vais.toml".to_string(),
        };
        assert!(err.to_string().contains("missing vais.toml"));
    }

    #[test]
    fn test_checksum_mismatch_error_message() {
        let err = RegistryError::ChecksumMismatch {
            name: "pkg".to_string(),
            version: "1.0.0".to_string(),
            expected: "abc".to_string(),
            actual: "def".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("checksum mismatch"));
        assert!(msg.contains("abc"));
        assert!(msg.contains("def"));
    }

    #[test]
    fn test_resolution_error_message() {
        let err = RegistryError::ResolutionError {
            message: "conflict".to_string(),
        };
        assert!(err.to_string().contains("resolution failed"));
    }

    #[test]
    fn test_cyclic_dependency_error_message() {
        let err = RegistryError::CyclicDependency {
            cycle: "a -> b -> a".to_string(),
        };
        assert!(err.to_string().contains("cyclic dependency"));
        assert!(err.to_string().contains("a -> b -> a"));
    }

    #[test]
    fn test_lock_file_error_message() {
        let err = RegistryError::LockFileError {
            message: "invalid syntax".to_string(),
        };
        assert!(err.to_string().contains("lock file"));
    }

    #[test]
    fn test_json_error_message() {
        let err = RegistryError::JsonError {
            message: "unexpected EOF".to_string(),
        };
        assert!(err.to_string().contains("JSON parse error"));
    }

    #[test]
    fn test_toml_error_message() {
        let err = RegistryError::TomlError {
            message: "missing field".to_string(),
        };
        assert!(err.to_string().contains("TOML parse error"));
    }

    #[test]
    fn test_publish_failed_error_message() {
        let err = RegistryError::PublishFailed {
            message: "not authorized".to_string(),
        };
        assert!(err.to_string().contains("publish failed"));
    }

    #[test]
    fn test_auth_required_error_message() {
        let err = RegistryError::AuthRequired {
            message: "token expired".to_string(),
        };
        assert!(err.to_string().contains("authentication required"));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let reg_err: RegistryError = json_err.into();
        match reg_err {
            RegistryError::JsonError { message } => {
                assert!(!message.is_empty());
            }
            _ => panic!("expected JsonError"),
        }
    }

    #[test]
    fn test_from_toml_error() {
        let toml_err = toml::from_str::<toml::Value>("[[[[invalid").unwrap_err();
        let reg_err: RegistryError = toml_err.into();
        match reg_err {
            RegistryError::TomlError { message } => {
                assert!(!message.is_empty());
            }
            _ => panic!("expected TomlError"),
        }
    }

    #[test]
    fn test_registry_result_ok() {
        let result: RegistryResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_registry_result_err() {
        let result: RegistryResult<i32> = Err(RegistryError::InvalidVersion("x".into()));
        assert!(result.is_err());
    }

    #[test]
    fn test_error_debug_format() {
        let err = RegistryError::PackageNotFound {
            name: "test".to_string(),
        };
        let debug = format!("{:?}", err);
        assert!(debug.contains("PackageNotFound"));
    }
}
