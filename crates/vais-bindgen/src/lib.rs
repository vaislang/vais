pub mod config;
pub mod generator;
pub mod parser;

pub use config::BindgenConfig;
use generator::Generator;
use parser::{CDeclaration, Parser};
use std::io::{self, Write};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BindgenError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Generation error: {0}")]
    GenerationError(String),
}

pub type Result<T> = std::result::Result<T, BindgenError>;

/// Main bindgen structure for generating Vais FFI bindings from C headers
pub struct Bindgen {
    config: BindgenConfig,
    declarations: Vec<CDeclaration>,
}

impl Bindgen {
    /// Create a new bindgen instance with default configuration
    pub fn new() -> Self {
        Self {
            config: BindgenConfig::default(),
            declarations: Vec::new(),
        }
    }

    /// Create a bindgen instance with custom configuration
    pub fn with_config(config: BindgenConfig) -> Self {
        Self {
            config,
            declarations: Vec::new(),
        }
    }

    /// Parse a C header file
    pub fn header<P: AsRef<Path>>(&mut self, path: P) -> Result<&mut Self> {
        let content = std::fs::read_to_string(path)?;
        self.parse_header(&content)?;
        Ok(self)
    }

    /// Parse C header content from a string
    pub fn parse_header(&mut self, content: &str) -> Result<&mut Self> {
        let parser = Parser::new(&self.config);
        let decls = parser.parse(content)?;
        self.declarations.extend(decls);
        Ok(self)
    }

    /// Generate Vais FFI bindings to a file
    pub fn generate_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = self.generate()?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Generate Vais FFI bindings to a writer
    pub fn generate_to_writer<W: Write>(&self, writer: &mut W) -> Result<()> {
        let content = self.generate()?;
        writer.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Generate Vais FFI bindings as a string
    pub fn generate(&self) -> Result<String> {
        let generator = Generator::new(&self.config);
        generator.generate(&self.declarations)
    }

    /// Configure the bindgen instance
    pub fn configure(&mut self, f: impl FnOnce(&mut BindgenConfig)) -> &mut Self {
        f(&mut self.config);
        self
    }
}

impl Default for Bindgen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bindgen_basic() {
        let header = r#"
            int add(int a, int b);
            void print_hello(void);
        "#;

        let mut bindgen = Bindgen::new();
        bindgen.parse_header(header).unwrap();
        let result = bindgen.generate().unwrap();

        assert!(result.contains("fn add"));
        assert!(result.contains("fn print_hello"));
        assert!(result.contains("extern"));
    }

    #[test]
    fn test_bindgen_with_struct() {
        let header = r#"
            typedef struct {
                int x;
                int y;
            } Point;

            Point create_point(int x, int y);
        "#;

        let mut bindgen = Bindgen::new();
        bindgen.parse_header(header).unwrap();
        let result = bindgen.generate().unwrap();

        assert!(result.contains("struct Point"));
        assert!(result.contains("fn create_point"));
    }

    #[test]
    fn test_bindgen_with_config() {
        let header = "int test(void);";

        let mut config = BindgenConfig::default();
        config.set_library_name("mylib");

        let mut bindgen = Bindgen::with_config(config);
        bindgen.parse_header(header).unwrap();
        let result = bindgen.generate().unwrap();

        assert!(result.contains("mylib"));
    }

    #[test]
    fn test_bindgen_custom_type_mapping() {
        let header = "size_t get_size(void);";

        let mut bindgen = Bindgen::new();
        bindgen.configure(|config| {
            config.add_type_mapping("size_t", "u64");
        });
        bindgen.parse_header(header).unwrap();
        let result = bindgen.generate().unwrap();

        assert!(result.contains("u64"));
    }
}
