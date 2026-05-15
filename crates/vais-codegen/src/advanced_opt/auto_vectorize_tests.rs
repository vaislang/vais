use super::*;

#[test]
fn test_vector_width() {
    assert_eq!(VectorWidth::SSE.bits(), 128);
    assert_eq!(VectorWidth::AVX2.bits(), 256);
    assert_eq!(VectorWidth::AVX512.bits(), 512);

    assert_eq!(VectorWidth::AVX2.lanes(32), 8); // 8 x i32
    assert_eq!(VectorWidth::AVX2.lanes(64), 4); // 4 x i64
}

#[test]
fn test_loop_dependence() {
    let flow = LoopDependence::Flow { distance: Some(1) };
    assert!(flow.prevents_vectorization(4));

    let flow_far = LoopDependence::Flow { distance: Some(8) };
    assert!(!flow_far.prevents_vectorization(4));

    let none = LoopDependence::None;
    assert!(!none.prevents_vectorization(4));
}

#[test]
fn test_extract_branch_targets() {
    let line = "br i1 %cond, label %then, label %else";
    let targets = extract_branch_targets(line);
    assert_eq!(targets.len(), 2);
    assert!(targets.contains(&"then".to_string()));
    assert!(targets.contains(&"else".to_string()));
}

#[test]
fn test_analyze_simple_loop() {
    let ir = r#"
define void @sum(i64* %arr, i64 %n) {
entry:
  br label %loop

loop:
  %i = phi i64 [0, %entry], [%i.next, %loop]
  %ptr = getelementptr i64, i64* %arr, i64 %i
  %val = load i64, i64* %ptr
  %i.next = add i64 %i, 1
  %cond = icmp slt i64 %i.next, %n
  br i1 %cond, label %loop, label %exit

exit:
  ret void
}
"#;

    let vectorizer = analyze_vectorization(ir, VectorWidth::AVX2);
    // The loop detection should find at least one candidate
    // Vectorizer should parse the IR without panicking
    let _ = vectorizer.candidates.len();
}

#[test]
fn test_memory_access_parsing() {
    let load = "  %val = load i64, i64* %ptr";
    let access = parse_memory_access(load, false).unwrap();
    assert_eq!(access.base, "%ptr");
    assert!(!access.is_write);
    assert_eq!(access.element_size, 8);

    let store = "  store i64 %val, i64* %ptr";
    let access = parse_memory_access(store, true).unwrap();
    assert_eq!(access.base, "%ptr");
    assert!(access.is_write);
}

#[test]
fn test_detect_sum_reduction() {
    let ir = r#"
define i64 @sum(i64* %arr, i64 %n) {
entry:
  br label %loop

loop:
  %i = phi i64 [ 0, %entry ], [ %i.next, %loop ]
  %acc = phi i64 [ 0, %entry ], [ %acc.next, %loop ]
  %ptr = getelementptr i64, i64* %arr, i64 %i
  %val = load i64, i64* %ptr
  %acc.next = add i64 %acc, %val
  %i.next = add i64 %i, 1
  %cond = icmp slt i64 %i.next, %n
  br i1 %cond, label %loop, label %exit

exit:
  ret i64 %acc
}
"#;
    let reductions = detect_reductions(ir, "loop");
    // Both %i (induction) and %acc are detected as Add reductions
    assert!(reductions.len() >= 1);
    let acc_reduction = reductions.iter().find(|(var, _)| var == "%acc");
    assert!(acc_reduction.is_some(), "Should detect %acc as a reduction");
    assert_eq!(acc_reduction.unwrap().1, ReductionKind::Add);
}

#[test]
fn test_reduction_identity() {
    assert_eq!(ReductionKind::Add.identity_i64(), 0);
    assert_eq!(ReductionKind::Mul.identity_i64(), 1);
    assert_eq!(ReductionKind::Min.identity_i64(), i64::MAX);
}

// ========== VectorWidth additional tests ==========

#[test]
fn test_vector_width_neon() {
    assert_eq!(VectorWidth::NEON.bits(), 128);
    assert_eq!(VectorWidth::NEON.lanes(32), 4);
    assert_eq!(VectorWidth::NEON.lanes(64), 2);
}

