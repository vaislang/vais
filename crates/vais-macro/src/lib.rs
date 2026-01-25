//! Vais Macro System
//!
//! Declarative macro expansion for the Vais programming language.
//! Supports pattern matching, repetition, and metavariable substitution.
//!
//! # Modules
//!
//! - [`expansion`]: AST-level macro expansion with hygienic naming
//! - [`derive`]: #[derive(...)] attribute macro framework
//!
//! # Example
//!
//! ```ignore
//! use vais_macro::{MacroRegistry, expand_macros, process_derives};
//! use vais_parser::parse;
//!
//! let source = r#"
//!     macro double! { ($x:expr) => { $x + $x } }
//!     F test() = double!(5)
//! "#;
//!
//! let mut module = parse(source).unwrap();
//! let mut registry = MacroRegistry::new();
//! collect_macros(&module, &mut registry);
//! let expanded = expand_macros(module, &registry).unwrap();
//! ```

pub mod expansion;
pub mod derive;
pub mod async_macros;

use std::collections::HashMap;
use thiserror::Error;
use vais_ast::*;

// Re-export main expansion functions
pub use expansion::{
    expand_macros, collect_macros, AstExpander,
    HygienicContext, ExpansionError, ExpansionResult,
};
pub use derive::{
    process_derives, DeriveRegistry, DeriveGenerator,
    DeriveError, DeriveResult,
};
pub use async_macros::{
    register_async_macros, AsyncMacroExpander,
    SELECT_MACRO, JOIN_MACRO, TIMEOUT_MACRO,
};

/// Error type for macro expansion failures.
#[derive(Debug, Error)]
pub enum MacroError {
    #[error("Undefined macro: {0}")]
    UndefinedMacro(String),
    #[error("No matching rule for macro invocation: {0}")]
    NoMatchingRule(String),
    #[error("Undefined metavariable: ${0}")]
    UndefinedMetaVar(String),
    #[error("Repetition mismatch: ${0} has different lengths")]
    RepetitionMismatch(String),
    #[error("Invalid fragment: expected {expected}, found {found}")]
    InvalidFragment { expected: String, found: String },
    #[error("Parse error during macro expansion: {0}")]
    ParseError(String),
}

type MacroResult<T> = Result<T, MacroError>;

/// Macro definition registry
#[derive(Debug, Default)]
pub struct MacroRegistry {
    macros: HashMap<String, MacroDef>,
}

impl MacroRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a macro definition
    pub fn register(&mut self, def: MacroDef) {
        self.macros.insert(def.name.node.clone(), def);
    }

    /// Get a macro definition by name
    pub fn get(&self, name: &str) -> Option<&MacroDef> {
        self.macros.get(name)
    }

    /// Check if a macro exists
    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    /// Get the number of registered macros
    pub fn macros_count(&self) -> usize {
        self.macros.len()
    }
}

/// Captured metavariable bindings during pattern matching
#[derive(Debug, Clone)]
pub enum Binding {
    /// Single value binding
    Single(Vec<MacroToken>),
    /// Repeated bindings
    Repeated(Vec<Binding>),
}

/// Macro expander that transforms macro invocations into expanded code
#[derive(Debug)]
pub struct MacroExpander<'a> {
    registry: &'a MacroRegistry,
}

impl<'a> MacroExpander<'a> {
    /// Create a new expander with the given registry
    pub fn new(registry: &'a MacroRegistry) -> Self {
        Self { registry }
    }

    /// Expand a macro invocation
    pub fn expand(&self, invoke: &MacroInvoke) -> MacroResult<Vec<MacroToken>> {
        let name = &invoke.name.node;
        let def = self.registry.get(name).ok_or_else(|| MacroError::UndefinedMacro(name.clone()))?;

        // Try each rule until one matches
        for rule in &def.rules {
            if let Some(bindings) = self.match_pattern(&rule.pattern, &invoke.tokens) {
                return self.expand_template(&rule.template, &bindings);
            }
        }

        Err(MacroError::NoMatchingRule(name.clone()))
    }

