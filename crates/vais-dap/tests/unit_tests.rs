//! Unit tests for DAP modules
//!
//! Tests cover: breakpoint management, variable management, stack management,
//! source map, step controller, error types, protocol types, and session helpers.

// ============================================================================
// 1. Breakpoint Module Tests
// ============================================================================

mod breakpoint_tests {
    use vais_dap::breakpoint::*;
    use vais_dap::types::*;

    #[test]
    fn test_hit_counter_new() {
        let counter = HitCounter::new();
        assert_eq!(counter.get(1), 0);
        assert_eq!(counter.get(999), 0);
    }

    #[test]
    fn test_hit_counter_increment() {
        let mut counter = HitCounter::new();
        assert_eq!(counter.increment(1), 1);
        assert_eq!(counter.increment(1), 2);
        assert_eq!(counter.increment(1), 3);
        assert_eq!(counter.get(1), 3);
    }

    #[test]
    fn test_hit_counter_multiple_ids() {
        let mut counter = HitCounter::new();
        counter.increment(1);
        counter.increment(2);
        counter.increment(1);
        assert_eq!(counter.get(1), 2);
        assert_eq!(counter.get(2), 1);
    }

    #[test]
    fn test_hit_counter_reset() {
        let mut counter = HitCounter::new();
        counter.increment(1);
        counter.increment(1);
        counter.reset(1);
        assert_eq!(counter.get(1), 0);
    }

    #[test]
    fn test_hit_counter_reset_all() {
        let mut counter = HitCounter::new();
        counter.increment(1);
        counter.increment(2);
        counter.increment(3);
        counter.reset_all();
        assert_eq!(counter.get(1), 0);
        assert_eq!(counter.get(2), 0);
        assert_eq!(counter.get(3), 0);
    }

    #[test]
    fn test_parse_hit_condition_equal_plain() {
        assert_eq!(parse_hit_condition("5"), Some(HitConditionOp::Equal(5)));
    }

    #[test]
    fn test_parse_hit_condition_equal_prefix() {
        assert_eq!(parse_hit_condition("= 5"), Some(HitConditionOp::Equal(5)));
    }

    #[test]
    fn test_parse_hit_condition_equal_whitespace() {
        assert_eq!(parse_hit_condition("  =  10  "), Some(HitConditionOp::Equal(10)));
    }

    #[test]
    fn test_parse_hit_condition_greater() {
        assert_eq!(parse_hit_condition("> 3"), Some(HitConditionOp::Greater(3)));
        assert_eq!(parse_hit_condition(">5"), Some(HitConditionOp::Greater(5)));
    }

    #[test]
    fn test_parse_hit_condition_greater_equal() {
        assert_eq!(parse_hit_condition(">= 3"), Some(HitConditionOp::GreaterEqual(3)));
        assert_eq!(parse_hit_condition(">=10"), Some(HitConditionOp::GreaterEqual(10)));
    }

    #[test]
    fn test_parse_hit_condition_multiple() {
        assert_eq!(parse_hit_condition("% 10"), Some(HitConditionOp::Multiple(10)));
        assert_eq!(parse_hit_condition("%5"), Some(HitConditionOp::Multiple(5)));
    }

    #[test]
    fn test_parse_hit_condition_invalid() {
        assert_eq!(parse_hit_condition("invalid"), None);
        assert_eq!(parse_hit_condition("abc"), None);
    }

    #[test]
    fn test_parse_hit_condition_empty() {
        assert_eq!(parse_hit_condition(""), None);
    }

    #[test]
    fn test_evaluate_hit_condition_equal() {
        assert!(evaluate_hit_condition(&HitConditionOp::Equal(5), 5));
        assert!(!evaluate_hit_condition(&HitConditionOp::Equal(5), 4));
        assert!(!evaluate_hit_condition(&HitConditionOp::Equal(5), 6));
    }

    #[test]
    fn test_evaluate_hit_condition_greater() {
        assert!(evaluate_hit_condition(&HitConditionOp::Greater(3), 4));
        assert!(!evaluate_hit_condition(&HitConditionOp::Greater(3), 3));
        assert!(!evaluate_hit_condition(&HitConditionOp::Greater(3), 2));
    }

