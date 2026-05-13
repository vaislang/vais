//! Tests for package management

#![cfg(test)]

use super::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_init_package() {
    let dir = tempdir().unwrap();
    init_package(dir.path(), Some("test-pkg")).unwrap();

    let manifest = load_manifest(dir.path()).unwrap();
    assert_eq!(manifest.package.name, "test-pkg");
    assert_eq!(manifest.package.version, "0.1.0");

    assert!(dir.path().join("src/main.vais").exists());
}

#[test]
fn test_load_manifest() {
    let dir = tempdir().unwrap();
    let toml_content = r#"
[package]
name = "my-pkg"
version = "1.0.0"
description = "A test package"

[dependencies]
other-lib = { path = "../other" }

[build]
opt_level = 2
"#;
    fs::write(dir.path().join("vais.toml"), toml_content).unwrap();

    let manifest = load_manifest(dir.path()).unwrap();
    assert_eq!(manifest.package.name, "my-pkg");
    assert_eq!(manifest.package.version, "1.0.0");
    assert_eq!(manifest.dependencies.len(), 1);
    assert_eq!(manifest.build.opt_level, Some(2));
}

#[test]
fn test_add_remove_dependency() {
    let dir = tempdir().unwrap();
    init_package(dir.path(), Some("test-pkg")).unwrap();

    let manifest_path = dir.path().join("vais.toml");

    // Add dependency
    add_dependency(&manifest_path, "my-lib", Some("../my-lib"), None).unwrap();

    let manifest = load_manifest(dir.path()).unwrap();
    assert!(manifest.dependencies.contains_key("my-lib"));

    // Remove dependency
    remove_dependency(&manifest_path, "my-lib").unwrap();

    let manifest = load_manifest(dir.path()).unwrap();
    assert!(!manifest.dependencies.contains_key("my-lib"));
}

#[test]
fn test_find_manifest() {
    let dir = tempdir().unwrap();
    init_package(dir.path(), Some("test-pkg")).unwrap();

    // Create nested directory
    let nested = dir.path().join("src/nested");
    fs::create_dir_all(&nested).unwrap();

    // Should find manifest from nested dir
    let found = find_manifest(&nested);
    assert!(found.is_some());
    assert_eq!(found.unwrap(), dir.path());
}

#[test]
fn test_resolve_all_dependencies_with_path_dep() {
    let root = tempdir().unwrap();

    // Create main package
    let main_dir = root.path().join("main-pkg");
    fs::create_dir_all(&main_dir).unwrap();
    init_package(&main_dir, Some("main-pkg")).unwrap();

    // Create dependency package
    let dep_dir = root.path().join("my-dep");
    fs::create_dir_all(&dep_dir).unwrap();
    init_package(&dep_dir, Some("my-dep")).unwrap();

    // Add path dependency to main package
    let manifest_path = main_dir.join("vais.toml");
    add_dependency(&manifest_path, "my-dep", Some("../my-dep"), None).unwrap();

    let manifest = load_manifest(&main_dir).unwrap();
    let deps = resolve_all_dependencies(&manifest, &main_dir, None).unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "my-dep");
}

#[test]
fn test_resolve_registry_dep_from_cache() {
    let root = tempdir().unwrap();

    // Create a fake registry cache with an extracted package
    let cache_root = root.path().join("registry");
    let extracted = cache_root
        .join("cache")
        .join("json-parser")
        .join("1.0.0")
        .join("extracted");
    fs::create_dir_all(&extracted).unwrap();

    // Create a minimal vais.toml in the extracted package
    let dep_manifest = r#"
[package]
name = "json-parser"
version = "1.0.0"
"#;
    fs::write(extracted.join("vais.toml"), dep_manifest).unwrap();
    fs::create_dir_all(extracted.join("src")).unwrap();
    fs::write(extracted.join("src/lib.vais"), "# json-parser lib\n").unwrap();

    // Create main package with a registry dependency
    let main_dir = root.path().join("main-pkg");
    fs::create_dir_all(&main_dir).unwrap();
    init_package(&main_dir, Some("main-pkg")).unwrap();

    // Write manifest with registry dep
    let toml_content = r#"
[package]
name = "main-pkg"
version = "0.1.0"

[dependencies]
json-parser = "1.0.0"
"#;
    fs::write(main_dir.join("vais.toml"), toml_content).unwrap();

    let manifest = load_manifest(&main_dir).unwrap();
    let deps = resolve_all_dependencies(&manifest, &main_dir, Some(&cache_root)).unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "json-parser");
    assert!(deps[0].path.to_string_lossy().contains("extracted"));
}

