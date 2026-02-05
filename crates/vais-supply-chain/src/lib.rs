pub mod audit;
pub mod sbom;
pub mod signing;

#[cfg(test)]
mod tests;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SupplyChainError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid manifest format: {0}")]
    InvalidManifest(String),

    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),

    #[error("Audit error: {0}")]
    AuditError(String),
}

pub type Result<T> = std::result::Result<T, SupplyChainError>;

// Re-exports for convenience
pub use audit::{AuditResult, DependencyAuditor, VulnerabilitySeverity};
pub use sbom::{SbomComponent, SbomDocument, SbomGenerator};
pub use signing::{PackageSignature, PackageSigner};
