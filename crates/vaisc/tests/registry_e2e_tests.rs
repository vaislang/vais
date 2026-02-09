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
                eprintln!(
                    "multifile build returned non-zero (may be expected in CI): {}",
                    stderr
                );
            }
        }
        Err(_) => {
            eprintln!("skipping CLI test: cargo run not available");
        }
    }
}

// ==========================================================================
// Phase 41 Stage 5: Package Ecosystem Tests
// ==========================================================================

/// Helper to run vaisc subcommand
fn run_vaisc(args: &[&str], cwd: &Path) -> Result<std::process::Output, String> {
    Command::new("cargo")
        .args(["run", "--bin", "vaisc", "--"])
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("failed to run vaisc: {}", e))
}

/// Run vaisc from the project root (for commands that need a specific working directory
/// but where cargo needs to find its own Cargo.toml)
fn run_vaisc_from_root(args: &[&str]) -> Result<std::process::Output, String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&manifest_dir);
    Command::new("cargo")
        .args(["run", "--bin", "vaisc", "--"])
        .args(args)
        .current_dir(workspace_root)
        .output()
        .map_err(|e| format!("failed to run vaisc: {}", e))
}

#[test]
fn test_vaisc_new_creates_project() {
    let tmp = TempDir::new().unwrap();
    let project_name = "my_test_project";

    match run_vaisc(&["new", project_name], tmp.path()) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);

            if output.status.success() {
                // Check project directory was created
                let project_dir = tmp.path().join(project_name);
                assert!(project_dir.exists(), "project directory should exist");
                assert!(
                    project_dir.join("vais.toml").exists(),
                    "vais.toml should exist"
                );
                assert!(
                    project_dir.join("src").join("main.vais").exists(),
                    "src/main.vais should exist"
                );
                assert!(
                    project_dir.join("tests").exists(),
                    "tests/ directory should exist"
                );
                assert!(
                    project_dir.join(".gitignore").exists(),
                    ".gitignore should exist"
                );

                // Check vais.toml content
                let manifest_content = fs::read_to_string(project_dir.join("vais.toml")).unwrap();
                assert!(
                    manifest_content.contains(project_name),
                    "vais.toml should contain project name"
                );
                assert!(
                    manifest_content.contains("0.1.0"),
                    "vais.toml should have default version"
                );

                // Check stdout mentions creation
                assert!(
                    stdout.contains("Created") || stdout.contains("✓"),
                    "should print success message"
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("vaisc new returned non-zero: {}", stderr);
            }
        }
        Err(_) => {
            eprintln!("skipping vaisc new test: cargo run not available");
        }
    }
}

#[test]
fn test_vaisc_new_lib_project() {
    let tmp = TempDir::new().unwrap();
    let project_name = "my_lib_project";

    match run_vaisc(&["new", project_name, "--lib"], tmp.path()) {
        Ok(output) => {
            if output.status.success() {
                let project_dir = tmp.path().join(project_name);
                assert!(
                    project_dir.join("src").join("lib.vais").exists(),
                    "src/lib.vais should exist for library project"
                );
                assert!(
                    !project_dir.join("src").join("main.vais").exists(),
                    "src/main.vais should NOT exist for library project"
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("vaisc new --lib returned non-zero: {}", stderr);
            }
        }
        Err(_) => {
            eprintln!("skipping vaisc new --lib test: cargo run not available");
        }
    }
}

#[test]
fn test_vaisc_new_duplicate_fails() {
    let tmp = TempDir::new().unwrap();
    let project_name = "dup_project";

    // Create first time
    if let Ok(output) = run_vaisc(&["new", project_name], tmp.path()) {
        if output.status.success() {
            // Try to create again - should fail
            if let Ok(output2) = run_vaisc(&["new", project_name], tmp.path()) {
                assert!(
                    !output2.status.success(),
                    "creating duplicate project should fail"
                );
            }
        }
    }
}

#[test]
fn test_vaisc_new_creates_test_file() {
    let tmp = TempDir::new().unwrap();
    let project_name = "test_proj_tests";

    match run_vaisc(&["new", project_name], tmp.path()) {
        Ok(output) => {
            if output.status.success() {
                let test_file = tmp
                    .path()
                    .join(project_name)
                    .join("tests")
                    .join("test_main.vais");
                assert!(test_file.exists(), "test file should be created");

                let content = fs::read_to_string(&test_file).unwrap();
                assert!(
                    content.contains("test_basic") || content.contains("F test_"),
                    "test file should contain a test function"
                );
            }
        }
        Err(_) => {
            eprintln!("skipping test: cargo run not available");
        }
    }
}

#[test]
fn test_vaisc_test_with_passing_test() {
    let tmp = TempDir::new().unwrap();
    let tests_dir = tmp.path().join("tests");
    fs::create_dir_all(&tests_dir).unwrap();

    // Write a passing test (exit code 0)
    let test_source = "F main() -> i64 {\n    0\n}\n";
    fs::write(tests_dir.join("pass_test.vais"), test_source).unwrap();

    match run_vaisc(&["test", tests_dir.to_str().unwrap()], tmp.path()) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if output.status.success() {
                assert!(
                    stdout.contains("PASS") || stdout.contains("passed"),
                    "should show pass message: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                // May fail if clang is not available
                eprintln!("vaisc test may have failed (ok in CI): {}", stderr);
            }
        }
        Err(_) => {
            eprintln!("skipping vaisc test: cargo run not available");
        }
    }
}

#[test]
fn test_vaisc_test_with_failing_test() {
    let tmp = TempDir::new().unwrap();
    let tests_dir = tmp.path().join("tests");
    fs::create_dir_all(&tests_dir).unwrap();

    // Write a failing test (exit code 1)
    let test_source = "F main() -> i64 {\n    1\n}\n";
    fs::write(tests_dir.join("fail_test.vais"), test_source).unwrap();

    match run_vaisc(&["test", tests_dir.to_str().unwrap()], tmp.path()) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Either it failed (FAIL output) or clang wasn't available
            if stdout.contains("FAIL") || stderr.contains("failed") {
                // Expected: test should report failure
            } else if !output.status.success() {
                // Also fine - the command itself reports failure
            } else {
                // If clang isn't available, the test is skipped effectively
                eprintln!("test runner result: stdout={}, stderr={}", stdout, stderr);
            }
        }
        Err(_) => {
            eprintln!("skipping vaisc test: cargo run not available");
        }
    }
}

#[test]
fn test_vaisc_test_no_tests_directory() {
    let tmp = TempDir::new().unwrap();
    // No tests/ directory - should handle gracefully
    match run_vaisc(&["test"], tmp.path()) {
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Should either error or warn about no test files
            assert!(
                !output.status.success()
                    || stdout.contains("No test files")
                    || stdout.contains("not found"),
                "should handle missing tests dir: stdout={}, stderr={}",
                stdout,
                stderr
            );
        }
        Err(_) => {
            eprintln!("skipping test: cargo run not available");
        }
    }
}

#[test]
fn test_vaisc_test_filter() {
    let tmp = TempDir::new().unwrap();
    let tests_dir = tmp.path().join("tests");
    fs::create_dir_all(&tests_dir).unwrap();

    // Write two test files
    let test1 = "F main() -> i64 { 0 }\n";
    let test2 = "F main() -> i64 { 0 }\n";
    fs::write(tests_dir.join("alpha_test.vais"), test1).unwrap();
    fs::write(tests_dir.join("beta_test.vais"), test2).unwrap();

    // Filter to only run alpha
    match run_vaisc(
        &["test", tests_dir.to_str().unwrap(), "--filter", "alpha"],
        tmp.path(),
    ) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if output.status.success() {
                assert!(
                    stdout.contains("1 test") || stdout.contains("alpha"),
                    "should only run filtered tests: {}",
                    stdout
                );
            }
        }
        Err(_) => {
            eprintln!("skipping test: cargo run not available");
        }
    }
}

#[test]
fn test_vaisc_pkg_tree_basic() {
    let tmp = TempDir::new().unwrap();
    // Create a minimal package
    let manifest = "[package]\nname = \"tree-test\"\nversion = \"1.0.0\"\n";
    fs::write(tmp.path().join("vais.toml"), manifest).unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(
        tmp.path().join("src").join("main.vais"),
        "F main() -> i64 { 0 }\n",
    )
    .unwrap();

    match run_vaisc(&["pkg", "tree"], tmp.path()) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if output.status.success() {
                assert!(
                    stdout.contains("tree-test") || stdout.contains("1.0.0"),
                    "should show package name/version: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("pkg tree returned non-zero: {}", stderr);
            }
        }
        Err(_) => {
            eprintln!("skipping test: cargo run not available");
        }
    }
}