    /// Match tokens against a pattern, returning bindings if successful
    fn match_pattern(
        &self,
        pattern: &MacroPattern,
        tokens: &[MacroToken],
    ) -> Option<HashMap<String, Binding>> {
        match pattern {
            MacroPattern::Empty => {
                if tokens.is_empty() {
                    Some(HashMap::new())
                } else {
                    None
                }
            }
            MacroPattern::Sequence(elements) => {
                let mut bindings = HashMap::new();
                let mut pos = 0;

                if self.match_elements(elements, tokens, &mut pos, &mut bindings) && pos == tokens.len() {
                    Some(bindings)
                } else {
                    None
                }
            }
        }
    }

    /// Match pattern elements against tokens
    fn match_elements(
        &self,
        elements: &[MacroPatternElement],
        tokens: &[MacroToken],
        pos: &mut usize,
        bindings: &mut HashMap<String, Binding>,
    ) -> bool {
        for element in elements {
            if !self.match_element(element, tokens, pos, bindings) {
                return false;
            }
        }
        true
    }

    /// Match a single pattern element
    fn match_element(
        &self,
        element: &MacroPatternElement,
        tokens: &[MacroToken],
        pos: &mut usize,
        bindings: &mut HashMap<String, Binding>,
    ) -> bool {
        match element {
            MacroPatternElement::Token(expected) => {
                if *pos >= tokens.len() {
                    return false;
                }
                if self.tokens_match(&tokens[*pos], expected) {
                    *pos += 1;
                    true
                } else {
                    false
                }
            }
            MacroPatternElement::MetaVar { name, kind } => {
                if *pos >= tokens.len() {
                    return false;
                }

                // Capture tokens based on the kind
                if let Some(captured) = self.capture_fragment(*kind, tokens, pos) {
                    bindings.insert(name.clone(), Binding::Single(captured));
                    true
                } else {
                    false
                }
            }
            MacroPatternElement::Repetition { patterns, separator, kind } => {
                let mut repetitions = Vec::new();
                let mut first = true;

                loop {
                    // Check for separator (except for first iteration)
                    if !first {
                        if let Some(sep) = separator {
                            if *pos >= tokens.len() || !self.tokens_match(&tokens[*pos], sep) {
                                break;
                            }
                            *pos += 1;
                        }
                    }
                    first = false;

                    // Try to match the pattern
                    let start_pos = *pos;
                    let mut inner_bindings = HashMap::new();

                    if self.match_elements(patterns, tokens, pos, &mut inner_bindings) {
                        repetitions.push(inner_bindings);
                    } else {
                        // Restore position and break
                        *pos = start_pos;
                        break;
                    }

                    // For ZeroOrOne, only one iteration
                    if *kind == RepetitionKind::ZeroOrOne && !repetitions.is_empty() {
                        break;
                    }
                }

                // Check minimum repetitions
                let min = match kind {
                    RepetitionKind::ZeroOrMore | RepetitionKind::ZeroOrOne => 0,
                    RepetitionKind::OneOrMore => 1,
                };

                if repetitions.len() >= min {
                    // Merge repeated bindings
                    self.merge_repetition_bindings(&repetitions, bindings);
                    true
                } else {
                    false
                }
            }
            MacroPatternElement::Group { delimiter, content } => {
                if *pos >= tokens.len() {
                    return false;
                }

                // Match opening delimiter
                if let MacroToken::Group(d, inner) = &tokens[*pos] {
                    if d == delimiter {
                        let mut inner_pos = 0;
                        let result = self.match_elements(content, inner, &mut inner_pos, bindings);
                        if result && inner_pos == inner.len() {
                            *pos += 1;
                            return true;
                        }
                    }
                }
                false
            }
        }
    }

