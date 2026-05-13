//! Integration tests for vais-supply-chain crate
//!
//! Tests SBOM generation, dependency auditing, and package signing.

use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;
use vais_supply_chain::{
    audit::{
        AuditStatus, DependencyAuditor, DependencyEntry, DependencyManifest, VulnerabilitySeverity,
    },
    sbom::{ComponentType, SbomComponent, SbomGenerator},
    signing::PackageSigner,
};

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
    assert_eq!(
        sbom.components[0].hashes.get("SHA-256"),
        Some(&"abc123".to_string())
    );
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
        dependencies: vec![DependencyEntry {
            name: "safe-package".to_string(),
            version: "1.0.0".to_string(),
        }],
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
        dependencies: vec![DependencyEntry {
            name: "old-crypto".to_string(),
            version: "1.0.0".to_string(),
        }],
    };

    let results = auditor.audit_dependencies(&manifest);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].dependency_name, "old-crypto");
    assert_eq!(results[0].status, AuditStatus::Vulnerable);
    assert!(results[0].has_vulnerabilities());
    assert_eq!(
        results[0].max_severity(),
        Some(VulnerabilitySeverity::Critical)
    );
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
        dependencies: vec![DependencyEntry {
            name: "old-crypto".to_string(),
            version: "1.0.0".to_string(),
        }],
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
    assert_eq!(
        signer.signer_info.email,
        Some("test@example.com".to_string())
    );
    assert_eq!(
        signer.signer_info.organization,
        Some("Test Org".to_string())
    );
}

#[test]
fn test_package_signer_default() {
    let signer = PackageSigner::default();
    assert_eq!(signer.signer_info.name, "vais-builder");
    assert_eq!(
        signer.signer_info.email,
        Some("build@vais-lang.org".to_string())
    );
    assert_eq!(
        signer.signer_info.organization,
        Some("Vais Team".to_string())
    );
}

#[test]
fn test_package_signature_structure() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"test content").unwrap();

    let signer = PackageSigner::default();
    let signature_result =
        signer.sign_package(&test_file, "test-package".to_string(), "1.0.0".to_string());

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
    let signature = signer
        .sign_package(&test_file, "test-package".to_string(), "1.0.0".to_string())
        .unwrap();

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
    let signature = signer
        .sign_package(&test_file, "test-package".to_string(), "1.0.0".to_string())
        .unwrap();

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
    let signature = signer
        .sign_package(&test_file, "test-package".to_string(), "1.0.0".to_string())
        .unwrap();

    // Serialize to JSON
    let json_result = signature.to_json();
    assert!(json_result.is_ok());
    let json = json_result.unwrap();
    assert!(json.contains("SHA-256"));
    assert!(json.contains("test-package"));

    // Write to file and read back
    let sig_file = temp_dir.path().join("signature.json");
    signature.write_to_file(&sig_file).unwrap();

    let loaded_signature_result =
        vais_supply_chain::signing::PackageSignature::from_file(&sig_file);
    assert!(loaded_signature_result.is_ok());

    let loaded_signature = loaded_signature_result.unwrap();
    assert_eq!(loaded_signature.hash, signature.hash);
    assert_eq!(loaded_signature.algorithm, signature.algorithm);
    assert_eq!(
        loaded_signature.metadata.package_name,
        signature.metadata.package_name
    );
}

#[test]
fn test_package_signature_is_recent() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"test content").unwrap();

    let signer = PackageSigner::default();
    let signature = signer
        .sign_package(&test_file, "test-package".to_string(), "1.0.0".to_string())
        .unwrap();

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
    assert!(sbom
        .components
        .iter()
        .any(|c| c.name == "my-lib" && c.version == "1.0.0"));
    assert!(sbom
        .components
        .iter()
        .any(|c| c.name == "my-app" && c.version == "2.0.0"));

    // Check dependencies
    assert!(sbom
        .dependencies
        .iter()
        .any(|d| d.component == "my-app" && d.depends_on.contains(&"my-lib".to_string())));
}

// ============================================================================
// SBOM Advanced Tests (3 tests)
// ============================================================================

#[test]
fn test_sbom_empty_validation() {
    let generator = SbomGenerator::default();
    let components = vec![];
    let dependencies = vec![];
    let sbom = generator.generate_from_components(components, dependencies);

    // Empty SBOM should validate successfully
    assert!(sbom.validate().is_ok());
    assert_eq!(sbom.components.len(), 0);
    assert_eq!(sbom.dependencies.len(), 0);
}

