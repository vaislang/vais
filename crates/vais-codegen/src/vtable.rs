//! VTable generation and dynamic dispatch for trait objects
//!
//! This module implements runtime polymorphism via vtable-based dispatch
//! for `dyn Trait` types in Vais.
//!
//! # Memory Layout
//!
//! A trait object (`dyn Trait`) is a fat pointer consisting of:
//! - Data pointer: `i8*` pointing to the actual object data
//! - VTable pointer: `i8*` pointing to the vtable for this trait
//!
//! # VTable Structure
//!
//! Each vtable contains:
//! - Drop function pointer (for cleanup)
//! - Size of the concrete type
//! - Alignment of the concrete type
//! - Method function pointers in declaration order

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Write;
use vais_types::{ResolvedType, TraitDef};

/// LLVM IR type for trait object (fat pointer)
pub const TRAIT_OBJECT_TYPE: &str = "{ i8*, i8* }";

/// Information about a vtable for a specific type implementing a trait
#[derive(Debug, Clone)]
pub struct VtableInfo {
    /// Name of the trait
    pub trait_name: String,
    /// Name of the implementing type
    pub impl_type: String,
    /// Global name for this vtable (e.g., @vtable_MyType_MyTrait)
    pub global_name: String,
    /// Method entries in order: (method_name, mangled_function_name)
    pub methods: Vec<(String, String)>,
}

/// VTable generator for trait objects
#[derive(Debug, Default)]
pub struct VtableGenerator {
    /// Generated vtables: key = (impl_type, trait_name)
    vtables: HashMap<(String, String), VtableInfo>,
    /// Counter for unique vtable names
    vtable_counter: usize,
    /// Generated drop functions for types
    drop_functions: HashSet<String>,
}

impl VtableGenerator {
    /// Create a new VTable generator
    pub fn new() -> Self {
        Self {
            vtables: HashMap::new(),
            vtable_counter: 0,
            drop_functions: HashSet::new(),
        }
    }

    /// Generate a drop function for a type that calls free() on the data pointer
    /// Returns the mangled drop function name
    pub fn generate_drop_function(&mut self, type_name: &str) -> String {
        let drop_fn_name = format!("__drop_{}", type_name);
        self.drop_functions.insert(drop_fn_name.clone());
        drop_fn_name
    }

    /// Generate LLVM IR for all drop functions
    /// Drop functions call free() on the data pointer to deallocate heap memory
    pub fn generate_drop_functions_ir(&self) -> String {
        let mut ir = String::new();

        for drop_fn in &self.drop_functions {
            write!(
                ir,
                r#"
define void @{}(i8* %ptr) {{
entry:
  %is_null = icmp eq i8* %ptr, null
  br i1 %is_null, label %done, label %do_free

do_free:
  %ptr_as_i64 = ptrtoint i8* %ptr to i64
  call void @free(i64 %ptr_as_i64)
  br label %done

done:
  ret void
}}
"#,
                drop_fn
            )
            .unwrap();
        }

        ir
    }

    /// Check if a drop function has been generated for a type
    pub fn has_drop_function(&self, type_name: &str) -> bool {
        let drop_fn_name = format!("__drop_{}", type_name);
        self.drop_functions.contains(&drop_fn_name)
    }

    /// Get the drop function name for a type
    pub fn get_drop_function_name(type_name: &str) -> String {
        format!("__drop_{}", type_name)
    }

