use super::*;
use std::fs;
use tempfile::TempDir;

// ========== SBOM Tests ==========

#[test]
fn test_sbom_generator_creation() {
    let generator = SbomGenerator::new(
        "test-tool".to_string(),
        "1.0.0".to_string(),
        "Test Vendor".to_string(),
    );

    let doc = generator.generate_from_components(vec![], vec![]);
    assert_eq!(doc.tool.name, "test-tool");
    assert_eq!(doc.tool.version, "1.0.0");
    assert_eq!(doc.tool.vendor, "Test Vendor");
}

#[test]
fn test_sbom_component_creation() {
    let component = sbom::SbomComponent {
        name: "test-lib".to_string(),
        version: "1.0.0".to_string(),
        component_type: sbom::ComponentType::Library,
        hashes: std::collections::HashMap::new(),
        license: Some("MIT".to_string()),
        description: Some("Test library".to_string()),
        publisher: Some("Test Publisher".to_string()),
    };

    assert_eq!(component.name, "test-lib");
    assert_eq!(component.version, "1.0.0");
    assert_eq!(component.license, Some("MIT".to_string()));
}

#[test]
fn test_sbom_json_serialization() {
    let generator = SbomGenerator::default();
    let component = sbom::SbomComponent {
        name: "my-lib".to_string(),
        version: "2.0.0".to_string(),
        component_type: sbom::ComponentType::Library,
        hashes: std::collections::HashMap::new(),
        license: Some("Apache-2.0".to_string()),
        description: None,
        publisher: None,
    };

    let doc = generator.generate_from_components(vec![component], vec![]);
    let json = doc.to_json().unwrap();

    assert!(json.contains("my-lib"));
    assert!(json.contains("2.0.0"));
    assert!(json.contains("Apache-2.0"));
}

#[test]
fn test_sbom_manifest_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("vais.toml");

    let manifest_content = r#"
# Test manifest
[component "my-app" "1.0.0"]
type = "application"
license = "MIT"
depends = "left-pad"
depends = "crypto"

[component "left-pad" "0.1.0"]
type = "library"
license = "MIT"

[component "crypto" "2.0.0"]
type = "library"
"#;

    fs::write(&manifest_path, manifest_content).unwrap();

    let generator = SbomGenerator::default();
    let doc = generator.generate_from_manifest(&manifest_path).unwrap();

    assert_eq!(doc.components.len(), 3);
    assert_eq!(doc.dependencies.len(), 1);

    // Check that my-app depends on left-pad and crypto
    let my_app_deps = doc
        .dependencies
        .iter()
        .find(|d| d.component == "my-app")
        .unwrap();
    assert_eq!(my_app_deps.depends_on.len(), 2);
    assert!(my_app_deps.depends_on.contains(&"left-pad".to_string()));
    assert!(my_app_deps.depends_on.contains(&"crypto".to_string()));
}

#[test]
fn test_sbom_validation_success() {
    let generator = SbomGenerator::default();
    let components = vec![
        sbom::SbomComponent {
            name: "app".to_string(),
            version: "1.0.0".to_string(),
            component_type: sbom::ComponentType::Application,
            hashes: std::collections::HashMap::new(),
            license: None,
            description: None,
            publisher: None,
        },
        sbom::SbomComponent {
            name: "lib".to_string(),
            version: "1.0.0".to_string(),
            component_type: sbom::ComponentType::Library,
            hashes: std::collections::HashMap::new(),
            license: None,
            description: None,
            publisher: None,
        },
    ];

    let dependencies = vec![sbom::Dependency {
        component: "app".to_string(),
        depends_on: vec!["lib".to_string()],
    }];

    let doc = generator.generate_from_components(components, dependencies);
    assert!(doc.validate().is_ok());
}

#[test]
fn test_sbom_validation_duplicate_names() {
    let generator = SbomGenerator::default();
    let components = vec![
        sbom::SbomComponent {
            name: "duplicate".to_string(),
            version: "1.0.0".to_string(),
            component_type: sbom::ComponentType::Library,
            hashes: std::collections::HashMap::new(),
            license: None,
            description: None,
            publisher: None,
        },
        sbom::SbomComponent {
            name: "duplicate".to_string(),
            version: "2.0.0".to_string(),
            component_type: sbom::ComponentType::Library,
            hashes: std::collections::HashMap::new(),
            license: None,
            description: None,
            publisher: None,
        },
    ];

    let doc = generator.generate_from_components(components, vec![]);
    assert!(doc.validate().is_err());
}

// ========== Signing Tests ==========

#[test]
fn test_package_signer_creation() {
    let signer = PackageSigner::new(
        "test-signer".to_string(),
        Some("test@example.com".to_string()),
        Some("Test Org".to_string()),
    );

    assert_eq!(signer.signer_info.name, "test-signer");
    assert_eq!(signer.signer_info.email, Some("test@example.com".to_string()));
}

#[test]
fn test_sign_package_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    fs::write(&file_path, b"Hello, world!").unwrap();

    let signer = PackageSigner::default();
    let signature = signer
        .sign_package(&file_path, "test-package".to_string(), "1.0.0".to_string())
        .unwrap();

    assert_eq!(signature.algorithm, "SHA-256");
    assert_eq!(signature.metadata.package_name, "test-package");
    assert_eq!(signature.metadata.package_version, "1.0.0");
    assert_eq!(signature.metadata.file_size, 13);
    assert!(!signature.hash.is_empty());
}

#[test]
fn test_verify_signature_success() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("package.bin");

    fs::write(&file_path, b"package contents").unwrap();

    let signer = PackageSigner::default();
    let signature = signer
        .sign_package(&file_path, "pkg".to_string(), "1.0.0".to_string())
        .unwrap();

    let verified = signer.verify_signature(&file_path, &signature).unwrap();
    assert!(verified);
}

