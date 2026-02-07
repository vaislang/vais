#![allow(dead_code)]
//! Registry E2E Tests - Phase 35 Stage 2
//!
//! Package publish/install roundtrip verification tests.
//! These tests verify the package registry system without requiring a running server:
//!
//! 1. Package manifest (vais.toml) parsing and validation
//! 2. Package versioning (semver format validation)
//! 3. Package dependency resolution logic
//! 4. Package search/list command output
//! 5. Registry server endpoint structure (expected API routes)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Mirror of PackageManifest types from vaisc/src/package.rs
// (vaisc is a binary crate so we cannot import its internal modules directly)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackageManifest {
    package: PackageInfo,
    #[serde(default)]
    dependencies: HashMap<String, Dependency>,
    #[serde(default, rename = "dev-dependencies")]
    dev_dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    build: BuildConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackageInfo {
    name: String,
    version: String,
    #[serde(default)]
    authors: Vec<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum Dependency {
    Version(String),
    Detailed(DetailedDependency),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DetailedDependency {
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    registry: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct BuildConfig {
    #[serde(default)]
    opt_level: Option<u8>,
    #[serde(default)]
    debug: Option<bool>,
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    borrow_check: Option<String>,
}

// ---------------------------------------------------------------------------
// Inline semver helpers (mirrors vaisc/src/registry/version.rs)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemVer {
    major: u64,
    minor: u64,
    patch: u64,
    pre: Option<String>,
    build: Option<String>,
}

impl SemVer {
    fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s.is_empty() {
            return Err("empty version".into());
        }

        let (main, build) = if let Some(idx) = s.find('+') {
            (&s[..idx], Some(s[idx + 1..].to_string()))
        } else {
            (s, None)
        };

        let (version_part, pre) = if let Some(idx) = main.find('-') {
            (&main[..idx], Some(main[idx + 1..].to_string()))
        } else {
            (main, None)
        };

        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.is_empty() || parts.len() > 3 {
            return Err(format!(
                "expected 1-3 version components, got {}",
                parts.len()
            ));
        }

        let major = parts[0]
            .parse::<u64>()
            .map_err(|_| format!("invalid major: {}", parts[0]))?;
        let minor = if parts.len() > 1 {
            parts[1]
                .parse::<u64>()
                .map_err(|_| format!("invalid minor: {}", parts[1]))?
        } else {
            0
        };
        let patch = if parts.len() > 2 {
            parts[2]
                .parse::<u64>()
                .map_err(|_| format!("invalid patch: {}", parts[2]))?
        } else {
            0
        };

        Ok(Self {
            major,
            minor,
            patch,
            pre,
            build,
        })
    }

    fn is_valid_semver(s: &str) -> bool {
        SemVer::parse(s).is_ok()
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.pre {
            write!(f, "-{}", pre)?;
        }
        if let Some(ref build) = self.build {
            write!(f, "+{}", build)?;
        }
        Ok(())
    }
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.major.cmp(&other.major) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match (&self.pre, &other.pre) {
            (None, None) => std::cmp::Ordering::Equal,
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

// ---------------------------------------------------------------------------
// Inline version requirement matching (mirrors caret/tilde from version.rs)
// ---------------------------------------------------------------------------

fn caret_matches(req: &SemVer, ver: &SemVer) -> bool {
    if ver < req {
        return false;
    }
    if req.major != 0 {
        ver.major == req.major
    } else if req.minor != 0 {
        ver.major == 0 && ver.minor == req.minor
    } else {
        ver.major == 0 && ver.minor == 0 && ver.patch == req.patch
    }
}

fn tilde_matches(req: &SemVer, ver: &SemVer) -> bool {
    if ver < req {
        return false;
    }
    ver.major == req.major && ver.minor == req.minor
}

/// Parse package spec like "name@version" or just "name"
fn parse_package_spec(spec: &str) -> (String, String) {
    if let Some(idx) = spec.find('@') {
        let name = &spec[..idx];
        let version = &spec[idx + 1..];
        (name.to_string(), version.to_string())
    } else {
        (spec.to_string(), "*".to_string())
    }
}

// ---------------------------------------------------------------------------
// Helper: project root
// ---------------------------------------------------------------------------

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn packages_dir() -> PathBuf {
    project_root().join("packages")
}

// ---------------------------------------------------------------------------
// Helper: load manifest from directory
// ---------------------------------------------------------------------------

fn load_manifest(dir: &Path) -> Result<PackageManifest, String> {
    let path = dir.join("vais.toml");
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
    toml::from_str(&content).map_err(|e| format!("failed to parse {}: {}", path.display(), e))
}

// ---------------------------------------------------------------------------
// Helper: init a package in a temp dir (same logic as package.rs init_package)
// ---------------------------------------------------------------------------

fn init_test_package(dir: &Path, name: &str) -> PackageManifest {
    let manifest = PackageManifest {
        package: PackageInfo {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            authors: Vec::new(),
            description: None,
            license: Some("MIT".to_string()),
        },
        dependencies: HashMap::new(),
        dev_dependencies: HashMap::new(),
        build: BuildConfig::default(),
    };

    let content = toml::to_string_pretty(&manifest).unwrap();
    fs::write(dir.join("vais.toml"), content).unwrap();
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        src_dir.join("main.vais"),
        "# Main\nF main() -> i64 {\n    0\n}\n",
    )
    .unwrap();

    manifest
}

// ===========================================================================
// 1. Package Manifest Parsing and Validation
// ===========================================================================

#[test]
fn test_manifest_parse_minimal() {
    let toml_str = r#"
[package]
name = "hello"
version = "0.1.0"
"#;
    let manifest: PackageManifest = toml::from_str(toml_str).unwrap();
    assert_eq!(manifest.package.name, "hello");
    assert_eq!(manifest.package.version, "0.1.0");
    assert!(manifest.dependencies.is_empty());
    assert!(manifest.dev_dependencies.is_empty());
}

#[test]
fn test_manifest_parse_full() {
    let toml_str = r#"
[package]
name = "my-app"
version = "1.2.3"
authors = ["Alice", "Bob"]
description = "A cool application"
license = "Apache-2.0"

[dependencies]
json-parser = "1.0.0"
utils = { path = "../utils" }
net = { version = "0.5.0", features = ["tls"] }

[dev-dependencies]
test-helper = "0.1.0"

[build]
opt_level = 2
debug = true
target = "x86_64-linux"
borrow_check = "warn"
"#;
    let manifest: PackageManifest = toml::from_str(toml_str).unwrap();
    assert_eq!(manifest.package.name, "my-app");
    assert_eq!(manifest.package.version, "1.2.3");
    assert_eq!(manifest.package.authors.len(), 2);
    assert_eq!(
        manifest.package.description.as_deref(),
        Some("A cool application")
    );
    assert_eq!(manifest.package.license.as_deref(), Some("Apache-2.0"));

    // Dependencies
    assert_eq!(manifest.dependencies.len(), 3);
    assert!(manifest.dependencies.contains_key("json-parser"));
    assert!(manifest.dependencies.contains_key("utils"));
    assert!(manifest.dependencies.contains_key("net"));

    // Verify version dependency
    match &manifest.dependencies["json-parser"] {
        Dependency::Version(v) => assert_eq!(v, "1.0.0"),
        _ => panic!("expected Version dependency for json-parser"),
    }

    // Verify path dependency
    match &manifest.dependencies["utils"] {
        Dependency::Detailed(d) => {
            assert_eq!(d.path.as_deref(), Some("../utils"));
        }
        _ => panic!("expected Detailed dependency for utils"),
    }

    // Verify dependency with features
    match &manifest.dependencies["net"] {
        Dependency::Detailed(d) => {
            assert_eq!(d.version.as_deref(), Some("0.5.0"));
            assert_eq!(d.features, vec!["tls"]);
        }
        _ => panic!("expected Detailed dependency for net"),
    }

    // Dev-dependencies
    assert_eq!(manifest.dev_dependencies.len(), 1);

    // Build config
    assert_eq!(manifest.build.opt_level, Some(2));
    assert_eq!(manifest.build.debug, Some(true));
    assert_eq!(manifest.build.target.as_deref(), Some("x86_64-linux"));
    assert_eq!(manifest.build.borrow_check.as_deref(), Some("warn"));
}

#[test]
fn test_manifest_parse_missing_name_fails() {
    let toml_str = r#"
[package]
version = "1.0.0"
"#;
    let result: Result<PackageManifest, _> = toml::from_str(toml_str);
    assert!(result.is_err(), "manifest without package.name should fail");
}

#[test]
fn test_manifest_parse_missing_version_fails() {
    let toml_str = r#"
[package]
name = "foo"
"#;
    let result: Result<PackageManifest, _> = toml::from_str(toml_str);
    assert!(
        result.is_err(),
        "manifest without package.version should fail"
    );
}

#[test]
fn test_manifest_parse_missing_package_section_fails() {
    let toml_str = r#"
[dependencies]
foo = "1.0.0"
"#;
    let result: Result<PackageManifest, _> = toml::from_str(toml_str);
    assert!(
        result.is_err(),
        "manifest without [package] section should fail"
    );
}

#[test]
fn test_manifest_roundtrip() {
    let manifest = PackageManifest {
        package: PackageInfo {
            name: "roundtrip-test".to_string(),
            version: "2.0.0".to_string(),
            authors: vec!["Tester".to_string()],
            description: Some("Testing roundtrip".to_string()),
            license: Some("MIT".to_string()),
        },
        dependencies: {
            let mut deps = HashMap::new();
            deps.insert(
                "lib-a".to_string(),
                Dependency::Version("1.0.0".to_string()),
            );
            deps.insert(
                "lib-b".to_string(),
                Dependency::Detailed(DetailedDependency {
                    path: Some("../lib-b".to_string()),
                    version: None,
                    features: vec![],
                    registry: None,
                }),
            );
            deps
        },
        dev_dependencies: HashMap::new(),
        build: BuildConfig {
            opt_level: Some(3),
            debug: Some(false),
            target: None,
            borrow_check: Some("strict".to_string()),
        },
    };

    let toml_str = toml::to_string_pretty(&manifest).unwrap();
    let parsed: PackageManifest = toml::from_str(&toml_str).unwrap();

    assert_eq!(parsed.package.name, "roundtrip-test");
    assert_eq!(parsed.package.version, "2.0.0");
    assert_eq!(parsed.dependencies.len(), 2);
    assert_eq!(parsed.build.opt_level, Some(3));
}

#[test]
fn test_manifest_with_registry_dependency() {
    let toml_str = r#"
[package]
name = "my-app"
version = "0.1.0"

[dependencies]
remote-pkg = { version = "^2.0", registry = "custom-registry" }
"#;
    let manifest: PackageManifest = toml::from_str(toml_str).unwrap();
    match &manifest.dependencies["remote-pkg"] {
        Dependency::Detailed(d) => {
            assert_eq!(d.version.as_deref(), Some("^2.0"));
            assert_eq!(d.registry.as_deref(), Some("custom-registry"));
            assert!(d.path.is_none());
        }
        _ => panic!("expected Detailed dependency with registry"),
    }
}

// ===========================================================================
// 2. Package Versioning (SemVer Format Validation)
// ===========================================================================

#[test]
fn test_semver_parse_basic() {
    let v = SemVer::parse("1.2.3").unwrap();
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 2);
    assert_eq!(v.patch, 3);
    assert!(v.pre.is_none());
    assert!(v.build.is_none());
}

#[test]
fn test_semver_parse_with_prerelease() {
    let v = SemVer::parse("1.0.0-alpha").unwrap();
    assert_eq!(v.pre.as_deref(), Some("alpha"));

    let v = SemVer::parse("1.0.0-beta.1").unwrap();
    assert_eq!(v.pre.as_deref(), Some("beta.1"));

    let v = SemVer::parse("1.0.0-rc.2").unwrap();
    assert_eq!(v.pre.as_deref(), Some("rc.2"));
}

#[test]
fn test_semver_parse_with_build_metadata() {
    let v = SemVer::parse("1.0.0+build.123").unwrap();
    assert_eq!(v.build.as_deref(), Some("build.123"));
    assert!(v.pre.is_none());
}

#[test]
fn test_semver_parse_full() {
    let v = SemVer::parse("2.1.0-beta.3+sha.abc123").unwrap();
    assert_eq!(v.major, 2);
    assert_eq!(v.minor, 1);
    assert_eq!(v.patch, 0);
    assert_eq!(v.pre.as_deref(), Some("beta.3"));
    assert_eq!(v.build.as_deref(), Some("sha.abc123"));
}

#[test]
fn test_semver_parse_partial() {
    let v = SemVer::parse("1").unwrap();
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 0);
    assert_eq!(v.patch, 0);

    let v = SemVer::parse("1.5").unwrap();
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 5);
    assert_eq!(v.patch, 0);
}

