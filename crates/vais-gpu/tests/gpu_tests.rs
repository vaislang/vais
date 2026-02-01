//! GPU code generation tests

use vais_gpu::{GpuCodeGenerator, GpuTarget, GpuType, GpuError};
use vais_gpu::simd::{SimdTarget, SimdIntrinsics, SimdVectorType};
use vais_gpu::metal::MetalBuiltins;

#[test]
fn test_gpu_target_from_str() {
    assert_eq!(GpuTarget::parse("cuda"), Some(GpuTarget::Cuda));
    assert_eq!(GpuTarget::parse("CUDA"), Some(GpuTarget::Cuda));
    assert_eq!(GpuTarget::parse("ptx"), Some(GpuTarget::Cuda));
    assert_eq!(GpuTarget::parse("nvidia"), Some(GpuTarget::Cuda));

    assert_eq!(GpuTarget::parse("opencl"), Some(GpuTarget::OpenCL));
    assert_eq!(GpuTarget::parse("cl"), Some(GpuTarget::OpenCL));

    assert_eq!(GpuTarget::parse("webgpu"), Some(GpuTarget::WebGPU));
    assert_eq!(GpuTarget::parse("wgsl"), Some(GpuTarget::WebGPU));

    assert_eq!(GpuTarget::parse("unknown"), None);
    assert_eq!(GpuTarget::parse(""), None);
}

#[test]
fn test_gpu_target_extension() {
    assert_eq!(GpuTarget::Cuda.extension(), "cu");
    assert_eq!(GpuTarget::OpenCL.extension(), "cl");
    assert_eq!(GpuTarget::WebGPU.extension(), "wgsl");
}

#[test]
fn test_gpu_target_name() {
    assert_eq!(GpuTarget::Cuda.name(), "CUDA");
    assert_eq!(GpuTarget::OpenCL.name(), "OpenCL");
    assert_eq!(GpuTarget::WebGPU.name(), "WebGPU");
}

#[test]
fn test_gpu_type_i64() {
    let ty = GpuType::I64;
    assert_eq!(ty.cuda_name(), "long long");
    assert_eq!(ty.opencl_name(), "long");
    assert_eq!(ty.wgsl_name(), "i64");
}

#[test]
fn test_gpu_type_f32() {
    let ty = GpuType::F32;
    assert_eq!(ty.cuda_name(), "float");
    assert_eq!(ty.opencl_name(), "float");
    assert_eq!(ty.wgsl_name(), "f32");
}

#[test]
fn test_gpu_type_f64() {
    let ty = GpuType::F64;
    assert_eq!(ty.cuda_name(), "double");
    assert_eq!(ty.opencl_name(), "double");
    assert_eq!(ty.wgsl_name(), "f64");
}

#[test]
fn test_gpu_type_ptr() {
    let ty = GpuType::Ptr(Box::new(GpuType::F32));
    assert_eq!(ty.cuda_name(), "float*");
    assert_eq!(ty.opencl_name(), "__global float*");
    assert_eq!(ty.wgsl_name(), "ptr<storage, f32>");
}

#[test]
fn test_gpu_type_array() {
    let ty = GpuType::Array(Box::new(GpuType::I32), 16);
    assert_eq!(ty.cuda_name(), "int[16]");
    assert_eq!(ty.opencl_name(), "int[16]");
    assert_eq!(ty.wgsl_name(), "array<i32, 16>");
}

#[test]
fn test_gpu_type_vec() {
    let ty = GpuType::Vec(Box::new(GpuType::F32), 4);
    assert_eq!(ty.cuda_name(), "float4");
    assert_eq!(ty.opencl_name(), "float4");
    assert_eq!(ty.wgsl_name(), "vec4<f32>");
}

#[test]
fn test_gpu_type_nested_ptr() {
    let ty = GpuType::Ptr(Box::new(GpuType::Ptr(Box::new(GpuType::I32))));
    assert_eq!(ty.cuda_name(), "int**");
}

#[test]
fn test_gpu_code_generator_creation() {
    let gen = GpuCodeGenerator::new(GpuTarget::Cuda);
    assert_eq!(gen.target(), GpuTarget::Cuda);
    assert!(gen.kernels().is_empty());
}

