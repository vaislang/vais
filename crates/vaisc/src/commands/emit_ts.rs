//! `vaisc emit-ts` command — TypeScript declaration file emitter (Stage 0)
//!
//! Emits a `.d.ts` file from a `.vais` schema source.
//!
//! Stage 0 scope:
//!   - CLI plumbing (options, report type, error type)
//!   - Reads input, runs vaisc parser + type checker
//!   - Emits a TS `interface` for each top-level `pub struct` with primitive fields
//!   - Primitive type lowering table: i8/i16/i32/i64/i128/u8-u128 → `number`,
//!     f32/f64 → `number`, bool → `boolean`, str/String → `string`
//!   - Any struct field whose type is NOT in the primitive table: emits EMIT_TS_999
//!   - Non-pub structs: skipped silently
//!   - Non-struct top-level items: skipped silently
//!
//! Stages 1+ will add full lowering table, EMIT_TS_001-021 specific error codes,
//! exhaustiveness tests, and composite-type support.

use colored::Colorize;
use std::fmt;
use std::path::{Path, PathBuf};
use vais_ast::{Item, Type};

// ==================== Public API ====================

/// Options forwarded from the CLI to `run_emit_ts`.
#[derive(Debug, Clone)]
pub struct EmitTsOptions {
    /// Input `.vais` file to read.
    pub input: PathBuf,
    /// Output `.d.ts` file to write.
    pub output: PathBuf,
}

/// A single TypeScript interface declaration produced by the emitter.
#[derive(Debug)]
pub struct TsInterface {
    /// The struct name (used as the TS interface name).
    pub name: String,
    /// The rendered lines of the interface body (field declarations).
    pub fields: Vec<TsField>,
}

/// A single field in a TypeScript interface.
#[derive(Debug)]
pub struct TsField {
    pub name: String,
    pub ts_type: String,
}

/// Aggregate report returned by `run_emit_ts`.
#[derive(Debug, Default)]
pub struct EmitTsReport {
    /// The interfaces that were successfully emitted.
    pub interfaces: Vec<TsInterface>,
    /// Whether the output file was written.
    pub written: bool,
}

/// Errors that `run_emit_ts` can return.
#[derive(Debug)]
pub enum EmitTsError {
    /// Could not read the input file or write the output file.
    Io(String),
    /// The input file failed to parse.
    Parse(String),
    /// The input file failed type-checking.
    TypeCheck(String),
    /// A struct field's type is not supported in Stage 0.
    ///
    /// The error code is always `EMIT_TS_999` in Stage 0 (catch-all).
    /// Stages 1+ replace this with specific `EMIT_TS_001`–`EMIT_TS_021` codes.
    UnsupportedField {
        /// Always `"EMIT_TS_999"` in Stage 0.
        code: String,
        /// Name of the struct containing the unsupported field.
        struct_name: String,
        /// Name of the unsupported field.
        field_name: String,
        /// The Vais type string that could not be lowered.
        vais_type: String,
    },
}

impl fmt::Display for EmitTsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmitTsError::Io(msg) => write!(f, "I/O error: {}", msg),
            EmitTsError::Parse(msg) => write!(f, "parse error: {}", msg),
            EmitTsError::TypeCheck(msg) => write!(f, "type-check error: {}", msg),
            EmitTsError::UnsupportedField {
                code,
                struct_name,
                field_name,
                vais_type,
            } => write!(
                f,
                "{}: struct '{}' field '{}' has unsupported type '{}' (composite types are Stage 1+ scope)",
                code, struct_name, field_name, vais_type
            ),
        }
    }
}

// ==================== Entry point ====================

