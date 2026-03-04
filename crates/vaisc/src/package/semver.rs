//! SemVer version negotiation for package dependency resolution (client-side)
//!
//! Provides proper Semantic Versioning 2.0.0 support for resolving
//! dependency version requirements against cached or registry versions.
//!
//! NOTE: The registry server has a parallel implementation in
//! `vais-registry-server::semver_resolve` for server-side resolution.
//! Both use the `semver` crate as the canonical parsing layer, so behavior
//! is consistent. If a shared `vais-semver` crate is extracted in the future,
//! both modules should delegate to it.

#![allow(dead_code)]

use super::types::PackageError;
use semver::{Version, VersionReq};
use std::path::{Path, PathBuf};

/// A resolved version with its path in the cache
#[derive(Debug, Clone)]
pub struct ResolvedVersion {
    pub version: String,
    pub path: PathBuf,
}

/// Resolve the best matching cached version for a dependency.
///
/// Scans `<cache_root>/cache/<name>/` for version directories, parses
/// them as semver, and picks the highest version satisfying `version_req`.
pub fn resolve_cached_version(
    cache_root: &Path,
    name: &str,
    version_req_str: &str,
) -> Result<ResolvedVersion, PackageError> {
    let pkg_cache_dir = cache_root.join("cache").join(name);
    if !pkg_cache_dir.exists() {
        return Err(PackageError::RegistryDepNotInstalled {
            name: name.to_string(),
            version: version_req_str.to_string(),
        });
    }

    let req = parse_version_req(version_req_str).ok_or_else(|| {
        PackageError::ParseError {
            path: pkg_cache_dir.clone(),
            message: format!("invalid version requirement: {}", version_req_str),
        }
    })?;

    // Collect all cached versions
    let mut candidates: Vec<(Version, PathBuf)> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&pkg_cache_dir) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if !entry_path.is_dir() {
                continue;
            }
            let extracted = entry_path.join("extracted");
            if !extracted.exists() {
                continue;
            }
            if let Some(dir_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                if let Ok(version) = Version::parse(dir_name) {
                    if req.matches(&version) {
                        candidates.push((version, extracted));
                    }
                }
            }
        }
    }

    if candidates.is_empty() {
        return Err(PackageError::RegistryDepNotInstalled {
            name: name.to_string(),
            version: version_req_str.to_string(),
        });
    }

    // Sort descending and prefer non-prerelease
    candidates.sort_by(|a, b| b.0.cmp(&a.0));

    // Pick highest stable, then fallback to highest prerelease
    let (version, path) = candidates
        .iter()
        .find(|(v, _)| v.pre.is_empty())
        .or_else(|| candidates.first())
        .cloned()
        .unwrap(); // safe: candidates is non-empty

    Ok(ResolvedVersion {
        version: version.to_string(),
        path,
    })
}

/// Parse a version requirement string with Vais conventions.
///
/// Bare version strings like "1.2.3" are treated as "^1.2.3".
/// "*" and "" are treated as wildcard.
fn parse_version_req(req_str: &str) -> Option<VersionReq> {
    let trimmed = req_str.trim();

    if trimmed.is_empty() || trimmed == "*" {
        return VersionReq::parse("*").ok();
    }

    // Bare version -> caret
    if trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        if Version::parse(trimmed).is_ok() {
            return VersionReq::parse(&format!("^{}", trimmed)).ok();
        }
        return VersionReq::parse(&format!("^{}", trimmed)).ok();
    }

    VersionReq::parse(trimmed).ok()
}

/// Check if a version string satisfies a requirement string
pub fn version_satisfies(version: &str, requirement: &str) -> bool {
    let v = match Version::parse(version) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let req = match parse_version_req(requirement) {
        Some(r) => r,
        None => return false,
    };
    req.matches(&v)
}