#[test]
fn test_resolve_registry_dep_not_installed() {
    let root = tempdir().unwrap();

    // Empty cache
    let cache_root = root.path().join("registry");
    fs::create_dir_all(&cache_root).unwrap();

    // Create main package with a registry dependency
    let main_dir = root.path().join("main-pkg");
    fs::create_dir_all(&main_dir).unwrap();

    let toml_content = r#"
[package]
name = "main-pkg"
version = "0.1.0"

[dependencies]
nonexistent-pkg = "2.0.0"
"#;
    fs::write(main_dir.join("vais.toml"), toml_content).unwrap();
    fs::create_dir_all(main_dir.join("src")).unwrap();
    fs::write(main_dir.join("src/main.vais"), "").unwrap();

    let manifest = load_manifest(&main_dir).unwrap();
    let result = resolve_all_dependencies(&manifest, &main_dir, Some(&cache_root));
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not installed"));
    assert!(err.contains("nonexistent-pkg"));
}

#[test]
fn test_resolve_mixed_path_and_registry_deps() {
    let root = tempdir().unwrap();

    // Create path dependency
    let path_dep_dir = root.path().join("local-lib");
    fs::create_dir_all(&path_dep_dir).unwrap();
    init_package(&path_dep_dir, Some("local-lib")).unwrap();

    // Create registry cache with another package
    let cache_root = root.path().join("registry");
    let extracted = cache_root
        .join("cache")
        .join("remote-lib")
        .join("0.5.0")
        .join("extracted");
    fs::create_dir_all(&extracted).unwrap();
    let dep_manifest = r#"
[package]
name = "remote-lib"
version = "0.5.0"
"#;
    fs::write(extracted.join("vais.toml"), dep_manifest).unwrap();
    fs::create_dir_all(extracted.join("src")).unwrap();
    fs::write(extracted.join("src/lib.vais"), "").unwrap();

    // Create main package with both types of deps
    let main_dir = root.path().join("main-pkg");
    fs::create_dir_all(&main_dir).unwrap();

    let toml_content = r#"
[package]
name = "main-pkg"
version = "0.1.0"

[dependencies]
local-lib = { path = "../local-lib" }
remote-lib = "0.5.0"
"#;
    fs::write(main_dir.join("vais.toml"), toml_content).unwrap();
    fs::create_dir_all(main_dir.join("src")).unwrap();
    fs::write(main_dir.join("src/main.vais"), "").unwrap();

    let manifest = load_manifest(&main_dir).unwrap();
    let deps = resolve_all_dependencies(&manifest, &main_dir, Some(&cache_root)).unwrap();
    assert_eq!(deps.len(), 2);

    let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"local-lib"));
    assert!(names.contains(&"remote-lib"));
}

#[test]
fn test_find_cached_registry_dep_with_version_prefix() {
    let root = tempdir().unwrap();

    // Create cache with version 1.2.3
    let cache_root = root.path().join("registry");
    let extracted = cache_root
        .join("cache")
        .join("my-pkg")
        .join("1.2.3")
        .join("extracted");
    fs::create_dir_all(&extracted).unwrap();

    // Should find with exact version
    assert!(find_cached_registry_dep(&cache_root, "my-pkg", "1.2.3").is_some());

    // Should find with ^ prefix (stripped)
    assert!(find_cached_registry_dep(&cache_root, "my-pkg", "^1.2.3").is_some());

    // Should find with ~ prefix (stripped)
    assert!(find_cached_registry_dep(&cache_root, "my-pkg", "~1.2.3").is_some());

    // Should not find nonexistent package
    assert!(find_cached_registry_dep(&cache_root, "no-such-pkg", "1.0.0").is_none());
}