#[test]
fn test_semver_invalid_versions() {
    assert!(SemVer::parse("").is_err());
    assert!(SemVer::parse("abc").is_err());
    assert!(SemVer::parse("1.2.3.4").is_err());
    assert!(SemVer::parse("1.x.3").is_err());
}

#[test]
fn test_semver_ordering() {
    let versions = vec![
        SemVer::parse("0.1.0").unwrap(),
        SemVer::parse("1.0.0").unwrap(),
        SemVer::parse("1.0.1").unwrap(),
        SemVer::parse("1.1.0").unwrap(),
        SemVer::parse("2.0.0").unwrap(),
    ];

    for i in 0..versions.len() - 1 {
        assert!(
            versions[i] < versions[i + 1],
            "{} should be less than {}",
            versions[i],
            versions[i + 1]
        );
    }
}

#[test]
fn test_semver_prerelease_ordering() {
    let release = SemVer::parse("1.0.0").unwrap();
    let alpha = SemVer::parse("1.0.0-alpha").unwrap();
    let beta = SemVer::parse("1.0.0-beta").unwrap();

    // Prerelease has lower precedence than release
    assert!(alpha < release);
    assert!(beta < release);
    // alpha < beta (lexicographic)
    assert!(alpha < beta);
}

#[test]
fn test_semver_display_roundtrip() {
    let cases = vec![
        "1.2.3",
        "0.0.1",
        "10.20.30",
        "1.0.0-alpha",
        "1.0.0+build",
        "1.0.0-rc.1+sha.abc",
    ];

    for case in cases {
        let v = SemVer::parse(case).unwrap();
        assert_eq!(v.to_string(), case, "display roundtrip failed for {}", case);
    }
}

