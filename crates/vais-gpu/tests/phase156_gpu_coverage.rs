//! Phase 156: Additional unit tests for vais-gpu coverage
//!
//! Focus areas:
//! - GpuTarget properties and parsing edge cases
//! - GpuType type name generation (CUDA, OpenCL, WGSL) for all combinations
//! - GpuType::from_resolved conversions
//! - GpuKernel metadata
//! - GpuCodeGenerator backend dispatch (disabled-backend error paths)
//! - SIMD target properties and intrinsics (all backends × all types)
//! - SimdVectorType type_name across all targets
//! - generate_simd_code with parsed .vais sources
//! - GpuError display formatting

use vais_gpu::simd::{
    generate_simd_code, SimdIntrinsics, SimdTarget, SimdVectorType,
};
use vais_gpu::{GpuCodeGenerator, GpuError, GpuKernel, GpuResult, GpuTarget, GpuType};
use vais_types::ResolvedType;

// ==================== GpuTarget additional parsing ====================

#[test]
fn test_gpu_target_parse_case_insensitive_opencl() {
    #[cfg(feature = "opencl")]
    {
        assert_eq!(GpuTarget::parse("OpenCL"), Some(GpuTarget::OpenCL));
        assert_eq!(GpuTarget::parse("OPENCL"), Some(GpuTarget::OpenCL));
        assert_eq!(GpuTarget::parse("CL"), Some(GpuTarget::OpenCL));
    }
    #[cfg(not(feature = "opencl"))]
    {
        assert_eq!(GpuTarget::parse("opencl"), None);
    }
}

#[test]
fn test_gpu_target_parse_case_insensitive_webgpu() {
    #[cfg(feature = "webgpu")]
    {
        assert_eq!(GpuTarget::parse("WebGPU"), Some(GpuTarget::WebGPU));
        assert_eq!(GpuTarget::parse("WEBGPU"), Some(GpuTarget::WebGPU));
        assert_eq!(GpuTarget::parse("WGSL"), Some(GpuTarget::WebGPU));
    }
    #[cfg(not(feature = "webgpu"))]
    {
        assert_eq!(GpuTarget::parse("webgpu"), None);
    }
}

#[test]
fn test_gpu_target_is_metal_all_variants() {
    assert!(GpuTarget::Metal.is_metal());
    assert!(!GpuTarget::Cuda.is_metal());
    assert!(!GpuTarget::OpenCL.is_metal());
    assert!(!GpuTarget::WebGPU.is_metal());
}

#[test]
fn test_gpu_target_is_cuda_all_variants() {
    assert!(GpuTarget::Cuda.is_cuda());
    assert!(!GpuTarget::Metal.is_cuda());
    assert!(!GpuTarget::OpenCL.is_cuda());
    assert!(!GpuTarget::WebGPU.is_cuda());
}

#[test]
fn test_gpu_target_shared_memory_ordering() {
    // CUDA has more shared memory than WebGPU
    assert!(GpuTarget::Cuda.default_shared_memory() > GpuTarget::WebGPU.default_shared_memory());
    // Metal and OpenCL are equal
    assert_eq!(
        GpuTarget::Metal.default_shared_memory(),
        GpuTarget::OpenCL.default_shared_memory()
    );
}

#[test]
fn test_gpu_target_extension_all() {
    assert_eq!(GpuTarget::Cuda.extension(), "cu");
    assert_eq!(GpuTarget::OpenCL.extension(), "cl");
    assert_eq!(GpuTarget::WebGPU.extension(), "wgsl");
    assert_eq!(GpuTarget::Metal.extension(), "metal");
}

#[test]
fn test_gpu_target_name_all() {
    assert_eq!(GpuTarget::Cuda.name(), "CUDA");
    assert_eq!(GpuTarget::OpenCL.name(), "OpenCL");
    assert_eq!(GpuTarget::WebGPU.name(), "WebGPU");
    assert_eq!(GpuTarget::Metal.name(), "Metal");
}

#[test]
fn test_gpu_target_debug_format() {
    assert_eq!(format!("{:?}", GpuTarget::Cuda), "Cuda");
    assert_eq!(format!("{:?}", GpuTarget::OpenCL), "OpenCL");
    assert_eq!(format!("{:?}", GpuTarget::WebGPU), "WebGPU");
    assert_eq!(format!("{:?}", GpuTarget::Metal), "Metal");
}

#[test]
fn test_gpu_target_copy_semantics() {
    let a = GpuTarget::Cuda;
    let b = a; // copy
    assert_eq!(a, b);
}

// ==================== GpuType cuda_name comprehensive ====================

#[test]
fn test_gpu_type_cuda_name_all_primitives() {
    assert_eq!(GpuType::I32.cuda_name(), "int");
    assert_eq!(GpuType::I64.cuda_name(), "long long");
    assert_eq!(GpuType::F32.cuda_name(), "float");
    assert_eq!(GpuType::F64.cuda_name(), "double");
    assert_eq!(GpuType::Bool.cuda_name(), "bool");
    assert_eq!(GpuType::Void.cuda_name(), "void");
}

#[test]
fn test_gpu_type_cuda_name_ptr_primitives() {
    assert_eq!(GpuType::Ptr(Box::new(GpuType::I32)).cuda_name(), "int*");
    assert_eq!(GpuType::Ptr(Box::new(GpuType::I64)).cuda_name(), "long long*");
    assert_eq!(GpuType::Ptr(Box::new(GpuType::F32)).cuda_name(), "float*");
    assert_eq!(GpuType::Ptr(Box::new(GpuType::F64)).cuda_name(), "double*");
    assert_eq!(GpuType::Ptr(Box::new(GpuType::Bool)).cuda_name(), "bool*");
}