/// Run `emit-ts` on `opts.input` and write the result to `opts.output`.
///
/// Returns `Ok(EmitTsReport)` on complete success.
/// Returns `Err(EmitTsError)` when any unsupported surface is encountered
/// or when I/O / parse fails.
pub fn run_emit_ts(opts: EmitTsOptions) -> Result<EmitTsReport, EmitTsError> {
    // Step 1 — read source.
    let source = std::fs::read_to_string(&opts.input).map_err(|e| {
        EmitTsError::Io(format!(
            "cannot read '{}': {}",
            opts.input.display(),
            e
        ))
    })?;

    // Step 2 — parse.
    let ast = vais_parser::parse(&source).map_err(|e| {
        EmitTsError::Parse(format!(
            "parse error in '{}': {}",
            opts.input.display(),
            e
        ))
    })?;

    // Step 3 — type-check (we run TC for correctness but do not use the
    // resolved types in Stage 0 — the lowering is purely AST-driven).
    let mut checker = vais_types::TypeChecker::new();
    if let Err(tc_error) = checker.check_module(&ast) {
        return Err(EmitTsError::TypeCheck(format!(
            "type-check failed in '{}': {}",
            opts.input.display(),
            tc_error
        )));
    }

    // Step 4 — walk top-level items, lower pub structs.
    let mut report = EmitTsReport::default();
    for item in &ast.items {
        if let Item::Struct(s) = &item.node {
            // Skip non-pub structs silently (Stage 0 spec: only pub structs are emitted).
            if !s.is_pub {
                continue;
            }

            let struct_name = s.name.node.clone();
            let mut ts_fields = Vec::new();

            for field in &s.fields {
                let field_name = field.name.node.clone();
                let ts_type = lower_primitive_type(&field.ty.node).ok_or_else(|| {
                    EmitTsError::UnsupportedField {
                        code: "EMIT_TS_999".to_string(),
                        struct_name: struct_name.clone(),
                        field_name: field_name.clone(),
                        vais_type: format_type(&field.ty.node),
                    }
                })?;

                ts_fields.push(TsField {
                    name: field_name,
                    ts_type,
                });
            }

            report.interfaces.push(TsInterface {
                name: struct_name,
                fields: ts_fields,
            });
        }
        // All other item kinds (Function, Enum, Trait, etc.) are skipped silently.
    }

    // Step 5 — render `.d.ts` content.
    let dts_content = render_dts(&report.interfaces);

    // Step 6 — write output file.
    if let Some(parent) = opts.output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| {
                EmitTsError::Io(format!(
                    "cannot create output directory '{}': {}",
                    parent.display(),
                    e
                ))
            })?;
        }
    }
    std::fs::write(&opts.output, &dts_content).map_err(|e| {
        EmitTsError::Io(format!(
            "cannot write '{}': {}",
            opts.output.display(),
            e
        ))
    })?;

    report.written = true;
    Ok(report)
}

// ==================== Type lowering ====================

/// Stage 0 primitive type lowering table.
///
/// Returns `Some(ts_type)` for the types we know how to lower, `None` for
/// everything else (composites, generics, structs, enums, …).
fn lower_primitive_type(ty: &Type) -> Option<String> {
    let name = match ty {
        Type::Named { name, generics } if generics.is_empty() => name.as_str(),
        _ => return None,
    };

    match name {
        // Integer types → number
        "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128" => {
            Some("number".to_string())
        }
        // Float types → number
        "f32" | "f64" => Some("number".to_string()),
        // Boolean → boolean
        "bool" => Some("boolean".to_string()),
        // String types → string
        "str" | "String" => Some("string".to_string()),
        // Everything else is unsupported in Stage 0
        _ => None,
    }
}

// ==================== Renderer ====================

/// Render a list of `TsInterface`s into a `.d.ts` file string.
///
/// Each interface is emitted as:
/// ```typescript
/// export interface Name {
///   readonly field: type;
/// }
/// ```
fn render_dts(interfaces: &[TsInterface]) -> String {
    let mut out = String::new();
    out.push_str("// Generated by vaisc emit-ts (Stage 0)\n");
    out.push_str("// Do not edit manually.\n");
    for iface in interfaces {
        out.push('\n');
        out.push_str(&format!("export interface {} {{\n", iface.name));
        for field in &iface.fields {
            out.push_str(&format!(
                "  readonly {}: {};\n",
                field.name, field.ts_type
            ));
        }
        out.push_str("}\n");
    }
    out
}

// ==================== Type formatter (for error messages) ====================

/// Format a Vais `Type` node into a human-readable string for error messages.
fn format_type(ty: &Type) -> String {
    match ty {
        Type::Named { name, generics } if generics.is_empty() => name.clone(),
        Type::Named { name, generics } => {
            let args: Vec<String> = generics.iter().map(|g| format_type(&g.node)).collect();
            format!("{}<{}>", name, args.join(", "))
        }
        Type::Array(inner) => format!("[{}]", format_type(&inner.node)),
        Type::Optional(inner) => format!("{}?", format_type(&inner.node)),
        Type::Result(inner) => format!("{}!", format_type(&inner.node)),
        Type::Tuple(elems) => {
            let parts: Vec<String> = elems.iter().map(|e| format_type(&e.node)).collect();
            format!("({})", parts.join(", "))
        }
        Type::Unit => "()".to_string(),
        _ => "<complex type>".to_string(),
    }
}

// ==================== CLI entry point ====================