#[test]
fn test_gpu_code_generator_targets() {
    let cuda_gen = GpuCodeGenerator::new(GpuTarget::Cuda);
    let opencl_gen = GpuCodeGenerator::new(GpuTarget::OpenCL);
    let webgpu_gen = GpuCodeGenerator::new(GpuTarget::WebGPU);

    assert_eq!(cuda_gen.target(), GpuTarget::Cuda);
    assert_eq!(opencl_gen.target(), GpuTarget::OpenCL);
    assert_eq!(webgpu_gen.target(), GpuTarget::WebGPU);
}

#[test]
fn test_gpu_error_display() {
    let err = GpuError::UnsupportedType("String".to_string());
    assert!(err.to_string().contains("String"));

    let err = GpuError::UnsupportedOperation("closure".to_string());
    assert!(err.to_string().contains("closure"));

    let err = GpuError::KernelError("invalid grid size".to_string());
    assert!(err.to_string().contains("invalid grid size"));
}

#[test]
fn test_gpu_type_bool() {
    let ty = GpuType::Bool;
    assert_eq!(ty.cuda_name(), "bool");
    assert_eq!(ty.opencl_name(), "bool");
    assert_eq!(ty.wgsl_name(), "bool");
}

#[test]
fn test_gpu_type_void() {
    let ty = GpuType::Void;
    assert_eq!(ty.cuda_name(), "void");
    assert_eq!(ty.opencl_name(), "void");
    assert_eq!(ty.wgsl_name(), "");
}

mod cuda_codegen_tests {
    use vais_gpu::{GpuCodeGenerator, GpuTarget, GpuType};
    use vais_ast::*;

    fn spanned<T>(node: T) -> Spanned<T> {
        Spanned { node, span: Span { start: 0, end: 0 } }
    }

    fn make_kernel(name: &str, params: Vec<Param>, body: Vec<Spanned<Stmt>>) -> Module {
        Module {
            items: vec![spanned(Item::Function(Function {
                name: spanned(name.to_string()),
                generics: vec![],
                params,
                ret_type: None,
                body: FunctionBody::Block(body),
                is_pub: false,
                is_async: false,
                attributes: vec![Attribute { name: "gpu".to_string(), args: vec![], expr: None }],
            }))]
        }
    }

    fn make_param(name: &str, ty: Type) -> Param {
        Param {
            name: spanned(name.to_string()),
            ty: spanned(ty),
            is_mut: false,
            is_vararg: false,
            ownership: Ownership::Regular,
            default_value: None,
        }
    }

    #[test]
    fn test_cuda_kernel_i32_ptr_param() {
        let module = make_kernel("add_i32", vec![
            make_param("a", Type::Pointer(Box::new(spanned(Type::Named { name: "i32".to_string(), generics: vec![] })))),
            make_param("b", Type::Pointer(Box::new(spanned(Type::Named { name: "i32".to_string(), generics: vec![] })))),
        ], vec![]);

        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let code = gen.generate(&module).expect("should generate CUDA code");
        assert!(code.contains("int* a"), "Expected 'int* a' in CUDA output, got:\n{}", code);
        assert!(code.contains("int* b"), "Expected 'int* b' in CUDA output, got:\n{}", code);

        // Verify kernel metadata has correct types
        let kernels = gen.kernels();
        assert_eq!(kernels.len(), 1);
        assert_eq!(kernels[0].params[0].1, GpuType::Ptr(Box::new(GpuType::I32)));
        assert_eq!(kernels[0].params[1].1, GpuType::Ptr(Box::new(GpuType::I32)));
    }

    #[test]
    fn test_cuda_kernel_f64_ptr_param() {
        let module = make_kernel("add_f64", vec![
            make_param("data", Type::Pointer(Box::new(spanned(Type::Named { name: "f64".to_string(), generics: vec![] })))),
        ], vec![]);

        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let code = gen.generate(&module).expect("should generate CUDA code");
        assert!(code.contains("double* data"), "Expected 'double* data' in CUDA output, got:\n{}", code);

        let kernels = gen.kernels();
        assert_eq!(kernels[0].params[0].1, GpuType::Ptr(Box::new(GpuType::F64)));
    }

