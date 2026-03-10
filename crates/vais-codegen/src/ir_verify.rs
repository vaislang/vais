//! LLVM IR text verification gate.
//!
//! Lightweight structural checks on text-mode LLVM IR before passing it to clang.
//! Catches common codegen bugs early with clear diagnostic messages instead of
//! opaque clang errors.
//!
//! # Checks Performed
//!
//! - **Unterminated basic blocks**: Every block must end with a terminator
//!   (`ret`, `br`, `switch`, `unreachable`, `resume`, `invoke`).
//! - **Phi placement**: `phi` instructions must appear at the start of a block
//!   (before any non-phi instructions).
//! - **Void phi**: `phi void` is invalid in LLVM IR.
//! - **Duplicate function definitions**: Two `define` for the same function name.
//! - **Mismatched braces**: Unbalanced `{` / `}` in function bodies.

use crate::CodegenResult;

/// A single verification diagnostic.
#[derive(Debug)]
pub struct IrDiagnostic {
    /// 1-based line number in the IR text.
    pub line: usize,
    /// Severity level.
    pub severity: DiagnosticSeverity,
    /// Human-readable message.
    pub message: String,
    /// Name of the enclosing function (if applicable).
    pub function_name: Option<String>,
}

/// Severity of an IR diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    /// Will definitely cause clang/LLVM to reject the IR.
    Error,
    /// Likely indicates a codegen bug but may not crash clang.
    Warning,
}

impl std::fmt::Display for IrDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tag = match self.severity {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
        };
        if let Some(ref fn_name) = self.function_name {
            write!(
                f,
                "IR {}:{} (in @{}): {}",
                tag, self.line, fn_name, self.message
            )
        } else {
            write!(f, "IR {}:{}: {}", tag, self.line, self.message)
        }
    }
}

/// Verify text-mode LLVM IR and collect diagnostics.
///
/// Returns a list of diagnostics. An empty list means the IR passed all checks.
pub fn verify_text_ir(ir: &str) -> Vec<IrDiagnostic> {
    let mut diagnostics = Vec::new();
    let lines: Vec<&str> = ir.lines().collect();

    let mut in_function = false;
    let mut current_function_name: Option<String> = None;
    let mut block_has_terminator = false;
    let mut current_block_start_line = 0usize;
    let mut brace_depth: i32 = 0;
    let mut defined_functions = std::collections::HashSet::new();
    let mut seen_non_phi_in_block = false;

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }

        // Track braces (before any continues so we never miss a brace).
        // Skip characters inside string constants (c"..." or "...") to avoid
        // counting braces that appear inside literal strings.
        let mut in_string = false;
        let mut prev_ch = '\0';
        for ch in trimmed.chars() {
            if ch == '"' && prev_ch != '\\' {
                in_string = !in_string;
            } else if !in_string {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => brace_depth -= 1,
                    _ => {}
                }
            }
            prev_ch = ch;
        }

        // Track function definitions
        if trimmed.starts_with("define ") {
            in_function = true;
            // The first block in a function is implicitly started by the opening brace.
            // We wait for the first label (e.g., "entry:") to begin block tracking.
            // Mark as terminated so we don't false-positive on the define->label transition.
            block_has_terminator = true;
            current_block_start_line = 0;
            seen_non_phi_in_block = false;

            // Extract function name for duplicate detection and diagnostic context
            current_function_name = extract_define_name(trimmed).map(|s| s.to_string());
            if let Some(ref name) = current_function_name {
                if !defined_functions.insert(name.clone()) {
                    diagnostics.push(IrDiagnostic {
                        line: line_num,
                        severity: DiagnosticSeverity::Error,
                        message: format!("duplicate function definition: @{}", name),
                        function_name: current_function_name.clone(),
                    });
                }
            }
            continue;
        }

        // End of function
        if trimmed == "}" && in_function {
            // Check that the last block was terminated
            if !block_has_terminator && current_block_start_line > 0 {
                diagnostics.push(IrDiagnostic {
                    line: line_num,
                    severity: DiagnosticSeverity::Error,
                    message: format!(
                        "unterminated basic block (block started at line {})",
                        current_block_start_line
                    ),
                    function_name: current_function_name.clone(),
                });
            }
            in_function = false;
            current_function_name = None;
            continue;
        }

        if !in_function {
            continue;
        }

        // Block label (e.g., "entry:" or "label42:")
        if !trimmed.starts_with('%')
            && !trimmed.starts_with('@')
            && !trimmed.starts_with("define")
            && trimmed.ends_with(':')
            && !trimmed.contains('=')
        {
            // New block: check previous block was terminated
            if !block_has_terminator && current_block_start_line > 0 {
                diagnostics.push(IrDiagnostic {
                    line: line_num,
                    severity: DiagnosticSeverity::Error,
                    message: format!(
                        "unterminated basic block before label '{}' (block started at line {})",
                        trimmed.trim_end_matches(':'),
                        current_block_start_line
                    ),
                    function_name: current_function_name.clone(),
                });
            }
            block_has_terminator = false;
            current_block_start_line = line_num;
            seen_non_phi_in_block = false;
            continue;
        }

        // Check for phi void (invalid LLVM IR)
        if trimmed.contains("phi void") {
            diagnostics.push(IrDiagnostic {
                line: line_num,
                severity: DiagnosticSeverity::Error,
                message: "invalid `phi void` instruction (void is not a first-class type)"
                    .to_string(),
                function_name: current_function_name.clone(),
            });
        }

        // Check phi placement (must be before non-phi instructions)
        if trimmed.contains(" = phi ") {
            if seen_non_phi_in_block {
                diagnostics.push(IrDiagnostic {
                    line: line_num,
                    severity: DiagnosticSeverity::Error,
                    message: "phi instruction after non-phi instruction in basic block".to_string(),
                    function_name: current_function_name.clone(),
                });
            }
        } else if !trimmed.ends_with(':') && !trimmed.is_empty() {
            // Any non-phi, non-label instruction marks non-phi zone
            seen_non_phi_in_block = true;
        }

        // Track terminators
        if is_terminator(trimmed) {
            block_has_terminator = true;
        }
    }

    // Check for mismatched braces at end
    if brace_depth != 0 {
        diagnostics.push(IrDiagnostic {
            line: lines.len(),
            severity: DiagnosticSeverity::Warning,
            message: format!(
                "mismatched braces in IR (depth {} at EOF, expected 0)",
                brace_depth
            ),
            function_name: None,
        });
    }

    // Check for undefined label references
    check_undefined_labels(&lines, &mut diagnostics);

    // Check for return type mismatches
    check_return_type_consistency(&lines, &mut diagnostics);

    diagnostics
}