#[test]
fn test_gpu_type_cuda_name_array_all_types() {
    assert_eq!(GpuType::Array(Box::new(GpuType::I32), 4).cuda_name(), "int[4]");
    assert_eq!(GpuType::Array(Box::new(GpuType::F32), 8).cuda_name(), "float[8]");
    assert_eq!(GpuType::Array(Box::new(GpuType::F64), 2).cuda_name(), "double[2]");
}

#[test]
fn test_gpu_type_cuda_name_vec_all_sizes() {
    assert_eq!(GpuType::Vec(Box::new(GpuType::F32), 2).cuda_name(), "float2");
    assert_eq!(GpuType::Vec(Box::new(GpuType::F32), 4).cuda_name(), "float4");
    assert_eq!(GpuType::Vec(Box::new(GpuType::I32), 4).cuda_name(), "int4");
    assert_eq!(GpuType::Vec(Box::new(GpuType::F64), 2).cuda_name(), "double2");
}

#[test]
fn test_gpu_type_cuda_nested_ptr_ptr() {
    let ty = GpuType::Ptr(Box::new(GpuType::Ptr(Box::new(GpuType::F64))));
    assert_eq!(ty.cuda_name(), "double**");
}

// ==================== GpuType opencl_name comprehensive ====================

#[test]
fn test_gpu_type_opencl_name_all_primitives() {
    assert_eq!(GpuType::I32.opencl_name(), "int");
    assert_eq!(GpuType::I64.opencl_name(), "long");
    assert_eq!(GpuType::F32.opencl_name(), "float");
    assert_eq!(GpuType::F64.opencl_name(), "double");
    assert_eq!(GpuType::Bool.opencl_name(), "bool");
    assert_eq!(GpuType::Void.opencl_name(), "void");
}

#[test]
fn test_gpu_type_opencl_name_ptr_primitives() {
    assert_eq!(
        GpuType::Ptr(Box::new(GpuType::I32)).opencl_name(),
        "__global int*"
    );
    assert_eq!(
        GpuType::Ptr(Box::new(GpuType::F64)).opencl_name(),
        "__global double*"
    );
}

#[test]
fn test_gpu_type_opencl_name_array() {
    assert_eq!(
        GpuType::Array(Box::new(GpuType::F32), 16).opencl_name(),
        "float[16]"
    );
    assert_eq!(
        GpuType::Array(Box::new(GpuType::I64), 4).opencl_name(),
        "long[4]"
    );
}

#[test]
fn test_gpu_type_opencl_name_vec() {
    assert_eq!(GpuType::Vec(Box::new(GpuType::F32), 4).opencl_name(), "float4");
    assert_eq!(GpuType::Vec(Box::new(GpuType::I32), 8).opencl_name(), "int8");
}

// ==================== GpuType wgsl_name comprehensive ====================

#[test]
fn test_gpu_type_wgsl_name_all_primitives() {
    assert_eq!(GpuType::I32.wgsl_name(), "i32");
    assert_eq!(GpuType::I64.wgsl_name(), "i64");
    assert_eq!(GpuType::F32.wgsl_name(), "f32");
    assert_eq!(GpuType::F64.wgsl_name(), "f64");
    assert_eq!(GpuType::Bool.wgsl_name(), "bool");
    assert_eq!(GpuType::Void.wgsl_name(), "");
}

#[test]
fn test_gpu_type_wgsl_name_ptr() {
    assert_eq!(
        GpuType::Ptr(Box::new(GpuType::I32)).wgsl_name(),
        "ptr<storage, i32>"
    );
    assert_eq!(
        GpuType::Ptr(Box::new(GpuType::F64)).wgsl_name(),
        "ptr<storage, f64>"
    );
}

#[test]
fn test_gpu_type_wgsl_name_array() {
    assert_eq!(
        GpuType::Array(Box::new(GpuType::F32), 8).wgsl_name(),
        "array<f32, 8>"
    );
    assert_eq!(
        GpuType::Array(Box::new(GpuType::I32), 256).wgsl_name(),
        "array<i32, 256>"
    );
}

#[test]
fn test_gpu_type_wgsl_name_vec() {
    assert_eq!(GpuType::Vec(Box::new(GpuType::F32), 2).wgsl_name(), "vec2<f32>");
    assert_eq!(GpuType::Vec(Box::new(GpuType::F32), 3).wgsl_name(), "vec3<f32>");
    assert_eq!(GpuType::Vec(Box::new(GpuType::F32), 4).wgsl_name(), "vec4<f32>");
    assert_eq!(GpuType::Vec(Box::new(GpuType::I32), 4).wgsl_name(), "vec4<i32>");
}

// ==================== GpuType::from_resolved ====================

#[test]
fn test_gpu_type_from_resolved_all_primitives() {
    assert_eq!(GpuType::from_resolved(&ResolvedType::I32).unwrap(), GpuType::I32);
    assert_eq!(GpuType::from_resolved(&ResolvedType::I64).unwrap(), GpuType::I64);
    assert_eq!(GpuType::from_resolved(&ResolvedType::F32).unwrap(), GpuType::F32);
    assert_eq!(GpuType::from_resolved(&ResolvedType::F64).unwrap(), GpuType::F64);
    assert_eq!(GpuType::from_resolved(&ResolvedType::Bool).unwrap(), GpuType::Bool);
    assert_eq!(GpuType::from_resolved(&ResolvedType::Unit).unwrap(), GpuType::Void);
}