#[test]
fn test_native_dependencies_simple() {
    let dir = tempdir().unwrap();
    let toml_content = r#"
[package]
name = "my-pkg"
version = "1.0.0"

[native-dependencies]
openssl = "ssl"
zlib = "z"
"#;
    fs::write(dir.path().join("vais.toml"), toml_content).unwrap();

    let manifest = load_manifest(dir.path()).unwrap();
    assert_eq!(manifest.native_dependencies.len(), 2);

    let ssl = &manifest.native_dependencies["openssl"];
    assert_eq!(ssl.lib_flags(), vec!["-lssl"]);
    assert!(ssl.include_flag().is_none());

    let z = &manifest.native_dependencies["zlib"];
    assert_eq!(z.lib_flags(), vec!["-lz"]);
}

#[test]
fn test_native_dependencies_detailed() {
    let dir = tempdir().unwrap();
    let toml_content = r#"
[package]
name = "my-pkg"
version = "1.0.0"

[native-dependencies.openssl]
libs = ["ssl", "crypto"]
include = "/usr/include/openssl"
lib_path = "/usr/lib"
system = true

[native-dependencies.custom]
libs = ["mylib"]
sources = ["vendor/mylib.c"]
"#;
    fs::write(dir.path().join("vais.toml"), toml_content).unwrap();

    let manifest = load_manifest(dir.path()).unwrap();
    assert_eq!(manifest.native_dependencies.len(), 2);

    let ssl = &manifest.native_dependencies["openssl"];
    assert_eq!(ssl.lib_flags(), vec!["-lssl", "-lcrypto"]);
    assert_eq!(
        ssl.include_flag(),
        Some("-I/usr/include/openssl".to_string())
    );
    assert_eq!(ssl.lib_path_flag(), Some("-L/usr/lib".to_string()));

    let custom = &manifest.native_dependencies["custom"];
    assert_eq!(custom.lib_flags(), vec!["-lmylib"]);
    assert_eq!(custom.source_files(), &["vendor/mylib.c"]);
}

#[test]
fn test_registry_dep_without_cache_root_skipped() {
    let root = tempdir().unwrap();

    let main_dir = root.path().join("main-pkg");
    fs::create_dir_all(&main_dir).unwrap();

    let toml_content = r#"
[package]
name = "main-pkg"
version = "0.1.0"

[dependencies]
some-registry-pkg = "1.0.0"
"#;
    fs::write(main_dir.join("vais.toml"), toml_content).unwrap();
    fs::create_dir_all(main_dir.join("src")).unwrap();
    fs::write(main_dir.join("src/main.vais"), "").unwrap();

    let manifest = load_manifest(&main_dir).unwrap();
    // Without cache_root, registry deps should be skipped (not cause an error)
    let deps = resolve_all_dependencies(&manifest, &main_dir, None).unwrap();
    assert_eq!(deps.len(), 0);
}

#[test]
fn test_workspace_manifest_parsing() {
    let dir = tempdir().unwrap();
    let toml_content = r#"
[workspace]
members = ["crates/*"]

[workspace.dependencies]
json = "1.0.0"

[package]
name = "workspace-root"
version = "0.1.0"
"#;
    fs::write(dir.path().join("vais.toml"), toml_content).unwrap();

    let manifest = load_manifest(dir.path()).unwrap();
    assert!(manifest.workspace.is_some());

    let ws = manifest.workspace.as_ref().unwrap();
    assert_eq!(ws.members, vec!["crates/*"]);
    assert!(ws.dependencies.contains_key("json"));
}