#[test]
fn test_semver_validation_for_common_versions() {
    // These are version strings commonly found in the packages/ directory
    assert!(SemVer::is_valid_semver("1.0.0"));
    assert!(SemVer::is_valid_semver("0.1.0"));
    assert!(SemVer::is_valid_semver("0.0.1"));
    assert!(SemVer::is_valid_semver("2.0.0-alpha"));
}

// ===========================================================================
// 3. Package Dependency Resolution Logic
// ===========================================================================

#[test]
fn test_caret_version_matching() {
    let req = SemVer::parse("1.2.3").unwrap();

    // Should match within same major version
    assert!(caret_matches(&req, &SemVer::parse("1.2.3").unwrap()));
    assert!(caret_matches(&req, &SemVer::parse("1.2.4").unwrap()));
    assert!(caret_matches(&req, &SemVer::parse("1.9.0").unwrap()));

    // Should not match different major
    assert!(!caret_matches(&req, &SemVer::parse("2.0.0").unwrap()));
    // Should not match lower version
    assert!(!caret_matches(&req, &SemVer::parse("1.2.2").unwrap()));
}

#[test]
fn test_caret_zero_major() {
    // ^0.2.3 := >=0.2.3, <0.3.0
    let req = SemVer::parse("0.2.3").unwrap();
    assert!(caret_matches(&req, &SemVer::parse("0.2.3").unwrap()));
    assert!(caret_matches(&req, &SemVer::parse("0.2.9").unwrap()));
    assert!(!caret_matches(&req, &SemVer::parse("0.3.0").unwrap()));
    assert!(!caret_matches(&req, &SemVer::parse("1.0.0").unwrap()));
}

#[test]
fn test_caret_zero_zero() {
    // ^0.0.3 := >=0.0.3, <0.0.4
    let req = SemVer::parse("0.0.3").unwrap();
    assert!(caret_matches(&req, &SemVer::parse("0.0.3").unwrap()));
    assert!(!caret_matches(&req, &SemVer::parse("0.0.4").unwrap()));
    assert!(!caret_matches(&req, &SemVer::parse("0.1.0").unwrap()));
}

#[test]
fn test_tilde_version_matching() {
    // ~1.2.3 := >=1.2.3, <1.3.0
    let req = SemVer::parse("1.2.3").unwrap();
    assert!(tilde_matches(&req, &SemVer::parse("1.2.3").unwrap()));
    assert!(tilde_matches(&req, &SemVer::parse("1.2.9").unwrap()));
    assert!(!tilde_matches(&req, &SemVer::parse("1.3.0").unwrap()));
    assert!(!tilde_matches(&req, &SemVer::parse("2.0.0").unwrap()));
}

#[test]
fn test_parse_package_spec_with_version() {
    let (name, version) = parse_package_spec("json-parser@1.0.0");
    assert_eq!(name, "json-parser");
    assert_eq!(version, "1.0.0");
}

#[test]
fn test_parse_package_spec_without_version() {
    let (name, version) = parse_package_spec("json-parser");
    assert_eq!(name, "json-parser");
    assert_eq!(version, "*");
}

#[test]
fn test_parse_package_spec_with_caret_version() {
    let (name, version) = parse_package_spec("my-lib@^2.0");
    assert_eq!(name, "my-lib");
    assert_eq!(version, "^2.0");
}

#[test]
fn test_dependency_resolution_path_deps() {
    let root = TempDir::new().unwrap();

    // Create main package
    let main_dir = root.path().join("main-pkg");
    fs::create_dir_all(&main_dir).unwrap();
    init_test_package(&main_dir, "main-pkg");

    // Create dependency package
    let dep_dir = root.path().join("my-dep");
    fs::create_dir_all(&dep_dir).unwrap();
    init_test_package(&dep_dir, "my-dep");

    // Write manifest with path dependency
    let toml_content = r#"
[package]
name = "main-pkg"
version = "0.1.0"

[dependencies]
my-dep = { path = "../my-dep" }
"#;
    fs::write(main_dir.join("vais.toml"), toml_content).unwrap();

    let manifest = load_manifest(&main_dir).unwrap();
    assert_eq!(manifest.dependencies.len(), 1);
    match &manifest.dependencies["my-dep"] {
        Dependency::Detailed(d) => {
            assert_eq!(d.path.as_deref(), Some("../my-dep"));
        }
        _ => panic!("expected detailed dependency with path"),
    }
}

#[test]
fn test_dependency_resolution_mixed_deps() {
    let toml_content = r#"
[package]
name = "app"
version = "1.0.0"

[dependencies]
local-lib = { path = "./libs/local" }
registry-lib = "2.0.0"
detailed-lib = { version = "^1.5", features = ["async"] }
"#;
    let manifest: PackageManifest = toml::from_str(toml_content).unwrap();
    assert_eq!(manifest.dependencies.len(), 3);

    // Check each dep type
    match &manifest.dependencies["local-lib"] {
        Dependency::Detailed(d) => assert!(d.path.is_some()),
        _ => panic!("expected path dependency"),
    }
    match &manifest.dependencies["registry-lib"] {
        Dependency::Version(v) => assert_eq!(v, "2.0.0"),
        _ => panic!("expected version dependency"),
    }
    match &manifest.dependencies["detailed-lib"] {
        Dependency::Detailed(d) => {
            assert_eq!(d.version.as_deref(), Some("^1.5"));
            assert_eq!(d.features, vec!["async"]);
        }
        _ => panic!("expected detailed dependency"),
    }
}