#[test]
fn test_sbom_multiple_components() {
    let generator = SbomGenerator::default();

    let component1 = SbomComponent {
        name: "lib-a".to_string(),
        version: "1.0.0".to_string(),
        component_type: ComponentType::Library,
        hashes: HashMap::new(),
        license: Some("MIT".to_string()),
        description: Some("Component A".to_string()),
        publisher: Some("Publisher A".to_string()),
    };

    let component2 = SbomComponent {
        name: "lib-b".to_string(),
        version: "2.0.0".to_string(),
        component_type: ComponentType::Framework,
        hashes: HashMap::new(),
        license: Some("Apache-2.0".to_string()),
        description: Some("Component B".to_string()),
        publisher: Some("Publisher B".to_string()),
    };

    let component3 = SbomComponent {
        name: "lib-c".to_string(),
        version: "3.0.0".to_string(),
        component_type: ComponentType::Application,
        hashes: HashMap::new(),
        license: Some("GPL-3.0".to_string()),
        description: Some("Component C".to_string()),
        publisher: Some("Publisher C".to_string()),
    };

    let components = vec![component1, component2, component3];
    let dependencies = vec![];
    let sbom = generator.generate_from_components(components, dependencies);

    assert_eq!(sbom.components.len(), 3);
    assert!(sbom.validate().is_ok());

    // Verify all components are present
    assert!(sbom.components.iter().any(|c| c.name == "lib-a"));
    assert!(sbom.components.iter().any(|c| c.name == "lib-b"));
    assert!(sbom.components.iter().any(|c| c.name == "lib-c"));
}

#[test]
fn test_sbom_dependencies_validation() {
    use vais_supply_chain::sbom::Dependency;

    let generator = SbomGenerator::default();

    let component1 = SbomComponent {
        name: "app".to_string(),
        version: "1.0.0".to_string(),
        component_type: ComponentType::Application,
        hashes: HashMap::new(),
        license: None,
        description: None,
        publisher: None,
    };

    let component2 = SbomComponent {
        name: "lib-a".to_string(),
        version: "1.0.0".to_string(),
        component_type: ComponentType::Library,
        hashes: HashMap::new(),
        license: None,
        description: None,
        publisher: None,
    };

    let component3 = SbomComponent {
        name: "lib-b".to_string(),
        version: "1.0.0".to_string(),
        component_type: ComponentType::Library,
        hashes: HashMap::new(),
        license: None,
        description: None,
        publisher: None,
    };

    let components = vec![component1, component2, component3];

    // app depends on lib-a and lib-b
    let dependencies = vec![Dependency {
        component: "app".to_string(),
        depends_on: vec!["lib-a".to_string(), "lib-b".to_string()],
    }];

    let sbom = generator.generate_from_components(components, dependencies);

    // Should validate successfully
    assert!(sbom.validate().is_ok());

    // Verify dependency relationships
    assert_eq!(sbom.dependencies.len(), 1);
    assert_eq!(sbom.dependencies[0].component, "app");
    assert_eq!(sbom.dependencies[0].depends_on.len(), 2);
    assert!(sbom.dependencies[0]
        .depends_on
        .contains(&"lib-a".to_string()));
    assert!(sbom.dependencies[0]
        .depends_on
        .contains(&"lib-b".to_string()));
}

// ============================================================================
// Audit Advanced Tests (3 tests)
// ============================================================================

#[test]
fn test_audit_multiple_vulnerabilities() {
    use vais_supply_chain::audit::Vulnerability;

    let mut auditor = DependencyAuditor::new();

    // Add a package with multiple vulnerabilities
    auditor.add_vulnerability(
        "multi-vuln-pkg",
        Vulnerability {
            id: "CVE-2024-0001".to_string(),
            severity: VulnerabilitySeverity::Low,
            description: "Minor issue".to_string(),
            affected_versions: vec!["1.0.0".to_string()],
            fixed_in: Some("1.0.1".to_string()),
            reference: None,
        },
    );

    auditor.add_vulnerability(
        "multi-vuln-pkg",
        Vulnerability {
            id: "CVE-2024-0002".to_string(),
            severity: VulnerabilitySeverity::Medium,
            description: "Medium issue".to_string(),
            affected_versions: vec!["1.0.0".to_string()],
            fixed_in: Some("1.0.2".to_string()),
            reference: None,
        },
    );

    auditor.add_vulnerability(
        "multi-vuln-pkg",
        Vulnerability {
            id: "CVE-2024-0003".to_string(),
            severity: VulnerabilitySeverity::High,
            description: "High severity issue".to_string(),
            affected_versions: vec!["1.0.0".to_string()],
            fixed_in: Some("1.1.0".to_string()),
            reference: None,
        },
    );

    let manifest = DependencyManifest {
        dependencies: vec![DependencyEntry {
            name: "multi-vuln-pkg".to_string(),
            version: "1.0.0".to_string(),
        }],
    };

    let results = auditor.audit_dependencies(&manifest);
    assert_eq!(results.len(), 1);

    let result = &results[0];
    assert_eq!(result.vulnerabilities.len(), 3);
    assert_eq!(result.status, AuditStatus::Vulnerable);
    assert_eq!(result.max_severity(), Some(VulnerabilitySeverity::High));

    // Check counts by severity
    assert_eq!(result.count_by_severity(VulnerabilitySeverity::Low), 1);
    assert_eq!(result.count_by_severity(VulnerabilitySeverity::Medium), 1);
    assert_eq!(result.count_by_severity(VulnerabilitySeverity::High), 1);
    assert_eq!(result.count_by_severity(VulnerabilitySeverity::Critical), 0);
}

