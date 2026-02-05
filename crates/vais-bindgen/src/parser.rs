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
    CppClass(CppClass),
    CppNamespace(CppNamespace),
}

/// Access specifier for C++ class members
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessSpecifier {
    Public,
    Protected,
    Private,
}

/// Represents a C++ method
#[derive(Debug, Clone)]
pub struct CppMethod {
    pub name: String,
    pub return_type: CType,
    pub parameters: Vec<(String, CType)>,
    pub is_virtual: bool,
    pub is_const: bool,
    pub is_static: bool,
    pub is_constructor: bool,
    pub is_destructor: bool,
    pub access: AccessSpecifier,
}

/// Represents a C++ class field
#[derive(Debug, Clone)]
pub struct CppClassField {
    pub name: String,
    pub field_type: CType,
    pub access: AccessSpecifier,
}

/// Represents a C++ class definition
#[derive(Debug, Clone)]
pub struct CppClass {
    pub name: String,
    pub base_classes: Vec<String>,
    pub methods: Vec<CppMethod>,
    pub fields: Vec<CppClassField>,
    pub is_template: bool,
    pub template_params: Vec<String>,
}

/// Represents a C++ namespace
#[derive(Debug, Clone)]
pub struct CppNamespace {
    pub name: String,
    pub items: Vec<CDeclaration>,
}

/// Parser mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserMode {
    C,
    Cpp,
}

/// Parser for C/C++ headers
pub struct Parser<'a> {
    config: &'a BindgenConfig,
    mode: ParserMode,
}

impl<'a> Parser<'a> {
    pub fn new(config: &'a BindgenConfig) -> Self {
        Self {
            config,
            mode: ParserMode::C,
        }
    }

    pub fn new_cpp(config: &'a BindgenConfig) -> Self {
        Self {
            config,
            mode: ParserMode::Cpp,
        }
    }

