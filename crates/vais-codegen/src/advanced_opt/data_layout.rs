//! Cache-friendly Data Layout Optimization
//!
//! This module implements data layout optimizations for better cache performance:
//!
//! - **Structure field reordering**: Minimize padding and improve cache locality
//! - **Padding optimization**: Reduce struct size while maintaining alignment
//! - **AoS to SoA transformation**: Convert Array of Structures to Structure of Arrays
//! - **Cache line alignment**: Align hot data to cache line boundaries
//! - **Hot/cold field separation**: Split structures based on access patterns

use std::collections::HashMap;

/// Cache line size (typically 64 bytes on modern CPUs)
pub const DEFAULT_CACHE_LINE_SIZE: usize = 64;

/// Information about a struct field
#[derive(Debug, Clone)]
pub struct FieldInfo {
    /// Field name
    pub name: String,
    /// Field type (LLVM IR type string)
    pub ty: String,
    /// Size in bytes
    pub size: usize,
    /// Required alignment in bytes
    pub alignment: usize,
    /// Offset within struct (after layout optimization)
    pub offset: usize,
    /// Estimated access frequency (0.0 = never, 1.0 = always)
    pub access_frequency: f64,
    /// Whether this field is considered "hot"
    pub is_hot: bool,
}

impl FieldInfo {
    /// Create a new field info
    pub fn new(name: impl Into<String>, ty: impl Into<String>) -> Self {
        let ty = ty.into();
        let (size, alignment) = type_size_align(&ty);
        Self {
            name: name.into(),
            ty,
            size,
            alignment,
            offset: 0,
            access_frequency: 0.5, // Default to moderate access
            is_hot: false,
        }
    }

    /// Set access frequency
    pub fn with_frequency(mut self, freq: f64) -> Self {
        self.access_frequency = freq;
        self.is_hot = freq > 0.7;
        self
    }
}

/// Struct layout information
#[derive(Debug, Clone)]
pub struct StructLayout {
    /// Struct name
    pub name: String,
    /// Fields with their layout info
    pub fields: Vec<FieldInfo>,
    /// Total size in bytes
    pub total_size: usize,
    /// Required alignment
    pub alignment: usize,
    /// Padding bytes
    pub padding: usize,
    /// Whether this struct is cache-line aligned
    pub cache_aligned: bool,
}

impl StructLayout {
    /// Create a new struct layout
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
            total_size: 0,
            alignment: 1,
            padding: 0,
            cache_aligned: false,
        }
    }

    /// Add a field
    pub fn add_field(&mut self, field: FieldInfo) {
        self.fields.push(field);
    }

    /// Calculate layout with given field order
    pub fn calculate_layout(&mut self) {
        let mut offset = 0usize;
        let mut max_align = 1usize;
        let mut total_padding = 0usize;

        for field in &mut self.fields {
            // Align offset to field alignment
            let aligned_offset = align_to(offset, field.alignment);
            total_padding += aligned_offset - offset;
            field.offset = aligned_offset;

            offset = aligned_offset + field.size;
            max_align = max_align.max(field.alignment);
        }

        // Final struct padding for alignment
        let final_size = align_to(offset, max_align);
        total_padding += final_size - offset;

        self.total_size = final_size;
        self.alignment = max_align;
        self.padding = total_padding;
        self.cache_aligned = self.total_size >= DEFAULT_CACHE_LINE_SIZE;
    }

    /// Get space efficiency (1.0 = no padding)
    pub fn efficiency(&self) -> f64 {
        if self.total_size == 0 {
            return 1.0;
        }
        let data_size: usize = self.fields.iter().map(|f| f.size).sum();
        data_size as f64 / self.total_size as f64
    }
}

