use super::*;

// ── SimdTarget::parse tests ──

#[test]
fn test_simd_target_from_str() {
    assert_eq!(SimdTarget::parse("avx512"), Some(SimdTarget::Avx512));
    assert_eq!(SimdTarget::parse("avx2"), Some(SimdTarget::Avx2));
    assert_eq!(SimdTarget::parse("sse4"), Some(SimdTarget::Sse4));
    assert_eq!(SimdTarget::parse("neon"), Some(SimdTarget::Neon));
    assert_eq!(SimdTarget::parse("sve"), Some(SimdTarget::Sve));
    assert_eq!(SimdTarget::parse("unknown"), None);
}

#[test]
fn test_simd_target_parse_aliases() {
    assert_eq!(SimdTarget::parse("avx-512"), Some(SimdTarget::Avx512));
    assert_eq!(SimdTarget::parse("avx-2"), Some(SimdTarget::Avx2));
    assert_eq!(SimdTarget::parse("sse4.2"), Some(SimdTarget::Sse4));
    assert_eq!(SimdTarget::parse("sse"), Some(SimdTarget::Sse4));
    assert_eq!(SimdTarget::parse("arm-neon"), Some(SimdTarget::Neon));
    assert_eq!(SimdTarget::parse("arm-sve"), Some(SimdTarget::Sve));
}

#[test]
fn test_simd_target_parse_case_insensitive() {
    assert_eq!(SimdTarget::parse("AVX512"), Some(SimdTarget::Avx512));
    assert_eq!(SimdTarget::parse("AVX2"), Some(SimdTarget::Avx2));
    assert_eq!(SimdTarget::parse("SSE4"), Some(SimdTarget::Sse4));
    assert_eq!(SimdTarget::parse("NEON"), Some(SimdTarget::Neon));
    assert_eq!(SimdTarget::parse("SVE"), Some(SimdTarget::Sve));
}

#[test]
fn test_simd_target_parse_unknown() {
    assert_eq!(SimdTarget::parse(""), None);
    assert_eq!(SimdTarget::parse("gpu"), None);
    assert_eq!(SimdTarget::parse("avx1"), None);
}

// ── SimdTarget::vector_bits tests ──

#[test]
fn test_simd_target_vector_bits() {
    assert_eq!(SimdTarget::Avx512.vector_bits(), 512);
    assert_eq!(SimdTarget::Avx2.vector_bits(), 256);
    assert_eq!(SimdTarget::Sse4.vector_bits(), 128);
    assert_eq!(SimdTarget::Neon.vector_bits(), 128);
    assert_eq!(SimdTarget::Sve.vector_bits(), 512);
}

// ── SimdTarget lane count tests ──

#[test]
fn test_simd_target_f32_lanes() {
    assert_eq!(SimdTarget::Avx512.f32_lanes(), 16);
    assert_eq!(SimdTarget::Avx2.f32_lanes(), 8);
    assert_eq!(SimdTarget::Sse4.f32_lanes(), 4);
    assert_eq!(SimdTarget::Neon.f32_lanes(), 4);
    assert_eq!(SimdTarget::Sve.f32_lanes(), 16);
}

#[test]
fn test_simd_target_f64_lanes() {
    assert_eq!(SimdTarget::Avx512.f64_lanes(), 8);
    assert_eq!(SimdTarget::Avx2.f64_lanes(), 4);
    assert_eq!(SimdTarget::Sse4.f64_lanes(), 2);
    assert_eq!(SimdTarget::Neon.f64_lanes(), 2);
    assert_eq!(SimdTarget::Sve.f64_lanes(), 8);
}

#[test]
fn test_simd_target_i32_lanes() {
    assert_eq!(SimdTarget::Avx512.i32_lanes(), 16);
    assert_eq!(SimdTarget::Avx2.i32_lanes(), 8);
    assert_eq!(SimdTarget::Sse4.i32_lanes(), 4);
    assert_eq!(SimdTarget::Neon.i32_lanes(), 4);
}

#[test]
fn test_simd_lanes_consistent_with_bits() {
    for target in [
        SimdTarget::Avx512,
        SimdTarget::Avx2,
        SimdTarget::Sse4,
        SimdTarget::Neon,
        SimdTarget::Sve,
    ] {
        assert_eq!(target.f32_lanes(), target.vector_bits() / 32);
        assert_eq!(target.f64_lanes(), target.vector_bits() / 64);
        assert_eq!(target.i32_lanes(), target.vector_bits() / 32);
    }
}

