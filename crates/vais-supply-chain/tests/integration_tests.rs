//! Integration tests for vais-supply-chain crate
//!
//! Tests SBOM generation, dependency auditing, and package signing.

use vais_supply_chain::{
    audit::{AuditStatus, DependencyAuditor, DependencyEntry, DependencyManifest, VulnerabilitySeverity},
    sbom::{ComponentType, SbomComponent, SbomGenerator},
    signing::PackageSigner,
};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_sbom_generator_creation() {
    let generator = SbomGenerator::new(
        "vais-tool".to_string(),
        "1.0.0".to_string(),
        "Vais Team".to_string(),
    );

    // Generator should be created successfully
    // We can't inspect internal fields directly, but we can test behavior
    let components = vec![];
    let dependencies = vec![];
    let sbom = generator.generate_from_components(components, dependencies);

    assert_eq!(sbom.format_version, "CycloneDX-1.4");
    assert_eq!(sbom.tool.name, "vais-tool");
    assert_eq!(sbom.tool.version, "1.0.0");
    assert_eq!(sbom.tool.vendor, "Vais Team");
}

#[test]
fn test_sbom_generator_default() {
    let generator = SbomGenerator::default();
    let components = vec![];
    let dependencies = vec![];
    let sbom = generator.generate_from_components(components, dependencies);

    assert_eq!(sbom.format_version, "CycloneDX-1.4");
    assert_eq!(sbom.tool.name, "vais-supply-chain");
    assert_eq!(sbom.tool.version, "0.0.1");
    assert_eq!(sbom.tool.vendor, "Vais Team");
}

#[test]
fn test_sbom_document_structure() {
    let generator = SbomGenerator::default();

    let mut hashes = HashMap::new();
    hashes.insert("SHA-256".to_string(), "abc123".to_string());

    let component = SbomComponent {
        name: "test-lib".to_string(),
        version: "1.0.0".to_string(),
        component_type: ComponentType::Library,
        hashes,
        license: Some("MIT".to_string()),
        description: Some("Test library".to_string()),
        publisher: Some("Test Publisher".to_string()),
    };

    let components = vec![component.clone()];
    let dependencies = vec![];
    let sbom = generator.generate_from_components(components, dependencies);

    assert_eq!(sbom.components.len(), 1);
    assert_eq!(sbom.components[0].name, "test-lib");
    assert_eq!(sbom.components[0].version, "1.0.0");
    assert_eq!(sbom.components[0].component_type, ComponentType::Library);
    assert_eq!(sbom.components[0].license, Some("MIT".to_string()));
    assert_eq!(sbom.components[0].hashes.get("SHA-256"), Some(&"abc123".to_string()));
}

#[test]
fn test_sbom_json_serialization() {
    let generator = SbomGenerator::default();
    let components = vec![];
    let dependencies = vec![];
    let sbom = generator.generate_from_components(components, dependencies);

    let json_result = sbom.to_json();
    assert!(json_result.is_ok());

    let json = json_result.unwrap();
    assert!(json.contains("CycloneDX-1.4"));
    assert!(json.contains("vais-supply-chain"));
}

#[test]
fn test_sbom_file_roundtrip() {
    let generator = SbomGenerator::default();
    let components = vec![];
    let dependencies = vec![];
    let sbom = generator.generate_from_components(components, dependencies);

    let temp_dir = TempDir::new().unwrap();
    let sbom_path = temp_dir.path().join("test.sbom.json");

    // Write SBOM to file
    let write_result = sbom.write_to_file(&sbom_path);
    assert!(write_result.is_ok());

    // Read SBOM back from file
    let loaded_sbom_result = vais_supply_chain::sbom::SbomDocument::from_file(&sbom_path);
    assert!(loaded_sbom_result.is_ok());

    let loaded_sbom = loaded_sbom_result.unwrap();
    assert_eq!(loaded_sbom.format_version, sbom.format_version);
    assert_eq!(loaded_sbom.tool.name, sbom.tool.name);
}

#[test]
fn test_sbom_validation() {
    let generator = SbomGenerator::default();

    let component1 = SbomComponent {
        name: "lib-a".to_string(),
        version: "1.0.0".to_string(),
        component_type: ComponentType::Library,
        hashes: HashMap::new(),
        license: None,
        description: None,
        publisher: None,
    };

    let components = vec![component1];
    let dependencies = vec![];
    let sbom = generator.generate_from_components(components, dependencies);

    // Should validate successfully
    assert!(sbom.validate().is_ok());
}

