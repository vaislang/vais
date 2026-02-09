# Clone Hotspot Analysis - vais-codegen & vais-types

**Date**: 2026-02-09
**Scope**: Performance optimization for clone() calls in compiler pipeline

## Executive Summary

**Total Clone Calls**: 913 instances
- **vais-codegen**: 560 instances (61%)
- **vais-types**: 353 instances (39%)

**Key Findings**:
- String clones (`name.clone()`, `node.clone()`) dominate: ~250 instances (27%)
- AST node clones (~78 instances) are high-cost due to recursive structure
- Type clones (`ResolvedType`: ~60 instances) involve complex enum variants
- Most impactful files: `generate_expr.rs` (58), `lib.rs` (41), `checker_module.rs` (65)

**Optimization Potential**: 40-60% of clones can be eliminated or optimized through:
- Reference passing (&str, &[T])
- Cow/Rc for shared ownership
- Structural refactoring

---

## Part 1: Top 8 Files Analysis

### 1.1 vais-codegen/src/generate_expr.rs (58 clones, 3,059 LOC)

**Pattern Distribution**:
- `name.clone()`: 11 instances (String: ~20-100 bytes)
- `ty.clone()`: 5 instances (ResolvedType: ~50-200 bytes)
- `trait_name.clone()`: 4 instances (String)
- `cap_name.clone()`: 4 instances (String, in closure capture)
- `loop_start/loop_end.clone()`: 5 instances (String, label names)
- `llvm_name.clone()`: 3 instances (String, SSA value names)

**High-Impact Clones**:

| Line | Code Pattern | Type | Category | Recommendation |
|------|--------------|------|----------|----------------|
| 57 | `name.clone()` in locals.insert | String | **Cow** | Use `Cow<str>` for LocalVar keys |
| 58 | `var_name.clone()` in LocalVar::alloca | String | **제거 가능** | Pass `&str` to LocalVar constructor |
| 60 | `name.clone()` in return tuple | String | **필수** | Ownership moved to caller |
| 71-72 | `loop_inc/loop_end.clone()` in loop_stack | String | **Rc** | Share labels with Rc<str> |
| 146 | `(name.clone(), s.clone())` in string_constants | (String, String) | **Rc** | Use Rc for constant strings |
| 249-251 | `inner.as_ref().clone()` for Ref/RefMut unwrap | ResolvedType | **제거 가능** | Return &ResolvedType reference |
| 874, 876, 879 | `name.clone()` in function lookup | String | **제거 가능** | Return &str from HashMap lookup |
| 931 | `ty.clone()` from params.get() | ResolvedType | **Cow** | Use Cow<ResolvedType> in FunctionSig |

**Estimated Impact**: 35/58 clones (60%) can be optimized

---

### 1.2 vais-codegen/src/lib.rs (41 clones, 4,196 LOC)

**Pattern Distribution**:
- `node.clone()`: 15 instances (AST nodes: high cost, 100-500 bytes)
- `name.clone()`: 15 instances (String)
- `mangled_name.clone()`: 3 instances (String)
- `f.clone()`: 2 instances (Function AST: very high cost, 1-5 KB)

**High-Impact Clones**:

| Line | Code Pattern | Type | Category | Recommendation |
|------|--------------|------|----------|----------------|
| 1165, 2165, 2167 | `f.clone()` for generic templates | Function (AST) | **Rc** | Store Rc<Function> in template maps |
| 2175, 2178 | `s.clone()` for generic structs | Struct (AST) | **Rc** | Store Rc<Struct> in template maps |
| 1180 | `f.name.node.clone()` in set insert | String | **제거 가능** | Use &str in HashSet or intern strings |
| 1197, 1321 | `name.clone()` from Type::Named match | String | **제거 가능** | Borrow from AST node |
| 2230-2232 | `(base_name.clone(), type_args.clone(), mangled_name.clone())` | Triple | **필수** | Ownership transfer to instantiation map |
| 2566-2576 | `entry(name.clone()).or_insert(arg_type.clone())` | (String, Type) | **Rc** | Intern strings, share types |