#[test]
fn test_find_workspace_root() {
    let root = tempdir().unwrap();

    // Create workspace root
    let ws_toml = r#"
[workspace]
members = ["crates/*"]

[package]
name = "ws-root"
version = "0.1.0"
"#;
    fs::write(root.path().join("vais.toml"), ws_toml).unwrap();

    // Create a member directory
    let member_dir = root.path().join("crates").join("my-lib");
    fs::create_dir_all(&member_dir).unwrap();
    let member_toml = r#"
[package]
name = "my-lib"
version = "0.1.0"
"#;
    fs::write(member_dir.join("vais.toml"), member_toml).unwrap();

    // Should find workspace root from member directory
    let found = find_workspace_root(&member_dir);
    assert!(found.is_some());
    assert_eq!(found.unwrap(), root.path().to_path_buf());
}

#[test]
fn test_resolve_workspace_members() {
    let root = tempdir().unwrap();

    // Create workspace root
    let ws_toml = r#"
[workspace]
members = ["crates/*"]
"#;
    fs::write(root.path().join("vais.toml"), ws_toml).unwrap();

    // Create two member packages
    for name in &["lib-a", "lib-b"] {
        let dir = root.path().join("crates").join(name);
        fs::create_dir_all(dir.join("src")).unwrap();
        let toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
"#,
            name
        );
        fs::write(dir.join("vais.toml"), toml).unwrap();
        fs::write(
            dir.join("src/lib.vais"),
            format!("# {}\nF greet() -> i64 {{ 0 }}\n", name),
        )
        .unwrap();
    }

    let workspace = resolve_workspace_members(root.path()).unwrap();
    assert_eq!(workspace.members.len(), 2);

    let names: Vec<&str> = workspace
        .members
        .iter()
        .map(|m| m.manifest.package.name.as_str())
        .collect();
    assert!(names.contains(&"lib-a"));
    assert!(names.contains(&"lib-b"));
}

#[test]
fn test_workspace_dependency_resolution() {
    let root = tempdir().unwrap();

    // Create workspace root with shared dependency version
    let ws_toml = r#"
[workspace]
members = ["crates/*"]

[workspace.dependencies]
json = "2.0.0"
"#;
    fs::write(root.path().join("vais.toml"), ws_toml).unwrap();

    // Create a member that uses workspace = true
    let dir = root.path().join("crates").join("my-app");
    fs::create_dir_all(dir.join("src")).unwrap();
    let member_toml = r#"
[package]
name = "my-app"
version = "0.1.0"

[dependencies]
json = { workspace = true }
"#;
    fs::write(dir.join("vais.toml"), member_toml).unwrap();
    fs::write(dir.join("src/main.vais"), "F main() -> i64 { 0 }\n").unwrap();

    let workspace = resolve_workspace_members(root.path()).unwrap();
    assert_eq!(workspace.members.len(), 1);

    // The dependency should be resolved from workspace
    let member = &workspace.members[0];
    let json_dep = member.manifest.dependencies.get("json").unwrap();
    match json_dep {
        Dependency::Version(v) => assert_eq!(v, "2.0.0"),
        Dependency::Detailed(d) => {
            panic!("expected Version dependency, got Detailed: {:?}", d)
        }
    }
}

