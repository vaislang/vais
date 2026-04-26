//! Type-Tagged IR Builder — Phase Ω P1.4 (Pillar 1 final stage)
//!
//! This module is the future single API into which the 763-site ad-hoc IR
//! emission will collapse. iter 105 (this iter) introduces only the type
//! signatures and a working skeleton; no migration is performed.
//!
//! See `vaislang/vais-lang/packages/vaisdb/ROADMAP.md` iter 104 entry
//! for the recon-derived migration plan (iter 105~125, ~20–25 iter total).
//!
//! ## Design goals
//!
//! 1. **Type safety at build time.** Every emitted SSA temporary carries its
//!    LLVM type as a [`TypedTemp`] value. A caller who tries to use a
//!    temporary in a position whose type does not match the recorded one
//!    must go through an explicit cast (which itself produces a new
//!    [`TypedTemp`]). This eliminates the class of bugs where the IR string
//!    says one type but downstream callers assumed another.
//!
//! 2. **Automatic ground-truth registration.** Every emit method calls
//!    [`TypeRegistrar::record_emitted_type`] for the produced temporary.
//!    This collapses the 289 manual `record_emitted_type` call sites
//!    counted in iter 104 recon into the wrapper itself.
//!
//! 3. **Incremental migration.** The legacy `write_ir!` macro continues to
//!    work in untouched code. Migration proceeds site-by-site under R2/R3
//!    audit (ADR 0002).
//!
//! 4. **Text-codegen scope only.** The inkwell backend uses the inkwell
//!    Builder API directly and is out of scope for this wrapper. See iter
//!    104 recon: `inkwell/` contains 0 `write_ir!` invocations.
//!
//! ## API surface (iter 105 skeleton)
//!
//! - [`TypedTemp`]      — an SSA name paired with its LLVM type string
//! - [`LlvmType`]       — a thin newtype around the LLVM type string
//! - [`TypeRegistrar`]  — a trait abstracting the function-level type
//!   registry the emitter writes to. `FunctionContext` already exposes
//!   `record_emitted_type` / `get_emitted_type`; iter 106+ adds an `impl
//!   TypeRegistrar for FunctionContext` line and switches sites to the
//!   wrapper. iter 105 keeps the trait abstract so the unit tests in this
//!   module can run without constructing a full `FunctionContext`.
//! - [`TypedEmitter`]   — borrowed view that bundles `&mut String` (the
//!   IR buffer), `&mut dyn TypeRegistrar`, and `&mut usize` (the temp
//!   counter, matching the existing `next_temp` pattern from
//!   `helpers.rs`).
//!
//! ## What iter 105 does NOT do
//!
//! - It does not migrate any of the 5 if-coerce branches identified in iter
//!   104 recon (those are iter 106~108).
//! - It does not migrate any of the 697 raw IR emit sites (those are iter
//!   116~125).
//! - It does not introduce a new `record_emitted_type` automation pathway
//!   for existing call sites (those continue to call manually).
//! - It does not yet `impl TypeRegistrar for FunctionContext` — that
//!   one-line bridge is added in iter 106 alongside the first migrated
//!   call site, so the wrapper crosses the production boundary at the
//!   same iter as the first real caller.

// iter 105 introduces only the API surface; the production callers land in
// iter 106+ with the first migrated site. Until then, the items below have
// no non-test users, and the `dead_code` lint correctly notes that. This
// allow is removed in iter 106 as soon as a real caller appears.
#![allow(dead_code)]

use std::fmt::Write as _;

/// A thin wrapper around an LLVM type string.
///
/// This is *not* a parsed type — it is the textual form that will appear in
/// the emitted IR. The wrapper exists so that call sites read as
/// `LlvmType::from("i64")` rather than as untyped string literals, and so
/// that future additions (e.g., a structural-equality comparison that
/// normalizes whitespace) have a place to live.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct LlvmType(String);

