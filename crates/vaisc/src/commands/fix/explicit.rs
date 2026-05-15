//! `vaisc fix --explicit` command — A4 codemod transformations (Stage 0)
//!
//! This module provides an AST-based codemod engine for the A4 explicit-typing
//! corrections defined in the Vais Master Plan v16.
//!
//! Stage 0 implements:
//!   - CLI plumbing (options, site enum, report type)
//!   - A4-01: detect `<ident>: i64 = <void_expr>` bindings and emit a diagnostic
//!   - A4-02 through A4-09: return `FixError::NotImplemented` (stage 1+)

use colored::Colorize;
use std::fmt;
use std::path::Path;
use vais_ast::*;

// ==================== Public API ====================

/// Options forwarded from the CLI to `run_explicit_fix`.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExplicitFixOptions {
    /// When true, report diagnostics but do not modify the input file.
    pub dry_run: bool,
    /// When `Some(id)`, only the named site is checked (e.g. `"A4-01"`).
    /// When `None`, all implemented sites are checked.
    pub site: Option<String>,
}

/// A single diagnostic finding produced by the codemod.
#[derive(Debug)]
pub struct FixFinding {
    /// The A4 site identifier (e.g. `"A4-01"`).
    pub site_id: String,
    /// 1-based line number of the offending source location.
    pub line: usize,
    /// Human-readable description of the problem.
    pub message: String,
}

/// Aggregate report returned by `run_explicit_fix`.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct ExplicitFixReport {
    /// All findings discovered in this run.
    pub findings: Vec<FixFinding>,
    /// Whether the source file was modified (always `false` in stage 0).
    pub modified: bool,
}

impl ExplicitFixReport {
    /// `true` if at least one finding was detected.
    #[allow(dead_code)]
    pub fn has_findings(&self) -> bool {
        !self.findings.is_empty()
    }
}

/// Errors that `run_explicit_fix` can return.
#[derive(Debug)]
pub enum FixError {
    /// Could not read the source file.
    Io(String),
    /// Source file failed to parse.
    Parse(String),
    /// The requested site is not yet implemented in this stage.
    NotImplemented(String),
}

impl fmt::Display for FixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixError::Io(msg) => write!(f, "I/O error: {}", msg),
            FixError::Parse(msg) => write!(f, "Parse error: {}", msg),
            FixError::NotImplemented(site) => {
                write!(f, "A4 site '{}' is not yet implemented in stage 0", site)
            }
        }
    }
}

// ==================== Site enum ====================

/// The 9 A4 codemod sites from Master Plan v16.
///
/// Each variant corresponds to one `A4-NN` identifier.  Stage 0 fully
/// implements only `A4_01`; the rest return `FixError::NotImplemented`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Site {
    A4_01, // Unit ↔ i64 binding rewrite
    A4_02, // pointer ↔ i64
    A4_03, // auto-deref
    A4_04, // pointer ↔ slice
    A4_05, // array pointer decay
    A4_06, // integer truthy
    A4_07, // numeric widening
    // A4-08 is not listed in the empirical directory — gap in numbering
    A4_09, // lifetime ref erasure
}

impl Site {
    /// Parse an `A4-NN` identifier string into a `Site` variant.
    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "A4-01" => Some(Site::A4_01),
            "A4-02" => Some(Site::A4_02),
            "A4-03" => Some(Site::A4_03),
            "A4-04" => Some(Site::A4_04),
            "A4-05" => Some(Site::A4_05),
            "A4-06" => Some(Site::A4_06),
            "A4-07" => Some(Site::A4_07),
            "A4-09" => Some(Site::A4_09),
            _ => None,
        }
    }

    /// The canonical string identifier for this site.
    #[allow(dead_code)]
    pub fn id(&self) -> &'static str {
        match self {
            Site::A4_01 => "A4-01",
            Site::A4_02 => "A4-02",
            Site::A4_03 => "A4-03",
            Site::A4_04 => "A4-04",
            Site::A4_05 => "A4-05",
            Site::A4_06 => "A4-06",
            Site::A4_07 => "A4-07",
            Site::A4_09 => "A4-09",
        }
    }

    /// `true` for sites that are implemented in stage 0.
    pub fn is_implemented(&self) -> bool {
        matches!(self, Site::A4_01)
    }
}

// ==================== Entry point ====================

