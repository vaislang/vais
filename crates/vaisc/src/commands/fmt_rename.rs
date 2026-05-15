//! `vaisc fmt --rename-keyword-collisions` — Step 15 stage 0 codemod.
//!
//! Step 15 (I-4 dual syntax) was BLOCKED in master-plan v23 because the
//! naive lexer change to add multi-character keywords (`fn`, `struct`,
//! `return`, `match`, `else`, ...) collided with existing identifiers
//! across `compiler/std/`, `lang/packages/vaisdb/`, and `vais-server`.
//! Examples observed in the prior attempt:
//!
//! - `fn_handler`, `match_arm`, `return_value`, `struct_size` — Logos
//!   priority swallowed the keyword prefix, breaking the parser.
//!
//! This module implements the prerequisite codemod identified in the
//! Step 15 status text: an AST-aware identifier scan that lists (and,
//! optionally, renames) sites where a future multi-char keyword would
//! collide with existing user code.
//!
//! Stage 0 deliverable (this commit): scan-only mode. Detects identifiers
//! whose name *starts with* a candidate keyword followed by `_` or an
//! ASCII letter (so `match` → `match_arm` triggers, but `matched` does
//! NOT trigger because `matched` is the existing identifier and not a
//! keyword-prefix collision per Logos longest-match rules).
//!
//! Stage 1+ (future): apply the rename via AST rewrite + cross-module
//! reference update. That step is intentionally out of scope here.

use colored::Colorize;
use std::path::Path;
use vais_ast::*;

/// Candidate multi-character keywords that Step 15 wants to add to the
/// lexer. The list mirrors the Step 15 status note in master-plan.toml
/// (`fn / struct / enum / else / match / return / mod / use / type /
/// pub / impl / trait / const`). Order is alphabetical (sorted helper
/// per LESSONS L-007).
pub const STEP15_CANDIDATE_KEYWORDS: &[&str] = &[
    "const", "else", "enum", "fn", "impl", "match", "mod", "pub", "return", "struct", "trait",
    "type", "use",
];

/// Options forwarded from the CLI.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RenameOptions {
    /// Report only; do not modify files.
    pub dry_run: bool,
    /// Limit scan to a single keyword. `None` scans all candidates.
    pub keyword: Option<String>,
    /// Rename prefix for stage 1+. Default `_`.
    #[allow(dead_code)]
    pub rename_prefix: String,
}

impl Default for RenameOptions {
    fn default() -> Self {
        Self {
            dry_run: true,
            keyword: None,
            rename_prefix: "_".to_string(),
        }
    }
}

/// A single collision finding.
#[derive(Debug, Clone)]
pub struct CollisionFinding {
    /// The candidate keyword that collides (e.g. `"fn"`).
    pub keyword: String,
    /// The colliding identifier (e.g. `"fn_handler"`).
    pub identifier: String,
    /// 1-based line number.
    pub line: usize,
    /// Site kind: function name, parameter, struct field, etc.
    pub kind: CollisionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollisionKind {
    FunctionName,
    Parameter,
    StructName,
    StructField,
    EnumName,
    EnumVariant,
    TraitName,
    TypeAlias,
}

impl CollisionKind {
    fn as_str(&self) -> &'static str {
        match self {
            CollisionKind::FunctionName => "function name",
            CollisionKind::Parameter => "parameter",
            CollisionKind::StructName => "struct name",
            CollisionKind::StructField => "struct field",
            CollisionKind::EnumName => "enum name",
            CollisionKind::EnumVariant => "enum variant",
            CollisionKind::TraitName => "trait name",
            CollisionKind::TypeAlias => "type alias",
        }
    }
}

/// Aggregate report.
#[derive(Debug, Default)]
pub struct RenameReport {
    pub findings: Vec<CollisionFinding>,
    /// Always `false` in stage 0 (dry-run only).
    #[allow(dead_code)]
    pub modified: bool,
}

/// Returns true if `ident` starts with `kw` followed by `_` or an ASCII
/// letter — i.e. `ident` would still tokenize as `Token::Ident` under
/// Logos longest-match, but a naive lexer change to make `kw` a keyword
/// would break it.
///
/// Examples (`kw = "fn"`):
/// - `"fn_handler"` → true (collides)
/// - `"fnHandler"` → true (collides)
/// - `"fn"` → false (not a collision; this is exactly the keyword and
///   would be intentionally consumed)
/// - `"function"` → true (collides — `fn` prefix + alpha continuation;
///   note: this is a false positive for keywords whose Vais form uses
///   the long word, mitigated by the kw list itself being short forms)
/// - `"matched"` → true (matches `match` prefix)
pub fn is_collision(ident: &str, kw: &str) -> bool {
    if !ident.starts_with(kw) || ident.len() <= kw.len() {
        return false;
    }
    let next = ident.as_bytes()[kw.len()];
    next == b'_' || next.is_ascii_alphabetic()
}