#[test]
fn test_verify_signature_failure() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("package.bin");

    fs::write(&file_path, b"original contents").unwrap();

    let signer = PackageSigner::default();
    let signature = signer
        .sign_package(&file_path, "pkg".to_string(), "1.0.0".to_string())
        .unwrap();

    // Modify the file
    fs::write(&file_path, b"modified contents").unwrap();

    let verified = signer.verify_signature(&file_path, &signature).unwrap();
    assert!(!verified);
}

#[test]
fn test_signature_json_serialization() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    fs::write(&file_path, b"test").unwrap();

    let signer = PackageSigner::default();
    let signature = signer
        .sign_package(&file_path, "test".to_string(), "1.0.0".to_string())
        .unwrap();

    let json = signature.to_json().unwrap();
    assert!(json.contains("SHA-256"));
    assert!(json.contains("test"));
}

#[test]
fn test_sign_directory() {
    let temp_dir = TempDir::new().unwrap();

    // Create some files
    fs::write(temp_dir.path().join("file1.txt"), b"content1").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), b"content2").unwrap();

    let signer = PackageSigner::default();
    let signature = signer
        .sign_directory(temp_dir.path(), "my-pkg".to_string(), "2.0.0".to_string())
        .unwrap();

    assert_eq!(signature.metadata.package_name, "my-pkg");
    assert!(signature.metadata.file_size > 0);
}

// ========== Audit Tests ==========

#[test]
fn test_dependency_auditor_creation() {
    let auditor = DependencyAuditor::new();
    assert!(auditor.vulnerability_count() > 0);
}

#[test]
fn test_audit_safe_dependency() {
    let auditor = DependencyAuditor::new();
    let result = auditor.audit_dependency("safe-package", "1.0.0");

    assert_eq!(result.status, audit::AuditStatus::Safe);
    assert!(!result.has_vulnerabilities());
}

#[test]
fn test_audit_vulnerable_dependency() {
    let auditor = DependencyAuditor::new();
    let result = auditor.audit_dependency("old-crypto", "1.0.0");

    assert_eq!(result.status, audit::AuditStatus::Vulnerable);
    assert!(result.has_vulnerabilities());
    assert!(result.vulnerabilities.len() > 0);
}

#[test]
fn test_audit_manifest() {
    let manifest = audit::DependencyManifest {
        dependencies: vec![
            audit::DependencyEntry {
                name: "safe-package".to_string(),
                version: "1.0.0".to_string(),
            },
            audit::DependencyEntry {
                name: "left-pad".to_string(),
                version: "1.0.0".to_string(),
            },
        ],
    };

    let auditor = DependencyAuditor::new();
    let results = auditor.audit_dependencies(&manifest);

    assert_eq!(results.len(), 2);
}

#[test]
fn test_vulnerability_severity_ordering() {
    use audit::VulnerabilitySeverity::*;

    assert!(Low < Medium);
    assert!(Medium < High);
    assert!(High < Critical);
}

#[test]
fn test_vulnerability_severity_score() {
    use audit::VulnerabilitySeverity::*;

    assert_eq!(Low.score(), 3);
    assert_eq!(Medium.score(), 5);
    assert_eq!(High.score(), 8);
    assert_eq!(Critical.score(), 10);
}

#[test]
fn test_audit_result_max_severity() {
    let result = audit::AuditResult {
        dependency_name: "test".to_string(),
        version: "1.0.0".to_string(),
        vulnerabilities: vec![
            audit::Vulnerability {
                id: "V1".to_string(),
                severity: audit::VulnerabilitySeverity::Low,
                description: "Low".to_string(),
                affected_versions: vec![],
                fixed_in: None,
                reference: None,
            },
            audit::Vulnerability {
                id: "V2".to_string(),
                severity: audit::VulnerabilitySeverity::Critical,
                description: "Critical".to_string(),
                affected_versions: vec![],
                fixed_in: None,
                reference: None,
            },
        ],
        status: audit::AuditStatus::Vulnerable,
        recommendations: vec![],
    };

    assert_eq!(
        result.max_severity(),
        Some(audit::VulnerabilitySeverity::Critical)
    );
}

#[test]
fn test_version_comparison() {
    let auditor = DependencyAuditor::new();

    // Test exact match
    let result = auditor.audit_dependency("insecure-deserialize", "1.0.0");
    assert!(result.has_vulnerabilities());

    // Test version not affected
    let result = auditor.audit_dependency("insecure-deserialize", "2.0.0");
    assert!(!result.has_vulnerabilities());
}

#[test]
fn test_add_custom_vulnerability() {
    let mut auditor = DependencyAuditor::new();

    auditor.add_vulnerability(
        "my-custom-package",
        audit::Vulnerability {
            id: "CUSTOM-001".to_string(),
            severity: audit::VulnerabilitySeverity::High,
            description: "Custom vulnerability".to_string(),
            affected_versions: vec!["1.0.0".to_string()],
            fixed_in: Some("1.1.0".to_string()),
            reference: None,
        },
    );

    let result = auditor.audit_dependency("my-custom-package", "1.0.0");
    assert!(result.has_vulnerabilities());
    assert_eq!(result.vulnerabilities[0].id, "CUSTOM-001");
}

#[test]
fn test_audit_recommendations() {
    let auditor = DependencyAuditor::new();
    let result = auditor.audit_dependency("old-crypto", "1.0.0");

    assert!(!result.recommendations.is_empty());
    // Should recommend upgrade
    assert!(result
        .recommendations
        .iter()
        .any(|r| r.contains("2.0.0")));
}
