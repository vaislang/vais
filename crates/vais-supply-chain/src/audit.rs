use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditResult {
    /// Dependency name
    pub dependency_name: String,

    /// Dependency version
    pub version: String,

    /// Known vulnerabilities
    pub vulnerabilities: Vec<Vulnerability>,

    /// Audit status
    pub status: AuditStatus,

    /// Recommendations
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Vulnerability {
    /// Vulnerability ID (e.g., CVE-2023-12345)
    pub id: String,

    /// Severity level
    pub severity: VulnerabilitySeverity,

    /// Description
    pub description: String,

    /// Affected versions
    pub affected_versions: Vec<String>,

    /// Fixed in version (if available)
    pub fixed_in: Option<String>,

    /// Reference URL
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum VulnerabilitySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuditStatus {
    Safe,
    Warning,
    Vulnerable,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyManifest {
    pub dependencies: Vec<DependencyEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEntry {
    pub name: String,
    pub version: String,
}

pub struct DependencyAuditor {
    /// In-memory vulnerability database
    vulnerability_db: HashMap<String, Vec<Vulnerability>>,
}

impl DependencyAuditor {
    pub fn new() -> Self {
        let mut auditor = Self {
            vulnerability_db: HashMap::new(),
        };

        // Initialize with some example vulnerabilities
        auditor.load_default_vulnerabilities();
        auditor
    }

    /// Load default vulnerability database
    fn load_default_vulnerabilities(&mut self) {
        // Example vulnerabilities for demonstration
        self.add_vulnerability(
            "left-pad",
            Vulnerability {
                id: "VAIS-2024-0001".to_string(),
                severity: VulnerabilitySeverity::Low,
                description: "Unpublished package vulnerability".to_string(),
                affected_versions: vec!["*".to_string()],
                fixed_in: None,
                reference: Some("https://example.com/advisories/left-pad".to_string()),
            },
        );

        self.add_vulnerability(
            "old-crypto",
            Vulnerability {
                id: "CVE-2023-99999".to_string(),
                severity: VulnerabilitySeverity::Critical,
                description: "Cryptographic weakness in hash function".to_string(),
                affected_versions: vec!["< 2.0.0".to_string()],
                fixed_in: Some("2.0.0".to_string()),
                reference: Some("https://nvd.nist.gov/vuln/detail/CVE-2023-99999".to_string()),
            },
        );

        self.add_vulnerability(
            "insecure-deserialize",
            Vulnerability {
                id: "CVE-2024-11111".to_string(),
                severity: VulnerabilitySeverity::High,
                description: "Remote code execution via unsafe deserialization".to_string(),
                affected_versions: vec!["1.0.0".to_string(), "1.1.0".to_string()],
                fixed_in: Some("1.2.0".to_string()),
                reference: Some("https://example.com/security/CVE-2024-11111".to_string()),
            },
        );

        self.add_vulnerability(
            "memory-leak",
            Vulnerability {
                id: "VAIS-2024-0002".to_string(),
                severity: VulnerabilitySeverity::Medium,
                description: "Memory leak in connection pooling".to_string(),
                affected_versions: vec!["0.5.0 - 0.5.9".to_string()],
                fixed_in: Some("0.6.0".to_string()),
                reference: None,
            },
        );
    }

    /// Add a vulnerability to the database
    pub fn add_vulnerability(&mut self, package_name: &str, vulnerability: Vulnerability) {
        self.vulnerability_db
            .entry(package_name.to_string())
            .or_default()
            .push(vulnerability);
    }

    /// Audit dependencies from a manifest
    pub fn audit_dependencies(&self, manifest: &DependencyManifest) -> Vec<AuditResult> {
        manifest
            .dependencies
            .iter()
            .map(|dep| self.audit_dependency(&dep.name, &dep.version))
            .collect()
    }

    /// Audit a single dependency
    pub fn audit_dependency(&self, name: &str, version: &str) -> AuditResult {
        let vulnerabilities = self.check_vulnerabilities(name, version);

        let status = if vulnerabilities.is_empty() {
            AuditStatus::Safe
        } else {
            let has_critical = vulnerabilities
                .iter()
                .any(|v| v.severity == VulnerabilitySeverity::Critical);
            let has_high = vulnerabilities
                .iter()
                .any(|v| v.severity == VulnerabilitySeverity::High);

            if has_critical || has_high {
                AuditStatus::Vulnerable
            } else {
                AuditStatus::Warning
            }
        };

        let recommendations = self.generate_recommendations(&vulnerabilities);

        AuditResult {
            dependency_name: name.to_string(),
            version: version.to_string(),
            vulnerabilities,
            status,
            recommendations,
        }
    }

    /// Check for vulnerabilities affecting a specific package version
    fn check_vulnerabilities(&self, name: &str, version: &str) -> Vec<Vulnerability> {
        if let Some(vulns) = self.vulnerability_db.get(name) {
            vulns
                .iter()
                .filter(|v| self.version_matches(version, &v.affected_versions))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Check if a version matches any of the affected version patterns
    fn version_matches(&self, version: &str, patterns: &[String]) -> bool {
        for pattern in patterns {
            if pattern == "*" {
                return true;
            }

            // Simple version matching
            if pattern.starts_with('<') {
                // e.g., "< 2.0.0"
                if let Some(target_version) = pattern.strip_prefix("< ") {
                    if self.compare_versions(version, target_version) < 0 {
                        return true;
                    }
                }
            } else if pattern.contains(" - ") {
                // Range: "0.5.0 - 0.5.9"
                let parts: Vec<&str> = pattern.split(" - ").collect();
                if parts.len() == 2
                    && self.compare_versions(version, parts[0]) >= 0
                    && self.compare_versions(version, parts[1]) <= 0
                {
                    return true;
                }
            } else if pattern == version {
                return true;
            }
        }

        false
    }

    /// Simple version comparison (returns -1, 0, or 1)
    fn compare_versions(&self, v1: &str, v2: &str) -> i32 {
        let parts1: Vec<u32> = v1.split('.').filter_map(|s| s.parse().ok()).collect();
        let parts2: Vec<u32> = v2.split('.').filter_map(|s| s.parse().ok()).collect();

        for i in 0..std::cmp::max(parts1.len(), parts2.len()) {
            let p1 = parts1.get(i).unwrap_or(&0);
            let p2 = parts2.get(i).unwrap_or(&0);

            if p1 < p2 {
                return -1;
            } else if p1 > p2 {
                return 1;
            }
        }

        0
    }

    /// Generate recommendations based on vulnerabilities
    fn generate_recommendations(&self, vulnerabilities: &[Vulnerability]) -> Vec<String> {
        let mut recommendations = Vec::new();

        if vulnerabilities.is_empty() {
            recommendations.push("No known vulnerabilities found.".to_string());
            return recommendations;
        }

        // Check for critical vulnerabilities
        let critical_count = vulnerabilities
            .iter()
            .filter(|v| v.severity == VulnerabilitySeverity::Critical)
            .count();

        if critical_count > 0 {
            recommendations.push(format!(
                "CRITICAL: {} critical vulnerabilities found. Immediate action required.",
                critical_count
            ));
        }

        // Suggest upgrades
        for vuln in vulnerabilities {
            if let Some(ref fixed_version) = vuln.fixed_in {
                recommendations.push(format!(
                    "Upgrade to version {} to fix {} ({})",
                    fixed_version, vuln.id, vuln.severity as u8
                ));
            } else {
                recommendations.push(format!(
                    "No fix available for {} ({}). Consider alternative packages.",
                    vuln.id, vuln.severity as u8
                ));
            }
        }

        recommendations
    }

    /// Get total vulnerability count in database
    pub fn vulnerability_count(&self) -> usize {
        self.vulnerability_db.values().map(|v| v.len()).sum()
    }

    /// Get all packages with known vulnerabilities
    pub fn vulnerable_packages(&self) -> Vec<String> {
        self.vulnerability_db.keys().cloned().collect()
    }
}

impl Default for DependencyAuditor {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditResult {
    /// Check if the dependency has any vulnerabilities
    pub fn has_vulnerabilities(&self) -> bool {
        !self.vulnerabilities.is_empty()
    }

    /// Get highest severity level
    pub fn max_severity(&self) -> Option<VulnerabilitySeverity> {
        self.vulnerabilities.iter().map(|v| v.severity).max()
    }

    /// Get count by severity
    pub fn count_by_severity(&self, severity: VulnerabilitySeverity) -> usize {
        self.vulnerabilities
            .iter()
            .filter(|v| v.severity == severity)
            .count()
    }
}

impl VulnerabilitySeverity {
    /// Get numeric score for severity (0-10 scale)
    pub fn score(&self) -> u8 {
        match self {
            VulnerabilitySeverity::Low => 3,
            VulnerabilitySeverity::Medium => 5,
            VulnerabilitySeverity::High => 8,
            VulnerabilitySeverity::Critical => 10,
        }
    }
}