// ── SimdTarget::compiler_flags tests ──

#[test]
fn test_simd_target_compiler_flags_all() {
    assert!(SimdTarget::Avx512.compiler_flags().contains("-mavx512f"));
    assert!(SimdTarget::Avx512.compiler_flags().contains("-mavx512dq"));
    assert!(SimdTarget::Avx2.compiler_flags().contains("-mavx2"));
    assert!(SimdTarget::Avx2.compiler_flags().contains("-mfma"));
    assert!(SimdTarget::Sse4.compiler_flags().contains("-msse4.2"));
    assert!(SimdTarget::Neon.compiler_flags().contains("-mfpu=neon"));
    assert!(SimdTarget::Sve
        .compiler_flags()
        .contains("-march=armv8-a+sve"));
}

// ── SimdTarget::headers tests ──

#[test]
fn test_simd_target_headers_all() {
    assert!(SimdTarget::Avx512.headers().contains("immintrin.h"));
    assert!(SimdTarget::Avx2.headers().contains("immintrin.h"));
    assert!(SimdTarget::Sse4.headers().contains("immintrin.h"));
    assert!(SimdTarget::Neon.headers().contains("arm_neon.h"));
    assert!(SimdTarget::Sve.headers().contains("arm_sve.h"));
}

#[test]
fn test_simd_target_intel_share_header() {
    // All Intel SIMD targets share the same header
    assert_eq!(SimdTarget::Avx512.headers(), SimdTarget::Avx2.headers());
    assert_eq!(SimdTarget::Avx2.headers(), SimdTarget::Sse4.headers());
}

// ── SimdTarget::name tests ──

#[test]
fn test_simd_target_name_all() {
    assert_eq!(SimdTarget::Avx512.name(), "AVX-512");
    assert_eq!(SimdTarget::Avx2.name(), "AVX2");
    assert_eq!(SimdTarget::Sse4.name(), "SSE4");
    assert_eq!(SimdTarget::Neon.name(), "NEON");
    assert_eq!(SimdTarget::Sve.name(), "SVE");
}

// ── SimdTarget traits ──

#[test]
fn test_simd_target_clone_copy() {
    let t = SimdTarget::Avx512;
    let cloned = t.clone();
    let copied = t;
    assert_eq!(t, cloned);
    assert_eq!(t, copied);
}

#[test]
fn test_simd_target_equality() {
    assert_eq!(SimdTarget::Avx512, SimdTarget::Avx512);
    assert_ne!(SimdTarget::Avx512, SimdTarget::Avx2);
}

#[test]
fn test_simd_target_debug() {
    let s = format!("{:?}", SimdTarget::Neon);
    assert_eq!(s, "Neon");
}

// ── SimdVectorType::type_name tests ──

#[test]
fn test_simd_vector_type_avx512() {
    assert_eq!(
        SimdVectorType::F32(16).type_name(SimdTarget::Avx512),
        "__m512"
    );
    assert_eq!(
        SimdVectorType::F64(8).type_name(SimdTarget::Avx512),
        "__m512d"
    );
    assert_eq!(
        SimdVectorType::I32(16).type_name(SimdTarget::Avx512),
        "__m512i"
    );
    assert_eq!(
        SimdVectorType::I64(8).type_name(SimdTarget::Avx512),
        "__m512i"
    );
}

#[test]
fn test_simd_vector_type_avx512_256() {
    assert_eq!(
        SimdVectorType::F32(8).type_name(SimdTarget::Avx512),
        "__m256"
    );
    assert_eq!(
        SimdVectorType::F64(4).type_name(SimdTarget::Avx512),
        "__m256d"
    );
}

#[test]
fn test_simd_vector_type_avx2() {
    assert_eq!(SimdVectorType::F32(8).type_name(SimdTarget::Avx2), "__m256");
    assert_eq!(
        SimdVectorType::F64(4).type_name(SimdTarget::Avx2),
        "__m256d"
    );
    assert_eq!(
        SimdVectorType::I32(8).type_name(SimdTarget::Avx2),
        "__m256i"
    );
    assert_eq!(
        SimdVectorType::I64(4).type_name(SimdTarget::Avx2),
        "__m256i"
    );
}