    #[test]
    fn test_evaluate_hit_condition_greater_equal() {
        assert!(evaluate_hit_condition(&HitConditionOp::GreaterEqual(3), 4));
        assert!(evaluate_hit_condition(&HitConditionOp::GreaterEqual(3), 3));
        assert!(!evaluate_hit_condition(&HitConditionOp::GreaterEqual(3), 2));
    }

    #[test]
    fn test_evaluate_hit_condition_multiple() {
        assert!(evaluate_hit_condition(&HitConditionOp::Multiple(3), 3));
        assert!(evaluate_hit_condition(&HitConditionOp::Multiple(3), 6));
        assert!(evaluate_hit_condition(&HitConditionOp::Multiple(3), 9));
        assert!(!evaluate_hit_condition(&HitConditionOp::Multiple(3), 1));
        assert!(!evaluate_hit_condition(&HitConditionOp::Multiple(3), 4));
    }

    #[test]
    fn test_evaluate_hit_condition_multiple_zero() {
        assert!(!evaluate_hit_condition(&HitConditionOp::Multiple(0), 0));
        assert!(!evaluate_hit_condition(&HitConditionOp::Multiple(0), 10));
    }

    #[test]
    fn test_breakpoint_manager_new() {
        let manager = BreakpointManager::new();
        assert!(manager.get_source_breakpoints("/any/path").is_none());
        assert_eq!(manager.get_exception_filters().len(), 0);
    }

    #[test]
    fn test_set_source_breakpoints() {
        let mut manager = BreakpointManager::new();
        let source = Source { path: Some("/test.vais".to_string()), ..Default::default() };
        let bps = vec![
            SourceBreakpoint { line: 10, column: None, condition: None, hit_condition: None, log_message: None },
            SourceBreakpoint { line: 20, column: Some(5), condition: Some("x > 0".to_string()), hit_condition: None, log_message: None },
        ];
        let managed = manager.set_source_breakpoints(&source, &bps);
        assert_eq!(managed.len(), 2);
        assert_eq!(managed[0].line, 10);
        assert_eq!(managed[1].line, 20);
        assert_eq!(managed[1].condition, Some("x > 0".to_string()));
        assert!(!managed[0].verified);
    }

    #[test]
    fn test_set_function_breakpoints() {
        let mut manager = BreakpointManager::new();
        let bps = vec![
            FunctionBreakpoint { name: "main".to_string(), condition: None, hit_condition: None },
            FunctionBreakpoint { name: "foo".to_string(), condition: Some("true".to_string()), hit_condition: None },
        ];
        let managed = manager.set_function_breakpoints(&bps);
        assert_eq!(managed.len(), 2);
        assert_eq!(managed[0].function_name, Some("main".to_string()));
        assert_eq!(managed[1].condition, Some("true".to_string()));
    }

    #[test]
    fn test_set_exception_filters() {
        let mut manager = BreakpointManager::new();
        manager.set_exception_filters(vec!["all".to_string(), "uncaught".to_string()]);
        assert_eq!(manager.get_exception_filters().len(), 2);
        assert_eq!(manager.get_exception_filters()[0], "all");
    }

    #[test]
    fn test_verify_breakpoint_source() {
        let mut manager = BreakpointManager::new();
        let source = Source { path: Some("/test.vais".to_string()), ..Default::default() };
        let bps = vec![SourceBreakpoint { line: 10, column: None, condition: None, hit_condition: None, log_message: None }];
        let managed = manager.set_source_breakpoints(&source, &bps);
        let id = managed[0].id;
        manager.verify_breakpoint(id, 0xABCD, Some(11));
        let bps = manager.get_source_breakpoints("/test.vais").unwrap();
        assert!(bps[0].verified);
        assert_eq!(bps[0].address, Some(0xABCD));
        assert_eq!(bps[0].line, 11);
    }

    #[test]
    fn test_verify_breakpoint_function() {
        let mut manager = BreakpointManager::new();
        let bps = vec![FunctionBreakpoint { name: "main".to_string(), condition: None, hit_condition: None }];
        let managed = manager.set_function_breakpoints(&bps);
        let id = managed[0].id;
        manager.verify_breakpoint(id, 0x1234, None);
        let bp = manager.get_function_breakpoints().next().unwrap();
        assert!(bp.verified);
        assert_eq!(bp.address, Some(0x1234));
    }

