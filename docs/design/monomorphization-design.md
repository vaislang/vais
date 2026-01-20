# Monomorphization Design for Vais

## Overview

Vais will implement **monomorphization** (also known as "specialization") for generics, similar to Rust and C++. Each unique combination of generic type parameters will generate a specialized version of the function or struct at compile time.

## Current State

Currently, Vais uses **type erasure**: all generic type parameters are converted to `i64` at runtime.

```vais
F identity<T>(x: T) -> T = x
# Currently generates single: @identity(i64 %x) -> i64
```

## Target State

After monomorphization, each unique instantiation will generate its own code:

```vais
F identity<T>(x: T) -> T = x

# Usage:
identity(42)      # instantiates identity<i64>
identity(3.14)    # instantiates identity<f64>
identity(point)   # instantiates identity<Point>
```

Generated LLVM:
```llvm
define i64 @identity$i64(i64 %x) { ret i64 %x }
define double @identity$f64(double %x) { ret double %x }
define %Point @identity$Point(%Point %x) { ret %Point %x }
```

## Architecture Changes

### 1. Type Checker Changes (vais-types)

#### 1.1 Generic Instantiation Tracking

```rust
// New struct to track generic instantiations needed
pub struct GenericInstantiation {
    pub base_name: String,              // e.g., "identity", "Vec"
    pub type_args: Vec<ResolvedType>,   // e.g., [i64], [f64]
    pub mangled_name: String,           // e.g., "identity$i64", "Vec$i64"
}

// Add to TypeChecker
pub struct TypeChecker {
    // ... existing fields ...

    // Track all generic instantiations needed
    generic_instantiations: Vec<GenericInstantiation>,

    // Generic function definitions (uninstantiated)
    generic_functions: HashMap<String, GenericFunctionDef>,

    // Generic struct definitions (uninstantiated)
    generic_structs: HashMap<String, GenericStructDef>,
}
```

#### 1.2 Type Inference for Generics

When a generic function is called, infer the concrete types:

```rust
fn check_generic_call(&mut self, func_name: &str, args: &[Expr]) -> TypeResult<ResolvedType> {
    let generic_fn = self.generic_functions.get(func_name)?;

    // Infer type arguments from argument types
    let mut type_args = HashMap::new();
    for (param, arg) in generic_fn.params.iter().zip(args) {
        let arg_type = self.check_expr(arg)?;
        self.infer_type_arg(&param.ty, &arg_type, &mut type_args)?;
    }

    // Create instantiation
    let type_arg_list: Vec<_> = generic_fn.generics.iter()
        .map(|g| type_args.get(&g.name).cloned().unwrap_or(ResolvedType::I64))
        .collect();

    let mangled = mangle_name(func_name, &type_arg_list);
    self.generic_instantiations.push(GenericInstantiation {
        base_name: func_name.to_string(),
        type_args: type_arg_list.clone(),
        mangled_name: mangled.clone(),
    });

    // Return substituted return type
    substitute_generics(&generic_fn.ret, &type_args)
}
```

### 2. Code Generator Changes (vais-codegen)

#### 2.1 Name Mangling

```rust
fn mangle_name(base: &str, type_args: &[ResolvedType]) -> String {
    if type_args.is_empty() {
        base.to_string()
    } else {
        let args_str = type_args.iter()
            .map(|t| mangle_type(t))
            .collect::<Vec<_>>()
            .join("_");
        format!("{}${}", base, args_str)
    }
}

fn mangle_type(ty: &ResolvedType) -> String {
    match ty {
        ResolvedType::I8 => "i8".to_string(),
        ResolvedType::I16 => "i16".to_string(),
        ResolvedType::I32 => "i32".to_string(),
        ResolvedType::I64 => "i64".to_string(),
        ResolvedType::F32 => "f32".to_string(),
        ResolvedType::F64 => "f64".to_string(),
        ResolvedType::Bool => "bool".to_string(),
        ResolvedType::Str => "str".to_string(),
        ResolvedType::Named { name, generics } => {
            if generics.is_empty() {
                name.clone()
            } else {
                let args = generics.iter()
                    .map(|g| mangle_type(g))
                    .collect::<Vec<_>>()
                    .join("_");
                format!("{}_{}", name, args)
            }
        }
        _ => "unknown".to_string(),
    }
}
```