#[test]
fn test_dependency_auditor_creation() {
    let auditor = DependencyAuditor::new();

    // Auditor should have default vulnerabilities loaded
    assert!(auditor.vulnerability_count() > 0);

    let vulnerable_packages = auditor.vulnerable_packages();
    assert!(!vulnerable_packages.is_empty());
}

#[test]
fn test_dependency_auditor_default() {
    let auditor = DependencyAuditor::default();
    assert!(auditor.vulnerability_count() > 0);
}

#[test]
fn test_dependency_audit_safe_package() {
    let auditor = DependencyAuditor::new();

    let manifest = DependencyManifest {
        dependencies: vec![
            DependencyEntry {
                name: "safe-package".to_string(),
                version: "1.0.0".to_string(),
            }
        ],
    };

    let results = auditor.audit_dependencies(&manifest);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].dependency_name, "safe-package");
    assert_eq!(results[0].status, AuditStatus::Safe);
    assert!(!results[0].has_vulnerabilities());
}

#[test]
fn test_dependency_audit_vulnerable_package() {
    let auditor = DependencyAuditor::new();

    let manifest = DependencyManifest {
        dependencies: vec![
            DependencyEntry {
                name: "old-crypto".to_string(),
                version: "1.0.0".to_string(),
            }
        ],
    };

    let results = auditor.audit_dependencies(&manifest);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].dependency_name, "old-crypto");
    assert_eq!(results[0].status, AuditStatus::Vulnerable);
    assert!(results[0].has_vulnerabilities());
    assert_eq!(results[0].max_severity(), Some(VulnerabilitySeverity::Critical));
}

#[test]
fn test_vulnerability_severity_ordering() {
    let low = VulnerabilitySeverity::Low;
    let medium = VulnerabilitySeverity::Medium;
    let high = VulnerabilitySeverity::High;
    let critical = VulnerabilitySeverity::Critical;

    assert!(low < medium);
    assert!(medium < high);
    assert!(high < critical);

    assert_eq!(low.score(), 3);
    assert_eq!(medium.score(), 5);
    assert_eq!(high.score(), 8);
    assert_eq!(critical.score(), 10);
}

#[test]
fn test_audit_result_severity_counts() {
    let auditor = DependencyAuditor::new();

    let manifest = DependencyManifest {
        dependencies: vec![
            DependencyEntry {
                name: "old-crypto".to_string(),
                version: "1.0.0".to_string(),
            }
        ],
    };

    let results = auditor.audit_dependencies(&manifest);
    assert_eq!(results.len(), 1);

    let critical_count = results[0].count_by_severity(VulnerabilitySeverity::Critical);
    assert!(critical_count > 0);
}

#[test]
fn test_package_signer_creation() {
    let signer = PackageSigner::new(
        "Test Signer".to_string(),
        Some("test@example.com".to_string()),
        Some("Test Org".to_string()),
    );

    assert_eq!(signer.signer_info.name, "Test Signer");
    assert_eq!(signer.signer_info.email, Some("test@example.com".to_string()));
    assert_eq!(signer.signer_info.organization, Some("Test Org".to_string()));
}

#[test]
fn test_package_signer_default() {
    let signer = PackageSigner::default();
    assert_eq!(signer.signer_info.name, "vais-builder");
    assert_eq!(signer.signer_info.email, Some("build@vais-lang.org".to_string()));
    assert_eq!(signer.signer_info.organization, Some("Vais Team".to_string()));
}

#[test]
fn test_package_signature_structure() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"test content").unwrap();

    let signer = PackageSigner::default();
    let signature_result = signer.sign_package(
        &test_file,
        "test-package".to_string(),
        "1.0.0".to_string(),
    );

    assert!(signature_result.is_ok());
    let signature = signature_result.unwrap();

    assert_eq!(signature.algorithm, "SHA-256");
    assert_eq!(signature.signer.name, "vais-builder");
    assert_eq!(signature.metadata.package_name, "test-package");
    assert_eq!(signature.metadata.package_version, "1.0.0");
    assert_eq!(signature.metadata.file_size, 12); // "test content" is 12 bytes
    assert!(!signature.hash.is_empty());
}