    /// Generate a vtable for a type implementing a trait
    /// Returns an error if any required method (without default implementation) is missing
    pub fn generate_vtable(
        &mut self,
        impl_type: &str,
        trait_def: &TraitDef,
        method_impls: &HashMap<String, String>, // method_name -> mangled_function_name
    ) -> Result<VtableInfo, String> {
        let key = (impl_type.to_string(), trait_def.name.clone());

        // Return cached vtable if exists
        if let Some(info) = self.vtables.get(&key) {
            return Ok(info.clone());
        }

        // Generate drop function for this type
        self.generate_drop_function(impl_type);

        let global_name = format!("@vtable_{}_{}", impl_type, trait_def.name);

        // Collect methods in declaration order
        let mut methods = Vec::new();
        for (method_name, method_sig) in &trait_def.methods {
            if let Some(impl_name) = method_impls.get(method_name) {
                methods.push((method_name.clone(), impl_name.clone()));
            } else if method_sig.has_default {
                // Method has default implementation, use the default
                // Generate default method name: Trait_methodname_default
                let default_name = format!("{}_{}_default", trait_def.name, method_name);
                methods.push((method_name.clone(), default_name));
            } else {
                // Required method not implemented - compile-time error
                return Err(format!(
                    "Trait `{}` method `{}` not implemented for type `{}`",
                    trait_def.name, method_name, impl_type
                ));
            }
        }

        let info = VtableInfo {
            trait_name: trait_def.name.clone(),
            impl_type: impl_type.to_string(),
            global_name,
            methods,
        };

        self.vtables.insert(key, info.clone());
        self.vtable_counter += 1;

        Ok(info)
    }

    /// Get LLVM IR type for a vtable struct
    pub fn vtable_struct_type(trait_def: &TraitDef) -> String {
        // VTable layout:
        // - drop: void(i8*)*       ; destructor
        // - size: i64              ; size of concrete type
        // - align: i64             ; alignment of concrete type
        // - methods: fn_ptr*...    ; method function pointers

        let mut fields = vec![
            "i8*".to_string(), // drop function or null
            "i64".to_string(), // size
            "i64".to_string(), // align
        ];

        // Add method function pointer types
        for method_sig in trait_def.methods.values() {
            // Method signature: (self: i8*, params...) -> ret
            let mut param_types = vec!["i8*".to_string()]; // self pointer
            for (_param_name, _param_ty, _) in &method_sig.params[1..] {
                // Skip self
                param_types.push("i64".to_string()); // Simplified: all args as i64
            }

            // For async methods, return type is always i64 (Future handle)
            let ret_type = if method_sig.is_async {
                "i64" // Async methods return a Future handle (i64 pointer to state)
            } else if matches!(method_sig.ret, ResolvedType::Unit) {
                "void"
            } else {
                "i64" // Simplified: all returns as i64
            };

            let fn_type = format!("{}({})*", ret_type, param_types.join(", "));
            fields.push(fn_type);
        }

        format!("{{ {} }}", fields.join(", "))
    }

    /// Generate LLVM IR for a vtable global constant
    pub fn generate_vtable_global(
        &self,
        info: &VtableInfo,
        trait_def: &TraitDef,
        type_size: usize,
        type_align: usize,
    ) -> String {
        let vtable_type = Self::vtable_struct_type(trait_def);

        // Generate drop function pointer - cast to i8* for vtable storage
        let drop_fn_name = Self::get_drop_function_name(&info.impl_type);
        let drop_fn_ptr = format!("i8* bitcast (void(i8*)* @{} to i8*)", drop_fn_name);

        let mut values = vec![
            drop_fn_ptr,                   // drop function pointer
            format!("i64 {}", type_size),  // size
            format!("i64 {}", type_align), // align
        ];

        // Add method function pointers
        for (method_name, impl_name) in &info.methods {
            if impl_name == "null" {
                values.push("null".to_string());
            } else {
                // Get method signature to generate correct function pointer type
                if let Some(method_sig) = trait_def.methods.get(method_name) {
                    // Build vtable function type (uses i8* for self)
                    let mut vtable_param_types = vec!["i8*".to_string()];
                    for _ in &method_sig.params[1..] {
                        vtable_param_types.push("i64".to_string());
                    }
                    let ret_type = if method_sig.is_async {
                        "i64"
                    } else if matches!(method_sig.ret, ResolvedType::Unit) {
                        "void"
                    } else {
                        "i64"
                    };

                    // Build concrete function type (uses %Type* for self)
                    let mut concrete_param_types = vec![format!("%{}*", info.impl_type)];
                    for _ in &method_sig.params[1..] {
                        concrete_param_types.push("i64".to_string());
                    }

                    let vtable_fn_type =
                        format!("{}({})*", ret_type, vtable_param_types.join(", "));
                    let concrete_fn_type =
                        format!("{}({})*", ret_type, concrete_param_types.join(", "));

                    // Cast from concrete function type to vtable function type
                    values.push(format!(
                        "{} bitcast ({} @{} to {})",
                        vtable_fn_type, concrete_fn_type, impl_name, vtable_fn_type
                    ));
                } else {
                    values.push("null".to_string());
                }
            }
        }

        format!(
            "{} = internal constant {} {{ {} }}",
            info.global_name,
            vtable_type,
            values.join(", ")
        )
    }