impl LlvmType {
    /// Construct an `LlvmType` from any string-like value.
    pub(crate) fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// The wrapped LLVM type string.
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for LlvmType {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for LlvmType {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for LlvmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A named SSA temporary paired with its actually emitted LLVM type.
///
/// The `name` field is the textual SSA reference as it appears in the IR
/// (e.g., `"%t0"`, `"%5"`, `"%my_var"`). The `ty` field is the LLVM type
/// that the producing instruction stamped on this temporary.
///
/// Invariant: every `TypedTemp` returned by a [`TypedEmitter`] method has
/// already been registered via
/// [`TypeRegistrar::record_emitted_type`]. Callers do not need to (and
/// must not) call `record_emitted_type` again for the same temporary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TypedTemp {
    /// The SSA name as it appears in the IR (with the leading `%`).
    pub(crate) name: String,
    /// The LLVM type the producing instruction emitted.
    pub(crate) ty: LlvmType,
}

impl TypedTemp {
    /// Construct a `TypedTemp` *without* registering it with a registry.
    /// Reserved for callers that hold a temporary whose emission they did
    /// not perform (e.g., function parameters reified at entry).
    pub(crate) fn unregistered(name: impl Into<String>, ty: impl Into<LlvmType>) -> Self {
        Self {
            name: name.into(),
            ty: ty.into(),
        }
    }

    /// The SSA name with its leading `%`.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// The LLVM type the producing instruction emitted.
    pub(crate) fn ty(&self) -> &LlvmType {
        &self.ty
    }
}

impl std::fmt::Display for TypedTemp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

/// Abstraction over the function-level temporary-type registry that the
/// emitter writes to.
///
/// `FunctionContext` already provides `record_emitted_type(&str, &str)` and
/// `get_emitted_type(&str) -> Option<&str>` with these exact shapes. iter
/// 106 adds a one-line `impl TypeRegistrar for FunctionContext` blanket
/// next to the first migrated call site.
///
/// In tests inside this module we implement the trait on a tiny in-memory
/// `HashMap` so that emitter behavior can be verified without spinning up
/// a full `FunctionContext`.
pub(crate) trait TypeRegistrar {
    /// Record the LLVM type that the producing instruction stamped onto
    /// `name`. Called by [`TypedEmitter`] for every named SSA temporary
    /// before returning the [`TypedTemp`].
    fn record_emitted_type(&mut self, name: &str, llvm_ty: &str);

    /// Look up a previously recorded LLVM type, if any. Used by tests and
    /// (in iter 116+) by IR-emit sites that need to consult the
    /// ground-truth type of an SSA they did not produce.
    fn get_emitted_type(&self, name: &str) -> Option<&str>;
}

/// Borrowed view that bundles the IR output buffer with the type registry
/// and a temp-name counter, exposing type-tagged emit methods.
///
/// The lifetime parameter is the lifetime for which the caller has
/// exclusive access to all three borrowed pieces. A `TypedEmitter` is
/// constructed at the start of an emission sequence and dropped at the
/// end; it does not own anything.
///
/// ## Why an external `&mut usize` counter
///
/// The existing `helpers.rs::next_temp(counter: &mut usize)` already
/// receives the counter from the caller (see `vtable.rs:277` for an
/// example). Keeping `TypedEmitter` consistent with that convention means
/// migrated call sites can drop in the wrapper without restructuring how
/// they obtain a fresh name.
///
/// ## Invariants
///
/// - Every `emit_*` method that produces a named SSA temporary pushes
///   exactly one IR line (terminated by `\n`) into `ir`, allocates a
///   fresh name from the counter, and calls
///   [`TypeRegistrar::record_emitted_type`] for that name before
///   returning.
/// - The `TypedTemp` returned by an `emit_*` method has its `name` field
///   set to the freshly allocated SSA name and its `ty` field set to the
///   LLVM type the caller requested.
pub(crate) struct TypedEmitter<'a, R: TypeRegistrar + ?Sized> {
    ir: &'a mut String,
    registry: &'a mut R,
    counter: &'a mut usize,
}

impl<'a, R: TypeRegistrar + ?Sized> TypedEmitter<'a, R> {
    /// Construct a `TypedEmitter` over the three borrowed pieces of state.
    pub(crate) fn new(
        ir: &'a mut String,
        registry: &'a mut R,
        counter: &'a mut usize,
    ) -> Self {
        Self {
            ir,
            registry,
            counter,
        }
    }