/// Run the explicit-fix codemod on `input` and return a report.
///
/// The function never modifies the source file in stage 0 — all findings are
/// diagnostic-only. The caller is responsible for printing the report and
/// deciding the process exit code.
pub fn run_explicit_fix(
    input: &Path,
    opts: ExplicitFixOptions,
) -> Result<ExplicitFixReport, FixError> {
    // Resolve the requested site(s).
    let sites_to_check: Vec<Site> = if let Some(ref site_id) = opts.site {
        let site = Site::parse(site_id).ok_or_else(|| {
            FixError::NotImplemented(format!(
                "{} (unknown site identifier; valid: A4-01..A4-07, A4-09)",
                site_id
            ))
        })?;
        if !site.is_implemented() {
            return Err(FixError::NotImplemented(site_id.clone()));
        }
        vec![site]
    } else {
        // All sites; unimplemented ones are silently skipped in the scan loop.
        vec![
            Site::A4_01,
            Site::A4_02,
            Site::A4_03,
            Site::A4_04,
            Site::A4_05,
            Site::A4_06,
            Site::A4_07,
            Site::A4_09,
        ]
    };

    let source = std::fs::read_to_string(input)
        .map_err(|e| FixError::Io(format!("cannot read '{}': {}", input.display(), e)))?;

    let ast = vais_parser::parse(&source)
        .map_err(|e| FixError::Parse(format!("parse error in '{}': {}", input.display(), e)))?;

    let mut report = ExplicitFixReport::default();

    for site in &sites_to_check {
        if !site.is_implemented() {
            // Skip unimplemented sites when running in "all" mode.
            continue;
        }
        match site {
            Site::A4_01 => {
                let findings = check_a4_01(&ast, &source);
                report.findings.extend(findings);
            }
            _ => {
                // Unreachable in stage 0 because is_implemented() guards above.
            }
        }
    }

    Ok(report)
}

/// Print a report to stdout/stderr and return the appropriate process exit code.
///
/// Returns `1` if any findings are present, `0` otherwise.
pub fn print_report_and_exit_code(input: &Path, report: &ExplicitFixReport, dry_run: bool) -> i32 {
    if report.findings.is_empty() {
        println!("{} No A4 issues found in {}", "OK".green(), input.display());
        return 0;
    }

    let mode_label = if dry_run { " (dry-run)" } else { "" };
    eprintln!(
        "{}{} {} A4 issue(s) detected in {}:",
        "error".red().bold(),
        mode_label,
        report.findings.len(),
        input.display()
    );

    for finding in &report.findings {
        eprintln!(
            "  [{site}] line {line}: {msg}",
            site = finding.site_id.yellow(),
            line = finding.line,
            msg = finding.message,
        );
    }

    if dry_run {
        eprintln!("(dry-run: source file not modified)");
    }

    1
}

// ==================== A4-01 checker ====================

