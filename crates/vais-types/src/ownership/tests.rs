//! Ownership checker tests

#[cfg(test)]
mod tests {
    use crate::ownership::*;
    use crate::types::{ResolvedType, TypeError};
    use vais_ast::Span;

    fn make_span() -> Span {
        Span { start: 0, end: 0 }
    }

    #[test]
    fn test_copy_types_are_not_moved() {
        let mut checker = OwnershipChecker::new();
        checker.define_var("x", ResolvedType::I64, false, Some(make_span()));

        // Using a Copy type multiple times should be fine
        assert!(checker.use_var("x", Some(make_span())).is_ok());
        assert!(checker.use_var("x", Some(make_span())).is_ok());
        assert!(checker.use_var("x", Some(make_span())).is_ok());
    }

    #[test]
    fn test_non_copy_type_moved_on_use() {
        let mut checker = OwnershipChecker::new();
        // Use a Named type which is non-Copy (Str is Copy as a fat pointer)
        let non_copy = ResolvedType::Named {
            name: "MyStruct".to_string(),
            generics: vec![],
        };
        checker.define_var("s", non_copy, false, Some(make_span()));

        // First use moves the value
        assert!(checker.use_var("s", Some(make_span())).is_ok());
        // Second use should fail - value was moved
        assert!(checker.use_var("s", Some(make_span())).is_err());
    }

    #[test]
    fn test_reassign_after_move_is_ok() {
        let mut checker = OwnershipChecker::new();
        checker.define_var("s", ResolvedType::Str, true, Some(make_span()));

        // Move the value
        assert!(checker.use_var("s", Some(make_span())).is_ok());
        // Reassign restores ownership
        assert!(checker
            .assign_var("s", ResolvedType::Str, Some(make_span()))
            .is_ok());
        // Now we can use it again
        assert!(checker.use_var("s", Some(make_span())).is_ok());
    }

    #[test]
    fn test_immutable_borrow_allows_multiple() {
        let mut checker = OwnershipChecker::new();
        checker.define_var(
            "x",
            ResolvedType::Named {
                name: "Vec".to_string(),
                generics: vec![],
            },
            false,
            Some(make_span()),
        );

        // Multiple immutable borrows are fine
        assert!(checker.borrow_var("r1", "x", Some(make_span())).is_ok());
        assert!(checker.borrow_var("r2", "x", Some(make_span())).is_ok());
    }

    #[test]
    fn test_mutable_borrow_exclusive() {
        let mut checker = OwnershipChecker::new();
        checker.define_var(
            "x",
            ResolvedType::Named {
                name: "Vec".to_string(),
                generics: vec![],
            },
            true,
            Some(make_span()),
        );

        // First mutable borrow is fine
        assert!(checker.borrow_var_mut("r1", "x", Some(make_span())).is_ok());
        // Second borrow (even immutable) conflicts
        assert!(checker.borrow_var("r2", "x", Some(make_span())).is_err());
    }

    #[test]
    fn test_mutable_borrow_after_release() {
        let mut checker = OwnershipChecker::new();
        checker.define_var(
            "x",
            ResolvedType::Named {
                name: "Vec".to_string(),
                generics: vec![],
            },
            true,
            Some(make_span()),
        );

        // Borrow and release
        assert!(checker.borrow_var_mut("r1", "x", Some(make_span())).is_ok());
        checker.release_borrow("r1");

        // Now we can borrow again
        assert!(checker.borrow_var_mut("r2", "x", Some(make_span())).is_ok());
    }

    #[test]
    fn test_cannot_mut_borrow_immutable_var() {
        let mut checker = OwnershipChecker::new();
        checker.define_var(
            "x",
            ResolvedType::Named {
                name: "Vec".to_string(),
                generics: vec![],
            },
            false,
            Some(make_span()),
        );

        // Cannot mutably borrow an immutable variable
        assert!(checker
            .borrow_var_mut("r1", "x", Some(make_span()))
            .is_err());
    }

    #[test]
    fn test_cannot_borrow_moved_value() {
        let mut checker = OwnershipChecker::new();
        let non_copy = ResolvedType::Named {
            name: "MyStruct".to_string(),
            generics: vec![],
        };
        checker.define_var("s", non_copy, false, Some(make_span()));

        // Move the value
        assert!(checker.use_var("s", Some(make_span())).is_ok());
        // Cannot borrow a moved value
        assert!(checker.borrow_var("r1", "s", Some(make_span())).is_err());
    }