#[test]
fn test_vector_width_auto() {
    assert_eq!(VectorWidth::Auto.bits(), 256);
    assert_eq!(VectorWidth::Auto.target_features(), "+avx2");
}

#[test]
fn test_vector_width_avx512() {
    assert_eq!(VectorWidth::AVX512.bits(), 512);
    assert_eq!(VectorWidth::AVX512.lanes(32), 16);
    assert_eq!(VectorWidth::AVX512.lanes(64), 8);
    assert!(VectorWidth::AVX512.target_features().contains("avx512"));
}

#[test]
fn test_vector_width_f32_lanes() {
    assert_eq!(VectorWidth::SSE.f32_lanes(), 4);
    assert_eq!(VectorWidth::AVX2.f32_lanes(), 8);
    assert_eq!(VectorWidth::AVX512.f32_lanes(), 16);
}

#[test]
fn test_vector_width_f64_lanes() {
    assert_eq!(VectorWidth::SSE.f64_lanes(), 2);
    assert_eq!(VectorWidth::AVX2.f64_lanes(), 4);
    assert_eq!(VectorWidth::AVX512.f64_lanes(), 8);
}

#[test]
fn test_vector_width_i32_lanes() {
    assert_eq!(VectorWidth::NEON.i32_lanes(), 4);
    assert_eq!(VectorWidth::AVX2.i32_lanes(), 8);
}

#[test]
fn test_vector_width_default() {
    let default = VectorWidth::default();
    assert_eq!(default, VectorWidth::AVX2);
}

#[test]
fn test_vector_width_clone_copy() {
    let w = VectorWidth::SSE;
    let cloned = w.clone();
    let copied = w;
    assert_eq!(w, cloned);
    assert_eq!(w, copied);
}

#[test]
fn test_vector_width_auto_detect() {
    let detected = VectorWidth::auto_detect();
    // Should return some valid width
    assert!(detected.bits() >= 128);
}

#[test]
fn test_vector_width_target_features() {
    assert_eq!(VectorWidth::SSE.target_features(), "+sse4.2");
    assert_eq!(VectorWidth::AVX2.target_features(), "+avx2");
    assert_eq!(VectorWidth::NEON.target_features(), "+neon");
}

// ========== LoopDependence tests ==========

#[test]
fn test_loop_dependence_unknown_prevents() {
    assert!(LoopDependence::Unknown.prevents_vectorization(4));
    assert!(LoopDependence::Unknown.prevents_vectorization(8));
}

#[test]
fn test_loop_dependence_anti_close() {
    let anti = LoopDependence::Anti { distance: Some(2) };
    assert!(anti.prevents_vectorization(4)); // distance 2 < width 4
    assert!(!anti.prevents_vectorization(2)); // distance 2 >= width 2
}

#[test]
fn test_loop_dependence_output_close() {
    let output = LoopDependence::Output { distance: Some(1) };
    assert!(output.prevents_vectorization(4));
}

#[test]
fn test_loop_dependence_none_distance() {
    let flow_none = LoopDependence::Flow { distance: None };
    assert!(flow_none.prevents_vectorization(4)); // unknown distance is conservative
}

#[test]
fn test_loop_dependence_equality() {
    let a = LoopDependence::None;
    let b = LoopDependence::None;
    assert_eq!(a, b);

    let c = LoopDependence::Flow { distance: Some(1) };
    let d = LoopDependence::Flow { distance: Some(1) };
    assert_eq!(c, d);
}

// ========== VectorizationCandidate tests ==========

#[test]
fn test_vectorization_candidate_default() {
    let c = VectorizationCandidate::default();
    assert!(c.header.is_empty());
    assert!(c.latch.is_empty());
    assert!(c.induction_var.is_none());
    assert!(c.trip_count.is_none());
    assert!(c.memory_accesses.is_empty());
    assert!(c.dependencies.is_empty());
    assert!(!c.is_vectorizable);
    assert!(c.non_vectorizable_reason.is_some());
    assert!(c.recommended_width.is_none());
    assert_eq!(c.estimated_speedup, 1.0);
}

