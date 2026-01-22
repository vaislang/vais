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