    /// Generate LLVM IR to create a trait object from a concrete value
    /// Returns (ir_code, result_value) where result_value is the trait object
    pub fn create_trait_object(
        &self,
        concrete_value: &str,
        concrete_type: &str,
        vtable_info: &VtableInfo,
        temp_counter: &mut usize,
    ) -> (String, String) {
        let mut ir = String::new();

        // Allocate space for the concrete value on the heap
        let alloc_name = format!("%trait_data_{}", *temp_counter);
        *temp_counter += 1;

        writeln!(ir, "  {} = call i8* @malloc(i64 8)", alloc_name).unwrap(); // Simplified: assume 8 bytes

        // Store the concrete value
        let cast_ptr = format!("%trait_cast_{}", *temp_counter);
        *temp_counter += 1;

        writeln!(
            ir,
            "  {} = bitcast i8* {} to {}*",
            cast_ptr, alloc_name, concrete_type
        )
        .unwrap();
        writeln!(
            ir,
            "  store {} {}, {}* {}",
            concrete_type, concrete_value, concrete_type, cast_ptr
        )
        .unwrap();

        // Build the trait object struct
        let trait_obj_tmp1 = format!("%trait_obj_{}", *temp_counter);
        *temp_counter += 1;
        let trait_obj_tmp2 = format!("%trait_obj_{}", *temp_counter);
        *temp_counter += 1;

        // Get vtable pointer as i8*
        let vtable_cast = format!("%vtable_cast_{}", *temp_counter);
        *temp_counter += 1;

        writeln!(
            ir,
            "  {} = bitcast %vtable_type* {} to i8*",
            vtable_cast, vtable_info.global_name
        )
        .unwrap(); // Placeholder type

        // Create the trait object { data_ptr, vtable_ptr }
        writeln!(
            ir,
            "  {} = insertvalue {} undef, i8* {}, 0",
            trait_obj_tmp1, TRAIT_OBJECT_TYPE, alloc_name
        )
        .unwrap();
        writeln!(
            ir,
            "  {} = insertvalue {} {}, i8* {}, 1",
            trait_obj_tmp2, TRAIT_OBJECT_TYPE, trait_obj_tmp1, vtable_cast
        )
        .unwrap();

        (ir, trait_obj_tmp2)
    }