    #[test]
    fn test_find_by_address() {
        let mut manager = BreakpointManager::new();
        let source = Source { path: Some("/test.vais".to_string()), ..Default::default() };
        let bps = vec![SourceBreakpoint { line: 10, column: None, condition: None, hit_condition: None, log_message: None }];
        let managed = manager.set_source_breakpoints(&source, &bps);
        let id = managed[0].id;
        manager.verify_breakpoint(id, 0x5678, None);
        let found = manager.find_by_address(0x5678);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, id);
        assert!(manager.find_by_address(0x9999).is_none());
    }

    #[test]
    fn test_to_dap_breakpoint_verified() {
        let manager = BreakpointManager::new();
        let bp = ManagedBreakpoint {
            id: 1, verified: true, line: 10, column: None,
            condition: None, hit_condition: None, log_message: None,
            address: Some(0x1000), function_name: None,
        };
        let dap_bp = manager.to_dap_breakpoint(&bp, None);
        assert_eq!(dap_bp.id, Some(1));
        assert!(dap_bp.verified);
        assert!(dap_bp.message.is_none());
        assert_eq!(dap_bp.line, Some(10));
    }

    #[test]
    fn test_to_dap_breakpoint_unverified() {
        let manager = BreakpointManager::new();
        let bp = ManagedBreakpoint {
            id: 2, verified: false, line: 20, column: Some(5),
            condition: None, hit_condition: None, log_message: None,
            address: None, function_name: None,
        };
        let dap_bp = manager.to_dap_breakpoint(&bp, None);
        assert!(!dap_bp.verified);
        assert_eq!(dap_bp.message, Some("Pending".to_string()));
    }

    #[test]
    fn test_should_break_no_condition() {
        let manager = BreakpointManager::new();
        let bp = ManagedBreakpoint {
            id: 1, verified: true, line: 10, column: None,
            condition: None, hit_condition: None, log_message: None,
            address: None, function_name: None,
        };
        assert!(manager.should_break(&bp, 1));
    }

    #[test]
    fn test_should_break_with_hit_condition() {
        let manager = BreakpointManager::new();
        let bp = ManagedBreakpoint {
            id: 1, verified: true, line: 10, column: None,
            condition: None, hit_condition: Some("5".to_string()), log_message: None,
            address: None, function_name: None,
        };
        assert!(!manager.should_break(&bp, 3));
        assert!(manager.should_break(&bp, 5));
        assert!(manager.should_break(&bp, 10));
    }

    #[test]
    fn test_get_log_message() {
        let manager = BreakpointManager::new();
        let bp_no_log = ManagedBreakpoint {
            id: 1, verified: true, line: 10, column: None,
            condition: None, hit_condition: None, log_message: None,
            address: None, function_name: None,
        };
        assert!(manager.get_log_message(&bp_no_log).is_none());

        let bp_with_log = ManagedBreakpoint {
            id: 2, verified: true, line: 10, column: None,
            condition: None, hit_condition: None, log_message: Some("log msg".to_string()),
            address: None, function_name: None,
        };
        assert_eq!(manager.get_log_message(&bp_with_log), Some("log msg"));
    }

    #[test]
    fn test_clear_all() {
        let mut manager = BreakpointManager::new();
        let source = Source { path: Some("/test.vais".to_string()), ..Default::default() };
        let bps = vec![SourceBreakpoint { line: 10, column: None, condition: None, hit_condition: None, log_message: None }];
        manager.set_source_breakpoints(&source, &bps);
        manager.set_function_breakpoints(&[FunctionBreakpoint { name: "main".to_string(), condition: None, hit_condition: None }]);
        manager.set_exception_filters(vec!["all".to_string()]);
        manager.clear_all();
        assert!(manager.get_source_breakpoints("/test.vais").is_none());
        assert_eq!(manager.get_exception_filters().len(), 0);
    }

    #[test]
    fn test_record_hit_break() {
        let mut manager = BreakpointManager::new();
        let bp = ManagedBreakpoint {
            id: 1, verified: true, line: 10, column: None,
            condition: None, hit_condition: None, log_message: None,
            address: None, function_name: None,
        };
        assert_eq!(manager.record_hit(&bp), HitResult::Break);
        assert_eq!(manager.get_hit_count(1), 1);
    }

    #[test]
    fn test_record_hit_skip() {
        let mut manager = BreakpointManager::new();
        let bp = ManagedBreakpoint {
            id: 1, verified: true, line: 10, column: None,
            condition: None, hit_condition: Some(">= 3".to_string()), log_message: None,
            address: None, function_name: None,
        };
        assert_eq!(manager.record_hit(&bp), HitResult::Skip);
        assert_eq!(manager.record_hit(&bp), HitResult::Skip);
        assert_eq!(manager.record_hit(&bp), HitResult::Break);
    }

    #[test]
    fn test_record_hit_logpoint() {
        let mut manager = BreakpointManager::new();
        let bp = ManagedBreakpoint {
            id: 1, verified: true, line: 10, column: None,
            condition: None, hit_condition: None, log_message: Some("hello".to_string()),
            address: None, function_name: None,
        };
        assert_eq!(manager.record_hit(&bp), HitResult::Log("hello".to_string()));
    }
}

