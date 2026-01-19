//! LLVM Debug Information Generator for Vais
//!
//! Generates DWARF debug metadata for source-level debugging support.
//! This module creates DIFile, DISubprogram, DILocalVariable, and other
//! debug information nodes that enable debuggers like LLDB and GDB to
//! map machine code back to source locations.

use std::collections::HashMap;

/// Configuration for debug info generation
#[derive(Debug, Clone)]
pub struct DebugConfig {
    /// Source file path
    pub source_file: String,
    /// Source directory
    pub source_dir: String,
    /// Producer string (compiler identification)
    pub producer: String,
    /// Enable debug info generation
    pub enabled: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            source_file: "<vais>".to_string(),
            source_dir: ".".to_string(),
            producer: "vaisc 0.1.0".to_string(),
            enabled: false,
        }
    }
}

impl DebugConfig {
    pub fn new(source_file: &str, source_dir: &str) -> Self {
        Self {
            source_file: source_file.to_string(),
            source_dir: source_dir.to_string(),
            producer: "vaisc 0.1.0".to_string(),
            enabled: true,
        }
    }
}

/// Debug information builder for LLVM IR
pub struct DebugInfoBuilder {
    /// Debug configuration
    config: DebugConfig,
    /// Metadata ID counter
    metadata_counter: usize,
    /// Source code for line number calculation
    source_code: Option<String>,
    /// Cached line start positions
    line_starts: Vec<usize>,
    /// DIFile metadata ID
    di_file_id: Option<usize>,
    /// DICompileUnit metadata ID
    di_compile_unit_id: Option<usize>,
    /// Function debug info: function name -> DISubprogram ID
    function_di: HashMap<String, usize>,
    /// Variable debug info for current scope
    current_scope_di: Option<usize>,
    /// All generated metadata nodes
    metadata_nodes: Vec<String>,
    /// Named metadata for !llvm.dbg.cu
    named_metadata: Vec<String>,
}

impl DebugInfoBuilder {
    pub fn new(config: DebugConfig) -> Self {
        Self {
            config,
            metadata_counter: 0,
            source_code: None,
            line_starts: Vec::new(),
            di_file_id: None,
            di_compile_unit_id: None,
            function_di: HashMap::new(),
            current_scope_di: None,
            metadata_nodes: Vec::new(),
            named_metadata: Vec::new(),
        }
    }

    /// Check if debug info generation is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Set the source code for line number calculation
    pub fn set_source_code(&mut self, source: &str) {
        self.source_code = Some(source.to_string());
        self.line_starts = self.compute_line_starts(source);
    }

    /// Compute line start positions from source code
    fn compute_line_starts(&self, source: &str) -> Vec<usize> {
        let mut starts = vec![0];
        for (i, c) in source.char_indices() {
            if c == '\n' {
                starts.push(i + 1);
            }
        }
        starts
    }

