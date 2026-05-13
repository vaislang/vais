//! Compile-time exhaustiveness for emit-ts.
//!
//! These tests do not exercise behavior. They exist so that if a new
//! `ResolvedType` variant or top-level `Item` kind is added to vais-types
//! or vais-ast without updating emit-ts's classification, `cargo test`
//! fails to compile this file.
//!
//! Run with: `cargo test --release --test emit_ts_exhaustiveness`.
//!
//! How it works
//! ------------
//! Each `#[test]` contains a private `fn classify(...)` whose body is a
//! `match` that covers every variant of the enum *without a wildcard arm*.
//! Rust's exhaustiveness checker therefore rejects a build if any variant
//! is missing. The function is then cast to a fn-pointer to prevent the
//! dead-code lint from removing the match entirely.
//!
//! What this does NOT test
//! -----------------------
//! The labels ("lowered: …" / "rejected: EMIT_TS_NNN") reflect the
//! intended classification documented in emit_ts.rs Stage 2 header.
//! They are NOT verified against emit-ts's runtime behaviour; that is
//! the job of emit_ts_skeleton.rs and the CLI smoke tests.
//! This file purely guards against silent growth of the enums.

use vais_ast::Item;
use vais_types::ResolvedType;

// ---------------------------------------------------------------------------
// Test 1 — ResolvedType exhaustiveness
// ---------------------------------------------------------------------------

#[test]
fn exhaustiveness_resolved_type() {
    fn classify(t: &ResolvedType) -> &'static str {
        match t {
            // ── Primitive numeric ────────────────────────────────────────
            ResolvedType::I8
            | ResolvedType::I16
            | ResolvedType::I32
            | ResolvedType::I64
            | ResolvedType::I128 => "lowered: number",

            ResolvedType::U8
            | ResolvedType::U16
            | ResolvedType::U32
            | ResolvedType::U64
            | ResolvedType::U128 => "lowered: number",

            ResolvedType::F32 | ResolvedType::F64 => "lowered: number",

            // ── Other primitives ─────────────────────────────────────────
            ResolvedType::Bool => "lowered: boolean",
            ResolvedType::Str => "lowered: string",
            ResolvedType::Unit => "lowered: null (as field value)",

            // ── Sequence / array / slice ──────────────────────────────────
            ResolvedType::Array(_) => "lowered: ReadonlyArray<T>",
            ResolvedType::Slice(_) | ResolvedType::SliceMut(_) => "lowered: ReadonlyArray<T>",
            ResolvedType::ConstArray { .. } => "rejected: EMIT_TS_010",

            // ── Map ───────────────────────────────────────────────────────
            // Named("HashMap") path is handled by the Named arm; the Map
            // resolved type is emitted as Map<K, V>.
            ResolvedType::Map(_, _) => "lowered: Map<K, V>",

            // ── Other compound ────────────────────────────────────────────
            ResolvedType::Tuple(_) => "lowered: readonly [...]",
            ResolvedType::Optional(_) => "lowered: T | null",
            ResolvedType::Result(_, _) => "lowered: { ok: T } | { err: E }",

            // ── Pointer / reference ───────────────────────────────────────
            ResolvedType::Pointer(_) => "rejected: EMIT_TS_009",
            ResolvedType::Ref(_) | ResolvedType::RefMut(_) => "lowered: T (ref erased)",
            ResolvedType::RefLifetime { .. } | ResolvedType::RefMutLifetime { .. } => {
                "rejected: EMIT_TS_004"
            }

            // ── Range / Future ────────────────────────────────────────────
            ResolvedType::Range(_) => "rejected: EMIT_TS_011",
            ResolvedType::Future(_) => "rejected: EMIT_TS_012",

            // ── Function types ────────────────────────────────────────────
            ResolvedType::Fn { .. } => "rejected: EMIT_TS_005",
            ResolvedType::FnPtr { .. } => "rejected: EMIT_TS_005",

            // ── Named (struct/enum references, Vec, HashMap, …) ──────────
            // Emit-ts handles Vec/HashMap specially inside this arm; all
            // other named types either resolve to an interface reference or
            // fall through to EMIT_TS_999.
            ResolvedType::Named { .. } => "lowered or rejected: EMIT_TS_999 (depends on name)",

            // ── Type variables / inference ────────────────────────────────
            ResolvedType::Var(_) => "rejected: EMIT_TS_001",
            ResolvedType::Generic(_) => "rejected: EMIT_TS_001",
            ResolvedType::ConstGeneric(_) => "rejected: EMIT_TS_999",

            // ── Error / bottom types ──────────────────────────────────────
            ResolvedType::Unknown => "rejected: EMIT_TS_999",
            ResolvedType::Never => "rejected: EMIT_TS_999",

            // ── SIMD ──────────────────────────────────────────────────────
            ResolvedType::Vector { .. } => "rejected: EMIT_TS_013",

            // ── Trait-related ─────────────────────────────────────────────
            ResolvedType::DynTrait { .. } => "rejected: EMIT_TS_003",
            ResolvedType::Associated { .. } => "rejected: EMIT_TS_999",

            // ── Linear / affine ownership wrappers ───────────────────────
            ResolvedType::Linear(_) => "rejected: EMIT_TS_999",
            ResolvedType::Affine(_) => "rejected: EMIT_TS_999",

            // ── Refinement / dependent types ─────────────────────────────
            ResolvedType::Dependent { .. } => "rejected: EMIT_TS_014",

            // ── Lifetime parameters ───────────────────────────────────────
            ResolvedType::Lifetime(_) => "rejected: EMIT_TS_004",
        }
    }

    // Make the function "used" so the compiler doesn't elide the match.
    let _ = classify as fn(&ResolvedType) -> &'static str;
}

// ---------------------------------------------------------------------------
// Test 2 — Item exhaustiveness
// ---------------------------------------------------------------------------

#[test]
fn exhaustiveness_top_level_item() {
    fn classify(item: &Item) -> &'static str {
        match item {
            // ── Lowered to TS declarations ────────────────────────────────
            Item::Struct(_) => "lowered: interface (when pub) or skipped",
            Item::Enum(_) => "lowered: discriminated union (when pub) or skipped",
            Item::TypeAlias(_) => "lowered (when pub) or rejected EMIT_TS_008 if RHS unsupported",

            // ── Rejected with stable error codes ─────────────────────────
            Item::Function(_) => "rejected: EMIT_TS_007 (when pub)",
            Item::Union(_) => "rejected: EMIT_TS_015",
            Item::Trait(_) => "rejected: EMIT_TS_016",
            Item::Const(_) => "rejected: EMIT_TS_017",
            Item::Global(_) => "rejected: EMIT_TS_018",
            Item::Impl(_) => "rejected: EMIT_TS_019",
            Item::ExternBlock(_) => "rejected: EMIT_TS_020",
            Item::Macro(_) => "rejected: EMIT_TS_021",

            // ── Silently skipped ──────────────────────────────────────────
            // Use/import statements carry no type information; emit-ts ignores them.
            Item::Use(_) => "skipped: import carries no TS type info",
            // Trait aliases are structural sugar; not emitted.
            Item::TraitAlias(_) => "skipped: trait alias has no direct TS equivalent",
            // Error recovery nodes are parser artifacts; never reach the emitter.
            Item::Error { .. } => "skipped: parser error-recovery node",
        }
    }

    // Make the function "used" so the compiler doesn't elide the match.
    let _ = classify as fn(&Item) -> &'static str;
}