// ============================================================================
// 2. Variables Module Tests
// ============================================================================

mod variable_tests {
    use vais_dap::variables::*;

    #[test]
    fn test_scope_kind_name() {
        assert_eq!(ScopeKind::Locals.name(), "Locals");
        assert_eq!(ScopeKind::Arguments.name(), "Arguments");
        assert_eq!(ScopeKind::Registers.name(), "Registers");
        assert_eq!(ScopeKind::Globals.name(), "Globals");
    }

    #[test]
    fn test_scope_kind_is_expensive() {
        assert!(!ScopeKind::Locals.is_expensive());
        assert!(!ScopeKind::Arguments.is_expensive());
        assert!(ScopeKind::Registers.is_expensive());
        assert!(ScopeKind::Globals.is_expensive());
    }

    #[test]
    fn test_variable_manager_new() {
        let manager = VariableManager::new();
        assert!(manager.get_ref_info(1).is_none());
        assert!(manager.get_cached_variables(1).is_none());
    }

    #[test]
    fn test_create_scopes() {
        let mut manager = VariableManager::new();
        let scopes = manager.create_scopes(1);
        assert_eq!(scopes.len(), 3);
        assert_eq!(scopes[0].name, "Locals");
        assert_eq!(scopes[1].name, "Arguments");
        assert_eq!(scopes[2].name, "Registers");
        // Verify ref info was stored
        assert!(manager.get_ref_info(scopes[0].variables_reference).is_some());
    }

    #[test]
    fn test_create_scopes_with_globals() {
        let mut manager = VariableManager::new();
        let scopes = manager.create_scopes_with_globals(1);
        assert_eq!(scopes.len(), 3);
        assert_eq!(scopes[0].name, "Locals");
        assert_eq!(scopes[1].name, "Arguments");
        assert_eq!(scopes[2].name, "Globals");
    }

    #[test]
    fn test_invalidate() {
        let mut manager = VariableManager::new();
        let scopes = manager.create_scopes(1);
        let ref_id = scopes[0].variables_reference;
        assert!(manager.get_ref_info(ref_id).is_some());
        manager.invalidate();
        assert!(manager.get_ref_info(ref_id).is_none());
    }

    #[test]
    fn test_evaluate_context_variants() {
        let _watch = EvaluateContext::Watch;
        let _hover = EvaluateContext::Hover;
        let _repl = EvaluateContext::Repl;
        let _clipboard = EvaluateContext::Clipboard;
        // Just ensure enum variants exist
    }

    #[test]
    fn test_get_evaluation_path_scope() {
        let mut manager = VariableManager::new();
        let scopes = manager.create_scopes(1);
        // Scope references don't have evaluation paths
        assert!(manager.get_evaluation_path(scopes[0].variables_reference).is_none());
    }
}