    /// Generate LLVM IR for a dynamic method call through vtable
    /// Returns (ir_code, result_value) for the call result
    pub fn generate_dynamic_call(
        &self,
        trait_object: &str,
        method_index: usize,
        args: &[String],
        ret_type: &str,
        trait_def: &TraitDef,
        temp_counter: &mut usize,
    ) -> (String, String) {
        let mut ir = String::new();

        // Extract data pointer from trait object
        let data_ptr = format!("%dyn_data_{}", *temp_counter);
        *temp_counter += 1;

        writeln!(
            ir,
            "  {} = extractvalue {} {}, 0",
            data_ptr, TRAIT_OBJECT_TYPE, trait_object
        )
        .unwrap();

        // Extract vtable pointer from trait object
        let vtable_ptr = format!("%dyn_vtable_{}", *temp_counter);
        *temp_counter += 1;

        writeln!(
            ir,
            "  {} = extractvalue {} {}, 1",
            vtable_ptr, TRAIT_OBJECT_TYPE, trait_object
        )
        .unwrap();

        // Cast vtable pointer to the correct vtable type
        let vtable_type = Self::vtable_struct_type(trait_def);
        let vtable_cast = format!("%vtable_typed_{}", *temp_counter);
        *temp_counter += 1;

        writeln!(
            ir,
            "  {} = bitcast i8* {} to {}*",
            vtable_cast, vtable_ptr, vtable_type
        )
        .unwrap();

        // Get the method function pointer from vtable
        // Method index is offset by 3 (drop, size, align)
        let vtable_slot = method_index + 3;
        let fn_ptr_ptr = format!("%fn_ptr_ptr_{}", *temp_counter);
        *temp_counter += 1;

        writeln!(
            ir,
            "  {} = getelementptr {}, {}* {}, i32 0, i32 {}",
            fn_ptr_ptr, vtable_type, vtable_type, vtable_cast, vtable_slot
        )
        .unwrap();

        // Load the function pointer
        let fn_ptr = format!("%fn_ptr_{}", *temp_counter);
        *temp_counter += 1;

        // Determine function type from method signature
        let extra_arg_types = args.iter().map(|_| "i64").collect::<Vec<_>>().join(", ");
        let fn_type = if extra_arg_types.is_empty() {
            format!("{}(i8*)*", ret_type)
        } else {
            format!("{}(i8*, {})*", ret_type, extra_arg_types)
        };

        writeln!(
            ir,
            "  {} = load {}, {}* {}",
            fn_ptr, fn_type, fn_type, fn_ptr_ptr
        )
        .unwrap();

        // Build argument list: data_ptr followed by method arguments
        let mut call_args = vec![format!("i8* {}", data_ptr)];
        for arg in args {
            call_args.push(format!("i64 {}", arg));
        }

        // Generate the indirect call
        let result = if ret_type == "void" {
            writeln!(
                ir,
                "  call {} {}({})",
                ret_type,
                fn_ptr,
                call_args.join(", ")
            )
            .unwrap();
            "".to_string()
        } else {
            let result_name = format!("%dyn_result_{}", *temp_counter);
            *temp_counter += 1;

            writeln!(
                ir,
                "  {} = call {} {}({})",
                result_name,
                ret_type,
                fn_ptr,
                call_args.join(", ")
            )
            .unwrap();
            result_name
        };

        (ir, result)
    }