/// Check that all `br label %X` targets reference labels that exist in the same function.
fn check_undefined_labels(lines: &[&str], diagnostics: &mut Vec<IrDiagnostic>) {
    let mut in_function = false;
    let mut fn_start_line = 0usize;
    let mut current_fn_name: Option<String> = None;
    let mut labels_defined = std::collections::HashSet::new();
    let mut label_refs: Vec<(usize, String)> = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("define ") {
            in_function = true;
            fn_start_line = i + 1;
            current_fn_name = extract_define_name(trimmed).map(|s| s.to_string());
            labels_defined.clear();
            label_refs.clear();
            continue;
        }

        if trimmed == "}" && in_function {
            // End of function: check all label refs
            for (ref_line, label) in &label_refs {
                if !labels_defined.contains(label.as_str()) {
                    diagnostics.push(IrDiagnostic {
                        line: *ref_line,
                        severity: DiagnosticSeverity::Warning,
                        message: format!(
                            "branch references undefined label '%{}' (function at line {})",
                            label, fn_start_line
                        ),
                        function_name: current_fn_name.clone(),
                    });
                }
            }
            in_function = false;
            current_fn_name = None;
            continue;
        }

        if !in_function {
            continue;
        }

        // Collect label definitions
        if !trimmed.starts_with('%')
            && !trimmed.starts_with('@')
            && trimmed.ends_with(':')
            && !trimmed.contains('=')
        {
            labels_defined.insert(trimmed.trim_end_matches(':').to_string());
            continue;
        }

        // Collect label references from br/switch instructions
        // Pattern: "label %name" (may appear multiple times)
        let mut search = trimmed;
        while let Some(pos) = search.find("label %") {
            let after = &search[pos + 7..];
            let end = after
                .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
                .unwrap_or(after.len());
            if end > 0 {
                label_refs.push((i + 1, after[..end].to_string()));
            }
            search = &search[pos + 7 + end..];
        }
    }
}