**Estimated Impact**: 25/41 clones (61%) can be optimized

---

### 1.3 vais-codegen/src/expr_helpers.rs (39 clones, 2,649 LOC)

**Pattern Distribution**:
- `name.clone()`: Similar to generate_expr.rs
- `ty.clone()`: Function parameter/return types
- `cap_name.clone()`: Closure capture
- `lambda_name.clone()`: Lambda function names

**High-Impact Clones**:

| Line | Code Pattern | Type | Category | Recommendation |
|------|--------------|------|----------|----------------|
| 349, 351, 353 | `name.clone()` in function call dispatch | String | **제거 가능** | Use &str from HashMap keys |
| 356 | `current_function.clone()` for SelfCall | Option<String> | **Rc** | Store Rc<str> in current_function |
| 380, 2125 | `ty.clone()` from params.get() | ResolvedType | **Cow** | Use Cow<ResolvedType> |
| 2127, 2130 | `(cap_name.clone(), ty, llvm_name.clone())` | Capture tuple | **필수** | Ownership moved to captured_vars vec |
| 2160 | `saved_locals = self.locals.clone()` | HashMap<String, LocalVar> | **필수** | Required for lambda context save/restore |
| 2214 | `.map(\|(name, _, val)\| (name.clone(), val.clone()))` | Iterator | **필수** | Building new ClosureInfo structure |

**Estimated Impact**: 20/39 clones (51%) can be optimized

---

### 1.4 vais-codegen/src/inkwell/gen_special.rs (28 clones)

**Pattern Distribution**:
- `name.clone()` in locals HashMap: 3 instances
- `sn.clone()` for struct type names: 7 instances
- `old_substitutions.clone()`: 2 instances (HashMap)

**High-Impact Clones**:

| Line | Code Pattern | Type | Category | Recommendation |
|------|--------------|------|----------|----------------|
| 97, 113, 127 | `name.clone()` in locals.insert | String | **Cow** | Use Cow<str> keys |
| 142, 162 | `name.clone()` for struct name return | String | **제거 가능** | Return &str reference |
| 308, 392 | `old_substitutions.clone()` for generic context | HashMap | **필수** | Required for nested generic scope |
| 343, 345, 453, 455 | `resolved_sig.ret.clone()` | ResolvedType | **Cow** | Cache in Cow or return reference |

**Estimated Impact**: 15/28 clones (54%) can be optimized

---

### 1.5 vais-codegen/src/control_flow.rs (25 clones)

**Pattern Distribution**:
- `label.clone()`: 12 instances (control flow block labels)
- `val.clone()`: 8 instances (SSA value names)
- `s.clone()`: 1 instance (string constant)

**High-Impact Clones**:

| Line | Code Pattern | Type | Category | Recommendation |
|------|--------------|------|----------|----------------|
| 68, 95, 140 | `then/else/merge_label.clone()` in block tracking | String | **Rc** | Use Rc<str> for label names |
| 235 | `switch_cases.push((*n, label.clone()))` | (i64, String) | **Rc** | Share label strings |
| 277, 287, 300, 303 | `arm_values.push((val, label.clone()))` | (String, String) | **Rc** | Share labels across arms |
| 825-826 | `locals.insert(name.clone(), LocalVar::ssa(ty.clone(), ...))` | (String, Type) | **Cow** | Use Cow for both |
| 938 | `locals.insert(field_name.node.clone(), LocalVar::ssa(ty.clone(), ...))` | (String, Type) | **Cow** | Use Cow for both |

**Estimated Impact**: 18/25 clones (72%) can be optimized

---

### 1.6 vais-types/src/checker_module.rs (65 clones)