    /// Get all generated vtables
    pub fn get_vtables(&self) -> impl Iterator<Item = &VtableInfo> {
        self.vtables.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vais_types::TraitMethodSig;

    fn create_test_trait() -> TraitDef {
        let mut methods = HashMap::new();
        methods.insert(
            "speak".to_string(),
            TraitMethodSig {
                name: "speak".to_string(),
                params: vec![(
                    "self".to_string(),
                    ResolvedType::Ref(Box::new(ResolvedType::Generic("Self".to_string()))),
                    false,
                )],
                ret: ResolvedType::I64,
                has_default: false,
                is_async: false,
                is_const: false,
            },
        );
        methods.insert(
            "move_to".to_string(),
            TraitMethodSig {
                name: "move_to".to_string(),
                params: vec![
                    (
                        "self".to_string(),
                        ResolvedType::Ref(Box::new(ResolvedType::Generic("Self".to_string()))),
                        false,
                    ),
                    ("x".to_string(), ResolvedType::I64, false),
                    ("y".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::Unit,
                has_default: false,
                is_async: false,
                is_const: false,
            },
        );

        TraitDef {
            name: "Animal".to_string(),
            generics: vec![],
            super_traits: vec![],
            associated_types: HashMap::new(),
            methods,
        }
    }

    #[test]
    fn test_vtable_generation() {
        let mut gen = VtableGenerator::new();
        let trait_def = create_test_trait();

        let mut impls = HashMap::new();
        impls.insert("speak".to_string(), "Dog_speak".to_string());
        impls.insert("move_to".to_string(), "Dog_move_to".to_string());

        let vtable = gen
            .generate_vtable("Dog", &trait_def, &impls)
            .expect("vtable generation failed");

        assert_eq!(vtable.trait_name, "Animal");
        assert_eq!(vtable.impl_type, "Dog");
        assert!(vtable.global_name.contains("Dog"));
        assert!(vtable.global_name.contains("Animal"));
        assert_eq!(vtable.methods.len(), 2);
    }

    #[test]
    fn test_vtable_struct_type() {
        let trait_def = create_test_trait();
        let vtable_type = VtableGenerator::vtable_struct_type(&trait_def);

        // Should contain: drop, size, align, speak_fn, move_to_fn
        assert!(vtable_type.starts_with("{ "));
        assert!(vtable_type.contains("i8*")); // drop pointer
        assert!(vtable_type.contains("i64")); // size and align
    }

    #[test]
    fn test_vtable_caching() {
        let mut gen = VtableGenerator::new();
        let mut trait_def = create_test_trait();

        // Add default implementations to avoid errors
        for method_sig in trait_def.methods.values_mut() {
            method_sig.has_default = true;
        }

        let impls = HashMap::new();

        let vtable1 = gen
            .generate_vtable("Cat", &trait_def, &impls)
            .expect("vtable generation failed");
        let vtable2 = gen
            .generate_vtable("Cat", &trait_def, &impls)
            .expect("vtable generation failed");

        // Should return same vtable (cached)
        assert_eq!(vtable1.global_name, vtable2.global_name);
    }

    #[test]
    fn test_trait_object_type() {
        assert_eq!(TRAIT_OBJECT_TYPE, "{ i8*, i8* }");
    }

    #[test]
    fn test_drop_function_generation() {
        let mut gen = VtableGenerator::new();

        // Generate drop function for Dog type
        let drop_fn = gen.generate_drop_function("Dog");
        assert_eq!(drop_fn, "__drop_Dog");
        assert!(gen.has_drop_function("Dog"));
        assert!(!gen.has_drop_function("Cat"));

        // Generate IR for drop functions
        let ir = gen.generate_drop_functions_ir();
        assert!(ir.contains("define void @__drop_Dog(i8* %ptr)"));
        assert!(ir.contains("call void @free(i64 %ptr_as_i64)"));
        assert!(ir.contains("icmp eq i8* %ptr, null"));
    }

    #[test]
    fn test_vtable_with_drop() {
        let mut gen = VtableGenerator::new();
        let trait_def = create_test_trait();

        let mut impls = HashMap::new();
        impls.insert("speak".to_string(), "Dog_speak".to_string());
        impls.insert("move_to".to_string(), "Dog_move_to".to_string());

        let vtable = gen
            .generate_vtable("Dog", &trait_def, &impls)
            .expect("vtable generation failed");

        // Verify drop function was generated
        assert!(gen.has_drop_function("Dog"));

        // Generate vtable global
        let vtable_ir = gen.generate_vtable_global(&vtable, &trait_def, 16, 8);

        // Verify vtable contains drop function pointer
        assert!(vtable_ir.contains("@__drop_Dog"));
        assert!(vtable_ir.contains("bitcast (void(i8*)* @__drop_Dog to i8*)"));
    }

    #[test]
    fn test_drop_function_name() {
        assert_eq!(
            VtableGenerator::get_drop_function_name("MyType"),
            "__drop_MyType"
        );
        assert_eq!(
            VtableGenerator::get_drop_function_name("Vec_i64"),
            "__drop_Vec_i64"
        );
    }

    #[test]
    fn test_missing_required_method_error() {
        let mut gen = VtableGenerator::new();
        let trait_def = create_test_trait();

        // Only implement one of the two required methods
        let mut impls = HashMap::new();
        impls.insert("speak".to_string(), "Cat_speak".to_string());
        // "move_to" is missing

        let result = gen.generate_vtable("Cat", &trait_def, &impls);

        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Trait `Animal` method `move_to` not implemented for type `Cat`"));
    }

    #[test]
    fn test_default_method_implementation() {
        let mut gen = VtableGenerator::new();
        let mut trait_def = create_test_trait();

        // Mark "move_to" as having a default implementation
        trait_def.methods.get_mut("move_to").unwrap().has_default = true;

        // Only implement "speak", let "move_to" use default
        let mut impls = HashMap::new();
        impls.insert("speak".to_string(), "Cat_speak".to_string());

        let result = gen.generate_vtable("Cat", &trait_def, &impls);

        assert!(result.is_ok());
        let vtable = result.unwrap();
        assert_eq!(vtable.methods.len(), 2);

        // Verify that the default implementation is used for move_to
        let move_to_impl = vtable
            .methods
            .iter()
            .find(|(name, _)| name == "move_to")
            .unwrap();
        assert_eq!(move_to_impl.1, "Animal_move_to_default");
    }
}