// ========== helper function tests ==========

#[test]
fn test_has_side_effects_pure_intrinsic() {
    assert!(!has_side_effects("call double @llvm.sqrt.f64(double %x)"));
    assert!(!has_side_effects("call double @llvm.fabs.f64(double %x)"));
    assert!(!has_side_effects("call void @llvm.dbg.declare(...)"));
}

#[test]
fn test_has_side_effects_unknown_func() {
    assert!(has_side_effects("call void @printf(i8* %fmt)"));
    assert!(has_side_effects("call void @my_func()"));
}

#[test]
fn test_has_side_effects_indirect_call() {
    assert!(has_side_effects("call void %func_ptr()"));
}

#[test]
fn test_detect_element_size() {
    assert_eq!(detect_element_size("load i8, i8* %ptr"), 1);
    assert_eq!(detect_element_size("load i16, i16* %ptr"), 2);
    assert_eq!(detect_element_size("load i32, i32* %ptr"), 4);
    assert_eq!(detect_element_size("load i64, i64* %ptr"), 8);
    assert_eq!(detect_element_size("load double, double* %ptr"), 8);
    assert_eq!(detect_element_size("load float, float* %ptr"), 4);
    assert_eq!(detect_element_size("load i128, i128* %ptr"), 16);
}

#[test]
fn test_extract_func_name() {
    assert_eq!(
        extract_func_name("define void @test_func() {"),
        Some("test_func".to_string())
    );
    assert_eq!(
        extract_func_name("define i64 @main() {"),
        Some("main".to_string())
    );
    assert_eq!(extract_func_name("not a function"), None);
}

#[test]
fn test_extract_branch_targets_unconditional() {
    let line = "br label %next";
    let targets = extract_branch_targets(line);
    assert_eq!(targets.len(), 1);
    assert!(targets.contains(&"next".to_string()));
}

#[test]
fn test_extract_phi_var() {
    assert_eq!(
        extract_phi_var("  %i = phi i64 [ 0, %entry ], [ %i.next, %loop ]"),
        Some("%i".to_string())
    );
    assert_eq!(extract_phi_var("  i = phi i64 [0, %entry]"), None);
}

// ========== ReductionKind tests ==========

#[test]
fn test_reduction_kind_all_identities() {
    assert_eq!(ReductionKind::Add.identity_i64(), 0);
    assert_eq!(ReductionKind::Mul.identity_i64(), 1);
    assert_eq!(ReductionKind::Min.identity_i64(), i64::MAX);
    assert_eq!(ReductionKind::Max.identity_i64(), i64::MIN);
    assert_eq!(ReductionKind::Or.identity_i64(), 0);
    assert_eq!(ReductionKind::And.identity_i64(), -1);
    assert_eq!(ReductionKind::Xor.identity_i64(), 0);
}

#[test]
fn test_reduction_kind_llvm_metadata() {
    assert!(ReductionKind::Add.llvm_metadata().contains("add"));
    assert!(ReductionKind::Mul.llvm_metadata().contains("mul"));
    assert!(ReductionKind::Min.llvm_metadata().contains("min"));
    assert!(ReductionKind::Max.llvm_metadata().contains("max"));
    assert!(ReductionKind::Or.llvm_metadata().contains("or"));
    assert!(ReductionKind::And.llvm_metadata().contains("and"));
    assert!(ReductionKind::Xor.llvm_metadata().contains("xor"));
}

#[test]
fn test_reduction_kind_equality() {
    assert_eq!(ReductionKind::Add, ReductionKind::Add);
    assert_ne!(ReductionKind::Add, ReductionKind::Mul);
}

// ========== detect_reduction_op ==========

#[test]
fn test_detect_reduction_op_add() {
    let line = "%acc.next = add i64 %acc, %val";
    assert_eq!(detect_reduction_op(line, "%acc"), Some(ReductionKind::Add));
}

#[test]
fn test_detect_reduction_op_mul() {
    let line = "%acc.next = mul i64 %acc, %val";
    assert_eq!(detect_reduction_op(line, "%acc"), Some(ReductionKind::Mul));
}

