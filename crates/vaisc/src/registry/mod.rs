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

pub use cache::PackageCache;
pub use client::RegistryClient;
pub use lockfile::LockFile;
pub use resolver::DependencyResolver;
pub use source::RegistrySource;
pub use vulnerability::VulnerabilityScanner;