#### 2.2 Generic Function Generation

```rust
fn generate_generic_instantiations(&mut self) -> CodegenResult<String> {
    let mut ir = String::new();

    for inst in &self.generic_instantiations {
        if let Some(generic_fn) = self.generic_functions.get(&inst.base_name) {
            // Create type substitution map
            let mut subst = HashMap::new();
            for (param, arg) in generic_fn.generics.iter().zip(&inst.type_args) {
                subst.insert(param.name.clone(), arg.clone());
            }

            // Generate specialized function
            ir.push_str(&self.generate_specialized_function(
                &inst.mangled_name,
                generic_fn,
                &subst,
            )?);
        }
    }

    Ok(ir)
}
```

#### 2.3 Generic Struct Generation

```rust
fn generate_generic_struct(&mut self, inst: &GenericInstantiation) -> CodegenResult<String> {
    let generic_struct = self.generic_structs.get(&inst.base_name)?;

    // Create type substitution
    let mut subst = HashMap::new();
    for (param, arg) in generic_struct.generics.iter().zip(&inst.type_args) {
        subst.insert(param.name.clone(), arg.clone());
    }

    // Generate specialized struct type
    let fields: Vec<_> = generic_struct.fields.iter()
        .map(|(name, ty)| {
            let concrete_ty = substitute_type(ty, &subst);
            (name.clone(), self.type_to_llvm(&concrete_ty))
        })
        .collect();

    let struct_ir = format!(
        "%{} = type {{ {} }}\n",
        inst.mangled_name,
        fields.iter().map(|(_, t)| t.as_str()).collect::<Vec<_>>().join(", ")
    );

    Ok(struct_ir)
}
```

### 3. Standard Library Changes

#### 3.1 Vec<T> Definition

```vais
# Vec<T> - generic dynamic array
S Vec<T> {
    data: i64,      # Pointer to T array (raw pointer)
    len: i64,       # Current number of elements
    cap: i64,       # Allocated capacity
    elem_size: i64  # Size of T in bytes (for generic support)
}

X Vec<T> {
    F with_capacity(capacity: i64) -> Vec<T> {
        elem_size := sizeof(T)
        data := malloc(capacity * elem_size)
        Vec<T> { data: data, len: 0, cap: capacity, elem_size: elem_size }
    }

    F push(&self, value: T) -> i64 {
        I self.len >= self.cap {
            @.grow()
        }
        ptr := self.data + self.len * self.elem_size
        store(ptr, value)  # Generic store
        self.len = self.len + 1
        self.len
    }

    F get(&self, index: i64) -> T {
        ptr := self.data + index * self.elem_size
        load(ptr)  # Generic load
    }

    # ... other methods
}
```

#### 3.2 HashMap<K, V> Definition

```vais
# Entry<K, V> - linked list node
S Entry<K, V> {
    key: K,
    value: V,
    next: i64       # Pointer to next Entry<K, V>
}

# HashMap<K, V> - generic hash map
S HashMap<K, V> {
    buckets: i64,   # Pointer to array of Entry<K, V> pointers
    size: i64,
    cap: i64,
    key_size: i64,
    val_size: i64
}

X HashMap<K, V> {
    F with_capacity(capacity: i64) -> HashMap<K, V> {
        cap := capacity
        I cap < 8 { cap = 8 }
        buckets := malloc(cap * 8)
        # Initialize buckets to null
        i := 0
        L {
            I i >= cap { B 0 }
            store_i64(buckets + i * 8, 0)
            i = i + 1
        }
        HashMap<K, V> {
            buckets: buckets,
            size: 0,
            cap: cap,
            key_size: sizeof(K),
            val_size: sizeof(V)
        }
    }

    # ... other methods with K, V types
}
```

### 4. Built-in Generic Functions

#### 4.1 sizeof<T>