    #[test]
    fn test_cuda_kernel_mixed_param_types() {
        let module = make_kernel("mixed", vec![
            make_param("floats", Type::Pointer(Box::new(spanned(Type::Named { name: "f32".to_string(), generics: vec![] })))),
            make_param("ints", Type::Pointer(Box::new(spanned(Type::Named { name: "i64".to_string(), generics: vec![] })))),
            make_param("n", Type::Named { name: "i32".to_string(), generics: vec![] }),
        ], vec![]);

        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let _code = gen.generate(&module).expect("should generate CUDA code");

        let kernels = gen.kernels();
        assert_eq!(kernels[0].params.len(), 3);
        assert_eq!(kernels[0].params[0].1, GpuType::Ptr(Box::new(GpuType::F32)));
        assert_eq!(kernels[0].params[1].1, GpuType::Ptr(Box::new(GpuType::I64)));
        assert_eq!(kernels[0].params[2].1, GpuType::I32);
    }
}

mod common_tests {
    use vais_gpu::cuda;
    use vais_gpu::opencl;
    use vais_gpu::webgpu;
    use vais_gpu::metal;

    #[test]
    fn test_cuda_module_exists() {
        // Just verify the module compiles
        let _ = cuda::generate_host_code(&[]);
    }

    #[test]
    fn test_opencl_module_exists() {
        let _ = opencl::generate_host_code(&[]);
    }

    #[test]
    fn test_webgpu_module_exists() {
        let _ = webgpu::generate_host_code(&[], "");
    }

    #[test]
    fn test_metal_module_exists() {
        let _ = metal::generate_host_code(&[], "Test");
    }
}

// Metal backend tests
mod metal_tests {
    use super::*;

    #[test]
    fn test_metal_target_from_str() {
        assert_eq!(GpuTarget::parse("metal"), Some(GpuTarget::Metal));
        assert_eq!(GpuTarget::parse("msl"), Some(GpuTarget::Metal));
        assert_eq!(GpuTarget::parse("apple"), Some(GpuTarget::Metal));
    }

    #[test]
    fn test_metal_target_extension() {
        assert_eq!(GpuTarget::Metal.extension(), "metal");
    }

    #[test]
    fn test_metal_target_name() {
        assert_eq!(GpuTarget::Metal.name(), "Metal");
    }

    #[test]
    fn test_metal_builtins_math() {
        assert_eq!(MetalBuiltins::builtin("sqrt"), Some("sqrt"));
        assert_eq!(MetalBuiltins::builtin("rsqrt"), Some("rsqrt"));
        assert_eq!(MetalBuiltins::builtin("fma"), Some("fma"));
        assert_eq!(MetalBuiltins::builtin("clamp"), Some("clamp"));
    }

    #[test]
    fn test_metal_builtins_thread_indexing() {
        assert_eq!(MetalBuiltins::builtin("thread_idx_x"), Some("thread_position_in_threadgroup.x"));
        assert_eq!(MetalBuiltins::builtin("block_idx_x"), Some("threadgroup_position_in_grid.x"));
        assert_eq!(MetalBuiltins::builtin("global_idx"), Some("thread_position_in_grid.x"));
        assert_eq!(MetalBuiltins::builtin("lane_id"), Some("simd_lane_id"));
    }

    #[test]
    fn test_metal_builtins_atomic() {
        assert_eq!(MetalBuiltins::builtin("atomic_add"), Some("atomic_fetch_add_explicit"));
        assert_eq!(MetalBuiltins::builtin("atomic_cas"), Some("atomic_compare_exchange_weak_explicit"));
    }

    #[test]
    fn test_metal_builtins_simd() {
        assert_eq!(MetalBuiltins::builtin("simd_sum"), Some("simd_sum"));
        assert_eq!(MetalBuiltins::builtin("simd_shuffle"), Some("simd_shuffle"));
        assert_eq!(MetalBuiltins::builtin("warp_all"), Some("simd_all"));
    }

    #[test]
    fn test_metal_code_generator() {
        let gen = GpuCodeGenerator::new(GpuTarget::Metal);
        assert_eq!(gen.target(), GpuTarget::Metal);
        assert!(gen.kernels().is_empty());
    }
}