#[test]
fn test_simd_vector_type_avx2_128() {
    assert_eq!(SimdVectorType::F32(4).type_name(SimdTarget::Avx2), "__m128");
    assert_eq!(
        SimdVectorType::F64(2).type_name(SimdTarget::Avx2),
        "__m128d"
    );
    assert_eq!(
        SimdVectorType::I32(4).type_name(SimdTarget::Avx2),
        "__m128i"
    );
    assert_eq!(
        SimdVectorType::I64(2).type_name(SimdTarget::Avx2),
        "__m128i"
    );
}

#[test]
fn test_simd_vector_type_sse4() {
    assert_eq!(SimdVectorType::F32(4).type_name(SimdTarget::Sse4), "__m128");
    assert_eq!(
        SimdVectorType::F64(2).type_name(SimdTarget::Sse4),
        "__m128d"
    );
    assert_eq!(
        SimdVectorType::I32(4).type_name(SimdTarget::Sse4),
        "__m128i"
    );
    assert_eq!(
        SimdVectorType::I64(2).type_name(SimdTarget::Sse4),
        "__m128i"
    );
}

#[test]
fn test_simd_vector_type_neon() {
    assert_eq!(
        SimdVectorType::F32(4).type_name(SimdTarget::Neon),
        "float32x4_t"
    );
    assert_eq!(
        SimdVectorType::F32(2).type_name(SimdTarget::Neon),
        "float32x2_t"
    );
    assert_eq!(
        SimdVectorType::F64(2).type_name(SimdTarget::Neon),
        "float64x2_t"
    );
    assert_eq!(
        SimdVectorType::F64(1).type_name(SimdTarget::Neon),
        "float64x1_t"
    );
    assert_eq!(
        SimdVectorType::I32(4).type_name(SimdTarget::Neon),
        "int32x4_t"
    );
    assert_eq!(
        SimdVectorType::I32(2).type_name(SimdTarget::Neon),
        "int32x2_t"
    );
    assert_eq!(
        SimdVectorType::I64(2).type_name(SimdTarget::Neon),
        "int64x2_t"
    );
    assert_eq!(
        SimdVectorType::I64(1).type_name(SimdTarget::Neon),
        "int64x1_t"
    );
}

#[test]
fn test_simd_vector_type_sve() {
    assert_eq!(
        SimdVectorType::F32(16).type_name(SimdTarget::Sve),
        "svfloat32_t"
    );
    assert_eq!(
        SimdVectorType::F64(8).type_name(SimdTarget::Sve),
        "svfloat64_t"
    );
    assert_eq!(
        SimdVectorType::I32(16).type_name(SimdTarget::Sve),
        "svint32_t"
    );
    assert_eq!(
        SimdVectorType::I64(8).type_name(SimdTarget::Sve),
        "svint64_t"
    );
}

#[test]
fn test_simd_vector_type_unsupported_returns_void() {
    // Unsupported lane counts should return "void" for x86 targets
    assert_eq!(SimdVectorType::F32(3).type_name(SimdTarget::Sse4), "void");
    assert_eq!(SimdVectorType::I32(7).type_name(SimdTarget::Avx2), "void");
}

#[test]
fn test_simd_vector_type_equality() {
    assert_eq!(SimdVectorType::F32(4), SimdVectorType::F32(4));
    assert_ne!(SimdVectorType::F32(4), SimdVectorType::F32(8));
    assert_ne!(SimdVectorType::F32(4), SimdVectorType::F64(4));
}

#[test]
fn test_simd_vector_type_clone() {
    let ty = SimdVectorType::I64(4);
    let cloned = ty.clone();
    assert_eq!(ty, cloned);
}

// ── SimdIntrinsics tests ──

#[test]
fn test_simd_intrinsics_load_all_targets() {
    // f32
    assert_eq!(
        SimdIntrinsics::load(SimdTarget::Avx512, "f32"),
        "_mm512_loadu_ps"
    );
    assert_eq!(
        SimdIntrinsics::load(SimdTarget::Avx2, "f32"),
        "_mm256_loadu_ps"
    );
    assert_eq!(
        SimdIntrinsics::load(SimdTarget::Sse4, "f32"),
        "_mm_loadu_ps"
    );
    assert_eq!(SimdIntrinsics::load(SimdTarget::Neon, "f32"), "vld1q_f32");
    assert_eq!(SimdIntrinsics::load(SimdTarget::Sve, "f32"), "svld1_f32");
}