#[test]
fn test_inter_workspace_path_deps() {
    let root = tempdir().unwrap();

    // Create workspace root
    let ws_toml = r#"
[workspace]
members = ["crates/*"]
"#;
    fs::write(root.path().join("vais.toml"), ws_toml).unwrap();

    // Create lib-core
    let core_dir = root.path().join("crates").join("lib-core");
    fs::create_dir_all(core_dir.join("src")).unwrap();
    fs::write(
        core_dir.join("vais.toml"),
        r#"[package]
name = "lib-core"
version = "0.1.0"
"#,
    )
    .unwrap();
    fs::write(core_dir.join("src/lib.vais"), "F core_fn() -> i64 { 42 }\n").unwrap();

    // Create my-app that depends on lib-core (by name, no path)
    let app_dir = root.path().join("crates").join("my-app");
    fs::create_dir_all(app_dir.join("src")).unwrap();
    fs::write(
        app_dir.join("vais.toml"),
        r#"[package]
name = "my-app"
version = "0.1.0"

[dependencies]
lib-core = "0.1.0"
"#,
    )
    .unwrap();
    fs::write(app_dir.join("src/main.vais"), "F main() -> i64 { 0 }\n").unwrap();

    let mut workspace = resolve_workspace_members(root.path()).unwrap();
    resolve_inter_workspace_deps(&mut workspace);

    // Find my-app member
    let app_member = workspace
        .members
        .iter()
        .find(|m| m.manifest.package.name == "my-app")
        .unwrap();

    // lib-core dependency should now have a path
    let dep = app_member.manifest.dependencies.get("lib-core").unwrap();
    match dep {
        Dependency::Detailed(d) => {
            assert!(
                d.path.is_some(),
                "expected path to be set for workspace member dep"
            );
            let path = d.path.as_ref().unwrap();
            assert!(
                path.contains("lib-core"),
                "path should reference lib-core: {}",
                path
            );
        }
        Dependency::Version(v) => {
            panic!("expected Detailed dependency with path, got Version: {}", v)
        }
    }
}

#[test]
fn test_feature_config_parsing() {
    let dir = tempdir().unwrap();
    let toml_content = r#"
[package]
name = "my-pkg"
version = "1.0.0"

[features]
default = ["json"]
json = []
async = ["json"]
full = ["json", "async"]
"#;
    fs::write(dir.path().join("vais.toml"), toml_content).unwrap();

    let manifest = load_manifest(dir.path()).unwrap();
    assert!(manifest.features.is_some());

    let fc = manifest.features.as_ref().unwrap();
    assert_eq!(fc.default, vec!["json"]);
    assert!(fc.features.contains_key("json"));
    assert!(fc.features.contains_key("async"));
    assert!(fc.features.contains_key("full"));
}

#[test]
fn test_feature_resolve_defaults() {
    use std::collections::HashMap;

    let fc = FeatureConfig {
        default: vec!["json".to_string()],
        features: {
            let mut m = HashMap::new();
            m.insert("json".to_string(), vec![]);
            m.insert("async".to_string(), vec!["json".to_string()]);
            m
        },
    };

    // With defaults
    let resolved = fc.resolve_features(&[], true);
    assert!(resolved.contains(&"json".to_string()));
    assert!(!resolved.contains(&"async".to_string()));

    // Without defaults
    let resolved = fc.resolve_features(&[], false);
    assert!(resolved.is_empty());
}

#[test]
fn test_feature_resolve_transitive() {
    use std::collections::HashMap;

    let fc = FeatureConfig {
        default: vec![],
        features: {
            let mut m = HashMap::new();
            m.insert("json".to_string(), vec![]);
            m.insert("async".to_string(), vec!["json".to_string()]);
            m.insert(
                "full".to_string(),
                vec!["json".to_string(), "async".to_string()],
            );
            m
        },
    };

    // Selecting "async" should also enable "json"
    let resolved = fc.resolve_features(&["async".to_string()], false);
    assert!(resolved.contains(&"async".to_string()));
    assert!(resolved.contains(&"json".to_string()));

    // Selecting "full" should enable all
    let resolved = fc.resolve_features(&["full".to_string()], false);
    assert_eq!(resolved.len(), 3);
}

#[test]
fn test_feature_all_features() {
    use std::collections::HashMap;

    let fc = FeatureConfig {
        default: vec!["json".to_string()],
        features: {
            let mut m = HashMap::new();
            m.insert("json".to_string(), vec![]);
            m.insert("async".to_string(), vec![]);
            m.insert("full".to_string(), vec![]);
            m
        },
    };

    let all = fc.all_features();
    assert_eq!(all.len(), 3);
    assert!(all.contains(&"async".to_string()));
    assert!(all.contains(&"full".to_string()));
    assert!(all.contains(&"json".to_string()));
}
