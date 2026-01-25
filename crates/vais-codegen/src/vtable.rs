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
}

impl VtableGenerator {
    /// Create a new VTable generator
    pub fn new() -> Self {
        Self {
            vtables: HashMap::new(),
            vtable_counter: 0,
        }
    }

    /// Generate a vtable for a type implementing a trait
    pub fn generate_vtable(
        &mut self,
        impl_type: &str,
        trait_def: &TraitDef,
        method_impls: &HashMap<String, String>, // method_name -> mangled_function_name
    ) -> VtableInfo {
        let key = (impl_type.to_string(), trait_def.name.clone());

        // Return cached vtable if exists
        if let Some(info) = self.vtables.get(&key) {
            return info.clone();
        }

        let global_name = format!("@vtable_{}_{}", impl_type, trait_def.name);

        // Collect methods in declaration order
        let mut methods = Vec::new();
        for (method_name, _method_sig) in &trait_def.methods {
            if let Some(impl_name) = method_impls.get(method_name) {
                methods.push((method_name.clone(), impl_name.clone()));
            } else {
                // Method not implemented - use null (will panic at runtime)
                methods.push((method_name.clone(), "null".to_string()));
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

        info
    }

    /// Get LLVM IR type for a vtable struct
    pub fn vtable_struct_type(trait_def: &TraitDef) -> String {
        // VTable layout:
        // - drop: void(i8*)*       ; destructor
        // - size: i64              ; size of concrete type
        // - align: i64             ; alignment of concrete type
        // - methods: fn_ptr*...    ; method function pointers

        let mut fields = vec![
            "i8*".to_string(),  // drop function or null
            "i64".to_string(),  // size
            "i64".to_string(),  // align
        ];

        // Add method function pointer types
        for (_method_name, method_sig) in &trait_def.methods {
            // Method signature: (self: i8*, params...) -> ret
            let mut param_types = vec!["i8*".to_string()]; // self pointer
            for (_param_name, _param_ty, _) in &method_sig.params[1..] { // Skip self
                param_types.push("i64".to_string()); // Simplified: all args as i64
            }

            let ret_type = if matches!(method_sig.ret, ResolvedType::Unit) {
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

        let mut values = vec![
            "null".to_string(),                   // drop (not implemented yet)
            format!("{}", type_size),             // size
            format!("{}", type_align),            // align
        ];

        // Add method function pointers
        for (method_name, impl_name) in &info.methods {
            if impl_name == "null" {
                values.push("null".to_string());
            } else {
                // Get method signature to generate correct function pointer type
                if let Some(method_sig) = trait_def.methods.get(method_name) {
                    let mut param_types = vec!["i8*".to_string()];
                    for _ in &method_sig.params[1..] {
                        param_types.push("i64".to_string());
                    }
                    let ret_type = if matches!(method_sig.ret, ResolvedType::Unit) {
                        "void"
                    } else {
                        "i64"
                    };
                    // Cast function to expected type
                    values.push(format!(
                        "bitcast ({}({})* @{} to {}({})*)",
                        ret_type,
                        param_types.join(", "),
                        impl_name,
                        ret_type,
                        param_types.join(", ")
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

        ir.push_str(&format!(
            "  {} = call i8* @malloc(i64 8)\n", // Simplified: assume 8 bytes
            alloc_name
        ));

        // Store the concrete value
        let cast_ptr = format!("%trait_cast_{}", *temp_counter);
        *temp_counter += 1;

        ir.push_str(&format!(
            "  {} = bitcast i8* {} to {}*\n",
            cast_ptr, alloc_name, concrete_type
        ));
        ir.push_str(&format!(
            "  store {} {}, {}* {}\n",
            concrete_type, concrete_value, concrete_type, cast_ptr
        ));

        // Build the trait object struct
        let trait_obj_tmp1 = format!("%trait_obj_{}", *temp_counter);
        *temp_counter += 1;
        let trait_obj_tmp2 = format!("%trait_obj_{}", *temp_counter);
        *temp_counter += 1;

        // Get vtable pointer as i8*
        let vtable_cast = format!("%vtable_cast_{}", *temp_counter);
        *temp_counter += 1;

        ir.push_str(&format!(
            "  {} = bitcast {}* {} to i8*\n",
            vtable_cast, "%vtable_type", vtable_info.global_name // Placeholder type
        ));

        // Create the trait object { data_ptr, vtable_ptr }
        ir.push_str(&format!(
            "  {} = insertvalue {} undef, i8* {}, 0\n",
            trait_obj_tmp1, TRAIT_OBJECT_TYPE, alloc_name
        ));
        ir.push_str(&format!(
            "  {} = insertvalue {} {}, i8* {}, 1\n",
            trait_obj_tmp2, TRAIT_OBJECT_TYPE, trait_obj_tmp1, vtable_cast
        ));

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

        ir.push_str(&format!(
            "  {} = extractvalue {} {}, 0\n",
            data_ptr, TRAIT_OBJECT_TYPE, trait_object
        ));

        // Extract vtable pointer from trait object
        let vtable_ptr = format!("%dyn_vtable_{}", *temp_counter);
        *temp_counter += 1;

        ir.push_str(&format!(
            "  {} = extractvalue {} {}, 1\n",
            vtable_ptr, TRAIT_OBJECT_TYPE, trait_object
        ));

        // Cast vtable pointer to the correct vtable type
        let vtable_type = Self::vtable_struct_type(trait_def);
        let vtable_cast = format!("%vtable_typed_{}", *temp_counter);
        *temp_counter += 1;

        ir.push_str(&format!(
            "  {} = bitcast i8* {} to {}*\n",
            vtable_cast, vtable_ptr, vtable_type
        ));

        // Get the method function pointer from vtable
        // Method index is offset by 3 (drop, size, align)
        let vtable_slot = method_index + 3;
        let fn_ptr_ptr = format!("%fn_ptr_ptr_{}", *temp_counter);
        *temp_counter += 1;

        ir.push_str(&format!(
            "  {} = getelementptr {}, {}* {}, i32 0, i32 {}\n",
            fn_ptr_ptr, vtable_type, vtable_type, vtable_cast, vtable_slot
        ));

        // Load the function pointer
        let fn_ptr = format!("%fn_ptr_{}", *temp_counter);
        *temp_counter += 1;

        // Determine function type from method signature
        let fn_type = format!("{}(i8*, {})*", ret_type,
            args.iter().map(|_| "i64").collect::<Vec<_>>().join(", "));

        ir.push_str(&format!(
            "  {} = load {}*, {}** {}\n",
            fn_ptr, fn_type.trim_end_matches('*'), fn_ptr_ptr, fn_ptr_ptr
        ));

        // Build argument list: data_ptr followed by method arguments
        let mut call_args = vec![format!("i8* {}", data_ptr)];
        for arg in args {
            call_args.push(format!("i64 {}", arg));
        }

        // Generate the indirect call
        let result = if ret_type == "void" {
            ir.push_str(&format!(
                "  call {} {} ({})\n",
                ret_type, fn_ptr, call_args.join(", ")
            ));
            "".to_string()
        } else {
            let result_name = format!("%dyn_result_{}", *temp_counter);
            *temp_counter += 1;

            ir.push_str(&format!(
                "  {} = call {} {} ({})\n",
                result_name, ret_type, fn_ptr, call_args.join(", ")
            ));
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
        methods.insert("speak".to_string(), TraitMethodSig {
            name: "speak".to_string(),
            params: vec![
                ("self".to_string(), ResolvedType::Ref(Box::new(ResolvedType::Generic("Self".to_string()))), false),
            ],
            ret: ResolvedType::I64,
            has_default: false,
            is_async: false,
        });
        methods.insert("move_to".to_string(), TraitMethodSig {
            name: "move_to".to_string(),
            params: vec![
                ("self".to_string(), ResolvedType::Ref(Box::new(ResolvedType::Generic("Self".to_string()))), false),
                ("x".to_string(), ResolvedType::I64, false),
                ("y".to_string(), ResolvedType::I64, false),
            ],
            ret: ResolvedType::Unit,
            has_default: false,
            is_async: false,
        });

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

        let vtable = gen.generate_vtable("Dog", &trait_def, &impls);

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
        let trait_def = create_test_trait();

        let impls = HashMap::new();

        let vtable1 = gen.generate_vtable("Cat", &trait_def, &impls);
        let vtable2 = gen.generate_vtable("Cat", &trait_def, &impls);

        // Should return same vtable (cached)
        assert_eq!(vtable1.global_name, vtable2.global_name);
    }

    #[test]
    fn test_trait_object_type() {
        assert_eq!(TRAIT_OBJECT_TYPE, "{ i8*, i8* }");
    }
}
