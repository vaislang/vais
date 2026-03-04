//! SemVer version resolution for the package registry
//!
//! Implements proper Semantic Versioning 2.0.0 negotiation when resolving
//! dependency version requirements against available versions in the registry.

use semver::{Version, VersionReq};

/// Find the best matching version from a list of available versions
/// given a version requirement string.
///
/// Returns the highest compatible version that satisfies the requirement,
/// preferring non-prerelease versions when possible.
pub fn resolve_best_version(
    version_req_str: &str,
    available_versions: &[&str],
) -> Option<String> {
    let req = parse_version_req(version_req_str)?;

    let mut candidates: Vec<Version> = available_versions
        .iter()
        .filter_map(|v| Version::parse(v).ok())
        .filter(|v| req.matches(v))
        .collect();

    // Sort descending (highest first)
    candidates.sort_by(|a, b| b.cmp(a));

    // Prefer non-prerelease versions
    if let Some(stable) = candidates.iter().find(|v| v.pre.is_empty()) {
        return Some(stable.to_string());
    }

    // Fall back to highest prerelease
    candidates.first().map(|v| v.to_string())
}

/// Parse a version requirement string, handling Vais-specific operators.
///
/// Supports:
/// - `^1.2.3` (caret/compatible) - default
/// - `~1.2.3` (tilde/approximately)
/// - `>=1.0, <2.0` (range)
/// - `=1.2.3` (exact)
/// - `*` (any)
/// - `1.2.3` (treated as `^1.2.3`)
pub fn parse_version_req(req_str: &str) -> Option<VersionReq> {
    let trimmed = req_str.trim();

    if trimmed.is_empty() || trimmed == "*" {
        return VersionReq::parse("*").ok();
    }

    // If it looks like a bare version (no operator), treat as caret
    if trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        // Check if it's a valid version first
        if Version::parse(trimmed).is_ok() {
            return VersionReq::parse(&format!("^{}", trimmed)).ok();
        }
        // Might be a partial version like "1.2" or "1"
        return VersionReq::parse(&format!("^{}", trimmed)).ok();
    }

    VersionReq::parse(trimmed).ok()
}

/// Check if two version requirements are compatible (could have a common solution).
///
/// This is a heuristic check for detecting potential conflicts early.
pub fn are_compatible(req_a: &str, req_b: &str) -> bool {
    let a = match parse_version_req(req_a) {
        Some(r) => r,
        None => return false,
    };
    let b = match parse_version_req(req_b) {
        Some(r) => r,
        None => return false,
    };

    // Generate some test versions and check if any satisfy both
    let test_versions = generate_test_versions();
    test_versions
        .iter()
        .any(|v| a.matches(v) && b.matches(v))
}

/// Compare two semver strings, returning the ordering.
/// Returns None if either string is not a valid semver.
pub fn compare_versions(a: &str, b: &str) -> Option<std::cmp::Ordering> {
    let va = Version::parse(a).ok()?;
    let vb = Version::parse(b).ok()?;
    Some(va.cmp(&vb))
}

