use vais_supply_chain::{
    audit::{AuditStatus, DependencyAuditor, DependencyEntry, DependencyManifest},
    sbom::{ComponentType, SbomComponent, SbomGenerator},
    signing::PackageSigner,
};

fn main() {
    println!("=== Vais Supply Chain Security Demo ===\n");

    // 1. SBOM Generation
    println!("1. SBOM Generation");
    println!("{}", "-".repeat(50));

    let sbom_gen = SbomGenerator::default();

    let components = vec![
        SbomComponent {
            name: "my-vais-app".to_string(),
            version: "1.0.0".to_string(),
            component_type: ComponentType::Application,
            hashes: std::collections::HashMap::new(),
            license: Some("MIT".to_string()),
            description: Some("Main application".to_string()),
            publisher: Some("Vais Team".to_string()),
        },
        SbomComponent {
            name: "crypto-lib".to_string(),
            version: "2.1.0".to_string(),
            component_type: ComponentType::Library,
            hashes: std::collections::HashMap::new(),
            license: Some("Apache-2.0".to_string()),
            description: Some("Cryptography utilities".to_string()),
            publisher: None,
        },
    ];

    let dependencies = vec![vais_supply_chain::sbom::Dependency {
        component: "my-vais-app".to_string(),
        depends_on: vec!["crypto-lib".to_string()],
    }];

    let sbom = sbom_gen.generate_from_components(components, dependencies);
    println!("Generated SBOM with {} components", sbom.components.len());
    println!("Format: {}", sbom.format_version);
    println!("Tool: {} v{}", sbom.tool.name, sbom.tool.version);

    if let Ok(json) = sbom.to_json() {
        println!(
            "\nSBOM JSON (first 200 chars):\n{}",
            &json[..200.min(json.len())]
        );
    }

    // 2. Package Signing
    println!("\n\n2. Package Signing");
    println!("{}", "-".repeat(50));

    let signer = PackageSigner::new(
        "vais-builder".to_string(),
        Some("build@vais-lang.org".to_string()),
        Some("Vais Team".to_string()),
    );

    println!("Signer: {}", signer.signer_info.name);
    if let Some(ref email) = signer.signer_info.email {
        println!("Email: {}", email);
    }

    // Note: In real usage, you would sign actual files
    println!("\nPackage signing would compute SHA-256 hashes of:");
    println!("  - Package files");
    println!("  - Metadata");
    println!("  - Timestamp");

    // 3. Dependency Audit
    println!("\n\n3. Dependency Audit");
    println!("{}", "-".repeat(50));

    let auditor = DependencyAuditor::new();
    println!(
        "Loaded vulnerability database with {} entries",
        auditor.vulnerability_count()
    );

    let manifest = DependencyManifest {
        dependencies: vec![
            DependencyEntry {
                name: "safe-lib".to_string(),
                version: "1.0.0".to_string(),
            },
            DependencyEntry {
                name: "old-crypto".to_string(),
                version: "1.0.0".to_string(),
            },
            DependencyEntry {
                name: "left-pad".to_string(),
                version: "0.1.0".to_string(),
            },
        ],
    };

    let results = auditor.audit_dependencies(&manifest);

    println!("\nAudit Results:");
    for result in results {
        println!(
            "\n  Package: {} v{}",
            result.dependency_name, result.version
        );
        print!("  Status: ");
        match result.status {
            AuditStatus::Safe => println!("✓ SAFE"),
            AuditStatus::Warning => println!("⚠ WARNING"),
            AuditStatus::Vulnerable => println!("✗ VULNERABLE"),
            AuditStatus::Unknown => println!("? UNKNOWN"),
        }

        if result.has_vulnerabilities() {
            println!("  Vulnerabilities found: {}", result.vulnerabilities.len());
            for vuln in &result.vulnerabilities {
                println!("    - {} ({:?})", vuln.id, vuln.severity);
                println!("      {}", vuln.description);
                if let Some(ref fixed_in) = vuln.fixed_in {
                    println!("      Fixed in: {}", fixed_in);
                }
            }
        }

        if !result.recommendations.is_empty() {
            println!("  Recommendations:");
            for rec in &result.recommendations {
                println!("    - {}", rec);
            }
        }
    }

    println!("\n=== Demo Complete ===");
}