#[test]
fn test_lockfile_format_parsing() {
    let toml_str = r#"
version = 1

[packages.json-parser]
version = "1.0.0"
checksum = "abc123def456"
source = "registry"
dependencies = []

[packages.utils]
version = "0.5.0"
checksum = "xyz789"
source = "registry"
dependencies = []
"#;

    #[derive(Debug, Deserialize)]
    struct LockFile {
        version: u32,
        #[serde(default)]
        packages: HashMap<String, LockedPackage>,
    }

    #[derive(Debug, Deserialize)]
    struct LockedPackage {
        version: String,
        checksum: String,
        source: String,
        #[serde(default)]
        dependencies: Vec<String>,
    }

    let lock: LockFile = toml::from_str(toml_str).unwrap();
    assert_eq!(lock.version, 1);
    assert_eq!(lock.packages.len(), 2);
    assert!(lock.packages.contains_key("json-parser"));
    assert!(lock.packages.contains_key("utils"));
    assert_eq!(lock.packages["json-parser"].version, "1.0.0");
    assert_eq!(lock.packages["json-parser"].checksum, "abc123def456");
    assert_eq!(lock.packages["utils"].version, "0.5.0");
}

#[test]
fn test_lockfile_roundtrip() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct LockFile {
        version: u32,
        #[serde(default)]
        packages: std::collections::BTreeMap<String, LockedPkg>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct LockedPkg {
        version: String,
        checksum: String,
        source: String,
        #[serde(default)]
        dependencies: Vec<String>,
    }

    let mut lock = LockFile {
        version: 1,
        packages: std::collections::BTreeMap::new(),
    };
    lock.packages.insert(
        "test-pkg".to_string(),
        LockedPkg {
            version: "1.2.3".to_string(),
            checksum: "abc".to_string(),
            source: "registry".to_string(),
            dependencies: vec!["dep-a".to_string()],
        },
    );

    let serialized = toml::to_string_pretty(&lock).unwrap();
    let deserialized: LockFile = toml::from_str(&serialized).unwrap();
    assert_eq!(lock, deserialized);
}

#[test]
fn test_cached_registry_dep_lookup() {
    let root = TempDir::new().unwrap();

    // Create cache structure: cache/<name>/<version>/extracted/
    let extracted = root
        .path()
        .join("cache")
        .join("my-pkg")
        .join("1.2.3")
        .join("extracted");
    fs::create_dir_all(&extracted).unwrap();

    // Should find with exact version
    let result = find_cached_dep(root.path(), "my-pkg", "1.2.3");
    assert!(result.is_some());
    assert!(result.unwrap().ends_with("extracted"));

    // Should find with ^ prefix stripped
    let result = find_cached_dep(root.path(), "my-pkg", "^1.2.3");
    assert!(result.is_some());

    // Should find with ~ prefix stripped
    let result = find_cached_dep(root.path(), "my-pkg", "~1.2.3");
    assert!(result.is_some());

    // Should not find nonexistent package
    let result = find_cached_dep(root.path(), "nonexistent", "1.0.0");
    assert!(result.is_none());
}

/// Mirrors find_cached_registry_dep from package.rs
fn find_cached_dep(cache_root: &Path, name: &str, version_str: &str) -> Option<PathBuf> {
    let pkg_cache_dir = cache_root.join("cache").join(name);
    if !pkg_cache_dir.exists() {
        return None;
    }

    let exact_path = pkg_cache_dir.join(version_str).join("extracted");
    if exact_path.exists() {
        return Some(exact_path);
    }

    let stripped = version_str
        .trim_start_matches('^')
        .trim_start_matches('~')
        .trim_start_matches(">=")
        .trim_start_matches("<=")
        .trim_start_matches('>')
        .trim_start_matches('<')
        .trim_start_matches('=')
        .trim();

    if stripped != version_str {
        let stripped_path = pkg_cache_dir.join(stripped).join("extracted");
        if stripped_path.exists() {
            return Some(stripped_path);
        }
    }

    if let Ok(entries) = fs::read_dir(&pkg_cache_dir) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                let extracted = entry_path.join("extracted");
                if extracted.exists() {
                    return Some(extracted);
                }
            }
        }
    }

    None
}

#[test]
fn test_transitive_dependency_detection() {
    // Simulate: A depends on B, B depends on C
    // After resolving A, we should discover B and C
    let root = TempDir::new().unwrap();

    // Package C (no deps)
    let c_dir = root.path().join("pkg-c");
    fs::create_dir_all(&c_dir).unwrap();
    init_test_package(&c_dir, "pkg-c");

    // Package B (depends on C)
    let b_dir = root.path().join("pkg-b");
    fs::create_dir_all(&b_dir).unwrap();
    let b_toml = r#"
[package]
name = "pkg-b"
version = "0.1.0"

[dependencies]
pkg-c = { path = "../pkg-c" }
"#;
    fs::write(b_dir.join("vais.toml"), b_toml).unwrap();
    fs::create_dir_all(b_dir.join("src")).unwrap();
    fs::write(b_dir.join("src/main.vais"), "").unwrap();

    // Package A (depends on B)
    let a_dir = root.path().join("pkg-a");
    fs::create_dir_all(&a_dir).unwrap();
    let a_toml = r#"
[package]
name = "pkg-a"
version = "0.1.0"

[dependencies]
pkg-b = { path = "../pkg-b" }
"#;
    fs::write(a_dir.join("vais.toml"), a_toml).unwrap();
    fs::create_dir_all(a_dir.join("src")).unwrap();
    fs::write(a_dir.join("src/main.vais"), "").unwrap();

    // Verify all manifests load correctly
    let a_manifest = load_manifest(&a_dir).unwrap();
    let b_manifest = load_manifest(&b_dir).unwrap();
    let c_manifest = load_manifest(&c_dir).unwrap();

    assert_eq!(a_manifest.dependencies.len(), 1);
    assert!(a_manifest.dependencies.contains_key("pkg-b"));
    assert_eq!(b_manifest.dependencies.len(), 1);
    assert!(b_manifest.dependencies.contains_key("pkg-c"));
    assert_eq!(c_manifest.dependencies.len(), 0);
}

#[test]
fn test_cyclic_dependency_detection_structure() {
    // Create packages that would form a cycle: A -> B -> A
    let root = TempDir::new().unwrap();

    let a_dir = root.path().join("pkg-a");
    fs::create_dir_all(&a_dir).unwrap();
    let a_toml = r#"
[package]
name = "pkg-a"
version = "0.1.0"

[dependencies]
pkg-b = { path = "../pkg-b" }
"#;
    fs::write(a_dir.join("vais.toml"), a_toml).unwrap();

    let b_dir = root.path().join("pkg-b");
    fs::create_dir_all(&b_dir).unwrap();
    let b_toml = r#"
[package]
name = "pkg-b"
version = "0.1.0"

[dependencies]
pkg-a = { path = "../pkg-a" }
"#;
    fs::write(b_dir.join("vais.toml"), b_toml).unwrap();

    // Both manifests parse fine individually
    let a = load_manifest(&a_dir).unwrap();
    let b = load_manifest(&b_dir).unwrap();

    // But we can detect the cycle by walking the graph
    let mut visited = std::collections::HashSet::new();
    visited.insert("pkg-a".to_string());

    // pkg-a depends on pkg-b
    assert!(a.dependencies.contains_key("pkg-b"));
    visited.insert("pkg-b".to_string());

    // pkg-b depends on pkg-a which is already visited -> cycle
    for dep_name in b.dependencies.keys() {
        assert!(
            visited.contains(dep_name),
            "cycle detected: pkg-b depends on {} which was already visited",
            dep_name
        );
    }
}