// ============================================================================
// 3. Stack Module Tests
// ============================================================================

mod stack_tests {
    use vais_dap::stack::*;

    #[test]
    fn test_stack_manager_new() {
        let manager = StackManager::new();
        assert!(manager.get_frame_info(1).is_none());
        assert!(manager.get_thread_frames(1).is_none());
    }

    #[test]
    fn test_cache_frames() {
        let mut manager = StackManager::new();
        let raw_frames = vec![
            RawFrame {
                function_name: "main".to_string(),
                source_path: Some("/test.vais".to_string()),
                line: 10, column: 1, instruction_pointer: 0x1000,
                module_name: Some("test".to_string()),
            },
            RawFrame {
                function_name: "foo".to_string(),
                source_path: None,
                line: 20, column: 5, instruction_pointer: 0x2000,
                module_name: None,
            },
        ];
        let dap_frames = manager.cache_frames(1, raw_frames);
        assert_eq!(dap_frames.len(), 2);
        assert_eq!(dap_frames[0].name, "main");
        assert_eq!(dap_frames[0].line, 10);
        assert_eq!(dap_frames[1].name, "foo");
    }

    #[test]
    fn test_get_frame_info() {
        let mut manager = StackManager::new();
        let raw_frames = vec![RawFrame {
            function_name: "main".to_string(),
            source_path: None, line: 1, column: 1, instruction_pointer: 0x1000,
            module_name: None,
        }];
        let dap_frames = manager.cache_frames(1, raw_frames);
        let info = manager.get_frame_info(dap_frames[0].id).unwrap();
        assert_eq!(info.thread_id, 1);
        assert_eq!(info.frame_index, 0);
    }

    #[test]
    fn test_get_cached_frame() {
        let mut manager = StackManager::new();
        let raw_frames = vec![RawFrame {
            function_name: "main".to_string(),
            source_path: Some("/test.vais".to_string()),
            line: 10, column: 1, instruction_pointer: 0x1000,
            module_name: None,
        }];
        let dap_frames = manager.cache_frames(1, raw_frames);
        let cached = manager.get_cached_frame(dap_frames[0].id).unwrap();
        assert_eq!(cached.name, "main");
        assert_eq!(cached.line, 10);
    }

    #[test]
    fn test_get_top_frame() {
        let mut manager = StackManager::new();
        let raw_frames = vec![
            RawFrame { function_name: "main".to_string(), source_path: None, line: 1, column: 1, instruction_pointer: 0x1000, module_name: None },
            RawFrame { function_name: "foo".to_string(), source_path: None, line: 2, column: 1, instruction_pointer: 0x2000, module_name: None },
        ];
        manager.cache_frames(1, raw_frames);
        let top = manager.get_top_frame(1).unwrap();
        assert_eq!(top.name, "main");
    }

    #[test]
    fn test_get_top_frame_none() {
        let manager = StackManager::new();
        assert!(manager.get_top_frame(1).is_none());
    }

    #[test]
    fn test_invalidate_stack() {
        let mut manager = StackManager::new();
        let raw_frames = vec![RawFrame {
            function_name: "main".to_string(),
            source_path: None, line: 1, column: 1, instruction_pointer: 0x1000,
            module_name: None,
        }];
        let dap_frames = manager.cache_frames(1, raw_frames);
        let id = dap_frames[0].id;
        assert!(manager.get_frame_info(id).is_some());
        manager.invalidate();
        assert!(manager.get_frame_info(id).is_none());
        assert!(manager.get_thread_frames(1).is_none());
    }

    #[test]
    fn test_step_controller_new() {
        let controller = StepController::new();
        assert!(!controller.is_stepping());
        assert!(controller.current_mode().is_none());
    }

    #[test]
    fn test_step_controller_default() {
        let controller = StepController::default();
        assert!(!controller.is_stepping());
    }

    #[test]
    fn test_step_granularity_default() {
        assert_eq!(StepGranularity::default(), StepGranularity::Statement);
    }

