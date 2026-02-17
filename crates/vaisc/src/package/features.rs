//! Feature flags management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Feature flags configuration section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureConfig {
    /// Default features enabled when no --no-default-features is passed
    #[serde(default)]
    pub default: Vec<String>,
    /// All available features and their dependencies
    /// Each feature maps to a list of other features/optional deps it enables
    #[serde(flatten)]
    pub features: HashMap<String, Vec<String>>,
}

impl FeatureConfig {
    /// Resolve the full set of enabled features given user selections
    pub fn resolve_features(&self, selected: &[String], use_defaults: bool) -> Vec<String> {
        let mut enabled = std::collections::HashSet::new();
        let mut stack: Vec<String> = selected.to_vec();

        if use_defaults {
            stack.extend(self.default.clone());
        }

        while let Some(feat) = stack.pop() {
            if enabled.insert(feat.clone()) {
                // Add transitive feature dependencies
                if let Some(deps) = self.features.get(&feat) {
                    for dep in deps {
                        if !enabled.contains(dep) {
                            stack.push(dep.clone());
                        }
                    }
                }
            }
        }

        let mut result: Vec<String> = enabled.into_iter().collect();
        result.sort();
        result
    }

    /// Get all defined feature names
    pub fn all_features(&self) -> Vec<String> {
        let mut all: Vec<String> = self.features.keys().cloned().collect();
        all.sort();
        all
    }
}