    #[test]
    fn test_assign_while_borrowed_fails() {
        let mut checker = OwnershipChecker::new();
        checker.define_var(
            "x",
            ResolvedType::Named {
                name: "Vec".to_string(),
                generics: vec![],
            },
            true,
            Some(make_span()),
        );

        // Borrow x
        assert!(checker.borrow_var("r1", "x", Some(make_span())).is_ok());
        // Cannot assign while borrowed
        assert!(checker
            .assign_var(
                "x",
                ResolvedType::Named {
                    name: "Vec".to_string(),
                    generics: vec![]
                },
                Some(make_span())
            )
            .is_err());
    }

    #[test]
    fn test_scope_releases_borrows() {
        let mut checker = OwnershipChecker::new();
        checker.define_var(
            "x",
            ResolvedType::Named {
                name: "Vec".to_string(),
                generics: vec![],
            },
            true,
            Some(make_span()),
        );

        // Borrow in inner scope
        checker.push_scope();
        assert!(checker.borrow_var_mut("r1", "x", Some(make_span())).is_ok());
        checker.pop_scope(); // Borrow released

        // Now we can borrow again
        assert!(checker.borrow_var_mut("r2", "x", Some(make_span())).is_ok());
    }

    #[test]
    fn test_is_copy_type() {
        assert!(OwnershipChecker::is_copy_type(&ResolvedType::I64));
        assert!(OwnershipChecker::is_copy_type(&ResolvedType::Bool));
        assert!(OwnershipChecker::is_copy_type(&ResolvedType::F64));
        assert!(OwnershipChecker::is_copy_type(&ResolvedType::Unit));
        assert!(OwnershipChecker::is_copy_type(&ResolvedType::Ref(
            Box::new(ResolvedType::I64)
        )));

        // Str is Copy — it's a fat pointer { ptr, len }, a borrowed view not owning data
        assert!(OwnershipChecker::is_copy_type(&ResolvedType::Str));
        assert!(!OwnershipChecker::is_copy_type(&ResolvedType::Array(
            Box::new(ResolvedType::I64)
        )));
        assert!(!OwnershipChecker::is_copy_type(&ResolvedType::Named {
            name: "Vec".to_string(),
            generics: vec![]
        }));
        assert!(!OwnershipChecker::is_copy_type(&ResolvedType::RefMut(
            Box::new(ResolvedType::I64)
        )));
    }

    #[test]
    fn test_collecting_mode() {
        let mut checker = OwnershipChecker::new_collecting();
        let non_copy1 = ResolvedType::Named {
            name: "MyStruct".to_string(),
            generics: vec![],
        };
        let non_copy2 = ResolvedType::Named {
            name: "MyStruct".to_string(),
            generics: vec![],
        };
        checker.define_var("s1", non_copy1, false, Some(make_span()));
        checker.define_var("s2", non_copy2, false, Some(make_span()));

        // Move s1
        assert!(checker.use_var("s1", Some(make_span())).is_ok());
        // Use after move - error collected but doesn't fail
        assert!(checker.use_var("s1", Some(make_span())).is_ok());

        // Move s2
        assert!(checker.use_var("s2", Some(make_span())).is_ok());
        // Use after move - another error collected
        assert!(checker.use_var("s2", Some(make_span())).is_ok());

        // Should have collected 2 errors
        assert_eq!(checker.errors().len(), 2);
    }

    // --- Dangling reference tests ---

    #[test]
    fn test_reference_to_outer_scope_is_ok() {
        // V x = 42
        // {
        //   V r = &x  -- x is in outer scope, reference is valid
        // }
        let mut checker = OwnershipChecker::new();
        checker.define_var("x", ResolvedType::I64, false, Some(make_span()));

        checker.push_scope();
        checker.define_var(
            "r",
            ResolvedType::Ref(Box::new(ResolvedType::I64)),
            false,
            Some(make_span()),
        );
        checker.register_reference("r", "x", false);
        checker.pop_scope(); // r goes out of scope, no error since x outlives r

        // No errors expected
    }