/// Layout suggestion for optimization
#[derive(Debug, Clone)]
pub enum LayoutSuggestion {
    /// Reorder fields to reduce padding
    ReorderFields {
        struct_name: String,
        new_order: Vec<String>,
        size_before: usize,
        size_after: usize,
        padding_saved: usize,
    },
    /// Align struct to cache line
    CacheLineAlign {
        struct_name: String,
        current_align: usize,
        suggested_align: usize,
    },
    /// Split hot and cold fields
    SplitHotCold {
        struct_name: String,
        hot_fields: Vec<String>,
        cold_fields: Vec<String>,
    },
    /// Convert AoS to SoA
    AosToSoa {
        struct_name: String,
        array_name: String,
        soa_struct_name: String,
        soa_fields: Vec<(String, String)>, // (field_name, array_type)
    },
    /// Add padding for cache alignment
    AddPadding {
        struct_name: String,
        field_after: String,
        padding_bytes: usize,
    },
}

impl LayoutSuggestion {
    /// Generate LLVM IR comment for this suggestion
    pub fn to_comment(&self) -> String {
        match self {
            LayoutSuggestion::ReorderFields {
                struct_name,
                new_order,
                size_before,
                size_after,
                padding_saved,
            } => {
                format!(
                    "; LAYOUT SUGGESTION: Reorder {} fields to {:?}\n; Size: {} -> {} bytes (saves {} bytes padding)",
                    struct_name, new_order, size_before, size_after, padding_saved
                )
            }
            LayoutSuggestion::CacheLineAlign {
                struct_name,
                current_align,
                suggested_align,
            } => {
                format!(
                    "; LAYOUT SUGGESTION: Align {} to {} bytes (currently {} bytes)",
                    struct_name, suggested_align, current_align
                )
            }
            LayoutSuggestion::SplitHotCold {
                struct_name,
                hot_fields,
                cold_fields,
            } => {
                format!(
                    "; LAYOUT SUGGESTION: Split {} into hot ({:?}) and cold ({:?}) structs",
                    struct_name, hot_fields, cold_fields
                )
            }
            LayoutSuggestion::AosToSoa {
                struct_name,
                array_name,
                soa_struct_name,
                soa_fields,
            } => {
                format!(
                    "; LAYOUT SUGGESTION: Convert {}[] ({}) to SoA {}: {:?}",
                    struct_name, array_name, soa_struct_name, soa_fields
                )
            }
            LayoutSuggestion::AddPadding {
                struct_name,
                field_after,
                padding_bytes,
            } => {
                format!(
                    "; LAYOUT SUGGESTION: Add {} bytes padding after {}.{}",
                    padding_bytes, struct_name, field_after
                )
            }
        }
    }
}

/// Data layout optimizer
#[derive(Debug)]
pub struct DataLayoutOptimizer {
    /// Known struct layouts
    pub structs: HashMap<String, StructLayout>,
    /// Generated suggestions
    pub suggestions: Vec<LayoutSuggestion>,
    /// Cache line size
    pub cache_line_size: usize,
    /// Field access patterns (struct.field -> access count)
    pub access_patterns: HashMap<String, usize>,
}

