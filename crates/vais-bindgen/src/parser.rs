use crate::config::BindgenConfig;
use crate::{BindgenError, Result};
use regex::Regex;

/// Represents a C type
#[derive(Debug, Clone, PartialEq)]
pub enum CType {
    Void,
    Char,
    Short,
    Int,
    Long,
    LongLong,
    UChar,
    UShort,
    UInt,
    ULong,
    ULongLong,
    Float,
    Double,
    Bool,
    SizeT,
    Custom(String),
    Pointer(Box<CType>),
    ConstPointer(Box<CType>),
    Array(Box<CType>, Option<usize>),
}

impl CType {
    pub fn is_pointer(&self) -> bool {
        matches!(self, CType::Pointer(_) | CType::ConstPointer(_))
    }

    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            CType::Void
                | CType::Char
                | CType::Short
                | CType::Int
                | CType::Long
                | CType::LongLong
                | CType::UChar
                | CType::UShort
                | CType::UInt
                | CType::ULong
                | CType::ULongLong
                | CType::Float
                | CType::Double
                | CType::Bool
        )
    }
}

/// Represents a C function signature
#[derive(Debug, Clone)]
pub struct CFunction {
    pub name: String,
    pub return_type: CType,
    pub parameters: Vec<(String, CType)>,
    pub is_variadic: bool,
}

/// Represents a C struct field
#[derive(Debug, Clone)]
pub struct CField {
    pub name: String,
    pub field_type: CType,
}

/// Represents a C struct definition
#[derive(Debug, Clone)]
pub struct CStruct {
    pub name: String,
    pub fields: Vec<CField>,
    pub is_opaque: bool,
}

/// Represents a C typedef
#[derive(Debug, Clone)]
pub struct CTypedef {
    pub name: String,
    pub underlying_type: CType,
}

/// Represents a C enum
#[derive(Debug, Clone)]
pub struct CEnum {
    pub name: String,
    pub variants: Vec<(String, Option<i64>)>,
}

/// Represents any C declaration
#[derive(Debug, Clone)]
pub enum CDeclaration {
    Function(CFunction),
    Struct(CStruct),
    Typedef(CTypedef),
    Enum(CEnum),
}

/// Parser for C headers
pub struct Parser<'a> {
    config: &'a BindgenConfig,
}

impl<'a> Parser<'a> {
    pub fn new(config: &'a BindgenConfig) -> Self {
        Self { config }
    }

    pub fn parse(&self, content: &str) -> Result<Vec<CDeclaration>> {
        let mut declarations = Vec::new();
        let cleaned = self.preprocess(content);

        // Parse typedefs
        declarations.extend(self.parse_typedefs(&cleaned)?);

        // Parse structs
        declarations.extend(self.parse_structs(&cleaned)?);

        // Parse enums
        declarations.extend(self.parse_enums(&cleaned)?);

        // Parse functions
        declarations.extend(self.parse_functions(&cleaned)?);

        Ok(declarations)
    }

    fn preprocess(&self, content: &str) -> String {
        let mut result = String::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip preprocessor directives
            if line.starts_with('#') {
                continue;
            }

            // Skip single-line comments
            if line.starts_with("//") {
                continue;
            }

            result.push_str(line);
            result.push(' ');
        }