    #[test]
    fn test_dangling_reference_detected() {
        // r is in outer scope, x is in inner scope -> dangling after inner scope ends
        let mut checker = OwnershipChecker::new_collecting();

        // Define r in the outer scope
        checker.define_var(
            "r",
            ResolvedType::Ref(Box::new(ResolvedType::I64)),
            false,
            Some(make_span()),
        );

        checker.push_scope();
        // Define x in inner scope
        checker.define_var("x", ResolvedType::I64, false, Some(make_span()));
        // r references x (which lives in inner scope)
        checker.register_reference("r", "x", false);
        checker.pop_scope(); // x is dropped, but r still references it

        assert!(!checker.errors().is_empty());
        let err = &checker.errors()[0];
        assert!(matches!(err, TypeError::DanglingReference { .. }));
    }

    #[test]
    fn test_return_local_ref_detected() {
        let mut checker = OwnershipChecker::new();
        checker.push_scope(); // function scope

        // Define a local variable in function scope
        checker.define_var("local_val", ResolvedType::I64, false, Some(make_span()));

        // Trying to return a reference to a local should fail
        let result = checker.check_return_local_ref("local_val", Some(make_span()));
        assert!(result.is_err());
        if let Err(TypeError::ReturnLocalRef { var_name, .. }) = result {
            assert_eq!(var_name, "local_val");
        } else {
            panic!("Expected ReturnLocalRef error");
        }

        checker.pop_scope();
    }

    #[test]
    fn test_return_param_ref_is_ok() {
        let mut checker = OwnershipChecker::new();

        // Parameters are defined at scope 0 (before push_scope for function body)
        checker.define_var(
            "param",
            ResolvedType::Ref(Box::new(ResolvedType::I64)),
            false,
            Some(make_span()),
        );

        checker.push_scope(); // function body scope

        // Returning a reference to a parameter should be fine
        let result = checker.check_return_local_ref("param", Some(make_span()));
        assert!(result.is_ok());

        checker.pop_scope();
    }

    #[test]
    fn test_reference_tracking() {
        let mut checker = OwnershipChecker::new();
        checker.define_var("x", ResolvedType::I64, false, Some(make_span()));

        // Register a reference: r -> x
        checker.register_reference("r", "x", false);

        // Verify reference is tracked
        assert!(checker.reference_sources.contains_key("r"));
        assert_eq!(checker.reference_sources["r"].source_var, "x");
        assert!(!checker.reference_sources["r"].is_mut);
    }

    #[test]
    fn test_nested_scope_dangling() {
        // Test deeply nested scope dangling detection
        let mut checker = OwnershipChecker::new_collecting();

        // outer_ref in scope 0
        checker.define_var(
            "outer_ref",
            ResolvedType::Ref(Box::new(ResolvedType::I64)),
            false,
            Some(make_span()),
        );

        checker.push_scope(); // scope 1
        checker.push_scope(); // scope 2

        checker.define_var("deep_local", ResolvedType::I64, false, Some(make_span()));
        checker.register_reference("outer_ref", "deep_local", false);

        checker.pop_scope(); // scope 2 ends - deep_local is dropped

        assert!(!checker.errors().is_empty());
        assert!(matches!(
            checker.errors()[0],
            TypeError::DanglingReference { .. }
        ));

        checker.pop_scope(); // scope 1 ends
    }

    #[test]
    fn test_error_messages_have_help() {
        // Verify all new error types provide help messages
        let err1 = TypeError::DanglingReference {
            ref_var: "r".to_string(),
            source_var: "x".to_string(),
            ref_scope_depth: 0,
            source_scope_depth: 1,
            ref_at: Some(make_span()),
            source_defined_at: Some(make_span()),
        };
        assert!(err1.help().is_some());
        assert!(err1.help().unwrap().contains("outlives"));

        let err2 = TypeError::ReturnLocalRef {
            var_name: "local".to_string(),
            return_at: Some(make_span()),
            defined_at: Some(make_span()),
        };
        assert!(err2.help().is_some());
        assert!(err2.help().unwrap().contains("owned value"));

        // Verify error codes
        assert_eq!(err1.error_code(), "E028");
        assert_eq!(err2.error_code(), "E029");
    }