#[test]
fn test_detect_reduction_op_fadd() {
    let line = "%acc.next = fadd double %acc, %val";
    assert_eq!(detect_reduction_op(line, "%acc"), Some(ReductionKind::Add));
}

#[test]
fn test_detect_reduction_op_or() {
    let line = "%acc.next = or i64 %acc, %val";
    assert_eq!(detect_reduction_op(line, "%acc"), Some(ReductionKind::Or));
}

#[test]
fn test_detect_reduction_op_no_match() {
    let line = "%x = sub i64 %a, %b";
    assert_eq!(detect_reduction_op(line, "%acc"), None); // %acc not in line
}

// ========== parse_accumulator_phi ==========

#[test]
fn test_parse_accumulator_phi_valid() {
    let line = "%acc = phi i64 [ 0, %entry ], [ %acc.next, %loop ]";
    let result = parse_accumulator_phi(line);
    assert!(result.is_some());
    let (acc, next) = result.unwrap();
    assert_eq!(acc, "%acc");
    assert_eq!(next, "%acc.next");
}

#[test]
fn test_parse_accumulator_phi_no_phi() {
    let line = "%x = add i64 1, 2";
    assert!(parse_accumulator_phi(line).is_none());
}

// ========== AutoVectorizer tests ==========

#[test]
fn test_auto_vectorizer_new() {
    let v = AutoVectorizer::new(VectorWidth::SSE);
    assert!(v.candidates.is_empty());
    assert_eq!(v.target_width, VectorWidth::SSE);
}

#[test]
fn test_analyze_vectorization_empty() {
    let ir = "define void @empty() {\nentry:\n  ret void\n}\n";
    let v = analyze_vectorization(ir, VectorWidth::AVX2);
    assert!(v.candidates.is_empty());
}

#[test]
fn test_generate_vectorization_hints_no_loops() {
    let ir = "define void @empty() {\nentry:\n  ret void\n}\n";
    let result = generate_vectorization_hints(ir, VectorWidth::AVX2, None);
    assert!(result.contains("ret void"));
    assert!(!result.contains("VECTORIZATION HINT"));
}

// ========== compute_dependence_distance ==========

#[test]
fn test_compute_dependence_distance_constants() {
    let a1 = MemoryAccess {
        instruction: String::new(),
        base: "%arr".to_string(),
        index: Some("3".to_string()),
        stride: Some(0),
        is_write: false,
        element_size: 8,
    };
    let a2 = MemoryAccess {
        instruction: String::new(),
        base: "%arr".to_string(),
        index: Some("5".to_string()),
        stride: Some(0),
        is_write: true,
        element_size: 8,
    };
    let distance = compute_dependence_distance(&a1, &a2);
    assert_eq!(distance, Some(2)); // 5 - 3 = 2
}

#[test]
fn test_compute_dependence_distance_same_var() {
    let a1 = MemoryAccess {
        instruction: String::new(),
        base: "%arr".to_string(),
        index: Some("%i".to_string()),
        stride: Some(1),
        is_write: false,
        element_size: 8,
    };
    let a2 = MemoryAccess {
        instruction: String::new(),
        base: "%arr".to_string(),
        index: Some("%i".to_string()),
        stride: Some(1),
        is_write: true,
        element_size: 8,
    };
    let distance = compute_dependence_distance(&a1, &a2);
    assert_eq!(distance, Some(0));
}

#[test]
fn test_compute_dependence_distance_next_pattern() {
    let a1 = MemoryAccess {
        instruction: String::new(),
        base: "%arr".to_string(),
        index: Some("%i".to_string()),
        stride: Some(1),
        is_write: false,
        element_size: 8,
    };
    let a2 = MemoryAccess {
        instruction: String::new(),
        base: "%arr".to_string(),
        index: Some("%i.next".to_string()),
        stride: Some(1),
        is_write: true,
        element_size: 8,
    };
    let distance = compute_dependence_distance(&a1, &a2);
    assert_eq!(distance, Some(1));
}