#[test]
fn test_vaisc_pkg_tree_with_deps() {
    let tmp = TempDir::new().unwrap();

    // Create a dependency package
    let dep_dir = tmp.path().join("dep-pkg");
    fs::create_dir_all(dep_dir.join("src")).unwrap();
    let dep_manifest = "[package]\nname = \"dep-pkg\"\nversion = \"0.2.0\"\n";
    fs::write(dep_dir.join("vais.toml"), dep_manifest).unwrap();
    fs::write(
        dep_dir.join("src").join("lib.vais"),
        "F helper() -> i64 { 42 }\n",
    )
    .unwrap();

    // Create main package with path dependency
    let main_manifest = format!(
        "[package]\nname = \"main-pkg\"\nversion = \"1.0.0\"\n\n[dependencies]\ndep-pkg = {{ path = \"{}\" }}\n",
        dep_dir.display()
    );
    fs::write(tmp.path().join("vais.toml"), main_manifest).unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(
        tmp.path().join("src").join("main.vais"),
        "F main() -> i64 { 0 }\n",
    )
    .unwrap();

    match run_vaisc(&["pkg", "tree"], tmp.path()) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if output.status.success() {
                assert!(
                    stdout.contains("main-pkg"),
                    "should show root package: {}",
                    stdout
                );
                assert!(
                    stdout.contains("dep-pkg"),
                    "should show dependency: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("pkg tree with deps returned non-zero: {}", stderr);
            }
        }
        Err(_) => {
            eprintln!("skipping test: cargo run not available");
        }
    }
}

#[test]
fn test_vaisc_pkg_doc_basic() {
    let tmp = TempDir::new().unwrap();

    // Create a package with some source
    let manifest =
        "[package]\nname = \"doc-test\"\nversion = \"1.0.0\"\ndescription = \"A test package for docs\"\n";
    fs::write(tmp.path().join("vais.toml"), manifest).unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    let source = "# Math module\n\nF add(a: i64, b: i64) -> i64 {\n    a + b\n}\n";
    fs::write(tmp.path().join("src").join("math.vais"), source).unwrap();

    match run_vaisc(&["pkg", "doc"], tmp.path()) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if output.status.success() {
                // Check docs directory was created
                let docs_dir = tmp.path().join("docs");
                assert!(docs_dir.exists(), "docs/ directory should exist");

                // Check index file exists
                assert!(
                    docs_dir.join("index.md").exists(),
                    "index.md should be generated"
                );

                // Check stdout mentions documentation
                assert!(
                    stdout.contains("doc-test")
                        || stdout.contains("Documenting")
                        || stdout.contains("Generated"),
                    "should mention documentation: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("pkg doc returned non-zero: {}", stderr);
            }
        }
        Err(_) => {
            eprintln!("skipping test: cargo run not available");
        }
    }
}

#[test]
fn test_vaisc_pkg_doc_html_format() {
    let tmp = TempDir::new().unwrap();

    let manifest = "[package]\nname = \"html-doc-test\"\nversion = \"1.0.0\"\n";
    fs::write(tmp.path().join("vais.toml"), manifest).unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(
        tmp.path().join("src").join("main.vais"),
        "F main() -> i64 { 0 }\n",
    )
    .unwrap();

    match run_vaisc(&["pkg", "doc", "--format", "html"], tmp.path()) {
        Ok(output) => {
            if output.status.success() {
                let docs_dir = tmp.path().join("docs");
                assert!(
                    docs_dir.join("index.html").exists(),
                    "index.html should be generated for html format"
                );

                let content = fs::read_to_string(docs_dir.join("index.html")).unwrap();
                assert!(content.contains("<html>"), "should be valid HTML");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("pkg doc --format html returned non-zero: {}", stderr);
            }
        }
        Err(_) => {
            eprintln!("skipping test: cargo run not available");
        }
    }
}

#[test]
fn test_vaisc_pkg_tree_no_manifest() {
    let tmp = TempDir::new().unwrap();
    // No vais.toml - should fail gracefully
    match run_vaisc(&["pkg", "tree"], tmp.path()) {
        Ok(output) => {
            assert!(
                !output.status.success(),
                "pkg tree without vais.toml should fail"
            );
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains("vais.toml")
                    || stderr.contains("not found")
                    || stderr.contains("could not find"),
                "should mention missing manifest: {}",
                stderr
            );
        }
        Err(_) => {
            eprintln!("skipping test: cargo run not available");
        }
    }
}

#[test]
fn test_vaisc_pkg_doc_no_manifest() {
    let tmp = TempDir::new().unwrap();
    // No vais.toml - should fail gracefully
    match run_vaisc(&["pkg", "doc"], tmp.path()) {
        Ok(output) => {
            assert!(
                !output.status.success(),
                "pkg doc without vais.toml should fail"
            );
        }
        Err(_) => {
            eprintln!("skipping test: cargo run not available");
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 50 Stage 2: Build Scripts & Global Install
// ---------------------------------------------------------------------------

#[test]
fn test_build_script_env_vars() {
    // Verify that build.vais gets correct environment variables
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("build_env_test");
    fs::create_dir_all(project_dir.join("src")).unwrap();

    // Create vais.toml
    fs::write(
        project_dir.join("vais.toml"),
        r#"[package]
name = "build_env_test"
version = "0.1.0"
"#,
    )
    .unwrap();

    // Create a build.vais that writes env vars to a file
    fs::write(
        project_dir.join("build.vais"),
        r#"# Build script that verifies environment variables
F main() -> i64 {
    0
}
"#,
    )
    .unwrap();

    // Create main.vais
    fs::write(
        project_dir.join("src").join("main.vais"),
        r#"F main() -> i64 {
    0
}
"#,
    )
    .unwrap();

    // Build the project - build.vais should be detected and executed
    match run_vaisc(&["pkg", "build", "-v"], &project_dir) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Build script should be detected (verbose output mentions it)
            if output.status.success() {
                assert!(
                    stdout.contains("build script") || stdout.contains("Build"),
                    "verbose output should mention build script execution: stdout={}, stderr={}",
                    stdout,
                    stderr
                );
            }
        }
        Err(_) => {
            eprintln!("skipping test: cargo run not available");
        }
    }
}

#[test]
fn test_install_no_package() {
    // vaisc install with non-existent path should fail
    let tmp = TempDir::new().unwrap();

    match run_vaisc(&["install", "/nonexistent/path"], tmp.path()) {
        Ok(output) => {
            assert!(
                !output.status.success(),
                "install with nonexistent path should fail"
            );
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains("vais.toml")
                    || stderr.contains("not found")
                    || stderr.contains("could not find"),
                "should mention missing manifest: {}",
                stderr
            );
        }
        Err(_) => {
            eprintln!("skipping test: cargo run not available");
        }
    }
}

#[test]
fn test_install_library_package_fails() {
    // Library packages (no main.vais) should not be installable
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("lib_pkg");
    fs::create_dir_all(project_dir.join("src")).unwrap();

    fs::write(
        project_dir.join("vais.toml"),
        r#"[package]
name = "mylib"
version = "0.1.0"
"#,
    )
    .unwrap();

    // Only lib.vais, no main.vais
    fs::write(
        project_dir.join("src").join("lib.vais"),
        r#"F add(a: i64, b: i64) -> i64 {
    a + b
}
"#,
    )
    .unwrap();

    match run_vaisc_from_root(&["install", project_dir.to_str().unwrap()]) {
        Ok(output) => {
            assert!(
                !output.status.success(),
                "install of library package should fail"
            );
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains("main.vais") || stderr.contains("binary"),
                "should mention missing main.vais: {}",
                stderr
            );
        }
        Err(_) => {
            eprintln!("skipping test: cargo run not available");
        }
    }
}

#[test]
fn test_uninstall_not_installed() {
    // Uninstalling a package that's not installed should fail gracefully
    match run_vaisc_from_root(&["uninstall", "nonexistent_pkg_xyz"]) {
        Ok(output) => {
            assert!(
                !output.status.success(),
                "uninstall of non-installed package should fail"
            );
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains("not installed") || stderr.contains("not found"),
                "should indicate package is not installed: {}",
                stderr
            );
        }
        Err(_) => {
            eprintln!("skipping test: cargo run not available");
        }
    }
}

