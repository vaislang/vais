//! Source Mapping
//!
//! Maps between source locations and instruction addresses using DWARF debug info.

use std::collections::HashMap;

use crate::error::{DapError, DapResult};

/// Source location mapping from DWARF debug info
#[derive(Debug, Default)]
pub struct SourceMap {
    /// Address to source location mapping
    addr_to_loc: HashMap<u64, SourceLocation>,
    /// Source file + line to addresses mapping
    loc_to_addrs: HashMap<(String, u64), Vec<u64>>,
    /// Source file contents (for source reference)
    source_contents: HashMap<String, String>,
    /// Source reference to path mapping
    source_refs: HashMap<i64, String>,
    /// Next source reference ID
    next_source_ref: i64,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: u64,
    pub column: u64,
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            addr_to_loc: HashMap::new(),
            loc_to_addrs: HashMap::new(),
            source_contents: HashMap::new(),
            source_refs: HashMap::new(),
            next_source_ref: 1,
        }
    }

    /// Load source mapping from a compiled binary with DWARF info
    pub async fn load_from_binary(&mut self, binary_path: &str) -> DapResult<()> {
        let data = std::fs::read(binary_path)
            .map_err(|e| DapError::DwarfParsing(format!("Failed to read binary: {}", e)))?;

        let object = object::File::parse(&*data)
            .map_err(|e| DapError::DwarfParsing(format!("Failed to parse object file: {}", e)))?;

        self.parse_dwarf(&object)?;

        Ok(())
    }

    fn parse_dwarf(&mut self, object: &object::File) -> DapResult<()> {
        use gimli::{EndianSlice, LittleEndian};
        use object::{Object, ObjectSection};

        let endian = LittleEndian;

        let load_section = |name| -> Result<&[u8], gimli::Error> {
            Ok(object
                .section_by_name(name)
                .map(|s| s.data().unwrap_or(&[]))
                .unwrap_or(&[]))
        };

        let dwarf = gimli::Dwarf::load(|id| -> Result<_, gimli::Error> {
            let data = load_section(id.name())?;
            Ok(EndianSlice::new(data, endian))
        })
        .map_err(|e| DapError::DwarfParsing(format!("Failed to load DWARF: {}", e)))?;

        // Parse line number program
        let mut iter = dwarf.units();
        while let Ok(Some(header)) = iter.next() {
            let unit = dwarf
                .unit(header)
                .map_err(|e| DapError::DwarfParsing(format!("Failed to parse unit: {}", e)))?;

            if let Some(line_program) = unit.line_program.clone() {
                self.parse_line_program(&dwarf, &unit, line_program)?;
            }
        }

        Ok(())
    }

    fn parse_line_program<R: gimli::Reader>(
        &mut self,
        dwarf: &gimli::Dwarf<R>,
        unit: &gimli::Unit<R>,
        program: gimli::IncompleteLineProgram<R, R::Offset>,
    ) -> DapResult<()> {
        let mut rows = program.rows();

        while let Ok(Some((header, row))) = rows.next_row() {
            if row.end_sequence() {
                continue;
            }

            let address = row.address();
            let line = row.line().map(|l| l.get()).unwrap_or(0);
            let column = match row.column() {
                gimli::ColumnType::LeftEdge => 1,
                gimli::ColumnType::Column(c) => c.get(),
            };

            // Get file path
            let file = if let Some(file_entry) = row.file(header) {
                let mut path_str = String::new();

                // Get directory
                if let Some(dir) = file_entry.directory(header) {
                    if let Ok(dir_str) = dwarf.attr_string(unit, dir) {
                        if let Ok(s) = dir_str.to_string() {
                            path_str.push_str(&s);
                            path_str.push('/');
                        }
                    }
                }

                // Get filename
                if let Ok(name) = dwarf.attr_string(unit, file_entry.path_name()) {
                    if let Ok(s) = name.to_string() {
                        path_str.push_str(&s);
                    }
                }

                path_str
            } else {
                continue;
            };

            if file.is_empty() || line == 0 {
                continue;
            }

            // Store mappings
            self.addr_to_loc.insert(
                address,
                SourceLocation {
                    file: file.clone(),
                    line,
                    column,
                },
            );

            self.loc_to_addrs
                .entry((file, line))
                .or_default()
                .push(address);
        }

        Ok(())
    }

    /// Get source file path for an address
    pub fn get_source_for_address(&self, address: u64) -> Option<String> {
        self.addr_to_loc.get(&address).map(|loc| loc.file.clone())
    }

    /// Get line and column for an address
    pub fn get_line_column(&self, address: u64) -> Option<(u64, u64)> {
        self.addr_to_loc
            .get(&address)
            .map(|loc| (loc.line, loc.column))
    }

    /// Get full source location for an address
    pub fn get_location(&self, address: u64) -> Option<&SourceLocation> {
        self.addr_to_loc.get(&address)
    }

    /// Find addresses for a source location
    pub fn get_addresses(&self, file: &str, line: u64) -> Option<&Vec<u64>> {
        self.loc_to_addrs.get(&(file.to_string(), line))
    }

    /// Find the nearest valid line for setting a breakpoint
    pub fn find_nearest_line(&self, file: &str, line: u64) -> Option<u64> {
        // First try exact match
        if self.loc_to_addrs.contains_key(&(file.to_string(), line)) {
            return Some(line);
        }

        // Search nearby lines (up to 10 lines forward)
        for offset in 1..=10 {
            if self
                .loc_to_addrs
                .contains_key(&(file.to_string(), line + offset))
            {
                return Some(line + offset);
            }
        }

        None
    }

    /// Register source content for a file
    pub fn register_source(&mut self, path: &str, content: String) -> i64 {
        let ref_id = self.next_source_ref;
        self.next_source_ref += 1;

        self.source_contents.insert(path.to_string(), content);
        self.source_refs.insert(ref_id, path.to_string());

        ref_id
    }

    /// Get source content by reference ID
    pub fn get_source_content(&self, source_ref: i64) -> Option<String> {
        let path = self.source_refs.get(&source_ref)?;
        self.source_contents.get(path).cloned()
    }

    /// Get source content by path
    pub fn get_source_content_by_path(&self, path: &str) -> Option<&String> {
        self.source_contents.get(path)
    }

    /// Load source file from disk
    pub fn load_source_file(&mut self, path: &str) -> DapResult<i64> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| DapError::SourceMapping(format!("Failed to read source: {}", e)))?;

        Ok(self.register_source(path, content))
    }

    /// Get all known source files
    pub fn get_source_files(&self) -> Vec<String> {
        self.loc_to_addrs
            .keys()
            .map(|(file, _)| file.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Check if we have debug info loaded
    pub fn is_loaded(&self) -> bool {
        !self.addr_to_loc.is_empty()
    }

    /// Get statistics about loaded debug info
    pub fn stats(&self) -> (usize, usize) {
        (self.addr_to_loc.len(), self.loc_to_addrs.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_content() {
        let mut map = SourceMap::new();

        let ref_id = map.register_source("/test/file.vais", "F main() { 42 }".to_string());

        let content = map.get_source_content(ref_id);
        assert!(content.is_some());
        assert!(content.unwrap().contains("main"));
    }

    #[test]
    fn test_manual_mapping() {
        let mut map = SourceMap::new();

        // Manually add some mappings for testing
        map.addr_to_loc.insert(
            0x1000,
            SourceLocation {
                file: "/test/main.vais".to_string(),
                line: 10,
                column: 1,
            },
        );

        map.loc_to_addrs
            .insert(("/test/main.vais".to_string(), 10), vec![0x1000]);

        assert_eq!(
            map.get_source_for_address(0x1000),
            Some("/test/main.vais".to_string())
        );
        assert_eq!(map.get_line_column(0x1000), Some((10, 1)));
        assert!(map.get_addresses("/test/main.vais", 10).is_some());
    }
}