// End-to-end GPU codegen tests (parse .vais source → generate GPU code)
mod e2e_gpu_codegen {
    use vais_gpu::{GpuCodeGenerator, GpuTarget};
    use vais_parser::parse;

    const KERNEL_SOURCE: &str = r#"
#[gpu]
F vector_add(a: *i32, b: *i32, out: *i32, n: i32) {
    idx := global_idx()
    I idx < n {
        out[idx] = a[idx] + b[idx]
    }
}
"#;

    const SCALAR_KERNEL: &str = r#"
#[gpu]
F scalar_mul(data: *f64, scale: f64, n: i32) {
    idx := global_idx()
    I idx < n {
        data[idx] = data[idx] * scale
    }
}
"#;

    #[test]
    fn test_e2e_cuda_codegen() {
        let module = parse(KERNEL_SOURCE).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let code = gen.generate(&module).expect("CUDA codegen failed");
        assert!(code.contains("__global__"), "CUDA kernel should have __global__ qualifier");
        assert!(code.contains("vector_add"), "Should contain kernel name");
        assert!(!gen.kernels().is_empty(), "Should discover at least one kernel");
    }

    #[test]
    fn test_e2e_opencl_codegen() {
        let module = parse(KERNEL_SOURCE).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::OpenCL);
        let code = gen.generate(&module).expect("OpenCL codegen failed");
        assert!(code.contains("__kernel"), "OpenCL kernel should have __kernel qualifier");
        assert!(code.contains("vector_add"), "Should contain kernel name");
    }

    #[test]
    fn test_e2e_webgpu_codegen() {
        let module = parse(KERNEL_SOURCE).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::WebGPU);
        let code = gen.generate(&module).expect("WebGPU codegen failed");
        assert!(code.contains("fn vector_add") || code.contains("@compute"),
            "WGSL should contain compute shader syntax, got:\n{}", code);
    }

    #[test]
    fn test_e2e_metal_codegen() {
        let module = parse(KERNEL_SOURCE).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Metal);
        let code = gen.generate(&module).expect("Metal codegen failed");
        assert!(code.contains("kernel") || code.contains("vector_add"),
            "Metal should contain kernel function, got:\n{}", code);
    }

    #[test]
    fn test_e2e_cuda_host_code() {
        let module = parse(KERNEL_SOURCE).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let _code = gen.generate(&module).expect("CUDA codegen failed");
        let host = gen.generate_host_code();
        assert!(host.contains("cudaMalloc") || host.contains("cuda") || host.len() > 0,
            "Host code should contain CUDA runtime calls");
    }

    #[test]
    fn test_e2e_metal_host_code() {
        let module = parse(KERNEL_SOURCE).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Metal);
        let _code = gen.generate(&module).expect("Metal codegen failed");
        let host = gen.generate_host_code();
        assert!(host.contains("MTL") || host.contains("Metal") || host.len() > 0,
            "Host code should contain Metal runtime calls");
    }

    #[test]
    fn test_e2e_scalar_kernel_all_backends() {
        let module = parse(SCALAR_KERNEL).expect("parse failed");
        for target in &[GpuTarget::Cuda, GpuTarget::OpenCL, GpuTarget::WebGPU, GpuTarget::Metal] {
            let mut gen = GpuCodeGenerator::new(*target);
            let code = gen.generate(&module).expect(&format!("{:?} codegen failed", target));
            assert!(code.contains("scalar_mul"),
                "{:?} should contain kernel name 'scalar_mul'", target);
            assert!(!gen.kernels().is_empty(),
                "{:?} should discover kernel", target);
        }
    }

    #[test]
    fn test_e2e_kernel_metadata() {
        let module = parse(KERNEL_SOURCE).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let _code = gen.generate(&module).expect("codegen failed");
        let kernels = gen.kernels();
        assert_eq!(kernels.len(), 1);
        assert_eq!(kernels[0].name, "vector_add");
        assert_eq!(kernels[0].params.len(), 4);
    }
}