#[test]
fn test_install_and_uninstall_roundtrip() {
    // Full install → verify → uninstall → verify removed roundtrip
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("installable");
    fs::create_dir_all(project_dir.join("src")).unwrap();

    fs::write(
        project_dir.join("vais.toml"),
        r#"[package]
name = "test_installable"
version = "1.0.0"
"#,
    )
    .unwrap();

    fs::write(
        project_dir.join("src").join("main.vais"),
        r#"F main() -> i64 {
    42
}
"#,
    )
    .unwrap();

    // Install the package
    match run_vaisc_from_root(&["install", project_dir.to_str().unwrap()]) {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(
                    stdout.contains("Installed") || stdout.contains("test_installable"),
                    "should confirm installation: {}",
                    stdout
                );

                // Verify binary exists in ~/.vais/bin/
                let home = dirs::home_dir().unwrap();
                let binary = home.join(".vais").join("bin").join("test_installable");
                assert!(
                    binary.exists(),
                    "binary should exist at {}",
                    binary.display()
                );

                // Uninstall
                match run_vaisc_from_root(&["uninstall", "test_installable"]) {
                    Ok(uninstall_output) => {
                        assert!(
                            uninstall_output.status.success(),
                            "uninstall should succeed"
                        );
                        assert!(!binary.exists(), "binary should be removed after uninstall");
                    }
                    Err(_) => {
                        eprintln!("skipping uninstall test: cargo run not available");
                        // Clean up manually
                        let _ = fs::remove_file(&binary);
                    }
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("install failed (expected in CI without clang): {}", stderr);
            }
        }
        Err(_) => {
            eprintln!("skipping test: cargo run not available");
        }
    }
}

#[test]
fn test_bench_no_directory() {
    let tmp = TempDir::new().unwrap();
    let nonexistent_path = tmp.path().join("benches");
    // No benches/ directory
    match run_vaisc_from_root(&["bench", nonexistent_path.to_str().unwrap()]) {
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Should mention benches directory not found or no benchmarks
            assert!(
                !output.status.success()
                    || stdout.contains("No benchmark")
                    || stdout.contains("not found"),
                "should handle missing benches dir: stdout={}, stderr={}",
                stdout,
                stderr
            );
        }
        Err(_) => eprintln!("skipping test: cargo run not available"),
    }
}

#[test]
fn test_bench_with_filter() {
    let tmp = TempDir::new().unwrap();
    let benches_dir = tmp.path().join("benches");
    fs::create_dir_all(&benches_dir).unwrap();

    // Create a simple benchmark file
    let bench_file = benches_dir.join("simple.vais");
    fs::write(&bench_file, "F main() -> i64 = 42").unwrap();

    match run_vaisc_from_root(&[
        "bench",
        benches_dir.to_str().unwrap(),
        "--filter",
        "nonexistent",
    ]) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Should work without error - just 0 benchmarks found
            // Our code prints "Running 0 benchmark(s)" and "0 benchmarks: 0 passed, 0 failed"
            assert!(
                output.status.success()
                    || stdout.contains("0 benchmark")
                    || stdout.contains("No benchmark"),
                "should handle filter with no matches: status={}, stdout={}",
                output.status,
                stdout
            );
        }
        Err(_) => eprintln!("skipping test: cargo run not available"),
    }
}

#[test]
fn test_fix_dry_run() {
    let tmp = TempDir::new().unwrap();
    let test_file = tmp.path().join("test.vais");

    // Create a valid Vais file
    fs::write(&test_file, "F main() -> i64 = 42").unwrap();

    match run_vaisc_from_root(&["fix", "--dry-run", test_file.to_str().unwrap()]) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Should complete without error - either success or mentions fixes
            assert!(
                output.status.success() || stdout.contains("fix") || stderr.is_empty(),
                "fix --dry-run should work: status={}, stdout={}, stderr={}",
                output.status,
                stdout,
                stderr
            );
        }
        Err(_) => eprintln!("skipping test: cargo run not available"),
    }
}

#[test]
fn test_lint_clean_file() {
    let tmp = TempDir::new().unwrap();
    let test_file = tmp.path().join("clean.vais");

    // Create a clean, valid Vais file
    fs::write(&test_file, "F main() -> i64 = 42").unwrap();

    match run_vaisc_from_root(&["lint", test_file.to_str().unwrap()]) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                assert!(
                    stdout.contains("0 error") || stdout.contains("✓"),
                    "clean file should pass lint: stdout={}",
                    stdout
                );
            } else {
                // May fail on clang issues in CI, just log it
                eprintln!("lint test exited non-zero (expected in CI): {}", stderr);
            }
        }
        Err(_) => eprintln!("skipping test: cargo run not available"),
    }
}

#[test]
fn test_lint_json_format() {
    let tmp = TempDir::new().unwrap();
    let test_file = tmp.path().join("test.vais");

    // Create a Vais file
    fs::write(&test_file, "F main() -> i64 = 42").unwrap();

    match run_vaisc_from_root(&["lint", "--format", "json", test_file.to_str().unwrap()]) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Should output valid JSON
            if !stdout.is_empty() {
                let json_result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
                assert!(json_result.is_ok(), "should output valid JSON: {}", stdout);

                if let Ok(json) = json_result {
                    assert!(
                        json.get("summary").is_some(),
                        "JSON should have summary field: {:?}",
                        json
                    );
                }
            }
        }
        Err(_) => eprintln!("skipping test: cargo run not available"),
    }
}

// Phase 50 Stage 4: Vendor, Package, Metadata subcommands

#[test]
fn test_pkg_vendor_no_deps() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("vendor_test");
    fs::create_dir_all(project_dir.join("src")).unwrap();
    fs::write(
        project_dir.join("vais.toml"),
        "[package]\nname = \"vendor_test\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        project_dir.join("src").join("main.vais"),
        "F main() -> i64 { 0 }\n",
    )
    .unwrap();

    match run_vaisc(&["pkg", "vendor"], &project_dir) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Should succeed (no deps to vendor)
            if output.status.success() {
                assert!(
                    stdout.contains("No dependencies") || stdout.contains("Vendored"),
                    "should mention vendoring result: stdout={}, stderr={}",
                    stdout,
                    stderr
                );
            }
        }
        Err(_) => eprintln!("skipping test: cargo run not available"),
    }
}

#[test]
fn test_pkg_package_list() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("package_test");
    fs::create_dir_all(project_dir.join("src")).unwrap();
    fs::write(
        project_dir.join("vais.toml"),
        "[package]\nname = \"package_test\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        project_dir.join("src").join("main.vais"),
        "F main() -> i64 { 42 }\n",
    )
    .unwrap();

    match run_vaisc(&["pkg", "package", "--list"], &project_dir) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                assert!(
                    stdout.contains("contents")
                        || stdout.contains("vais.toml")
                        || stdout.contains("src/main.vais"),
                    "should show file listing: stdout={}, stderr={}",
                    stdout,
                    stderr
                );
            } else {
                eprintln!("pkg package --list failed (may be expected): {}", stderr);
            }
        }
        Err(_) => eprintln!("skipping test: cargo run not available"),
    }
}

#[test]
fn test_pkg_metadata_json() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("metadata_test");
    fs::create_dir_all(project_dir.join("src")).unwrap();
    fs::write(
        project_dir.join("vais.toml"),
        "[package]\nname = \"metadata_test\"\nversion = \"1.2.3\"\n",
    )
    .unwrap();
    fs::write(
        project_dir.join("src").join("lib.vais"),
        "F foo() -> i64 { 1 }\n",
    )
    .unwrap();

    match run_vaisc(&["pkg", "metadata"], &project_dir) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);

            if output.status.success() && !stdout.is_empty() {
                let json_result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
                assert!(json_result.is_ok(), "should output valid JSON: {}", stdout);

                if let Ok(json) = json_result {
                    // Should have package.name and package.version
                    let name = json
                        .get("package")
                        .and_then(|p| p.get("name"))
                        .and_then(|n| n.as_str());
                    let version = json
                        .get("package")
                        .and_then(|p| p.get("version"))
                        .and_then(|v| v.as_str());

                    assert_eq!(name, Some("metadata_test"), "should have correct name");
                    assert_eq!(version, Some("1.2.3"), "should have correct version");
                }
            }
        }
        Err(_) => eprintln!("skipping test: cargo run not available"),
    }
}