    // --- Lifetime integration tests ---

    #[test]
    fn test_lifetime_inferencer_available() {
        // OwnershipChecker now has an integrated LifetimeInferencer
        let mut checker = OwnershipChecker::new();
        // Register a named lifetime and verify it works
        checker.lifetime_inferencer.register_named_lifetime("a");
        let lt = checker.lifetime_inferencer.resolve_lifetime_name("a");
        assert_eq!(
            lt,
            crate::lifetime::Lifetime::Named("a".to_string())
        );
    }

    #[test]
    fn test_lifetime_var_tracking() {
        // Test that variable lifetimes can be tracked through the ownership checker
        let mut checker = OwnershipChecker::new();
        checker.define_var("x", ResolvedType::I64, false, Some(make_span()));

        let lt = crate::lifetime::Lifetime::Named("a".to_string());
        checker.lifetime_inferencer.register_var_lifetime("x", lt.clone());

        assert_eq!(
            checker.lifetime_inferencer.get_var_lifetime("x"),
            Some(&lt)
        );
    }

    #[test]
    fn test_multiple_immutable_borrows_same_scope() {
        // Multiple immutable borrows in the same scope should be fine
        let mut checker = OwnershipChecker::new();
        checker.define_var("x", ResolvedType::I64, false, Some(make_span()));

        assert!(checker.borrow_var("r1", "x", Some(make_span())).is_ok());
        assert!(checker.borrow_var("r2", "x", Some(make_span())).is_ok());
        assert!(checker.borrow_var("r3", "x", Some(make_span())).is_ok());
    }

    #[test]
    fn test_mut_borrow_after_immutable_borrow_fails() {
        // Cannot take a mutable borrow while an immutable borrow is active
        let mut checker = OwnershipChecker::new();
        checker.define_var(
            "x",
            ResolvedType::Named {
                name: "Vec".to_string(),
                generics: vec![],
            },
            true,
            Some(make_span()),
        );

        assert!(checker.borrow_var("r1", "x", Some(make_span())).is_ok());
        // Mutable borrow while immutable borrow is active should fail
        let result = checker.borrow_var_mut("r2", "x", Some(make_span()));
        assert!(result.is_err());
        if let Err(TypeError::BorrowConflict {
            existing_is_mut,
            new_is_mut,
            ..
        }) = result
        {
            assert!(!existing_is_mut); // existing is immutable
            assert!(new_is_mut); // new is mutable
        } else {
            panic!("Expected BorrowConflict error");
        }
    }

    #[test]
    fn test_double_move_detection() {
        // Moving a value twice should detect on the second move
        let mut checker = OwnershipChecker::new();
        let non_copy = ResolvedType::Named {
            name: "Buffer".to_string(),
            generics: vec![],
        };
        checker.define_var("buf", non_copy, false, Some(make_span()));

        assert!(checker.use_var("buf", Some(make_span())).is_ok());
        let err = checker.use_var("buf", Some(make_span())).unwrap_err();
        assert!(matches!(err, TypeError::UseAfterMove { .. }));
    }

    #[test]
    fn test_copy_tuple_is_copy() {
        // Tuple of Copy types is Copy
        let mut checker = OwnershipChecker::new();
        let tuple_ty = ResolvedType::Tuple(vec![ResolvedType::I64, ResolvedType::Bool]);
        checker.define_var("t", tuple_ty, false, Some(make_span()));

        // Using a Copy tuple multiple times should be fine
        assert!(checker.use_var("t", Some(make_span())).is_ok());
        assert!(checker.use_var("t", Some(make_span())).is_ok());
    }

    #[test]
    fn test_non_copy_tuple_moves() {
        // Tuple containing a non-Copy type is not Copy
        let mut checker = OwnershipChecker::new();
        let tuple_ty = ResolvedType::Tuple(vec![
            ResolvedType::I64,
            ResolvedType::Named {
                name: "String".to_string(),
                generics: vec![],
            },
        ]);
        checker.define_var("t", tuple_ty, false, Some(make_span()));

        assert!(checker.use_var("t", Some(make_span())).is_ok());
        assert!(checker.use_var("t", Some(make_span())).is_err());
    }