Returns the size in bytes of type T at compile time:

```vais
sizeof(i8)    # => 1
sizeof(i64)   # => 8
sizeof(f64)   # => 8
sizeof(Point) # => struct size
```

Implementation in codegen:
```rust
fn builtin_sizeof(&self, ty: &ResolvedType) -> i64 {
    match ty {
        ResolvedType::I8 | ResolvedType::Bool => 1,
        ResolvedType::I16 => 2,
        ResolvedType::I32 | ResolvedType::F32 => 4,
        ResolvedType::I64 | ResolvedType::F64 => 8,
        ResolvedType::Named { name, .. } => {
            self.structs.get(name)
                .map(|s| s.fields.iter().map(|(_, t)| self.type_size(t)).sum())
                .unwrap_or(8)
        }
        _ => 8, // Default pointer size
    }
}
```

#### 4.2 Generic load/store

```rust
// Generic load: read T from memory address
fn builtin_load<T>(ptr: i64) -> T

// Generic store: write T to memory address
fn builtin_store<T>(ptr: i64, value: T)
```

### 5. Implementation Phases

#### Phase 1: Infrastructure
1. Add `GenericInstantiation` tracking to TypeChecker
2. Implement name mangling utilities
3. Add `sizeof` built-in function

#### Phase 2: Type Checker
1. Modify `check_call` to handle generic functions
2. Implement type argument inference
3. Track required instantiations

#### Phase 3: Code Generator
1. Generate specialized structs
2. Generate specialized functions
3. Update function calls to use mangled names

#### Phase 4: Standard Library
1. Convert Vec to Vec<T>
2. Convert HashMap to HashMap<K, V>
3. Add tests for generic collections

### 6. Limitations & Future Work

#### Current Limitations
- No higher-kinded types (no `Monad<F<_>>`)
- No specialization (no `impl<T: Copy>` vs `impl<T>`)
- No associated types in traits yet

#### Future Improvements
- Trait-based generic bounds with method dispatch
- Default type parameters (`Vec<T = i64>`)
- Const generics (`Array<T, N: const>`)

## Example Compilation

### Source
```vais
S Pair<T> {
    first: T,
    second: T
}

F swap<T>(p: Pair<T>) -> Pair<T> =
    Pair<T> { first: p.second, second: p.first }

F main() -> i64 {
    p1 := Pair<i64> { first: 1, second: 2 }
    p2 := swap(p1)

    pf := Pair<f64> { first: 1.0, second: 2.0 }
    pf2 := swap(pf)

    p2.first + p2.second
}
```

### Generated LLVM IR
```llvm
; Specialized structs
%Pair$i64 = type { i64, i64 }
%Pair$f64 = type { double, double }

; Specialized functions
define %Pair$i64 @swap$i64(%Pair$i64 %p) {
entry:
  %first = extractvalue %Pair$i64 %p, 0
  %second = extractvalue %Pair$i64 %p, 1
  %result = insertvalue %Pair$i64 undef, i64 %second, 0
  %result2 = insertvalue %Pair$i64 %result, i64 %first, 1
  ret %Pair$i64 %result2
}

define %Pair$f64 @swap$f64(%Pair$f64 %p) {
entry:
  %first = extractvalue %Pair$f64 %p, 0
  %second = extractvalue %Pair$f64 %p, 1
  %result = insertvalue %Pair$f64 undef, double %second, 0
  %result2 = insertvalue %Pair$f64 %result, double %first, 1
  ret %Pair$f64 %result2
}

define i64 @main() {
entry:
  ; Create Pair<i64>
  %p1 = alloca %Pair$i64
  ; ... initialize and call swap$i64 ...

  ; Create Pair<f64>
  %pf = alloca %Pair$f64
  ; ... initialize and call swap$f64 ...

  ret i64 3
}
```

## Testing Strategy

1. **Unit tests**: Test name mangling, type substitution
2. **Integration tests**: Full compilation of generic code
3. **Standard library tests**: Vec<T>, HashMap<K, V> with various types
4. **Performance tests**: Ensure no runtime overhead vs hand-written code
