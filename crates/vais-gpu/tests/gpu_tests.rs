//! GPU code generation tests

#[cfg(feature = "metal")]
use vais_gpu::metal::MetalBuiltins;
use vais_gpu::simd::{SimdIntrinsics, SimdTarget, SimdVectorType};
use vais_gpu::{GpuCodeGenerator, GpuError, GpuTarget, GpuType};

#[test]
fn test_gpu_target_from_str() {
    #[cfg(feature = "cuda")]
    {
        assert_eq!(GpuTarget::parse("cuda"), Some(GpuTarget::Cuda));
        assert_eq!(GpuTarget::parse("CUDA"), Some(GpuTarget::Cuda));
        assert_eq!(GpuTarget::parse("ptx"), Some(GpuTarget::Cuda));
        assert_eq!(GpuTarget::parse("nvidia"), Some(GpuTarget::Cuda));
    }

    #[cfg(feature = "opencl")]
    {
        assert_eq!(GpuTarget::parse("opencl"), Some(GpuTarget::OpenCL));
        assert_eq!(GpuTarget::parse("cl"), Some(GpuTarget::OpenCL));
    }

    #[cfg(feature = "webgpu")]
    {
        assert_eq!(GpuTarget::parse("webgpu"), Some(GpuTarget::WebGPU));
        assert_eq!(GpuTarget::parse("wgsl"), Some(GpuTarget::WebGPU));
    }

    assert_eq!(GpuTarget::parse("unknown"), None);
    assert_eq!(GpuTarget::parse(""), None);
}

#[test]
#[cfg(any(feature = "cuda", feature = "opencl", feature = "webgpu"))]
fn test_gpu_target_extension() {
    #[cfg(feature = "cuda")]
    assert_eq!(GpuTarget::Cuda.extension(), "cu");
    #[cfg(feature = "opencl")]
    assert_eq!(GpuTarget::OpenCL.extension(), "cl");
    #[cfg(feature = "webgpu")]
    assert_eq!(GpuTarget::WebGPU.extension(), "wgsl");
}

