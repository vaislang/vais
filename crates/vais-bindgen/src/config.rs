use std::collections::HashMap;

/// Configuration for bindgen
#[derive(Debug, Clone)]
pub struct BindgenConfig {
    /// Library name for extern block
    library_name: Option<String>,

    /// Custom type mappings (C type -> Vais type)
    type_mappings: HashMap<String, String>,

    /// Whether to generate comments
    generate_comments: bool,

    /// Whether to generate wrapper functions
    generate_wrappers: bool,

    /// Prefix for generated items
    prefix: Option<String>,

    /// Suffix for generated items
    suffix: Option<String>,

    /// Types to allowlist (if empty, all types are included)
    allowlist: Vec<String>,

    /// Types to blocklist
    blocklist: Vec<String>,
}

impl BindgenConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the library name for the extern block
    pub fn set_library_name(&mut self, name: &str) -> &mut Self {
        self.library_name = Some(name.to_string());
        self
    }

    /// Get the library name
    pub fn library_name(&self) -> Option<&str> {
        self.library_name.as_deref()
    }

    /// Add a custom type mapping
    pub fn add_type_mapping(&mut self, c_type: &str, vais_type: &str) -> &mut Self {
        self.type_mappings
            .insert(c_type.to_string(), vais_type.to_string());
        self
    }

    /// Get a type mapping
    pub fn get_type_mapping(&self, c_type: &str) -> Option<&str> {
        self.type_mappings.get(c_type).map(|s| s.as_str())
    }

    /// Enable or disable comment generation
    pub fn set_generate_comments(&mut self, enable: bool) -> &mut Self {
        self.generate_comments = enable;
        self
    }

    /// Check if comments should be generated
    pub fn generate_comments(&self) -> bool {
        self.generate_comments
    }

    /// Enable or disable wrapper function generation
    pub fn set_generate_wrappers(&mut self, enable: bool) -> &mut Self {
        self.generate_wrappers = enable;
        self
    }

    /// Check if wrappers should be generated
    pub fn generate_wrappers(&self) -> bool {
        self.generate_wrappers
    }

    /// Set prefix for generated items
    pub fn set_prefix(&mut self, prefix: &str) -> &mut Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    /// Get the prefix
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    /// Set suffix for generated items
    pub fn set_suffix(&mut self, suffix: &str) -> &mut Self {
        self.suffix = Some(suffix.to_string());
        self
    }

    /// Get the suffix
    pub fn suffix(&self) -> Option<&str> {
        self.suffix.as_deref()
    }

    /// Add a type to the allowlist
    pub fn allowlist_type(&mut self, type_name: &str) -> &mut Self {
        self.allowlist.push(type_name.to_string());
        self
    }

    /// Add a type to the blocklist
    pub fn blocklist_type(&mut self, type_name: &str) -> &mut Self {
        self.blocklist.push(type_name.to_string());
        self
    }

    /// Check if a type is allowed
    pub fn is_type_allowed(&self, type_name: &str) -> bool {
        // If blocklist contains the type, it's not allowed
        if self.blocklist.contains(&type_name.to_string()) {
            return false;
        }

        // If allowlist is empty, all types are allowed
        if self.allowlist.is_empty() {
            return true;
        }

        // Otherwise, type must be in allowlist
        self.allowlist.contains(&type_name.to_string())
    }

    /// Get all type mappings
    pub fn type_mappings(&self) -> &HashMap<String, String> {
        &self.type_mappings
    }
}

impl Default for BindgenConfig {
    fn default() -> Self {
        let mut type_mappings = HashMap::new();

        // Common type mappings
        type_mappings.insert("uint8_t".to_string(), "u8".to_string());
        type_mappings.insert("uint16_t".to_string(), "u16".to_string());
        type_mappings.insert("uint32_t".to_string(), "u32".to_string());
        type_mappings.insert("uint64_t".to_string(), "u64".to_string());
        type_mappings.insert("int8_t".to_string(), "i8".to_string());
        type_mappings.insert("int16_t".to_string(), "i16".to_string());
        type_mappings.insert("int32_t".to_string(), "i32".to_string());
        type_mappings.insert("int64_t".to_string(), "i64".to_string());
        type_mappings.insert("size_t".to_string(), "usize".to_string());
        type_mappings.insert("ssize_t".to_string(), "isize".to_string());
        type_mappings.insert("ptrdiff_t".to_string(), "isize".to_string());
        type_mappings.insert("intptr_t".to_string(), "isize".to_string());
        type_mappings.insert("uintptr_t".to_string(), "usize".to_string());

        Self {
            library_name: None,
            type_mappings,
            generate_comments: true,
            generate_wrappers: false,
            prefix: None,
            suffix: None,
            allowlist: Vec::new(),
            blocklist: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BindgenConfig::default();
        assert!(config.library_name.is_none());
        assert!(config.generate_comments);
        assert!(!config.generate_wrappers);
    }

    #[test]
    fn test_set_library_name() {
        let mut config = BindgenConfig::default();
        config.set_library_name("mylib");
        assert_eq!(config.library_name(), Some("mylib"));
    }

    #[test]
    fn test_type_mapping() {
        let mut config = BindgenConfig::default();
        config.add_type_mapping("MyType", "i32");
        assert_eq!(config.get_type_mapping("MyType"), Some("i32"));
    }

    #[test]
    fn test_default_type_mappings() {
        let config = BindgenConfig::default();
        assert_eq!(config.get_type_mapping("uint8_t"), Some("u8"));
        assert_eq!(config.get_type_mapping("int32_t"), Some("i32"));
        assert_eq!(config.get_type_mapping("size_t"), Some("usize"));
    }

    #[test]
    fn test_allowlist() {
        let mut config = BindgenConfig::default();
        config.allowlist_type("MyType");
        assert!(config.is_type_allowed("MyType"));
        assert!(!config.is_type_allowed("OtherType"));
    }

    #[test]
    fn test_blocklist() {
        let mut config = BindgenConfig::default();
        config.blocklist_type("BadType");
        assert!(!config.is_type_allowed("BadType"));
        assert!(config.is_type_allowed("GoodType"));
    }

    #[test]
    fn test_prefix_suffix() {
        let mut config = BindgenConfig::default();
        config.set_prefix("ffi_");
        config.set_suffix("_raw");
        assert_eq!(config.prefix(), Some("ffi_"));
        assert_eq!(config.suffix(), Some("_raw"));
    }
}