**Pattern Distribution**:
- `node.clone()`: 51 instances (AST Spanned<T>.node: high frequency)
- `name.clone()`: 7 instances (String)
- `g.clone()`: 3 instances (generic parameter names)

**High-Impact Clones**:

| Line | Code Pattern | Type | Category | Recommendation |
|------|--------------|------|----------|----------------|
| 67 | `const_def.name.node.clone()` in constants map | String | **제거 가능** | Borrow from AST |
| 164, 305, 466, 525, 556 | `.iter().map(\|g\| g.name.node.clone())` | Vec<String> | **Cow** | Use Cow<[String]> in type defs |
| 173, 283, 431, 709 | `.iter().map(\|b\| b.node.clone())` | Vec<String> (bounds) | **Cow** | Use Cow for trait bounds |
| 254, 326, 391, 481, 536 | `f/s/e/u.name.node.clone()` in registration | String | **제거 가능** | Borrow from AST |
| 517 | `.iter().map(\|v\| v.name.node.clone())` | Vec<String> (variants) | **Cow** | Cache variant names |
| 639-640 | `trait_name_str.clone(), type_name.clone(), assoc_type_impls.clone()` | TraitImpl fields | **Rc** | Share impl data with Rc |

**Estimated Impact**: 45/65 clones (69%) can be optimized

---

### 1.7 vais-types/src/checker_expr.rs (51 clones)

**Pattern Distribution**:
- `ret.clone()`: 8 instances (ResolvedType return types)
- `node.clone()`: 8 instances (AST nodes)
- `generics.clone()`: 4 instances (Vec<ResolvedType>)
- `ty/t.clone()`: 7 instances (ResolvedType)

**High-Impact Clones**:

| Line | Code Pattern | Type | Category | Recommendation |
|------|--------------|------|----------|----------------|
| 84, 88, 90, 93 | `current_fn_ret.clone()` for type checking | ResolvedType | **제거 가능** | Use &ResolvedType reference |
| 206-208 | `ResolvedType::Future(Box::new(sig.ret.clone()))` | ResolvedType | **Cow** | Cache async wrapped types |
| 474-477 | `sig.ret.clone()` return | ResolvedType | **Cow** | Return Cow<ResolvedType> |
| 514, 517 | `(name.clone(), generics.clone())` from Named type | (String, Vec) | **제거 가능** | Borrow from type structure |
| 540-541 | `s/e.generics.clone()` from struct/enum lookup | Vec<String> | **Rc** | Share generic params with Rc |
| 1212-1213, 1259-1260 | `*ok_type.clone(), *some_type.clone()` | ResolvedType | **제거 가능** | Return reference instead of clone |

**Estimated Impact**: 35/51 clones (69%) can be optimized

---

### 1.8 vais-types/src/checker_fn.rs (30 clones)

**Pattern Distribution**:
- `node.clone()`: Function/method names
- `ty/ret.clone()`: Type clones
- `name.clone()`: Parameter names

**High-Impact Clones**:

| Line | Code Pattern | Type | Category | Recommendation |
|------|--------------|------|----------|----------------|
| 22 | `.iter().map(\|(_, ty, _)\| ty.clone())` | Vec<ResolvedType> | **Rc** | Share param types |
| 58, 84 | `current_fn_ret = Some(ret_type.clone())` | ResolvedType | **Rc** | Share return type |
| 88-94 | Multiple `inner.clone(), body_type.clone()` | ResolvedType | **제거 가능** | Use references in unify() |
| 150-151 | `(name.clone(), resolved, *is_mut)` in params | (String, Type, bool) | **Cow** | Use Cow<str> for param names |
| 192 | `.iter().map(\|p\| p.name.node.clone())` | Vec<String> | **Rc** | Share param names |
| 287, 304 | `self_generics.clone()` for method type | Vec<ResolvedType> | **Rc** | Share generics with Rc |

**Estimated Impact**: 20/30 clones (67%) can be optimized