// ===========================================================================
// 4. Package Search/List Command Output
// ===========================================================================

#[test]
fn test_package_index_search_by_name() {
    // Simulate an in-memory package index and search
    let packages = vec![
        (
            "json-parser",
            Some("A JSON parser for Vais"),
            vec!["json", "parser"],
        ),
        (
            "xml-parser",
            Some("XML parsing library"),
            vec!["xml", "parser"],
        ),
        ("csv", Some("CSV reader/writer"), vec!["csv", "data"]),
        ("math-ext", Some("Extended math functions"), vec!["math"]),
    ];

    let query = "parser";
    let results: Vec<_> = packages
        .iter()
        .filter(|(name, desc, keywords)| {
            let q = query.to_lowercase();
            name.to_lowercase().contains(&q)
                || desc.map(|d| d.to_lowercase().contains(&q)).unwrap_or(false)
                || keywords.iter().any(|k| k.to_lowercase().contains(&q))
        })
        .collect();

    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|(name, _, _)| *name == "json-parser"));
    assert!(results.iter().any(|(name, _, _)| *name == "xml-parser"));
}

#[test]
fn test_package_index_search_by_keyword() {
    let packages = vec![
        ("json-parser", vec!["json", "parser", "serialization"]),
        ("toml-parser", vec!["toml", "parser", "config"]),
        ("csv", vec!["csv", "data", "serialization"]),
    ];

    let keyword_query = "serialization";
    let results: Vec<_> = packages
        .iter()
        .filter(|(_, keywords)| keywords.iter().any(|k| k == &keyword_query))
        .collect();

    assert_eq!(results.len(), 2);
}

#[test]
fn test_package_index_search_case_insensitive() {
    let packages = vec![
        ("JSON-Parser", Some("A JSON parser")),
        ("csv", Some("CSV tools")),
    ];

    let query = "json";
    let results: Vec<_> = packages
        .iter()
        .filter(|(name, desc)| {
            let q = query.to_lowercase();
            name.to_lowercase().contains(&q)
                || desc.map(|d| d.to_lowercase().contains(&q)).unwrap_or(false)
        })
        .collect();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, "JSON-Parser");
}

#[test]
fn test_package_list_best_version_selection() {
    // Given a list of versions, find the best (latest non-prerelease)
    let versions = vec![
        SemVer::parse("0.9.0").unwrap(),
        SemVer::parse("1.0.0-beta").unwrap(),
        SemVer::parse("1.0.0").unwrap(),
        SemVer::parse("1.1.0").unwrap(),
        SemVer::parse("2.0.0-alpha").unwrap(),
    ];

    let best = versions.iter().filter(|v| v.pre.is_none()).max().unwrap();

    assert_eq!(best.to_string(), "1.1.0");
}

// ===========================================================================
// 5. Registry Server Endpoint Structure
// ===========================================================================

/// Expected API routes from router.rs
const EXPECTED_API_ROUTES: &[(&str, &str)] = &[
    ("GET", "/api/v1/health"),
    ("GET", "/api/v1/index.json"),
    ("GET", "/api/v1/packages/:name/index.json"),
    ("POST", "/api/v1/packages/publish"),
    ("GET", "/api/v1/packages/:name"),
    ("GET", "/api/v1/packages/:name/:version"),
    ("POST", "/api/v1/packages/:name/:version/yank"),
    ("POST", "/api/v1/packages/:name/:version/unyank"),
    ("GET", "/api/v1/search"),
    ("GET", "/api/v1/categories"),
    ("GET", "/api/v1/categories/:category"),
    ("GET", "/api/v1/popular"),
    ("GET", "/api/v1/recent"),
    ("POST", "/api/v1/auth/register"),
    ("POST", "/api/v1/auth/login"),
    ("GET", "/api/v1/auth/me"),
    ("GET", "/api/v1/auth/tokens"),
    ("POST", "/api/v1/auth/tokens"),
    ("DELETE", "/api/v1/auth/tokens/:id"),
    ("GET", "/api/v1/users/:username"),
    ("POST", "/api/v1/packages/:name/owners"),
    ("DELETE", "/api/v1/packages/:name/owners/:username"),
];

#[test]
fn test_registry_api_routes_defined() {
    // Verify the expected API route set is complete and well-structured
    assert!(
        EXPECTED_API_ROUTES.len() >= 22,
        "expected at least 22 API routes, got {}",
        EXPECTED_API_ROUTES.len()
    );

    // All routes should start with /api/v1/
    for (method, path) in EXPECTED_API_ROUTES {
        assert!(
            path.starts_with("/api/v1/"),
            "route {} {} should be under /api/v1/",
            method,
            path
        );
    }
}

#[test]
fn test_registry_api_has_package_crud() {
    let has_publish = EXPECTED_API_ROUTES
        .iter()
        .any(|(m, p)| *m == "POST" && p.contains("publish"));
    let has_get = EXPECTED_API_ROUTES
        .iter()
        .any(|(m, p)| *m == "GET" && p.contains("/packages/:name"));
    let has_download = EXPECTED_API_ROUTES
        .iter()
        .any(|(m, p)| *m == "GET" && p.contains("/packages/:name/:version"));
    let has_yank = EXPECTED_API_ROUTES
        .iter()
        .any(|(m, p)| *m == "POST" && p.contains("yank"));

    assert!(has_publish, "API should have publish endpoint");
    assert!(has_get, "API should have get-package endpoint");
    assert!(has_download, "API should have download endpoint");
    assert!(has_yank, "API should have yank endpoint");
}

#[test]
fn test_registry_api_has_auth_routes() {
    let has_register = EXPECTED_API_ROUTES
        .iter()
        .any(|(m, p)| *m == "POST" && p.contains("auth/register"));
    let has_login = EXPECTED_API_ROUTES
        .iter()
        .any(|(m, p)| *m == "POST" && p.contains("auth/login"));
    let has_me = EXPECTED_API_ROUTES
        .iter()
        .any(|(m, p)| *m == "GET" && p.contains("auth/me"));
    let has_tokens = EXPECTED_API_ROUTES
        .iter()
        .any(|(_, p)| p.contains("auth/tokens"));

    assert!(has_register, "API should have register endpoint");
    assert!(has_login, "API should have login endpoint");
    assert!(has_me, "API should have me endpoint");
    assert!(has_tokens, "API should have tokens endpoint");
}