#[test]
fn test_simd_intrinsics_load_f64() {
    assert_eq!(
        SimdIntrinsics::load(SimdTarget::Avx512, "f64"),
        "_mm512_loadu_pd"
    );
    assert_eq!(
        SimdIntrinsics::load(SimdTarget::Avx2, "f64"),
        "_mm256_loadu_pd"
    );
    assert_eq!(
        SimdIntrinsics::load(SimdTarget::Sse4, "f64"),
        "_mm_loadu_pd"
    );
    assert_eq!(SimdIntrinsics::load(SimdTarget::Neon, "f64"), "vld1q_f64");
    assert_eq!(SimdIntrinsics::load(SimdTarget::Sve, "f64"), "svld1_f64");
}

#[test]
fn test_simd_intrinsics_load_i32() {
    assert_eq!(
        SimdIntrinsics::load(SimdTarget::Avx512, "i32"),
        "_mm512_loadu_si512"
    );
    assert_eq!(
        SimdIntrinsics::load(SimdTarget::Avx2, "i32"),
        "_mm256_loadu_si256"
    );
    assert_eq!(
        SimdIntrinsics::load(SimdTarget::Sse4, "i32"),
        "_mm_loadu_si128"
    );
    assert_eq!(SimdIntrinsics::load(SimdTarget::Neon, "i32"), "vld1q_s32");
}

#[test]
fn test_simd_intrinsics_load_unknown_type() {
    assert_eq!(
        SimdIntrinsics::load(SimdTarget::Avx512, "i8"),
        "unknown_load"
    );
}

#[test]
fn test_simd_intrinsics_store_all_targets_f32() {
    assert_eq!(
        SimdIntrinsics::store(SimdTarget::Avx512, "f32"),
        "_mm512_storeu_ps"
    );
    assert_eq!(
        SimdIntrinsics::store(SimdTarget::Avx2, "f32"),
        "_mm256_storeu_ps"
    );
    assert_eq!(
        SimdIntrinsics::store(SimdTarget::Sse4, "f32"),
        "_mm_storeu_ps"
    );
    assert_eq!(SimdIntrinsics::store(SimdTarget::Neon, "f32"), "vst1q_f32");
    assert_eq!(SimdIntrinsics::store(SimdTarget::Sve, "f32"), "svst1_f32");
}

#[test]
fn test_simd_intrinsics_add_all_targets_f32() {
    assert_eq!(
        SimdIntrinsics::add(SimdTarget::Avx512, "f32"),
        "_mm512_add_ps"
    );
    assert_eq!(
        SimdIntrinsics::add(SimdTarget::Avx2, "f32"),
        "_mm256_add_ps"
    );
    assert_eq!(SimdIntrinsics::add(SimdTarget::Sse4, "f32"), "_mm_add_ps");
    assert_eq!(SimdIntrinsics::add(SimdTarget::Neon, "f32"), "vaddq_f32");
    assert_eq!(SimdIntrinsics::add(SimdTarget::Sve, "f32"), "svadd_f32_x");
}

#[test]
fn test_simd_intrinsics_sub_all_targets_f32() {
    assert_eq!(
        SimdIntrinsics::sub(SimdTarget::Avx512, "f32"),
        "_mm512_sub_ps"
    );
    assert_eq!(
        SimdIntrinsics::sub(SimdTarget::Avx2, "f32"),
        "_mm256_sub_ps"
    );
    assert_eq!(SimdIntrinsics::sub(SimdTarget::Sse4, "f32"), "_mm_sub_ps");
    assert_eq!(SimdIntrinsics::sub(SimdTarget::Neon, "f32"), "vsubq_f32");
}

