//! `vaisc emit-ts` command — TypeScript declaration file emitter (Stage 2)
//!
//! Emits a `.d.ts` file from a `.vais` schema source.
//!
//! Stage 0 scope (LANDED):
//!   - CLI plumbing (options, report type, error type)
//!   - Reads input, runs vaisc parser + type checker
//!   - Emits a TS `interface` for each top-level `pub struct` with primitive fields
//!   - Primitive type lowering table: i8/i16/i32/i64/i128/u8-u128 → `number`,
//!     f32/f64 → `number`, bool → `boolean`, str/String → `string`
//!   - Any struct field whose type is NOT in the primitive table: emits EMIT_TS_999
//!   - Non-pub structs: skipped silently
//!   - Non-struct top-level items: skipped silently
//!
//! Stage 1 scope (LANDED):
//!   - Composite type lowering:
//!       Vec<T>              → ReadonlyArray<T_lowered>
//!       &[T], &mut [T]      → ReadonlyArray<T_lowered>
//!       &T, &mut T          → T_lowered  (reference distinction lost; documented in header)
//!       ()  (Unit)          → null
//!       Option<T>           → T_lowered | null
//!       Result<T, E>        → { ok: T_lowered } | { err: E_lowered }
//!       (T1, T2, …)         → readonly [T1_lowered, T2_lowered, …]
//!       HashMap<K, V>       → Map<K_lowered, V_lowered>
//!       nested struct S     → S  (interface name reference)
//!       enum X              → discriminated union per variant
//!   - pub enum items are now emitted as tagged TS discriminated unions
//!   - Topological emit order (leaves first) for readability
//!
//! Stage 2 scope (this file):
//!   - Specific stable error codes EMIT_TS_001–EMIT_TS_021 for each enumerated
//!     unsupported surface, replacing the catch-all EMIT_TS_999 for known cases.
//!   - EMIT_TS_001: generic type parameter in struct/enum
//!   - EMIT_TS_002: trait-bounded generic parameter
//!   - EMIT_TS_003: dyn Trait field
//!   - EMIT_TS_004: lifetime parameter (in struct generics or RefLifetime field)
//!   - EMIT_TS_005: function pointer type (`fn(…)→…` or `FnPtr`)
//!   - EMIT_TS_006: impl Trait (not present in current AST — future-proofed)
//!   - EMIT_TS_007: pub function at module top-level
//!   - EMIT_TS_008: pub type alias whose RHS is an unsupported surface
//!   - EMIT_TS_009: raw pointer (`*T`)
//!   - EMIT_TS_010: fixed-size array (`[T; N]`)
//!   - EMIT_TS_011: Range types (Named "Range", "RangeInclusive", etc.)
//!   - EMIT_TS_012: Future/async types (Named "Future" or ResolvedType::Future)
//!   - EMIT_TS_013: SIMD vector / ConstArray (ResolvedType::Vector)
//!   - EMIT_TS_014: Dependent/refinement types
//!   - EMIT_TS_015: pub union
//!   - EMIT_TS_016: pub trait
//!   - EMIT_TS_017: pub const at module scope
//!   - EMIT_TS_018: pub global
//!   - EMIT_TS_019: impl block at module scope
//!   - EMIT_TS_020: extern block
//!   - EMIT_TS_021: pub macro definition
//!   - EMIT_TS_999: catch-all for genuinely unclassified surfaces

use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};
use vais_ast::{GenericParamKind, Item, Type, VariantFields};

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

/// A TypeScript type alias for an enum (discriminated union).
#[derive(Debug)]
pub struct TsTypeAlias {
    /// The enum name (used as the TS type name).
    pub name: String,
    /// The full TS type expression (a union of object literals).
    pub rhs: String,
}

/// A top-level TypeScript declaration emitted for one Vais item.
#[derive(Debug)]
pub enum TsDecl {
    Interface(TsInterface),
    TypeAlias(TsTypeAlias),
}