/// Check if a version satisfies a version requirement
pub fn satisfies(version: &str, requirement: &str) -> bool {
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

/// Generate a set of test versions for compatibility checking.
///
/// Covers major versions 0-20 with select minor/patch combinations to provide
/// reasonable coverage without excessive iteration.
///
/// NOTE: This heuristic is only used for `are_compatible()` which checks if
/// two version requirements can be simultaneously satisfied. For exact
/// resolution, use `resolve_best_version()` which operates on real version lists.
fn generate_test_versions() -> Vec<Version> {
    let mut versions = Vec::new();
    for major in 0..20 {
        for minor in 0..5 {
            for patch in 0..3 {
                if let Ok(v) = Version::parse(&format!("{}.{}.{}", major, minor, patch)) {
                    versions.push(v);
                }
            }
        }
    }
    versions
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== resolve_best_version tests ==========

    #[test]
    fn test_resolve_caret_requirement() {
        let versions = vec!["1.0.0", "1.1.0", "1.2.0", "2.0.0"];
        let result = resolve_best_version("^1.0.0", &versions);
        assert_eq!(result, Some("1.2.0".to_string()));
    }

    #[test]
    fn test_resolve_tilde_requirement() {
        let versions = vec!["1.0.0", "1.0.5", "1.1.0", "2.0.0"];
        let result = resolve_best_version("~1.0.0", &versions);
        assert_eq!(result, Some("1.0.5".to_string()));
    }

    #[test]
    fn test_resolve_exact_requirement() {
        let versions = vec!["1.0.0", "1.1.0", "1.2.0"];
        let result = resolve_best_version("=1.1.0", &versions);
        assert_eq!(result, Some("1.1.0".to_string()));
    }

    #[test]
    fn test_resolve_wildcard() {
        let versions = vec!["1.0.0", "2.0.0", "3.0.0"];
        let result = resolve_best_version("*", &versions);
        assert_eq!(result, Some("3.0.0".to_string()));
    }

    #[test]
    fn test_resolve_no_match() {
        let versions = vec!["1.0.0", "1.1.0"];
        let result = resolve_best_version("^2.0.0", &versions);
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_empty_versions() {
        let versions: Vec<&str> = vec![];
        let result = resolve_best_version("^1.0.0", &versions);
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_bare_version_as_caret() {
        let versions = vec!["1.0.0", "1.5.0", "2.0.0"];
        let result = resolve_best_version("1.0.0", &versions);
        // Bare "1.0.0" -> "^1.0.0" -> matches 1.0.0 and 1.5.0
        assert_eq!(result, Some("1.5.0".to_string()));
    }

    #[test]
    fn test_resolve_prefers_stable() {
        let versions = vec!["1.0.0", "1.1.0-alpha", "1.1.0"];
        let result = resolve_best_version("^1.0.0", &versions);
        assert_eq!(result, Some("1.1.0".to_string()));
    }

    #[test]
    fn test_resolve_falls_back_to_prerelease() {
        let versions = vec!["2.0.0-alpha", "2.0.0-beta"];
        let result = resolve_best_version(">=2.0.0-alpha", &versions);
        assert_eq!(result, Some("2.0.0-beta".to_string()));
    }

    #[test]
    fn test_resolve_range_requirement() {
        let versions = vec!["1.0.0", "1.5.0", "2.0.0", "2.5.0"];
        let result = resolve_best_version(">=1.0.0, <2.0.0", &versions);
        assert_eq!(result, Some("1.5.0".to_string()));
    }

    // ========== parse_version_req tests ==========

    #[test]
    fn test_parse_caret() {
        let req = parse_version_req("^1.2.3");
        assert!(req.is_some());
    }

    #[test]
    fn test_parse_tilde() {
        let req = parse_version_req("~1.2.3");
        assert!(req.is_some());
    }

    #[test]
    fn test_parse_exact() {
        let req = parse_version_req("=1.2.3");
        assert!(req.is_some());
    }

    #[test]
    fn test_parse_range() {
        let req = parse_version_req(">=1.0, <2.0");
        assert!(req.is_some());
    }

    #[test]
    fn test_parse_wildcard() {
        let req = parse_version_req("*");
        assert!(req.is_some());
    }

    #[test]
    fn test_parse_empty() {
        let req = parse_version_req("");
        assert!(req.is_some()); // treated as "*"
    }

    #[test]
    fn test_parse_bare_version() {
        let req = parse_version_req("1.2.3");
        assert!(req.is_some());
        // Should match 1.2.3 and 1.3.0 but not 2.0.0
        let v123 = Version::parse("1.2.3").unwrap();
        let v130 = Version::parse("1.3.0").unwrap();
        let v200 = Version::parse("2.0.0").unwrap();
        let r = req.unwrap();
        assert!(r.matches(&v123));
        assert!(r.matches(&v130));
        assert!(!r.matches(&v200));
    }

    // ========== are_compatible tests ==========

    #[test]
    fn test_compatible_overlapping() {
        assert!(are_compatible("^1.0.0", "^1.2.0"));
    }

    #[test]
    fn test_incompatible_disjoint() {
        assert!(!are_compatible("^1.0.0", "^2.0.0"));
    }

    #[test]
    fn test_compatible_wildcard() {
        assert!(are_compatible("*", "^1.0.0"));
    }

    #[test]
    fn test_incompatible_invalid() {
        assert!(!are_compatible("not_a_version", "^1.0.0"));
    }

    // ========== compare_versions tests ==========

    #[test]
    fn test_compare_equal() {
        assert_eq!(
            compare_versions("1.0.0", "1.0.0"),
            Some(std::cmp::Ordering::Equal)
        );
    }

    #[test]
    fn test_compare_less() {
        assert_eq!(
            compare_versions("1.0.0", "2.0.0"),
            Some(std::cmp::Ordering::Less)
        );
    }

    #[test]
    fn test_compare_greater() {
        assert_eq!(
            compare_versions("2.0.0", "1.0.0"),
            Some(std::cmp::Ordering::Greater)
        );
    }

    #[test]
    fn test_compare_minor() {
        assert_eq!(
            compare_versions("1.0.0", "1.1.0"),
            Some(std::cmp::Ordering::Less)
        );
    }

    #[test]
    fn test_compare_patch() {
        assert_eq!(
            compare_versions("1.0.0", "1.0.1"),
            Some(std::cmp::Ordering::Less)
        );
    }

    #[test]
    fn test_compare_prerelease() {
        assert_eq!(
            compare_versions("1.0.0-alpha", "1.0.0"),
            Some(std::cmp::Ordering::Less)
        );
    }

    #[test]
    fn test_compare_invalid() {
        assert!(compare_versions("not_valid", "1.0.0").is_none());
    }

    // ========== satisfies tests ==========

    #[test]
    fn test_satisfies_basic() {
        assert!(satisfies("1.2.3", "^1.0.0"));
        assert!(!satisfies("2.0.0", "^1.0.0"));
    }

    #[test]
    fn test_satisfies_exact() {
        assert!(satisfies("1.2.3", "=1.2.3"));
        assert!(!satisfies("1.2.4", "=1.2.3"));
    }

    #[test]
    fn test_satisfies_wildcard() {
        assert!(satisfies("99.99.99", "*"));
    }

    #[test]
    fn test_satisfies_invalid_version() {
        assert!(!satisfies("not_valid", "^1.0.0"));
    }

    #[test]
    fn test_satisfies_invalid_requirement() {
        assert!(!satisfies("1.0.0", "totally invalid!!!"));
    }
}