#[test]
fn test_simd_intrinsics_mul_all_targets_f32() {
    assert_eq!(
        SimdIntrinsics::mul(SimdTarget::Avx512, "f32"),
        "_mm512_mul_ps"
    );
    assert_eq!(
        SimdIntrinsics::mul(SimdTarget::Avx2, "f32"),
        "_mm256_mul_ps"
    );
    assert_eq!(SimdIntrinsics::mul(SimdTarget::Sse4, "f32"), "_mm_mul_ps");
    assert_eq!(SimdIntrinsics::mul(SimdTarget::Neon, "f32"), "vmulq_f32");
}

#[test]
fn test_simd_intrinsics_div_all_targets_f32() {
    assert_eq!(
        SimdIntrinsics::div(SimdTarget::Avx512, "f32"),
        "_mm512_div_ps"
    );
    assert_eq!(
        SimdIntrinsics::div(SimdTarget::Avx2, "f32"),
        "_mm256_div_ps"
    );
    assert_eq!(SimdIntrinsics::div(SimdTarget::Sse4, "f32"), "_mm_div_ps");
    assert_eq!(SimdIntrinsics::div(SimdTarget::Neon, "f32"), "vdivq_f32");
}

#[test]
fn test_simd_intrinsics_div_no_i32() {
    // Integer division is not natively supported
    assert_eq!(
        SimdIntrinsics::div(SimdTarget::Avx512, "i32"),
        "unknown_div"
    );
    assert_eq!(SimdIntrinsics::div(SimdTarget::Sse4, "i32"), "unknown_div");
}

#[test]
fn test_simd_intrinsics_fma_all_targets() {
    assert_eq!(
        SimdIntrinsics::fma(SimdTarget::Avx512, "f32"),
        "_mm512_fmadd_ps"
    );
    assert_eq!(
        SimdIntrinsics::fma(SimdTarget::Avx512, "f64"),
        "_mm512_fmadd_pd"
    );
    assert_eq!(
        SimdIntrinsics::fma(SimdTarget::Avx2, "f32"),
        "_mm256_fmadd_ps"
    );
    assert_eq!(SimdIntrinsics::fma(SimdTarget::Sse4, "f32"), "_mm_fmadd_ps");
    assert_eq!(SimdIntrinsics::fma(SimdTarget::Neon, "f32"), "vfmaq_f32");
    assert_eq!(SimdIntrinsics::fma(SimdTarget::Sve, "f64"), "svmla_f64_x");
}

#[test]
fn test_simd_intrinsics_fma_no_i32() {
    assert_eq!(
        SimdIntrinsics::fma(SimdTarget::Avx512, "i32"),
        "unknown_fma"
    );
}

#[test]
fn test_simd_intrinsics_sqrt_all_targets() {
    assert_eq!(
        SimdIntrinsics::sqrt(SimdTarget::Avx512, "f32"),
        "_mm512_sqrt_ps"
    );
    assert_eq!(
        SimdIntrinsics::sqrt(SimdTarget::Avx2, "f32"),
        "_mm256_sqrt_ps"
    );
    assert_eq!(SimdIntrinsics::sqrt(SimdTarget::Sse4, "f32"), "_mm_sqrt_ps");
    assert_eq!(SimdIntrinsics::sqrt(SimdTarget::Neon, "f32"), "vsqrtq_f32");
    assert_eq!(SimdIntrinsics::sqrt(SimdTarget::Sve, "f32"), "svsqrt_f32_x");
}

#[test]
fn test_simd_intrinsics_sqrt_no_i32() {
    assert_eq!(
        SimdIntrinsics::sqrt(SimdTarget::Avx512, "i32"),
        "unknown_sqrt"
    );
}

#[test]
fn test_simd_intrinsics_reduce_add() {
    assert_eq!(
        SimdIntrinsics::reduce_add(SimdTarget::Avx512, "f32"),
        "_mm512_reduce_add_ps"
    );
    assert_eq!(
        SimdIntrinsics::reduce_add(SimdTarget::Avx512, "f64"),
        "_mm512_reduce_add_pd"
    );
    assert_eq!(
        SimdIntrinsics::reduce_add(SimdTarget::Avx512, "i32"),
        "_mm512_reduce_add_epi32"
    );
    assert_eq!(
        SimdIntrinsics::reduce_add(SimdTarget::Neon, "f32"),
        "vaddvq_f32"
    );
    assert_eq!(
        SimdIntrinsics::reduce_add(SimdTarget::Sve, "f32"),
        "svaddv_f32"
    );
}