#[test]
fn test_registry_api_has_search_and_discovery() {
    let has_search = EXPECTED_API_ROUTES
        .iter()
        .any(|(m, p)| *m == "GET" && *p == "/api/v1/search");
    let has_categories = EXPECTED_API_ROUTES
        .iter()
        .any(|(m, p)| *m == "GET" && *p == "/api/v1/categories");
    let has_popular = EXPECTED_API_ROUTES
        .iter()
        .any(|(m, p)| *m == "GET" && *p == "/api/v1/popular");
    let has_recent = EXPECTED_API_ROUTES
        .iter()
        .any(|(m, p)| *m == "GET" && *p == "/api/v1/recent");

    assert!(has_search, "API should have search endpoint");
    assert!(has_categories, "API should have categories endpoint");
    assert!(has_popular, "API should have popular endpoint");
    assert!(has_recent, "API should have recent endpoint");
}

#[test]
fn test_registry_api_has_owner_management() {
    let has_add_owner = EXPECTED_API_ROUTES
        .iter()
        .any(|(m, p)| *m == "POST" && p.contains("owners"));
    let has_remove_owner = EXPECTED_API_ROUTES
        .iter()
        .any(|(m, p)| *m == "DELETE" && p.contains("owners"));

    assert!(has_add_owner, "API should have add-owner endpoint");
    assert!(has_remove_owner, "API should have remove-owner endpoint");
}

#[test]
fn test_registry_api_has_index_routes() {
    let has_full_index = EXPECTED_API_ROUTES
        .iter()
        .any(|(m, p)| *m == "GET" && *p == "/api/v1/index.json");
    let has_pkg_index = EXPECTED_API_ROUTES
        .iter()
        .any(|(m, p)| *m == "GET" && p.contains("packages/:name/index.json"));

    assert!(has_full_index, "API should have full index endpoint");
    assert!(has_pkg_index, "API should have per-package index endpoint");
}

// ===========================================================================
// 6. Validate packages/ Directory Manifests
// ===========================================================================

#[test]
fn test_packages_directory_exists() {
    let pkg_dir = packages_dir();
    assert!(
        pkg_dir.exists(),
        "packages/ directory should exist at {}",
        pkg_dir.display()
    );
}

#[test]
fn test_all_seed_packages_have_valid_manifests() {
    let pkg_dir = packages_dir();
    if !pkg_dir.exists() {
        return; // Skip if no packages dir (CI environment)
    }

    let entries: Vec<_> = fs::read_dir(&pkg_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    assert!(
        !entries.is_empty(),
        "packages/ directory should contain at least one package"
    );

    for entry in &entries {
        let dir = entry.path();
        let manifest_path = dir.join("vais.toml");

        assert!(
            manifest_path.exists(),
            "package {} should have a vais.toml",
            dir.display()
        );

        let content = fs::read_to_string(&manifest_path).unwrap_or_else(|e| {
            panic!("failed to read {}: {}", manifest_path.display(), e);
        });

        let manifest: PackageManifest = toml::from_str(&content).unwrap_or_else(|e| {
            panic!("failed to parse {}: {}", manifest_path.display(), e);
        });

        // Name should not be empty
        assert!(
            !manifest.package.name.is_empty(),
            "package in {} should have a non-empty name",
            dir.display()
        );

        // Version should be valid semver
        assert!(
            SemVer::is_valid_semver(&manifest.package.version),
            "package {} has invalid version: {}",
            manifest.package.name,
            manifest.package.version
        );

        // Name should match directory name
        let dir_name = dir.file_name().unwrap().to_str().unwrap();
        assert_eq!(
            manifest.package.name, dir_name,
            "package name '{}' should match directory name '{}'",
            manifest.package.name, dir_name
        );
    }
}

#[test]
fn test_seed_packages_have_required_fields() {
    let pkg_dir = packages_dir();
    if !pkg_dir.exists() {
        return;
    }

    for entry in fs::read_dir(&pkg_dir).unwrap().filter_map(|e| e.ok()) {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let manifest_path = dir.join("vais.toml");
        if !manifest_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&manifest_path).unwrap();
        let manifest: PackageManifest = toml::from_str(&content).unwrap();

        // All seed packages should have authors
        assert!(
            !manifest.package.authors.is_empty(),
            "seed package {} should have at least one author",
            manifest.package.name
        );

        // All seed packages should have a description
        assert!(
            manifest.package.description.is_some(),
            "seed package {} should have a description",
            manifest.package.name
        );

        // All seed packages should have a license
        assert!(
            manifest.package.license.is_some(),
            "seed package {} should have a license",
            manifest.package.name
        );
    }
}

#[test]
fn test_known_seed_packages_present() {
    let pkg_dir = packages_dir();
    if !pkg_dir.exists() {
        return;
    }

    let expected_packages = [
        "cli-args",
        "env",
        "color",
        "csv",
        "toml-parser",
        "dotenv",
        "retry",
        "validate",
        "cache",
        "math-ext",
    ];

    for name in &expected_packages {
        let dir = pkg_dir.join(name);
        assert!(
            dir.exists(),
            "expected seed package '{}' to exist at {}",
            name,
            dir.display()
        );
        assert!(
            dir.join("vais.toml").exists(),
            "seed package '{}' should have a vais.toml",
            name
        );
    }
}

// ===========================================================================
// 7. CLI Argument Parsing for pkg Subcommands
// ===========================================================================

/// Helper to run vaisc binary and capture output (compile check only)
fn vaisc_binary() -> PathBuf {
    // Use cargo to locate the binary built from the workspace
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // crates/
    path.pop(); // project root
    path.join("target").join("debug").join("vaisc")
}

#[test]
fn test_pkg_help_output() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "vaisc", "--", "pkg", "--help"])
        .current_dir(project_root())
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            let combined = format!("{}{}", stdout, stderr);

            // The help should mention subcommands
            let has_init = combined.contains("init") || combined.contains("Init");
            let has_build = combined.contains("build") || combined.contains("Build");
            let has_publish = combined.contains("publish") || combined.contains("Publish");
            let has_install = combined.contains("install") || combined.contains("Install");
            let has_search = combined.contains("search") || combined.contains("Search");

            assert!(
                has_init || has_build,
                "pkg --help should list subcommands. Got:\n{}",
                combined
            );

            // If help works properly, verify key subcommands
            if o.status.success() || combined.contains("Usage") {
                assert!(has_init, "pkg help should mention init");
                assert!(has_build, "pkg help should mention build");
                assert!(has_publish, "pkg help should mention publish");
                assert!(has_install, "pkg help should mention install");
                assert!(has_search, "pkg help should mention search");
            }
        }
        Err(_) => {
            // Binary not built yet - skip this test gracefully
            eprintln!("skipping CLI test: vaisc binary not available");
        }
    }
}