/// Check that return instructions match the declared function return type.
fn check_return_type_consistency(lines: &[&str], diagnostics: &mut Vec<IrDiagnostic>) {
    let mut current_ret_type: Option<String> = None;
    let mut current_fn_name: Option<String> = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if let Some(after_define) = trimmed.strip_prefix("define ") {
            // Extract return type: the last whitespace-delimited token before `@`.
            // LLVM IR `define` lines may include linkage, visibility, calling
            // convention, and parameter attributes before the return type, e.g.:
            //   define dso_local fastcc i64 @name(...)
            // The return type is always the token immediately before `@`.
            if let Some(at_pos) = after_define.find('@') {
                let prefix = after_define[..at_pos].trim();
                // Take last whitespace-delimited token as the return type
                let ret_type = prefix
                    .rsplit_once(char::is_whitespace)
                    .map_or(prefix, |(_, last)| last);
                current_ret_type = Some(ret_type.to_string());
            }
            current_fn_name = extract_define_name(trimmed).map(|s| s.to_string());
            continue;
        }

        if trimmed == "}" {
            current_ret_type = None;
            current_fn_name = None;
            continue;
        }

        // Check ret instructions
        if let Some(ret_part) = trimmed.strip_prefix("ret ") {
            if let Some(ref expected) = current_ret_type {
                let ret_type = if ret_part == "void" {
                    "void".to_string()
                } else if let Some(space) = ret_part.find(' ') {
                    ret_part[..space].to_string()
                } else {
                    ret_part.to_string()
                };

                if !expected.is_empty()
                    && ret_type != *expected
                    && *expected != "void"
                    && ret_type != "void"
                {
                    diagnostics.push(IrDiagnostic {
                        line: i + 1,
                        severity: DiagnosticSeverity::Warning,
                        message: format!(
                            "return type mismatch: function declares '{}' but returns '{}'",
                            expected, ret_type
                        ),
                        function_name: current_fn_name.clone(),
                    });
                }
            }
        }
    }
}

/// Verify text IR and return an error if any Error-level diagnostics are found.
///
/// This is the main entry point for the verification gate. Call it after
/// `generate_module()` and before writing the IR to a file.
pub fn verify_text_ir_or_error(ir: &str) -> CodegenResult<()> {
    let diagnostics = verify_text_ir(ir);

    // In debug builds, assert that no Warning-level diagnostics exist.
    #[cfg(debug_assertions)]
    {
        let warnings: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Warning)
            .collect();
        if !warnings.is_empty() {
            eprintln!("[IR verify] {} warning(s) (non-fatal)", warnings.len());
        }
    }

    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();

    if errors.is_empty() {
        return Ok(());
    }

    // Build a combined error message
    let mut msg = format!("LLVM IR verification found {} error(s):\n", errors.len());
    for (i, diag) in errors.iter().enumerate().take(10) {
        msg.push_str(&format!("  {}. {}\n", i + 1, diag));
    }
    if errors.len() > 10 {
        msg.push_str(&format!("  ... and {} more\n", errors.len() - 10));
    }

    Err(crate::CodegenError::LlvmError(msg))
}

/// Check if an instruction is a basic block terminator.
fn is_terminator(trimmed: &str) -> bool {
    trimmed.starts_with("ret ")
        || trimmed == "ret void"
        || trimmed.starts_with("br ")
        || trimmed.starts_with("switch ")
        || trimmed == "unreachable"
        || trimmed.starts_with("resume ")
        || trimmed.starts_with("invoke ")
        || trimmed.starts_with("indirectbr ")
        || trimmed.starts_with("callbr ")
}