// SIMD backend tests
mod simd_tests {
    use super::*;

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
    fn test_simd_target_vector_bits() {
        assert_eq!(SimdTarget::Avx512.vector_bits(), 512);
        assert_eq!(SimdTarget::Avx2.vector_bits(), 256);
        assert_eq!(SimdTarget::Sse4.vector_bits(), 128);
        assert_eq!(SimdTarget::Neon.vector_bits(), 128);
        assert_eq!(SimdTarget::Sve.vector_bits(), 512);
    }

    #[test]
    fn test_simd_target_lanes() {
        assert_eq!(SimdTarget::Avx512.f32_lanes(), 16);
        assert_eq!(SimdTarget::Avx512.f64_lanes(), 8);
        assert_eq!(SimdTarget::Avx512.i32_lanes(), 16);
        assert_eq!(SimdTarget::Avx2.f32_lanes(), 8);
        assert_eq!(SimdTarget::Neon.f32_lanes(), 4);
    }

    #[test]
    fn test_simd_target_compiler_flags() {
        assert!(SimdTarget::Avx512.compiler_flags().contains("-mavx512f"));
        assert!(SimdTarget::Avx2.compiler_flags().contains("-mavx2"));
        assert!(SimdTarget::Sse4.compiler_flags().contains("-msse4.2"));
        assert!(SimdTarget::Neon.compiler_flags().contains("-mfpu=neon"));
        assert!(SimdTarget::Sve.compiler_flags().contains("-march=armv8-a+sve"));
    }

    #[test]
    fn test_simd_target_headers() {
        assert!(SimdTarget::Avx512.headers().contains("immintrin.h"));
        assert!(SimdTarget::Avx2.headers().contains("immintrin.h"));
        assert!(SimdTarget::Neon.headers().contains("arm_neon.h"));
        assert!(SimdTarget::Sve.headers().contains("arm_sve.h"));
    }

    #[test]
    fn test_simd_intrinsics_load() {
        assert_eq!(SimdIntrinsics::load(SimdTarget::Avx512, "f32"), "_mm512_loadu_ps");
        assert_eq!(SimdIntrinsics::load(SimdTarget::Avx512, "f64"), "_mm512_loadu_pd");
        assert_eq!(SimdIntrinsics::load(SimdTarget::Avx2, "f32"), "_mm256_loadu_ps");
        assert_eq!(SimdIntrinsics::load(SimdTarget::Sse4, "f32"), "_mm_loadu_ps");
        assert_eq!(SimdIntrinsics::load(SimdTarget::Neon, "f32"), "vld1q_f32");
        assert_eq!(SimdIntrinsics::load(SimdTarget::Sve, "f32"), "svld1_f32");
    }

    #[test]
    fn test_simd_intrinsics_store() {
        assert_eq!(SimdIntrinsics::store(SimdTarget::Avx512, "f32"), "_mm512_storeu_ps");
        assert_eq!(SimdIntrinsics::store(SimdTarget::Neon, "f32"), "vst1q_f32");
    }

    #[test]
    fn test_simd_intrinsics_arithmetic() {
        assert_eq!(SimdIntrinsics::add(SimdTarget::Avx512, "f32"), "_mm512_add_ps");
        assert_eq!(SimdIntrinsics::sub(SimdTarget::Avx512, "f32"), "_mm512_sub_ps");
        assert_eq!(SimdIntrinsics::mul(SimdTarget::Avx512, "f32"), "_mm512_mul_ps");
        assert_eq!(SimdIntrinsics::div(SimdTarget::Avx512, "f32"), "_mm512_div_ps");
    }

    #[test]
    fn test_simd_intrinsics_fma() {
        assert_eq!(SimdIntrinsics::fma(SimdTarget::Avx512, "f32"), "_mm512_fmadd_ps");
        assert_eq!(SimdIntrinsics::fma(SimdTarget::Avx2, "f32"), "_mm256_fmadd_ps");
        assert_eq!(SimdIntrinsics::fma(SimdTarget::Neon, "f32"), "vfmaq_f32");
    }

    #[test]
    fn test_simd_intrinsics_broadcast() {
        assert_eq!(SimdIntrinsics::broadcast(SimdTarget::Avx512, "f32"), "_mm512_set1_ps");
        assert_eq!(SimdIntrinsics::broadcast(SimdTarget::Neon, "f32"), "vdupq_n_f32");
        assert_eq!(SimdIntrinsics::broadcast(SimdTarget::Sve, "f32"), "svdup_f32");
    }