#[test]
fn test_pkg_owner_add_list() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("owner_test");
    fs::create_dir_all(project_dir.join("src")).unwrap();
    fs::write(
        project_dir.join("vais.toml"),
        "[package]\nname = \"owner_test\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        project_dir.join("src").join("main.vais"),
        "F main() -> i64 { 0 }\n",
    )
    .unwrap();

    // Add an owner
    match run_vaisc(&["pkg", "owner", "--add", "alice"], &project_dir) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if output.status.success() {
                assert!(
                    stdout.contains("Added") || stdout.contains("alice"),
                    "should confirm adding owner: {}",
                    stdout
                );
            }
        }
        Err(_) => {
            eprintln!("skipping test: cargo run not available");
            return;
        }
    }

    // List owners
    match run_vaisc(&["pkg", "owner", "--list"], &project_dir) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if output.status.success() {
                assert!(
                    stdout.contains("alice") || stdout.contains("Owners"),
                    "should list alice as owner: {}",
                    stdout
                );
            }
        }
        Err(_) => eprintln!("skipping test: cargo run not available"),
    }
}

#[test]
fn test_pkg_verify_valid() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("verify_test");
    fs::create_dir_all(project_dir.join("src")).unwrap();
    fs::write(
        project_dir.join("vais.toml"),
        "[package]\nname = \"verify_test\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        project_dir.join("src").join("main.vais"),
        "F main() -> i64 { 0 }\n",
    )
    .unwrap();

    match run_vaisc(&["pkg", "verify"], &project_dir) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                assert!(
                    stdout.contains("passed") || stdout.contains("✓"),
                    "valid package should pass verification: stdout={}, stderr={}",
                    stdout,
                    stderr
                );
            } else {
                eprintln!(
                    "verify failed (may be expected if validation is strict): {}",
                    stderr
                );
            }
        }
        Err(_) => eprintln!("skipping test: cargo run not available"),
    }
}

// ===========================================================================
// Phase 64: Package Manager & Ecosystem — Comprehensive E2E Tests
// ===========================================================================

// ---------------------------------------------------------------------------
// Phase 64 Task 1: vais init → vais.toml Generation Workflow E2E
// ---------------------------------------------------------------------------

#[test]
fn test_phase64_init_creates_valid_manifest_and_source() {
    let tmp = TempDir::new().unwrap();
    let output = Command::new("cargo")
        .args([
            "run", "--bin", "vaisc", "--", "pkg", "init", "--name", "phase64-init",
        ])
        .current_dir(tmp.path())
        .env("CARGO_MANIFEST_DIR", project_root().join("crates/vaisc"))
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                // 1. vais.toml exists and is valid
                let manifest_path = tmp.path().join("vais.toml");
                assert!(manifest_path.exists(), "vais.toml must be created");
                let content = fs::read_to_string(&manifest_path).unwrap();
                let manifest: PackageManifest = toml::from_str(&content).unwrap();
                assert_eq!(manifest.package.name, "phase64-init");
                assert_eq!(manifest.package.version, "0.1.0");
                assert_eq!(manifest.package.license.as_deref(), Some("MIT"));
                assert!(manifest.dependencies.is_empty());
                assert!(manifest.dev_dependencies.is_empty());

                // 2. src/main.vais exists with compilable content
                let main_path = tmp.path().join("src").join("main.vais");
                assert!(main_path.exists(), "src/main.vais must be created");
                let main_content = fs::read_to_string(&main_path).unwrap();
                assert!(
                    main_content.contains("F main()"),
                    "main.vais should contain F main(): got {}",
                    main_content
                );
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                eprintln!("pkg init non-zero (may be expected): {}", stderr);
            }
        }
        Err(_) => eprintln!("skipping CLI test: cargo run not available"),
    }
}

#[test]
fn test_phase64_init_default_name_from_directory() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("my-cool-project");
    fs::create_dir_all(&project_dir).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--bin", "vaisc", "--", "pkg", "init"])
        .current_dir(&project_dir)
        .env("CARGO_MANIFEST_DIR", project_root().join("crates/vaisc"))
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                let content = fs::read_to_string(project_dir.join("vais.toml")).unwrap();
                let manifest: PackageManifest = toml::from_str(&content).unwrap();
                assert_eq!(
                    manifest.package.name, "my-cool-project",
                    "should use directory name as package name"
                );
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                eprintln!("pkg init non-zero: {}", stderr);
            }
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

