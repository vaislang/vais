//! Package manager for Vais
//!
//! Handles vais.toml parsing, dependency resolution, and package builds.

// Submodules
mod types;
mod features;
mod workspace;
mod manifest;
mod resolution;

#[cfg(test)]
mod tests;

// Re-export public types and functions
pub use types::{
    BuildConfig, Dependency, DetailedDependency, NativeDependency,
    PackageError, PackageInfo, PackageManifest, PackageResult, ResolvedDependency,
};

pub use features::FeatureConfig;

pub use workspace::{
    find_workspace_root, resolve_inter_workspace_deps, resolve_workspace_members,
    WorkspaceConfig,
};

pub use manifest::{add_dependency, find_manifest, init_package, load_manifest, remove_dependency};

pub use resolution::{
    default_registry_cache_root, find_cached_registry_dep, resolve_all_dependencies,
};