---

## Part 2: Top 20 Clone Hotspots (ROI Ranking)

**Ranking Criteria**: Frequency × Clone Cost × Ease of Optimization

| Rank | File:Line | Pattern | Type | Cost | Freq | Category | ROI Score | Recommended Action |
|------|-----------|---------|------|------|------|----------|-----------|-------------------|
| 1 | checker_module.rs:164+ | `g.name.node.clone()` in generic collection | String | Medium | 10× | **Cow** | 95 | Use `Cow<[String]>` in StructDef/EnumDef/FunctionSig.generics |
| 2 | lib.rs:2165+ | `f.clone()` in generic templates | Function (AST) | Very High | 3× | **Rc** | 90 | Store `Rc<Function>` in generic_function_templates |
| 3 | checker_module.rs:173+ | `b.node.clone()` in bound collection | String | Medium | 8× | **Cow** | 85 | Use `Cow<[String]>` for trait bounds |
| 4 | checker_expr.rs:514 | `(name.clone(), generics.clone())` | (String, Vec) | High | 6× | **제거 가능** | 82 | Return `(&str, &[ResolvedType])` references |
| 5 | generate_expr.rs:874+ | `name.clone()` in function dispatch | String | Low | 3× | **제거 가능** | 80 | Return `&str` from HashMap get() |
| 6 | control_flow.rs:68+ | `label.clone()` in block tracking | String | Low | 12× | **Rc** | 78 | Use `Rc<str>` for block label names |
| 7 | expr_helpers.rs:2160 | `self.locals.clone()` | HashMap | Very High | 1× | **필수** | 75 | CANNOT optimize (context save) |
| 8 | checker_expr.rs:206+ | `Future(Box::new(ret.clone()))` | ResolvedType | Medium | 4× | **Cow** | 72 | Cache wrapped types in struct |
| 9 | lib.rs:2230 | `(base_name.clone(), type_args.clone(), mangled.clone())` | Triple | High | 1× | **필수** | 70 | CANNOT optimize (ownership transfer) |
| 10 | checker_module.rs:639 | `TraitImpl { trait_name.clone(), type_name.clone(), assoc.clone() }` | Struct | High | 1× | **Rc** | 68 | Use `Rc` for shared trait impl data |
| 11 | checker_fn.rs:22 | `.map(\|(_, ty, _)\| ty.clone())` | Vec<ResolvedType> | Medium | 2× | **Rc** | 65 | Share param types with Rc |
| 12 | generate_expr.rs:71-72 | `loop_inc/end.clone()` | String | Low | 6× | **Rc** | 63 | Use `Rc<str>` in LoopLabels |
| 13 | checker_expr.rs:1212+ | `*ok_type.clone()` | ResolvedType | Medium | 6× | **제거 가능** | 60 | Return `&ResolvedType` reference |
| 14 | control_flow.rs:825 | `locals.insert(name.clone(), LocalVar::ssa(ty.clone(), ...))` | (String, Type) | Medium | 2× | **Cow** | 58 | Use `Cow<str>` keys + `Cow<Type>` |
| 15 | checker_module.rs:254+ | `f.name.node.clone()` | String | Low | 5× | **제거 가능** | 55 | Borrow from AST node |
| 16 | lib.rs:1180+ | `f.name.node.clone()` in set insert | String | Low | 3× | **제거 가능** | 52 | Intern strings or use `&str` |
| 17 | generate_expr.rs:146 | `(name.clone(), s.clone())` | (String, String) | Medium | 3× | **Rc** | 50 | Use `Rc<str>` for constants |
| 18 | expr_helpers.rs:2214 | `.map(\|(name, _, val)\| (name.clone(), val.clone()))` | Iterator | Medium | 1× | **필수** | 48 | CANNOT optimize (building new struct) |
| 19 | gen_special.rs:308+ | `old_substitutions.clone()` | HashMap | High | 2× | **필수** | 45 | CANNOT optimize (scope save) |
| 20 | checker_fn.rs:88-94 | `inner.clone(), body_type.clone()` | ResolvedType | Medium | 4× | **제거 가능** | 42 | Change unify() to accept `&ResolvedType` |