impl TsDecl {
    pub fn name(&self) -> &str {
        match self {
            TsDecl::Interface(i) => &i.name,
            TsDecl::TypeAlias(a) => &a.name,
        }
    }
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
    /// A struct/enum field's type is not supported (EMIT_TS_001–EMIT_TS_999).
    ///
    /// Stage 2 routes known surfaces to specific EMIT_TS_001–EMIT_TS_021 codes.
    /// EMIT_TS_999 is reserved for genuinely unclassified surfaces.
    UnsupportedField {
        /// Specific code: `"EMIT_TS_001"` through `"EMIT_TS_021"`, or `"EMIT_TS_999"`.
        code: String,
        /// Name of the struct/enum containing the unsupported field.
        struct_name: String,
        /// Name of the unsupported field.
        field_name: String,
        /// The Vais type string that could not be lowered.
        vais_type: String,
    },
    /// A top-level item kind is not supported in schema context (EMIT_TS_007–EMIT_TS_021).
    UnsupportedItem {
        /// Specific code: `"EMIT_TS_007"` through `"EMIT_TS_021"`.
        code: String,
        /// Kind label for the item (e.g., "function", "union", "trait").
        kind: String,
        /// Name of the unsupported item.
        item_name: String,
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
                "[{}] struct '{}' field '{}' has unsupported type '{}'",
                code, struct_name, field_name, vais_type
            ),
            EmitTsError::UnsupportedItem {
                code,
                kind,
                item_name,
            } => write!(
                f,
                "[{}] pub {} '{}' is not supported in schema emit (vaisc emit-ts emits types only)",
                code, kind, item_name
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
    // resolved types in Stage 0/1 — the lowering is purely AST-driven).
    let mut checker = vais_types::TypeChecker::new();
    if let Err(tc_error) = checker.check_module(&ast) {
        return Err(EmitTsError::TypeCheck(format!(
            "type-check failed in '{}': {}",
            opts.input.display(),
            tc_error
        )));
    }

    // Step 4 — build a registry of all struct/enum names defined in this file
    // so we can reference them as nested types.
    let mut known_types: HashSet<String> = HashSet::new();
    for item in &ast.items {
        match &item.node {
            Item::Struct(s) if s.is_pub => {
                known_types.insert(s.name.node.clone());
            }
            Item::Enum(e) if e.is_pub => {
                known_types.insert(e.name.node.clone());
            }
            _ => {}
        }
    }

    // Step 5 — walk top-level items.
    //   - pub struct  → emit as TS interface (after checking generics/lifetimes)
    //   - pub enum    → emit as TS discriminated union (after checking generics)
    //   - pub type alias → emit as TS type alias (after checking RHS for unsupported surfaces)
    //   - pub function   → EMIT_TS_007
    //   - pub union      → EMIT_TS_015
    //   - pub trait      → EMIT_TS_016
    //   - pub const      → EMIT_TS_017
    //   - pub global     → EMIT_TS_018
    //   - impl block     → EMIT_TS_019 (impl blocks have no is_pub; always hard-fail)
    //   - extern block   → EMIT_TS_020
    //   - pub macro      → EMIT_TS_021
    //   - non-pub items  → skipped silently
    //   - Use, TraitAlias, Error → skipped silently
    let mut decls: Vec<TsDecl> = Vec::new();
    let mut report = EmitTsReport::default();

    for item in &ast.items {
        match &item.node {
            Item::Struct(s) => {
                if !s.is_pub {
                    continue;
                }
                let struct_name = s.name.node.clone();

                // EMIT_TS_004: lifetime parameters on the struct declaration.
                for gp in &s.generics {
                    if matches!(gp.kind, GenericParamKind::Lifetime { .. }) {
                        return Err(EmitTsError::UnsupportedItem {
                            code: "EMIT_TS_004".to_string(),
                            kind: "struct with lifetime parameter".to_string(),
                            item_name: struct_name.clone(),
                        });
                    }
                    // EMIT_TS_002: generic parameter with trait bounds.
                    if matches!(gp.kind, GenericParamKind::Type { .. }) && !gp.bounds.is_empty() {
                        return Err(EmitTsError::UnsupportedItem {
                            code: "EMIT_TS_002".to_string(),
                            kind: "struct with trait-bounded generic parameter".to_string(),
                            item_name: struct_name.clone(),
                        });
                    }
                    // EMIT_TS_001: any type generic parameter (even without bounds).
                    if matches!(gp.kind, GenericParamKind::Type { .. }) {
                        return Err(EmitTsError::UnsupportedItem {
                            code: "EMIT_TS_001".to_string(),
                            kind: "struct with generic type parameter".to_string(),
                            item_name: struct_name.clone(),
                        });
                    }
                }

                let mut ts_fields = Vec::new();

                for field in &s.fields {
                    let field_name = field.name.node.clone();
                    let ts_type =
                        lower_type(&field.ty.node, &known_types).ok_or_else(|| {
                            let code = classify_type_error_code(&field.ty.node);
                            EmitTsError::UnsupportedField {
                                code,
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

                // Also add to the flat interfaces list for backward-compat.
                report.interfaces.push(TsInterface {
                    name: struct_name.clone(),
                    fields: ts_fields.iter().map(|f| TsField { name: f.name.clone(), ts_type: f.ts_type.clone() }).collect(),
                });

                decls.push(TsDecl::Interface(TsInterface {
                    name: struct_name,
                    fields: ts_fields,
                }));
            }
            Item::Enum(e) => {
                if !e.is_pub {
                    continue;
                }
                let enum_name = e.name.node.clone();

                // EMIT_TS_004: lifetime parameters on the enum declaration.
                for gp in &e.generics {
                    if matches!(gp.kind, GenericParamKind::Lifetime { .. }) {
                        return Err(EmitTsError::UnsupportedItem {
                            code: "EMIT_TS_004".to_string(),
                            kind: "enum with lifetime parameter".to_string(),
                            item_name: enum_name.clone(),
                        });
                    }
                    // EMIT_TS_002: generic parameter with trait bounds.
                    if matches!(gp.kind, GenericParamKind::Type { .. }) && !gp.bounds.is_empty() {
                        return Err(EmitTsError::UnsupportedItem {
                            code: "EMIT_TS_002".to_string(),
                            kind: "enum with trait-bounded generic parameter".to_string(),
                            item_name: enum_name.clone(),
                        });
                    }
                    // EMIT_TS_001: any type generic parameter (even without bounds).
                    if matches!(gp.kind, GenericParamKind::Type { .. }) {
                        return Err(EmitTsError::UnsupportedItem {
                            code: "EMIT_TS_001".to_string(),
                            kind: "enum with generic type parameter".to_string(),
                            item_name: enum_name.clone(),
                        });
                    }
                }

                // Build the discriminated union type expression.
                let mut arms: Vec<String> = Vec::new();
                for variant in &e.variants {
                    let vname = &variant.name.node;
                    let arm = match &variant.fields {
                        VariantFields::Unit => {
                            format!("{{ kind: \"{}\" }}", vname)
                        }
                        VariantFields::Tuple(types) => {
                            let mut parts = format!("{{ kind: \"{}\"", vname);
                            for (idx, ty_spanned) in types.iter().enumerate() {
                                let ts_t =
                                    lower_type(&ty_spanned.node, &known_types).ok_or_else(|| {
                                        let code = classify_type_error_code(&ty_spanned.node);
                                        EmitTsError::UnsupportedField {
                                            code,
                                            struct_name: enum_name.clone(),
                                            field_name: format!("{}._{}",vname, idx),
                                            vais_type: format_type(&ty_spanned.node),
                                        }
                                    })?;
                                parts.push_str(&format!(", _{}: {}", idx, ts_t));
                            }
                            parts.push_str(" }");
                            parts
                        }
                        VariantFields::Struct(fields) => {
                            let mut parts = format!("{{ kind: \"{}\"", vname);
                            for field in fields {
                                let fname = &field.name.node;
                                let ts_t =
                                    lower_type(&field.ty.node, &known_types).ok_or_else(|| {
                                        let code = classify_type_error_code(&field.ty.node);
                                        EmitTsError::UnsupportedField {
                                            code,
                                            struct_name: enum_name.clone(),
                                            field_name: format!("{}.{}", vname, fname),
                                            vais_type: format_type(&field.ty.node),
                                        }
                                    })?;
                                parts.push_str(&format!(", {}: {}", fname, ts_t));
                            }
                            parts.push_str(" }");
                            parts
                        }
                    };
                    arms.push(arm);
                }

                let rhs = arms.join(" | ");
                decls.push(TsDecl::TypeAlias(TsTypeAlias {
                    name: enum_name,
                    rhs,
                }));
            }
            // EMIT_TS_008: pub type alias — emit if RHS is supported, hard-fail otherwise.
            Item::TypeAlias(ta) => {
                if !ta.is_pub {
                    continue;
                }
                let alias_name = ta.name.node.clone();
                match lower_type(&ta.ty.node, &known_types) {
                    Some(ts_rhs) => {
                        decls.push(TsDecl::TypeAlias(TsTypeAlias {
                            name: alias_name,
                            rhs: ts_rhs,
                        }));
                    }
                    None => {
                        return Err(EmitTsError::UnsupportedItem {
                            code: "EMIT_TS_008".to_string(),
                            kind: "type alias to unsupported surface".to_string(),
                            item_name: alias_name,
                        });
                    }
                }
            }
            // EMIT_TS_007: pub function at module top-level.
            Item::Function(f) => {
                if !f.is_pub {
                    continue;
                }
                return Err(EmitTsError::UnsupportedItem {
                    code: "EMIT_TS_007".to_string(),
                    kind: "function".to_string(),
                    item_name: f.name.node.clone(),
                });
            }
            // EMIT_TS_015: pub union.
            Item::Union(u) => {
                if !u.is_pub {
                    continue;
                }
                return Err(EmitTsError::UnsupportedItem {
                    code: "EMIT_TS_015".to_string(),
                    kind: "union".to_string(),
                    item_name: u.name.node.clone(),
                });
            }
            // EMIT_TS_016: pub trait.
            Item::Trait(t) => {
                if !t.is_pub {
                    continue;
                }
                return Err(EmitTsError::UnsupportedItem {
                    code: "EMIT_TS_016".to_string(),
                    kind: "trait".to_string(),
                    item_name: t.name.node.clone(),
                });
            }
            // EMIT_TS_017: pub const at module scope.
            Item::Const(c) => {
                if !c.is_pub {
                    continue;
                }
                return Err(EmitTsError::UnsupportedItem {
                    code: "EMIT_TS_017".to_string(),
                    kind: "const".to_string(),
                    item_name: c.name.node.clone(),
                });
            }
            // EMIT_TS_018: pub global.
            Item::Global(g) => {
                if !g.is_pub {
                    continue;
                }
                return Err(EmitTsError::UnsupportedItem {
                    code: "EMIT_TS_018".to_string(),
                    kind: "global".to_string(),
                    item_name: g.name.node.clone(),
                });
            }
            // EMIT_TS_019: impl block (no is_pub on Impl; always hard-fail at module scope).
            Item::Impl(_impl_block) => {
                return Err(EmitTsError::UnsupportedItem {
                    code: "EMIT_TS_019".to_string(),
                    kind: "impl block".to_string(),
                    item_name: "<impl>".to_string(),
                });
            }
            // EMIT_TS_020: extern block (no is_pub; always hard-fail at module scope).
            Item::ExternBlock(_ext) => {
                return Err(EmitTsError::UnsupportedItem {
                    code: "EMIT_TS_020".to_string(),
                    kind: "extern block".to_string(),
                    item_name: "<extern>".to_string(),
                });
            }
            // EMIT_TS_021: pub macro definition.
            Item::Macro(m) => {
                if !m.is_pub {
                    continue;
                }
                return Err(EmitTsError::UnsupportedItem {
                    code: "EMIT_TS_021".to_string(),
                    kind: "macro".to_string(),
                    item_name: m.name.node.clone(),
                });
            }
            // Use, TraitAlias, Error → skipped silently (no schema content).
            Item::Use(_) | Item::TraitAlias(_) | Item::Error { .. } => {}
        }
    }

    // Step 6 — topological sort: emit leaves (types with no intra-file
    // dependencies) first. TypeScript handles mutual recursion natively, so
    // we only need a best-effort ordering for readability.
    let sorted_decls = topo_sort(decls);

    // Step 7 — render `.d.ts` content.
    let dts_content = render_dts_stage1(&sorted_decls);

    // Step 8 — write output file.
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

// ==================== Topological sort ====================

/// Sort declarations so that types referenced by other types come first.
///
/// Uses a simple DFS-based topological sort.  Cycles (mutual recursion) are
/// handled by TypeScript natively, so we just emit both without panicking.
fn topo_sort(decls: Vec<TsDecl>) -> Vec<TsDecl> {
    // Build an index: name → position in `decls`.
    let name_to_idx: HashMap<String, usize> = decls
        .iter()
        .enumerate()
        .map(|(i, d)| (d.name().to_string(), i))
        .collect();

    // For each decl, compute which other decls it references (by name).
    let deps: Vec<HashSet<usize>> = decls
        .iter()
        .map(|d| {
            let names = referenced_type_names(d);
            names
                .into_iter()
                .filter_map(|n| name_to_idx.get(&n).copied())
                .collect()
        })
        .collect();

    // DFS topological sort.
    let n = decls.len();
    let mut visited = vec![false; n];
    let mut order: Vec<usize> = Vec::with_capacity(n);

    fn visit(
        idx: usize,
        deps: &[HashSet<usize>],
        visited: &mut Vec<bool>,
        order: &mut Vec<usize>,
    ) {
        if visited[idx] {
            return;
        }
        visited[idx] = true;
        for &dep in &deps[idx] {
            visit(dep, deps, visited, order);
        }
        order.push(idx);
    }

    for i in 0..n {
        visit(i, &deps, &mut visited, &mut order);
    }

    // Re-assemble in sorted order.  We must consume `decls` so use indices.
    let mut decls_opt: Vec<Option<TsDecl>> = decls.into_iter().map(Some).collect();
    order
        .into_iter()
        .map(|i| {
            decls_opt[i]
                .take()
                .expect("invariant: topo order is a permutation of 0..n; each index taken exactly once")
        })
        .collect()
}

/// Collect all type names that a `TsDecl` references by examining its rendered
/// output for identifiers that look like they could be type names.
///
/// This is intentionally conservative (it only looks at what the lowering
/// emitted, not re-traversing the AST) to avoid a circular dependency on the
/// AST walking infrastructure.
fn referenced_type_names(decl: &TsDecl) -> Vec<String> {
    let rendered = match decl {
        TsDecl::Interface(i) => i.fields.iter().map(|f| f.ts_type.clone()).collect::<Vec<_>>().join(" "),
        TsDecl::TypeAlias(a) => a.rhs.clone(),
    };
    // Extract bare identifiers that start with an uppercase letter — these
    // could be other struct/enum names emitted in this file.
    let mut names = Vec::new();
    let mut chars = rendered.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_ascii_uppercase() {
            let mut ident = String::new();
            ident.push(ch);
            while let Some(&nc) = chars.peek() {
                if nc.is_alphanumeric() || nc == '_' {
                    ident.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            // Filter out known TS built-ins.
            match ident.as_str() {
                "ReadonlyArray" | "Map" | "Array" => {}
                _ => names.push(ident),
            }
        }
    }
    names
}

// ==================== Type error classification (Stage 2) ====================

/// Classify an unsupported AST `Type` node to the most specific stable error code.
///
/// Called only when `lower_type` returns `None` (i.e., the type is not lowerable).
/// Returns the EMIT_TS_NNN code string that best describes why the type is rejected.
///
/// Code assignments:
/// - EMIT_TS_003: `dyn Trait`
/// - EMIT_TS_004: explicit lifetime (`&'a T`, `&'a mut T`)
/// - EMIT_TS_005: function pointer types (`FnPtr` or `Fn`)
/// - EMIT_TS_009: raw pointer (`*T`)
/// - EMIT_TS_010: fixed-size array (`[T; N]`)
/// - EMIT_TS_011: range types (`Range`, `RangeInclusive`, `RangeFrom`, `RangeTo`, `RangeFull`)
/// - EMIT_TS_012: future/async types (`Future`, `async`)
/// - EMIT_TS_013: SIMD vector (`Vector` or named SIMD type) — not currently reachable via
///                AST `Type`; future-proofed here for completeness
/// - EMIT_TS_014: dependent/refinement types
/// - EMIT_TS_999: any other unlowerable type (catch-all)
fn classify_type_error_code(ty: &Type) -> String {
    match ty {
        // EMIT_TS_009: raw pointer.
        Type::Pointer(_) => "EMIT_TS_009".to_string(),

        // EMIT_TS_010: fixed-size array [T; N].
        Type::ConstArray { .. } => "EMIT_TS_010".to_string(),

        // EMIT_TS_003: dyn Trait.
        Type::DynTrait { .. } => "EMIT_TS_003".to_string(),

        // EMIT_TS_004: explicit lifetime in reference (&'a T, &'a mut T).
        // Stage 1's lower_type passes &'a T → T_lowered (erasing the lifetime).
        // If lower_type returned None it means the *inner* type is unsupported —
        // delegate to the inner type's code so the user gets the precise reason.
        Type::RefLifetime { inner, .. } | Type::RefMutLifetime { inner, .. } => {
            classify_type_error_code(&inner.node)
        }

        // EMIT_TS_005: function pointer or function type.
        Type::FnPtr { .. } | Type::Fn { .. } => "EMIT_TS_005".to_string(),

        // EMIT_TS_014: dependent/refinement type.
        Type::Dependent { .. } => "EMIT_TS_014".to_string(),

        // EMIT_TS_011: range types — detected via named types.
        // EMIT_TS_012: future/async types — detected via named types.
        Type::Named { name, .. } => {
            // Range family
            match name.as_str() {
                "Range" | "RangeInclusive" | "RangeFrom" | "RangeTo" | "RangeFull" => {
                    "EMIT_TS_011".to_string()
                }
                // Future / async
                "Future" | "AsyncFuture" => "EMIT_TS_012".to_string(),
                _ => "EMIT_TS_999".to_string(),
            }
        }

        // EMIT_TS_014: linear / affine types (Vais-specific ownership types that
        // carry semantic contracts not representable in TypeScript).
        Type::Linear(_) | Type::Affine(_) => "EMIT_TS_014".to_string(),

        // EMIT_TS_999: fallback for anything else that isn't lowerable and doesn't
        // match a specific code (e.g., Infer, Associated, Map-type-that-failed, etc.).
        _ => "EMIT_TS_999".to_string(),
    }
}

// ==================== Type lowering ====================

/// Stage 1 composite + primitive type lowering.
///
/// Returns `Some(ts_type_string)` for supported types, `None` for unsupported
/// ones that will trigger `EMIT_TS_999`.
///
/// `known_types` is the set of struct/enum names defined in the same file so
/// that bare `S` (nested struct) references can be passed through as `S`.
fn lower_type(ty: &Type, known_types: &HashSet<String>) -> Option<String> {
    match ty {
        // ── Primitive bare names ──────────────────────────────────────────
        Type::Named { name, generics } if generics.is_empty() => {
            match name.as_str() {
                // Integer types → branded number (Stage 4 per design v6).
                // i64 → f64 schema change shows up as a typed change in TS
                // because each width has a distinct brand.
                "i8"   => Some("VaisI8".to_string()),
                "i16"  => Some("VaisI16".to_string()),
                "i32"  => Some("VaisI32".to_string()),
                "i64"  => Some("VaisI64".to_string()),
                "i128" => Some("VaisI128".to_string()),
                "u8"   => Some("VaisU8".to_string()),
                "u16"  => Some("VaisU16".to_string()),
                "u32"  => Some("VaisU32".to_string()),
                "u64"  => Some("VaisU64".to_string()),
                "u128" => Some("VaisU128".to_string()),
                "f32"  => Some("VaisF32".to_string()),
                "f64"  => Some("VaisF64".to_string()),
                // Boolean → boolean
                "bool" => Some("boolean".to_string()),
                // String types → string
                "str" | "String" => Some("string".to_string()),
                // Nested struct/enum reference (same file)
                other if known_types.contains(other) => Some(other.to_string()),
                // Everything else unknown
                _ => None,
            }
        }

        // ── Generic named types ───────────────────────────────────────────
        Type::Named { name, generics } => {
            match name.as_str() {
                // Vec<T> → ReadonlyArray<T_lowered>
                "Vec" if generics.len() == 1 => {
                    let inner = lower_type(&generics[0].node, known_types)?;
                    Some(format!("ReadonlyArray<{}>", inner))
                }
                // Option<T> → T_lowered | null
                "Option" if generics.len() == 1 => {
                    let inner = lower_type(&generics[0].node, known_types)?;
                    Some(format!("{} | null", inner))
                }
                // Result<T, E> → { ok: T_lowered } | { err: E_lowered }
                "Result" if generics.len() == 2 => {
                    let ok_t = lower_type(&generics[0].node, known_types)?;
                    let err_t = lower_type(&generics[1].node, known_types)?;
                    Some(format!("{{ ok: {} }} | {{ err: {} }}", ok_t, err_t))
                }
                // HashMap<K, V> → Map<K_lowered, V_lowered>
                "HashMap" if generics.len() == 2 => {
                    let k = lower_type(&generics[0].node, known_types)?;
                    let v = lower_type(&generics[1].node, known_types)?;
                    Some(format!("Map<{}, {}>", k, v))
                }
                // Any other generic type — unsupported (EMIT_TS_999 / stage 2+)
                _ => None,
            }
        }

        // ── Unit () → null ────────────────────────────────────────────────
        Type::Unit => Some("null".to_string()),

        // ── Tuples → readonly [T1_lowered, …] ────────────────────────────
        Type::Tuple(elems) => {
            let parts: Option<Vec<String>> = elems
                .iter()
                .map(|e| lower_type(&e.node, known_types))
                .collect();
            parts.map(|p| format!("readonly [{}]", p.join(", ")))
        }

        // ── References &T / &mut T → T_lowered (ownership erased) ────────
        Type::Ref(inner) | Type::RefMut(inner) => {
            lower_type(&inner.node, known_types)
        }
        Type::RefLifetime { inner, .. } | Type::RefMutLifetime { inner, .. } => {
            lower_type(&inner.node, known_types)
        }

        // ── Slices &[T] / &mut [T] → ReadonlyArray<T_lowered> ────────────
        Type::Slice(inner) | Type::SliceMut(inner) => {
            let inner_t = lower_type(&inner.node, known_types)?;
            Some(format!("ReadonlyArray<{}>", inner_t))
        }

        // ── Everything else: unsupported in Stage 1 ───────────────────────
        _ => None,
    }
}

// ==================== Renderer ====================

/// Render a list of `TsDecl`s into a `.d.ts` file string (Stage 1).
///
/// Each struct is emitted as:
/// ```typescript
/// export interface Name {
///   readonly field: type;
/// }
/// ```
///
/// Each enum is emitted as:
/// ```typescript
/// export type Name = { kind: "A" } | { kind: "B", _0: number };
/// ```
fn render_dts_stage1(decls: &[TsDecl]) -> String {
    let mut out = String::new();
    out.push_str("// Generated by vaisc emit-ts (Stage 4)\n");
    out.push_str("// Do not edit manually.\n");
    out.push_str("// Note: Vais reference types (&T, &mut T) have their ownership\n");
    out.push_str("// information erased in this TypeScript declaration file.\n");
    out.push_str("// Numeric types are branded by width so that an i64 → f64 schema\n");
    out.push_str("// change is a typed change in TypeScript (per design v6).\n");
    out.push('\n');
    out.push_str("// ── Numeric brand prelude ──────────────────────────────────────\n");
    out.push_str("declare const __vaisInt: unique symbol;\n");
    out.push_str("declare const __vaisUint: unique symbol;\n");
    out.push_str("declare const __vaisFloat: unique symbol;\n");
    out.push_str("export type VaisI8   = number & { readonly [__vaisInt]: 8   };\n");
    out.push_str("export type VaisI16  = number & { readonly [__vaisInt]: 16  };\n");
    out.push_str("export type VaisI32  = number & { readonly [__vaisInt]: 32  };\n");
    out.push_str("export type VaisI64  = number & { readonly [__vaisInt]: 64  };\n");
    out.push_str("export type VaisI128 = number & { readonly [__vaisInt]: 128 };\n");
    out.push_str("export type VaisU8   = number & { readonly [__vaisUint]: 8   };\n");
    out.push_str("export type VaisU16  = number & { readonly [__vaisUint]: 16  };\n");
    out.push_str("export type VaisU32  = number & { readonly [__vaisUint]: 32  };\n");
    out.push_str("export type VaisU64  = number & { readonly [__vaisUint]: 64  };\n");
    out.push_str("export type VaisU128 = number & { readonly [__vaisUint]: 128 };\n");
    out.push_str("export type VaisF32  = number & { readonly [__vaisFloat]: 32 };\n");
    out.push_str("export type VaisF64  = number & { readonly [__vaisFloat]: 64 };\n");
    for decl in decls {
        out.push('\n');
        match decl {
            TsDecl::Interface(iface) => {
                out.push_str(&format!("export interface {} {{\n", iface.name));
                for field in &iface.fields {
                    out.push_str(&format!(
                        "  readonly {}: {};\n",
                        field.name, field.ts_type
                    ));
                }
                out.push_str("}\n");
            }
            TsDecl::TypeAlias(alias) => {
                out.push_str(&format!("export type {} = {};\n", alias.name, alias.rhs));
            }
        }
    }
    out
}

/// Render a list of `TsInterface`s into a `.d.ts` file string (Stage 0 compat).
///
/// Kept for internal unit tests that pre-date Stage 1.
#[cfg(test)]
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
                "{} [{}]: struct '{}' field '{}' has unsupported type '{}'",
                "error".red().bold(),
                code,
                struct_name,
                field_name,
                vais_type
            );
            // Exit code 1 for emit errors (EMIT_TS_NNN).
            Err(format!(
                "[{}]: unsupported field '{}' in struct '{}' (type: '{}')",
                code, field_name, struct_name, vais_type
            ))
        }
        Err(EmitTsError::UnsupportedItem {
            code,
            kind,
            item_name,
        }) => {
            eprintln!(
                "{} [{}]: pub {} '{}' is not supported in schema emit",
                "error".red().bold(),
                code,
                kind,
                item_name
            );
            // Exit code 1 for emit errors (EMIT_TS_NNN).
            Err(format!(
                "[{}]: unsupported {} '{}' at module scope",
                code, kind, item_name
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

    fn make_named(name: &str) -> Type {
        Type::Named { name: name.to_string(), generics: vec![] }
    }

    fn make_named_generic(name: &str, args: Vec<Type>) -> Type {
        Type::Named {
            name: name.to_string(),
            generics: args.into_iter().map(|t| vais_ast::Spanned {
                node: t,
                span: vais_ast::Span { file_id: 0, start: 0, end: 0 },
            }).collect(),
        }
    }

    fn spanned(t: Type) -> vais_ast::Spanned<Type> {
        vais_ast::Spanned { node: t, span: vais_ast::Span { file_id: 0, start: 0, end: 0 } }
    }

    fn empty_known() -> HashSet<String> {
        HashSet::new()
    }

    // ── primitive lowering ────────────────────────────────────────────────

    #[test]
    fn lower_integer_types() {
        // Stage 4: each width has a distinct branded TS alias.
        for (name, expected) in &[
            ("i8", "VaisI8"), ("i16", "VaisI16"), ("i32", "VaisI32"),
            ("i64", "VaisI64"), ("i128", "VaisI128"),
            ("u8", "VaisU8"), ("u16", "VaisU16"), ("u32", "VaisU32"),
            ("u64", "VaisU64"), ("u128", "VaisU128"),
        ] {
            let ty = make_named(name);
            assert_eq!(
                lower_type(&ty, &empty_known()),
                Some(expected.to_string()),
                "expected '{}' for {}", expected, name
            );
        }
    }

    #[test]
    fn lower_float_types() {
        for (name, expected) in &[("f32", "VaisF32"), ("f64", "VaisF64")] {
            let ty = make_named(name);
            assert_eq!(lower_type(&ty, &empty_known()), Some(expected.to_string()));
        }
    }

    #[test]
    fn lower_bool() {
        let ty = make_named("bool");
        assert_eq!(lower_type(&ty, &empty_known()), Some("boolean".to_string()));
    }

    #[test]
    fn lower_str_and_string() {
        for name in &["str", "String"] {
            let ty = make_named(name);
            assert_eq!(lower_type(&ty, &empty_known()), Some("string".to_string()));
        }
    }

    // ── stage 1 composite lowering ────────────────────────────────────────

    #[test]
    fn lower_vec_i64() {
        let ty = make_named_generic("Vec", vec![make_named("i64")]);
        assert_eq!(
            lower_type(&ty, &empty_known()),
            Some("ReadonlyArray<VaisI64>".to_string())
        );
    }

    #[test]
    fn lower_option_str() {
        let ty = make_named_generic("Option", vec![make_named("str")]);
        assert_eq!(
            lower_type(&ty, &empty_known()),
            Some("string | null".to_string())
        );
    }

    #[test]
    fn lower_result_i64_str() {
        let ty = make_named_generic("Result", vec![make_named("i64"), make_named("str")]);
        assert_eq!(
            lower_type(&ty, &empty_known()),
            Some("{ ok: VaisI64 } | { err: string }".to_string())
        );
    }

    #[test]
    fn lower_hashmap_str_i64() {
        let ty = make_named_generic("HashMap", vec![make_named("str"), make_named("i64")]);
        assert_eq!(
            lower_type(&ty, &empty_known()),
            Some("Map<string, VaisI64>".to_string())
        );
    }

    #[test]
    fn lower_tuple() {
        let ty = Type::Tuple(vec![spanned(make_named("i64")), spanned(make_named("str"))]);
        assert_eq!(
            lower_type(&ty, &empty_known()),
            Some("readonly [VaisI64, string]".to_string())
        );
    }

    #[test]
    fn lower_unit() {
        assert_eq!(lower_type(&Type::Unit, &empty_known()), Some("null".to_string()));
    }

    #[test]
    fn lower_ref_i64() {
        let ty = Type::Ref(Box::new(spanned(make_named("i64"))));
        assert_eq!(lower_type(&ty, &empty_known()), Some("VaisI64".to_string()));
    }

    #[test]
    fn lower_slice_i64() {
        let ty = Type::Slice(Box::new(spanned(make_named("i64"))));
        assert_eq!(
            lower_type(&ty, &empty_known()),
            Some("ReadonlyArray<VaisI64>".to_string())
        );
    }

    #[test]
    fn lower_nested_struct_ref() {
        let mut known = HashSet::new();
        known.insert("User".to_string());
        let ty = make_named("User");
        assert_eq!(lower_type(&ty, &known), Some("User".to_string()));
    }

    #[test]
    fn lower_unknown_bare_type_returns_none() {
        let ty = make_named("UnknownFoo");
        assert_eq!(lower_type(&ty, &empty_known()), None);
    }

    // ── render_dts (stage 0 compat unit test) ────────────────────────────

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