impl Default for DataLayoutOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl DataLayoutOptimizer {
    /// Create a new data layout optimizer
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
            suggestions: Vec::new(),
            cache_line_size: DEFAULT_CACHE_LINE_SIZE,
            access_patterns: HashMap::new(),
        }
    }

    /// Set cache line size
    pub fn with_cache_line_size(mut self, size: usize) -> Self {
        self.cache_line_size = size;
        self
    }

    /// Analyze IR for struct definitions and usage
    pub fn analyze(&mut self, ir: &str) {
        self.parse_struct_definitions(ir);
        self.analyze_access_patterns(ir);
        self.generate_suggestions();
    }

    /// Parse struct type definitions from IR
    fn parse_struct_definitions(&mut self, ir: &str) {
        for line in ir.lines() {
            let trimmed = line.trim();

            // Pattern: %StructName = type { field1, field2, ... }
            if trimmed.starts_with('%') && trimmed.contains(" = type {") {
                if let Some((name, fields)) = parse_struct_type(trimmed) {
                    let mut layout = StructLayout::new(name.clone());

                    for (idx, field_ty) in fields.iter().enumerate() {
                        let field = FieldInfo::new(format!("field{}", idx), field_ty);
                        layout.add_field(field);
                    }

                    layout.calculate_layout();
                    self.structs.insert(name, layout);
                }
            }
        }
    }

    /// Analyze field access patterns
    fn analyze_access_patterns(&mut self, ir: &str) {
        for line in ir.lines() {
            let trimmed = line.trim();

            // Look for GEP instructions accessing struct fields
            // Pattern: getelementptr %StructType, %StructType* %ptr, i32 0, i32 N
            if trimmed.contains("getelementptr") {
                if let Some((struct_name, field_idx)) = parse_gep_struct_access(trimmed) {
                    let key = format!("{}.field{}", struct_name, field_idx);
                    *self.access_patterns.entry(key).or_insert(0) += 1;
                }
            }
        }

        // Update field access frequencies in struct layouts
        for (key, count) in &self.access_patterns {
            let parts: Vec<&str> = key.split('.').collect();
            if parts.len() == 2 {
                let struct_name = parts[0];
                let field_name = parts[1];

                if let Some(layout) = self.structs.get_mut(struct_name) {
                    // Find total accesses for this struct
                    let total: usize = self
                        .access_patterns
                        .iter()
                        .filter(|(k, _)| k.starts_with(&format!("{}.", struct_name)))
                        .map(|(_, c)| c)
                        .sum();

                    if total > 0 {
                        for field in &mut layout.fields {
                            if field.name == field_name {
                                field.access_frequency = *count as f64 / total as f64;
                                field.is_hot = field.access_frequency > 0.3;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Generate optimization suggestions
    fn generate_suggestions(&mut self) {
        for (name, layout) in &self.structs {
            // Check if field reordering would help
            if let Some(suggestion) = self.suggest_field_reorder(name, layout) {
                self.suggestions.push(suggestion);
            }

            // Check if cache alignment would help
            if layout.total_size >= self.cache_line_size / 2
                && layout.alignment < self.cache_line_size
            {
                self.suggestions.push(LayoutSuggestion::CacheLineAlign {
                    struct_name: name.clone(),
                    current_align: layout.alignment,
                    suggested_align: self.cache_line_size,
                });
            }

            // Check for hot/cold field separation
            if let Some(suggestion) = self.suggest_hot_cold_split(name, layout) {
                self.suggestions.push(suggestion);
            }
        }
    }

    /// Suggest field reordering to reduce padding
    fn suggest_field_reorder(&self, name: &str, layout: &StructLayout) -> Option<LayoutSuggestion> {
        if layout.fields.len() < 2 {
            return None;
        }

        // Create optimized field order: sort by alignment descending
        let mut optimized = layout.clone();
        optimized
            .fields
            .sort_by(|a, b| b.alignment.cmp(&a.alignment));
        optimized.calculate_layout();

        if optimized.padding < layout.padding {
            let new_order: Vec<String> = optimized.fields.iter().map(|f| f.name.clone()).collect();
            Some(LayoutSuggestion::ReorderFields {
                struct_name: name.to_string(),
                new_order,
                size_before: layout.total_size,
                size_after: optimized.total_size,
                padding_saved: layout.padding - optimized.padding,
            })
        } else {
            None
        }
    }

    /// Suggest hot/cold field separation
    fn suggest_hot_cold_split(
        &self,
        name: &str,
        layout: &StructLayout,
    ) -> Option<LayoutSuggestion> {
        let hot_fields: Vec<String> = layout
            .fields
            .iter()
            .filter(|f| f.is_hot)
            .map(|f| f.name.clone())
            .collect();

        let cold_fields: Vec<String> = layout
            .fields
            .iter()
            .filter(|f| !f.is_hot)
            .map(|f| f.name.clone())
            .collect();

        // Only suggest split if there's a meaningful separation
        if !hot_fields.is_empty() && !cold_fields.is_empty() && layout.fields.len() >= 4 {
            Some(LayoutSuggestion::SplitHotCold {
                struct_name: name.to_string(),
                hot_fields,
                cold_fields,
            })
        } else {
            None
        }
    }

    /// Generate optimized struct definitions
    pub fn generate_optimized_ir(&self, ir: &str) -> String {
        let mut result = String::new();

        // Add suggestions as comments at the start
        for suggestion in &self.suggestions {
            result.push_str(&suggestion.to_comment());
            result.push('\n');
        }
        result.push('\n');

        // Copy the original IR
        result.push_str(ir);

        result
    }
}

/// Optimize struct layout in IR
pub fn optimize_struct_layout(ir: &str) -> String {
    let mut optimizer = DataLayoutOptimizer::new();
    optimizer.analyze(ir);
    optimizer.generate_optimized_ir(ir)
}

/// Suggest AoS to SoA transformation
pub fn suggest_aos_to_soa(
    struct_name: &str,
    array_name: &str,
    fields: &[(String, String)],
) -> LayoutSuggestion {
    let soa_fields: Vec<(String, String)> = fields
        .iter()
        .map(|(name, ty)| {
            (name.clone(), format!("{}*", ty)) // Array of each field
        })
        .collect();

    LayoutSuggestion::AosToSoa {
        struct_name: struct_name.to_string(),
        array_name: array_name.to_string(),
        soa_struct_name: format!("{}_SoA", struct_name),
        soa_fields,
    }
}

/// Suggest optimal field ordering for a struct to minimize padding.
///
/// Sorts fields by alignment (descending), so larger fields come first,
/// naturally reducing internal padding.
pub fn suggest_field_reorder(fields: &[(String, String)]) -> Vec<(String, String)> {
    let mut fields_with_align: Vec<(String, String, usize)> = fields
        .iter()
        .map(|(name, ty)| {
            let (_, align) = type_size_align(ty);
            (name.clone(), ty.clone(), align)
        })
        .collect();

    // Sort by alignment descending (stable sort to preserve order among same-alignment fields)
    fields_with_align.sort_by(|a, b| b.2.cmp(&a.2));

    fields_with_align
        .into_iter()
        .map(|(name, ty, _)| (name, ty))
        .collect()
}

/// Calculate the padding savings from reordering struct fields.
pub fn padding_savings(fields: &[(String, String)]) -> (usize, usize) {
    // Original layout
    let original_padding = calculate_padding(fields);

    // Optimized layout
    let optimized = suggest_field_reorder(fields);
    let optimized_padding = calculate_padding(&optimized);

    (original_padding, optimized_padding)
}

/// Calculate total padding for a given field order.
fn calculate_padding(fields: &[(String, String)]) -> usize {
    let mut offset = 0usize;
    let mut total_data = 0usize;

    for (_, ty) in fields {
        let (size, align) = type_size_align(ty);
        offset = align_to(offset, align);
        offset += size;
        total_data += size;
    }

    // Final alignment to largest field
    let max_align = fields
        .iter()
        .map(|(_, ty)| type_size_align(ty).1)
        .max()
        .unwrap_or(1);
    offset = align_to(offset, max_align);

    offset - total_data
}

// Helper functions

fn align_to(offset: usize, alignment: usize) -> usize {
    (offset + alignment - 1) & !(alignment - 1)
}

fn type_size_align(ty: &str) -> (usize, usize) {
    match ty {
        "i1" => (1, 1),
        "i8" => (1, 1),
        "i16" => (2, 2),
        "i32" | "float" => (4, 4),
        "i64" | "double" => (8, 8),
        "i128" => (16, 16),
        _ if ty.ends_with('*') => (8, 8), // Pointer
        _ if ty.starts_with('[') => {
            // Array: [N x T]
            if let Some((count, elem_ty)) = parse_array_type(ty) {
                let (elem_size, elem_align) = type_size_align(&elem_ty);
                (count * elem_size, elem_align)
            } else {
                (8, 8)
            }
        }
        _ if ty.starts_with('<') => {
            // Vector: <N x T>
            if let Some((count, elem_ty)) = parse_vector_type(ty) {
                let (elem_size, _) = type_size_align(&elem_ty);
                let total = count * elem_size;
                (total, total) // Vectors are aligned to their size
            } else {
                (16, 16)
            }
        }
        _ => (8, 8), // Default to pointer size
    }
}

fn parse_array_type(ty: &str) -> Option<(usize, String)> {
    // [N x T]
    let inner = ty.strip_prefix('[')?.strip_suffix(']')?;
    let parts: Vec<&str> = inner.split(" x ").collect();
    if parts.len() == 2 {
        let count: usize = parts[0].parse().ok()?;
        let elem_ty = parts[1].to_string();
        Some((count, elem_ty))
    } else {
        None
    }
}

fn parse_vector_type(ty: &str) -> Option<(usize, String)> {
    // <N x T>
    let inner = ty.strip_prefix('<')?.strip_suffix('>')?;
    let parts: Vec<&str> = inner.split(" x ").collect();
    if parts.len() == 2 {
        let count: usize = parts[0].parse().ok()?;
        let elem_ty = parts[1].to_string();
        Some((count, elem_ty))
    } else {
        None
    }
}

fn parse_struct_type(line: &str) -> Option<(String, Vec<String>)> {
    // %StructName = type { field1, field2, ... }
    let name_end = line.find(" = type {")?;
    let name = line[1..name_end].trim().to_string();

    let fields_start = line.find('{')? + 1;
    let fields_end = line.rfind('}')?;
    let fields_str = &line[fields_start..fields_end];

    let fields: Vec<String> = fields_str
        .split(',')
        .map(|f| f.trim().to_string())
        .filter(|f| !f.is_empty())
        .collect();

    Some((name, fields))
}

fn parse_gep_struct_access(line: &str) -> Option<(String, usize)> {
    // getelementptr %StructType, %StructType* %ptr, i32 0, i32 N
    if !line.contains("getelementptr") {
        return None;
    }

    // Extract struct type
    let type_start = line.find('%')?;
    let type_end = line[type_start..]
        .find([',', '*'])
        .map(|i| type_start + i)?;
    let struct_name = line[type_start + 1..type_end].to_string();

    // Find field index (last i32 constant)
    let parts: Vec<&str> = line.split("i32 ").collect();
    if parts.len() >= 3 {
        let last_idx_str = parts.last()?;
        let idx_end = last_idx_str
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(last_idx_str.len());
        let field_idx: usize = last_idx_str[..idx_end].parse().ok()?;
        return Some((struct_name, field_idx));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_info() {
        let field = FieldInfo::new("x", "i64");
        assert_eq!(field.size, 8);
        assert_eq!(field.alignment, 8);
    }

    #[test]
    fn test_struct_layout() {
        let mut layout = StructLayout::new("Point");
        layout.add_field(FieldInfo::new("x", "i64"));
        layout.add_field(FieldInfo::new("y", "i64"));
        layout.calculate_layout();

        assert_eq!(layout.total_size, 16);
        assert_eq!(layout.padding, 0);
        assert_eq!(layout.efficiency(), 1.0);
    }

    #[test]
    fn test_struct_layout_with_padding() {
        let mut layout = StructLayout::new("Mixed");
        layout.add_field(FieldInfo::new("a", "i8")); // 1 byte
        layout.add_field(FieldInfo::new("b", "i64")); // 8 bytes
        layout.add_field(FieldInfo::new("c", "i8")); // 1 byte
        layout.calculate_layout();

        // a: offset 0, size 1
        // (7 bytes padding)
        // b: offset 8, size 8
        // c: offset 16, size 1
        // (7 bytes padding for alignment)
        // Total: 24 bytes, 14 bytes padding
        assert_eq!(layout.total_size, 24);
        assert_eq!(layout.padding, 14);
    }

    #[test]
    fn test_optimized_layout() {
        let mut layout = StructLayout::new("Mixed");
        layout.add_field(FieldInfo::new("b", "i64")); // 8 bytes first
        layout.add_field(FieldInfo::new("a", "i8")); // 1 byte
        layout.add_field(FieldInfo::new("c", "i8")); // 1 byte
        layout.calculate_layout();

        // b: offset 0, size 8
        // a: offset 8, size 1
        // c: offset 9, size 1
        // (6 bytes padding for alignment)
        // Total: 16 bytes, 6 bytes padding
        assert_eq!(layout.total_size, 16);
        assert_eq!(layout.padding, 6);
    }

    #[test]
    fn test_type_size_align() {
        assert_eq!(type_size_align("i8"), (1, 1));
        assert_eq!(type_size_align("i32"), (4, 4));
        assert_eq!(type_size_align("i64"), (8, 8));
        assert_eq!(type_size_align("double"), (8, 8));
        assert_eq!(type_size_align("i64*"), (8, 8));
        assert_eq!(type_size_align("[4 x i32]"), (16, 4));
        assert_eq!(type_size_align("<4 x float>"), (16, 16));
    }

    #[test]
    fn test_align_to() {
        assert_eq!(align_to(0, 8), 0);
        assert_eq!(align_to(1, 8), 8);
        assert_eq!(align_to(7, 8), 8);
        assert_eq!(align_to(8, 8), 8);
        assert_eq!(align_to(9, 8), 16);
    }

    #[test]
    fn test_parse_struct_type() {
        let line = "%Point = type { i64, i64 }";
        let (name, fields) = parse_struct_type(line).unwrap();
        assert_eq!(name, "Point");
        assert_eq!(fields, vec!["i64", "i64"]);
    }

    #[test]
    fn test_aos_to_soa_suggestion() {
        let suggestion = suggest_aos_to_soa(
            "Particle",
            "particles",
            &[
                ("x".to_string(), "f32".to_string()),
                ("y".to_string(), "f32".to_string()),
                ("mass".to_string(), "f32".to_string()),
            ],
        );

        if let LayoutSuggestion::AosToSoa {
            soa_struct_name,
            soa_fields,
            ..
        } = suggestion
        {
            assert_eq!(soa_struct_name, "Particle_SoA");
            assert_eq!(soa_fields.len(), 3);
        } else {
            panic!("Expected AosToSoa suggestion");
        }
    }

    #[test]
    fn test_data_layout_optimizer() {
        let ir = r#"
%Point = type { i64, i64 }
%Mixed = type { i8, i64, i8 }

define void @test() {
entry:
  %p = alloca %Point
  %px = getelementptr %Point, %Point* %p, i32 0, i32 0
  %py = getelementptr %Point, %Point* %p, i32 0, i32 1
  ret void
}
"#;

        let mut optimizer = DataLayoutOptimizer::new();
        optimizer.analyze(ir);

        assert!(optimizer.structs.contains_key("Point"));
        assert!(optimizer.structs.contains_key("Mixed"));

        // Mixed should have a reorder suggestion
        let has_reorder = optimizer.suggestions.iter().any(|s| {
            matches!(s, LayoutSuggestion::ReorderFields { struct_name, .. } if struct_name == "Mixed")
        });
        assert!(has_reorder);
    }

    #[test]
    fn test_suggest_field_reorder() {
        let fields = vec![
            ("a".to_string(), "i8".to_string()),
            ("b".to_string(), "i64".to_string()),
            ("c".to_string(), "i32".to_string()),
            ("d".to_string(), "i8".to_string()),
        ];
        let reordered = suggest_field_reorder(&fields);
        // i64 should come first, then i32, then i8s
        assert_eq!(reordered[0].1, "i64");
        assert_eq!(reordered[1].1, "i32");
    }

    #[test]
    fn test_padding_savings() {
        let fields = vec![
            ("a".to_string(), "i8".to_string()),
            ("b".to_string(), "i64".to_string()),
            ("c".to_string(), "i8".to_string()),
        ];
        let (original, optimized) = padding_savings(&fields);
        assert!(
            original > optimized,
            "Reordering should reduce padding: {} > {}",
            original,
            optimized
        );
    }
}