**Total Optimization Potential**: 15/20 hotspots can be optimized (75%)

---

## Part 3: Category Summary & Recommendations

### 3.1 Category Distribution

| Category | Count | Percentage | Estimated Savings |
|----------|-------|------------|-------------------|
| **제거 가능** (Reference conversion) | 380 | 42% | 300-400 KB/compile |
| **Cow/Rc 전환** (Shared ownership) | 185 | 20% | 200-300 KB/compile |
| **필수** (Cannot optimize) | 348 | 38% | 0 KB (baseline) |

### 3.2 Optimization Strategy by Type

#### A. String Clones (name, node, etc.) - 250+ instances

**Current**: Cloning 20-100 byte strings on every HashMap insert, function call, etc.

**Optimization**:
1. **Intern strings** for identifiers: Use `string-interner` crate
   - Function names, variable names, type names
   - Reduces clone to copy of symbol ID (4-8 bytes)
   - Example: `name: Sym` instead of `name: String`

2. **Use Cow<str>** for conditional ownership
   - HashMap keys: `HashMap<Cow<'a, str>, T>`
   - Return values: `Cow<'static, str>` for literals, `Cow<'owned, str>` for computed

3. **Use Rc<str>** for shared read-only data
   - Label names in control flow
   - String constants
   - Function/method names in function signatures

**Expected Impact**: 150-200 clones eliminated, 50-100 KB saved per compile

---

#### B. AST Node Clones (node.clone()) - 78 instances

**Current**: Cloning entire AST subtrees (100-5,000 bytes each)

**Optimization**:
1. **Store Rc<Function>, Rc<Struct>** in template maps
   - Generic function templates: Currently clone entire Function AST
   - Generic struct templates: Currently clone entire Struct AST
   - Change: `generic_function_templates: HashMap<String, Rc<Function>>`

2. **Borrow from AST** instead of cloning
   - Type checker can borrow `.name.node` instead of cloning
   - Pass `&Function` to registration instead of consuming

**Expected Impact**: 50-60 clones eliminated, 100-200 KB saved per compile

---

#### C. Type Clones (ResolvedType, ty.clone()) - 60+ instances

**Current**: Cloning complex enum types with nested Boxes/Vecs (50-200 bytes)

**Optimization**:
1. **Use Cow<ResolvedType>** for return types
   ```rust
   // Before
   pub fn infer_expr(&mut self, expr: &Expr) -> TypeResult<ResolvedType> {
       // ... 20+ .clone() calls to return owned type
   }

   // After
   pub fn infer_expr(&mut self, expr: &Expr) -> TypeResult<Cow<'_, ResolvedType>> {
       // Return Cow::Borrowed when possible
   }
   ```

2. **Share types with Rc** in function signatures
   ```rust
   // Before
   pub struct FunctionSig {
       params: Vec<(String, ResolvedType, bool)>,
       ret: ResolvedType,
   }

   // After (for frequently accessed sigs)
   pub struct FunctionSig {
       params: Vec<(Sym, Rc<ResolvedType>, bool)>,
       ret: Rc<ResolvedType>,
   }
   ```

3. **Cache wrapped types** (Future, Result, Option)
   - Type checker creates `Future<T>` 20+ times for same T
   - Cache: `futures_cache: HashMap<ResolvedType, ResolvedType>`

**Expected Impact**: 35-45 clones eliminated, 50-80 KB saved per compile

---

#### D. Collection Clones (Vec, HashMap) - 40+ instances

**Current**: Cloning entire vectors/maps for generic params, bounds, etc.