/// Public entry point called from `main.rs`.
///
/// Prints diagnostics to stderr and returns the appropriate process exit code:
/// - `0`: success, `.d.ts` written.
/// - `1`: emit error (EMIT_TS_NNN), `.d.ts` not written.
/// - `2`: I/O / parse / type-check error, `.d.ts` not written.
pub(crate) fn cmd_emit_ts(input: &Path, output: &Path) -> Result<(), String> {
    let opts = EmitTsOptions {
        input: input.to_path_buf(),
        output: output.to_path_buf(),
    };

    match run_emit_ts(opts) {
        Ok(report) => {
            let iface_count = report.interfaces.len();
            println!(
                "{} Emitted {} interface(s) to {}",
                "OK".green().bold(),
                iface_count,
                output.display()
            );
            Ok(())
        }
        Err(EmitTsError::UnsupportedField {
            code,
            struct_name,
            field_name,
            vais_type,
        }) => {
            eprintln!(
                "{}: {} struct '{}' field '{}' has unsupported type '{}' (composite types are Stage 1+ scope)",
                "error".red().bold(),
                code,
                struct_name,
                field_name,
                vais_type
            );
            // Exit code 1 for emit errors (EMIT_TS_NNN).
            // We use Err here but the caller exits 1 for any Err.
            Err(format!(
                "{}: unsupported field '{}' in struct '{}' (type: '{}')",
                code, field_name, struct_name, vais_type
            ))
        }
        Err(e @ EmitTsError::Io(_)) | Err(e @ EmitTsError::Parse(_)) | Err(e @ EmitTsError::TypeCheck(_)) => {
            // Exit code 2 for I/O / parse / type-check errors.
            Err(format!("{}", e))
        }
    }
}

// ==================== Unit tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lower_integer_types() {
        for name in &["i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128"] {
            let ty = Type::Named {
                name: name.to_string(),
                generics: vec![],
            };
            assert_eq!(
                lower_primitive_type(&ty),
                Some("number".to_string()),
                "expected 'number' for {}",
                name
            );
        }
    }

    #[test]
    fn lower_float_types() {
        for name in &["f32", "f64"] {
            let ty = Type::Named {
                name: name.to_string(),
                generics: vec![],
            };
            assert_eq!(lower_primitive_type(&ty), Some("number".to_string()));
        }
    }

    #[test]
    fn lower_bool() {
        let ty = Type::Named {
            name: "bool".to_string(),
            generics: vec![],
        };
        assert_eq!(lower_primitive_type(&ty), Some("boolean".to_string()));
    }

    #[test]
    fn lower_str_and_string() {
        for name in &["str", "String"] {
            let ty = Type::Named {
                name: name.to_string(),
                generics: vec![],
            };
            assert_eq!(lower_primitive_type(&ty), Some("string".to_string()));
        }
    }

    #[test]
    fn lower_vec_returns_none() {
        // Vec<i64> is not a primitive — must return None (EMIT_TS_999 will fire).
        let ty = Type::Named {
            name: "Vec".to_string(),
            generics: vec![vais_ast::Spanned {
                node: Type::Named {
                    name: "i64".to_string(),
                    generics: vec![],
                },
                span: vais_ast::Span { file_id: 0, start: 0, end: 0 },
            }],
        };
        assert_eq!(lower_primitive_type(&ty), None);
    }

    #[test]
    fn render_dts_basic() {
        let interfaces = vec![TsInterface {
            name: "User".to_string(),
            fields: vec![
                TsField { name: "id".to_string(), ts_type: "number".to_string() },
                TsField { name: "name".to_string(), ts_type: "string".to_string() },
                TsField { name: "active".to_string(), ts_type: "boolean".to_string() },
            ],
        }];
        let rendered = render_dts(&interfaces);
        assert!(rendered.contains("export interface User {"));
        assert!(rendered.contains("readonly id: number;"));
        assert!(rendered.contains("readonly name: string;"));
        assert!(rendered.contains("readonly active: boolean;"));
    }

    #[test]
    fn render_dts_empty() {
        let rendered = render_dts(&[]);
        assert!(rendered.contains("Generated by vaisc emit-ts"));
        // No interface declarations
        assert!(!rendered.contains("export interface"));
    }

    #[test]
    fn emit_ts_error_display_unsupported() {
        let err = EmitTsError::UnsupportedField {
            code: "EMIT_TS_999".to_string(),
            struct_name: "Foo".to_string(),
            field_name: "v".to_string(),
            vais_type: "Vec<i64>".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("EMIT_TS_999"));
        assert!(msg.contains("Foo"));
        assert!(msg.contains("v"));
        assert!(msg.contains("Vec<i64>"));
    }

    #[test]
    fn emit_ts_error_display_io() {
        let err = EmitTsError::Io("no such file".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("I/O error"));
    }
}