    #[test]
    fn test_step_over() {
        let mut controller = StepController::new();
        controller.start_step(StepMode::Over, StepGranularity::Statement, 1, 10, 1);
        assert!(controller.is_stepping());
        // Same line = don't stop
        assert!(!controller.should_stop(1, 10, Some("main"), 1));
        // Different line, same depth = stop
        assert!(controller.should_stop(1, 11, Some("main"), 1));
        // Deeper = don't stop
        assert!(!controller.should_stop(2, 20, Some("foo"), 1));
    }

    #[test]
    fn test_step_in_no_target() {
        let mut controller = StepController::new();
        controller.start_step(StepMode::In { target_function: None }, StepGranularity::Statement, 1, 10, 1);
        // Same line = don't stop
        assert!(!controller.should_stop(1, 10, Some("main"), 1));
        // Deeper = stop (entered function)
        assert!(controller.should_stop(2, 20, Some("foo"), 1));
    }

    #[test]
    fn test_step_in_with_target() {
        let mut controller = StepController::new();
        controller.start_step(
            StepMode::In { target_function: Some("target".to_string()) },
            StepGranularity::Statement, 1, 10, 1,
        );
        assert!(!controller.should_stop(2, 20, Some("other"), 1));
        assert!(controller.should_stop(2, 30, Some("target"), 1));
        assert!(!controller.should_stop(2, 30, None, 1));
    }

    #[test]
    fn test_step_out() {
        let mut controller = StepController::new();
        controller.start_step(StepMode::Out, StepGranularity::Statement, 2, 20, 1);
        assert!(!controller.should_stop(2, 21, Some("foo"), 1));
        assert!(!controller.should_stop(3, 30, Some("bar"), 1));
        assert!(controller.should_stop(1, 10, Some("main"), 1));
    }

    #[test]
    fn test_step_instruction_granularity() {
        let mut controller = StepController::new();
        controller.start_step(StepMode::Over, StepGranularity::Instruction, 1, 10, 1);
        // Instruction level stops at same depth regardless of line
        assert!(controller.should_stop(1, 10, Some("main"), 1));
    }

    #[test]
    fn test_step_thread_isolation() {
        let mut controller = StepController::new();
        controller.start_step(StepMode::Over, StepGranularity::Statement, 1, 10, 1);
        assert!(controller.should_stop(1, 11, Some("main"), 1));
        assert!(!controller.should_stop(1, 11, Some("main"), 2)); // different thread
    }

    #[test]
    fn test_step_complete() {
        let mut controller = StepController::new();
        controller.start_step(StepMode::Over, StepGranularity::Statement, 1, 10, 1);
        assert!(controller.is_stepping());
        controller.complete_step();
        assert!(!controller.is_stepping());
    }

    #[test]
    fn test_step_cancel() {
        let mut controller = StepController::new();
        controller.start_step(StepMode::Over, StepGranularity::Statement, 1, 10, 1);
        controller.cancel();
        assert!(!controller.is_stepping());
    }

    #[test]
    fn test_step_no_active_step() {
        let controller = StepController::new();
        assert!(!controller.should_stop(1, 10, Some("main"), 1));
    }
}

// ============================================================================
// 4. Error Module Tests
// ============================================================================

mod error_tests {
    use vais_dap::DapError;

    #[test]
    fn test_error_display_all_variants() {
        assert!(DapError::Protocol("bad".to_string()).to_string().contains("bad"));
        assert!(DapError::InvalidRequest("missing".to_string()).to_string().contains("missing"));
        assert_eq!(DapError::NotInitialized.to_string(), "Session not initialized");
        assert_eq!(DapError::NoActiveSession.to_string(), "No active debug session");
        assert_eq!(DapError::ProcessNotRunning.to_string(), "Process not running");
        assert!(DapError::ThreadNotFound(42).to_string().contains("42"));
        assert!(DapError::FrameNotFound(7).to_string().contains("7"));
        assert!(DapError::VariableNotFound(99).to_string().contains("99"));
        assert!(DapError::DwarfParsing("corrupt".to_string()).to_string().contains("corrupt"));
        assert!(DapError::Unsupported("feature".to_string()).to_string().contains("feature"));
        assert!(DapError::Timeout("response".to_string()).to_string().contains("response"));
        assert!(DapError::Debugger("crash".to_string()).to_string().contains("crash"));
        assert!(DapError::Breakpoint("invalid".to_string()).to_string().contains("invalid"));
        assert!(DapError::SourceMapping("missing".to_string()).to_string().contains("missing"));
    }

