use vais_ast::*;
use vais_lexer::Token;

use crate::{ParseResult, Parser};

impl Parser {
    /// Parse trait definition: `W Name { methods }`
    pub(super) fn parse_trait(&mut self, is_pub: bool) -> ParseResult<Trait> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;

        // Parse super traits: `W Iterator: Iterable + Clone`
        let super_traits = if self.check(&Token::Colon) {
            self.advance();
            self.parse_trait_bounds()?
        } else {
            Vec::new()
        };

        // Parse where clause
        let where_clause = self.parse_where_clause()?;

        let lbrace_span = self.current_span();
        self.expect(&Token::LBrace)?;

        let mut methods = Vec::new();
        let mut associated_types = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Check for associated type: `T Item` or `T Item: Trait`
            if self.check(&Token::TypeKeyword) {
                self.advance();
                associated_types.push(self.parse_associated_type()?);
            } else {
                methods.push(self.parse_trait_method()?);
            }
        }

        self.expect_closing(&Token::RBrace, lbrace_span)?;

        Ok(Trait {
            name,
            generics,
            super_traits,
            associated_types,
            methods,
            is_pub,
            where_clause,
        })
    }

    /// Parse associated type: `T Item` or `T Item: Trait` or `T Item = DefaultType`
    fn parse_associated_type(&mut self) -> ParseResult<AssociatedType> {
        let name = self.parse_ident()?;

        // GAT: Optional generic parameters (e.g., `T Item<'a, B: Clone>`)
        let generics = self.parse_generics()?;

        // Optional trait bounds
        let bounds = if self.check(&Token::Colon) {
            self.advance();
            self.parse_trait_bounds()?
        } else {
            Vec::new()
        };

        // Optional default type
        let default = if self.check(&Token::Eq) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        Ok(AssociatedType {
            name,
            generics,
            bounds,
            default,
        })
    }

    /// Parse trait method signature
    fn parse_trait_method(&mut self) -> ParseResult<TraitMethod> {
        // Check for const keyword: `C F method_name()` (const trait method)
        let is_const = if self.check(&Token::Const) {
            self.advance();
            true
        } else {
            false
        };

        // Check for async keyword: `A F method_name()`
        let is_async = if self.check(&Token::Async) {
            self.advance();
            true
        } else {
            false
        };

        self.expect(&Token::Function)?;
        let name = self.parse_ident()?;

        let lparen_span = self.current_span();
        self.expect(&Token::LParen)?;
        let params = self.parse_params()?;
        self.expect_closing(&Token::RParen, lparen_span)?;

        let ret_type = if self.check(&Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Optional default implementation
        let default_body = if self.check(&Token::Eq) {
            self.advance();
            Some(FunctionBody::Expr(Box::new(self.parse_expr()?)))
        } else if self.check(&Token::LBrace) {
            let lbrace_span = self.current_span();
            self.advance();
            let stmts = self.parse_block_contents()?;
            self.expect_closing(&Token::RBrace, lbrace_span)?;
            Some(FunctionBody::Block(stmts))
        } else {
            None
        };

        Ok(TraitMethod {
            name,
            params,
            ret_type,
            default_body,
            is_async,
            is_const,
        })
    }

    /// Parse impl block: `X Type: Trait { methods }`
    pub(crate) fn parse_impl(&mut self) -> ParseResult<Impl> {
        let generics = self.parse_generics()?;
        let target_type = self.parse_type()?;

        // Optional trait name
        let trait_name = if self.check(&Token::Colon) {
            self.advance();
            Some(self.parse_ident()?)
        } else {
            None
        };

        let lbrace_span = self.current_span();
        self.expect(&Token::LBrace)?;

        let mut methods = Vec::new();
        let mut associated_types = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Check for associated type: `T Item = SomeType`
            if self.check(&Token::TypeKeyword) {
                self.advance();
                associated_types.push(self.parse_associated_type_impl()?);
            } else {
                let start = self.current_span().start;
                let method_attrs = self.parse_attributes()?;
                self.expect(&Token::Function)?;
                let func = self.parse_function(false, false, method_attrs)?;
                let end = self.prev_span().end;
                methods.push(Spanned::new(func, Span::new(start, end)));
            }
        }

        self.expect_closing(&Token::RBrace, lbrace_span)?;

        Ok(Impl {
            target_type,
            trait_name,
            generics,
            associated_types,
            methods,
        })
    }

    /// Parse associated type implementation: `T Item = SomeType`
    fn parse_associated_type_impl(&mut self) -> ParseResult<AssociatedTypeImpl> {
        let name = self.parse_ident()?;
        self.expect(&Token::Eq)?;
        let ty = self.parse_type()?;
        Ok(AssociatedTypeImpl { name, ty })
    }
}