    /// Allocate a fresh SSA name and emit a `call` instruction whose return
    /// type is `ret_ty`.
    ///
    /// The returned [`TypedTemp`] has been registered with the registry.
    /// The IR line is of the form
    /// `  %tN = call <ret_ty> <callee>(<arg_ty> <arg>, ...)`.
    pub(crate) fn emit_call(
        &mut self,
        ret_ty: LlvmType,
        callee: &str,
        args: &[(LlvmType, &str)],
    ) -> TypedTemp {
        let name = self.fresh_temp();
        let _ = write!(self.ir, "  {} = call {} {}(", name, ret_ty, callee);
        for (i, (ty, val)) in args.iter().enumerate() {
            if i > 0 {
                let _ = write!(self.ir, ", ");
            }
            let _ = write!(self.ir, "{} {}", ty, val);
        }
        let _ = writeln!(self.ir, ")");
        self.registry.record_emitted_type(&name, ret_ty.as_str());
        TypedTemp { name, ty: ret_ty }
    }

    /// Emit a void `call` instruction (the call has no return value).
    ///
    /// Returns no `TypedTemp`. The IR line is of the form
    /// `  call void <callee>(<arg_ty> <arg>, ...)`.
    pub(crate) fn emit_call_void(&mut self, callee: &str, args: &[(LlvmType, &str)]) {
        let _ = write!(self.ir, "  call void {}(", callee);
        for (i, (ty, val)) in args.iter().enumerate() {
            if i > 0 {
                let _ = write!(self.ir, ", ");
            }
            let _ = write!(self.ir, "{} {}", ty, val);
        }
        let _ = writeln!(self.ir, ")");
    }