    #[test]
    fn test_optional_copy_is_copy() {
        let mut checker = OwnershipChecker::new();
        let opt_ty = ResolvedType::Optional(Box::new(ResolvedType::I64));
        checker.define_var("x", opt_ty, false, Some(make_span()));

        assert!(checker.use_var("x", Some(make_span())).is_ok());
        assert!(checker.use_var("x", Some(make_span())).is_ok());
    }

    #[test]
    fn test_optional_non_copy_moves() {
        let mut checker = OwnershipChecker::new();
        let opt_ty = ResolvedType::Optional(Box::new(ResolvedType::Named {
            name: "Vec".to_string(),
            generics: vec![],
        }));
        checker.define_var("x", opt_ty, false, Some(make_span()));

        assert!(checker.use_var("x", Some(make_span())).is_ok());
        assert!(checker.use_var("x", Some(make_span())).is_err());
    }

    #[test]
    fn test_scope_isolates_variables() {
        // Variables in inner scope should not be visible in outer scope
        let mut checker = OwnershipChecker::new();

        checker.push_scope();
        checker.define_var("inner", ResolvedType::I64, false, Some(make_span()));
        checker.pop_scope();

        // inner is no longer tracked
        assert!(checker.lookup_var("inner").is_none());
    }

    #[test]
    fn test_reference_source_cleanup_on_scope_exit() {
        // Reference tracking should be cleaned up when variables go out of scope
        let mut checker = OwnershipChecker::new();
        checker.define_var("x", ResolvedType::I64, false, Some(make_span()));

        checker.push_scope();
        checker.define_var(
            "r",
            ResolvedType::Ref(Box::new(ResolvedType::I64)),
            false,
            Some(make_span()),
        );
        checker.register_reference("r", "x", false);
        assert!(checker.reference_sources.contains_key("r"));
        checker.pop_scope();

        // After inner scope exit, the reference tracking for 'r' should be cleaned up
        assert!(!checker.reference_sources.contains_key("r"));
    }