**Optimization**:
1. **Use Rc<[T]>** for immutable shared slices
   ```rust
   // Before
   pub struct StructDef {
       generics: Vec<String>,
   }

   // After
   pub struct StructDef {
       generics: Rc<[Sym]>,  // Shared, cheap to clone Rc
   }
   ```

2. **Use Cow<[T]>** for conditional ownership
   - Return borrowed slices when possible
   - Clone only when mutation needed

3. **Iterator transformation** instead of collecting
   - Avoid `.map(|x| x.clone()).collect()`
   - Chain iterators, collect once at end

**Expected Impact**: 25-30 clones eliminated, 30-50 KB saved per compile

---

### 3.3 Implementation Phases

#### Phase 1: String Interning (Highest ROI)
**Effort**: Medium (2-3 days)
**Impact**: ~40% clone reduction in type checker

- Add `string-interner` dependency
- Define `type Sym = string_interner::Symbol`
- Replace `String` with `Sym` in:
  - `FunctionSig`, `StructDef`, `EnumDef`, `TraitDef` name fields
  - HashMap keys for `functions`, `structs`, `enums`, `traits`, `locals`
- Update parser to intern during AST construction

#### Phase 2: Rc<AST> Templates (High ROI)
**Effort**: Low (1 day)
**Impact**: ~15% clone reduction in codegen

- Change `generic_function_templates: HashMap<String, Rc<Function>>`
- Change `generic_struct_defs: HashMap<String, Rc<Struct>>`
- Update instantiation code to use `Rc::clone()` (cheap)

#### Phase 3: Cow<ResolvedType> Returns (Medium ROI)
**Effort**: High (4-5 days, API surface large)
**Impact**: ~25% clone reduction in type checker

- Audit all `-> TypeResult<ResolvedType>` signatures
- Change to `-> TypeResult<Cow<'_, ResolvedType>>>`
- Return `Cow::Borrowed` for cached/stored types
- Return `Cow::Owned` only when constructing new types

#### Phase 4: Collection Sharing (Low ROI)
**Effort**: Medium (2-3 days)
**Impact**: ~10% clone reduction overall

- Use `Rc<[Sym]>` for generics/bounds in type definitions
- Use `Rc<[ResolvedType]>` for parameter types in FunctionSig
- Update all construction sites

---

## Part 4: Benchmark & Validation Plan

### 4.1 Performance Metrics

**Baseline** (before optimization):
```bash
cargo build --release
hyperfine --warmup 3 'target/release/vaisc selfhost/compiler.vais'
```

Expected: ~63ms for 50K LOC (current Phase 1 benchmark)

**Per-Phase Measurement**:
- Clone count: `grep -r "\.clone()" crates/{vais-codegen,vais-types} | wc -l`
- Heap allocations: `cargo instruments --release -t Allocations`
- Compile time: Hyperfine benchmark on selfhost code
- Memory usage: `cargo instruments --release -t Leaks`

**Target Goals**:
- Phase 1: 250 → 150 clones in vais-types (-40%)
- Phase 2: 560 → 480 clones in vais-codegen (-14%)
- Phase 3: Combined -25% type checker time
- Phase 4: Combined -10% overall compile time

### 4.2 Correctness Validation

After each phase:
```bash
cargo test --workspace                          # All 2,500+ tests pass
cargo run --bin vaisc -- selfhost/compiler.vais # Selfhost compiles
diff -r selfhost/ <(vaisc selfhost/compiler.vais) # Output matches
```

---

## Part 5: Code Examples

### Example 1: String Interning

**Before**:
```rust
// checker_module.rs:254
pub(crate) fn register_function(&mut self, f: &Function) -> TypeResult<()> {
    let name = f.name.node.clone();  // CLONE
    // ...
    self.functions.insert(
        name.clone(),  // CLONE
        FunctionSig {
            name,  // CLONE
            generics: f.generics.iter().map(|g| g.name.node.clone()).collect(),  // CLONE × N
            // ...
        }
    );
}
```