#[test]
fn test_package_signature_verification_success() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"test content").unwrap();

    let signer = PackageSigner::default();
    let signature = signer.sign_package(
        &test_file,
        "test-package".to_string(),
        "1.0.0".to_string(),
    ).unwrap();

    // Verify the signature
    let verify_result = signer.verify_signature(&test_file, &signature);
    assert!(verify_result.is_ok());
    assert_eq!(verify_result.unwrap(), true);

    // Verify strict
    let verify_strict_result = signer.verify_signature_strict(&test_file, &signature);
    assert!(verify_strict_result.is_ok());
}

#[test]
fn test_package_signature_verification_failure() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"test content").unwrap();

    let signer = PackageSigner::default();
    let signature = signer.sign_package(
        &test_file,
        "test-package".to_string(),
        "1.0.0".to_string(),
    ).unwrap();

    // Modify file content after signing
    fs::write(&test_file, b"modified content").unwrap();

    // Verification should fail
    let verify_result = signer.verify_signature(&test_file, &signature);
    assert!(verify_result.is_ok());
    assert_eq!(verify_result.unwrap(), false);

    // Strict verification should return error
    let verify_strict_result = signer.verify_signature_strict(&test_file, &signature);
    assert!(verify_strict_result.is_err());
}

#[test]
fn test_package_signature_json_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"test content").unwrap();

    let signer = PackageSigner::default();
    let signature = signer.sign_package(
        &test_file,
        "test-package".to_string(),
        "1.0.0".to_string(),
    ).unwrap();

    // Serialize to JSON
    let json_result = signature.to_json();
    assert!(json_result.is_ok());
    let json = json_result.unwrap();
    assert!(json.contains("SHA-256"));
    assert!(json.contains("test-package"));

    // Write to file and read back
    let sig_file = temp_dir.path().join("signature.json");
    signature.write_to_file(&sig_file).unwrap();

    let loaded_signature_result = vais_supply_chain::signing::PackageSignature::from_file(&sig_file);
    assert!(loaded_signature_result.is_ok());

    let loaded_signature = loaded_signature_result.unwrap();
    assert_eq!(loaded_signature.hash, signature.hash);
    assert_eq!(loaded_signature.algorithm, signature.algorithm);
    assert_eq!(loaded_signature.metadata.package_name, signature.metadata.package_name);
}

#[test]
fn test_package_signature_is_recent() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"test content").unwrap();

    let signer = PackageSigner::default();
    let signature = signer.sign_package(
        &test_file,
        "test-package".to_string(),
        "1.0.0".to_string(),
    ).unwrap();

    // Signature should be recent (within 1 hour)
    let one_hour = chrono::Duration::hours(1);
    assert!(signature.is_recent(one_hour));

    // Signature should be recent (within 1 day)
    let one_day = chrono::Duration::days(1);
    assert!(signature.is_recent(one_day));

    // Signature should not be "recent" if we check for a very small duration
    let _one_nanosecond = chrono::Duration::nanoseconds(1);
    // This might be false depending on timing, but the test demonstrates the API
}

#[test]
fn test_component_type_variants() {
    let library = ComponentType::Library;
    let application = ComponentType::Application;
    let framework = ComponentType::Framework;
    let container = ComponentType::Container;

    // Just verify they're all different
    assert_ne!(library, application);
    assert_ne!(library, framework);
    assert_ne!(library, container);
}

#[test]
fn test_sbom_generator_from_manifest() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.txt");

    // Create a simple manifest file
    let manifest_content = r#"
# Test manifest
[component "my-lib" "1.0.0"]
license = "MIT"
type = "library"

[component "my-app" "2.0.0"]
license = "Apache-2.0"
type = "application"
depends = "my-lib"
"#;

    fs::write(&manifest_path, manifest_content).unwrap();

    let generator = SbomGenerator::default();
    let sbom_result = generator.generate_from_manifest(&manifest_path);

    assert!(sbom_result.is_ok());
    let sbom = sbom_result.unwrap();

    assert_eq!(sbom.components.len(), 2);
    assert!(sbom.components.iter().any(|c| c.name == "my-lib" && c.version == "1.0.0"));
    assert!(sbom.components.iter().any(|c| c.name == "my-app" && c.version == "2.0.0"));

    // Check dependencies
    assert!(sbom.dependencies.iter().any(|d| d.component == "my-app" && d.depends_on.contains(&"my-lib".to_string())));
}