#[test]
fn test_simd_intrinsics_reduce_add_no_avx2() {
    // AVX2 doesn't have native reduce_add
    assert_eq!(
        SimdIntrinsics::reduce_add(SimdTarget::Avx2, "f32"),
        "unknown_reduce_add"
    );
}

#[test]
fn test_simd_intrinsics_broadcast_all_targets() {
    assert_eq!(
        SimdIntrinsics::broadcast(SimdTarget::Avx512, "f32"),
        "_mm512_set1_ps"
    );
    assert_eq!(
        SimdIntrinsics::broadcast(SimdTarget::Avx2, "f32"),
        "_mm256_set1_ps"
    );
    assert_eq!(
        SimdIntrinsics::broadcast(SimdTarget::Sse4, "f32"),
        "_mm_set1_ps"
    );
    assert_eq!(
        SimdIntrinsics::broadcast(SimdTarget::Neon, "f32"),
        "vdupq_n_f32"
    );
    assert_eq!(
        SimdIntrinsics::broadcast(SimdTarget::Sve, "f32"),
        "svdup_f32"
    );
}

#[test]
fn test_simd_intrinsics_min_all_targets() {
    assert_eq!(
        SimdIntrinsics::min(SimdTarget::Avx512, "f32"),
        "_mm512_min_ps"
    );
    assert_eq!(
        SimdIntrinsics::min(SimdTarget::Avx2, "f32"),
        "_mm256_min_ps"
    );
    assert_eq!(SimdIntrinsics::min(SimdTarget::Sse4, "f32"), "_mm_min_ps");
    assert_eq!(SimdIntrinsics::min(SimdTarget::Neon, "f32"), "vminq_f32");
    assert_eq!(SimdIntrinsics::min(SimdTarget::Sve, "f32"), "svmin_f32_x");
}

#[test]
fn test_simd_intrinsics_max_all_targets() {
    assert_eq!(
        SimdIntrinsics::max(SimdTarget::Avx512, "f32"),
        "_mm512_max_ps"
    );
    assert_eq!(
        SimdIntrinsics::max(SimdTarget::Avx2, "f32"),
        "_mm256_max_ps"
    );
    assert_eq!(SimdIntrinsics::max(SimdTarget::Sse4, "f32"), "_mm_max_ps");
    assert_eq!(SimdIntrinsics::max(SimdTarget::Neon, "f32"), "vmaxq_f32");
    assert_eq!(SimdIntrinsics::max(SimdTarget::Sve, "f32"), "svmax_f32_x");
}

#[test]
fn test_simd_intrinsics_i32_arithmetic() {
    assert_eq!(
        SimdIntrinsics::add(SimdTarget::Avx512, "i32"),
        "_mm512_add_epi32"
    );
    assert_eq!(
        SimdIntrinsics::sub(SimdTarget::Avx512, "i32"),
        "_mm512_sub_epi32"
    );
    assert_eq!(
        SimdIntrinsics::mul(SimdTarget::Avx512, "i32"),
        "_mm512_mullo_epi32"
    );
}

#[test]
fn test_simd_intrinsics_neon_i32() {
    assert_eq!(SimdIntrinsics::add(SimdTarget::Neon, "i32"), "vaddq_s32");
    assert_eq!(SimdIntrinsics::sub(SimdTarget::Neon, "i32"), "vsubq_s32");
    assert_eq!(SimdIntrinsics::mul(SimdTarget::Neon, "i32"), "vmulq_s32");
    assert_eq!(SimdIntrinsics::min(SimdTarget::Neon, "i32"), "vminq_s32");
    assert_eq!(SimdIntrinsics::max(SimdTarget::Neon, "i32"), "vmaxq_s32");
}

#[test]
fn test_simd_intrinsics_sve_f64() {
    assert_eq!(SimdIntrinsics::add(SimdTarget::Sve, "f64"), "svadd_f64_x");
    assert_eq!(SimdIntrinsics::sub(SimdTarget::Sve, "f64"), "svsub_f64_x");
    assert_eq!(SimdIntrinsics::mul(SimdTarget::Sve, "f64"), "svmul_f64_x");
    assert_eq!(SimdIntrinsics::div(SimdTarget::Sve, "f64"), "svdiv_f64_x");
}