#[test]
fn test_audit_severity_scores() {
    // Verify severity score mappings
    assert_eq!(VulnerabilitySeverity::Low.score(), 3);
    assert_eq!(VulnerabilitySeverity::Medium.score(), 5);
    assert_eq!(VulnerabilitySeverity::High.score(), 8);
    assert_eq!(VulnerabilitySeverity::Critical.score(), 10);

    // Verify ordering
    let mut severities = vec![
        VulnerabilitySeverity::Critical,
        VulnerabilitySeverity::Low,
        VulnerabilitySeverity::High,
        VulnerabilitySeverity::Medium,
    ];

    severities.sort();

    assert_eq!(severities[0], VulnerabilitySeverity::Low);
    assert_eq!(severities[1], VulnerabilitySeverity::Medium);
    assert_eq!(severities[2], VulnerabilitySeverity::High);
    assert_eq!(severities[3], VulnerabilitySeverity::Critical);

    // Verify scores increase with severity
    assert!(VulnerabilitySeverity::Low.score() < VulnerabilitySeverity::Medium.score());
    assert!(VulnerabilitySeverity::Medium.score() < VulnerabilitySeverity::High.score());
    assert!(VulnerabilitySeverity::High.score() < VulnerabilitySeverity::Critical.score());
}

#[test]
fn test_audit_all_statuses() {
    let mut auditor = DependencyAuditor::new();

    // Safe package (no vulnerabilities)
    let safe_result = auditor.audit_dependency("safe-pkg", "1.0.0");
    assert_eq!(safe_result.status, AuditStatus::Safe);
    assert!(!safe_result.has_vulnerabilities());

    // Warning status (Low/Medium severity only)
    use vais_supply_chain::audit::Vulnerability;
    auditor.add_vulnerability(
        "warning-pkg",
        Vulnerability {
            id: "WARN-001".to_string(),
            severity: VulnerabilitySeverity::Low,
            description: "Low severity issue".to_string(),
            affected_versions: vec!["1.0.0".to_string()],
            fixed_in: Some("1.0.1".to_string()),
            reference: None,
        },
    );

    let warning_result = auditor.audit_dependency("warning-pkg", "1.0.0");
    assert_eq!(warning_result.status, AuditStatus::Warning);
    assert!(warning_result.has_vulnerabilities());

    // Vulnerable status (High severity) - using existing package
    let vulnerable_result = auditor.audit_dependency("insecure-deserialize", "1.0.0");
    assert_eq!(vulnerable_result.status, AuditStatus::Vulnerable);
    assert!(vulnerable_result.has_vulnerabilities());

    // Vulnerable status (Critical severity) - using existing package
    let critical_result = auditor.audit_dependency("old-crypto", "1.0.0");
    assert_eq!(critical_result.status, AuditStatus::Vulnerable);
    assert!(critical_result.has_vulnerabilities());
}

// ============================================================================
// Signing Advanced Tests (2 tests)
// ============================================================================

#[test]
fn test_signing_directory_modification_detection() {
    let temp_dir = TempDir::new().unwrap();
    let pkg_dir = temp_dir.path().join("package");
    fs::create_dir(&pkg_dir).unwrap();

    // Create multiple files in directory
    let file1 = pkg_dir.join("file1.txt");
    let file2 = pkg_dir.join("file2.txt");
    fs::write(&file1, b"content 1").unwrap();
    fs::write(&file2, b"content 2").unwrap();

    let signer = PackageSigner::default();

    // Sign the directory
    let signature = signer
        .sign_directory(&pkg_dir, "test-pkg".to_string(), "1.0.0".to_string())
        .unwrap();

    // Verify signature is valid
    assert!(signer.verify_signature(&pkg_dir, &signature).unwrap());

    // Modify one file
    fs::write(&file1, b"modified content 1").unwrap();

    // Verification should now fail
    let verify_after_modification = signer.verify_signature(&pkg_dir, &signature).unwrap();
    assert!(!verify_after_modification);

    // Strict verification should return error
    let strict_result = signer.verify_signature_strict(&pkg_dir, &signature);
    assert!(strict_result.is_err());
}