    /// Allocate a fresh SSA name for the next temporary.
    ///
    /// Names are generated as `%t{counter}`, matching the `next_temp`
    /// helper in `helpers.rs:210`. iter 106+ migration sites that already
    /// use that helper drop into the wrapper without renaming.
    fn fresh_temp(&mut self) -> String {
        let n = *self.counter;
        *self.counter += 1;
        format!("%t{}", n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Minimal in-memory `TypeRegistrar` so that emitter tests do not need
    /// a full `FunctionContext`.
    #[derive(Default)]
    struct StubRegistry {
        types: HashMap<String, String>,
    }

    impl TypeRegistrar for StubRegistry {
        fn record_emitted_type(&mut self, name: &str, llvm_ty: &str) {
            self.types.insert(name.to_string(), llvm_ty.to_string());
        }

        fn get_emitted_type(&self, name: &str) -> Option<&str> {
            self.types.get(name).map(String::as_str)
        }
    }

    #[test]
    fn llvm_type_from_str() {
        let t = LlvmType::from("i64");
        assert_eq!(t.as_str(), "i64");
        assert_eq!(t.to_string(), "i64");
    }

    #[test]
    fn llvm_type_from_string() {
        let t: LlvmType = "i32".to_string().into();
        assert_eq!(t.as_str(), "i32");
    }

    #[test]
    fn llvm_type_equality() {
        assert_eq!(LlvmType::from("i64"), LlvmType::from("i64"));
        assert_ne!(LlvmType::from("i64"), LlvmType::from("i32"));
    }

    #[test]
    fn llvm_type_new_accepts_owned_or_borrowed() {
        let a = LlvmType::new("i8");
        let b = LlvmType::new(String::from("i8"));
        assert_eq!(a, b);
    }

    #[test]
    fn typed_temp_unregistered_does_not_touch_registry() {
        let registry = StubRegistry::default();
        let t = TypedTemp::unregistered("%arg.0", "i64");
        // `unregistered` is documented to not call `record_emitted_type`.
        assert!(registry.get_emitted_type(&t.name).is_none());
        assert_eq!(t.name(), "%arg.0");
        assert_eq!(t.ty().as_str(), "i64");
    }

    #[test]
    fn typed_temp_display_is_the_ssa_name() {
        let t = TypedTemp::unregistered("%my.temp.42", "i64");
        assert_eq!(format!("{}", t), "%my.temp.42");
    }

    #[test]
    fn emit_call_writes_one_line_and_registers_type() {
        let mut ir = String::new();
        let mut reg = StubRegistry::default();
        let mut counter: usize = 0;
        let result = {
            let mut te = TypedEmitter::new(&mut ir, &mut reg, &mut counter);
            te.emit_call(
                LlvmType::from("i64"),
                "@some_fn",
                &[(LlvmType::from("i64"), "42")],
            )
        };
        assert_eq!(ir, "  %t0 = call i64 @some_fn(i64 42)\n");
        assert_eq!(result.name(), "%t0");
        assert_eq!(result.ty().as_str(), "i64");
        assert_eq!(reg.get_emitted_type("%t0"), Some("i64"));
        assert_eq!(counter, 1);
    }

    #[test]
    fn emit_call_with_zero_args() {
        let mut ir = String::new();
        let mut reg = StubRegistry::default();
        let mut counter: usize = 0;
        let result = {
            let mut te = TypedEmitter::new(&mut ir, &mut reg, &mut counter);
            te.emit_call(LlvmType::from("i64"), "@noargs", &[])
        };
        assert_eq!(ir, "  %t0 = call i64 @noargs()\n");
        assert_eq!(result.ty().as_str(), "i64");
    }

    #[test]
    fn emit_call_with_multiple_args_uses_comma_separator() {
        let mut ir = String::new();
        let mut reg = StubRegistry::default();
        let mut counter: usize = 0;
        let _ = {
            let mut te = TypedEmitter::new(&mut ir, &mut reg, &mut counter);
            te.emit_call(
                LlvmType::from("i32"),
                "@add",
                &[
                    (LlvmType::from("i32"), "%a"),
                    (LlvmType::from("i32"), "%b"),
                ],
            )
        };
        assert_eq!(ir, "  %t0 = call i32 @add(i32 %a, i32 %b)\n");
    }

    #[test]
    fn emit_call_void_does_not_emit_assignment_and_does_not_register() {
        let mut ir = String::new();
        let mut reg = StubRegistry::default();
        let mut counter: usize = 0;
        {
            let mut te = TypedEmitter::new(&mut ir, &mut reg, &mut counter);
            te.emit_call_void("@noreturn", &[(LlvmType::from("i64"), "0")]);
        }
        assert_eq!(ir, "  call void @noreturn(i64 0)\n");
        assert_eq!(counter, 0);
        assert!(reg.get_emitted_type("%t0").is_none());
    }

    #[test]
    fn fresh_temp_counter_increments_across_calls() {
        let mut ir = String::new();
        let mut reg = StubRegistry::default();
        let mut counter: usize = 0;
        let (a, b) = {
            let mut te = TypedEmitter::new(&mut ir, &mut reg, &mut counter);
            let a = te.emit_call(LlvmType::from("i64"), "@f", &[]);
            let b = te.emit_call(LlvmType::from("i64"), "@g", &[]);
            (a, b)
        };
        assert_eq!(a.name(), "%t0");
        assert_eq!(b.name(), "%t1");
        assert_eq!(ir, "  %t0 = call i64 @f()\n  %t1 = call i64 @g()\n");
        assert_eq!(counter, 2);
    }

    #[test]
    fn emit_records_distinct_types_per_temp() {
        let mut ir = String::new();
        let mut reg = StubRegistry::default();
        let mut counter: usize = 0;
        {
            let mut te = TypedEmitter::new(&mut ir, &mut reg, &mut counter);
            te.emit_call(LlvmType::from("i64"), "@f", &[]);
            te.emit_call(LlvmType::from("i32"), "@g", &[]);
            te.emit_call(LlvmType::from("%MyStruct"), "@h", &[]);
        }
        assert_eq!(reg.get_emitted_type("%t0"), Some("i64"));
        assert_eq!(reg.get_emitted_type("%t1"), Some("i32"));
        assert_eq!(reg.get_emitted_type("%t2"), Some("%MyStruct"));
    }

    #[test]
    fn external_counter_seed_is_honored() {
        // A caller that has already allocated `%t0..%t4` elsewhere passes
        // counter=5 in. The next emitted name must be `%t5`, not `%t0`.
        let mut ir = String::new();
        let mut reg = StubRegistry::default();
        let mut counter: usize = 5;
        let result = {
            let mut te = TypedEmitter::new(&mut ir, &mut reg, &mut counter);
            te.emit_call(LlvmType::from("i64"), "@f", &[])
        };
        assert_eq!(result.name(), "%t5");
        assert_eq!(ir, "  %t5 = call i64 @f()\n");
        assert_eq!(counter, 6);
    }
}