/// Compare two semver version strings.
/// Returns None if either is invalid.
pub fn compare_versions(a: &str, b: &str) -> Option<std::cmp::Ordering> {
    let va = Version::parse(a).ok()?;
    let vb = Version::parse(b).ok()?;
    Some(va.cmp(&vb))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_cache(
        tmp: &Path,
        name: &str,
        versions: &[&str],
    ) -> PathBuf {
        let cache_root = tmp.join("registry");
        for ver in versions {
            let extracted = cache_root.join("cache").join(name).join(ver).join("extracted");
            fs::create_dir_all(&extracted).unwrap();
            // Create a minimal vais.toml
            fs::write(
                extracted.join("vais.toml"),
                format!(
                    "[package]\nname = \"{}\"\nversion = \"{}\"",
                    name, ver
                ),
            )
            .unwrap();
        }
        cache_root
    }

    #[test]
    fn test_resolve_caret() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache_root = setup_cache(tmp.path(), "foo", &["1.0.0", "1.2.0", "2.0.0"]);

        let result = resolve_cached_version(&cache_root, "foo", "^1.0.0").unwrap();
        assert_eq!(result.version, "1.2.0");
    }

    #[test]
    fn test_resolve_tilde() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache_root = setup_cache(tmp.path(), "bar", &["1.0.0", "1.0.5", "1.1.0"]);

        let result = resolve_cached_version(&cache_root, "bar", "~1.0.0").unwrap();
        assert_eq!(result.version, "1.0.5");
    }

    #[test]
    fn test_resolve_exact() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache_root = setup_cache(tmp.path(), "baz", &["1.0.0", "1.1.0"]);

        let result = resolve_cached_version(&cache_root, "baz", "=1.0.0").unwrap();
        assert_eq!(result.version, "1.0.0");
    }

    #[test]
    fn test_resolve_wildcard() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache_root = setup_cache(tmp.path(), "qux", &["1.0.0", "2.0.0", "3.0.0"]);

        let result = resolve_cached_version(&cache_root, "qux", "*").unwrap();
        assert_eq!(result.version, "3.0.0");
    }

    #[test]
    fn test_resolve_no_match() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache_root = setup_cache(tmp.path(), "nope", &["1.0.0"]);

        let result = resolve_cached_version(&cache_root, "nope", "^2.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_not_installed() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache_root = tmp.path().join("registry");
        fs::create_dir_all(&cache_root).unwrap();

        let result = resolve_cached_version(&cache_root, "missing", "^1.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_bare_version() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache_root = setup_cache(tmp.path(), "pkg", &["1.0.0", "1.5.0", "2.0.0"]);

        let result = resolve_cached_version(&cache_root, "pkg", "1.0.0").unwrap();
        assert_eq!(result.version, "1.5.0"); // ^1.0.0 semantics
    }

    #[test]
    fn test_version_satisfies() {
        assert!(version_satisfies("1.2.3", "^1.0.0"));
        assert!(!version_satisfies("2.0.0", "^1.0.0"));
        assert!(version_satisfies("1.2.3", "=1.2.3"));
        assert!(!version_satisfies("1.2.4", "=1.2.3"));
        assert!(version_satisfies("99.0.0", "*"));
    }

    #[test]
    fn test_version_satisfies_invalid() {
        assert!(!version_satisfies("not_valid", "^1.0.0"));
        assert!(!version_satisfies("1.0.0", "totally_invalid!!!"));
    }

    #[test]
    fn test_compare_versions() {
        assert_eq!(
            compare_versions("1.0.0", "2.0.0"),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            compare_versions("2.0.0", "1.0.0"),
            Some(std::cmp::Ordering::Greater)
        );
        assert_eq!(
            compare_versions("1.0.0", "1.0.0"),
            Some(std::cmp::Ordering::Equal)
        );
        assert!(compare_versions("bad", "1.0.0").is_none());
    }

    #[test]
    fn test_parse_version_req_variants() {
        assert!(parse_version_req("^1.2.3").is_some());
        assert!(parse_version_req("~1.2.3").is_some());
        assert!(parse_version_req("=1.2.3").is_some());
        assert!(parse_version_req(">=1.0, <2.0").is_some());
        assert!(parse_version_req("*").is_some());
        assert!(parse_version_req("").is_some());
        assert!(parse_version_req("1.2.3").is_some());
    }
}
