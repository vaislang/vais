use crate::{Result, SupplyChainError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomDocument {
    /// Format version (e.g., "CycloneDX-1.4")
    pub format_version: String,

    /// Tool information
    pub tool: ToolInfo,

    /// Timestamp of SBOM generation
    pub timestamp: DateTime<Utc>,

    /// List of components in the software
    pub components: Vec<SbomComponent>,

    /// Dependency relationships
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub vendor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SbomComponent {
    /// Component name
    pub name: String,

    /// Component version
    pub version: String,

    /// Component type (library, application, framework, etc.)
    #[serde(rename = "type")]
    pub component_type: ComponentType,

    /// SHA-256 hash of the component
    pub hashes: HashMap<String, String>,

    /// License identifier (SPDX format)
    pub license: Option<String>,

    /// Component description
    pub description: Option<String>,

    /// Publisher/author
    pub publisher: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ComponentType {
    Library,
    Application,
    Framework,
    Container,
    OperatingSystem,
    Device,
    Firmware,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Reference to the component (by name)
    pub component: String,

    /// List of dependencies this component depends on
    pub depends_on: Vec<String>,
}

pub struct SbomGenerator {
    tool_info: ToolInfo,
}

impl SbomGenerator {
    pub fn new(tool_name: String, tool_version: String, vendor: String) -> Self {
        Self {
            tool_info: ToolInfo {
                name: tool_name,
                version: tool_version,
                vendor,
            },
        }
    }

    /// Generate SBOM from a Vais manifest file (vais.toml or similar)
    pub fn generate_from_manifest<P: AsRef<Path>>(&self, path: P) -> Result<SbomDocument> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;

        // Parse the manifest (simple TOML-like format)
        let (components, dependencies) = self.parse_manifest(&content)?;

        Ok(SbomDocument {
            format_version: "CycloneDX-1.4".to_string(),
            tool: self.tool_info.clone(),
            timestamp: Utc::now(),
            components,
            dependencies,
        })
    }

    /// Generate SBOM from component list
    pub fn generate_from_components(
        &self,
        components: Vec<SbomComponent>,
        dependencies: Vec<Dependency>,
    ) -> SbomDocument {
        SbomDocument {
            format_version: "CycloneDX-1.4".to_string(),
            tool: self.tool_info.clone(),
            timestamp: Utc::now(),
            components,
            dependencies,
        }
    }

    /// Parse a simple manifest format
    fn parse_manifest(&self, content: &str) -> Result<(Vec<SbomComponent>, Vec<Dependency>)> {
        let mut components = Vec::new();
        let mut dependencies = Vec::new();
        let mut current_component: Option<String> = None;
        let mut current_deps: Vec<String> = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Component declaration: [component "name" "version"]
            if line.starts_with("[component") {
                // Save previous component's dependencies
                if let Some(comp_name) = current_component.take() {
                    if !current_deps.is_empty() {
                        dependencies.push(Dependency {
                            component: comp_name,
                            depends_on: current_deps.clone(),
                        });
                        current_deps.clear();
                    }
                }

                let parts: Vec<&str> = line
                    .trim_start_matches("[component")
                    .trim_end_matches(']')
                    .split('"')
                    .filter(|s| !s.trim().is_empty())
                    .collect();

                if parts.len() >= 2 {
                    let name = parts[0].to_string();
                    let version = parts[1].to_string();

                    components.push(SbomComponent {
                        name: name.clone(),
                        version,
                        component_type: ComponentType::Library,
                        hashes: HashMap::new(),
                        license: None,
                        description: None,
                        publisher: None,
                    });

                    current_component = Some(name);
                }
            }
            // Dependency declaration: depends = "name"
            else if line.starts_with("depends") {
                if let Some(dep_name) = line.split('"').nth(1) {
                    current_deps.push(dep_name.to_string());
                }
            }
            // License declaration: license = "MIT"
            else if line.starts_with("license") {
                if let Some(license) = line.split('"').nth(1) {
                    if let Some(last_component) = components.last_mut() {
                        last_component.license = Some(license.to_string());
                    }
                }
            }
            // Type declaration: type = "application"
            else if line.starts_with("type") {
                if let Some(type_str) = line.split('"').nth(1) {
                    if let Some(last_component) = components.last_mut() {
                        last_component.component_type = match type_str {
                            "application" => ComponentType::Application,
                            "framework" => ComponentType::Framework,
                            "container" => ComponentType::Container,
                            _ => ComponentType::Library,
                        };
                    }
                }
            }
        }

        // Save last component's dependencies
        if let Some(comp_name) = current_component {
            if !current_deps.is_empty() {
                dependencies.push(Dependency {
                    component: comp_name,
                    depends_on: current_deps,
                });
            }
        }

        Ok((components, dependencies))
    }

    /// Compute hash for a file and add to component
    pub fn add_file_hash(&self, component: &mut SbomComponent, file_path: &Path) -> Result<()> {
        let content = fs::read(file_path)?;
        let hash = self.compute_sha256(&content);
        component.hashes.insert("SHA-256".to_string(), hash);
        Ok(())
    }

    /// Compute SHA-256 hash
    fn compute_sha256(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
}

impl SbomDocument {
    /// Serialize SBOM to JSON
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Serialize SBOM to JSON and write to file
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = self.to_json()?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load SBOM from JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    /// Validate SBOM integrity
    pub fn validate(&self) -> Result<()> {
        // Check for duplicate component names
        let mut names = std::collections::HashSet::new();
        for component in &self.components {
            if !names.insert(&component.name) {
                return Err(SupplyChainError::InvalidManifest(format!(
                    "Duplicate component name: {}",
                    component.name
                )));
            }
        }

        // Verify all dependencies reference valid components
        for dep in &self.dependencies {
            if !names.contains(&dep.component) {
                return Err(SupplyChainError::InvalidManifest(format!(
                    "Dependency references unknown component: {}",
                    dep.component
                )));
            }

            for dep_name in &dep.depends_on {
                if !names.contains(dep_name) {
                    return Err(SupplyChainError::InvalidManifest(format!(
                        "Dependency '{}' references unknown component: {}",
                        dep.component, dep_name
                    )));
                }
            }
        }

        Ok(())
    }
}

impl Default for SbomGenerator {
    fn default() -> Self {
        Self::new(
            "vais-supply-chain".to_string(),
            "0.0.1".to_string(),
            "Vais Team".to_string(),
        )
    }
}