    #[test]
    fn test_error_response_format() {
        let err = DapError::Debugger("crash".to_string());
        let resp = err.to_error_response();
        let error_obj = resp.get("error").unwrap();
        assert!(error_obj.get("id").is_some());
        assert!(error_obj.get("format").is_some());
        assert!(error_obj.get("showUser").is_some());
    }

    #[test]
    fn test_error_response_debugger_show_user() {
        let err = DapError::Debugger("err".to_string());
        let resp = err.to_error_response();
        assert!(resp["error"]["showUser"].as_bool().unwrap());
    }

    #[test]
    fn test_error_response_protocol_hide_from_user() {
        let err = DapError::Protocol("err".to_string());
        let resp = err.to_error_response();
        assert!(!resp["error"]["showUser"].as_bool().unwrap());
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let dap_err: DapError = io_err.into();
        assert!(matches!(dap_err, DapError::Io(_)));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_err = serde_json::from_str::<i32>("bad").unwrap_err();
        let dap_err: DapError = json_err.into();
        assert!(matches!(dap_err, DapError::Json(_)));
    }
}

// ============================================================================
// 5. Source Map Tests
// ============================================================================

mod source_map_tests {
    use vais_dap::source_map::SourceMap;

    #[test]
    fn test_source_map_new() {
        let map = SourceMap::new();
        assert!(map.get_location(0x0).is_none());
        assert!(!map.is_loaded());
    }

    #[test]
    fn test_source_map_no_location() {
        let map = SourceMap::new();
        assert!(map.get_location(0x1234).is_none());
        assert!(map.get_source_for_address(0x1234).is_none());
        assert!(map.get_line_column(0x1234).is_none());
    }

    #[test]
    fn test_source_map_no_addresses() {
        let map = SourceMap::new();
        assert!(map.get_addresses("/nonexistent.vais", 1).is_none());
    }

    #[test]
    fn test_source_map_register_source() {
        let mut map = SourceMap::new();
        let ref_id = map.register_source("/test.vais", "F main()->i64=42".to_string());
        assert!(ref_id > 0);
        let content = map.get_source_content(ref_id);
        assert!(content.is_some());
        assert!(content.unwrap().contains("main"));
    }

    #[test]
    fn test_source_map_register_multiple_sources() {
        let mut map = SourceMap::new();
        let ref1 = map.register_source("/a.vais", "F a()->i64=1".to_string());
        let ref2 = map.register_source("/b.vais", "F b()->i64=2".to_string());
        assert_ne!(ref1, ref2);
        assert!(map.get_source_content(ref1).unwrap().contains("a"));
        assert!(map.get_source_content(ref2).unwrap().contains("b"));
    }

    #[test]
    fn test_source_map_content_by_path() {
        let mut map = SourceMap::new();
        map.register_source("/test.vais", "content here".to_string());
        let content = map.get_source_content_by_path("/test.vais");
        assert!(content.is_some());
        assert_eq!(content.unwrap(), "content here");
    }

    #[test]
    fn test_source_map_content_not_found() {
        let map = SourceMap::new();
        assert!(map.get_source_content(999).is_none());
        assert!(map.get_source_content_by_path("/nonexistent").is_none());
    }

    #[test]
    fn test_source_map_stats_empty() {
        let map = SourceMap::new();
        let (addrs, locs) = map.stats();
        assert_eq!(addrs, 0);
        assert_eq!(locs, 0);
    }

    #[test]
    fn test_source_map_source_files_empty() {
        let map = SourceMap::new();
        assert!(map.get_source_files().is_empty());
    }

    #[test]
    fn test_source_map_find_nearest_line_empty() {
        let map = SourceMap::new();
        assert!(map.find_nearest_line("/test.vais", 10).is_none());
    }

    #[test]
    fn test_load_source_file_not_found() {
        let mut map = SourceMap::new();
        let result = map.load_source_file("/nonexistent/file.vais");
        assert!(result.is_err());
    }
}