/// Scan an AST module for collisions with the given keyword set.
pub fn scan_module(module: &Module, keywords: &[&str]) -> Vec<CollisionFinding> {
    let mut findings = Vec::new();

    for item in &module.items {
        match &item.node {
            Item::Function(func) => {
                check_ident(
                    &func.name,
                    CollisionKind::FunctionName,
                    keywords,
                    &mut findings,
                );
                for param in &func.params {
                    check_ident(
                        &param.name,
                        CollisionKind::Parameter,
                        keywords,
                        &mut findings,
                    );
                }
            }
            Item::Struct(s) => {
                check_ident(&s.name, CollisionKind::StructName, keywords, &mut findings);
                for field in &s.fields {
                    check_ident(
                        &field.name,
                        CollisionKind::StructField,
                        keywords,
                        &mut findings,
                    );
                }
            }
            Item::Enum(e) => {
                check_ident(&e.name, CollisionKind::EnumName, keywords, &mut findings);
                for variant in &e.variants {
                    check_ident(
                        &variant.name,
                        CollisionKind::EnumVariant,
                        keywords,
                        &mut findings,
                    );
                }
            }
            Item::Trait(t) => {
                check_ident(&t.name, CollisionKind::TraitName, keywords, &mut findings);
            }
            Item::TypeAlias(t) => {
                check_ident(&t.name, CollisionKind::TypeAlias, keywords, &mut findings);
            }
            _ => {}
        }
    }

    findings
}

fn check_ident(
    name: &Spanned<String>,
    kind: CollisionKind,
    keywords: &[&str],
    out: &mut Vec<CollisionFinding>,
) {
    for kw in keywords {
        if is_collision(&name.node, kw) {
            out.push(CollisionFinding {
                keyword: kw.to_string(),
                identifier: name.node.clone(),
                // Line number computed by caller from source.
                line: 0,
                kind: kind.clone(),
            });
        }
    }
}

/// Run the rename codemod against a single source file.
pub fn run_rename(input: &Path, options: &RenameOptions) -> Result<RenameReport, String> {
    let source =
        std::fs::read_to_string(input).map_err(|e| format!("read {}: {}", input.display(), e))?;

    let module = vais_parser::parse(&source).map_err(|e| format!("parse: {:?}", e))?;

    let keywords: Vec<&str> = match &options.keyword {
        Some(k) => {
            if !STEP15_CANDIDATE_KEYWORDS.contains(&k.as_str()) {
                return Err(format!(
                    "unknown keyword '{}'; valid: {}",
                    k,
                    STEP15_CANDIDATE_KEYWORDS.join(", ")
                ));
            }
            vec![k.as_str()]
        }
        None => STEP15_CANDIDATE_KEYWORDS.to_vec(),
    };

    let mut findings = scan_module(&module, &keywords);

    // Resolve line numbers from token spans (best-effort; line=0 is the
    // initial value from check_ident).
    for f in &mut findings {
        // Find the first token whose lexeme equals the identifier.
        // The AST loses token positions for non-Spanned fields, so this
        // is a best-effort linear scan.
        if let Some(line) = source.lines().enumerate().find_map(|(i, line)| {
            if line.contains(&f.identifier) {
                Some(i + 1)
            } else {
                None
            }
        }) {
            f.line = line;
        }
    }

    let modified = false; // stage 0 is dry-run only
    Ok(RenameReport { findings, modified })
}

/// CLI entry point.
pub fn cmd_fmt_rename_keywords(input: &Path, options: &RenameOptions) -> Result<(), String> {
    let report = run_rename(input, options)?;

    if report.findings.is_empty() {
        println!(
            "{} No keyword collisions found in {}",
            "OK".green().bold(),
            input.display()
        );
        return Ok(());
    }

    println!(
        "{} {} collision(s) in {}:",
        "FOUND".yellow().bold(),
        report.findings.len(),
        input.display()
    );
    for f in &report.findings {
        println!(
            "  line {:>4}  {:<14}  {} (would collide with proposed `{}` keyword)",
            f.line,
            f.kind.as_str(),
            f.identifier,
            f.keyword
        );
    }

    if options.dry_run {
        println!(
            "\n{}: this is a dry-run report. Stage 1+ will apply renames.",
            "NOTE".cyan().bold()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_collision_basic() {
        assert!(is_collision("fn_handler", "fn"));
        assert!(is_collision("fnHandler", "fn"));
        assert!(is_collision("matched", "match"));
        assert!(is_collision("returns", "return"));
        assert!(!is_collision("fn", "fn"));
        assert!(!is_collision("foo", "fn"));
        assert!(!is_collision("f", "fn"));
        // digits do not extend the keyword (Logos kw boundary)
        assert!(!is_collision("fn1", "fn"));
    }

    #[test]
    fn candidate_list_sorted() {
        // L-007: deterministic ordering of metadata layouts.
        let mut sorted = STEP15_CANDIDATE_KEYWORDS.to_vec();
        sorted.sort();
        assert_eq!(sorted, STEP15_CANDIDATE_KEYWORDS);
    }
}