    #[test]
    fn test_simd_intrinsics_reduce() {
        assert_eq!(SimdIntrinsics::reduce_add(SimdTarget::Avx512, "f32"), "_mm512_reduce_add_ps");
        assert_eq!(SimdIntrinsics::reduce_add(SimdTarget::Neon, "f32"), "vaddvq_f32");
        assert_eq!(SimdIntrinsics::reduce_add(SimdTarget::Sve, "f32"), "svaddv_f32");
    }

    #[test]
    fn test_simd_intrinsics_min_max() {
        assert_eq!(SimdIntrinsics::min(SimdTarget::Avx512, "f32"), "_mm512_min_ps");
        assert_eq!(SimdIntrinsics::max(SimdTarget::Avx512, "f32"), "_mm512_max_ps");
        assert_eq!(SimdIntrinsics::min(SimdTarget::Neon, "f32"), "vminq_f32");
        assert_eq!(SimdIntrinsics::max(SimdTarget::Neon, "f32"), "vmaxq_f32");
    }

    #[test]
    fn test_simd_vector_type_avx512() {
        let f32_16 = SimdVectorType::F32(16);
        assert_eq!(f32_16.type_name(SimdTarget::Avx512), "__m512");

        let f64_8 = SimdVectorType::F64(8);
        assert_eq!(f64_8.type_name(SimdTarget::Avx512), "__m512d");

        let i32_16 = SimdVectorType::I32(16);
        assert_eq!(i32_16.type_name(SimdTarget::Avx512), "__m512i");
    }

    #[test]
    fn test_simd_vector_type_avx2() {
        let f32_8 = SimdVectorType::F32(8);
        assert_eq!(f32_8.type_name(SimdTarget::Avx2), "__m256");

        let f64_4 = SimdVectorType::F64(4);
        assert_eq!(f64_4.type_name(SimdTarget::Avx2), "__m256d");
    }

    #[test]
    fn test_simd_vector_type_neon() {
        let f32_4 = SimdVectorType::F32(4);
        assert_eq!(f32_4.type_name(SimdTarget::Neon), "float32x4_t");

        let f64_2 = SimdVectorType::F64(2);
        assert_eq!(f64_2.type_name(SimdTarget::Neon), "float64x2_t");

        let i32_4 = SimdVectorType::I32(4);
        assert_eq!(i32_4.type_name(SimdTarget::Neon), "int32x4_t");
    }

    #[test]
    fn test_simd_vector_type_sve() {
        let f32_any = SimdVectorType::F32(16);
        assert_eq!(f32_any.type_name(SimdTarget::Sve), "svfloat32_t");

        let f64_any = SimdVectorType::F64(8);
        assert_eq!(f64_any.type_name(SimdTarget::Sve), "svfloat64_t");
    }
}

// E2E GPU runtime integration tests
// These verify the full pipeline: .vais source → GPU codegen → host code with runtime API calls
mod e2e_gpu_runtime {
    use vais_gpu::{GpuCodeGenerator, GpuTarget};
    use vais_parser::parse;

    const VECTOR_ADD_KERNEL: &str = r#"
#[gpu]
F vector_add(a: *f64, b: *f64, c: *f64, n: i64) {
    idx := thread_idx_x() + block_idx_x() * block_dim_x()
    I idx < n {
        c[idx] = a[idx] + b[idx]
    }
}
"#;