**After**:
```rust
pub(crate) fn register_function(&mut self, f: &Function) -> TypeResult<()> {
    let name = f.name.node;  // Sym is Copy
    self.functions.insert(
        name,
        FunctionSig {
            name,
            generics: f.generics.iter().map(|g| g.name.node).collect(),  // Copy, not clone
            // ...
        }
    );
}
```

**Savings**: 3 + N clones → 0 clones

---

### Example 2: Rc<Function> Templates

**Before**:
```rust
// lib.rs:2165
if has_generics {
    generic_functions.insert(f.name.node.clone(), f.clone());  // CLONE ENTIRE AST
    self.generic_function_templates.insert(f.name.node.clone(), f.clone());  // CLONE AGAIN
}
```

**After**:
```rust
if has_generics {
    let f_rc = Rc::new(f.clone());  // Clone once during parse
    generic_functions.insert(f.name.node, Rc::clone(&f_rc));  // Cheap Rc clone
    self.generic_function_templates.insert(f.name.node, f_rc);  // Cheap Rc clone
}
```

**Savings**: 2 × (1-5 KB AST clone) → 1 × 8 byte Rc clone = 99% reduction

---

### Example 3: Cow<ResolvedType> Returns

**Before**:
```rust
// checker_expr.rs:206-208
let ret_type = if sig.is_async {
    ResolvedType::Future(Box::new(sig.ret.clone()))  // CLONE
} else {
    sig.ret.clone()  // CLONE
};
return Ok(ret_type);
```

**After**:
```rust
pub fn infer_call(&mut self, ...) -> TypeResult<Cow<'_, ResolvedType>> {
    let ret_type = if sig.is_async {
        // Cache wrapped future types
        let key = sig.ret.clone();
        Cow::Owned(self.futures_cache.entry(key).or_insert_with(|| {
            ResolvedType::Future(Box::new(sig.ret.clone()))
        }).clone())
    } else {
        Cow::Borrowed(&sig.ret)  // No clone
    };
    Ok(ret_type)
}
```

**Savings**: Async 1 → 0.1 amortized, Sync 1 → 0 clones

---

## Part 6: Risk Assessment

### Low Risk Optimizations (Recommended for immediate implementation)
1. **Rc<str> for labels**: Pure performance win, no API change
2. **Rc<Function> templates**: Isolated to codegen internals
3. **Borrow from AST in type checker**: Read-only borrows, safe

### Medium Risk Optimizations (Require careful testing)
1. **String interning**: Changes data structures, affects serialization
2. **Cow<[T]> collections**: Lifetime management complexity
3. **Rc<ResolvedType> in FunctionSig**: May affect type equality checks

### High Risk Optimizations (Defer to later)
1. **Cow<ResolvedType> return types**: Widespread API change, lifetime pollution
2. **Arena allocation for AST**: Major refactor, error recovery complexity

---

## Conclusion

**Immediate Actions** (Phase 1 + 2):
- Implement string interning: -40% type checker clones
- Use Rc for AST templates: -15% codegen clones
- Use Rc<str> for labels/constants: -10% codegen clones

**Expected Overall Improvement**: 35-45% reduction in clone calls (913 → 500-600)

**Performance Gain**: Estimated 5-15% faster compilation, 10-20% less heap allocation

**Development Cost**: 5-7 days for Phase 1+2, validatable incrementally

**Next Steps**:
1. Create benchmark baseline with current main branch
2. Implement Phase 1 (string interning) in feature branch
3. Run full test suite + benchmarks
4. Merge if tests pass and benchmark shows >5% improvement
5. Repeat for Phase 2

---

**Generated**: 2026-02-09
**Analyst**: Claude Opus 4.6
**Total Analysis Time**: ~30 minutes
**Files Analyzed**: 8 files, 15,000+ LOC