    /// Capture a fragment of the given kind
    fn capture_fragment(
        &self,
        kind: MetaVarKind,
        tokens: &[MacroToken],
        pos: &mut usize,
    ) -> Option<Vec<MacroToken>> {
        if *pos >= tokens.len() {
            return None;
        }

        match kind {
            MetaVarKind::Ident => {
                if let MacroToken::Ident(_) = &tokens[*pos] {
                    let token = tokens[*pos].clone();
                    *pos += 1;
                    Some(vec![token])
                } else {
                    None
                }
            }
            MetaVarKind::Lit => {
                if let MacroToken::Literal(_) = &tokens[*pos] {
                    let token = tokens[*pos].clone();
                    *pos += 1;
                    Some(vec![token])
                } else {
                    None
                }
            }
            MetaVarKind::Tt => {
                // Token tree: single token or group
                let token = tokens[*pos].clone();
                *pos += 1;
                Some(vec![token])
            }
            MetaVarKind::Expr => {
                // For expressions, we need to capture balanced tokens
                // This is a simplified version that captures until a separator
                self.capture_balanced(tokens, pos)
            }
            MetaVarKind::Ty => {
                // For types, capture until separator
                self.capture_balanced(tokens, pos)
            }
            MetaVarKind::Pat => {
                // For patterns, capture until =>
                self.capture_until_fat_arrow(tokens, pos)
            }
            MetaVarKind::Stmt | MetaVarKind::Block | MetaVarKind::Item => {
                // For statements/blocks/items, capture balanced braces
                self.capture_balanced(tokens, pos)
            }
        }
    }