#[test]
fn test_signing_complete_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.bin");
    let test_content = b"binary content data";
    fs::write(&test_file, test_content).unwrap();

    // Create signer with full metadata
    let signer = PackageSigner::new(
        "John Doe".to_string(),
        Some("john@example.com".to_string()),
        Some("Example Corp".to_string()),
    );

    let signature = signer
        .sign_package(&test_file, "example-pkg".to_string(), "2.3.4".to_string())
        .unwrap();

    // Verify all SignerInfo fields
    assert_eq!(signature.signer.name, "John Doe");
    assert_eq!(signature.signer.email, Some("john@example.com".to_string()));
    assert_eq!(
        signature.signer.organization,
        Some("Example Corp".to_string())
    );

    // Verify all SignatureMetadata fields
    assert_eq!(signature.metadata.package_name, "example-pkg");
    assert_eq!(signature.metadata.package_version, "2.3.4");
    assert_eq!(signature.metadata.file_size, test_content.len() as u64);
    assert_eq!(signature.metadata.tool_version, "0.0.1");

    // Verify other signature fields
    assert_eq!(signature.algorithm, "SHA-256");
    assert!(!signature.hash.is_empty());
    assert_eq!(signature.hash.len(), 64); // SHA-256 hex string is 64 characters

    // Verify signature timestamp is recent
    let one_minute = chrono::Duration::minutes(1);
    assert!(signature.is_recent(one_minute));
}

// ============================================================================
// Edge Cases Tests (2 tests)
// ============================================================================

#[test]
fn test_sbom_empty_manifest_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("empty_manifest.txt");

    // Create empty manifest with only comments and whitespace
    let empty_manifest = r#"
# This is a comment
# Another comment

    # Indented comment

"#;

    fs::write(&manifest_path, empty_manifest).unwrap();

    let generator = SbomGenerator::default();
    let sbom_result = generator.generate_from_manifest(&manifest_path);

    assert!(sbom_result.is_ok());
    let sbom = sbom_result.unwrap();

    // Should produce empty SBOM
    assert_eq!(sbom.components.len(), 0);
    assert_eq!(sbom.dependencies.len(), 0);

    // Empty SBOM should validate
    assert!(sbom.validate().is_ok());
}

#[test]
fn test_component_type_all_variants() {
    // Test all ComponentType variants
    let types = vec![
        ComponentType::Library,
        ComponentType::Application,
        ComponentType::Framework,
        ComponentType::Container,
        ComponentType::OperatingSystem,
        ComponentType::Device,
        ComponentType::Firmware,
        ComponentType::File,
    ];

    let generator = SbomGenerator::default();

    // Create components with each type
    let components: Vec<SbomComponent> = types
        .iter()
        .enumerate()
        .map(|(i, component_type)| SbomComponent {
            name: format!("component-{}", i),
            version: "1.0.0".to_string(),
            component_type: component_type.clone(),
            hashes: HashMap::new(),
            license: None,
            description: None,
            publisher: None,
        })
        .collect();

    let sbom = generator.generate_from_components(components.clone(), vec![]);

    assert_eq!(sbom.components.len(), 8);

    // Verify each component type is present
    assert!(sbom
        .components
        .iter()
        .any(|c| c.component_type == ComponentType::Library));
    assert!(sbom
        .components
        .iter()
        .any(|c| c.component_type == ComponentType::Application));
    assert!(sbom
        .components
        .iter()
        .any(|c| c.component_type == ComponentType::Framework));
    assert!(sbom
        .components
        .iter()
        .any(|c| c.component_type == ComponentType::Container));
    assert!(sbom
        .components
        .iter()
        .any(|c| c.component_type == ComponentType::OperatingSystem));
    assert!(sbom
        .components
        .iter()
        .any(|c| c.component_type == ComponentType::Device));
    assert!(sbom
        .components
        .iter()
        .any(|c| c.component_type == ComponentType::Firmware));
    assert!(sbom
        .components
        .iter()
        .any(|c| c.component_type == ComponentType::File));

    // All types should be distinct
    for (i, type1) in types.iter().enumerate() {
        for (j, type2) in types.iter().enumerate() {
            if i != j {
                assert_ne!(type1, type2);
            }
        }
    }
}
