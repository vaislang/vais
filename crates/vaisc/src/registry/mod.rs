//! Package registry system for Vais
//!
//! This module provides functionality for:
//! - Semantic versioning (SemVer) parsing and comparison
//! - Package registry client (HTTP and local)
//! - Package caching and extraction
//! - Dependency resolution with lock files

mod archive;
mod cache;
mod client;
mod error;
mod index;
mod lockfile;
mod resolver;
mod source;
mod version;
mod vulnerability;

pub use archive::{pack_package, unpack_package};
pub use cache::PackageCache;
pub use client::RegistryClient;
pub use error::{RegistryError, RegistryResult};
pub use index::{PackageIndex, PackageMetadata, VersionEntry};
pub use lockfile::{LockFile, LockedPackage};
pub use resolver::{DependencyResolver, ResolvedPackage};
pub use source::{RegistryConfig, RegistrySource};
pub use version::{Version, VersionReq};
pub use vulnerability::{VulnerabilityScanner, Vulnerability};
