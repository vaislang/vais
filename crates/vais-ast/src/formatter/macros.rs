//! Format macro definitions and related structures

use super::*;

impl Formatter {
    /// Format a macro definition
    pub(crate) fn format_macro(&mut self, m: &MacroDef) {
        let indent = self.indent();
        self.output.push_str(&indent);

        if m.is_pub {
            self.output.push_str("P ");
        }

        self.output.push_str("macro ");
        self.output.push_str(&m.name.node);
        self.output.push_str("! {\n");

        // Format each rule
        for rule in &m.rules {
            self.output.push_str(&indent);
            self.output.push_str("    ");
            self.format_macro_pattern(&rule.pattern);
            self.output.push_str(" => ");
            self.format_macro_template(&rule.template);
            self.output.push('\n');
        }

        self.output.push_str(&indent);
        self.output.push_str("}\n");
    }

    pub(crate) fn format_macro_pattern(&mut self, pattern: &MacroPattern) {
        self.output.push('(');
        match pattern {
            MacroPattern::Empty => {}
            MacroPattern::Sequence(elements) => {
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_macro_pattern_element(element);
                }
            }
        }
        self.output.push(')');
    }

    pub(crate) fn format_macro_pattern_element(&mut self, element: &MacroPatternElement) {
        match element {
            MacroPatternElement::Token(tok) => self.format_macro_token(tok),
            MacroPatternElement::MetaVar { name, kind } => {
                self.output.push('$');
                self.output.push_str(name);
                self.output.push(':');
                self.output.push_str(match kind {
                    MetaVarKind::Expr => "expr",
                    MetaVarKind::Ty => "ty",
                    MetaVarKind::Ident => "ident",
                    MetaVarKind::Pat => "pat",
                    MetaVarKind::Stmt => "stmt",
                    MetaVarKind::Block => "block",
                    MetaVarKind::Item => "item",
                    MetaVarKind::Lit => "lit",
                    MetaVarKind::Tt => "tt",
                });
            }
            MacroPatternElement::Repetition {
                patterns,
                separator,
                kind,
            } => {
                self.output.push_str("$(");
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_macro_pattern_element(p);
                }
                self.output.push(')');
                if let Some(sep) = separator {
                    self.format_macro_token(sep);
                }
                self.output.push(match kind {
                    RepetitionKind::ZeroOrMore => '*',
                    RepetitionKind::OneOrMore => '+',
                    RepetitionKind::ZeroOrOne => '?',
                });
            }
            MacroPatternElement::Group { delimiter, content } => {
                self.output.push(match delimiter {
                    Delimiter::Paren => '(',
                    Delimiter::Bracket => '[',
                    Delimiter::Brace => '{',
                });
                for (i, c) in content.iter().enumerate() {
                    if i > 0 {
                        self.output.push(' ');
                    }
                    self.format_macro_pattern_element(c);
                }
                self.output.push(match delimiter {
                    Delimiter::Paren => ')',
                    Delimiter::Bracket => ']',
                    Delimiter::Brace => '}',
                });
            }
        }
    }

    pub(crate) fn format_macro_template(&mut self, template: &MacroTemplate) {
        self.output.push('{');
        match template {
            MacroTemplate::Empty => {}
            MacroTemplate::Sequence(elements) => {
                for element in elements {
                    self.output.push(' ');
                    self.format_macro_template_element(element);
                }
            }
        }
        self.output.push_str(" }");
    }

    pub(crate) fn format_macro_template_element(&mut self, element: &MacroTemplateElement) {
        match element {
            MacroTemplateElement::Token(tok) => self.format_macro_token(tok),
            MacroTemplateElement::MetaVar(name) => {
                self.output.push('$');
                self.output.push_str(name);
            }
            MacroTemplateElement::Repetition {
                elements,
                separator,
                kind,
            } => {
                self.output.push_str("$(");
                for e in elements {
                    self.format_macro_template_element(e);
                }
                self.output.push(')');
                if let Some(sep) = separator {
                    self.format_macro_token(sep);
                }
                self.output.push(match kind {
                    RepetitionKind::ZeroOrMore => '*',
                    RepetitionKind::OneOrMore => '+',
                    RepetitionKind::ZeroOrOne => '?',
                });
            }
            MacroTemplateElement::Group { delimiter, content } => {
                self.output.push(match delimiter {
                    Delimiter::Paren => '(',
                    Delimiter::Bracket => '[',
                    Delimiter::Brace => '{',
                });
                for c in content {
                    self.format_macro_template_element(c);
                }
                self.output.push(match delimiter {
                    Delimiter::Paren => ')',
                    Delimiter::Bracket => ']',
                    Delimiter::Brace => '}',
                });
            }
        }
    }

    pub(crate) fn format_macro_token(&mut self, token: &MacroToken) {
        match token {
            MacroToken::Ident(s) => self.output.push_str(s),
            MacroToken::Punct(c) => self.output.push(*c),
            MacroToken::Literal(lit) => match lit {
                MacroLiteral::Int(n) => self.output.push_str(&n.to_string()),
                MacroLiteral::Float(n) => self.output.push_str(&n.to_string()),
                MacroLiteral::String(s) => {
                    self.output.push('"');
                    self.output.push_str(s);
                    self.output.push('"');
                }
                MacroLiteral::Bool(b) => self.output.push_str(if *b { "true" } else { "false" }),
            },
            MacroToken::Group(delim, inner) => {
                self.output.push(match delim {
                    Delimiter::Paren => '(',
                    Delimiter::Bracket => '[',
                    Delimiter::Brace => '{',
                });
                for tok in inner {
                    self.format_macro_token(tok);
                }
                self.output.push(match delim {
                    Delimiter::Paren => ')',
                    Delimiter::Bracket => ']',
                    Delimiter::Brace => '}',
                });
            }
        }
    }

    pub(crate) fn format_macro_tokens(&self, tokens: &[MacroToken]) -> String {
        let mut result = String::with_capacity(256);
        for token in tokens {
            match token {
                MacroToken::Ident(s) => result.push_str(s),
                MacroToken::Punct(c) => result.push(*c),
                MacroToken::Literal(lit) => match lit {
                    MacroLiteral::Int(n) => result.push_str(&n.to_string()),
                    MacroLiteral::Float(n) => result.push_str(&n.to_string()),
                    MacroLiteral::String(s) => {
                        result.push('"');
                        result.push_str(s);
                        result.push('"');
                    }
                    MacroLiteral::Bool(b) => result.push_str(if *b { "true" } else { "false" }),
                },
                MacroToken::Group(delim, inner) => {
                    result.push(match delim {
                        Delimiter::Paren => '(',
                        Delimiter::Bracket => '[',
                        Delimiter::Brace => '{',
                    });
                    result.push_str(&self.format_macro_tokens(inner));
                    result.push(match delim {
                        Delimiter::Paren => ')',
                        Delimiter::Bracket => ']',
                        Delimiter::Brace => '}',
                    });
                }
            }
        }
        result
    }
}