    pub fn parse(&self, content: &str) -> Result<Vec<CDeclaration>> {
        let mut declarations = Vec::new();
        let cleaned = self.preprocess(content);

        if self.mode == ParserMode::Cpp {
            // Parse C++ namespaces
            declarations.extend(self.parse_namespaces(&cleaned)?);

            // Parse C++ classes
            declarations.extend(self.parse_classes(&cleaned)?);
        }

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
        let typedef_re =
            Regex::new(r"typedef\s+struct\s*\{([^}]*)\}\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*;").unwrap();

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
        let simple_typedef_re =
            Regex::new(r"typedef\s+([a-zA-Z_][a-zA-Z0-9_*\s]+)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*;")
                .unwrap();

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
        let struct_re = Regex::new(r"struct\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\{([^}]*)\}\s*;").unwrap();

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
        let field_re =
            Regex::new(r"([a-zA-Z_][a-zA-Z0-9_*\s]+)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*;").unwrap();

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
        let enum_re = Regex::new(r"enum\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\{([^}]*)\}\s*;").unwrap();

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
        let func_re =
            Regex::new(r"([a-zA-Z_][a-zA-Z0-9_*\s]+)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*;")
                .unwrap();

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

        // Remove qualifiers and keywords
        let type_str = type_str.trim_start_matches("const").trim();
        let type_str = type_str.trim_start_matches("static").trim(); // Add static handling
        let type_str = type_str.trim_start_matches("virtual").trim(); // Add virtual handling

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

    // C++ specific parsing methods

    fn parse_namespaces(&self, content: &str) -> Result<Vec<CDeclaration>> {
        let mut declarations = Vec::new();
        let namespace_re =
            Regex::new(r"namespace\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\{([^}]*)\}").unwrap();

        for cap in namespace_re.captures_iter(content) {
            let name = cap[1].to_string();
            let namespace_content = &cap[2];

            // Recursively parse namespace content
            let items = self.parse(namespace_content)?;

            declarations.push(CDeclaration::CppNamespace(CppNamespace { name, items }));
        }

        Ok(declarations)
    }

    fn parse_classes(&self, content: &str) -> Result<Vec<CDeclaration>> {
        let mut declarations = Vec::new();

        // Parse class declarations with bodies
        let class_re = Regex::new(
            r"(?:template\s*<([^>]*)>\s*)?class\s+([a-zA-Z_][a-zA-Z0-9_]*)(?:\s*:\s*(?:public|private|protected)\s+([a-zA-Z_][a-zA-Z0-9_]*))?\s*\{([^}]*)\}\s*;"
        ).unwrap();

        for cap in class_re.captures_iter(content) {
            let template_params_str = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let name = cap[2].to_string();
            let base_class = cap.get(3).map(|m| m.as_str().to_string());
            let body = &cap[4];

            let is_template = !template_params_str.is_empty();
            let template_params = if is_template {
                self.parse_template_params(template_params_str)?
            } else {
                Vec::new()
            };

            let base_classes = if let Some(base) = base_class {
                vec![base]
            } else {
                Vec::new()
            };

            let (methods, fields) = self.parse_class_body(body)?;

            declarations.push(CDeclaration::CppClass(CppClass {
                name,
                base_classes,
                methods,
                fields,
                is_template,
                template_params,
            }));
        }

        Ok(declarations)
    }

    fn parse_template_params(&self, params_str: &str) -> Result<Vec<String>> {
        let mut params = Vec::new();
        let param_re = Regex::new(r"(?:typename|class)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();

        for cap in param_re.captures_iter(params_str) {
            params.push(cap[1].to_string());
        }

        Ok(params)
    }

    fn parse_class_body(&self, body: &str) -> Result<(Vec<CppMethod>, Vec<CppClassField>)> {
        let mut methods = Vec::new();
        let mut fields = Vec::new();

        // Split by access specifiers
        let sections: Vec<&str> = body.split("public:").collect();

        if sections.len() > 1 {
            // Parse sections after public:
            for section in sections.iter().skip(1) {
                let private_parts: Vec<&str> = section.split("private:").collect();

                if private_parts.len() > 1 {
                    // Parse public part
                    self.parse_class_section(
                        private_parts[0],
                        AccessSpecifier::Public,
                        &mut methods,
                        &mut fields,
                    )?;

                    // Parse private parts
                    for private_part in private_parts.iter().skip(1) {
                        self.parse_class_section(
                            private_part,
                            AccessSpecifier::Private,
                            &mut methods,
                            &mut fields,
                        )?;
                    }
                } else {
                    self.parse_class_section(
                        section,
                        AccessSpecifier::Public,
                        &mut methods,
                        &mut fields,
                    )?;
                }
            }
        }

        // Parse first section (before any access specifier)
        if !sections.is_empty() {
            self.parse_class_section(
                sections[0],
                AccessSpecifier::Private,
                &mut methods,
                &mut fields,
            )?;
        }

        Ok((methods, fields))
    }

    fn parse_class_section(
        &self,
        section: &str,
        access: AccessSpecifier,
        methods: &mut Vec<CppMethod>,
        fields: &mut Vec<CppClassField>,
    ) -> Result<()> {
        // Collect all declarations in one pass
        let mut cleaned = String::new();
        for line in section.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                cleaned.push_str(trimmed);
                cleaned.push(' ');
            }
        }

        // Parse methods with a more flexible regex
        // Matches: [static|virtual] type name(params)[const][= 0];
        // Use word boundaries and be careful with greedy matching
        let method_re = Regex::new(
            r"(?:(static|virtual)\s+)?([a-zA-Z_][a-zA-Z0-9_*]*(?:\s*\*)*)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*(const)?\s*(=\s*0)?\s*;"
        ).unwrap();

        for cap in method_re.captures_iter(&cleaned) {
            let modifier = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let return_type_str = cap[2].trim();
            let name = cap[3].to_string();
            let params_str = cap[4].trim();
            let const_modifier = cap.get(5).is_some();

            let is_virtual = modifier == "virtual";
            let is_static = modifier == "static";
            let is_const = const_modifier;

            let return_type = self.parse_type(return_type_str).unwrap_or(CType::Void);
            let parameters = self.parse_parameters(params_str)?;

            methods.push(CppMethod {
                name,
                return_type,
                parameters,
                is_virtual,
                is_const,
                is_static,
                is_constructor: false,
                is_destructor: false,
                access,
            });
        }

        // Parse fields - simpler pattern
        let field_re =
            Regex::new(r"([a-zA-Z_][a-zA-Z0-9_*]+)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*;").unwrap();

        for cap in field_re.captures_iter(&cleaned) {
            let type_str = cap[1].trim();
            let name = cap[2].to_string();

            // Skip if it's actually a method (would have been captured above)
            if cleaned.contains(&format!("{}(", name)) {
                continue;
            }

            if let Ok(field_type) = self.parse_type(type_str) {
                fields.push(CppClassField {
                    name,
                    field_type,
                    access,
                });
            }
        }

        Ok(())
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

    #[test]
    fn test_parse_cpp_class() {
        let header = r#"
            class MyClass {
            public:
                int getValue();
                void setValue(int v);
            private:
                int value;
            };
        "#;
        let config = BindgenConfig::default();
        let parser = Parser::new_cpp(&config);
        let decls = parser.parse(header).unwrap();

        assert!(!decls.is_empty());
        match &decls[0] {
            CDeclaration::CppClass(cls) => {
                assert_eq!(cls.name, "MyClass");
                assert!(!cls.methods.is_empty());
                assert!(!cls.fields.is_empty());
            }
            _ => panic!("Expected CppClass declaration"),
        }
    }

    #[test]
    fn test_parse_cpp_namespace() {
        let header = r#"
            namespace MyNamespace {
                int myFunction(int x);
            }
        "#;
        let config = BindgenConfig::default();
        let parser = Parser::new_cpp(&config);
        let decls = parser.parse(header).unwrap();

        assert!(!decls.is_empty());
        match &decls[0] {
            CDeclaration::CppNamespace(ns) => {
                assert_eq!(ns.name, "MyNamespace");
                assert!(!ns.items.is_empty());
            }
            _ => panic!("Expected CppNamespace declaration"),
        }
    }

    #[test]
    fn test_parse_template_class() {
        let header = r#"
            template<typename T>
            class Vector {
            public:
                void push(T item);
                T get(int index);
            };
        "#;
        let config = BindgenConfig::default();
        let parser = Parser::new_cpp(&config);
        let decls = parser.parse(header).unwrap();

        assert!(!decls.is_empty());
        match &decls[0] {
            CDeclaration::CppClass(cls) => {
                assert_eq!(cls.name, "Vector");
                assert!(cls.is_template);
                assert_eq!(cls.template_params.len(), 1);
                assert_eq!(cls.template_params[0], "T");
            }
            _ => panic!("Expected CppClass declaration"),
        }
    }
}
