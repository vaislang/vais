//! AOEL Parser
//!
//! Recursive descent parser for the AOEL language.

use aoel_lexer::{Lexer, Token, TokenKind, Span};
use aoel_ast::*;
use crate::error::{ParseError, ParseResult};

/// AOEL Parser
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    eof_token: Token,
}

impl Parser {
    /// Create a new parser for the given source code
    pub fn new(source: &str) -> Self {
        let lexer = Lexer::new(source);
        let tokens: Vec<Token> = lexer.into_iter().collect();

        let eof_span = tokens.last()
            .map(|t| Span::new(t.span.end, t.span.end))
            .unwrap_or(Span::new(0, 0));

        Self {
            tokens,
            current: 0,
            eof_token: Token {
                kind: TokenKind::Eof,
                span: eof_span,
                text: String::new(),
            },
        }
    }

    /// Parse the source code into a Unit AST
    pub fn parse(source: &str) -> ParseResult<Unit> {
        let mut parser = Parser::new(source);
        parser.parse_unit()
    }

    // =========================================================================
    // Token navigation
    // =========================================================================

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or(&self.eof_token)
    }

    fn peek_kind(&self) -> TokenKind {
        self.peek().kind.clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek_kind() == TokenKind::Eof
    }

    fn advance(&mut self) -> Token {
        let token = self.peek().clone();
        if !self.is_at_end() {
            self.current += 1;
        }
        token
    }

    fn previous(&self) -> Token {
        self.tokens.get(self.current.saturating_sub(1))
            .cloned()
            .unwrap_or_else(|| self.eof_token.clone())
    }

    fn check(&self, kind: &TokenKind) -> bool {
        &self.peek_kind() == kind
    }

    fn consume(&mut self, kind: TokenKind, expected: &str) -> ParseResult<Token> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: expected.to_string(),
                found: self.peek_kind(),
                span: self.peek().span,
            })
        }
    }

    fn skip_newlines(&mut self) {
        while self.check(&TokenKind::Newline) {
            self.advance();
        }
    }

    fn expect_newline(&mut self) -> ParseResult<()> {
        // Always skip newlines if present, but don't require them
        // This allows for more flexible formatting
        self.skip_newlines();
        Ok(())
    }

    // =========================================================================
    // Unit parsing
    // =========================================================================

    fn parse_unit(&mut self) -> ParseResult<Unit> {
        self.skip_newlines();

        let start_span = self.peek().span;

        // Parse UNIT header
        let header = self.parse_unit_header()?;
        self.expect_newline()?;

        // Parse all blocks
        let meta = self.parse_meta_block()?;
        let input = self.parse_input_block()?;
        let output = self.parse_output_block()?;
        let intent = self.parse_intent_block()?;
        let constraint = self.parse_constraint_block()?;
        let flow = self.parse_flow_block()?;
        let execution = self.parse_execution_block()?;
        let verify = self.parse_verify_block()?;

        // Parse END
        self.skip_newlines();
        let end_token = self.consume(TokenKind::End, "END")?;

        let span = Span::new(start_span.start, end_token.span.end);

        Ok(Unit {
            header,
            meta,
            input,
            output,
            intent,
            constraint,
            flow,
            execution,
            verify,
            span,
        })
    }

    fn parse_unit_header(&mut self) -> ParseResult<UnitHeader> {
        let start = self.peek().span;

        self.consume(TokenKind::Unit, "UNIT")?;

        // Parse unit kind
        let kind = match self.peek_kind() {
            TokenKind::Function => { self.advance(); UnitKind::Function }
            TokenKind::Service => { self.advance(); UnitKind::Service }
            TokenKind::Pipeline => { self.advance(); UnitKind::Pipeline }
            TokenKind::Module => { self.advance(); UnitKind::Module }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "FUNCTION, SERVICE, PIPELINE, or MODULE".to_string(),
                    found: self.peek_kind(),
                    span: self.peek().span,
                });
            }
        };

        // Parse qualified name
        let name = self.parse_qualified_name()?;

        // Parse optional version
        let version = if self.check(&TokenKind::Version) {
            let token = self.advance();
            Some(Version::parse(&token.text, token.span)
                .ok_or_else(|| ParseError::InvalidVersion {
                    version: token.text.clone(),
                    span: token.span,
                })?)
        } else {
            None
        };

        let span = Span::new(start.start, self.previous().span.end);

        Ok(UnitHeader { kind, name, version, span })
    }

    fn parse_qualified_name(&mut self) -> ParseResult<QualifiedName> {
        let start = self.peek().span;
        let mut parts = Vec::new();

        // First identifier
        let token = self.consume(TokenKind::Identifier, "identifier")?;
        parts.push(Ident {
            name: token.text.clone(),
            span: token.span,
        });

        // Additional parts after dots
        while self.check(&TokenKind::Dot) {
            self.advance();
            let token = self.consume(TokenKind::Identifier, "identifier")?;
            parts.push(Ident {
                name: token.text.clone(),
                span: token.span,
            });
        }

        let span = Span::new(start.start, self.previous().span.end);

        Ok(QualifiedName { parts, span })
    }

    // =========================================================================
    // META block
    // =========================================================================

    fn parse_meta_block(&mut self) -> ParseResult<MetaBlock> {
        self.skip_newlines();
        let start = self.peek().span;

        self.consume(TokenKind::Meta, "META")?;
        self.expect_newline()?;

        let mut entries = Vec::new();

        while !self.check(&TokenKind::EndMeta) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::EndMeta) {
                break;
            }
            entries.push(self.parse_meta_entry()?);
            self.expect_newline()?;
        }

        let end_token = self.consume(TokenKind::EndMeta, "ENDMETA")?;
        self.expect_newline()?;

        Ok(MetaBlock {
            entries,
            span: Span::new(start.start, end_token.span.end),
        })
    }

    fn parse_meta_entry(&mut self) -> ParseResult<MetaEntry> {
        let start = self.peek().span;

        let key = match self.peek_kind() {
            TokenKind::Domain => { self.advance(); MetaKey::Domain }
            TokenKind::Determinism => { self.advance(); MetaKey::Determinism }
            TokenKind::Idempotent => { self.advance(); MetaKey::Idempotent }
            TokenKind::Pure => { self.advance(); MetaKey::Pure }
            TokenKind::Timeout => { self.advance(); MetaKey::Timeout }
            TokenKind::Retry => { self.advance(); MetaKey::Retry }
            _ => {
                return Err(ParseError::InvalidMetaKey {
                    key: self.peek().text.clone(),
                    span: self.peek().span,
                });
            }
        };

        let value = self.parse_meta_value()?;

        Ok(MetaEntry {
            key,
            value,
            span: Span::new(start.start, self.previous().span.end),
        })
    }

    fn parse_meta_value(&mut self) -> ParseResult<MetaValue> {
        match self.peek_kind() {
            TokenKind::True => {
                self.advance();
                Ok(MetaValue::Bool(true))
            }
            TokenKind::False => {
                self.advance();
                Ok(MetaValue::Bool(false))
            }
            TokenKind::Integer => {
                let token = self.advance();
                let value = token.text.parse::<i64>()
                    .map_err(|_| ParseError::InvalidLiteral { span: token.span })?;
                Ok(MetaValue::Integer(value))
            }
            TokenKind::StringLiteral => {
                let token = self.advance();
                let value = token.text[1..token.text.len()-1].to_string();
                Ok(MetaValue::String(value))
            }
            TokenKind::Duration => {
                let token = self.advance();
                Ok(MetaValue::Duration(token.text.clone()))
            }
            TokenKind::Identifier => {
                // Handle qualified names like "examples.basic"
                let first = self.advance();
                let mut value = first.text.clone();

                while self.check(&TokenKind::Dot) {
                    self.advance();
                    if self.check(&TokenKind::Identifier) {
                        let next = self.advance();
                        value.push('.');
                        value.push_str(&next.text);
                    } else {
                        break;
                    }
                }

                Ok(MetaValue::String(value))
            }
            _ => Err(ParseError::InvalidLiteral { span: self.peek().span }),
        }
    }

    // =========================================================================
    // INPUT block
    // =========================================================================

    fn parse_input_block(&mut self) -> ParseResult<InputBlock> {
        self.skip_newlines();
        let start = self.peek().span;

        self.consume(TokenKind::Input, "INPUT")?;
        self.expect_newline()?;

        let mut fields = Vec::new();

        while !self.check(&TokenKind::EndInput) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::EndInput) {
                break;
            }
            fields.push(self.parse_input_field()?);
        }

        let end_token = self.consume(TokenKind::EndInput, "ENDINPUT")?;
        self.expect_newline()?;

        Ok(InputBlock {
            fields,
            span: Span::new(start.start, end_token.span.end),
        })
    }

    fn parse_input_field(&mut self) -> ParseResult<InputField> {
        let start = self.peek().span;

        let name_token = self.consume(TokenKind::Identifier, "field name")?;
        let name = Ident {
            name: name_token.text.clone(),
            span: name_token.span,
        };

        self.consume(TokenKind::Colon, ":")?;
        let ty = self.parse_type()?;

        // Parse optional constraints in brackets
        let mut constraints = Vec::new();
        if self.check(&TokenKind::LBracket) {
            self.advance();
            while !self.check(&TokenKind::RBracket) && !self.is_at_end() {
                let expr = self.parse_expression()?;
                constraints.push(FieldConstraint {
                    expr,
                    span: self.previous().span,
                });
                if self.check(&TokenKind::Comma) {
                    self.advance();
                }
            }
            self.consume(TokenKind::RBracket, "]")?;
        }

        self.expect_newline()?;

        Ok(InputField {
            name,
            ty,
            constraints,
            span: Span::new(start.start, self.previous().span.end),
        })
    }

    // =========================================================================
    // OUTPUT block
    // =========================================================================

    fn parse_output_block(&mut self) -> ParseResult<OutputBlock> {
        self.skip_newlines();
        let start = self.peek().span;

        self.consume(TokenKind::Output, "OUTPUT")?;
        self.expect_newline()?;

        let mut fields = Vec::new();

        while !self.check(&TokenKind::EndOutput) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::EndOutput) {
                break;
            }
            fields.push(self.parse_output_field()?);
        }

        let end_token = self.consume(TokenKind::EndOutput, "ENDOUTPUT")?;
        self.expect_newline()?;

        Ok(OutputBlock {
            fields,
            span: Span::new(start.start, end_token.span.end),
        })
    }

    fn parse_output_field(&mut self) -> ParseResult<OutputField> {
        let start = self.peek().span;

        let name_token = self.consume(TokenKind::Identifier, "field name")?;
        let name = Ident {
            name: name_token.text.clone(),
            span: name_token.span,
        };

        self.consume(TokenKind::Colon, ":")?;
        let ty = self.parse_type()?;

        // Parse optional constraints in brackets
        let mut constraints = Vec::new();
        if self.check(&TokenKind::LBracket) {
            self.advance();
            while !self.check(&TokenKind::RBracket) && !self.is_at_end() {
                let expr = self.parse_expression()?;
                constraints.push(FieldConstraint {
                    expr,
                    span: self.previous().span,
                });
                if self.check(&TokenKind::Comma) {
                    self.advance();
                }
            }
            self.consume(TokenKind::RBracket, "]")?;
        }

        self.expect_newline()?;

        Ok(OutputField {
            name,
            ty,
            constraints,
            span: Span::new(start.start, self.previous().span.end),
        })
    }

    // =========================================================================
    // INTENT block
    // =========================================================================

    fn parse_intent_block(&mut self) -> ParseResult<IntentBlock> {
        self.skip_newlines();
        let start = self.peek().span;

        self.consume(TokenKind::Intent, "INTENT")?;
        self.expect_newline()?;

        let mut goal_type = GoalType::Transform;
        let mut goal_spec = GoalSpec {
            inputs: Vec::new(),
            outputs: Vec::new(),
            span: start,
        };
        let mut priorities = Vec::new();
        let mut on_failure = None;

        while !self.check(&TokenKind::EndIntent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::EndIntent) {
                break;
            }

            match self.peek_kind() {
                TokenKind::Goal => {
                    self.advance();
                    goal_type = self.parse_goal_type()?;
                    self.consume(TokenKind::Colon, ":")?;
                    goal_spec = self.parse_goal_spec()?;
                }
                TokenKind::Priority => {
                    self.advance();
                    priorities = self.parse_priorities()?;
                }
                TokenKind::OnFailure => {
                    self.advance();
                    on_failure = Some(self.parse_failure_strategy()?);
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "GOAL, PRIORITY, or ON_FAILURE".to_string(),
                        found: self.peek_kind(),
                        span: self.peek().span,
                    });
                }
            }

            self.expect_newline()?;
        }

        let end_token = self.consume(TokenKind::EndIntent, "ENDINTENT")?;
        self.expect_newline()?;

        Ok(IntentBlock {
            goal_type,
            goal_spec,
            priorities,
            on_failure,
            span: Span::new(start.start, end_token.span.end),
        })
    }

    fn parse_goal_type(&mut self) -> ParseResult<GoalType> {
        match self.peek_kind() {
            TokenKind::Transform => { self.advance(); Ok(GoalType::Transform) }
            TokenKind::Validate => { self.advance(); Ok(GoalType::Validate) }
            TokenKind::Aggregate => { self.advance(); Ok(GoalType::Aggregate) }
            TokenKind::Filter => { self.advance(); Ok(GoalType::Filter) }
            TokenKind::Route => { self.advance(); Ok(GoalType::Route) }
            TokenKind::Compose => { self.advance(); Ok(GoalType::Compose) }
            TokenKind::Fetch => { self.advance(); Ok(GoalType::Fetch) }
            _ => Err(ParseError::InvalidGoalType {
                goal: self.peek().text.clone(),
                span: self.peek().span,
            }),
        }
    }

    fn parse_goal_spec(&mut self) -> ParseResult<GoalSpec> {
        let start = self.peek().span;
        let mut inputs = Vec::new();

        // Parse input list: (a, b, c) or a, b, c or just a single expression
        if self.check(&TokenKind::LParen) {
            self.advance();
            while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                inputs.push(self.parse_expression()?);
                if self.check(&TokenKind::Comma) {
                    self.advance();
                }
            }
            self.consume(TokenKind::RParen, ")")?;
        } else {
            // Single expression or comma-separated list without parentheses
            inputs.push(self.parse_expression()?);
            // Continue parsing if there are more comma-separated inputs
            while self.check(&TokenKind::Comma) {
                self.advance();
                // Check if next token might start an output (after ->)
                // We need to peek ahead to see if this is still an input
                if self.check(&TokenKind::Arrow) {
                    break;
                }
                inputs.push(self.parse_expression()?);
            }
        }

        // Parse arrow
        self.consume(TokenKind::Arrow, "->")?;

        // Parse output list: (x, y) or x, y or just a single expression
        let mut outputs = Vec::new();
        if self.check(&TokenKind::LParen) {
            self.advance();
            while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                outputs.push(self.parse_expression()?);
                if self.check(&TokenKind::Comma) {
                    self.advance();
                }
            }
            self.consume(TokenKind::RParen, ")")?;
        } else {
            // Single expression or comma-separated list without parentheses
            outputs.push(self.parse_expression()?);
            // Continue parsing if there are more comma-separated outputs
            while self.check(&TokenKind::Comma) {
                self.advance();
                outputs.push(self.parse_expression()?);
            }
        }

        Ok(GoalSpec {
            inputs,
            outputs,
            span: Span::new(start.start, self.previous().span.end),
        })
    }

    fn parse_priorities(&mut self) -> ParseResult<Vec<PriorityKind>> {
        let mut priorities = Vec::new();

        loop {
            let priority = match self.peek_kind() {
                TokenKind::Correctness => { self.advance(); PriorityKind::Correctness }
                TokenKind::Performance => { self.advance(); PriorityKind::Performance }
                TokenKind::Latency => { self.advance(); PriorityKind::Latency }
                TokenKind::Throughput => { self.advance(); PriorityKind::Throughput }
                TokenKind::Memory => { self.advance(); PriorityKind::Memory }
                _ => break,
            };
            priorities.push(priority);

            // Accept comma, '>' or ',' as separator (CORRECTNESS > LATENCY or CORRECTNESS, LATENCY)
            if self.check(&TokenKind::Comma) || self.check(&TokenKind::Gt) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(priorities)
    }

    fn parse_failure_strategy(&mut self) -> ParseResult<FailureStrategy> {
        match self.peek_kind() {
            TokenKind::Abort => {
                self.advance();
                Ok(FailureStrategy::Abort)
            }
            TokenKind::Retry => {
                self.advance();
                Ok(FailureStrategy::Retry)
            }
            TokenKind::Fallback => {
                self.advance();
                let ext_ref = self.parse_external_ref()?;
                Ok(FailureStrategy::Fallback(ext_ref))
            }
            TokenKind::Default => {
                self.advance();
                let expr = self.parse_expression()?;
                Ok(FailureStrategy::Default(expr))
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "ABORT, RETRY, FALLBACK, or DEFAULT".to_string(),
                found: self.peek_kind(),
                span: self.peek().span,
            }),
        }
    }

    // =========================================================================
    // CONSTRAINT block
    // =========================================================================

    fn parse_constraint_block(&mut self) -> ParseResult<ConstraintBlock> {
        self.skip_newlines();
        let start = self.peek().span;

        self.consume(TokenKind::Constraint, "CONSTRAINT")?;
        self.expect_newline()?;

        let mut constraints = Vec::new();

        while !self.check(&TokenKind::EndConstraint) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::EndConstraint) {
                break;
            }
            constraints.push(self.parse_constraint()?);
            self.expect_newline()?;
        }

        let end_token = self.consume(TokenKind::EndConstraint, "ENDCONSTRAINT")?;
        self.expect_newline()?;

        Ok(ConstraintBlock {
            constraints,
            span: Span::new(start.start, end_token.span.end),
        })
    }

    fn parse_constraint(&mut self) -> ParseResult<Constraint> {
        let start = self.peek().span;

        let kind = match self.peek_kind() {
            TokenKind::Require => { self.advance(); ConstraintKind::Require }
            TokenKind::Forbid => { self.advance(); ConstraintKind::Forbid }
            TokenKind::Prefer => { self.advance(); ConstraintKind::Prefer }
            TokenKind::Invariant => { self.advance(); ConstraintKind::Invariant }
            _ => {
                return Err(ParseError::InvalidConstraintKind {
                    kind: self.peek().text.clone(),
                    span: self.peek().span,
                });
            }
        };

        // Special handling for WITHIN constraint: REQUIRE WITHIN 10s
        let expr = if self.check(&TokenKind::Within) {
            let within_token = self.advance();
            // Parse the duration value
            let duration_token = self.consume(TokenKind::Duration, "duration (e.g., 10s, 5m)")?;
            let duration_literal = Literal::new(
                LiteralKind::Duration(duration_token.text.clone()),
                duration_token.span,
            );
            // Create a WITHIN(duration) call expression
            Expr::Call(Box::new(CallExpr::new(
                Ident { name: "WITHIN".to_string(), span: within_token.span },
                vec![Expr::Literal(duration_literal)],
                Span::new(within_token.span.start, duration_token.span.end),
            )))
        } else {
            self.parse_expression()?
        };

        Ok(Constraint {
            kind,
            expr,
            span: Span::new(start.start, self.previous().span.end),
        })
    }

    // =========================================================================
    // FLOW block
    // =========================================================================

    fn parse_flow_block(&mut self) -> ParseResult<FlowBlock> {
        self.skip_newlines();
        let start = self.peek().span;

        self.consume(TokenKind::Flow, "FLOW")?;
        self.expect_newline()?;

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        while !self.check(&TokenKind::EndFlow) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::EndFlow) {
                break;
            }

            match self.peek_kind() {
                TokenKind::Node => {
                    nodes.push(self.parse_flow_node()?);
                }
                TokenKind::Edge => {
                    edges.push(self.parse_flow_edge()?);
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "NODE or EDGE".to_string(),
                        found: self.peek_kind(),
                        span: self.peek().span,
                    });
                }
            }

            self.expect_newline()?;
        }

        let end_token = self.consume(TokenKind::EndFlow, "ENDFLOW")?;
        self.expect_newline()?;

        Ok(FlowBlock {
            nodes,
            edges,
            span: Span::new(start.start, end_token.span.end),
        })
    }

    fn parse_flow_node(&mut self) -> ParseResult<FlowNode> {
        let start = self.peek().span;

        self.consume(TokenKind::Node, "NODE")?;

        let id_token = self.consume(TokenKind::Identifier, "node identifier")?;
        let id = Ident {
            name: id_token.text.clone(),
            span: id_token.span,
        };

        self.consume(TokenKind::Colon, ":")?;

        // Parse operation type (or external ref)
        let (op_type, custom_op) = self.parse_op_type()?;

        // Parse optional parameters in parentheses
        let mut params = Vec::new();
        if self.check(&TokenKind::LParen) {
            self.advance();
            while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                params.push(self.parse_node_param()?);
                if self.check(&TokenKind::Comma) {
                    self.advance();
                }
            }
            self.consume(TokenKind::RParen, ")")?;
        }

        Ok(FlowNode {
            id,
            op_type,
            params,
            custom_op,
            span: Span::new(start.start, self.previous().span.end),
        })
    }

    fn parse_op_type(&mut self) -> ParseResult<(OpType, Option<ExternalRef>)> {
        // Check for external reference first
        if self.check(&TokenKind::ExternalRef) {
            let ext_ref = self.parse_external_ref()?;
            return Ok((OpType::Call, Some(ext_ref)));
        }

        let op = match self.peek_kind() {
            TokenKind::Map => { self.advance(); OpType::Map }
            TokenKind::Filter => { self.advance(); OpType::Filter }
            TokenKind::Reduce => { self.advance(); OpType::Reduce }
            TokenKind::Transform => { self.advance(); OpType::Transform }
            TokenKind::Branch => { self.advance(); OpType::Branch }
            TokenKind::Merge => { self.advance(); OpType::Merge }
            TokenKind::Split => { self.advance(); OpType::Split }
            TokenKind::Join => { self.advance(); OpType::Join }
            TokenKind::Race => { self.advance(); OpType::Race }
            TokenKind::Fetch => { self.advance(); OpType::Fetch }
            TokenKind::Store => { self.advance(); OpType::Store }
            TokenKind::Call => { self.advance(); OpType::Call }
            TokenKind::Emit => { self.advance(); OpType::Emit }
            TokenKind::Subscribe => { self.advance(); OpType::Subscribe }
            TokenKind::Validate => { self.advance(); OpType::Validate }
            TokenKind::Sanitize => { self.advance(); OpType::Sanitize }
            TokenKind::Authorize => { self.advance(); OpType::Authorize }
            _ => {
                return Err(ParseError::InvalidFlowOp {
                    op: self.peek().text.clone(),
                    span: self.peek().span,
                });
            }
        };

        Ok((op, None))
    }

    fn parse_node_param(&mut self) -> ParseResult<NodeParam> {
        let start = self.peek().span;

        let name_token = self.consume(TokenKind::Identifier, "parameter name")?;
        let name = Ident {
            name: name_token.text.clone(),
            span: name_token.span,
        };

        self.consume(TokenKind::Assign, "=")?;

        let value = self.parse_expression()?;

        Ok(NodeParam {
            name,
            value,
            span: Span::new(start.start, self.previous().span.end),
        })
    }

    fn parse_flow_edge(&mut self) -> ParseResult<FlowEdge> {
        let start = self.peek().span;

        self.consume(TokenKind::Edge, "EDGE")?;

        let source = self.parse_edge_target_expr()?;

        self.consume(TokenKind::Arrow, "->")?;

        let target = self.parse_edge_target_expr()?;

        // Parse optional edge params: (key=value, ...)
        let params = if self.check(&TokenKind::LParen) {
            self.advance();
            let mut params = Vec::new();
            while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                params.push(self.parse_node_param()?);
                if self.check(&TokenKind::Comma) {
                    self.advance();
                }
            }
            self.consume(TokenKind::RParen, ")")?;
            params
        } else {
            Vec::new()
        };

        // Parse optional condition
        let condition = if self.check(&TokenKind::When) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(FlowEdge {
            source,
            target,
            params,
            condition,
            span: Span::new(start.start, self.previous().span.end),
        })
    }

    // Parse edge target expression without function call parsing
    // This avoids confusion between edge params (key=value) and function call syntax
    fn parse_edge_target_expr(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_primary_expr()?;

        loop {
            if self.check(&TokenKind::Dot) {
                self.advance();
                let field_token = self.consume(TokenKind::Identifier, "field name")?;
                let span = Span::new(expr.span().start, field_token.span.end);
                let field = Ident {
                    name: field_token.text.clone(),
                    span: field_token.span,
                };
                expr = Expr::FieldAccess(Box::new(FieldAccess { base: expr, field, span }));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    // =========================================================================
    // EXECUTION block
    // =========================================================================

    fn parse_execution_block(&mut self) -> ParseResult<ExecutionBlock> {
        self.skip_newlines();
        let start = self.peek().span;

        self.consume(TokenKind::Execution, "EXECUTION")?;
        self.expect_newline()?;

        let mut parallel = false;
        let mut target = TargetKind::Any;
        let mut memory = MemoryMode::Unbounded;
        let mut isolation = IsolationKind::None;
        let mut cache = CacheMode::None;

        while !self.check(&TokenKind::EndExecution) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::EndExecution) {
                break;
            }

            match self.peek_kind() {
                TokenKind::Parallel => {
                    self.advance();
                    parallel = match self.peek_kind() {
                        TokenKind::True => { self.advance(); true }
                        TokenKind::False => { self.advance(); false }
                        _ => true,
                    };
                }
                TokenKind::Target => {
                    self.advance();
                    target = match self.peek_kind() {
                        TokenKind::Any => { self.advance(); TargetKind::Any }
                        TokenKind::Cpu => { self.advance(); TargetKind::Cpu }
                        TokenKind::Gpu => { self.advance(); TargetKind::Gpu }
                        TokenKind::Wasm => { self.advance(); TargetKind::Wasm }
                        TokenKind::Native => { self.advance(); TargetKind::Native }
                        _ => {
                            return Err(ParseError::UnexpectedToken {
                                expected: "ANY, CPU, GPU, WASM, or NATIVE".to_string(),
                                found: self.peek_kind(),
                                span: self.peek().span,
                            });
                        }
                    };
                }
                TokenKind::Memory => {
                    self.advance();
                    memory = match self.peek_kind() {
                        TokenKind::Bounded => {
                            self.advance();
                            let size_token = self.consume(TokenKind::Size, "size value")?;
                            MemoryMode::Bounded(size_token.text.clone())
                        }
                        TokenKind::Unbounded => { self.advance(); MemoryMode::Unbounded }
                        TokenKind::StackOnly => { self.advance(); MemoryMode::StackOnly }
                        _ => {
                            return Err(ParseError::UnexpectedToken {
                                expected: "BOUNDED, UNBOUNDED, or STACK_ONLY".to_string(),
                                found: self.peek_kind(),
                                span: self.peek().span,
                            });
                        }
                    };
                }
                TokenKind::Isolation => {
                    self.advance();
                    isolation = match self.peek_kind() {
                        TokenKind::None_ => { self.advance(); IsolationKind::None }
                        TokenKind::Thread => { self.advance(); IsolationKind::Thread }
                        TokenKind::Process => { self.advance(); IsolationKind::Process }
                        TokenKind::Container => { self.advance(); IsolationKind::Container }
                        _ => {
                            return Err(ParseError::UnexpectedToken {
                                expected: "NONE, THREAD, PROCESS, or CONTAINER".to_string(),
                                found: self.peek_kind(),
                                span: self.peek().span,
                            });
                        }
                    };
                }
                TokenKind::Cache => {
                    self.advance();
                    cache = match self.peek_kind() {
                        TokenKind::None_ => { self.advance(); CacheMode::None }
                        TokenKind::Lru => {
                            self.advance();
                            let capacity = if self.check(&TokenKind::Integer) {
                                let token = self.advance();
                                Some(token.text.parse::<u64>().unwrap_or(0))
                            } else {
                                None
                            };
                            CacheMode::Lru(capacity)
                        }
                        TokenKind::Ttl => {
                            self.advance();
                            let duration = if self.check(&TokenKind::Duration) {
                                let token = self.advance();
                                Some(token.text.clone())
                            } else {
                                None
                            };
                            CacheMode::Ttl(duration)
                        }
                        _ => {
                            return Err(ParseError::UnexpectedToken {
                                expected: "NONE, LRU, or TTL".to_string(),
                                found: self.peek_kind(),
                                span: self.peek().span,
                            });
                        }
                    };
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "PARALLEL, TARGET, MEMORY, ISOLATION, or CACHE".to_string(),
                        found: self.peek_kind(),
                        span: self.peek().span,
                    });
                }
            }

            self.expect_newline()?;
        }

        let end_token = self.consume(TokenKind::EndExecution, "ENDEXECUTION")?;
        self.expect_newline()?;

        Ok(ExecutionBlock {
            parallel,
            target,
            memory,
            isolation,
            cache,
            span: Span::new(start.start, end_token.span.end),
        })
    }

    // =========================================================================
    // VERIFY block
    // =========================================================================

    fn parse_verify_block(&mut self) -> ParseResult<VerifyBlock> {
        self.skip_newlines();
        let start = self.peek().span;

        self.consume(TokenKind::Verify, "VERIFY")?;
        self.expect_newline()?;

        let mut entries = Vec::new();

        while !self.check(&TokenKind::EndVerify) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::EndVerify) {
                break;
            }
            entries.push(self.parse_verify_entry()?);
            self.expect_newline()?;
        }

        let end_token = self.consume(TokenKind::EndVerify, "ENDVERIFY")?;
        self.expect_newline()?;

        Ok(VerifyBlock {
            entries,
            span: Span::new(start.start, end_token.span.end),
        })
    }

    fn parse_verify_entry(&mut self) -> ParseResult<VerifyEntry> {
        let start = self.peek().span;

        let kind = match self.peek_kind() {
            TokenKind::Assert => { self.advance(); VerifyKind::Assert }
            TokenKind::Property => { self.advance(); VerifyKind::Property }
            TokenKind::Invariant => { self.advance(); VerifyKind::Invariant }
            TokenKind::Postcondition => { self.advance(); VerifyKind::Postcondition }
            TokenKind::Test => { self.advance(); VerifyKind::Test }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "ASSERT, PROPERTY, INVARIANT, POSTCONDITION, or TEST".to_string(),
                    found: self.peek_kind(),
                    span: self.peek().span,
                });
            }
        };

        // Parse expression or external reference
        let (expr, test_ref) = if self.check(&TokenKind::ExternalRef) {
            (None, Some(self.parse_external_ref()?))
        } else {
            (Some(self.parse_expression()?), None)
        };

        Ok(VerifyEntry {
            kind,
            expr,
            test_ref,
            span: Span::new(start.start, self.previous().span.end),
        })
    }

    // =========================================================================
    // Type parsing
    // =========================================================================

    fn parse_type(&mut self) -> ParseResult<Type> {
        let start = self.peek().span;

        match self.peek_kind() {
            // Primitive types
            TokenKind::Int => { self.advance(); Ok(Type::Primitive(PrimitiveType::new(PrimitiveKind::Int, start))) }
            TokenKind::Int8 => { self.advance(); Ok(Type::Primitive(PrimitiveType::new(PrimitiveKind::Int8, start))) }
            TokenKind::Int16 => { self.advance(); Ok(Type::Primitive(PrimitiveType::new(PrimitiveKind::Int16, start))) }
            TokenKind::Int32 => { self.advance(); Ok(Type::Primitive(PrimitiveType::new(PrimitiveKind::Int32, start))) }
            TokenKind::Int64 => { self.advance(); Ok(Type::Primitive(PrimitiveType::new(PrimitiveKind::Int64, start))) }
            TokenKind::Uint => { self.advance(); Ok(Type::Primitive(PrimitiveType::new(PrimitiveKind::Uint, start))) }
            TokenKind::Uint8 => { self.advance(); Ok(Type::Primitive(PrimitiveType::new(PrimitiveKind::Uint8, start))) }
            TokenKind::Uint16 => { self.advance(); Ok(Type::Primitive(PrimitiveType::new(PrimitiveKind::Uint16, start))) }
            TokenKind::Uint32 => { self.advance(); Ok(Type::Primitive(PrimitiveType::new(PrimitiveKind::Uint32, start))) }
            TokenKind::Uint64 => { self.advance(); Ok(Type::Primitive(PrimitiveType::new(PrimitiveKind::Uint64, start))) }
            TokenKind::Float32 => { self.advance(); Ok(Type::Primitive(PrimitiveType::new(PrimitiveKind::Float32, start))) }
            TokenKind::Float64 => { self.advance(); Ok(Type::Primitive(PrimitiveType::new(PrimitiveKind::Float64, start))) }
            TokenKind::Bool => { self.advance(); Ok(Type::Primitive(PrimitiveType::new(PrimitiveKind::Bool, start))) }
            TokenKind::String_ => { self.advance(); Ok(Type::Primitive(PrimitiveType::new(PrimitiveKind::String, start))) }
            TokenKind::Bytes => { self.advance(); Ok(Type::Primitive(PrimitiveType::new(PrimitiveKind::Bytes, start))) }
            TokenKind::Void => { self.advance(); Ok(Type::Primitive(PrimitiveType::new(PrimitiveKind::Void, start))) }

            // Compound types
            TokenKind::Array => {
                self.advance();
                self.consume(TokenKind::Lt, "<")?;
                let element_type = self.parse_type()?;
                self.consume(TokenKind::Gt, ">")?;
                let span = Span::new(start.start, self.previous().span.end);
                Ok(Type::Array(Box::new(ArrayType::new(element_type, span))))
            }

            TokenKind::Optional => {
                self.advance();
                self.consume(TokenKind::Lt, "<")?;
                let inner_type = self.parse_type()?;
                self.consume(TokenKind::Gt, ">")?;
                let span = Span::new(start.start, self.previous().span.end);
                Ok(Type::Optional(Box::new(OptionalType::new(inner_type, span))))
            }

            TokenKind::Struct => {
                self.advance();
                self.consume(TokenKind::LBrace, "{")?;

                let mut fields = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                    let name_token = self.consume(TokenKind::Identifier, "field name")?;
                    let field_start = name_token.span;
                    let name = Ident {
                        name: name_token.text.clone(),
                        span: name_token.span,
                    };
                    self.consume(TokenKind::Colon, ":")?;
                    let ty = self.parse_type()?;
                    let field_span = Span::new(field_start.start, self.previous().span.end);
                    fields.push(StructField { name, ty, span: field_span });

                    if self.check(&TokenKind::Comma) {
                        self.advance();
                    }
                }

                self.consume(TokenKind::RBrace, "}")?;
                let span = Span::new(start.start, self.previous().span.end);
                Ok(Type::Struct(StructType::new(fields, span)))
            }

            TokenKind::Union => {
                self.advance();
                self.consume(TokenKind::Lt, "<")?;

                let mut types = Vec::new();
                types.push(self.parse_type()?);

                while self.check(&TokenKind::Pipe) {
                    self.advance();
                    types.push(self.parse_type()?);
                }

                self.consume(TokenKind::Gt, ">")?;
                let span = Span::new(start.start, self.previous().span.end);
                Ok(Type::Union(UnionType::new(types, span)))
            }

            // Named type (user-defined) - just an identifier
            TokenKind::Identifier => {
                let token = self.advance();
                let name = self.parse_qualified_name_from_token(token)?;
                Ok(Type::Ref(ExternalRef { path: name.full_name(), span: name.span }))
            }

            _ => Err(ParseError::InvalidType {
                ty: self.peek().text.clone(),
                span: self.peek().span,
            }),
        }
    }

    fn parse_qualified_name_from_token(&mut self, first: Token) -> ParseResult<QualifiedName> {
        let start = first.span;
        let mut parts = vec![Ident {
            name: first.text.clone(),
            span: first.span,
        }];

        while self.check(&TokenKind::Dot) {
            self.advance();
            let token = self.consume(TokenKind::Identifier, "identifier")?;
            parts.push(Ident {
                name: token.text.clone(),
                span: token.span,
            });
        }

        let span = Span::new(start.start, self.previous().span.end);
        Ok(QualifiedName { parts, span })
    }

    // =========================================================================
    // Expression parsing
    // =========================================================================

    fn parse_expression(&mut self) -> ParseResult<Expr> {
        self.parse_implies_expr()
    }

    fn parse_implies_expr(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_or_expr()?;

        // IMPLIES is right-associative with lowest precedence
        while self.check(&TokenKind::Implies) {
            self.advance();
            let right = self.parse_implies_expr()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(Box::new(BinaryExpr::new(left, BinaryOp::Implies, right, span)));
        }

        Ok(left)
    }

    fn parse_or_expr(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_and_expr()?;

        while self.check(&TokenKind::Or) || self.check(&TokenKind::Xor) {
            let op = if self.check(&TokenKind::Xor) {
                self.advance();
                BinaryOp::Xor
            } else {
                self.advance();
                BinaryOp::Or
            };
            let right = self.parse_and_expr()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(Box::new(BinaryExpr::new(left, op, right, span)));
        }

        Ok(left)
    }

    fn parse_and_expr(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_comparison_expr()?;

        while self.check(&TokenKind::And) {
            self.advance();
            let right = self.parse_comparison_expr()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(Box::new(BinaryExpr::new(left, BinaryOp::And, right, span)));
        }

        Ok(left)
    }

    fn parse_comparison_expr(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_additive_expr()?;

        loop {
            let op = match self.peek_kind() {
                TokenKind::Eq => BinaryOp::Eq,
                TokenKind::Neq => BinaryOp::Neq,
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::Lte => BinaryOp::Lte,
                TokenKind::Gte => BinaryOp::Gte,
                TokenKind::In => BinaryOp::In,
                TokenKind::Match => BinaryOp::Match,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive_expr()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(Box::new(BinaryExpr::new(left, op, right, span)));
        }

        Ok(left)
    }

    fn parse_additive_expr(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_multiplicative_expr()?;

        loop {
            let op = match self.peek_kind() {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative_expr()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(Box::new(BinaryExpr::new(left, op, right, span)));
        }

        Ok(left)
    }

    fn parse_multiplicative_expr(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_unary_expr()?;

        loop {
            let op = match self.peek_kind() {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary_expr()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(Box::new(BinaryExpr::new(left, op, right, span)));
        }

        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> ParseResult<Expr> {
        if self.check(&TokenKind::Not) {
            let start = self.peek().span;
            self.advance();
            let operand = self.parse_unary_expr()?;
            let span = Span::new(start.start, operand.span().end);
            return Ok(Expr::Unary(Box::new(UnaryExpr::new(UnaryOp::Not, operand, span))));
        }

        if self.check(&TokenKind::Minus) {
            let start = self.peek().span;
            self.advance();
            let operand = self.parse_unary_expr()?;
            let span = Span::new(start.start, operand.span().end);
            return Ok(Expr::Unary(Box::new(UnaryExpr::new(UnaryOp::Neg, operand, span))));
        }

        self.parse_postfix_expr()
    }

    fn parse_postfix_expr(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_primary_expr()?;

        loop {
            if self.check(&TokenKind::Dot) {
                self.advance();
                let field_token = self.consume(TokenKind::Identifier, "field name")?;
                let span = Span::new(expr.span().start, field_token.span.end);
                let field = Ident {
                    name: field_token.text.clone(),
                    span: field_token.span,
                };
                expr = Expr::FieldAccess(Box::new(FieldAccess { base: expr, field, span }));
            } else if self.check(&TokenKind::LBracket) {
                self.advance();
                let index = self.parse_expression()?;
                self.consume(TokenKind::RBracket, "]")?;
                let span = Span::new(expr.span().start, self.previous().span.end);
                expr = Expr::Index(Box::new(IndexExpr { base: expr, index, span }));
            } else if self.check(&TokenKind::LParen) {
                // Function call
                self.advance();
                let mut args = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                    args.push(self.parse_expression()?);
                    if self.check(&TokenKind::Comma) {
                        self.advance();
                    }
                }
                self.consume(TokenKind::RParen, ")")?;
                let span = Span::new(expr.span().start, self.previous().span.end);

                // Convert to function call
                if let Expr::Ident(ident) = expr {
                    expr = Expr::Call(Box::new(CallExpr::new(ident, args, span)));
                } else {
                    return Err(ParseError::InvalidExpression { span: expr.span() });
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary_expr(&mut self) -> ParseResult<Expr> {
        match self.peek_kind() {
            // Literals
            TokenKind::Integer => {
                let token = self.advance();
                let value = token.text.parse::<i64>()
                    .map_err(|_| ParseError::InvalidLiteral { span: token.span })?;
                Ok(Expr::Literal(Literal::integer(value, token.span)))
            }

            TokenKind::Float => {
                let token = self.advance();
                let value = token.text.parse::<f64>()
                    .map_err(|_| ParseError::InvalidLiteral { span: token.span })?;
                Ok(Expr::Literal(Literal::float(value, token.span)))
            }

            TokenKind::StringLiteral => {
                let token = self.advance();
                let value = token.text[1..token.text.len()-1].to_string();
                Ok(Expr::Literal(Literal::string(value, token.span)))
            }

            TokenKind::True => {
                let token = self.advance();
                Ok(Expr::Literal(Literal::bool(true, token.span)))
            }

            TokenKind::False => {
                let token = self.advance();
                Ok(Expr::Literal(Literal::bool(false, token.span)))
            }

            TokenKind::Void => {
                let token = self.advance();
                Ok(Expr::Literal(Literal::void(token.span)))
            }

            TokenKind::Regex => {
                let token = self.advance();
                let pattern = token.text[1..token.text.len()-1].to_string();
                Ok(Expr::Literal(Literal::new(LiteralKind::Regex(pattern), token.span)))
            }

            // Duration literal (e.g., 10s, 5m, 100ms)
            TokenKind::Duration => {
                let token = self.advance();
                Ok(Expr::Literal(Literal::new(LiteralKind::Duration(token.text.clone()), token.span)))
            }

            // Size literal (e.g., 256MB, 1GB)
            TokenKind::Size => {
                let token = self.advance();
                Ok(Expr::Literal(Literal::new(LiteralKind::Size(token.text.clone()), token.span)))
            }

            // External reference
            TokenKind::ExternalRef => {
                let ext_ref = self.parse_external_ref()?;
                Ok(Expr::ExternalRef(ext_ref))
            }

            // Built-in functions or special identifiers
            TokenKind::Len | TokenKind::Contains | TokenKind::Range |
            TokenKind::Now | TokenKind::Sum | TokenKind::Count |
            TokenKind::Forall | TokenKind::Exists => {
                let token = self.advance();
                let name = Ident {
                    name: token.text.clone(),
                    span: token.span,
                };

                // If followed by '(', parse as function call
                // Otherwise treat as identifier (e.g., LEN in field constraints)
                if self.check(&TokenKind::LParen) {
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                        args.push(self.parse_expression()?);
                        if self.check(&TokenKind::Comma) {
                            self.advance();
                        }
                    }
                    self.consume(TokenKind::RParen, ")")?;

                    Ok(Expr::Call(Box::new(CallExpr::new(
                        name,
                        args,
                        Span::new(token.span.start, self.previous().span.end),
                    ))))
                } else {
                    // Treat as identifier (special field reference like LEN in constraints)
                    Ok(Expr::Ident(name))
                }
            }

            // Parenthesized expression
            TokenKind::LParen => {
                let start = self.peek().span;
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(TokenKind::RParen, ")")?;
                let span = Span::new(start.start, self.previous().span.end);
                Ok(Expr::Grouped(Box::new(GroupedExpr::new(expr, span))))
            }

            // Identifier
            TokenKind::Identifier => {
                let token = self.advance();
                Ok(Expr::Ident(Ident {
                    name: token.text.clone(),
                    span: token.span,
                }))
            }

            // Allow some keywords to be used as identifiers in expressions
            TokenKind::Output | TokenKind::Input => {
                let token = self.advance();
                Ok(Expr::Ident(Ident {
                    name: token.text.clone(),
                    span: token.span,
                }))
            }

            _ => Err(ParseError::InvalidExpression { span: self.peek().span }),
        }
    }

    fn parse_external_ref(&mut self) -> ParseResult<ExternalRef> {
        let token = self.consume(TokenKind::ExternalRef, "@reference")?;
        let path = token.text[1..].to_string(); // Remove '@' prefix
        Ok(ExternalRef {
            path,
            span: token.span,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_unit() {
        let source = r#"
UNIT FUNCTION test V1.0.0
META
  DOMAIN test
  DETERMINISM true
ENDMETA
INPUT
ENDINPUT
OUTPUT
ENDOUTPUT
INTENT
  GOAL TRANSFORM: () -> ()
ENDINTENT
CONSTRAINT
ENDCONSTRAINT
FLOW
ENDFLOW
EXECUTION
  PARALLEL false
ENDEXECUTION
VERIFY
ENDVERIFY
END
"#;
        let result = Parser::parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let unit = result.unwrap();
        assert_eq!(unit.header.kind, UnitKind::Function);
        assert_eq!(unit.full_name(), "test");
    }

    #[test]
    fn test_parse_types() {
        let source = r#"
UNIT FUNCTION types_test V1.0.0
META
  DOMAIN test
ENDMETA
INPUT
  name: STRING
  age: INT
  scores: ARRAY<FLOAT64>
  email: OPTIONAL<STRING>
ENDINPUT
OUTPUT
  result: BOOL
ENDOUTPUT
INTENT
  GOAL VALIDATE: (name, age) -> (result)
ENDINTENT
CONSTRAINT
ENDCONSTRAINT
FLOW
ENDFLOW
EXECUTION
ENDEXECUTION
VERIFY
ENDVERIFY
END
"#;
        let result = Parser::parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());

        let unit = result.unwrap();
        assert_eq!(unit.input.fields.len(), 4);
    }
}