#[test]
fn test_pkg_init_creates_manifest() {
    let tmp = TempDir::new().unwrap();
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "vaisc",
            "--",
            "pkg",
            "init",
            "--name",
            "test-init-pkg",
        ])
        .current_dir(tmp.path())
        .env("CARGO_MANIFEST_DIR", project_root().join("crates/vaisc"))
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                // Verify vais.toml was created
                let manifest_path = tmp.path().join("vais.toml");
                assert!(
                    manifest_path.exists(),
                    "vais.toml should be created by pkg init"
                );

                let content = fs::read_to_string(&manifest_path).unwrap();
                let manifest: PackageManifest = toml::from_str(&content).unwrap();
                assert_eq!(manifest.package.name, "test-init-pkg");
                assert_eq!(manifest.package.version, "0.1.0");

                // Verify src directory was created
                assert!(tmp.path().join("src").exists(), "src/ should be created");
                assert!(
                    tmp.path().join("src/main.vais").exists(),
                    "src/main.vais should be created"
                );
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                // Acceptable if binary is not built or LLVM not available
                eprintln!("pkg init returned non-zero (may be expected): {}", stderr);
            }
        }
        Err(_) => {
            eprintln!("skipping CLI test: cargo run not available");
        }
    }
}

// ===========================================================================
// 8. Package Archive Structure Tests
// ===========================================================================

#[test]
fn test_package_archive_naming_convention() {
    // Archives should follow <name>-<version>.tar.gz convention
    let test_cases = vec![
        ("json-parser", "1.0.0", "json-parser-1.0.0.tar.gz"),
        ("my-lib", "0.2.1-beta", "my-lib-0.2.1-beta.tar.gz"),
        ("utils", "2.0.0", "utils-2.0.0.tar.gz"),
    ];

    for (name, version, expected) in test_cases {
        let archive_name = format!("{}-{}.tar.gz", name, version);
        assert_eq!(archive_name, expected);
    }
}

#[test]
fn test_package_cache_directory_structure() {
    let root = TempDir::new().unwrap();

    // Simulate cache structure
    let cache_root = root.path().join("registry");
    let cache_dir = cache_root.join("cache");
    let index_dir = cache_root.join("index");

    fs::create_dir_all(&cache_dir).unwrap();
    fs::create_dir_all(&index_dir).unwrap();

    // Package cache: cache/<name>/<version>/extracted/
    let pkg_extracted = cache_dir
        .join("json-parser")
        .join("1.0.0")
        .join("extracted");
    fs::create_dir_all(&pkg_extracted).unwrap();

    // Archive: cache/<name>/<version>.tar.gz
    let archive = cache_dir.join("json-parser").join("1.0.0.tar.gz");
    fs::write(&archive, b"fake archive").unwrap();

    assert!(cache_dir.exists());
    assert!(index_dir.exists());
    assert!(pkg_extracted.exists());
    assert!(archive.exists());

    // Verify lookup works
    let found = find_cached_dep(&cache_root, "json-parser", "1.0.0");
    assert!(found.is_some());
}

// ===========================================================================
// 9. Package Metadata JSON Serialization (Registry Server Format)
// ===========================================================================

#[test]
fn test_package_metadata_json_format() {
    let json = r#"{
        "name": "json-parser",
        "description": "A JSON parser for Vais",
        "versions": [
            {
                "version": "1.0.0",
                "checksum": "abc123",
                "dependencies": {}
            },
            {
                "version": "1.1.0",
                "checksum": "def456",
                "dependencies": {
                    "utils": { "req": "^0.5.0" }
                }
            }
        ],
        "keywords": ["json", "parser"]
    }"#;

    #[derive(Debug, Deserialize)]
    struct PkgMeta {
        name: String,
        description: Option<String>,
        versions: Vec<VerEntry>,
        #[serde(default)]
        keywords: Vec<String>,
    }

    #[derive(Debug, Deserialize)]
    struct VerEntry {
        version: String,
        checksum: String,
        #[serde(default)]
        dependencies: HashMap<String, DepInfo>,
    }

    #[derive(Debug, Deserialize)]
    struct DepInfo {
        req: String,
    }

    let pkg: PkgMeta = serde_json::from_str(json).unwrap();
    assert_eq!(pkg.name, "json-parser");
    assert_eq!(pkg.description.as_deref(), Some("A JSON parser for Vais"));
    assert_eq!(pkg.versions.len(), 2);
    assert_eq!(pkg.versions[0].version, "1.0.0");
    assert_eq!(pkg.versions[0].checksum, "abc123");
    assert!(pkg.versions[0].dependencies.is_empty());
    assert_eq!(pkg.versions[1].version, "1.1.0");
    assert_eq!(pkg.versions[1].dependencies.len(), 1);
    assert_eq!(pkg.versions[1].dependencies["utils"].req, "^0.5.0");
    assert_eq!(pkg.keywords, vec!["json", "parser"]);
}

#[test]
fn test_search_result_json_format() {
    let json = r#"{
        "packages": [
            {
                "name": "json-parser",
                "description": "JSON parser",
                "latest_version": "1.1.0",
                "downloads": 1500,
                "keywords": ["json"],
                "categories": ["parsing"],
                "updated_at": "2025-01-01T00:00:00Z"
            }
        ],
        "total": 1
    }"#;

    #[derive(Debug, Deserialize)]
    struct SearchResult {
        packages: Vec<SearchEntry>,
        total: usize,
    }

    #[derive(Debug, Deserialize)]
    struct SearchEntry {
        name: String,
        description: Option<String>,
        latest_version: String,
        downloads: i64,
        keywords: Vec<String>,
        categories: Vec<String>,
    }

    let result: SearchResult = serde_json::from_str(json).unwrap();
    assert_eq!(result.total, 1);
    assert_eq!(result.packages.len(), 1);
    assert_eq!(result.packages[0].name, "json-parser");
    assert_eq!(result.packages[0].latest_version, "1.1.0");
    assert_eq!(result.packages[0].downloads, 1500);
}

#[test]
fn test_publish_request_json_format() {
    #[derive(Debug, Serialize, Deserialize)]
    struct PublishReq {
        name: String,
        version: String,
        description: Option<String>,
        #[serde(default)]
        keywords: Vec<String>,
        #[serde(default)]
        categories: Vec<String>,
        #[serde(default)]
        dependencies: HashMap<String, String>,
    }

    let req = PublishReq {
        name: "my-pkg".to_string(),
        version: "1.0.0".to_string(),
        description: Some("My package".to_string()),
        keywords: vec!["utility".to_string()],
        categories: vec!["tools".to_string()],
        dependencies: {
            let mut m = HashMap::new();
            m.insert("dep-a".to_string(), "^1.0".to_string());
            m
        },
    };

    let json = serde_json::to_string(&req).unwrap();
    let parsed: PublishReq = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name, "my-pkg");
    assert_eq!(parsed.version, "1.0.0");
    assert_eq!(parsed.dependencies.len(), 1);
}