    /// Convert byte offset to line number (1-indexed)
    pub fn offset_to_line(&self, offset: usize) -> usize {
        if self.line_starts.is_empty() {
            return 1;
        }
        match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx + 1,
            Err(idx) => idx,
        }
    }

    /// Convert byte offset to column number (1-indexed)
    pub fn offset_to_column(&self, offset: usize) -> usize {
        if self.line_starts.is_empty() {
            return 1;
        }
        let line = self.offset_to_line(offset);
        if line == 0 || line > self.line_starts.len() {
            return 1;
        }
        let line_start = self.line_starts[line - 1];
        offset - line_start + 1
    }

    /// Allocate a new metadata ID
    fn next_metadata_id(&mut self) -> usize {
        let id = self.metadata_counter;
        self.metadata_counter += 1;
        id
    }

    /// Initialize debug info: create DIFile and DICompileUnit
    pub fn initialize(&mut self) {
        if !self.config.enabled {
            return;
        }

        // Create DIFile
        let file_id = self.next_metadata_id();
        self.di_file_id = Some(file_id);
        self.metadata_nodes.push(format!(
            "!{} = !DIFile(filename: \"{}\", directory: \"{}\")",
            file_id, self.config.source_file, self.config.source_dir
        ));

        // Create DICompileUnit
        let cu_id = self.next_metadata_id();
        self.di_compile_unit_id = Some(cu_id);
        self.metadata_nodes.push(format!(
            "!{} = distinct !DICompileUnit(language: DW_LANG_C99, file: !{}, producer: \"{}\", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug, splitDebugInlining: false)",
            cu_id, file_id, self.config.producer
        ));

        // Create llvm.dbg.cu named metadata
        self.named_metadata.push(format!("!llvm.dbg.cu = !{{!{}}}", cu_id));

        // Add debug info version
        let version_id = self.next_metadata_id();
        self.metadata_nodes.push(format!("!{} = !{{i32 2, !\"Debug Info Version\", i32 3}}", version_id));
        self.named_metadata.push(format!("!llvm.module.flags = !{{!{}}}", version_id));
    }

    /// Create DISubprogram for a function
    pub fn create_function_debug_info(
        &mut self,
        func_name: &str,
        line: usize,
        is_definition: bool,
    ) -> Option<usize> {
        if !self.config.enabled {
            return None;
        }

        let file_id = self.di_file_id?;
        let cu_id = self.di_compile_unit_id?;

        // Create subroutine type (void for now, simplified)
        let type_id = self.next_metadata_id();
        self.metadata_nodes.push(format!(
            "!{} = !DISubroutineType(types: !{{}})",
            type_id
        ));

        // Create DISubprogram
        let sp_id = self.next_metadata_id();
        let sp_flags = if is_definition { "DISPFlagDefinition" } else { "DISPFlagLocalToUnit" };
        self.metadata_nodes.push(format!(
            "!{} = distinct !DISubprogram(name: \"{}\", scope: !{}, file: !{}, line: {}, type: !{}, scopeLine: {}, spFlags: {}, unit: !{}, retainedNodes: !{{}})",
            sp_id, func_name, file_id, file_id, line, type_id, line, sp_flags, cu_id
        ));

        self.function_di.insert(func_name.to_string(), sp_id);
        self.current_scope_di = Some(sp_id);

        Some(sp_id)
    }

    /// Get DISubprogram ID for a function
    pub fn get_function_debug_info(&self, func_name: &str) -> Option<usize> {
        self.function_di.get(func_name).copied()
    }

    /// Create DILocalVariable for a local variable
    pub fn create_local_variable_debug_info(
        &mut self,
        var_name: &str,
        line: usize,
        arg_no: Option<usize>,
    ) -> Option<usize> {
        if !self.config.enabled {
            return None;
        }

        let scope_id = self.current_scope_di?;
        let file_id = self.di_file_id?;

        let var_id = self.next_metadata_id();

        if let Some(arg) = arg_no {
            self.metadata_nodes.push(format!(
                "!{} = !DILocalVariable(name: \"{}\", arg: {}, scope: !{}, file: !{}, line: {}, type: !{{}})",
                var_id, var_name, arg, scope_id, file_id, line
            ));
        } else {
            self.metadata_nodes.push(format!(
                "!{} = !DILocalVariable(name: \"{}\", scope: !{}, file: !{}, line: {}, type: !{{}})",
                var_id, var_name, scope_id, file_id, line
            ));
        }

        Some(var_id)
    }

    /// Create DILocation for a source location
    pub fn create_location(&mut self, line: usize, column: usize) -> Option<usize> {
        if !self.config.enabled {
            return None;
        }

        let scope_id = self.current_scope_di?;
        let loc_id = self.next_metadata_id();
        self.metadata_nodes.push(format!(
            "!{} = !DILocation(line: {}, column: {}, scope: !{})",
            loc_id, line, column, scope_id
        ));

        Some(loc_id)
    }

    /// Generate !dbg metadata reference for an instruction
    pub fn dbg_ref(&self, loc_id: usize) -> String {
        format!(", !dbg !{}", loc_id)
    }

    /// Generate !dbg metadata reference from source offset
    pub fn dbg_ref_from_offset(&mut self, offset: usize) -> String {
        if !self.config.enabled {
            return String::new();
        }

        let line = self.offset_to_line(offset);
        let column = self.offset_to_column(offset);

        if let Some(loc_id) = self.create_location(line, column) {
            self.dbg_ref(loc_id)
        } else {
            String::new()
        }
    }

    /// Set the current debug scope (e.g., when entering a function)
    pub fn set_current_scope(&mut self, func_name: &str) {
        if let Some(id) = self.function_di.get(func_name) {
            self.current_scope_di = Some(*id);
        }
    }

    /// Generate all debug metadata as IR string
    pub fn finalize(&self) -> String {
        if !self.config.enabled || self.metadata_nodes.is_empty() {
            return String::new();
        }

        let mut ir = String::new();
        ir.push_str("\n; Debug Information\n");

        // Named metadata first
        for named in &self.named_metadata {
            ir.push_str(named);
            ir.push('\n');
        }
        ir.push('\n');

        // All metadata nodes
        for node in &self.metadata_nodes {
            ir.push_str(node);
            ir.push('\n');
        }

        ir
    }

    /// Generate llvm.dbg.declare intrinsic call for a variable
    pub fn generate_dbg_declare(
        &mut self,
        alloca_reg: &str,
        var_name: &str,
        line: usize,
    ) -> String {
        if !self.config.enabled {
            return String::new();
        }

        let var_id = match self.create_local_variable_debug_info(var_name, line, None) {
            Some(id) => id,
            None => return String::new(),
        };

        let loc_id = match self.create_location(line, 1) {
            Some(id) => id,
            None => return String::new(),
        };

        format!(
            "  call void @llvm.dbg.declare(metadata ptr {}, metadata !{}, metadata !DIExpression()), !dbg !{}\n",
            alloca_reg, var_id, loc_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_calculation() {
        let source = "line1\nline2\nline3";
        let config = DebugConfig::new("test.vais", ".");
        let mut builder = DebugInfoBuilder::new(config);
        builder.set_source_code(source);

        assert_eq!(builder.offset_to_line(0), 1);  // 'l' of line1
        assert_eq!(builder.offset_to_line(5), 1);  // '\n' after line1
        assert_eq!(builder.offset_to_line(6), 2);  // 'l' of line2
        assert_eq!(builder.offset_to_line(12), 3); // 'l' of line3
    }

    #[test]
    fn test_column_calculation() {
        let source = "hello\nworld";
        let config = DebugConfig::new("test.vais", ".");
        let mut builder = DebugInfoBuilder::new(config);
        builder.set_source_code(source);

        assert_eq!(builder.offset_to_column(0), 1);  // 'h'
        assert_eq!(builder.offset_to_column(4), 5);  // 'o' in hello
        assert_eq!(builder.offset_to_column(6), 1);  // 'w' in world
        assert_eq!(builder.offset_to_column(10), 5); // 'd' in world
    }

    #[test]
    fn test_debug_info_disabled() {
        let config = DebugConfig::default();
        let builder = DebugInfoBuilder::new(config);
        assert!(!builder.is_enabled());
    }

    #[test]
    fn test_debug_info_enabled() {
        let config = DebugConfig::new("test.vais", "/path/to/dir");
        let builder = DebugInfoBuilder::new(config);
        assert!(builder.is_enabled());
    }
}