        // Remove multi-line comments
        let comment_re = Regex::new(r"/\*.*?\*/").unwrap();
        comment_re.replace_all(&result, "").to_string()
    }

    fn parse_typedefs(&self, content: &str) -> Result<Vec<CDeclaration>> {
        let mut declarations = Vec::new();
        let typedef_re = Regex::new(
            r"typedef\s+struct\s*\{([^}]*)\}\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*;"
        ).unwrap();

        for cap in typedef_re.captures_iter(content) {
            let name = cap[2].to_string();
            let fields_str = &cap[1];
            let fields = self.parse_struct_fields(fields_str)?;

            declarations.push(CDeclaration::Struct(CStruct {
                name: name.clone(),
                fields,
                is_opaque: false,
            }));
        }

        // Simple typedefs
        let simple_typedef_re = Regex::new(
            r"typedef\s+([a-zA-Z_][a-zA-Z0-9_*\s]+)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*;"
        ).unwrap();

        for cap in simple_typedef_re.captures_iter(content) {
            let type_str = cap[1].trim();
            let name = cap[2].to_string();

            if !type_str.contains("struct") && !type_str.contains('{') {
                if let Ok(underlying_type) = self.parse_type(type_str) {
                    declarations.push(CDeclaration::Typedef(CTypedef {
                        name,
                        underlying_type,
                    }));
                }
            }
        }

        Ok(declarations)
    }

    fn parse_structs(&self, content: &str) -> Result<Vec<CDeclaration>> {
        let mut declarations = Vec::new();
        let struct_re = Regex::new(
            r"struct\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\{([^}]*)\}\s*;"
        ).unwrap();

        for cap in struct_re.captures_iter(content) {
            let name = cap[1].to_string();
            let fields_str = &cap[2];
            let fields = self.parse_struct_fields(fields_str)?;

            declarations.push(CDeclaration::Struct(CStruct {
                name,
                fields,
                is_opaque: false,
            }));
        }

        // Opaque structs
        let opaque_re = Regex::new(r"struct\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*;").unwrap();
        for cap in opaque_re.captures_iter(content) {
            let name = cap[1].to_string();
            declarations.push(CDeclaration::Struct(CStruct {
                name,
                fields: Vec::new(),
                is_opaque: true,
            }));
        }

        Ok(declarations)
    }

    fn parse_struct_fields(&self, fields_str: &str) -> Result<Vec<CField>> {
        let mut fields = Vec::new();
        let field_re = Regex::new(r"([a-zA-Z_][a-zA-Z0-9_*\s]+)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*;").unwrap();

        for cap in field_re.captures_iter(fields_str) {
            let type_str = cap[1].trim();
            let name = cap[2].to_string();

            match self.parse_type(type_str) {
                Ok(field_type) => {
                    fields.push(CField { name, field_type });
                }
                Err(_) => continue,
            }
        }

        Ok(fields)
    }

    fn parse_enums(&self, content: &str) -> Result<Vec<CDeclaration>> {
        let mut declarations = Vec::new();
        let enum_re = Regex::new(
            r"enum\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\{([^}]*)\}\s*;"
        ).unwrap();

        for cap in enum_re.captures_iter(content) {
            let name = cap[1].to_string();
            let variants_str = &cap[2];
            let variants = self.parse_enum_variants(variants_str)?;

            declarations.push(CDeclaration::Enum(CEnum { name, variants }));
        }

        Ok(declarations)
    }

    fn parse_enum_variants(&self, variants_str: &str) -> Result<Vec<(String, Option<i64>)>> {
        let mut variants = Vec::new();
        let variant_re = Regex::new(r"([a-zA-Z_][a-zA-Z0-9_]*)\s*(?:=\s*(-?\d+))?").unwrap();

        for cap in variant_re.captures_iter(variants_str) {
            let name = cap[1].to_string();
            let value = cap.get(2).and_then(|m| m.as_str().parse().ok());
            variants.push((name, value));
        }

        Ok(variants)
    }

    fn parse_functions(&self, content: &str) -> Result<Vec<CDeclaration>> {
        let mut declarations = Vec::new();
        let func_re = Regex::new(
            r"([a-zA-Z_][a-zA-Z0-9_*\s]+)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*;"
        ).unwrap();

        for cap in func_re.captures_iter(content) {
            let return_type_str = cap[1].trim();
            let name = cap[2].to_string();
            let params_str = cap[3].trim();

            let return_type = self.parse_type(return_type_str)?;
            let parameters = self.parse_parameters(params_str)?;

            declarations.push(CDeclaration::Function(CFunction {
                name,
                return_type,
                parameters,
                is_variadic: params_str.contains("..."),
            }));
        }

        Ok(declarations)
    }

    fn parse_parameters(&self, params_str: &str) -> Result<Vec<(String, CType)>> {
        if params_str.trim().is_empty() || params_str.trim() == "void" {
            return Ok(Vec::new());
        }

        let mut parameters = Vec::new();
        let parts: Vec<&str> = params_str.split(',').collect();

        for (idx, part) in parts.iter().enumerate() {
            let part = part.trim();
            if part == "..." {
                continue;
            }

            let tokens: Vec<&str> = part.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }

            let name = if tokens.len() == 1 {
                format!("arg{}", idx)
            } else {
                tokens.last().unwrap().trim_start_matches('*').to_string()
            };

            let type_str = if tokens.len() == 1 {
                tokens[0]
            } else {
                &tokens[..tokens.len() - 1].join(" ")
            };

            match self.parse_type(type_str) {
                Ok(param_type) => {
                    parameters.push((name, param_type));
                }
                Err(_) => continue,
            }
        }

        Ok(parameters)
    }

    fn parse_type(&self, type_str: &str) -> Result<CType> {
        let type_str = type_str.trim();

        // Check for pointer
        if type_str.ends_with('*') {
            let base_type_str = type_str.trim_end_matches('*').trim();
            let is_const = base_type_str.starts_with("const");
            let base_type_str = base_type_str.trim_start_matches("const").trim();

            let base_type = self.parse_type(base_type_str)?;
            return Ok(if is_const {
                CType::ConstPointer(Box::new(base_type))
            } else {
                CType::Pointer(Box::new(base_type))
            });
        }

        // Remove const qualifier
        let type_str = type_str.trim_start_matches("const").trim();

        // Remove struct/enum keywords
        let type_str = type_str.trim_start_matches("struct").trim();
        let type_str = type_str.trim_start_matches("enum").trim();

        // Check custom type mappings
        if let Some(mapped) = self.config.get_type_mapping(type_str) {
            return Ok(CType::Custom(mapped.to_string()));
        }

        // Parse primitive types
        match type_str {
            "void" => Ok(CType::Void),
            "char" => Ok(CType::Char),
            "short" => Ok(CType::Short),
            "int" => Ok(CType::Int),
            "long" => Ok(CType::Long),
            "long long" => Ok(CType::LongLong),
            "unsigned char" => Ok(CType::UChar),
            "unsigned short" => Ok(CType::UShort),
            "unsigned int" | "unsigned" => Ok(CType::UInt),
            "unsigned long" => Ok(CType::ULong),
            "unsigned long long" => Ok(CType::ULongLong),
            "float" => Ok(CType::Float),
            "double" => Ok(CType::Double),
            "_Bool" | "bool" => Ok(CType::Bool),
            "size_t" => Ok(CType::SizeT),
            _ => {
                // Custom type
                if type_str.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    Ok(CType::Custom(type_str.to_string()))
                } else {
                    Err(BindgenError::ParseError(format!(
                        "Unknown type: {}",
                        type_str
                    )))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let header = "int add(int a, int b);";
        let config = BindgenConfig::default();
        let parser = Parser::new(&config);
        let decls = parser.parse(header).unwrap();

        assert_eq!(decls.len(), 1);
        match &decls[0] {
            CDeclaration::Function(func) => {
                assert_eq!(func.name, "add");
                assert_eq!(func.return_type, CType::Int);
                assert_eq!(func.parameters.len(), 2);
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parse_void_function() {
        let header = "void print_hello(void);";
        let config = BindgenConfig::default();
        let parser = Parser::new(&config);
        let decls = parser.parse(header).unwrap();

        assert_eq!(decls.len(), 1);
        match &decls[0] {
            CDeclaration::Function(func) => {
                assert_eq!(func.name, "print_hello");
                assert_eq!(func.return_type, CType::Void);
                assert_eq!(func.parameters.len(), 0);
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parse_pointer_function() {
        let header = "char* get_string(void);";
        let config = BindgenConfig::default();
        let parser = Parser::new(&config);
        let decls = parser.parse(header).unwrap();

        match &decls[0] {
            CDeclaration::Function(func) => {
                assert!(func.return_type.is_pointer());
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parse_struct() {
        let header = r#"
            typedef struct {
                int x;
                int y;
            } Point;
        "#;
        let config = BindgenConfig::default();
        let parser = Parser::new(&config);
        let decls = parser.parse(header).unwrap();

        assert_eq!(decls.len(), 1);
        match &decls[0] {
            CDeclaration::Struct(s) => {
                assert_eq!(s.name, "Point");
                assert_eq!(s.fields.len(), 2);
                assert!(!s.is_opaque);
            }
            _ => panic!("Expected struct declaration"),
        }
    }

    #[test]
    fn test_parse_opaque_struct() {
        let header = "struct OpaqueHandle;";
        let config = BindgenConfig::default();
        let parser = Parser::new(&config);
        let decls = parser.parse(header).unwrap();

        match &decls[0] {
            CDeclaration::Struct(s) => {
                assert_eq!(s.name, "OpaqueHandle");
                assert!(s.is_opaque);
            }
            _ => panic!("Expected struct declaration"),
        }
    }

    #[test]
    fn test_parse_enum() {
        let header = r#"
            enum Status {
                OK = 0,
                ERROR = 1
            };
        "#;
        let config = BindgenConfig::default();
        let parser = Parser::new(&config);
        let decls = parser.parse(header).unwrap();

        assert_eq!(decls.len(), 1);
        match &decls[0] {
            CDeclaration::Enum(e) => {
                assert_eq!(e.name, "Status");
                assert_eq!(e.variants.len(), 2);
            }
            _ => panic!("Expected enum declaration"),
        }
    }

    #[test]
    fn test_parse_typedef() {
        let header = "typedef unsigned long size_t;";
        let config = BindgenConfig::default();
        let parser = Parser::new(&config);
        let decls = parser.parse(header).unwrap();

        assert!(!decls.is_empty());
    }
}
