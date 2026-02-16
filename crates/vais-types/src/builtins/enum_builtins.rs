//! Built-in enum types (Result, Option)

use std::collections::HashMap;

use super::*;

impl TypeChecker {
    pub(super) fn register_enum_builtins(&mut self) {
        // Register built-in Result<T, E> enum
        {
            let mut variants = HashMap::new();
            variants.insert(
                "Ok".to_string(),
                VariantFieldTypes::Tuple(vec![ResolvedType::Generic("T".to_string())]),
            );
            variants.insert(
                "Err".to_string(),
                VariantFieldTypes::Tuple(vec![ResolvedType::Generic("E".to_string())]),
            );
            self.enums.insert(
                "Result".to_string(),
                EnumDef {
                    name: "Result".to_string(),
                    generics: vec!["T".to_string(), "E".to_string()],
                    variants,
                    methods: HashMap::new(),
                },
            );
            self.exhaustiveness_checker
                .register_enum("Result", vec!["Ok".to_string(), "Err".to_string()]);
        }

        // Register built-in Option<T> enum
        if !self.enums.contains_key("Option") {
            let mut variants = HashMap::new();
            variants.insert("None".to_string(), VariantFieldTypes::Unit);
            variants.insert(
                "Some".to_string(),
                VariantFieldTypes::Tuple(vec![ResolvedType::Generic("T".to_string())]),
            );
            self.enums.insert(
                "Option".to_string(),
                EnumDef {
                    name: "Option".to_string(),
                    generics: vec!["T".to_string()],
                    variants,
                    methods: HashMap::new(),
                },
            );
            self.exhaustiveness_checker
                .register_enum("Option", vec!["None".to_string(), "Some".to_string()]);
        }
    }
}