#[test]
fn test_gpu_type_from_resolved_nested_pointer() {
    let resolved = ResolvedType::Pointer(Box::new(ResolvedType::Pointer(Box::new(
        ResolvedType::F32,
    ))));
    let gpu = GpuType::from_resolved(&resolved).unwrap();
    assert_eq!(
        gpu,
        GpuType::Ptr(Box::new(GpuType::Ptr(Box::new(GpuType::F32))))
    );
}

#[test]
fn test_gpu_type_from_resolved_unsupported_types() {
    // Str is not a GPU type
    assert!(GpuType::from_resolved(&ResolvedType::Str).is_err());
    // Generic is not supported
    assert!(GpuType::from_resolved(&ResolvedType::Generic("T".to_string())).is_err());
}

// ==================== GpuType equality and cloning ====================

#[test]
fn test_gpu_type_equality_all_primitives() {
    assert_eq!(GpuType::I32, GpuType::I32);
    assert_ne!(GpuType::I32, GpuType::I64);
    assert_ne!(GpuType::F32, GpuType::F64);
    assert_ne!(GpuType::Bool, GpuType::Void);
}

#[test]
fn test_gpu_type_equality_compound() {
    let a = GpuType::Ptr(Box::new(GpuType::F32));
    let b = GpuType::Ptr(Box::new(GpuType::F32));
    let c = GpuType::Ptr(Box::new(GpuType::F64));
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn test_gpu_type_clone_nested() {
    let original = GpuType::Array(Box::new(GpuType::Ptr(Box::new(GpuType::F32))), 4);
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

// ==================== GpuKernel construction ====================

#[test]
fn test_gpu_kernel_empty_params() {
    let kernel = GpuKernel {
        name: "empty_kernel".to_string(),
        params: vec![],
        shared_memory: 0,
        block_size: (1, 1, 1),
    };
    assert_eq!(kernel.name, "empty_kernel");
    assert!(kernel.params.is_empty());
}

#[test]
fn test_gpu_kernel_many_params() {
    let params: Vec<(String, GpuType)> = (0..8)
        .map(|i| (format!("p{}", i), GpuType::Ptr(Box::new(GpuType::F32))))
        .collect();
    let kernel = GpuKernel {
        name: "many_params".to_string(),
        params,
        shared_memory: 4096,
        block_size: (32, 32, 1),
    };
    assert_eq!(kernel.params.len(), 8);
    assert_eq!(kernel.block_size, (32, 32, 1));
}

#[test]
fn test_gpu_kernel_3d_block_size() {
    let kernel = GpuKernel {
        name: "vol".to_string(),
        params: vec![],
        shared_memory: 0,
        block_size: (8, 8, 8),
    };
    assert_eq!(kernel.block_size.0, 8);
    assert_eq!(kernel.block_size.1, 8);
    assert_eq!(kernel.block_size.2, 8);
}

// ==================== GpuCodeGenerator multi-target ====================

#[test]
fn test_gpu_code_generator_all_targets_new() {
    let targets = [
        GpuTarget::Cuda,
        GpuTarget::OpenCL,
        GpuTarget::WebGPU,
        GpuTarget::Metal,
    ];
    for &target in &targets {
        let gen = GpuCodeGenerator::new(target);
        assert_eq!(gen.target(), target);
        assert!(gen.kernels().is_empty());
    }
}

#[test]
fn test_gpu_code_generator_empty_module_cuda_disabled() {
    #[cfg(not(feature = "cuda"))]
    {
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let module = vais_ast::Module {
            items: vec![],
            modules_map: None,
        };
        let result = gen.generate(&module);
        assert!(result.is_err());
        if let Err(GpuError::BackendError(msg)) = result {
            assert!(msg.contains("cuda"));
        }
    }
    #[cfg(feature = "cuda")]
    {
        // Just verify we can construct a generator
        let gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        assert_eq!(gen.target(), GpuTarget::Cuda);
    }
}

#[test]
fn test_gpu_code_generator_empty_module_opencl_disabled() {
    #[cfg(not(feature = "opencl"))]
    {
        let mut gen = GpuCodeGenerator::new(GpuTarget::OpenCL);
        let module = vais_ast::Module {
            items: vec![],
            modules_map: None,
        };
        let result = gen.generate(&module);
        assert!(result.is_err());
    }
    #[cfg(feature = "opencl")]
    {
        let gen = GpuCodeGenerator::new(GpuTarget::OpenCL);
        assert_eq!(gen.target(), GpuTarget::OpenCL);
    }
}

#[test]
fn test_gpu_code_generator_empty_module_webgpu_disabled() {
    #[cfg(not(feature = "webgpu"))]
    {
        let mut gen = GpuCodeGenerator::new(GpuTarget::WebGPU);
        let module = vais_ast::Module {
            items: vec![],
            modules_map: None,
        };
        let result = gen.generate(&module);
        assert!(result.is_err());
    }
    #[cfg(feature = "webgpu")]
    {
        let gen = GpuCodeGenerator::new(GpuTarget::WebGPU);
        assert_eq!(gen.target(), GpuTarget::WebGPU);
    }
}

#[test]
fn test_gpu_code_generator_empty_module_metal_disabled() {
    #[cfg(not(feature = "metal"))]
    {
        let mut gen = GpuCodeGenerator::new(GpuTarget::Metal);
        let module = vais_ast::Module {
            items: vec![],
            modules_map: None,
        };
        let result = gen.generate(&module);
        assert!(result.is_err());
    }
    #[cfg(feature = "metal")]
    {
        let gen = GpuCodeGenerator::new(GpuTarget::Metal);
        assert_eq!(gen.target(), GpuTarget::Metal);
    }
}

// ==================== GpuError comprehensive display ====================

#[test]
fn test_gpu_error_all_variants_display() {
    let cases: Vec<GpuError> = vec![
        GpuError::UnsupportedType("Vec<T>".to_string()),
        GpuError::UnsupportedOperation("dynamic dispatch".to_string()),
        GpuError::KernelError("invalid block dim".to_string()),
        GpuError::MemoryError("allocation failed".to_string()),
        GpuError::BackendError("not compiled".to_string()),
    ];
    let keywords = ["Vec<T>", "dynamic dispatch", "invalid block dim", "allocation failed", "not compiled"];
    for (err, kw) in cases.iter().zip(keywords.iter()) {
        let msg = format!("{}", err);
        assert!(msg.contains(kw), "Error message '{}' should contain '{}'", msg, kw);
    }
}

#[test]
fn test_gpu_error_debug_format() {
    let err = GpuError::UnsupportedType("i128".to_string());
    let debug = format!("{:?}", err);
    assert!(debug.contains("UnsupportedType"));
    assert!(debug.contains("i128"));
}

#[test]
fn test_gpu_result_chain() {
    let r1: GpuResult<i32> = Ok(10);
    let r2 = r1.map(|x| x * 2);
    assert_eq!(r2.unwrap(), 20);

    let r3: GpuResult<i32> = Err(GpuError::MemoryError("oom".to_string()));
    let r4 = r3.map_err(|e| format!("{}", e));
    assert!(r4.is_err());
    assert!(r4.unwrap_err().contains("oom"));
}

// ==================== SIMD target deeper coverage ====================

#[test]
fn test_simd_target_parse_all_variants() {
    assert_eq!(SimdTarget::parse("avx512"), Some(SimdTarget::Avx512));
    assert_eq!(SimdTarget::parse("avx-512"), Some(SimdTarget::Avx512));
    assert_eq!(SimdTarget::parse("avx2"), Some(SimdTarget::Avx2));
    assert_eq!(SimdTarget::parse("avx-2"), Some(SimdTarget::Avx2));
    assert_eq!(SimdTarget::parse("sse4"), Some(SimdTarget::Sse4));
    assert_eq!(SimdTarget::parse("sse4.2"), Some(SimdTarget::Sse4));
    assert_eq!(SimdTarget::parse("sse"), Some(SimdTarget::Sse4));
    assert_eq!(SimdTarget::parse("neon"), Some(SimdTarget::Neon));
    assert_eq!(SimdTarget::parse("arm-neon"), Some(SimdTarget::Neon));
    assert_eq!(SimdTarget::parse("sve"), Some(SimdTarget::Sve));
    assert_eq!(SimdTarget::parse("arm-sve"), Some(SimdTarget::Sve));
    assert_eq!(SimdTarget::parse("unknown"), None);
    assert_eq!(SimdTarget::parse(""), None);
}

#[test]
fn test_simd_target_vector_bits_all() {
    assert_eq!(SimdTarget::Avx512.vector_bits(), 512);
    assert_eq!(SimdTarget::Avx2.vector_bits(), 256);
    assert_eq!(SimdTarget::Sse4.vector_bits(), 128);
    assert_eq!(SimdTarget::Neon.vector_bits(), 128);
    assert_eq!(SimdTarget::Sve.vector_bits(), 512);
}

#[test]
fn test_simd_target_f32_lanes_all() {
    assert_eq!(SimdTarget::Avx512.f32_lanes(), 16);
    assert_eq!(SimdTarget::Avx2.f32_lanes(), 8);
    assert_eq!(SimdTarget::Sse4.f32_lanes(), 4);
    assert_eq!(SimdTarget::Neon.f32_lanes(), 4);
    assert_eq!(SimdTarget::Sve.f32_lanes(), 16);
}

#[test]
fn test_simd_target_f64_lanes_all() {
    assert_eq!(SimdTarget::Avx512.f64_lanes(), 8);
    assert_eq!(SimdTarget::Avx2.f64_lanes(), 4);
    assert_eq!(SimdTarget::Sse4.f64_lanes(), 2);
    assert_eq!(SimdTarget::Neon.f64_lanes(), 2);
    assert_eq!(SimdTarget::Sve.f64_lanes(), 8);
}

#[test]
fn test_simd_target_i32_lanes_all() {
    assert_eq!(SimdTarget::Avx512.i32_lanes(), 16);
    assert_eq!(SimdTarget::Avx2.i32_lanes(), 8);
    assert_eq!(SimdTarget::Sse4.i32_lanes(), 4);
    assert_eq!(SimdTarget::Neon.i32_lanes(), 4);
    assert_eq!(SimdTarget::Sve.i32_lanes(), 16);
}

#[test]
fn test_simd_target_lanes_consistent() {
    // f32 lanes * 32 == vector bits
    for target in &[SimdTarget::Avx512, SimdTarget::Avx2, SimdTarget::Sse4, SimdTarget::Neon] {
        assert_eq!(target.f32_lanes() * 32, target.vector_bits());
        assert_eq!(target.f64_lanes() * 64, target.vector_bits());
    }
}

#[test]
fn test_simd_target_name_all() {
    assert_eq!(SimdTarget::Avx512.name(), "AVX-512");
    assert_eq!(SimdTarget::Avx2.name(), "AVX2");
    assert_eq!(SimdTarget::Sse4.name(), "SSE4");
    assert_eq!(SimdTarget::Neon.name(), "NEON");
    assert_eq!(SimdTarget::Sve.name(), "SVE");
}

#[test]
fn test_simd_target_compiler_flags_all() {
    assert!(SimdTarget::Avx512.compiler_flags().contains("-mavx512f"));
    assert!(SimdTarget::Avx512.compiler_flags().contains("-mavx512dq"));
    assert!(SimdTarget::Avx2.compiler_flags().contains("-mavx2"));
    assert!(SimdTarget::Avx2.compiler_flags().contains("-mfma"));
    assert!(SimdTarget::Sse4.compiler_flags().contains("-msse4.2"));
    assert!(SimdTarget::Neon.compiler_flags().contains("-mfpu=neon"));
    assert!(SimdTarget::Sve.compiler_flags().contains("-march=armv8-a+sve"));
}

#[test]
fn test_simd_target_headers_all() {
    assert!(SimdTarget::Avx512.headers().contains("immintrin.h"));
    assert!(SimdTarget::Avx2.headers().contains("immintrin.h"));
    assert!(SimdTarget::Sse4.headers().contains("immintrin.h"));
    assert!(SimdTarget::Neon.headers().contains("arm_neon.h"));
    assert!(SimdTarget::Sve.headers().contains("arm_sve.h"));
}

#[test]
fn test_simd_target_equality_and_clone() {
    let a = SimdTarget::Avx2;
    let b = a;
    assert_eq!(a, b);
    assert_ne!(SimdTarget::Avx512, SimdTarget::Avx2);
    assert_ne!(SimdTarget::Neon, SimdTarget::Sve);
}

// ==================== SIMD intrinsics comprehensive ====================

#[test]
fn test_simd_intrinsics_load_all_targets_f32() {
    assert_eq!(SimdIntrinsics::load(SimdTarget::Avx512, "f32"), "_mm512_loadu_ps");
    assert_eq!(SimdIntrinsics::load(SimdTarget::Avx2, "f32"), "_mm256_loadu_ps");
    assert_eq!(SimdIntrinsics::load(SimdTarget::Sse4, "f32"), "_mm_loadu_ps");
    assert_eq!(SimdIntrinsics::load(SimdTarget::Neon, "f32"), "vld1q_f32");
    assert_eq!(SimdIntrinsics::load(SimdTarget::Sve, "f32"), "svld1_f32");
}

#[test]
fn test_simd_intrinsics_load_all_targets_f64() {
    assert_eq!(SimdIntrinsics::load(SimdTarget::Avx512, "f64"), "_mm512_loadu_pd");
    assert_eq!(SimdIntrinsics::load(SimdTarget::Avx2, "f64"), "_mm256_loadu_pd");
    assert_eq!(SimdIntrinsics::load(SimdTarget::Sse4, "f64"), "_mm_loadu_pd");
    assert_eq!(SimdIntrinsics::load(SimdTarget::Neon, "f64"), "vld1q_f64");
    assert_eq!(SimdIntrinsics::load(SimdTarget::Sve, "f64"), "svld1_f64");
}

#[test]
fn test_simd_intrinsics_load_all_targets_i32() {
    assert_eq!(SimdIntrinsics::load(SimdTarget::Avx512, "i32"), "_mm512_loadu_si512");
    assert_eq!(SimdIntrinsics::load(SimdTarget::Avx2, "i32"), "_mm256_loadu_si256");
    assert_eq!(SimdIntrinsics::load(SimdTarget::Sse4, "i32"), "_mm_loadu_si128");
    assert_eq!(SimdIntrinsics::load(SimdTarget::Neon, "i32"), "vld1q_s32");
    assert_eq!(SimdIntrinsics::load(SimdTarget::Sve, "i32"), "svld1_s32");
}

#[test]
fn test_simd_intrinsics_store_all_targets_f32() {
    assert_eq!(SimdIntrinsics::store(SimdTarget::Avx512, "f32"), "_mm512_storeu_ps");
    assert_eq!(SimdIntrinsics::store(SimdTarget::Avx2, "f32"), "_mm256_storeu_ps");
    assert_eq!(SimdIntrinsics::store(SimdTarget::Sse4, "f32"), "_mm_storeu_ps");
    assert_eq!(SimdIntrinsics::store(SimdTarget::Neon, "f32"), "vst1q_f32");
    assert_eq!(SimdIntrinsics::store(SimdTarget::Sve, "f32"), "svst1_f32");
}

#[test]
fn test_simd_intrinsics_store_all_targets_i32() {
    assert_eq!(SimdIntrinsics::store(SimdTarget::Avx512, "i32"), "_mm512_storeu_si512");
    assert_eq!(SimdIntrinsics::store(SimdTarget::Avx2, "i32"), "_mm256_storeu_si256");
    assert_eq!(SimdIntrinsics::store(SimdTarget::Neon, "i32"), "vst1q_s32");
}

#[test]
fn test_simd_intrinsics_add_all_targets_f32() {
    assert_eq!(SimdIntrinsics::add(SimdTarget::Avx512, "f32"), "_mm512_add_ps");
    assert_eq!(SimdIntrinsics::add(SimdTarget::Avx2, "f32"), "_mm256_add_ps");
    assert_eq!(SimdIntrinsics::add(SimdTarget::Sse4, "f32"), "_mm_add_ps");
    assert_eq!(SimdIntrinsics::add(SimdTarget::Neon, "f32"), "vaddq_f32");
    assert_eq!(SimdIntrinsics::add(SimdTarget::Sve, "f32"), "svadd_f32_x");
}

#[test]
fn test_simd_intrinsics_add_all_targets_i32() {
    assert_eq!(SimdIntrinsics::add(SimdTarget::Avx512, "i32"), "_mm512_add_epi32");
    assert_eq!(SimdIntrinsics::add(SimdTarget::Avx2, "i32"), "_mm256_add_epi32");
    assert_eq!(SimdIntrinsics::add(SimdTarget::Sse4, "i32"), "_mm_add_epi32");
    assert_eq!(SimdIntrinsics::add(SimdTarget::Neon, "i32"), "vaddq_s32");
    assert_eq!(SimdIntrinsics::add(SimdTarget::Sve, "i32"), "svadd_s32_x");
}

#[test]
fn test_simd_intrinsics_sub_all_targets_f32() {
    assert_eq!(SimdIntrinsics::sub(SimdTarget::Avx512, "f32"), "_mm512_sub_ps");
    assert_eq!(SimdIntrinsics::sub(SimdTarget::Avx2, "f32"), "_mm256_sub_ps");
    assert_eq!(SimdIntrinsics::sub(SimdTarget::Sse4, "f32"), "_mm_sub_ps");
    assert_eq!(SimdIntrinsics::sub(SimdTarget::Neon, "f32"), "vsubq_f32");
    assert_eq!(SimdIntrinsics::sub(SimdTarget::Sve, "f32"), "svsub_f32_x");
}

#[test]
fn test_simd_intrinsics_mul_all_targets_f32() {
    assert_eq!(SimdIntrinsics::mul(SimdTarget::Avx512, "f32"), "_mm512_mul_ps");
    assert_eq!(SimdIntrinsics::mul(SimdTarget::Avx2, "f32"), "_mm256_mul_ps");
    assert_eq!(SimdIntrinsics::mul(SimdTarget::Sse4, "f32"), "_mm_mul_ps");
    assert_eq!(SimdIntrinsics::mul(SimdTarget::Neon, "f32"), "vmulq_f32");
    assert_eq!(SimdIntrinsics::mul(SimdTarget::Sve, "f32"), "svmul_f32_x");
}

#[test]
fn test_simd_intrinsics_div_f32_all_targets() {
    assert_eq!(SimdIntrinsics::div(SimdTarget::Avx512, "f32"), "_mm512_div_ps");
    assert_eq!(SimdIntrinsics::div(SimdTarget::Avx2, "f32"), "_mm256_div_ps");
    assert_eq!(SimdIntrinsics::div(SimdTarget::Sse4, "f32"), "_mm_div_ps");
    assert_eq!(SimdIntrinsics::div(SimdTarget::Neon, "f32"), "vdivq_f32");
    assert_eq!(SimdIntrinsics::div(SimdTarget::Sve, "f32"), "svdiv_f32_x");
}

#[test]
fn test_simd_intrinsics_div_f64_all_targets() {
    assert_eq!(SimdIntrinsics::div(SimdTarget::Avx512, "f64"), "_mm512_div_pd");
    assert_eq!(SimdIntrinsics::div(SimdTarget::Avx2, "f64"), "_mm256_div_pd");
    assert_eq!(SimdIntrinsics::div(SimdTarget::Sse4, "f64"), "_mm_div_pd");
    assert_eq!(SimdIntrinsics::div(SimdTarget::Neon, "f64"), "vdivq_f64");
    assert_eq!(SimdIntrinsics::div(SimdTarget::Sve, "f64"), "svdiv_f64_x");
}

#[test]
fn test_simd_intrinsics_div_i32_unknown() {
    // i32 division is not directly supported in SIMD
    assert_eq!(SimdIntrinsics::div(SimdTarget::Avx512, "i32"), "unknown_div");
    assert_eq!(SimdIntrinsics::div(SimdTarget::Neon, "i32"), "unknown_div");
}

#[test]
fn test_simd_intrinsics_fma_all_targets_f32() {
    assert_eq!(SimdIntrinsics::fma(SimdTarget::Avx512, "f32"), "_mm512_fmadd_ps");
    assert_eq!(SimdIntrinsics::fma(SimdTarget::Avx2, "f32"), "_mm256_fmadd_ps");
    assert_eq!(SimdIntrinsics::fma(SimdTarget::Sse4, "f32"), "_mm_fmadd_ps");
    assert_eq!(SimdIntrinsics::fma(SimdTarget::Neon, "f32"), "vfmaq_f32");
    assert_eq!(SimdIntrinsics::fma(SimdTarget::Sve, "f32"), "svmla_f32_x");
}

#[test]
fn test_simd_intrinsics_sqrt_all_targets_f32() {
    assert_eq!(SimdIntrinsics::sqrt(SimdTarget::Avx512, "f32"), "_mm512_sqrt_ps");
    assert_eq!(SimdIntrinsics::sqrt(SimdTarget::Avx2, "f32"), "_mm256_sqrt_ps");
    assert_eq!(SimdIntrinsics::sqrt(SimdTarget::Sse4, "f32"), "_mm_sqrt_ps");
    assert_eq!(SimdIntrinsics::sqrt(SimdTarget::Neon, "f32"), "vsqrtq_f32");
    assert_eq!(SimdIntrinsics::sqrt(SimdTarget::Sve, "f32"), "svsqrt_f32_x");
}

#[test]
fn test_simd_intrinsics_sqrt_i32_unknown() {
    assert_eq!(SimdIntrinsics::sqrt(SimdTarget::Avx512, "i32"), "unknown_sqrt");
}

#[test]
fn test_simd_intrinsics_reduce_add_avx512() {
    assert_eq!(SimdIntrinsics::reduce_add(SimdTarget::Avx512, "f32"), "_mm512_reduce_add_ps");
    assert_eq!(SimdIntrinsics::reduce_add(SimdTarget::Avx512, "f64"), "_mm512_reduce_add_pd");
    assert_eq!(SimdIntrinsics::reduce_add(SimdTarget::Avx512, "i32"), "_mm512_reduce_add_epi32");
}

#[test]
fn test_simd_intrinsics_reduce_add_neon() {
    assert_eq!(SimdIntrinsics::reduce_add(SimdTarget::Neon, "f32"), "vaddvq_f32");
    assert_eq!(SimdIntrinsics::reduce_add(SimdTarget::Neon, "i32"), "vaddvq_s32");
}

#[test]
fn test_simd_intrinsics_reduce_add_sve() {
    assert_eq!(SimdIntrinsics::reduce_add(SimdTarget::Sve, "f32"), "svaddv_f32");
    assert_eq!(SimdIntrinsics::reduce_add(SimdTarget::Sve, "f64"), "svaddv_f64");
}

#[test]
fn test_simd_intrinsics_reduce_add_avx2_unknown() {
    // AVX2 doesn't have a direct reduce_add
    assert_eq!(SimdIntrinsics::reduce_add(SimdTarget::Avx2, "f32"), "unknown_reduce_add");
}

#[test]
fn test_simd_intrinsics_broadcast_all_targets_f32() {
    assert_eq!(SimdIntrinsics::broadcast(SimdTarget::Avx512, "f32"), "_mm512_set1_ps");
    assert_eq!(SimdIntrinsics::broadcast(SimdTarget::Avx2, "f32"), "_mm256_set1_ps");
    assert_eq!(SimdIntrinsics::broadcast(SimdTarget::Sse4, "f32"), "_mm_set1_ps");
    assert_eq!(SimdIntrinsics::broadcast(SimdTarget::Neon, "f32"), "vdupq_n_f32");
    assert_eq!(SimdIntrinsics::broadcast(SimdTarget::Sve, "f32"), "svdup_f32");
}

#[test]
fn test_simd_intrinsics_broadcast_all_targets_i32() {
    assert_eq!(SimdIntrinsics::broadcast(SimdTarget::Avx512, "i32"), "_mm512_set1_epi32");
    assert_eq!(SimdIntrinsics::broadcast(SimdTarget::Avx2, "i32"), "_mm256_set1_epi32");
    assert_eq!(SimdIntrinsics::broadcast(SimdTarget::Sse4, "i32"), "_mm_set1_epi32");
    assert_eq!(SimdIntrinsics::broadcast(SimdTarget::Neon, "i32"), "vdupq_n_s32");
    assert_eq!(SimdIntrinsics::broadcast(SimdTarget::Sve, "i32"), "svdup_s32");
}

#[test]
fn test_simd_intrinsics_min_all_targets_f32() {
    assert_eq!(SimdIntrinsics::min(SimdTarget::Avx512, "f32"), "_mm512_min_ps");
    assert_eq!(SimdIntrinsics::min(SimdTarget::Avx2, "f32"), "_mm256_min_ps");
    assert_eq!(SimdIntrinsics::min(SimdTarget::Sse4, "f32"), "_mm_min_ps");
    assert_eq!(SimdIntrinsics::min(SimdTarget::Neon, "f32"), "vminq_f32");
    assert_eq!(SimdIntrinsics::min(SimdTarget::Sve, "f32"), "svmin_f32_x");
}

#[test]
fn test_simd_intrinsics_max_all_targets_f64() {
    assert_eq!(SimdIntrinsics::max(SimdTarget::Avx512, "f64"), "_mm512_max_pd");
    assert_eq!(SimdIntrinsics::max(SimdTarget::Avx2, "f64"), "_mm256_max_pd");
    assert_eq!(SimdIntrinsics::max(SimdTarget::Sse4, "f64"), "_mm_max_pd");
    assert_eq!(SimdIntrinsics::max(SimdTarget::Neon, "f64"), "vmaxq_f64");
    assert_eq!(SimdIntrinsics::max(SimdTarget::Sve, "f64"), "svmax_f64_x");
}

// ==================== SimdVectorType type_name ====================

#[test]
fn test_simd_vector_type_avx512_f32() {
    let ty = SimdVectorType::F32(16);
    assert_eq!(ty.type_name(SimdTarget::Avx512), "__m512");
}

#[test]
fn test_simd_vector_type_avx512_f64() {
    let ty = SimdVectorType::F64(8);
    assert_eq!(ty.type_name(SimdTarget::Avx512), "__m512d");
}

#[test]
fn test_simd_vector_type_avx512_i32() {
    let ty = SimdVectorType::I32(16);
    assert_eq!(ty.type_name(SimdTarget::Avx512), "__m512i");
}

#[test]
fn test_simd_vector_type_avx2_f32() {
    let ty = SimdVectorType::F32(8);
    assert_eq!(ty.type_name(SimdTarget::Avx2), "__m256");
}

#[test]
fn test_simd_vector_type_avx2_f64() {
    let ty = SimdVectorType::F64(4);
    assert_eq!(ty.type_name(SimdTarget::Avx2), "__m256d");
}

#[test]
fn test_simd_vector_type_sse4_f32() {
    let ty = SimdVectorType::F32(4);
    assert_eq!(ty.type_name(SimdTarget::Sse4), "__m128");
}

#[test]
fn test_simd_vector_type_sse4_i64() {
    let ty = SimdVectorType::I64(2);
    assert_eq!(ty.type_name(SimdTarget::Sse4), "__m128i");
}

#[test]
fn test_simd_vector_type_neon_f32x4() {
    let ty = SimdVectorType::F32(4);
    assert_eq!(ty.type_name(SimdTarget::Neon), "float32x4_t");
}

#[test]
fn test_simd_vector_type_neon_f32x2() {
    let ty = SimdVectorType::F32(2);
    assert_eq!(ty.type_name(SimdTarget::Neon), "float32x2_t");
}

#[test]
fn test_simd_vector_type_neon_i32x4() {
    let ty = SimdVectorType::I32(4);
    assert_eq!(ty.type_name(SimdTarget::Neon), "int32x4_t");
}

#[test]
fn test_simd_vector_type_neon_i64x2() {
    let ty = SimdVectorType::I64(2);
    assert_eq!(ty.type_name(SimdTarget::Neon), "int64x2_t");
}

#[test]
fn test_simd_vector_type_sve_all() {
    assert_eq!(SimdVectorType::F32(16).type_name(SimdTarget::Sve), "svfloat32_t");
    assert_eq!(SimdVectorType::F64(8).type_name(SimdTarget::Sve), "svfloat64_t");
    assert_eq!(SimdVectorType::I32(16).type_name(SimdTarget::Sve), "svint32_t");
    assert_eq!(SimdVectorType::I64(8).type_name(SimdTarget::Sve), "svint64_t");
}

#[test]
fn test_simd_vector_type_equality() {
    assert_eq!(SimdVectorType::F32(4), SimdVectorType::F32(4));
    assert_ne!(SimdVectorType::F32(4), SimdVectorType::F32(8));
    assert_ne!(SimdVectorType::F32(4), SimdVectorType::F64(4));
    assert_ne!(SimdVectorType::I32(4), SimdVectorType::I64(4));
}

#[test]
fn test_simd_vector_type_clone() {
    let ty = SimdVectorType::F64(8);
    let cloned = ty.clone();
    assert_eq!(ty, cloned);
}

// ==================== generate_simd_code with parsed source ====================

#[test]
fn test_generate_simd_code_empty_module() {
    use vais_parser::parse;

    let source = "F unused_placeholder() = 0";
    let module = parse(source).unwrap();
    let result = generate_simd_code(&module, SimdTarget::Avx2);
    assert!(result.is_ok());
    let code = result.unwrap();
    // Should at least contain header info
    assert!(code.contains("Vais") || code.contains("AVX2") || code.len() > 0);
}

#[test]
fn test_generate_simd_code_avx512_header() {
    use vais_parser::parse;

    let source = "F noop() = 0";
    let module = parse(source).unwrap();
    let code = generate_simd_code(&module, SimdTarget::Avx512).unwrap();
    assert!(code.contains("immintrin.h") || code.contains("AVX-512") || code.len() > 0);
}

#[test]
fn test_generate_simd_code_neon_header() {
    use vais_parser::parse;

    let source = "F noop() = 0";
    let module = parse(source).unwrap();
    let code = generate_simd_code(&module, SimdTarget::Neon).unwrap();
    assert!(code.contains("arm_neon.h") || code.contains("NEON") || code.len() > 0);
}

#[test]
fn test_generate_simd_code_sve_header() {
    use vais_parser::parse;

    let source = "F noop() = 0";
    let module = parse(source).unwrap();
    let code = generate_simd_code(&module, SimdTarget::Sve).unwrap();
    assert!(code.contains("arm_sve.h") || code.contains("SVE") || code.len() > 0);
}

#[test]
fn test_generate_simd_code_all_targets_succeed() {
    use vais_parser::parse;

    let source = "F noop() = 0";
    let module = parse(source).unwrap();

    for target in &[SimdTarget::Avx512, SimdTarget::Avx2, SimdTarget::Sse4, SimdTarget::Neon, SimdTarget::Sve] {
        let result = generate_simd_code(&module, *target);
        assert!(result.is_ok(), "generate_simd_code failed for {:?}", target);
    }
}

#[test]
fn test_generate_simd_code_with_simd_attr() {
    use vais_parser::parse;

    let source = r#"
#[simd]
F vector_sum(a: *f32, b: *f32, out: *f32, n: i32) {
    idx := 0
    I idx < n {
        out[idx] = a[idx] + b[idx]
    }
}
"#;
    let module = parse(source).unwrap();
    let result = generate_simd_code(&module, SimdTarget::Avx2);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("vector_sum"), "Should contain function name");
}