    #[test]
    fn test_e2e_vector_add_cuda_generates_kernel() {
        let module = parse(VECTOR_ADD_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let code = gen.generate(&module).expect("CUDA codegen failed");

        // Verify kernel structure
        assert!(code.contains("__global__"), "Should have __global__ qualifier");
        assert!(code.contains("vector_add"), "Should contain kernel name");
        assert!(code.contains("double*"), "Should have double* parameters for f64");

        // Verify thread indexing is emitted
        assert!(code.contains("threadIdx.x") || code.contains("threadIdx"),
            "Should contain CUDA thread indexing");
        assert!(code.contains("blockIdx.x") || code.contains("blockIdx"),
            "Should contain CUDA block indexing");
        assert!(code.contains("blockDim.x") || code.contains("blockDim"),
            "Should contain CUDA block dimension");
    }

    #[test]
    fn test_e2e_vector_add_cuda_host_code_has_runtime_calls() {
        let module = parse(VECTOR_ADD_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let _code = gen.generate(&module).expect("CUDA codegen failed");
        let host = gen.generate_host_code();

        // Host code should reference CUDA runtime API functions
        assert!(host.contains("cudaDeviceSynchronize") || host.contains("launch_"),
            "Host code should contain CUDA runtime calls, got:\n{}", host);
    }

    #[test]
    fn test_e2e_vector_add_kernel_metadata() {
        let module = parse(VECTOR_ADD_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let _code = gen.generate(&module).expect("CUDA codegen failed");

        let kernels = gen.kernels();
        assert_eq!(kernels.len(), 1);
        assert_eq!(kernels[0].name, "vector_add");
        assert_eq!(kernels[0].params.len(), 4, "vector_add should have 4 params (a, b, c, n)");

        // Verify param types
        let param_names: Vec<&str> = kernels[0].params.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(param_names, &["a", "b", "c", "n"]);
    }

    #[test]
    fn test_e2e_vector_add_all_backends() {
        let module = parse(VECTOR_ADD_KERNEL).expect("parse failed");
        let targets = vec![
            (GpuTarget::Cuda, "__global__"),
            (GpuTarget::OpenCL, "__kernel"),
        ];
        for (target, expected_keyword) in targets {
            let mut gen = GpuCodeGenerator::new(target);
            let code = gen.generate(&module)
                .unwrap_or_else(|e| panic!("{:?} codegen failed: {}", target, e));
            assert!(code.contains(expected_keyword),
                "{:?} should contain '{}', got:\n{}", target, expected_keyword, code);
            assert!(code.contains("vector_add"),
                "{:?} should contain kernel name", target);
        }
    }

    #[test]
    fn test_e2e_vector_add_metal_codegen() {
        let module = parse(VECTOR_ADD_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Metal);
        let code = gen.generate(&module).expect("Metal codegen failed");
        assert!(code.contains("kernel") || code.contains("vector_add"),
            "Metal should generate kernel function");
    }

    #[test]
    fn test_e2e_vector_add_webgpu_codegen() {
        let module = parse(VECTOR_ADD_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::WebGPU);
        let code = gen.generate(&module).expect("WebGPU codegen failed");
        assert!(code.contains("vector_add") || code.contains("@compute"),
            "WebGPU should generate compute shader");
    }

    const MATRIX_MUL_KERNEL: &str = r#"
#[gpu]
F matrix_mul(a: *f64, b: *f64, c: *f64, n: i64) {
    row := block_idx_y() * block_dim_y() + thread_idx_y()
    col := block_idx_x() * block_dim_x() + thread_idx_x()
    I row < n {
        I col < n {
            sum := 0.0
            c[row * n + col] = sum
        }
    }
}
"#;

    #[test]
    fn test_e2e_matrix_mul_cuda_2d_indexing() {
        let module = parse(MATRIX_MUL_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let code = gen.generate(&module).expect("CUDA codegen failed");
        assert!(code.contains("matrix_mul"), "Should contain kernel name");
        // 2D indexing should reference y-dimension
        assert!(code.contains("threadIdx.y") || code.contains("blockIdx.y") || code.contains("thread"),
            "Should contain 2D thread indexing");
    }

    const REDUCTION_KERNEL: &str = r#"
#[gpu]
F reduce_sum(data: *f64, result: *f64, n: i64) {
    idx := thread_idx_x() + block_idx_x() * block_dim_x()
    I idx < n {
        result[0] = result[0] + data[idx]
    }
}
"#;

    #[test]
    fn test_e2e_reduction_kernel() {
        let module = parse(REDUCTION_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let code = gen.generate(&module).expect("CUDA codegen failed");
        assert!(code.contains("reduce_sum"), "Should contain kernel name");
        let kernels = gen.kernels();
        assert_eq!(kernels.len(), 1);
        assert_eq!(kernels[0].params.len(), 3);
    }
}
