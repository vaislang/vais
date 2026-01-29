# vais-supply-chain

Supply chain security tools for the Vais compiler ecosystem.

## Features

### 1. SBOM (Software Bill of Materials) Generation

Generate CycloneDX-inspired Software Bill of Materials for your Vais projects.

- Support for multiple component types (library, application, framework, etc.)
- SHA-256 hash computation for components
- License tracking (SPDX format)
- Dependency relationship mapping
- JSON serialization/deserialization
- Manifest validation

**Example:**

```rust
use vais_supply_chain::sbom::{SbomGenerator, SbomComponent, ComponentType};

let generator = SbomGenerator::default();

let components = vec![
    SbomComponent {
        name: "my-app".to_string(),
        version: "1.0.0".to_string(),
        component_type: ComponentType::Application,
        hashes: Default::default(),
        license: Some("MIT".to_string()),
        description: Some("My application".to_string()),
        publisher: None,
    },
];

let sbom = generator.generate_from_components(components, vec![]);
let json = sbom.to_json().unwrap();
```

### 2. Package Signing

Sign packages and verify their integrity using SHA-256 hashing.

- File and directory signing
- Timestamp tracking
- Signer information
- Metadata embedding
- Signature verification
- JSON format for signatures

**Example:**

```rust
use vais_supply_chain::signing::PackageSigner;

let signer = PackageSigner::new(
    "builder".to_string(),
    Some("build@example.com".to_string()),
    Some("My Org".to_string()),
);

let signature = signer.sign_package(
    "path/to/package.tar.gz",
    "my-package".to_string(),
    "1.0.0".to_string(),
).unwrap();

// Verify later
let is_valid = signer.verify_signature("path/to/package.tar.gz", &signature).unwrap();
```

### 3. Dependency Audit

Audit dependencies against a vulnerability database.

- In-memory vulnerability database
- Multiple severity levels (Low, Medium, High, Critical)
- Version range matching
- Actionable recommendations
- CVE tracking
- Bulk dependency auditing

**Example:**

```rust
use vais_supply_chain::audit::{DependencyAuditor, DependencyManifest, DependencyEntry};

let auditor = DependencyAuditor::new();

let manifest = DependencyManifest {
    dependencies: vec![
        DependencyEntry {
            name: "some-lib".to_string(),
            version: "1.0.0".to_string(),
        },
    ],
};

let results = auditor.audit_dependencies(&manifest);

for result in results {
    println!("Package: {} - Status: {:?}", result.dependency_name, result.status);
    if result.has_vulnerabilities() {
        for vuln in &result.vulnerabilities {
            println!("  - {} ({:?}): {}", vuln.id, vuln.severity, vuln.description);
        }
    }
}
```

## Manifest Format

The SBOM generator can parse simple Vais manifest files:

```toml
# vais.toml
[component "my-app" "1.0.0"]
type = "application"
license = "MIT"
depends = "crypto-lib"
depends = "json-lib"

[component "crypto-lib" "2.0.0"]
type = "library"
license = "Apache-2.0"

[component "json-lib" "1.5.0"]
type = "library"
license = "MIT"
```

## Security Considerations

- **Hash Algorithm**: Uses SHA-256 for package integrity verification
- **Timestamp Validation**: Signatures include timestamps for freshness checks
- **Vulnerability Database**: Default database includes example vulnerabilities; integrate with real CVE databases for production use
- **Version Matching**: Supports exact matches, ranges, and comparison operators

## Running Examples

```bash
cargo run -p vais-supply-chain --example usage
```

## Running Tests

```bash
cargo test -p vais-supply-chain
```

All 22 tests cover:
- SBOM generation and validation
- Package signing and verification
- Dependency auditing
- Version comparison logic
- Severity levels and scoring

## Integration with Vais Compiler

This crate can be integrated into the Vais compiler toolchain to:

1. Generate SBOMs during package building
2. Sign compiled packages before distribution
3. Audit dependencies before compilation
4. Verify package integrity during installation

## License

MIT
