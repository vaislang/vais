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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> FeatureConfig {
        let mut features = HashMap::new();
        features.insert("json".to_string(), vec![]);
        features.insert("async".to_string(), vec!["json".to_string()]);
        features.insert(
            "full".to_string(),
            vec!["json".to_string(), "async".to_string()],
        );
        FeatureConfig {
            default: vec!["json".to_string()],
            features,
        }
    }

    #[test]
    fn test_default_feature_config() {
        let fc = FeatureConfig::default();
        assert!(fc.default.is_empty());
        assert!(fc.features.is_empty());
    }

    #[test]
    fn test_resolve_with_defaults() {
        let fc = make_config();
        let resolved = fc.resolve_features(&[], true);
        assert!(resolved.contains(&"json".to_string()));
        assert!(!resolved.contains(&"async".to_string()));
    }

    #[test]
    fn test_resolve_without_defaults() {
        let fc = make_config();
        let resolved = fc.resolve_features(&[], false);
        assert!(resolved.is_empty());
    }

    #[test]
    fn test_resolve_explicit_feature() {
        let fc = make_config();
        let resolved = fc.resolve_features(&["async".to_string()], false);
        assert!(resolved.contains(&"async".to_string()));
        assert!(resolved.contains(&"json".to_string())); // transitive
    }

    #[test]
    fn test_resolve_transitive_full() {
        let fc = make_config();
        let resolved = fc.resolve_features(&["full".to_string()], false);
        assert_eq!(resolved.len(), 3);
        assert!(resolved.contains(&"full".to_string()));
        assert!(resolved.contains(&"async".to_string()));
        assert!(resolved.contains(&"json".to_string()));
    }

    #[test]
    fn test_resolve_explicit_plus_defaults() {
        let fc = make_config();
        let resolved = fc.resolve_features(&["async".to_string()], true);
        assert!(resolved.contains(&"json".to_string()));
        assert!(resolved.contains(&"async".to_string()));
    }

    #[test]
    fn test_resolve_unknown_feature() {
        let fc = make_config();
        let resolved = fc.resolve_features(&["nonexistent".to_string()], false);
        // Unknown features are included but have no transitive deps
        assert!(resolved.contains(&"nonexistent".to_string()));
        assert_eq!(resolved.len(), 1);
    }

    #[test]
    fn test_resolve_duplicates() {
        let fc = make_config();
        let resolved = fc.resolve_features(&["json".to_string(), "json".to_string()], false);
        assert_eq!(resolved.iter().filter(|f| *f == "json").count(), 1);
    }

    #[test]
    fn test_resolve_sorted() {
        let fc = make_config();
        let resolved = fc.resolve_features(&["full".to_string()], false);
        let mut sorted = resolved.clone();
        sorted.sort();
        assert_eq!(resolved, sorted);
    }

    #[test]
    fn test_all_features() {
        let fc = make_config();
        let all = fc.all_features();
        assert_eq!(all.len(), 3);
        assert_eq!(all, vec!["async", "full", "json"]); // sorted
    }

    #[test]
    fn test_all_features_empty() {
        let fc = FeatureConfig::default();
        assert!(fc.all_features().is_empty());
    }

    #[test]
    fn test_feature_config_serde_roundtrip() {
        let fc = FeatureConfig {
            default: vec!["json".to_string()],
            features: {
                let mut m = HashMap::new();
                m.insert("json".to_string(), vec![]);
                m
            },
        };
        let json = serde_json::to_string(&fc).unwrap();
        let parsed: FeatureConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.default, vec!["json"]);
    }

    #[test]
    fn test_resolve_circular_features() {
        // Features that reference each other
        let mut features = HashMap::new();
        features.insert("a".to_string(), vec!["b".to_string()]);
        features.insert("b".to_string(), vec!["a".to_string()]);
        let fc = FeatureConfig {
            default: vec![],
            features,
        };
        // Should not infinite loop
        let resolved = fc.resolve_features(&["a".to_string()], false);
        assert!(resolved.contains(&"a".to_string()));
        assert!(resolved.contains(&"b".to_string()));
    }

    #[test]
    fn test_resolve_deep_transitive() {
        let mut features = HashMap::new();
        features.insert("a".to_string(), vec!["b".to_string()]);
        features.insert("b".to_string(), vec!["c".to_string()]);
        features.insert("c".to_string(), vec!["d".to_string()]);
        features.insert("d".to_string(), vec![]);
        let fc = FeatureConfig {
            default: vec![],
            features,
        };
        let resolved = fc.resolve_features(&["a".to_string()], false);
        assert_eq!(resolved.len(), 4);
    }
}