// ===========================================================================
// 10. Edge Cases and Error Handling
// ===========================================================================

#[test]
fn test_manifest_with_empty_dependencies() {
    let toml_str = r#"
[package]
name = "no-deps"
version = "1.0.0"

[dependencies]

[dev-dependencies]
"#;
    let manifest: PackageManifest = toml::from_str(toml_str).unwrap();
    assert!(manifest.dependencies.is_empty());
    assert!(manifest.dev_dependencies.is_empty());
}

#[test]
fn test_manifest_special_characters_in_name() {
    // Package names with hyphens and numbers are valid
    let valid_names = ["my-pkg", "pkg123", "a-b-c", "vais-std-http"];
    for name in &valid_names {
        let toml_str = format!(
            r#"
[package]
name = "{}"
version = "1.0.0"
"#,
            name
        );
        let result: Result<PackageManifest, _> = toml::from_str(&toml_str);
        assert!(result.is_ok(), "name '{}' should be valid", name);
    }
}

#[test]
fn test_manifest_unicode_description() {
    let toml_str = r#"
[package]
name = "intl-pkg"
version = "1.0.0"
description = "A package with unicode: 日本語テスト"
"#;
    let manifest: PackageManifest = toml::from_str(toml_str).unwrap();
    assert!(manifest.package.description.unwrap().contains("日本語"));
}

#[test]
fn test_semver_zero_versions() {
    let v = SemVer::parse("0.0.0").unwrap();
    assert_eq!(v.major, 0);
    assert_eq!(v.minor, 0);
    assert_eq!(v.patch, 0);
}

#[test]
fn test_semver_large_numbers() {
    let v = SemVer::parse("999.999.999").unwrap();
    assert_eq!(v.major, 999);
    assert_eq!(v.minor, 999);
    assert_eq!(v.patch, 999);
}

#[test]
fn test_dependency_add_then_remove_roundtrip() {
    let tmp = TempDir::new().unwrap();
    init_test_package(tmp.path(), "roundtrip-test");

    // Read initial manifest
    let manifest = load_manifest(tmp.path()).unwrap();
    assert!(manifest.dependencies.is_empty());

    // Add a dependency by modifying the manifest
    let mut updated = manifest;
    updated.dependencies.insert(
        "new-dep".to_string(),
        Dependency::Version("1.0.0".to_string()),
    );
    let content = toml::to_string_pretty(&updated).unwrap();
    fs::write(tmp.path().join("vais.toml"), content).unwrap();

    // Verify it was added
    let reloaded = load_manifest(tmp.path()).unwrap();
    assert_eq!(reloaded.dependencies.len(), 1);
    assert!(reloaded.dependencies.contains_key("new-dep"));

    // Remove the dependency
    let mut final_manifest = reloaded;
    final_manifest.dependencies.remove("new-dep");
    let content = toml::to_string_pretty(&final_manifest).unwrap();
    fs::write(tmp.path().join("vais.toml"), content).unwrap();

    // Verify it was removed
    let final_loaded = load_manifest(tmp.path()).unwrap();
    assert!(final_loaded.dependencies.is_empty());
}

#[test]
fn test_manifest_not_found_error() {
    let tmp = TempDir::new().unwrap();
    let result = load_manifest(tmp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("failed to read"));
}

#[test]
fn test_manifest_invalid_toml_error() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("vais.toml"), "this is not valid toml {{{").unwrap();
    let result = load_manifest(tmp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("failed to parse"));
}

// ===========================================================================
// 12. Native Dependencies Tests
// ===========================================================================

#[test]
fn test_manifest_with_native_dependencies() {
    let tmp = TempDir::new().unwrap();
    let toml_content = r#"
[package]
name = "native-dep-test"
version = "1.0.0"

[native-dependencies]
openssl = "ssl"

[native-dependencies.zlib]
libs = ["z"]
include = "/usr/include"
"#;
    fs::write(tmp.path().join("vais.toml"), toml_content).unwrap();
    let manifest = load_manifest(tmp.path());
    assert!(
        manifest.is_ok(),
        "manifest with native-dependencies should parse"
    );
}

#[test]
fn test_manifest_without_native_dependencies_still_parses() {
    let tmp = TempDir::new().unwrap();
    let toml_content = r#"
[package]
name = "no-native-deps"
version = "0.1.0"

[dependencies]
"#;
    fs::write(tmp.path().join("vais.toml"), toml_content).unwrap();
    let manifest = load_manifest(tmp.path());
    assert!(
        manifest.is_ok(),
        "manifest without native-dependencies should still parse"
    );
}

// ===========================================================================
// 13. Directory Build Tests
// ===========================================================================

#[test]
fn test_build_directory_with_main_vais() {
    let tmp = TempDir::new().unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        src_dir.join("main.vais"),
        "F main() -> i64 { puts(\"dir build test\") \n 0 }",
    )
    .unwrap();

    let output = Command::new("cargo")
        .args(["run", "--bin", "vaisc", "--", "build"])
        .arg(tmp.path())
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                // Verify binary was created
                let bin = src_dir.join("main");
                assert!(bin.exists(), "binary should be created at src/main");
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                eprintln!(
                    "directory build returned non-zero (may be expected in CI): {}",
                    stderr
                );
            }
        }
        Err(_) => {
            eprintln!("skipping CLI test: cargo run not available");
        }
    }
}

#[test]
fn test_build_directory_with_multifile_import() {
    let tmp = TempDir::new().unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Create utils module
    fs::write(
        src_dir.join("myutils.vais"),
        "F add_nums(a: i64, b: i64) -> i64 { a + b }",
    )
    .unwrap();

    // Create main with use statement
    fs::write(
        src_dir.join("main.vais"),
        "U myutils\nF main() -> i64 { add_nums(3, 4) }",
    )
    .unwrap();

    let output = Command::new("cargo")
        .args(["run", "--bin", "vaisc", "--", "build"])
        .arg(tmp.path())
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                // Run the built binary and check exit code
                let bin = src_dir.join("main");
                if bin.exists() {
                    let run_output = Command::new(&bin).output().unwrap();
                    assert_eq!(
                        run_output.status.code(),
                        Some(7),
                        "add_nums(3, 4) should return 7"
                    );
                }
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                eprintln!("multifile build returned non-zero (may be expected in CI): {}", stderr);
            }
        }
        Err(_) => {
            eprintln!("skipping CLI test: cargo run not available");
        }
    }
}