/// Extract function name from a `define` line.
/// Example: `define i64 @main() {` -> `main`
fn extract_define_name(line: &str) -> Option<&str> {
    let at_pos = line.find('@')?;
    let after_at = &line[at_pos + 1..];
    let end = after_at.find('(')?;
    Some(&after_at[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ir() {
        let ir = r#"
define i64 @main() {
entry:
  ret i64 0
}
"#;
        let diags = verify_text_ir(ir);
        assert!(
            diags.is_empty(),
            "Expected no diagnostics, got: {:?}",
            diags
        );
    }

    #[test]
    fn test_unterminated_block() {
        let ir = r#"
define i64 @main() {
entry:
  %x = add i64 1, 2
}
"#;
        let diags = verify_text_ir(ir);
        assert!(
            diags.iter().any(|d| d.message.contains("unterminated")),
            "Expected unterminated block diagnostic, got: {:?}",
            diags
        );
    }

    #[test]
    fn test_phi_void_detected() {
        let ir = r#"
define i64 @main() {
entry:
  br label %merge
merge:
  %x = phi void [ 0, %entry ]
  ret i64 0
}
"#;
        let diags = verify_text_ir(ir);
        assert!(
            diags.iter().any(|d| d.message.contains("phi void")),
            "Expected phi void diagnostic, got: {:?}",
            diags
        );
    }

    #[test]
    fn test_duplicate_function() {
        let ir = r#"
define i64 @foo() {
entry:
  ret i64 0
}

define i64 @foo() {
entry:
  ret i64 1
}
"#;
        let diags = verify_text_ir(ir);
        assert!(
            diags.iter().any(|d| d.message.contains("duplicate")),
            "Expected duplicate function diagnostic, got: {:?}",
            diags
        );
    }

    #[test]
    fn test_multiple_blocks_all_terminated() {
        let ir = r#"
define i64 @main() {
entry:
  %cond = icmp eq i64 1, 1
  br i1 %cond, label %then, label %else
then:
  br label %merge
else:
  br label %merge
merge:
  ret i64 0
}
"#;
        let diags = verify_text_ir(ir);
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .collect();
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_verify_or_error_ok() {
        let ir = "define i64 @main() {\nentry:\n  ret i64 0\n}\n";
        assert!(verify_text_ir_or_error(ir).is_ok());
    }

    #[test]
    fn test_verify_or_error_fails() {
        let ir = "define i64 @main() {\nentry:\n  %x = add i64 1, 2\n}\n";
        assert!(verify_text_ir_or_error(ir).is_err());
    }

    #[test]
    fn test_undefined_label_detected() {
        let ir = r#"
define i64 @main() {
entry:
  br label %nonexistent
}
"#;
        let diags = verify_text_ir(ir);
        assert!(
            diags.iter().any(|d| d.message.contains("undefined label")),
            "Expected undefined label diagnostic, got: {:?}",
            diags
        );
    }

    #[test]
    fn test_valid_label_refs_no_warning() {
        let ir = r#"
define i64 @main() {
entry:
  br label %done
done:
  ret i64 0
}
"#;
        let diags = verify_text_ir(ir);
        let label_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("undefined label"))
            .collect();
        assert!(
            label_diags.is_empty(),
            "Expected no label warnings, got: {:?}",
            label_diags
        );
    }

    #[test]
    fn test_return_type_mismatch() {
        let ir = r#"
define i64 @foo() {
entry:
  ret i32 42
}
"#;
        let diags = verify_text_ir(ir);
        assert!(
            diags
                .iter()
                .any(|d| d.message.contains("return type mismatch")),
            "Expected return type mismatch, got: {:?}",
            diags
        );
    }

    #[test]
    fn test_return_type_matches_ok() {
        let ir = r#"
define i64 @foo() {
entry:
  ret i64 42
}
"#;
        let diags = verify_text_ir(ir);
        let ret_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("return type mismatch"))
            .collect();
        assert!(
            ret_diags.is_empty(),
            "Expected no return type warnings, got: {:?}",
            ret_diags
        );
    }

    #[test]
    fn test_return_type_with_linkage_modifiers() {
        // LLVM IR with linkage, visibility, and calling convention before return type
        let ir = r#"
define dso_local fastcc i64 @foo() {
entry:
  ret i64 42
}
"#;
        let diags = verify_text_ir(ir);
        let ret_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("return type mismatch"))
            .collect();
        assert!(
            ret_diags.is_empty(),
            "Should not report mismatch for linkage-prefixed define, got: {:?}",
            ret_diags
        );
    }

    #[test]
    fn test_brace_in_string_constant() {
        // Braces inside string constants should not affect brace counting
        let ir = r#"
@.str = private constant [12 x i8] c"hello {}\0a\00"
define i64 @main() {
entry:
  ret i64 0
}
"#;
        let diags = verify_text_ir(ir);
        let brace_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.message.contains("mismatched braces"))
            .collect();
        assert!(
            brace_diags.is_empty(),
            "Should not report mismatched braces from string constants, got: {:?}",
            brace_diags
        );
    }
}