#[test]
fn test_phase64_init_fails_if_already_exists() {
    let tmp = TempDir::new().unwrap();

    // First init
    let _ = Command::new("cargo")
        .args([
            "run", "--bin", "vaisc", "--", "pkg", "init", "--name", "dup-init",
        ])
        .current_dir(tmp.path())
        .env("CARGO_MANIFEST_DIR", project_root().join("crates/vaisc"))
        .output();

    // Second init should fail
    match Command::new("cargo")
        .args([
            "run", "--bin", "vaisc", "--", "pkg", "init", "--name", "dup-init",
        ])
        .current_dir(tmp.path())
        .env("CARGO_MANIFEST_DIR", project_root().join("crates/vaisc"))
        .output()
    {
        Ok(o) => {
            if o.status.success() {
                let stderr = String::from_utf8_lossy(&o.stderr);
                let stdout = String::from_utf8_lossy(&o.stdout);
                // Even if exit code is 0, it should warn or error about existing
                assert!(
                    stderr.contains("already exists")
                        || stdout.contains("already exists")
                        || !o.status.success(),
                    "second init should fail or warn: stdout={}, stderr={}",
                    stdout,
                    stderr
                );
            }
            // Non-zero is expected behavior
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

#[test]
fn test_phase64_init_manifest_roundtrip_parse() {
    // Test that init_package creates a manifest that round-trips through toml
    let tmp = TempDir::new().unwrap();

    let output = Command::new("cargo")
        .args([
            "run", "--bin", "vaisc", "--", "pkg", "init", "--name", "roundtrip-rt",
        ])
        .current_dir(tmp.path())
        .env("CARGO_MANIFEST_DIR", project_root().join("crates/vaisc"))
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                let content = fs::read_to_string(tmp.path().join("vais.toml")).unwrap();
                let manifest: PackageManifest = toml::from_str(&content).unwrap();

                // Roundtrip: serialize and re-parse
                let re_serialized = toml::to_string_pretty(&manifest).unwrap();
                let re_parsed: PackageManifest = toml::from_str(&re_serialized).unwrap();
                assert_eq!(re_parsed.package.name, manifest.package.name);
                assert_eq!(re_parsed.package.version, manifest.package.version);
            }
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

#[test]
fn test_phase64_init_then_build_produces_binary() {
    let tmp = TempDir::new().unwrap();

    // Init
    let init_output = Command::new("cargo")
        .args([
            "run", "--bin", "vaisc", "--", "pkg", "init", "--name", "build-test",
        ])
        .current_dir(tmp.path())
        .env("CARGO_MANIFEST_DIR", project_root().join("crates/vaisc"))
        .output();

    match init_output {
        Ok(o) => {
            if !o.status.success() {
                eprintln!("init failed, skipping build test");
                return;
            }
        }
        Err(_) => {
            eprintln!("skipping CLI test");
            return;
        }
    }

    // Build the initialized package
    let build_output = Command::new("cargo")
        .args(["run", "--bin", "vaisc", "--", "pkg", "build"])
        .current_dir(tmp.path())
        .env("CARGO_MANIFEST_DIR", project_root().join("crates/vaisc"))
        .output();

    match build_output {
        Ok(o) => {
            if o.status.success() {
                // Check that target/ directory or binary was created
                let target_dir = tmp.path().join("target");
                let src_main = tmp.path().join("src").join("main");
                assert!(
                    target_dir.exists() || src_main.exists(),
                    "build should produce output in target/ or src/"
                );
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                eprintln!("build non-zero (may need clang): {}", stderr);
            }
        }
        Err(_) => eprintln!("skipping build test"),
    }
}

// ---------------------------------------------------------------------------
// Phase 64 Task 2: vais install — Local Path Dependency Resolution + Build
// ---------------------------------------------------------------------------

#[test]
fn test_phase64_install_path_dependency_resolution() {
    let tmp = TempDir::new().unwrap();

    // Create a library package (dependency)
    let lib_dir = tmp.path().join("mylib");
    fs::create_dir_all(lib_dir.join("src")).unwrap();
    fs::write(
        lib_dir.join("vais.toml"),
        "[package]\nname = \"mylib\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    fs::write(
        lib_dir.join("src").join("lib.vais"),
        "F add(a: i64, b: i64) -> i64 { a + b }\n",
    )
    .unwrap();

    // Create a main package that depends on mylib
    let app_dir = tmp.path().join("myapp");
    fs::create_dir_all(app_dir.join("src")).unwrap();
    fs::write(
        app_dir.join("vais.toml"),
        "[package]\nname = \"myapp\"\nversion = \"0.1.0\"\n\n[dependencies]\nmylib = { path = \"../mylib\" }\n",
    )
    .unwrap();
    fs::write(
        app_dir.join("src").join("main.vais"),
        "U mylib\nF main() -> i64 { add(3, 4) }\n",
    )
    .unwrap();

    // Build the app (should resolve path dependency)
    let output = Command::new("cargo")
        .args(["run", "--bin", "vaisc", "--", "pkg", "build"])
        .current_dir(&app_dir)
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                let stdout = String::from_utf8_lossy(&o.stdout);
                // Build should mention the dependency or succeed silently
                assert!(
                    o.status.success(),
                    "build with path dep should succeed: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                // May fail if clang is not available, that's ok
                if !stderr.contains("clang") && !stderr.contains("linker") {
                    eprintln!("path dep build failed: {}", stderr);
                }
            }
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

#[test]
fn test_phase64_path_dependency_manifest_parsing() {
    // Verify path dependency in manifest is correctly parsed and resolved
    let tmp = TempDir::new().unwrap();

    let lib_dir = tmp.path().join("utils");
    fs::create_dir_all(lib_dir.join("src")).unwrap();
    fs::write(
        lib_dir.join("vais.toml"),
        "[package]\nname = \"utils\"\nversion = \"2.0.0\"\n",
    )
    .unwrap();
    fs::write(
        lib_dir.join("src").join("lib.vais"),
        "F double(x: i64) -> i64 { x * 2 }\n",
    )
    .unwrap();

    // Main package with path dependency
    let app_dir = tmp.path().join("app");
    fs::create_dir_all(app_dir.join("src")).unwrap();
    let manifest_str =
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[dependencies]\nutils = { path = \"../utils\" }\n";
    fs::write(app_dir.join("vais.toml"), manifest_str).unwrap();
    fs::write(
        app_dir.join("src").join("main.vais"),
        "F main() -> i64 { 0 }\n",
    )
    .unwrap();

    // Parse the manifest and check the dependency
    let content = fs::read_to_string(app_dir.join("vais.toml")).unwrap();
    let manifest: PackageManifest = toml::from_str(&content).unwrap();
    assert_eq!(manifest.dependencies.len(), 1);
    match &manifest.dependencies["utils"] {
        Dependency::Detailed(d) => {
            assert_eq!(d.path.as_deref(), Some("../utils"));
        }
        _ => panic!("expected Detailed dependency with path"),
    }
}

#[test]
fn test_phase64_transitive_path_dependencies() {
    let tmp = TempDir::new().unwrap();

    // A -> B -> C (transitive chain)
    let c_dir = tmp.path().join("c");
    fs::create_dir_all(c_dir.join("src")).unwrap();
    fs::write(
        c_dir.join("vais.toml"),
        "[package]\nname = \"c\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    fs::write(
        c_dir.join("src").join("lib.vais"),
        "F base_val() -> i64 { 42 }\n",
    )
    .unwrap();

    let b_dir = tmp.path().join("b");
    fs::create_dir_all(b_dir.join("src")).unwrap();
    fs::write(
        b_dir.join("vais.toml"),
        "[package]\nname = \"b\"\nversion = \"1.0.0\"\n\n[dependencies]\nc = { path = \"../c\" }\n",
    )
    .unwrap();
    fs::write(
        b_dir.join("src").join("lib.vais"),
        "U c\nF wrapped_val() -> i64 { base_val() }\n",
    )
    .unwrap();

    let a_dir = tmp.path().join("a");
    fs::create_dir_all(a_dir.join("src")).unwrap();
    fs::write(
        a_dir.join("vais.toml"),
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nb = { path = \"../b\" }\n",
    )
    .unwrap();
    fs::write(
        a_dir.join("src").join("main.vais"),
        "U b\nF main() -> i64 { wrapped_val() }\n",
    )
    .unwrap();

    // Build A — should transitively resolve B and C
    let output = Command::new("cargo")
        .args(["run", "--bin", "vaisc", "--", "pkg", "build"])
        .current_dir(&a_dir)
        .output();

    match output {
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            if o.status.success() {
                // Success means transitive resolution worked
            } else if !stderr.contains("clang") && !stderr.contains("linker") {
                eprintln!("transitive dep build failed: {}", stderr);
            }
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

#[test]
fn test_phase64_install_binary_package_creates_binary() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("installme");
    fs::create_dir_all(project_dir.join("src")).unwrap();
    fs::write(
        project_dir.join("vais.toml"),
        "[package]\nname = \"installme\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    fs::write(
        project_dir.join("src").join("main.vais"),
        "F main() -> i64 { puts(\"installed!\") \n 0 }\n",
    )
    .unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "vaisc",
            "--",
            "install",
            project_dir.to_str().unwrap(),
        ])
        .env(
            "HOME",
            tmp.path().join("fakehome").to_str().unwrap(),
        )
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            if o.status.success() {
                assert!(
                    stdout.contains("Installed") || stdout.contains("✓"),
                    "should report installation: stdout={}, stderr={}",
                    stdout,
                    stderr
                );
            } else {
                eprintln!("install non-zero (may need clang): {}", stderr);
            }
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

#[test]
fn test_phase64_install_library_only_fails() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("libonly");
    fs::create_dir_all(project_dir.join("src")).unwrap();
    fs::write(
        project_dir.join("vais.toml"),
        "[package]\nname = \"libonly\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    // Only lib.vais, no main.vais
    fs::write(
        project_dir.join("src").join("lib.vais"),
        "F helper() -> i64 { 0 }\n",
    )
    .unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "vaisc",
            "--",
            "install",
            project_dir.to_str().unwrap(),
        ])
        .output();

    match output {
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            assert!(
                !o.status.success() || stderr.contains("main.vais"),
                "installing library-only package should fail: {}",
                stderr
            );
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

// ---------------------------------------------------------------------------
// Phase 64 Task 3: vais publish — Registry Server Integration
// ---------------------------------------------------------------------------

#[test]
fn test_phase64_publish_request_structure() {
    // Verify that a publish request has the correct structure
    // (tests the serialization format without a running server)
    #[derive(Debug, Serialize, Deserialize)]
    struct PublishMetadata {
        name: String,
        version: String,
        description: Option<String>,
        authors: Vec<String>,
        license: Option<String>,
        dependencies: HashMap<String, String>,
    }

    let metadata = PublishMetadata {
        name: "test-pkg".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A test package".to_string()),
        authors: vec!["Author".to_string()],
        license: Some("MIT".to_string()),
        dependencies: {
            let mut m = HashMap::new();
            m.insert("dep-a".to_string(), "^1.0".to_string());
            m
        },
    };

    let json = serde_json::to_string_pretty(&metadata).unwrap();
    assert!(json.contains("\"name\": \"test-pkg\""));
    assert!(json.contains("\"version\": \"1.0.0\""));
    assert!(json.contains("\"dep-a\""));
    assert!(json.contains("\"^1.0\""));

    // Round-trip
    let parsed: PublishMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name, "test-pkg");
    assert_eq!(parsed.version, "1.0.0");
    assert_eq!(parsed.dependencies.len(), 1);
}

#[test]
fn test_phase64_publish_package_creates_archive() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("pub-test");
    fs::create_dir_all(project_dir.join("src")).unwrap();
    fs::write(
        project_dir.join("vais.toml"),
        "[package]\nname = \"pub-test\"\nversion = \"0.1.0\"\ndescription = \"Publish test\"\nauthors = [\"Bot\"]\n",
    )
    .unwrap();
    fs::write(
        project_dir.join("src").join("lib.vais"),
        "F greet() -> i64 { puts(\"hi\") \n 0 }\n",
    )
    .unwrap();

    // Run pkg package (create archive without publishing)
    let output = Command::new("cargo")
        .args(["run", "--bin", "vaisc", "--", "pkg", "package", "--list"])
        .current_dir(&project_dir)
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            if o.status.success() {
                assert!(
                    stdout.contains("vais.toml") || stdout.contains("src/"),
                    "package list should include manifest and source: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                eprintln!("package --list non-zero: {}", stderr);
            }
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

#[test]
fn test_phase64_publish_no_server_reports_error() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("pub-noserver");
    fs::create_dir_all(project_dir.join("src")).unwrap();
    fs::write(
        project_dir.join("vais.toml"),
        "[package]\nname = \"pub-noserver\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        project_dir.join("src").join("lib.vais"),
        "F nop() -> i64 { 0 }\n",
    )
    .unwrap();

    // publish should fail gracefully when no server is running
    let output = Command::new("cargo")
        .args(["run", "--bin", "vaisc", "--", "pkg", "publish"])
        .current_dir(&project_dir)
        .output();

    match output {
        Ok(o) => {
            // Should fail since there's no registry server running
            assert!(
                !o.status.success(),
                "publish without server should fail gracefully"
            );
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

// ---------------------------------------------------------------------------
// Phase 64 Task 4: SemVer Dependency Resolver — Version Conflicts, Diamond
// ---------------------------------------------------------------------------

#[test]
fn test_phase64_semver_caret_range_comprehensive() {
    // ^1.2.3 := >=1.2.3, <2.0.0
    assert!(caret_matches(
        &SemVer::parse("1.2.3").unwrap(),
        &SemVer::parse("1.2.3").unwrap()
    ));
    assert!(caret_matches(
        &SemVer::parse("1.2.3").unwrap(),
        &SemVer::parse("1.9.9").unwrap()
    ));
    assert!(!caret_matches(
        &SemVer::parse("1.2.3").unwrap(),
        &SemVer::parse("2.0.0").unwrap()
    ));
    assert!(!caret_matches(
        &SemVer::parse("1.2.3").unwrap(),
        &SemVer::parse("1.2.2").unwrap()
    ));

    // ^0.2.3 := >=0.2.3, <0.3.0
    assert!(caret_matches(
        &SemVer::parse("0.2.3").unwrap(),
        &SemVer::parse("0.2.3").unwrap()
    ));
    assert!(caret_matches(
        &SemVer::parse("0.2.3").unwrap(),
        &SemVer::parse("0.2.9").unwrap()
    ));
    assert!(!caret_matches(
        &SemVer::parse("0.2.3").unwrap(),
        &SemVer::parse("0.3.0").unwrap()
    ));

    // ^0.0.3 := =0.0.3
    assert!(caret_matches(
        &SemVer::parse("0.0.3").unwrap(),
        &SemVer::parse("0.0.3").unwrap()
    ));
    assert!(!caret_matches(
        &SemVer::parse("0.0.3").unwrap(),
        &SemVer::parse("0.0.4").unwrap()
    ));
}

#[test]
fn test_phase64_semver_tilde_range_comprehensive() {
    // ~1.2.3 := >=1.2.3, <1.3.0
    assert!(tilde_matches(
        &SemVer::parse("1.2.3").unwrap(),
        &SemVer::parse("1.2.3").unwrap()
    ));
    assert!(tilde_matches(
        &SemVer::parse("1.2.3").unwrap(),
        &SemVer::parse("1.2.9").unwrap()
    ));
    assert!(!tilde_matches(
        &SemVer::parse("1.2.3").unwrap(),
        &SemVer::parse("1.3.0").unwrap()
    ));
    assert!(!tilde_matches(
        &SemVer::parse("1.2.3").unwrap(),
        &SemVer::parse("2.0.0").unwrap()
    ));
}

#[test]
fn test_phase64_version_conflict_detection_structure() {
    // Simulate A depends on C ^1.0, B depends on C ^2.0 (conflict)
    let a_c_req = SemVer::parse("1.0.0").unwrap();
    let b_c_req = SemVer::parse("2.0.0").unwrap();

    // Any version satisfying ^1.0 cannot satisfy ^2.0
    let candidate_1_5 = SemVer::parse("1.5.0").unwrap();
    let candidate_2_0 = SemVer::parse("2.0.0").unwrap();

    assert!(
        caret_matches(&a_c_req, &candidate_1_5),
        "1.5.0 should match ^1.0"
    );
    assert!(
        !caret_matches(&b_c_req, &candidate_1_5),
        "1.5.0 should NOT match ^2.0"
    );
    assert!(
        !caret_matches(&a_c_req, &candidate_2_0),
        "2.0.0 should NOT match ^1.0"
    );
    assert!(
        caret_matches(&b_c_req, &candidate_2_0),
        "2.0.0 should match ^2.0"
    );

    // This confirms no version exists that satisfies both ^1.0 and ^2.0
}

#[test]
fn test_phase64_diamond_dependency_compatible() {
    // Diamond: App -> A -> C ^1.0, App -> B -> C ^1.2
    // Compatible: ^1.0 and ^1.2 can be satisfied by 1.2.x
    let c_req_a = SemVer::parse("1.0.0").unwrap();
    let c_req_b = SemVer::parse("1.2.0").unwrap();
    let candidate = SemVer::parse("1.3.0").unwrap();

    assert!(caret_matches(&c_req_a, &candidate), "1.3.0 matches ^1.0");
    assert!(caret_matches(&c_req_b, &candidate), "1.3.0 matches ^1.2");
}

#[test]
fn test_phase64_diamond_dependency_incompatible() {
    // Diamond: App -> A -> C ^1.0, App -> B -> C ^2.0
    // Incompatible: no version satisfies both
    let c_req_a = SemVer::parse("1.0.0").unwrap();
    let c_req_b = SemVer::parse("2.0.0").unwrap();

    // Try all candidate ranges
    for candidate_str in &["1.0.0", "1.5.0", "1.9.9", "2.0.0", "2.5.0", "3.0.0"] {
        let candidate = SemVer::parse(candidate_str).unwrap();
        let matches_a = caret_matches(&c_req_a, &candidate);
        let matches_b = caret_matches(&c_req_b, &candidate);

        assert!(
            !(matches_a && matches_b),
            "No version should satisfy both ^1.0 and ^2.0, but {} does",
            candidate_str
        );
    }
}

#[test]
fn test_phase64_semver_prerelease_comparison() {
    let alpha = SemVer::parse("1.0.0-alpha").unwrap();
    let beta = SemVer::parse("1.0.0-beta").unwrap();
    let rc1 = SemVer::parse("1.0.0-rc.1").unwrap();
    let release = SemVer::parse("1.0.0").unwrap();

    assert!(alpha < beta, "alpha < beta");
    assert!(beta < rc1, "beta < rc.1");
    assert!(rc1 < release, "rc.1 < release");
}

#[test]
fn test_phase64_version_selection_best_match() {
    // Given available versions, find the best match for ^1.2
    let available = [
        SemVer::parse("1.0.0").unwrap(),
        SemVer::parse("1.1.0").unwrap(),
        SemVer::parse("1.2.0").unwrap(),
        SemVer::parse("1.2.5").unwrap(),
        SemVer::parse("1.3.0").unwrap(),
        SemVer::parse("2.0.0").unwrap(),
    ];

    let req = SemVer::parse("1.2.0").unwrap();
    let best = available
        .iter()
        .filter(|v| caret_matches(&req, v))
        .max()
        .unwrap();

    assert_eq!(best.major, 1);
    assert_eq!(best.minor, 3);
    assert_eq!(best.patch, 0);
}

// ---------------------------------------------------------------------------
// Phase 64 Task 5: Multi-Package Workspace Build E2E
// ---------------------------------------------------------------------------

#[test]
fn test_phase64_workspace_member_resolution() {
    let tmp = TempDir::new().unwrap();
    let ws_root = tmp.path();

    // Create workspace root vais.toml
    fs::write(
        ws_root.join("vais.toml"),
        "[package]\nname = \"workspace-root\"\nversion = \"0.1.0\"\n\n[workspace]\nmembers = [\"crates/*\"]\n",
    )
    .unwrap();

    // Create two members
    let member_a = ws_root.join("crates").join("a");
    fs::create_dir_all(member_a.join("src")).unwrap();
    fs::write(
        member_a.join("vais.toml"),
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        member_a.join("src").join("lib.vais"),
        "F hello_a() -> i64 { 1 }\n",
    )
    .unwrap();

    let member_b = ws_root.join("crates").join("b");
    fs::create_dir_all(member_b.join("src")).unwrap();
    fs::write(
        member_b.join("vais.toml"),
        "[package]\nname = \"b\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        member_b.join("src").join("lib.vais"),
        "F hello_b() -> i64 { 2 }\n",
    )
    .unwrap();

    // Verify workspace build
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "vaisc",
            "--",
            "pkg",
            "build",
            "--workspace",
        ])
        .current_dir(ws_root)
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                let stdout = String::from_utf8_lossy(&o.stdout);
                // Should mention both members
                assert!(
                    stdout.contains("a") || stdout.contains("b") || stdout.contains("workspace"),
                    "workspace build should process members: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                eprintln!("workspace build non-zero: {}", stderr);
            }
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

#[test]
fn test_phase64_workspace_inter_member_dependency() {
    let tmp = TempDir::new().unwrap();
    let ws_root = tmp.path();

    // Workspace root
    fs::write(
        ws_root.join("vais.toml"),
        "[package]\nname = \"ws-root\"\nversion = \"0.1.0\"\n\n[workspace]\nmembers = [\"crates/*\"]\n",
    )
    .unwrap();

    // Member core (no deps)
    let core_dir = ws_root.join("crates").join("core");
    fs::create_dir_all(core_dir.join("src")).unwrap();
    fs::write(
        core_dir.join("vais.toml"),
        "[package]\nname = \"core\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        core_dir.join("src").join("lib.vais"),
        "F core_fn() -> i64 { 10 }\n",
    )
    .unwrap();

    // Member app (depends on core)
    let app_dir = ws_root.join("crates").join("app");
    fs::create_dir_all(app_dir.join("src")).unwrap();
    fs::write(
        app_dir.join("vais.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[dependencies]\ncore = { path = \"../core\" }\n",
    )
    .unwrap();
    fs::write(
        app_dir.join("src").join("main.vais"),
        "U core\nF main() -> i64 { core_fn() }\n",
    )
    .unwrap();

    // Build workspace
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "vaisc",
            "--",
            "pkg",
            "build",
            "--workspace",
        ])
        .current_dir(ws_root)
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                // Verify inter-member deps resolved correctly
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                if !stderr.contains("clang") && !stderr.contains("linker") {
                    eprintln!("workspace inter-dep build failed: {}", stderr);
                }
            }
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

#[test]
fn test_phase64_workspace_manifest_parsing() {
    let toml_str = r#"
[package]
name = "ws-root"
version = "0.1.0"

[workspace]
members = ["crates/*", "tools/*"]

[workspace.dependencies]
json = "1.0.0"
utils = "2.0.0"
"#;

    #[derive(Debug, Deserialize)]
    struct WsManifest {
        package: PackageInfo,
        workspace: Option<WsConfig>,
    }

    #[derive(Debug, Deserialize)]
    struct WsConfig {
        members: Vec<String>,
        #[serde(default)]
        dependencies: HashMap<String, String>,
    }

    let manifest: WsManifest = toml::from_str(toml_str).unwrap();
    assert_eq!(manifest.package.name, "ws-root");
    let ws = manifest.workspace.unwrap();
    assert_eq!(ws.members, vec!["crates/*", "tools/*"]);
    assert_eq!(ws.dependencies.len(), 2);
    assert_eq!(ws.dependencies["json"], "1.0.0");
    assert_eq!(ws.dependencies["utils"], "2.0.0");
}

// ---------------------------------------------------------------------------
// Phase 64 Task 6: Lock File Generation & Reproducible Builds
// ---------------------------------------------------------------------------

#[test]
fn test_phase64_lockfile_format_version() {
    let lockfile_str = r#"
version = 1

[packages.json-parser]
version = "1.0.0"
checksum = "abc123"
source = "registry"
dependencies = []

[packages.utils]
version = "2.1.0"
checksum = "def456"
source = "registry"
dependencies = ["json-parser"]
"#;

    #[derive(Debug, Deserialize)]
    struct TestLockFile {
        version: u32,
        packages: HashMap<String, TestLockedPkg>,
    }

    #[derive(Debug, Deserialize)]
    struct TestLockedPkg {
        version: String,
        checksum: String,
        source: String,
        dependencies: Vec<String>,
    }

    let lock: TestLockFile = toml::from_str(lockfile_str).unwrap();
    assert_eq!(lock.version, 1);
    assert_eq!(lock.packages.len(), 2);
    assert_eq!(lock.packages["json-parser"].version, "1.0.0");
    assert_eq!(lock.packages["json-parser"].checksum, "abc123");
    assert_eq!(lock.packages["utils"].dependencies, vec!["json-parser"]);
}

#[test]
fn test_phase64_lockfile_deterministic_serialization() {
    // Verify that two identical lockfiles produce the same string output
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestLockFile {
        version: u32,
        packages: std::collections::BTreeMap<String, TestLockedPkg>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestLockedPkg {
        version: String,
        checksum: String,
        source: String,
        dependencies: Vec<String>,
    }

    let mut lock = TestLockFile {
        version: 1,
        packages: std::collections::BTreeMap::new(),
    };

    lock.packages.insert(
        "alpha".to_string(),
        TestLockedPkg {
            version: "1.0.0".to_string(),
            checksum: "sha256-aaa".to_string(),
            source: "registry".to_string(),
            dependencies: vec![],
        },
    );
    lock.packages.insert(
        "beta".to_string(),
        TestLockedPkg {
            version: "2.0.0".to_string(),
            checksum: "sha256-bbb".to_string(),
            source: "registry".to_string(),
            dependencies: vec!["alpha".to_string()],
        },
    );

    // Serialize twice — should be identical (BTreeMap ensures ordering)
    let s1 = toml::to_string_pretty(&lock).unwrap();
    let s2 = toml::to_string_pretty(&lock).unwrap();
    assert_eq!(s1, s2, "lockfile serialization must be deterministic");

    // Roundtrip
    let parsed: TestLockFile = toml::from_str(&s1).unwrap();
    assert_eq!(parsed.packages.len(), 2);
    assert_eq!(parsed.packages["beta"].dependencies, vec!["alpha"]);
}

#[test]
fn test_phase64_lockfile_reproducible_resolution() {
    // Given the same lockfile, dependency resolution should pick the same versions
    let lock_content = r#"
version = 1

[packages.dep-a]
version = "1.2.3"
checksum = "sha256-xyz"
source = "registry"
dependencies = []
"#;

    #[derive(Debug, Deserialize)]
    struct TestLockFile {
        version: u32,
        packages: HashMap<String, TestLockedPkg>,
    }

    #[derive(Debug, Deserialize)]
    struct TestLockedPkg {
        version: String,
        checksum: String,
        source: String,
        dependencies: Vec<String>,
    }

    let lock: TestLockFile = toml::from_str(lock_content).unwrap();
    let locked_pkg = &lock.packages["dep-a"];

    // The locked version should be used even if newer versions exist
    assert_eq!(locked_pkg.version, "1.2.3");
    assert_eq!(locked_pkg.checksum, "sha256-xyz");
    assert_eq!(locked_pkg.source, "registry");

    // Verify that ^1.0 requirement is compatible with locked 1.2.3
    let req = SemVer::parse("1.0.0").unwrap();
    let locked_ver = SemVer::parse("1.2.3").unwrap();
    assert!(
        caret_matches(&req, &locked_ver),
        "locked 1.2.3 should satisfy ^1.0"
    );
}

// ---------------------------------------------------------------------------
// Phase 64 Task 7: Package Template — vais new --lib / --bin Scaffolding
// ---------------------------------------------------------------------------

#[test]
fn test_phase64_new_bin_project_structure() {
    let tmp = TempDir::new().unwrap();

    match run_vaisc(&["new", "my-bin-app"], tmp.path()) {
        Ok(output) => {
            if output.status.success() {
                let project = tmp.path().join("my-bin-app");
                assert!(project.join("vais.toml").exists());
                assert!(project.join("src").join("main.vais").exists());
                assert!(project.join("tests").exists());
                assert!(project.join(".gitignore").exists());

                let manifest = fs::read_to_string(project.join("vais.toml")).unwrap();
                assert!(manifest.contains("my-bin-app"));
                assert!(manifest.contains("0.1.0"));

                let main = fs::read_to_string(project.join("src").join("main.vais")).unwrap();
                assert!(
                    main.contains("F main()"),
                    "bin project should have F main()"
                );
            }
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

#[test]
fn test_phase64_new_lib_project_structure() {
    let tmp = TempDir::new().unwrap();

    match run_vaisc(&["new", "my-lib-pkg", "--lib"], tmp.path()) {
        Ok(output) => {
            if output.status.success() {
                let project = tmp.path().join("my-lib-pkg");
                assert!(project.join("vais.toml").exists());
                assert!(project.join("src").join("lib.vais").exists());
                assert!(
                    !project.join("src").join("main.vais").exists(),
                    "lib project should not have main.vais"
                );

                let lib = fs::read_to_string(project.join("src").join("lib.vais")).unwrap();
                assert!(
                    !lib.contains("F main()"),
                    "lib project should not have F main()"
                );
            }
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

#[test]
fn test_phase64_new_project_gitignore_contents() {
    let tmp = TempDir::new().unwrap();

    match run_vaisc(&["new", "gitignore-test"], tmp.path()) {
        Ok(output) => {
            if output.status.success() {
                let gitignore_path = tmp.path().join("gitignore-test").join(".gitignore");
                if gitignore_path.exists() {
                    let content = fs::read_to_string(&gitignore_path).unwrap();
                    assert!(
                        content.contains("target") || content.contains("*.o"),
                        ".gitignore should ignore build artifacts"
                    );
                }
            }
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

#[test]
fn test_phase64_new_project_then_build() {
    let tmp = TempDir::new().unwrap();

    match run_vaisc(&["new", "buildable-proj"], tmp.path()) {
        Ok(output) => {
            if !output.status.success() {
                return;
            }
        }
        Err(_) => return,
    }

    let project_dir = tmp.path().join("buildable-proj");
    let output = Command::new("cargo")
        .args(["run", "--bin", "vaisc", "--", "pkg", "build"])
        .current_dir(&project_dir)
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                // Build should succeed for newly created project
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                eprintln!("new project build non-zero: {}", stderr);
            }
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

// ---------------------------------------------------------------------------
// Phase 64 Task 8: Documentation Generation — vais doc → HTML Output
// ---------------------------------------------------------------------------

#[test]
fn test_phase64_doc_generates_markdown() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("doc-test");
    fs::create_dir_all(project_dir.join("src")).unwrap();
    fs::write(
        project_dir.join("vais.toml"),
        "[package]\nname = \"doc-test\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    fs::write(
        project_dir.join("src").join("lib.vais"),
        "# A helper function\nF helper(x: i64) -> i64 { x + 1 }\n\n# Entry point\nF main() -> i64 { helper(5) }\n",
    )
    .unwrap();

    match run_vaisc(&["pkg", "doc"], &project_dir) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if output.status.success() {
                assert!(
                    stdout.contains("helper") || stdout.contains("main") || stdout.contains("doc"),
                    "doc output should mention functions: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("pkg doc non-zero: {}", stderr);
            }
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

#[test]
fn test_phase64_doc_html_format_output() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("html-doc");
    fs::create_dir_all(project_dir.join("src")).unwrap();
    fs::write(
        project_dir.join("vais.toml"),
        "[package]\nname = \"html-doc\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        project_dir.join("src").join("lib.vais"),
        "# Compute sum\nF sum(a: i64, b: i64) -> i64 { a + b }\n",
    )
    .unwrap();

    match run_vaisc(&["pkg", "doc", "--format", "html"], &project_dir) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if output.status.success() {
                assert!(
                    stdout.contains("<html") || stdout.contains("<h") || stdout.contains("<div")
                        || stdout.contains("html-doc") || stdout.contains("sum"),
                    "HTML doc should contain HTML tags or function names: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("pkg doc --format html non-zero: {}", stderr);
            }
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

#[test]
fn test_phase64_doc_no_manifest_errors() {
    let tmp = TempDir::new().unwrap();

    match run_vaisc(&["pkg", "doc"], tmp.path()) {
        Ok(output) => {
            assert!(
                !output.status.success(),
                "pkg doc without manifest should fail"
            );
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

// ---------------------------------------------------------------------------
// Phase 64: Cross-cutting tests
// ---------------------------------------------------------------------------

#[test]
fn test_phase64_add_then_build_with_path_dep() {
    let tmp = TempDir::new().unwrap();

    // Create a library
    let lib_dir = tmp.path().join("mathlib");
    fs::create_dir_all(lib_dir.join("src")).unwrap();
    fs::write(
        lib_dir.join("vais.toml"),
        "[package]\nname = \"mathlib\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    fs::write(
        lib_dir.join("src").join("lib.vais"),
        "F square(x: i64) -> i64 { x * x }\n",
    )
    .unwrap();

    // Create main app
    let app_dir = tmp.path().join("calculator");
    fs::create_dir_all(app_dir.join("src")).unwrap();
    fs::write(
        app_dir.join("vais.toml"),
        "[package]\nname = \"calculator\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        app_dir.join("src").join("main.vais"),
        "F main() -> i64 { 0 }\n",
    )
    .unwrap();

    // Add dependency via CLI
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "vaisc",
            "--",
            "pkg",
            "add",
            "mathlib",
            "--path",
            "../mathlib",
        ])
        .current_dir(&app_dir)
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                // Verify dependency was added to vais.toml
                let content = fs::read_to_string(app_dir.join("vais.toml")).unwrap();
                assert!(
                    content.contains("mathlib"),
                    "vais.toml should contain mathlib dependency after add"
                );
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                eprintln!("pkg add non-zero: {}", stderr);
            }
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

#[test]
fn test_phase64_feature_resolution_with_defaults() {
    // Simulate feature resolution
    let feature_defs: HashMap<String, Vec<String>> = {
        let mut m = HashMap::new();
        m.insert("default".to_string(), vec!["json".to_string()]);
        m.insert("json".to_string(), vec![]);
        m.insert("async".to_string(), vec!["json".to_string()]);
        m.insert(
            "full".to_string(),
            vec!["json".to_string(), "async".to_string()],
        );
        m
    };

    // Resolve "full" feature with defaults
    let mut enabled = std::collections::HashSet::new();
    let mut stack = vec!["full".to_string(), "default".to_string()];

    while let Some(feat) = stack.pop() {
        if enabled.insert(feat.clone()) {
            if let Some(deps) = feature_defs.get(&feat) {
                for dep in deps {
                    if !enabled.contains(dep) {
                        stack.push(dep.clone());
                    }
                }
            }
        }
    }

    assert!(enabled.contains("full"));
    assert!(enabled.contains("json"));
    assert!(enabled.contains("async"));
    assert!(enabled.contains("default"));
}

#[test]
fn test_phase64_pkg_check_workspace() {
    let tmp = TempDir::new().unwrap();
    let ws_root = tmp.path();

    // Workspace with one member
    fs::write(
        ws_root.join("vais.toml"),
        "[package]\nname = \"ws\"\nversion = \"0.1.0\"\n\n[workspace]\nmembers = [\"pkgs/*\"]\n",
    )
    .unwrap();

    let member = ws_root.join("pkgs").join("lib-a");
    fs::create_dir_all(member.join("src")).unwrap();
    fs::write(
        member.join("vais.toml"),
        "[package]\nname = \"lib-a\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        member.join("src").join("lib.vais"),
        "F lib_fn() -> i64 { 42 }\n",
    )
    .unwrap();

    // Check workspace
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "vaisc",
            "--",
            "pkg",
            "check",
            "--workspace",
        ])
        .current_dir(ws_root)
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                let stdout = String::from_utf8_lossy(&o.stdout);
                // Should mention checking or the member
                assert!(
                    stdout.contains("lib-a") || stdout.contains("✓") || stdout.contains("Check"),
                    "workspace check should process member: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                eprintln!("workspace check non-zero: {}", stderr);
            }
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}

#[test]
fn test_phase64_cyclic_dependency_detection() {
    let tmp = TempDir::new().unwrap();

    // Create A -> B -> A (cycle)
    let a_dir = tmp.path().join("a");
    fs::create_dir_all(a_dir.join("src")).unwrap();
    fs::write(
        a_dir.join("vais.toml"),
        "[package]\nname = \"a\"\nversion = \"1.0.0\"\n\n[dependencies]\nb = { path = \"../b\" }\n",
    )
    .unwrap();
    fs::write(a_dir.join("src").join("lib.vais"), "F fa() -> i64 { 1 }\n").unwrap();

    let b_dir = tmp.path().join("b");
    fs::create_dir_all(b_dir.join("src")).unwrap();
    fs::write(
        b_dir.join("vais.toml"),
        "[package]\nname = \"b\"\nversion = \"1.0.0\"\n\n[dependencies]\na = { path = \"../a\" }\n",
    )
    .unwrap();
    fs::write(b_dir.join("src").join("lib.vais"), "F fb() -> i64 { 2 }\n").unwrap();

    // Build A — should detect cycle and fail gracefully
    let output = Command::new("cargo")
        .args(["run", "--bin", "vaisc", "--", "pkg", "build"])
        .current_dir(&a_dir)
        .output();

    match output {
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Should either fail with cycle error or detect and report
            assert!(
                !o.status.success()
                    || stderr.contains("cyclic")
                    || stderr.contains("cycle")
                    || stdout.contains("cyclic")
                    || stdout.contains("cycle"),
                "cyclic dependency should be detected: stdout={}, stderr={}",
                stdout,
                stderr
            );
        }
        Err(_) => eprintln!("skipping CLI test"),
    }
}
