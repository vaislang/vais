//! FFI (Foreign Function Interface) parsing support
//!
//! Parses extern blocks, function pointers, and variadic functions.

use crate::{ParseError, ParseResult, Parser};
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
    fn parse_extern_function(&mut self) -> ParseResult<ExternFunction> {
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

        Ok(Type::FnPtr { params, ret, is_vararg })
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
        match &module.items[0].node {
            Item::ExternBlock(block) => {
                assert_eq!(block.abi, "C");
                assert_eq!(block.functions.len(), 1);
                assert_eq!(block.functions[0].name.node, "puts");
            }
            _ => panic!("Expected extern block"),
        }
    }

    #[test]
    fn test_extern_block_vararg() {
        let source = r#"N "C" { F printf(fmt: *i8, ...) -> i32; }"#;
        let tokens = tokenize(source).unwrap();
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().unwrap();

        assert_eq!(module.items.len(), 1);
        match &module.items[0].node {
            Item::ExternBlock(block) => {
                assert_eq!(block.functions[0].is_vararg, true);
                assert_eq!(block.functions[0].params.len(), 1);
            }
            _ => panic!("Expected extern block"),
        }
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
        match &module.items[0].node {
            Item::ExternBlock(block) => {
                assert_eq!(block.functions.len(), 3);
                assert_eq!(block.functions[0].name.node, "malloc");
                assert_eq!(block.functions[1].name.node, "free");
                assert_eq!(block.functions[2].name.node, "printf");
                assert_eq!(block.functions[2].is_vararg, true);
            }
            _ => panic!("Expected extern block"),
        }
    }

    #[test]
    fn test_function_pointer_type() {
        let source = "F test(callback: fn(i64, i64) -> i64) -> i64 = callback(1, 2)";
        let tokens = tokenize(source).unwrap();
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().unwrap();

        assert_eq!(module.items.len(), 1);
        match &module.items[0].node {
            Item::Function(f) => {
                assert_eq!(f.params.len(), 1);
                match &f.params[0].ty.node {
                    Type::FnPtr { params, is_vararg, .. } => {
                        assert_eq!(params.len(), 2);
                        assert_eq!(*is_vararg, false);
                    }
                    _ => panic!("Expected function pointer type"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_function_pointer_vararg() {
        let source = "S Handler { callback: fn(i32, ...) -> i32 }";
        let tokens = tokenize(source).unwrap();
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module().unwrap();

        assert_eq!(module.items.len(), 1);
        match &module.items[0].node {
            Item::Struct(s) => {
                assert_eq!(s.fields.len(), 1);
                match &s.fields[0].ty.node {
                    Type::FnPtr { is_vararg, .. } => {
                        assert_eq!(*is_vararg, true);
                    }
                    _ => panic!("Expected function pointer type"),
                }
            }
            _ => panic!("Expected struct"),
        }
    }
}