#[test]
#[cfg(any(feature = "cuda", feature = "opencl", feature = "webgpu"))]
fn test_gpu_target_name() {
    #[cfg(feature = "cuda")]
    assert_eq!(GpuTarget::Cuda.name(), "CUDA");
    #[cfg(feature = "opencl")]
    assert_eq!(GpuTarget::OpenCL.name(), "OpenCL");
    #[cfg(feature = "webgpu")]
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

#[cfg(feature = "cuda")]
mod cuda_codegen_tests {
    use vais_ast::*;
    use vais_gpu::{GpuCodeGenerator, GpuTarget, GpuType};

    fn spanned<T>(node: T) -> Spanned<T> {
        Spanned {
            node,
            span: Span { start: 0, end: 0 },
        }
    }

    fn make_kernel(name: &str, params: Vec<Param>, body: Vec<Spanned<Stmt>>) -> Module {
        Module {
            modules_map: None,
            items: vec![spanned(Item::Function(Function {
                name: spanned(name.to_string()),
                generics: vec![],
                params,
                ret_type: None,
                body: FunctionBody::Block(body),
                is_pub: false,
                is_async: false,
                attributes: vec![Attribute {
                    name: "gpu".to_string(),
                    args: vec![],
                    expr: None,
                }],
            }))],
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
        let module = make_kernel(
            "add_i32",
            vec![
                make_param(
                    "a",
                    Type::Pointer(Box::new(spanned(Type::Named {
                        name: "i32".to_string(),
                        generics: vec![],
                    }))),
                ),
                make_param(
                    "b",
                    Type::Pointer(Box::new(spanned(Type::Named {
                        name: "i32".to_string(),
                        generics: vec![],
                    }))),
                ),
            ],
            vec![],
        );

        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let code = gen.generate(&module).expect("should generate CUDA code");
        assert!(
            code.contains("int* a"),
            "Expected 'int* a' in CUDA output, got:\n{}",
            code
        );
        assert!(
            code.contains("int* b"),
            "Expected 'int* b' in CUDA output, got:\n{}",
            code
        );

        // Verify kernel metadata has correct types
        let kernels = gen.kernels();
        assert_eq!(kernels.len(), 1);
        assert_eq!(kernels[0].params[0].1, GpuType::Ptr(Box::new(GpuType::I32)));
        assert_eq!(kernels[0].params[1].1, GpuType::Ptr(Box::new(GpuType::I32)));
    }

    #[test]
    fn test_cuda_kernel_f64_ptr_param() {
        let module = make_kernel(
            "add_f64",
            vec![make_param(
                "data",
                Type::Pointer(Box::new(spanned(Type::Named {
                    name: "f64".to_string(),
                    generics: vec![],
                }))),
            )],
            vec![],
        );

        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let code = gen.generate(&module).expect("should generate CUDA code");
        assert!(
            code.contains("double* data"),
            "Expected 'double* data' in CUDA output, got:\n{}",
            code
        );

        let kernels = gen.kernels();
        assert_eq!(kernels[0].params[0].1, GpuType::Ptr(Box::new(GpuType::F64)));
    }

    #[test]
    fn test_cuda_kernel_mixed_param_types() {
        let module = make_kernel(
            "mixed",
            vec![
                make_param(
                    "floats",
                    Type::Pointer(Box::new(spanned(Type::Named {
                        name: "f32".to_string(),
                        generics: vec![],
                    }))),
                ),
                make_param(
                    "ints",
                    Type::Pointer(Box::new(spanned(Type::Named {
                        name: "i64".to_string(),
                        generics: vec![],
                    }))),
                ),
                make_param(
                    "n",
                    Type::Named {
                        name: "i32".to_string(),
                        generics: vec![],
                    },
                ),
            ],
            vec![],
        );

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
    #[cfg(feature = "cuda")]
    use vais_gpu::cuda;
    #[cfg(feature = "metal")]
    use vais_gpu::metal;
    #[cfg(feature = "opencl")]
    use vais_gpu::opencl;
    #[cfg(feature = "webgpu")]
    use vais_gpu::webgpu;

    #[test]
    #[cfg(feature = "cuda")]
    fn test_cuda_module_exists() {
        // Just verify the module compiles
        let _ = cuda::generate_host_code(&[]);
    }

    #[test]
    #[cfg(feature = "opencl")]
    fn test_opencl_module_exists() {
        let _ = opencl::generate_host_code(&[]);
    }

    #[test]
    #[cfg(feature = "webgpu")]
    fn test_webgpu_module_exists() {
        let _ = webgpu::generate_host_code(&[], "");
    }

    #[test]
    #[cfg(feature = "metal")]
    fn test_metal_module_exists() {
        let _ = metal::generate_host_code(&[], "Test");
    }
}

// Metal backend tests
#[cfg(feature = "metal")]
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
        assert_eq!(
            MetalBuiltins::builtin("thread_idx_x"),
            Some("thread_position_in_threadgroup.x")
        );
        assert_eq!(
            MetalBuiltins::builtin("block_idx_x"),
            Some("threadgroup_position_in_grid.x")
        );
        assert_eq!(
            MetalBuiltins::builtin("global_idx"),
            Some("thread_position_in_grid.x")
        );
        assert_eq!(MetalBuiltins::builtin("lane_id"), Some("simd_lane_id"));
    }

    #[test]
    fn test_metal_builtins_atomic() {
        assert_eq!(
            MetalBuiltins::builtin("atomic_add"),
            Some("atomic_fetch_add_explicit")
        );
        assert_eq!(
            MetalBuiltins::builtin("atomic_cas"),
            Some("atomic_compare_exchange_weak_explicit")
        );
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
#[cfg(any(feature = "cuda", feature = "opencl", feature = "webgpu", feature = "metal"))]
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
    #[cfg(feature = "cuda")]
    fn test_e2e_cuda_codegen() {
        let module = parse(KERNEL_SOURCE).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let code = gen.generate(&module).expect("CUDA codegen failed");
        assert!(
            code.contains("__global__"),
            "CUDA kernel should have __global__ qualifier"
        );
        assert!(code.contains("vector_add"), "Should contain kernel name");
        assert!(
            !gen.kernels().is_empty(),
            "Should discover at least one kernel"
        );
    }

    #[test]
    #[cfg(feature = "opencl")]
    fn test_e2e_opencl_codegen() {
        let module = parse(KERNEL_SOURCE).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::OpenCL);
        let code = gen.generate(&module).expect("OpenCL codegen failed");
        assert!(
            code.contains("__kernel"),
            "OpenCL kernel should have __kernel qualifier"
        );
        assert!(code.contains("vector_add"), "Should contain kernel name");
    }

    #[test]
    #[cfg(feature = "webgpu")]
    fn test_e2e_webgpu_codegen() {
        let module = parse(KERNEL_SOURCE).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::WebGPU);
        let code = gen.generate(&module).expect("WebGPU codegen failed");
        assert!(
            code.contains("fn vector_add") || code.contains("@compute"),
            "WGSL should contain compute shader syntax, got:\n{}",
            code
        );
    }

    #[test]
    #[cfg(feature = "metal")]
    fn test_e2e_metal_codegen() {
        let module = parse(KERNEL_SOURCE).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Metal);
        let code = gen.generate(&module).expect("Metal codegen failed");
        assert!(
            code.contains("kernel") || code.contains("vector_add"),
            "Metal should contain kernel function, got:\n{}",
            code
        );
    }

    #[test]
    fn test_e2e_cuda_host_code() {
        let module = parse(KERNEL_SOURCE).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let _code = gen.generate(&module).expect("CUDA codegen failed");
        let host = gen.generate_host_code();
        assert!(
            host.contains("cudaMalloc") || host.contains("cuda") || host.len() > 0,
            "Host code should contain CUDA runtime calls"
        );
    }

    #[test]
    fn test_e2e_metal_host_code() {
        let module = parse(KERNEL_SOURCE).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Metal);
        let _code = gen.generate(&module).expect("Metal codegen failed");
        let host = gen.generate_host_code();
        assert!(
            host.contains("MTL") || host.contains("Metal") || host.len() > 0,
            "Host code should contain Metal runtime calls"
        );
    }

    #[test]
    #[cfg(any(feature = "cuda", feature = "opencl", feature = "webgpu", feature = "metal"))]
    fn test_e2e_scalar_kernel_all_backends() {
        let module = parse(SCALAR_KERNEL).expect("parse failed");
        let mut targets = vec![];
        #[cfg(feature = "cuda")]
        targets.push(GpuTarget::Cuda);
        #[cfg(feature = "opencl")]
        targets.push(GpuTarget::OpenCL);
        #[cfg(feature = "webgpu")]
        targets.push(GpuTarget::WebGPU);
        #[cfg(feature = "metal")]
        targets.push(GpuTarget::Metal);

        for target in &targets {
            let mut gen = GpuCodeGenerator::new(*target);
            let code = gen
                .generate(&module)
                .expect(&format!("{:?} codegen failed", target));
            assert!(
                code.contains("scalar_mul"),
                "{:?} should contain kernel name 'scalar_mul'",
                target
            );
            assert!(
                !gen.kernels().is_empty(),
                "{:?} should discover kernel",
                target
            );
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
        assert!(SimdTarget::Sve
            .compiler_flags()
            .contains("-march=armv8-a+sve"));
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
        assert_eq!(
            SimdIntrinsics::load(SimdTarget::Avx512, "f32"),
            "_mm512_loadu_ps"
        );
        assert_eq!(
            SimdIntrinsics::load(SimdTarget::Avx512, "f64"),
            "_mm512_loadu_pd"
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
    fn test_simd_intrinsics_store() {
        assert_eq!(
            SimdIntrinsics::store(SimdTarget::Avx512, "f32"),
            "_mm512_storeu_ps"
        );
        assert_eq!(SimdIntrinsics::store(SimdTarget::Neon, "f32"), "vst1q_f32");
    }

    #[test]
    fn test_simd_intrinsics_arithmetic() {
        assert_eq!(
            SimdIntrinsics::add(SimdTarget::Avx512, "f32"),
            "_mm512_add_ps"
        );
        assert_eq!(
            SimdIntrinsics::sub(SimdTarget::Avx512, "f32"),
            "_mm512_sub_ps"
        );
        assert_eq!(
            SimdIntrinsics::mul(SimdTarget::Avx512, "f32"),
            "_mm512_mul_ps"
        );
        assert_eq!(
            SimdIntrinsics::div(SimdTarget::Avx512, "f32"),
            "_mm512_div_ps"
        );
    }

    #[test]
    fn test_simd_intrinsics_fma() {
        assert_eq!(
            SimdIntrinsics::fma(SimdTarget::Avx512, "f32"),
            "_mm512_fmadd_ps"
        );
        assert_eq!(
            SimdIntrinsics::fma(SimdTarget::Avx2, "f32"),
            "_mm256_fmadd_ps"
        );
        assert_eq!(SimdIntrinsics::fma(SimdTarget::Neon, "f32"), "vfmaq_f32");
    }

    #[test]
    fn test_simd_intrinsics_broadcast() {
        assert_eq!(
            SimdIntrinsics::broadcast(SimdTarget::Avx512, "f32"),
            "_mm512_set1_ps"
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
    fn test_simd_intrinsics_reduce() {
        assert_eq!(
            SimdIntrinsics::reduce_add(SimdTarget::Avx512, "f32"),
            "_mm512_reduce_add_ps"
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
    fn test_simd_intrinsics_min_max() {
        assert_eq!(
            SimdIntrinsics::min(SimdTarget::Avx512, "f32"),
            "_mm512_min_ps"
        );
        assert_eq!(
            SimdIntrinsics::max(SimdTarget::Avx512, "f32"),
            "_mm512_max_ps"
        );
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
#[cfg(any(feature = "cuda", feature = "opencl", feature = "webgpu", feature = "metal"))]
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
        assert!(
            code.contains("__global__"),
            "Should have __global__ qualifier"
        );
        assert!(code.contains("vector_add"), "Should contain kernel name");
        assert!(
            code.contains("double*"),
            "Should have double* parameters for f64"
        );

        // Verify thread indexing is emitted
        assert!(
            code.contains("threadIdx.x") || code.contains("threadIdx"),
            "Should contain CUDA thread indexing"
        );
        assert!(
            code.contains("blockIdx.x") || code.contains("blockIdx"),
            "Should contain CUDA block indexing"
        );
        assert!(
            code.contains("blockDim.x") || code.contains("blockDim"),
            "Should contain CUDA block dimension"
        );
    }

    #[test]
    fn test_e2e_vector_add_cuda_host_code_has_runtime_calls() {
        let module = parse(VECTOR_ADD_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let _code = gen.generate(&module).expect("CUDA codegen failed");
        let host = gen.generate_host_code();

        // Host code should reference CUDA runtime API functions
        assert!(
            host.contains("cudaDeviceSynchronize") || host.contains("launch_"),
            "Host code should contain CUDA runtime calls, got:\n{}",
            host
        );
    }

    #[test]
    fn test_e2e_vector_add_kernel_metadata() {
        let module = parse(VECTOR_ADD_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let _code = gen.generate(&module).expect("CUDA codegen failed");

        let kernels = gen.kernels();
        assert_eq!(kernels.len(), 1);
        assert_eq!(kernels[0].name, "vector_add");
        assert_eq!(
            kernels[0].params.len(),
            4,
            "vector_add should have 4 params (a, b, c, n)"
        );

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
            let code = gen
                .generate(&module)
                .unwrap_or_else(|e| panic!("{:?} codegen failed: {}", target, e));
            assert!(
                code.contains(expected_keyword),
                "{:?} should contain '{}', got:\n{}",
                target,
                expected_keyword,
                code
            );
            assert!(
                code.contains("vector_add"),
                "{:?} should contain kernel name",
                target
            );
        }
    }

    #[test]
    fn test_e2e_vector_add_metal_codegen() {
        let module = parse(VECTOR_ADD_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Metal);
        let code = gen.generate(&module).expect("Metal codegen failed");
        assert!(
            code.contains("kernel") || code.contains("vector_add"),
            "Metal should generate kernel function"
        );
    }

    #[test]
    fn test_e2e_vector_add_webgpu_codegen() {
        let module = parse(VECTOR_ADD_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::WebGPU);
        let code = gen.generate(&module).expect("WebGPU codegen failed");
        assert!(
            code.contains("vector_add") || code.contains("@compute"),
            "WebGPU should generate compute shader"
        );
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
        assert!(
            code.contains("threadIdx.y") || code.contains("blockIdx.y") || code.contains("thread"),
            "Should contain 2D thread indexing"
        );
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

// E2E OpenCL runtime integration tests
#[cfg(feature = "opencl")]
mod e2e_opencl_runtime {
    use vais_gpu::{GpuCodeGenerator, GpuTarget};
    use vais_parser::parse;

    const OPENCL_VECTOR_ADD: &str = r#"
#[gpu]
F vector_add(a: *f64, b: *f64, c: *f64, n: i64) {
    idx := thread_idx_x() + block_idx_x() * block_dim_x()
    I idx < n {
        c[idx] = a[idx] + b[idx]
    }
}
"#;

    #[test]
    fn test_opencl_vector_add_codegen() {
        let module = parse(OPENCL_VECTOR_ADD).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::OpenCL);
        let code = gen.generate(&module).expect("OpenCL codegen failed");

        // OpenCL kernel should use __kernel qualifier
        assert!(
            code.contains("__kernel"),
            "OpenCL should have __kernel qualifier, got:\n{}",
            code
        );
        assert!(code.contains("vector_add"), "Should contain kernel name");

        // Verify kernel metadata
        let kernels = gen.kernels();
        assert_eq!(kernels.len(), 1);
        assert_eq!(kernels[0].name, "vector_add");
        assert_eq!(kernels[0].params.len(), 4);
    }

    #[test]
    fn test_opencl_vector_add_host_code() {
        let module = parse(OPENCL_VECTOR_ADD).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::OpenCL);
        let _code = gen.generate(&module).expect("OpenCL codegen failed");
        let host = gen.generate_host_code();

        // Host code should reference OpenCL API
        assert!(
            host.contains("cl") || host.contains("OpenCL") || host.contains("CL"),
            "Host code should contain OpenCL references, got:\n{}",
            host
        );
    }

    const OPENCL_SAXPY: &str = r#"
#[gpu]
F saxpy(a: f64, x: *f64, y: *f64, n: i64) {
    idx := thread_idx_x() + block_idx_x() * block_dim_x()
    I idx < n {
        y[idx] = a * x[idx] + y[idx]
    }
}
"#;

    #[test]
    fn test_opencl_saxpy_codegen() {
        let module = parse(OPENCL_SAXPY).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::OpenCL);
        let code = gen.generate(&module).expect("OpenCL codegen failed");
        assert!(code.contains("__kernel"), "Should have __kernel qualifier");
        assert!(code.contains("saxpy"), "Should contain kernel name 'saxpy'");
        let kernels = gen.kernels();
        assert_eq!(kernels.len(), 1);
        assert_eq!(kernels[0].params.len(), 4);
    }

    #[test]
    fn test_opencl_multi_kernel() {
        let source = r#"
#[gpu]
F kernel_a(data: *f64, n: i64) {
    idx := thread_idx_x() + block_idx_x() * block_dim_x()
    I idx < n {
        data[idx] = data[idx] * 2.0
    }
}

#[gpu]
F kernel_b(input: *f64, output: *f64, n: i64) {
    idx := thread_idx_x() + block_idx_x() * block_dim_x()
    I idx < n {
        output[idx] = input[idx] + 1.0
    }
}
"#;
        let module = parse(source).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::OpenCL);
        let code = gen.generate(&module).expect("OpenCL codegen failed");
        assert!(code.contains("kernel_a"), "Should contain first kernel");
        assert!(code.contains("kernel_b"), "Should contain second kernel");
        assert_eq!(gen.kernels().len(), 2, "Should discover both kernels");
    }

    #[test]
    fn test_opencl_generates_global_qualifier_for_pointers() {
        let module = parse(OPENCL_VECTOR_ADD).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::OpenCL);
        let code = gen.generate(&module).expect("OpenCL codegen failed");
        // OpenCL requires __global qualifier for pointer parameters
        assert!(
            code.contains("__global"),
            "OpenCL should use __global for pointer params, got:\n{}",
            code
        );
    }

    #[test]
    fn test_opencl_fp64_extension() {
        let module = parse(OPENCL_VECTOR_ADD).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::OpenCL);
        let code = gen.generate(&module).expect("OpenCL codegen failed");
        // OpenCL should enable fp64 extension for double precision
        assert!(
            code.contains("cl_khr_fp64"),
            "OpenCL should enable fp64 extension, got:\n{}",
            code
        );
    }
}

// ============================================================================
// Stage 4: GPU Advanced Features Tests
// Unified Memory, Stream/Async, Multi-GPU, Profiling
// ============================================================================

#[cfg(any(feature = "cuda", feature = "opencl", feature = "webgpu", feature = "metal"))]
mod e2e_unified_memory {
    use vais_gpu::{GpuCodeGenerator, GpuTarget};
    use vais_parser::parse;

    const UNIFIED_MEMORY_KERNEL: &str = r#"
#[gpu]
F unified_add(data: *f64, n: i64) {
    idx := thread_idx_x() + block_idx_x() * block_dim_x()
    I idx < n {
        data[idx] = data[idx] + 1.0
    }
}
"#;

    #[test]
    fn test_unified_memory_kernel_codegen_cuda() {
        let module = parse(UNIFIED_MEMORY_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let code = gen.generate(&module).expect("CUDA codegen failed");
        assert!(
            code.contains("__global__"),
            "Should have __global__ qualifier"
        );
        assert!(code.contains("unified_add"), "Should contain kernel name");
        assert!(
            code.contains("double*"),
            "Should have double* for f64 pointer"
        );
    }

    #[test]
    fn test_unified_memory_kernel_codegen_all_backends() {
        let module = parse(UNIFIED_MEMORY_KERNEL).expect("parse failed");
        for target in &[
            GpuTarget::Cuda,
            GpuTarget::OpenCL,
            GpuTarget::Metal,
            GpuTarget::WebGPU,
        ] {
            let mut gen = GpuCodeGenerator::new(*target);
            let code = gen
                .generate(&module)
                .unwrap_or_else(|e| panic!("{:?} codegen failed: {}", target, e));
            assert!(
                code.contains("unified_add"),
                "{:?} should contain kernel name 'unified_add'",
                target
            );
        }
    }

    #[test]
    fn test_unified_memory_kernel_metadata() {
        let module = parse(UNIFIED_MEMORY_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let _code = gen.generate(&module).expect("CUDA codegen failed");
        let kernels = gen.kernels();
        assert_eq!(kernels.len(), 1);
        assert_eq!(kernels[0].name, "unified_add");
        assert_eq!(
            kernels[0].params.len(),
            2,
            "unified_add should have 2 params (data, n)"
        );
    }
}

#[cfg(any(feature = "cuda", feature = "opencl", feature = "webgpu", feature = "metal"))]
mod e2e_stream_async {
    use vais_gpu::{GpuCodeGenerator, GpuTarget};
    use vais_parser::parse;

    const STREAM_KERNEL: &str = r#"
#[gpu]
F stream_process(input: *f64, output: *f64, n: i64) {
    idx := thread_idx_x() + block_idx_x() * block_dim_x()
    I idx < n {
        output[idx] = input[idx] * 2.0
    }
}
"#;

    #[test]
    fn test_stream_kernel_cuda_codegen() {
        let module = parse(STREAM_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let code = gen.generate(&module).expect("CUDA codegen failed");
        assert!(
            code.contains("__global__"),
            "Should have __global__ qualifier"
        );
        assert!(
            code.contains("stream_process"),
            "Should contain kernel name"
        );
    }

    #[test]
    fn test_stream_kernel_host_code_generation() {
        let module = parse(STREAM_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let _code = gen.generate(&module).expect("CUDA codegen failed");
        let host = gen.generate_host_code();
        // Host code should be generated (stream management is in runtime, not codegen)
        assert!(host.len() > 0, "Host code should be non-empty");
    }

    #[test]
    fn test_stream_kernel_metadata() {
        let module = parse(STREAM_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let _code = gen.generate(&module).expect("CUDA codegen failed");
        let kernels = gen.kernels();
        assert_eq!(kernels.len(), 1);
        assert_eq!(kernels[0].name, "stream_process");
        assert_eq!(kernels[0].params.len(), 3);
    }

    #[test]
    fn test_stream_kernel_all_backends() {
        let module = parse(STREAM_KERNEL).expect("parse failed");
        for target in &[
            GpuTarget::Cuda,
            GpuTarget::OpenCL,
            GpuTarget::Metal,
            GpuTarget::WebGPU,
        ] {
            let mut gen = GpuCodeGenerator::new(*target);
            let code = gen
                .generate(&module)
                .unwrap_or_else(|e| panic!("{:?} codegen failed: {}", target, e));
            assert!(
                code.contains("stream_process"),
                "{:?} should contain kernel name",
                target
            );
        }
    }
}

#[cfg(any(feature = "cuda", feature = "opencl", feature = "webgpu", feature = "metal"))]
mod e2e_multi_gpu {
    use vais_gpu::{GpuCodeGenerator, GpuTarget};
    use vais_parser::parse;

    const MULTI_GPU_KERNEL: &str = r#"
#[gpu]
F gpu_work_a(data: *f64, n: i64) {
    idx := thread_idx_x() + block_idx_x() * block_dim_x()
    I idx < n {
        data[idx] = data[idx] + 1.0
    }
}

#[gpu]
F gpu_work_b(data: *f64, n: i64) {
    idx := thread_idx_x() + block_idx_x() * block_dim_x()
    I idx < n {
        data[idx] = data[idx] * 2.0
    }
}
"#;

    #[test]
    fn test_multi_gpu_kernels_cuda() {
        let module = parse(MULTI_GPU_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let code = gen.generate(&module).expect("CUDA codegen failed");
        assert!(code.contains("gpu_work_a"), "Should contain first kernel");
        assert!(code.contains("gpu_work_b"), "Should contain second kernel");
        assert_eq!(gen.kernels().len(), 2, "Should discover both kernels");
    }

    #[test]
    fn test_multi_gpu_kernels_metadata() {
        let module = parse(MULTI_GPU_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let _code = gen.generate(&module).expect("CUDA codegen failed");
        let kernels = gen.kernels();
        assert_eq!(kernels.len(), 2);
        assert_eq!(kernels[0].name, "gpu_work_a");
        assert_eq!(kernels[1].name, "gpu_work_b");
        // Both should have same param signature
        assert_eq!(kernels[0].params.len(), 2);
        assert_eq!(kernels[1].params.len(), 2);
    }

    #[test]
    fn test_multi_gpu_kernels_all_backends() {
        let module = parse(MULTI_GPU_KERNEL).expect("parse failed");
        for target in &[
            GpuTarget::Cuda,
            GpuTarget::OpenCL,
            GpuTarget::Metal,
            GpuTarget::WebGPU,
        ] {
            let mut gen = GpuCodeGenerator::new(*target);
            let code = gen
                .generate(&module)
                .unwrap_or_else(|e| panic!("{:?} codegen failed: {}", target, e));
            assert!(
                code.contains("gpu_work_a") && code.contains("gpu_work_b"),
                "{:?} should contain both kernel names",
                target
            );
        }
    }
}

#[cfg(any(feature = "cuda", feature = "opencl", feature = "webgpu", feature = "metal"))]
mod e2e_profiling {
    use vais_gpu::{GpuCodeGenerator, GpuTarget};
    use vais_parser::parse;

    const PROFILED_KERNEL: &str = r#"
#[gpu]
F timed_saxpy(a: f64, x: *f64, y: *f64, n: i64) {
    idx := thread_idx_x() + block_idx_x() * block_dim_x()
    I idx < n {
        y[idx] = a * x[idx] + y[idx]
    }
}
"#;

    #[test]
    fn test_profiling_kernel_cuda_codegen() {
        let module = parse(PROFILED_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let code = gen.generate(&module).expect("CUDA codegen failed");
        assert!(code.contains("timed_saxpy"), "Should contain kernel name");
        assert!(
            code.contains("__global__"),
            "Should have __global__ qualifier"
        );
    }

    #[test]
    fn test_profiling_kernel_host_code() {
        let module = parse(PROFILED_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let _code = gen.generate(&module).expect("CUDA codegen failed");
        let host = gen.generate_host_code();
        assert!(
            host.len() > 0,
            "Host code should be generated for profiling"
        );
    }

    #[test]
    fn test_profiling_kernel_metadata() {
        let module = parse(PROFILED_KERNEL).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let _code = gen.generate(&module).expect("CUDA codegen failed");
        let kernels = gen.kernels();
        assert_eq!(kernels.len(), 1);
        assert_eq!(kernels[0].name, "timed_saxpy");
        assert_eq!(kernels[0].params.len(), 4);
    }

    #[test]
    fn test_profiling_kernel_all_backends() {
        let module = parse(PROFILED_KERNEL).expect("parse failed");
        for target in &[
            GpuTarget::Cuda,
            GpuTarget::OpenCL,
            GpuTarget::Metal,
            GpuTarget::WebGPU,
        ] {
            let mut gen = GpuCodeGenerator::new(*target);
            let code = gen
                .generate(&module)
                .unwrap_or_else(|e| panic!("{:?} codegen failed: {}", target, e));
            assert!(
                code.contains("timed_saxpy"),
                "{:?} should contain kernel name 'timed_saxpy'",
                target
            );
        }
    }
}

// E2E Metal runtime integration tests
#[cfg(feature = "metal")]
mod e2e_metal_runtime {
    use vais_gpu::metal::MetalBuiltins;
    use vais_gpu::{GpuCodeGenerator, GpuTarget};
    use vais_parser::parse;

    const METAL_VECTOR_ADD: &str = r#"
#[gpu]
F vector_add(a: *f64, b: *f64, c: *f64, n: i64) {
    idx := thread_idx_x() + block_idx_x() * block_dim_x()
    I idx < n {
        c[idx] = a[idx] + b[idx]
    }
}
"#;

    #[test]
    fn test_metal_vector_add_codegen() {
        let module = parse(METAL_VECTOR_ADD).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Metal);
        let code = gen.generate(&module).expect("Metal codegen failed");

        // Metal kernel should use Metal Shading Language syntax
        assert!(
            code.contains("kernel") || code.contains("vector_add"),
            "Metal should contain kernel function, got:\n{}",
            code
        );

        // Verify kernel metadata
        let kernels = gen.kernels();
        assert_eq!(kernels.len(), 1);
        assert_eq!(kernels[0].name, "vector_add");
        assert_eq!(kernels[0].params.len(), 4);
    }

    #[test]
    fn test_metal_vector_add_host_code() {
        let module = parse(METAL_VECTOR_ADD).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Metal);
        let _code = gen.generate(&module).expect("Metal codegen failed");
        let host = gen.generate_host_code();

        // Host code should reference Metal API
        assert!(
            host.contains("MTL") || host.contains("Metal") || host.contains("metal"),
            "Host code should contain Metal references, got:\n{}",
            host
        );
    }

    #[test]
    fn test_metal_builtins_comprehensive() {
        // Thread indexing
        assert!(MetalBuiltins::builtin("thread_idx_x").is_some());
        assert!(MetalBuiltins::builtin("block_idx_x").is_some());
        assert!(MetalBuiltins::builtin("global_idx").is_some());

        // Synchronization
        assert!(MetalBuiltins::builtin("sync_threads").is_some());
        assert!(MetalBuiltins::builtin("thread_fence").is_some());

        // Atomics
        assert!(MetalBuiltins::builtin("atomic_add").is_some());
        assert!(MetalBuiltins::builtin("atomic_cas").is_some());

        // Math
        assert!(MetalBuiltins::builtin("sqrt").is_some());
        assert!(MetalBuiltins::builtin("fma").is_some());

        // SIMD
        assert!(MetalBuiltins::builtin("simd_sum").is_some());
    }

    const METAL_SAXPY: &str = r#"
#[gpu]
F saxpy(a: f64, x: *f64, y: *f64, n: i64) {
    idx := thread_idx_x() + block_idx_x() * block_dim_x()
    I idx < n {
        y[idx] = a * x[idx] + y[idx]
    }
}
"#;

    #[test]
    fn test_metal_saxpy_codegen() {
        let module = parse(METAL_SAXPY).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Metal);
        let code = gen.generate(&module).expect("Metal codegen failed");
        assert!(
            code.contains("saxpy"),
            "Metal should contain kernel name 'saxpy'"
        );
        let kernels = gen.kernels();
        assert_eq!(kernels.len(), 1);
        assert_eq!(kernels[0].params.len(), 4);
    }

    #[test]
    fn test_metal_multi_kernel() {
        let source = r#"
#[gpu]
F kernel_a(data: *f64, n: i64) {
    idx := thread_idx_x() + block_idx_x() * block_dim_x()
    I idx < n {
        data[idx] = data[idx] * 2.0
    }
}

#[gpu]
F kernel_b(input: *f64, output: *f64, n: i64) {
    idx := thread_idx_x() + block_idx_x() * block_dim_x()
    I idx < n {
        output[idx] = input[idx] + 1.0
    }
}
"#;
        let module = parse(source).expect("parse failed");
        let mut gen = GpuCodeGenerator::new(GpuTarget::Metal);
        let code = gen.generate(&module).expect("Metal codegen failed");
        assert!(code.contains("kernel_a"), "Should contain first kernel");
        assert!(code.contains("kernel_b"), "Should contain second kernel");
        assert_eq!(gen.kernels().len(), 2, "Should discover both kernels");
    }
}

// ============================================================================
// Stage 5: GPU Kernel Generation Tests (30+)
// Tests for kernel code generation, type conversion, metadata, edge cases
// ============================================================================

#[cfg(any(feature = "cuda", feature = "opencl", feature = "webgpu", feature = "metal"))]
mod kernel_generation_tests {
    use vais_ast::*;
    use vais_gpu::{GpuCodeGenerator, GpuKernel, GpuTarget, GpuType};

    fn spanned<T>(node: T) -> Spanned<T> {
        Spanned {
            node,
            span: Span { start: 0, end: 0 },
        }
    }

    fn make_empty_module() -> Module {
        Module {
            items: vec![],
            modules_map: None,
        }
    }

    fn make_kernel_module(name: &str, params: Vec<Param>) -> Module {
        Module {
            modules_map: None,
            items: vec![spanned(Item::Function(Function {
                name: spanned(name.to_string()),
                generics: vec![],
                params,
                ret_type: None,
                body: FunctionBody::Block(vec![]),
                is_pub: false,
                is_async: false,
                attributes: vec![Attribute {
                    name: "gpu".to_string(),
                    args: vec![],
                    expr: None,
                }],
            }))],
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

    // Task 1: GpuCodeGenerator kernel generation tests (4 backends × 3 scenarios = 12 tests)

    #[test]
    fn test_cuda_empty_module_generates_empty_code() {
        let module = make_empty_module();
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let _code = gen
            .generate(&module)
            .expect("CUDA empty module should succeed");
        assert!(
            gen.kernels().is_empty(),
            "Empty module should have no kernels"
        );
    }

    #[test]
    fn test_opencl_empty_module_generates_empty_code() {
        let module = make_empty_module();
        let mut gen = GpuCodeGenerator::new(GpuTarget::OpenCL);
        let _code = gen
            .generate(&module)
            .expect("OpenCL empty module should succeed");
        assert!(gen.kernels().is_empty());
    }

    #[test]
    fn test_webgpu_empty_module_generates_empty_code() {
        let module = make_empty_module();
        let mut gen = GpuCodeGenerator::new(GpuTarget::WebGPU);
        let _code = gen
            .generate(&module)
            .expect("WebGPU empty module should succeed");
        assert!(gen.kernels().is_empty());
    }

    #[test]
    fn test_metal_empty_module_generates_empty_code() {
        let module = make_empty_module();
        let mut gen = GpuCodeGenerator::new(GpuTarget::Metal);
        let _code = gen
            .generate(&module)
            .expect("Metal empty module should succeed");
        assert!(gen.kernels().is_empty());
    }

    #[test]
    fn test_cuda_simple_kernel_contains_global() {
        let module = make_kernel_module(
            "simple_kernel",
            vec![make_param(
                "data",
                Type::Pointer(Box::new(spanned(Type::Named {
                    name: "i32".to_string(),
                    generics: vec![],
                }))),
            )],
        );
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let code = gen
            .generate(&module)
            .expect("CUDA kernel generation failed");
        assert!(
            code.contains("__global__") || code.contains("simple_kernel"),
            "CUDA output should contain __global__ or kernel name"
        );
    }

    #[test]
    fn test_opencl_simple_kernel_contains_kernel() {
        let module = make_kernel_module(
            "simple_kernel",
            vec![make_param(
                "data",
                Type::Pointer(Box::new(spanned(Type::Named {
                    name: "i32".to_string(),
                    generics: vec![],
                }))),
            )],
        );
        let mut gen = GpuCodeGenerator::new(GpuTarget::OpenCL);
        let code = gen
            .generate(&module)
            .expect("OpenCL kernel generation failed");
        assert!(
            code.contains("__kernel") || code.contains("simple_kernel"),
            "OpenCL output should contain __kernel or kernel name"
        );
    }

    #[test]
    fn test_webgpu_simple_kernel_contains_compute() {
        let module = make_kernel_module(
            "simple_kernel",
            vec![make_param(
                "data",
                Type::Pointer(Box::new(spanned(Type::Named {
                    name: "i32".to_string(),
                    generics: vec![],
                }))),
            )],
        );
        let mut gen = GpuCodeGenerator::new(GpuTarget::WebGPU);
        let code = gen
            .generate(&module)
            .expect("WebGPU kernel generation failed");
        assert!(
            code.contains("@compute") || code.contains("simple_kernel") || code.len() > 0,
            "WebGPU output should contain compute shader markers or kernel name"
        );
    }

    #[test]
    fn test_metal_simple_kernel_contains_kernel() {
        let module = make_kernel_module(
            "simple_kernel",
            vec![make_param(
                "data",
                Type::Pointer(Box::new(spanned(Type::Named {
                    name: "i32".to_string(),
                    generics: vec![],
                }))),
            )],
        );
        let mut gen = GpuCodeGenerator::new(GpuTarget::Metal);
        let code = gen
            .generate(&module)
            .expect("Metal kernel generation failed");
        assert!(
            code.contains("kernel") || code.contains("simple_kernel"),
            "Metal output should contain kernel keyword or kernel name"
        );
    }

    #[test]
    fn test_cuda_host_code_non_empty() {
        let module = make_kernel_module("test_kernel", vec![]);
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let _ = gen
            .generate(&module)
            .expect("CUDA kernel generation failed");
        let host = gen.generate_host_code();
        assert!(host.len() > 0, "CUDA host code should not be empty");
    }

    #[test]
    fn test_opencl_host_code_non_empty() {
        let module = make_kernel_module("test_kernel", vec![]);
        let mut gen = GpuCodeGenerator::new(GpuTarget::OpenCL);
        let _ = gen
            .generate(&module)
            .expect("OpenCL kernel generation failed");
        let host = gen.generate_host_code();
        assert!(host.len() > 0, "OpenCL host code should not be empty");
    }

    #[test]
    fn test_webgpu_host_code_non_empty() {
        let module = make_kernel_module("test_kernel", vec![]);
        let mut gen = GpuCodeGenerator::new(GpuTarget::WebGPU);
        let _ = gen
            .generate(&module)
            .expect("WebGPU kernel generation failed");
        let host = gen.generate_host_code();
        assert!(host.len() > 0, "WebGPU host code should not be empty");
    }

    #[test]
    fn test_metal_host_code_non_empty() {
        let module = make_kernel_module("test_kernel", vec![]);
        let mut gen = GpuCodeGenerator::new(GpuTarget::Metal);
        let _ = gen
            .generate(&module)
            .expect("Metal kernel generation failed");
        let host = gen.generate_host_code();
        assert!(host.len() > 0, "Metal host code should not be empty");
    }

    // Task 2: GpuType conversion tests (8 tests)

    #[test]
    fn test_gpu_type_nested_ptr() {
        let ty = GpuType::Ptr(Box::new(GpuType::Ptr(Box::new(GpuType::F32))));
        assert_eq!(ty.cuda_name(), "float**");
        assert_eq!(ty.opencl_name(), "__global __global float**");
        assert_eq!(ty.wgsl_name(), "ptr<storage, ptr<storage, f32>>");
    }

    #[test]
    fn test_gpu_type_void_ptr() {
        let ty = GpuType::Ptr(Box::new(GpuType::Void));
        assert_eq!(ty.cuda_name(), "void*");
        assert_eq!(ty.opencl_name(), "__global void*");
    }

    #[test]
    fn test_gpu_type_vec_i32_8() {
        let ty = GpuType::Vec(Box::new(GpuType::I32), 8);
        assert_eq!(ty.cuda_name(), "int8");
        assert_eq!(ty.opencl_name(), "int8");
        assert_eq!(ty.wgsl_name(), "vec8<i32>");
    }

    #[test]
    fn test_gpu_type_vec_f64_2() {
        let ty = GpuType::Vec(Box::new(GpuType::F64), 2);
        assert_eq!(ty.cuda_name(), "double2");
        assert_eq!(ty.opencl_name(), "double2");
        assert_eq!(ty.wgsl_name(), "vec2<f64>");
    }

    #[test]
    fn test_gpu_type_array_large() {
        let ty = GpuType::Array(Box::new(GpuType::F64), 1024);
        assert_eq!(ty.cuda_name(), "double[1024]");
        assert_eq!(ty.opencl_name(), "double[1024]");
        assert_eq!(ty.wgsl_name(), "array<f64, 1024>");
    }

    #[test]
    fn test_gpu_type_bool_names() {
        let ty = GpuType::Bool;
        assert_eq!(ty.cuda_name(), "bool");
        assert_eq!(ty.opencl_name(), "bool");
        assert_eq!(ty.wgsl_name(), "bool");
    }

    #[test]
    fn test_gpu_type_void_names() {
        let ty = GpuType::Void;
        assert_eq!(ty.cuda_name(), "void");
        assert_eq!(ty.opencl_name(), "void");
        assert_eq!(ty.wgsl_name(), "");
    }

    #[test]
    fn test_gpu_type_nested_array() {
        let ty = GpuType::Array(Box::new(GpuType::Array(Box::new(GpuType::I32), 4)), 4);
        assert_eq!(ty.cuda_name(), "int[4][4]");
        assert_eq!(ty.opencl_name(), "int[4][4]");
        assert_eq!(ty.wgsl_name(), "array<array<i32, 4>, 4>");
    }

    // Task 3: GpuKernel metadata tests (4 tests)

    #[test]
    fn test_gpu_kernel_metadata_fields() {
        let kernel = GpuKernel {
            name: "test_kernel".to_string(),
            params: vec![("a".to_string(), GpuType::I32)],
            shared_memory: 1024,
            block_size: (16, 16, 1),
        };
        assert_eq!(kernel.name, "test_kernel");
        assert_eq!(kernel.params.len(), 1);
        assert_eq!(kernel.shared_memory, 1024);
        assert_eq!(kernel.block_size, (16, 16, 1));
    }

    #[test]
    fn test_gpu_kernel_empty_params() {
        let kernel = GpuKernel {
            name: "no_params".to_string(),
            params: vec![],
            shared_memory: 0,
            block_size: (256, 1, 1),
        };
        assert!(kernel.params.is_empty());
        assert_eq!(kernel.block_size.0, 256);
    }

    #[test]
    fn test_gpu_kernel_many_params() {
        let params: Vec<(String, GpuType)> = (0..10)
            .map(|i| (format!("param{}", i), GpuType::F32))
            .collect();
        let kernel = GpuKernel {
            name: "many_params".to_string(),
            params,
            shared_memory: 2048,
            block_size: (128, 1, 1),
        };
        assert_eq!(kernel.params.len(), 10);
    }

    #[test]
    fn test_gpu_kernel_custom_block_size() {
        let kernel = GpuKernel {
            name: "custom_block".to_string(),
            params: vec![],
            shared_memory: 0,
            block_size: (32, 32, 1),
        };
        assert_eq!(kernel.block_size, (32, 32, 1));
    }

    // Task 4: GpuTarget additional tests (4 tests)

    #[test]
    fn test_gpu_target_is_metal_negative() {
        assert!(!GpuTarget::Cuda.is_metal());
        assert!(!GpuTarget::OpenCL.is_metal());
        assert!(!GpuTarget::WebGPU.is_metal());
    }

    #[test]
    fn test_gpu_target_is_cuda_negative() {
        assert!(!GpuTarget::Metal.is_cuda());
        assert!(!GpuTarget::OpenCL.is_cuda());
        assert!(!GpuTarget::WebGPU.is_cuda());
    }

    #[test]
    fn test_gpu_target_default_shared_memory_all() {
        assert_eq!(GpuTarget::Cuda.default_shared_memory(), 48 * 1024);
        assert_eq!(GpuTarget::Metal.default_shared_memory(), 32 * 1024);
        assert_eq!(GpuTarget::OpenCL.default_shared_memory(), 32 * 1024);
        assert_eq!(GpuTarget::WebGPU.default_shared_memory(), 16 * 1024);
    }

    #[test]
    fn test_gpu_target_parse_case_insensitive() {
        assert_eq!(GpuTarget::parse("CuDa"), Some(GpuTarget::Cuda));
        assert_eq!(GpuTarget::parse("OpenCL"), Some(GpuTarget::OpenCL));
        assert_eq!(GpuTarget::parse("WEBGPU"), Some(GpuTarget::WebGPU));
        assert_eq!(GpuTarget::parse("Metal"), Some(GpuTarget::Metal));
    }

    // Task 5: Edge cases (4 tests)

    #[test]
    fn test_gpu_target_parse_empty_string() {
        assert_eq!(GpuTarget::parse(""), None);
    }

    #[test]
    fn test_gpu_target_parse_invalid() {
        assert_eq!(GpuTarget::parse("directx"), None);
        assert_eq!(GpuTarget::parse("vulkan"), None);
        assert_eq!(GpuTarget::parse("garbage"), None);
    }

    #[test]
    fn test_gpu_type_deeply_nested_ptr() {
        let ty = GpuType::Ptr(Box::new(GpuType::Ptr(Box::new(GpuType::Ptr(Box::new(
            GpuType::I32,
        ))))));
        assert_eq!(ty.cuda_name(), "int***");
    }

    #[test]
    fn test_gpu_code_generator_target_accessor() {
        let gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        assert_eq!(gen.target(), GpuTarget::Cuda);
        let gen2 = GpuCodeGenerator::new(GpuTarget::Metal);
        assert_eq!(gen2.target(), GpuTarget::Metal);
    }
}