/// Detect `<ident>: i64 = <expr>` bindings where the type annotation is `i64`
/// but the right-hand expression is a call to a void-returning function.
///
/// Stage 0 heuristic: any call expression whose callee resolves to a function
/// with an explicit `Unit` (no return type) or an implicit void return is
/// flagged.  We cannot run the full type checker inside the codemod at this
/// stage, so we use the AST-level heuristic: the RHS is a bare `Call` or
/// `MethodCall` expression (not nested in arithmetic, etc.).  The type checker
/// already lets this through (`exit_code 96` per the empirical fixture), so the
/// codemod provides an early warning layer.
fn check_a4_01(module: &Module, source: &str) -> Vec<FixFinding> {
    // Build a set of function names that have no return type annotation (void).
    let void_fns: std::collections::HashSet<String> = module
        .items
        .iter()
        .filter_map(|item| {
            if let Item::Function(func) = &item.node {
                if func.ret_type.is_none() {
                    Some(func.name.node.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let mut findings = Vec::new();

    for item in &module.items {
        let func = match &item.node {
            Item::Function(f) => f,
            _ => continue,
        };

        let stmts = match &func.body {
            FunctionBody::Block(stmts) => stmts,
            FunctionBody::Expr(_) => continue,
        };

        for stmt in stmts {
            if let Stmt::Let {
                name: _, ty, value, ..
            } = &stmt.node
            {
                // Only flag bindings with an explicit `i64` type annotation.
                let is_i64_binding = match ty {
                    Some(t) => {
                        matches!(&t.node, Type::Named { name, generics } if name == "i64" && generics.is_empty())
                    }
                    None => false,
                };
                if !is_i64_binding {
                    continue;
                }

                // Check if the RHS is a call to a known void function.
                if let Some(callee_name) = extract_simple_call_name(&value.node) {
                    if void_fns.contains(callee_name) {
                        let line = byte_offset_to_line(source, stmt.span.start);
                        findings.push(FixFinding {
                            site_id: "A4-01".to_string(),
                            line,
                            message: format!(
                                "binding '{name}' is declared as i64 but '{callee}()' returns Unit (void); \
                                 the binding is meaningless. Either remove the binding or make \
                                 '{callee}()' return i64 explicitly.",
                                name = extract_let_name(&stmt.node),
                                callee = callee_name,
                            ),
                        });
                    }
                }
            }
        }
    }

    findings
}

/// If `expr` is a simple function call (`Expr::Call` with an `Ident` callee),
/// return the callee name.  Returns `None` for method calls, nested expressions,
/// etc.
fn extract_simple_call_name(expr: &Expr) -> Option<&str> {
    if let Expr::Call { func, args: _ } = expr {
        if let Expr::Ident(name) = &func.node {
            return Some(name.as_str());
        }
    }
    None
}

/// Extract the binding name from a `Stmt::Let` node (best-effort, returns `"_"` on mismatch).
fn extract_let_name(stmt: &Stmt) -> &str {
    if let Stmt::Let { name, .. } = stmt {
        name.node.as_str()
    } else {
        "_"
    }
}

/// Convert a byte offset in `source` to a 1-based line number.
fn byte_offset_to_line(source: &str, offset: usize) -> usize {
    let safe_offset = offset.min(source.len());
    source.as_bytes()[..safe_offset]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
        + 1
}

// ==================== Unit tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_parse_known() {
        assert_eq!(Site::parse("A4-01"), Some(Site::A4_01));
        assert_eq!(Site::parse("A4-07"), Some(Site::A4_07));
        assert_eq!(Site::parse("A4-09"), Some(Site::A4_09));
    }

    #[test]
    fn site_parse_unknown() {
        assert_eq!(Site::parse("A4-08"), None);
        assert_eq!(Site::parse("X1-01"), None);
        assert_eq!(Site::parse(""), None);
    }

    #[test]
    fn site_implemented_only_a4_01() {
        assert!(Site::A4_01.is_implemented());
        assert!(!Site::A4_02.is_implemented());
        assert!(!Site::A4_09.is_implemented());
    }

    #[test]
    fn a4_01_detects_void_binding() {
        let source = r#"fn void_fn() {
    R
}

fn main() -> i64 {
    x: i64 = void_fn()
    return x
}
"#;
        let ast = vais_parser::parse(source).expect("parse failed");
        let findings = check_a4_01(&ast, source);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].site_id, "A4-01");
        assert!(findings[0].message.contains("A4-01") || findings[0].message.contains("void_fn"));
        // line 6 in the fixture above
        assert_eq!(findings[0].line, 6);
    }

    #[test]
    fn a4_01_no_finding_for_i64_returning_fn() {
        let source = r#"fn returns_i64() -> i64 {
    42
}

fn main() -> i64 {
    x: i64 = returns_i64()
    return x
}
"#;
        let ast = vais_parser::parse(source).expect("parse failed");
        let findings = check_a4_01(&ast, source);
        assert!(
            findings.is_empty(),
            "should not flag a binding to an i64-returning function"
        );
    }

    #[test]
    fn a4_01_no_finding_without_type_annotation() {
        let source = r#"fn void_fn() {
    R
}

fn main() -> i64 {
    x := void_fn()
    0
}
"#;
        let ast = vais_parser::parse(source).expect("parse failed");
        let findings = check_a4_01(&ast, source);
        assert!(
            findings.is_empty(),
            "should not flag bindings without an explicit i64 annotation"
        );
    }

    #[test]
    fn byte_offset_to_line_basic() {
        let source = "line1\nline2\nline3";
        assert_eq!(byte_offset_to_line(source, 0), 1);
        assert_eq!(byte_offset_to_line(source, 6), 2);
        assert_eq!(byte_offset_to_line(source, 12), 3);
    }

    #[test]
    fn explicit_fix_report_has_findings() {
        let report = ExplicitFixReport {
            findings: vec![FixFinding {
                site_id: "A4-01".to_string(),
                line: 6,
                message: "test".to_string(),
            }],
            modified: false,
        };
        assert!(report.has_findings());
    }

    #[test]
    fn explicit_fix_report_empty() {
        let report = ExplicitFixReport::default();
        assert!(!report.has_findings());
    }

    #[test]
    fn fix_error_display_not_implemented() {
        let err = FixError::NotImplemented("A4-02".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("A4-02"));
        assert!(msg.contains("stage 0"));
    }
}
