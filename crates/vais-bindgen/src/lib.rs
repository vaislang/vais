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

/// Language mode for bindgen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageMode {
    C,
    Cpp,
}

/// Main bindgen structure for generating Vais FFI bindings from C/C++ headers
pub struct Bindgen {
    config: BindgenConfig,
    declarations: Vec<CDeclaration>,
    mode: LanguageMode,
}

impl Bindgen {
    /// Create a new bindgen instance with default configuration (C mode)
    pub fn new() -> Self {
        Self {
            config: BindgenConfig::default(),
            declarations: Vec::new(),
            mode: LanguageMode::C,
        }
    }

    /// Create a new bindgen instance for C++ headers
    pub fn new_cpp() -> Self {
        Self {
            config: BindgenConfig::default(),
            declarations: Vec::new(),
            mode: LanguageMode::Cpp,
        }
    }

    /// Create a bindgen instance with custom configuration
    pub fn with_config(config: BindgenConfig) -> Self {
        Self {
            config,
            declarations: Vec::new(),
            mode: LanguageMode::C,
        }
    }

    /// Create a bindgen instance with custom configuration for C++
    pub fn with_config_cpp(config: BindgenConfig) -> Self {
        Self {
            config,
            declarations: Vec::new(),
            mode: LanguageMode::Cpp,
        }
    }

    /// Set the language mode
    pub fn set_mode(&mut self, mode: LanguageMode) -> &mut Self {
        self.mode = mode;
        self
    }

    /// Parse a C header file
    pub fn header<P: AsRef<Path>>(&mut self, path: P) -> Result<&mut Self> {
        let content = std::fs::read_to_string(path)?;
        self.parse_header(&content)?;
        Ok(self)
    }

    /// Parse C/C++ header content from a string
    pub fn parse_header(&mut self, content: &str) -> Result<&mut Self> {
        let parser = match self.mode {
            LanguageMode::C => Parser::new(&self.config),
            LanguageMode::Cpp => Parser::new_cpp(&self.config),
        };
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
        let generator = match self.mode {
            LanguageMode::C => Generator::new(&self.config),
            LanguageMode::Cpp => Generator::new_cpp(&self.config),
        };
        generator.generate(&self.declarations)
    }

    /// Generate C wrapper header for C++ code
    pub fn generate_cpp_wrapper_header(&self) -> Result<String> {
        if self.mode != LanguageMode::Cpp {
            return Err(BindgenError::GenerationError(
                "C wrapper header generation is only available in C++ mode".to_string(),
            ));
        }
        let generator = Generator::new_cpp(&self.config);
        generator.generate_cpp_wrapper_header(&self.declarations)
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

    #[test]
    fn test_bindgen_cpp_class() {
        let header = r#"
            class Calculator {
            public:
                int add(int a, int b);
                int subtract(int a, int b);
            };
        "#;

        let mut bindgen = Bindgen::new_cpp();
        bindgen.parse_header(header).unwrap();
        let result = bindgen.generate().unwrap();

        assert!(result.contains("Calculator"));
        assert!(result.contains("CalculatorHandle"));
    }

    #[test]
    fn test_bindgen_cpp_namespace() {
        let header = r#"
            namespace Math {
                int square(int x);
            }
        "#;

        let mut bindgen = Bindgen::new_cpp();
        bindgen.parse_header(header).unwrap();
        let result = bindgen.generate().unwrap();

        assert!(result.contains("Math"));
        assert!(result.contains("Math_square"));
    }

    #[test]
    fn test_bindgen_cpp_template() {
        let header = r#"
            template<typename T>
            class Container {
            public:
                void add(T item);
                T get(int index);
            };
        "#;

        let mut bindgen = Bindgen::new_cpp();
        bindgen.parse_header(header).unwrap();
        let result = bindgen.generate().unwrap();

        assert!(result.contains("Container"));
    }

    #[test]
    fn test_bindgen_cpp_wrapper_header() {
        let header = r#"
            class MyClass {
            public:
                int getValue();
                void setValue(int v);
            };
        "#;

        let mut bindgen = Bindgen::new_cpp();
        bindgen.parse_header(header).unwrap();
        let wrapper = bindgen.generate_cpp_wrapper_header().unwrap();

        assert!(wrapper.contains("MyClassHandle"));
        assert!(wrapper.contains("MyClass_new"));
        assert!(wrapper.contains("MyClass_delete"));
        assert!(wrapper.contains("MyClass_getValue"));
        assert!(wrapper.contains("MyClass_setValue"));
    }
}