    #[test]
    fn test_borrow_after_move_is_error() {
        let mut checker = OwnershipChecker::new();
        let non_copy = ResolvedType::Named {
            name: "Data".to_string(),
            generics: vec![],
        };
        checker.define_var("d", non_copy, false, Some(make_span()));

        // Move the value
        assert!(checker.use_var("d", Some(make_span())).is_ok());
        // Try to mutably borrow the moved value
        let result = checker.borrow_var_mut("r1", "d", Some(make_span()));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TypeError::BorrowAfterMove { .. }));
    }

    #[test]
    fn test_multiple_scopes_borrow_release() {
        // Borrows from different scopes should be properly managed
        let mut checker = OwnershipChecker::new();
        checker.define_var(
            "x",
            ResolvedType::Named {
                name: "Vec".to_string(),
                generics: vec![],
            },
            true,
            Some(make_span()),
        );

        // Scope 1: take mutable borrow
        checker.push_scope();
        assert!(checker.borrow_var_mut("r1", "x", Some(make_span())).is_ok());
        checker.pop_scope(); // borrow released

        // Scope 2: take another mutable borrow
        checker.push_scope();
        assert!(checker.borrow_var_mut("r2", "x", Some(make_span())).is_ok());
        checker.pop_scope(); // borrow released

        // Scope 3: take an immutable borrow
        assert!(checker.borrow_var("r3", "x", Some(make_span())).is_ok());
    }

    #[test]
    fn test_ref_type_is_copy() {
        // Immutable references are Copy
        assert!(OwnershipChecker::is_copy_type(&ResolvedType::Ref(
            Box::new(ResolvedType::Named {
                name: "Vec".to_string(),
                generics: vec![]
            })
        )));
    }

    #[test]
    fn test_ref_mut_type_is_not_copy() {
        // Mutable references are NOT Copy (uniqueness requirement)
        assert!(!OwnershipChecker::is_copy_type(&ResolvedType::RefMut(
            Box::new(ResolvedType::I64)
        )));
    }

    #[test]
    fn test_fn_type_is_copy() {
        assert!(OwnershipChecker::is_copy_type(&ResolvedType::Fn {
            params: vec![ResolvedType::I64],
            ret: Box::new(ResolvedType::Bool),
            effects: None,
        }));
    }

    #[test]
    fn test_linear_type_not_copy() {
        assert!(!OwnershipChecker::is_copy_type(&ResolvedType::Linear(
            Box::new(ResolvedType::I64)
        )));
    }

    #[test]
    fn test_collecting_mode_gathers_all_errors() {
        let mut checker = OwnershipChecker::new_collecting();
        let non_copy = ResolvedType::Named {
            name: "A".to_string(),
            generics: vec![],
        };

        // Create 3 variables and move all of them
        for name in &["a", "b", "c"] {
            checker.define_var(name, non_copy.clone(), false, Some(make_span()));
            let _ = checker.use_var(name, Some(make_span())); // move
            let _ = checker.use_var(name, Some(make_span())); // use-after-move (error collected)
        }

        assert_eq!(checker.errors().len(), 3);
        for err in checker.errors() {
            assert!(matches!(err, TypeError::UseAfterMove { .. }));
        }
    }

    #[test]
    fn test_take_errors_clears_list() {
        let mut checker = OwnershipChecker::new_collecting();
        let non_copy = ResolvedType::Named {
            name: "X".to_string(),
            generics: vec![],
        };
        checker.define_var("x", non_copy, false, Some(make_span()));
        let _ = checker.use_var("x", Some(make_span()));
        let _ = checker.use_var("x", Some(make_span()));

        assert_eq!(checker.errors().len(), 1);
        let taken = checker.take_errors();
        assert_eq!(taken.len(), 1);
        assert!(checker.errors().is_empty());
    }

    #[test]
    fn test_assign_resets_moved_state() {
        let mut checker = OwnershipChecker::new();
        let non_copy = ResolvedType::Named {
            name: "Data".to_string(),
            generics: vec![],
        };
        checker.define_var("d", non_copy.clone(), true, Some(make_span()));

        // Move
        assert!(checker.use_var("d", Some(make_span())).is_ok());
        // Assign new value
        assert!(checker.assign_var("d", non_copy, Some(make_span())).is_ok());
        // Use again - should be fine
        assert!(checker.use_var("d", Some(make_span())).is_ok());
    }

    #[test]
    fn test_dangling_ref_in_conditional_branch() {
        // Reference created in one branch pointing to inner-scope data
        let mut checker = OwnershipChecker::new_collecting();

        checker.define_var(
            "result_ref",
            ResolvedType::Ref(Box::new(ResolvedType::I64)),
            false,
            Some(make_span()),
        );

        // Simulate: if true { x := 5; result_ref = &x }
        checker.push_scope();
        checker.define_var("x", ResolvedType::I64, false, Some(make_span()));
        checker.register_reference("result_ref", "x", false);
        checker.pop_scope(); // x is dropped, result_ref dangles

        assert!(!checker.errors().is_empty());
        assert!(matches!(
            checker.errors()[0],
            TypeError::DanglingReference { .. }
        ));
    }

    #[test]
    fn test_const_array_copy_if_element_copy() {
        use crate::types::resolved::ResolvedConst;
        assert!(OwnershipChecker::is_copy_type(&ResolvedType::ConstArray {
            element: Box::new(ResolvedType::I64),
            size: ResolvedConst::Value(10),
        }));
    }

    #[test]
    fn test_const_array_not_copy_if_element_not_copy() {
        use crate::types::resolved::ResolvedConst;
        assert!(!OwnershipChecker::is_copy_type(
            &ResolvedType::ConstArray {
                element: Box::new(ResolvedType::Named {
                    name: "Vec".to_string(),
                    generics: vec![]
                }),
                size: ResolvedConst::Value(10),
            }
        ));
    }

    #[test]
    fn test_result_copy_if_both_copy() {
        assert!(OwnershipChecker::is_copy_type(&ResolvedType::Result(
            Box::new(ResolvedType::I64),
            Box::new(ResolvedType::Bool),
        )));
    }

    #[test]
    fn test_result_not_copy_if_err_not_copy() {
        assert!(!OwnershipChecker::is_copy_type(&ResolvedType::Result(
            Box::new(ResolvedType::I64),
            Box::new(ResolvedType::Named {
                name: "Error".to_string(),
                generics: vec![]
            }),
        )));
    }
}
