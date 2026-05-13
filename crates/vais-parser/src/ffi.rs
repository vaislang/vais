//! FFI (Foreign Function Interface) parsing support
//!
//! Parses extern blocks, function pointers, and variadic functions.

use crate::{ParseResult, Parser};
use vais_ast::*;
use vais_lexer::Token;

impl Parser {
    /// Parse extern block: `N "C" { declarations }`
    pub(crate) fn parse_extern_block(&mut self) -> ParseResult<ExternBlock> {
        // Parse ABI string (e.g., "C")
        let abi = if let Some(tok) = self.peek() {
            if let Token::String(s) = &tok.token {
                let abi = s.clone();
                self.advance();
                abi
            } else {
                "C".to_string() // Default to C ABI
            }
        } else {
            "C".to_string()
        };

        self.expect(&Token::LBrace)?;

        let mut functions = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            functions.push(self.parse_extern_function()?);
        }

        self.expect(&Token::RBrace)?;

        Ok(ExternBlock { abi, functions })
    }

    /// Parse extern function declaration: `F name(params) -> ret_type;`
    /// Supports attributes like `#\[wasm_import("env", "js_alert")\]` before the function keyword.
    fn parse_extern_function(&mut self) -> ParseResult<ExternFunction> {
        // Parse optional attributes before function keyword
        let attributes = self.parse_attributes()?;

        self.expect(&Token::Function)?;

        let name = self.parse_ident()?;

        self.expect(&Token::LParen)?;

        let (params, is_vararg) = self.parse_extern_params()?;

        self.expect(&Token::RParen)?;

        let ret_type = if self.check(&Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Expect semicolon at the end of extern function declaration
        if self.check(&Token::Semi) {
            self.advance();
        }

        Ok(ExternFunction {
            name,
            params,
            ret_type,
            is_vararg,
            attributes,
        })
    }

    /// Parse extern function parameters, detecting variadic (...) at the end
    fn parse_extern_params(&mut self) -> ParseResult<(Vec<Param>, bool)> {
        let mut params = Vec::new();
        let mut is_vararg = false;

        while !self.check(&Token::RParen) && !self.is_at_end() {
            // Check for variadic ... at the end
            if self.check(&Token::Ellipsis) {
                self.advance();
                is_vararg = true;
                break;
            }

            let _start = self.current_span().start;
            let name = self.parse_ident()?;
            self.expect(&Token::Colon)?;
            let ty = self.parse_type()?;
            let _end = self.prev_span().end;

            params.push(Param {
                name,
                ty,
                is_mut: false,
                is_vararg: false,
                ownership: Ownership::Regular,
                default_value: None,
            });

            if !self.check(&Token::RParen) {
                if self.check(&Token::Comma) {
                    self.advance();
                    // Check for trailing ... after comma
                    if self.check(&Token::Ellipsis) {
                        self.advance();
                        is_vararg = true;
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        Ok((params, is_vararg))
    }

    /// Parse function pointer type: `fn(A, B) -> C` or `fn(A, ...) -> C`
    pub(crate) fn parse_fn_ptr_type(&mut self) -> ParseResult<Type> {
        // We expect 'fn' keyword here (as identifier)
        self.expect(&Token::LParen)?;

        let mut params = Vec::new();
        let mut is_vararg = false;

        while !self.check(&Token::RParen) && !self.is_at_end() {
            if self.check(&Token::Ellipsis) {
                self.advance();
                is_vararg = true;
                break;
            }

            params.push(self.parse_type()?);

            if !self.check(&Token::RParen) {
                if self.check(&Token::Comma) {
                    self.advance();
                    if self.check(&Token::Ellipsis) {
                        self.advance();
                        is_vararg = true;
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        self.expect(&Token::RParen)?;

        let ret = if self.check(&Token::Arrow) {
            self.advance();
            Box::new(self.parse_type()?)
        } else {
            // Default to unit type if no return specified
            let span = self.current_span();
            Box::new(Spanned::new(Type::Unit, Span::new(span.start, span.end)))
        };

        Ok(Type::FnPtr {
            params,
            ret,
            is_vararg,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vais_lexer::tokenize;

    #[test]
    fn test_extern_block_simple() {
        let source = r#"N "C" { F puts(s: *i8) -> i32; }"#;
        let tokens = tokenize(source).unwrap();
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().unwrap();

        assert_eq!(module.items.len(), 1);
        let Item::ExternBlock(block) = &module.items[0].node else {
            panic!("Expected extern block, got {:?}", &module.items[0].node);
        };
        assert_eq!(block.abi, "C");
        assert_eq!(block.functions.len(), 1);
        assert_eq!(block.functions[0].name.node, "puts");
    }

    #[test]
    fn test_extern_block_vararg() {
        let source = r#"N "C" { F printf(fmt: *i8, ...) -> i32; }"#;
        let tokens = tokenize(source).unwrap();
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().unwrap();

        assert_eq!(module.items.len(), 1);
        let Item::ExternBlock(block) = &module.items[0].node else {
            panic!("Expected extern block, got {:?}", &module.items[0].node);
        };
        assert!(block.functions[0].is_vararg);
        assert_eq!(block.functions[0].params.len(), 1);
    }

    #[test]
    fn test_extern_block_multiple_functions() {
        let source = r#"
        N "C" {
            F malloc(size: i64) -> *i8;
            F free(ptr: *i8) -> ();
            F printf(fmt: *i8, ...) -> i32;
        }
        "#;
        let tokens = tokenize(source).unwrap();
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().unwrap();

        assert_eq!(module.items.len(), 1);
        let Item::ExternBlock(block) = &module.items[0].node else {
            panic!("Expected extern block, got {:?}", &module.items[0].node);
        };
        assert_eq!(block.functions.len(), 3);
        assert_eq!(block.functions[0].name.node, "malloc");
        assert_eq!(block.functions[1].name.node, "free");
        assert_eq!(block.functions[2].name.node, "printf");
        assert!(block.functions[2].is_vararg);
    }

    #[test]
    fn test_wasm_import_attribute_on_extern_function() {
        let source = r#"
        N "C" {
            #[wasm_import("env", "js_alert")]
            F alert(msg: *i8);
        }
        "#;
        let tokens = tokenize(source).unwrap();
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().unwrap();

        assert_eq!(module.items.len(), 1);
        let Item::ExternBlock(block) = &module.items[0].node else {
            panic!("Expected extern block, got {:?}", &module.items[0].node);
        };
        assert_eq!(block.functions.len(), 1);
        let func = &block.functions[0];
        assert_eq!(func.name.node, "alert");
        assert_eq!(func.attributes.len(), 1);
        assert_eq!(func.attributes[0].name, "wasm_import");
        assert_eq!(func.attributes[0].args, vec!["env", "js_alert"]);
    }

    #[test]
    fn test_wasm_export_attribute_on_function() {
        let source = r#"
        #[wasm_export("add")]
        F add(a: i64, b: i64) -> i64 = a + b
        "#;
        let tokens = tokenize(source).unwrap();
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().unwrap();

        assert_eq!(module.items.len(), 1);
        let Item::Function(f) = &module.items[0].node else {
            panic!("Expected function, got {:?}", &module.items[0].node);
        };
        assert_eq!(f.name.node, "add");
        assert_eq!(f.attributes.len(), 1);
        assert_eq!(f.attributes[0].name, "wasm_export");
        assert_eq!(f.attributes[0].args, vec!["add"]);
    }

    #[test]
    fn test_wasm_import_no_args() {
        // wasm_import with no args uses function name as import name
        let source = r#"
        N "C" {
            #[wasm_import]
            F console_log(msg: *i8);
        }
        "#;
        let tokens = tokenize(source).unwrap();
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().unwrap();

        let Item::ExternBlock(block) = &module.items[0].node else {
            panic!("Expected extern block, got {:?}", &module.items[0].node);
        };
        let func = &block.functions[0];
        assert_eq!(func.attributes.len(), 1);
        assert_eq!(func.attributes[0].name, "wasm_import");
        assert!(func.attributes[0].args.is_empty());
    }

    #[test]
    fn test_wasm_export_on_single_extern() {
        // X F syntax with wasm_export attribute
        let source = r#"
        #[wasm_export("greet")]
        X F greet(name: *i8) -> i64
        "#;
        let tokens = tokenize(source).unwrap();
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().unwrap();

        assert_eq!(module.items.len(), 1);
        let Item::ExternBlock(block) = &module.items[0].node else {
            panic!("Expected extern block, got {:?}", &module.items[0].node);
        };
        let func = &block.functions[0];
        assert_eq!(func.name.node, "greet");
        assert_eq!(func.attributes.len(), 1);
        assert_eq!(func.attributes[0].name, "wasm_export");
        assert_eq!(func.attributes[0].args, vec!["greet"]);
    }

    #[test]
    fn test_multiple_wasm_attributes() {
        let source = r#"
        N "C" {
            #[wasm_import("env", "fetch")]
            F js_fetch(url: *i8) -> i64;

            F malloc(size: i64) -> *i8;

            #[wasm_import("env", "setTimeout")]
            F js_set_timeout(callback: i64, ms: i64);
        }
        "#;
        let tokens = tokenize(source).unwrap();
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().unwrap();

        let Item::ExternBlock(block) = &module.items[0].node else {
            panic!("Expected extern block, got {:?}", &module.items[0].node);
        };
        assert_eq!(block.functions.len(), 3);
        // First has wasm_import
        assert_eq!(block.functions[0].attributes.len(), 1);
        assert_eq!(block.functions[0].attributes[0].name, "wasm_import");
        // Second has no attributes
        assert!(block.functions[1].attributes.is_empty());
        // Third has wasm_import
        assert_eq!(block.functions[2].attributes.len(), 1);
        assert_eq!(block.functions[2].attributes[0].name, "wasm_import");
        assert_eq!(
            block.functions[2].attributes[0].args,
            vec!["env", "setTimeout"]
        );
    }

    #[test]
    fn test_function_pointer_type() {
        let source = "F test(callback: fn(i64, i64) -> i64) -> i64 = callback(1, 2)";
        let tokens = tokenize(source).unwrap();
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().unwrap();

        assert_eq!(module.items.len(), 1);
        let Item::Function(f) = &module.items[0].node else {
            panic!("Expected function, got {:?}", &module.items[0].node);
        };
        assert_eq!(f.params.len(), 1);
        let Type::FnPtr {
            params, is_vararg, ..
        } = &f.params[0].ty.node
        else {
            panic!(
                "Expected function pointer type, got {:?}",
                &f.params[0].ty.node
            );
        };
        assert_eq!(params.len(), 2);
        assert!(!(*is_vararg));
    }

    #[test]
    fn test_function_pointer_vararg() {
        let source = "S Handler { callback: fn(i32, ...) -> i32 }";
        let tokens = tokenize(source).unwrap();
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().unwrap();

        assert_eq!(module.items.len(), 1);
        let Item::Struct(s) = &module.items[0].node else {
            panic!("Expected struct, got {:?}", &module.items[0].node);
        };
        assert_eq!(s.fields.len(), 1);
        let Type::FnPtr { is_vararg, .. } = &s.fields[0].ty.node else {
            panic!(
                "Expected function pointer type, got {:?}",
                &s.fields[0].ty.node
            );
        };
        assert!(*is_vararg);
    }
}