    /// Capture balanced tokens (handles nested groups)
    fn capture_balanced(&self, tokens: &[MacroToken], pos: &mut usize) -> Option<Vec<MacroToken>> {
        let mut result = Vec::new();
        let mut depth = 0;

        while *pos < tokens.len() {
            let token = &tokens[*pos];

            // Check for end conditions
            match token {
                MacroToken::Punct(',') if depth == 0 => break,
                MacroToken::Punct(';') if depth == 0 => break,
                MacroToken::Punct(')') if depth == 0 => break,
                MacroToken::Punct(']') if depth == 0 => break,
                MacroToken::Punct('}') if depth == 0 => break,
                MacroToken::Group(_, _) => {
                    result.push(token.clone());
                    *pos += 1;
                }
                MacroToken::Punct('(') | MacroToken::Punct('[') | MacroToken::Punct('{') => {
                    depth += 1;
                    result.push(token.clone());
                    *pos += 1;
                }
                MacroToken::Punct(')') | MacroToken::Punct(']') | MacroToken::Punct('}') => {
                    depth -= 1;
                    result.push(token.clone());
                    *pos += 1;
                }
                _ => {
                    result.push(token.clone());
                    *pos += 1;
                }
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Capture until fat arrow (for patterns)
    fn capture_until_fat_arrow(&self, tokens: &[MacroToken], pos: &mut usize) -> Option<Vec<MacroToken>> {
        let mut result = Vec::new();

        while *pos < tokens.len() {
            // Check for => (two separate tokens: = and >)
            if *pos + 1 < tokens.len() {
                if let (MacroToken::Punct('='), MacroToken::Punct('>')) = (&tokens[*pos], &tokens[*pos + 1]) {
                    break;
                }
            }
            result.push(tokens[*pos].clone());
            *pos += 1;
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Merge repetition bindings into the main bindings map
    fn merge_repetition_bindings(
        &self,
        repetitions: &[HashMap<String, Binding>],
        bindings: &mut HashMap<String, Binding>,
    ) {
        if repetitions.is_empty() {
            return;
        }

        // Collect all variable names from the first repetition
        let names: Vec<_> = repetitions[0].keys().cloned().collect();

        for name in names {
            let repeated: Vec<Binding> = repetitions
                .iter()
                .filter_map(|r| r.get(&name).cloned())
                .collect();
            bindings.insert(name, Binding::Repeated(repeated));
        }
    }

    /// Check if two macro tokens match
    fn tokens_match(&self, actual: &MacroToken, expected: &MacroToken) -> bool {
        match (actual, expected) {
            (MacroToken::Ident(a), MacroToken::Ident(b)) => a == b,
            (MacroToken::Punct(a), MacroToken::Punct(b)) => a == b,
            (MacroToken::Literal(a), MacroToken::Literal(b)) => a == b,
            (MacroToken::Group(d1, t1), MacroToken::Group(d2, t2)) => {
                d1 == d2 && t1.len() == t2.len() && t1.iter().zip(t2).all(|(a, b)| self.tokens_match(a, b))
            }
            _ => false,
        }
    }

    /// Expand a template with the given bindings
    fn expand_template(
        &self,
        template: &MacroTemplate,
        bindings: &HashMap<String, Binding>,
    ) -> MacroResult<Vec<MacroToken>> {
        match template {
            MacroTemplate::Empty => Ok(Vec::new()),
            MacroTemplate::Sequence(elements) => {
                self.expand_template_elements(elements, bindings, 0)
            }
        }
    }

    /// Expand template elements
    fn expand_template_elements(
        &self,
        elements: &[MacroTemplateElement],
        bindings: &HashMap<String, Binding>,
        rep_index: usize,
    ) -> MacroResult<Vec<MacroToken>> {
        let mut result = Vec::new();

        for element in elements {
            result.extend(self.expand_template_element(element, bindings, rep_index)?);
        }

        Ok(result)
    }

    /// Expand a single template element
    fn expand_template_element(
        &self,
        element: &MacroTemplateElement,
        bindings: &HashMap<String, Binding>,
        rep_index: usize,
    ) -> MacroResult<Vec<MacroToken>> {
        match element {
            MacroTemplateElement::Token(token) => Ok(vec![token.clone()]),
            MacroTemplateElement::MetaVar(name) => {
                let binding = bindings.get(name).ok_or_else(|| MacroError::UndefinedMetaVar(name.clone()))?;

                match binding {
                    Binding::Single(tokens) => Ok(tokens.clone()),
                    Binding::Repeated(items) => {
                        if rep_index < items.len() {
                            if let Binding::Single(tokens) = &items[rep_index] {
                                Ok(tokens.clone())
                            } else {
                                Err(MacroError::RepetitionMismatch(name.clone()))
                            }
                        } else {
                            Err(MacroError::RepetitionMismatch(name.clone()))
                        }
                    }
                }
            }
            MacroTemplateElement::Repetition { elements, separator, kind: _ } => {
                // Find the repetition count from bindings
                let rep_count = self.find_repetition_count(elements, bindings)?;

                let mut result = Vec::new();
                for i in 0..rep_count {
                    if i > 0 {
                        if let Some(sep) = separator {
                            result.push(sep.clone());
                        }
                    }
                    result.extend(self.expand_template_elements(elements, bindings, i)?);
                }

                Ok(result)
            }
            MacroTemplateElement::Group { delimiter, content } => {
                let inner = self.expand_template_elements(content, bindings, rep_index)?;
                Ok(vec![MacroToken::Group(*delimiter, inner)])
            }
        }
    }

    /// Find the repetition count from template elements and bindings
    fn find_repetition_count(
        &self,
        elements: &[MacroTemplateElement],
        bindings: &HashMap<String, Binding>,
    ) -> MacroResult<usize> {
        for element in elements {
            if let MacroTemplateElement::MetaVar(name) = element {
                if let Some(Binding::Repeated(items)) = bindings.get(name) {
                    return Ok(items.len());
                }
            }
        }
        Ok(0)
    }
}

/// Convert macro tokens to a string representation
pub fn tokens_to_string(tokens: &[MacroToken]) -> String {
    let mut result = String::new();
    let mut prev_was_ident = false;

    for token in tokens {
        let needs_space = prev_was_ident && matches!(token, MacroToken::Ident(_) | MacroToken::Literal(_));
        if needs_space {
            result.push(' ');
        }

        match token {
            MacroToken::Ident(s) => {
                result.push_str(s);
                prev_was_ident = true;
            }
            MacroToken::Punct(c) => {
                result.push(*c);
                prev_was_ident = false;
            }
            MacroToken::Literal(lit) => {
                match lit {
                    MacroLiteral::Int(n) => result.push_str(&n.to_string()),
                    MacroLiteral::Float(n) => result.push_str(&n.to_string()),
                    MacroLiteral::String(s) => {
                        result.push('"');
                        result.push_str(s);
                        result.push('"');
                    }
                    MacroLiteral::Bool(b) => result.push_str(if *b { "true" } else { "false" }),
                }
                prev_was_ident = true;
            }
            MacroToken::Group(delim, inner) => {
                match delim {
                    Delimiter::Paren => result.push('('),
                    Delimiter::Bracket => result.push('['),
                    Delimiter::Brace => result.push('{'),
                }
                result.push_str(&tokens_to_string(inner));
                match delim {
                    Delimiter::Paren => result.push(')'),
                    Delimiter::Bracket => result.push(']'),
                    Delimiter::Brace => result.push('}'),
                }
                prev_was_ident = false;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_macro() {
        let mut registry = MacroRegistry::new();

        let macro_def = MacroDef {
            name: Spanned::new("empty".to_string(), Span::new(0, 5)),
            rules: vec![MacroRule {
                pattern: MacroPattern::Empty,
                template: MacroTemplate::Sequence(vec![
                    MacroTemplateElement::Token(MacroToken::Literal(MacroLiteral::Int(42))),
                ]),
            }],
            is_pub: false,
        };

        registry.register(macro_def);

        let expander = MacroExpander::new(&registry);
        let invoke = MacroInvoke {
            name: Spanned::new("empty".to_string(), Span::new(0, 5)),
            delimiter: Delimiter::Paren,
            tokens: vec![],
        };

        let result = expander.expand(&invoke).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(&result[0], MacroToken::Literal(MacroLiteral::Int(42))));
    }

    #[test]
    fn test_simple_substitution() {
        let mut registry = MacroRegistry::new();

        // macro double! { ($x:expr) => { $x + $x } }
        let macro_def = MacroDef {
            name: Spanned::new("double".to_string(), Span::new(0, 6)),
            rules: vec![MacroRule {
                pattern: MacroPattern::Sequence(vec![
                    MacroPatternElement::MetaVar {
                        name: "x".to_string(),
                        kind: MetaVarKind::Expr,
                    },
                ]),
                template: MacroTemplate::Sequence(vec![
                    MacroTemplateElement::MetaVar("x".to_string()),
                    MacroTemplateElement::Token(MacroToken::Punct('+')),
                    MacroTemplateElement::MetaVar("x".to_string()),
                ]),
            }],
            is_pub: false,
        };

        registry.register(macro_def);

        let expander = MacroExpander::new(&registry);
        let invoke = MacroInvoke {
            name: Spanned::new("double".to_string(), Span::new(0, 6)),
            delimiter: Delimiter::Paren,
            tokens: vec![MacroToken::Literal(MacroLiteral::Int(5))],
        };

        let result = expander.expand(&invoke).unwrap();
        assert_eq!(tokens_to_string(&result), "5+5");
    }

    #[test]
    fn test_repetition() {
        let mut registry = MacroRegistry::new();

        // macro vec! { ($($x:expr),*) => { [$($x),*] } }
        let macro_def = MacroDef {
            name: Spanned::new("vec".to_string(), Span::new(0, 3)),
            rules: vec![MacroRule {
                pattern: MacroPattern::Sequence(vec![
                    MacroPatternElement::Repetition {
                        patterns: vec![MacroPatternElement::MetaVar {
                            name: "x".to_string(),
                            kind: MetaVarKind::Expr,
                        }],
                        separator: Some(MacroToken::Punct(',')),
                        kind: RepetitionKind::ZeroOrMore,
                    },
                ]),
                template: MacroTemplate::Sequence(vec![
                    MacroTemplateElement::Group {
                        delimiter: Delimiter::Bracket,
                        content: vec![MacroTemplateElement::Repetition {
                            elements: vec![MacroTemplateElement::MetaVar("x".to_string())],
                            separator: Some(MacroToken::Punct(',')),
                            kind: RepetitionKind::ZeroOrMore,
                        }],
                    },
                ]),
            }],
            is_pub: false,
        };

        registry.register(macro_def);

        let expander = MacroExpander::new(&registry);
        let invoke = MacroInvoke {
            name: Spanned::new("vec".to_string(), Span::new(0, 3)),
            delimiter: Delimiter::Paren,
            tokens: vec![
                MacroToken::Literal(MacroLiteral::Int(1)),
                MacroToken::Punct(','),
                MacroToken::Literal(MacroLiteral::Int(2)),
                MacroToken::Punct(','),
                MacroToken::Literal(MacroLiteral::Int(3)),
            ],
        };

        let result = expander.expand(&invoke).unwrap();
        assert_eq!(tokens_to_string(&result), "[1,2,3]");
    }

    #[test]
    fn test_undefined_macro() {
        let registry = MacroRegistry::new();
        let expander = MacroExpander::new(&registry);

        let invoke = MacroInvoke {
            name: Spanned::new("undefined".to_string(), Span::new(0, 9)),
            delimiter: Delimiter::Paren,
            tokens: vec![],
        };

        let result = expander.expand(&invoke);
        assert!(matches!(result, Err(MacroError::UndefinedMacro(_))));
    }
}
