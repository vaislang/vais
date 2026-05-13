use super::*;

// ── MetalBuiltins tests ──

#[test]
fn test_metal_builtins_math() {
    assert_eq!(MetalBuiltins::builtin("sqrt"), Some("sqrt"));
    assert_eq!(MetalBuiltins::builtin("rsqrt"), Some("rsqrt"));
    assert_eq!(MetalBuiltins::builtin("sin"), Some("sin"));
    assert_eq!(MetalBuiltins::builtin("cos"), Some("cos"));
    assert_eq!(MetalBuiltins::builtin("tan"), Some("tan"));
    assert_eq!(MetalBuiltins::builtin("exp"), Some("exp"));
    assert_eq!(MetalBuiltins::builtin("exp2"), Some("exp2"));
    assert_eq!(MetalBuiltins::builtin("log"), Some("log"));
    assert_eq!(MetalBuiltins::builtin("log2"), Some("log2"));
    assert_eq!(MetalBuiltins::builtin("log10"), Some("log10"));
    assert_eq!(MetalBuiltins::builtin("pow"), Some("pow"));
    assert_eq!(MetalBuiltins::builtin("abs"), Some("abs"));
    assert_eq!(MetalBuiltins::builtin("fabs"), Some("fabs"));
    assert_eq!(MetalBuiltins::builtin("floor"), Some("floor"));
    assert_eq!(MetalBuiltins::builtin("ceil"), Some("ceil"));
    assert_eq!(MetalBuiltins::builtin("round"), Some("round"));
    assert_eq!(MetalBuiltins::builtin("trunc"), Some("trunc"));
    assert_eq!(MetalBuiltins::builtin("fract"), Some("fract"));
    assert_eq!(MetalBuiltins::builtin("min"), Some("min"));
    assert_eq!(MetalBuiltins::builtin("max"), Some("max"));
    assert_eq!(MetalBuiltins::builtin("clamp"), Some("clamp"));
    assert_eq!(MetalBuiltins::builtin("mix"), Some("mix"));
    assert_eq!(MetalBuiltins::builtin("step"), Some("step"));
    assert_eq!(MetalBuiltins::builtin("smoothstep"), Some("smoothstep"));
    assert_eq!(MetalBuiltins::builtin("fma"), Some("fma"));
}

#[test]
fn test_metal_builtins_trig_inverse() {
    assert_eq!(MetalBuiltins::builtin("asin"), Some("asin"));
    assert_eq!(MetalBuiltins::builtin("acos"), Some("acos"));
    assert_eq!(MetalBuiltins::builtin("atan"), Some("atan"));
    assert_eq!(MetalBuiltins::builtin("atan2"), Some("atan2"));
}

#[test]
fn test_metal_builtins_hyperbolic() {
    assert_eq!(MetalBuiltins::builtin("sinh"), Some("sinh"));
    assert_eq!(MetalBuiltins::builtin("cosh"), Some("cosh"));
    assert_eq!(MetalBuiltins::builtin("tanh"), Some("tanh"));
}

#[test]
fn test_metal_builtins_atomics() {
    assert_eq!(
        MetalBuiltins::builtin("atomic_add"),
        Some("atomic_fetch_add_explicit")
    );
    assert_eq!(
        MetalBuiltins::builtin("atomic_sub"),
        Some("atomic_fetch_sub_explicit")
    );
    assert_eq!(
        MetalBuiltins::builtin("atomic_min"),
        Some("atomic_fetch_min_explicit")
    );
    assert_eq!(
        MetalBuiltins::builtin("atomic_max"),
        Some("atomic_fetch_max_explicit")
    );
    assert_eq!(
        MetalBuiltins::builtin("atomic_and"),
        Some("atomic_fetch_and_explicit")
    );
    assert_eq!(
        MetalBuiltins::builtin("atomic_or"),
        Some("atomic_fetch_or_explicit")
    );
    assert_eq!(
        MetalBuiltins::builtin("atomic_xor"),
        Some("atomic_fetch_xor_explicit")
    );
    assert_eq!(
        MetalBuiltins::builtin("atomic_cas"),
        Some("atomic_compare_exchange_weak_explicit")
    );
    assert_eq!(
        MetalBuiltins::builtin("atomic_exch"),
        Some("atomic_exchange_explicit")
    );
}

#[test]
fn test_metal_builtins_sync() {
    assert_eq!(
        MetalBuiltins::builtin("sync_threads"),
        Some("threadgroup_barrier(mem_flags::mem_threadgroup)")
    );
    assert_eq!(
        MetalBuiltins::builtin("thread_fence"),
        Some("threadgroup_barrier(mem_flags::mem_device)")
    );
    assert_eq!(
        MetalBuiltins::builtin("thread_fence_block"),
        Some("threadgroup_barrier(mem_flags::mem_threadgroup)")
    );
}

#[test]
fn test_metal_builtins_thread_indexing() {
    assert_eq!(
        MetalBuiltins::builtin("thread_idx_x"),
        Some("thread_position_in_threadgroup.x")
    );
    assert_eq!(
        MetalBuiltins::builtin("thread_idx_y"),
        Some("thread_position_in_threadgroup.y")
    );
    assert_eq!(
        MetalBuiltins::builtin("thread_idx_z"),
        Some("thread_position_in_threadgroup.z")
    );
    assert_eq!(
        MetalBuiltins::builtin("block_idx_x"),
        Some("threadgroup_position_in_grid.x")
    );
    assert_eq!(
        MetalBuiltins::builtin("block_dim_x"),
        Some("threads_per_threadgroup.x")
    );
    assert_eq!(
        MetalBuiltins::builtin("grid_dim_x"),
        Some("threadgroups_per_grid.x")
    );
    assert_eq!(
        MetalBuiltins::builtin("global_idx"),
        Some("thread_position_in_grid.x")
    );
    assert_eq!(
        MetalBuiltins::builtin("global_idx_x"),
        Some("thread_position_in_grid.x")
    );
    assert_eq!(
        MetalBuiltins::builtin("global_idx_y"),
        Some("thread_position_in_grid.y")
    );
    assert_eq!(
        MetalBuiltins::builtin("global_idx_z"),
        Some("thread_position_in_grid.z")
    );
    assert_eq!(MetalBuiltins::builtin("lane_id"), Some("simd_lane_id"));
}

#[test]
fn test_metal_builtins_simd() {
    assert_eq!(MetalBuiltins::builtin("simd_sum"), Some("simd_sum"));
    assert_eq!(MetalBuiltins::builtin("simd_min"), Some("simd_min"));
    assert_eq!(MetalBuiltins::builtin("simd_max"), Some("simd_max"));
    assert_eq!(
        MetalBuiltins::builtin("simd_broadcast"),
        Some("simd_broadcast")
    );
    assert_eq!(MetalBuiltins::builtin("simd_shuffle"), Some("simd_shuffle"));
    assert_eq!(
        MetalBuiltins::builtin("simd_shuffle_down"),
        Some("simd_shuffle_down")
    );
    assert_eq!(
        MetalBuiltins::builtin("simd_shuffle_up"),
        Some("simd_shuffle_up")
    );
    assert_eq!(
        MetalBuiltins::builtin("simd_shuffle_xor"),
        Some("simd_shuffle_xor")
    );
}

#[test]
fn test_metal_builtins_warp_vote() {
    assert_eq!(MetalBuiltins::builtin("warp_all"), Some("simd_all"));
    assert_eq!(MetalBuiltins::builtin("warp_any"), Some("simd_any"));
    assert_eq!(MetalBuiltins::builtin("warp_ballot"), Some("simd_ballot"));
}

#[test]
fn test_metal_builtins_unknown() {
    assert_eq!(MetalBuiltins::builtin("nonexistent"), None);
    assert_eq!(MetalBuiltins::builtin(""), None);
    assert_eq!(MetalBuiltins::builtin("SQRT"), None);
}

// ── type_to_metal tests ──

#[test]
fn test_metal_type_integer_types() {
    let gen = MetalGenerator::new();
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "i64".to_string(),
            generics: vec![]
        }),
        "int64_t"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "i32".to_string(),
            generics: vec![]
        }),
        "int"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "i16".to_string(),
            generics: vec![]
        }),
        "short"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "i8".to_string(),
            generics: vec![]
        }),
        "char"
    );
}

#[test]
fn test_metal_type_unsigned_types() {
    let gen = MetalGenerator::new();
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "u64".to_string(),
            generics: vec![]
        }),
        "uint64_t"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "u32".to_string(),
            generics: vec![]
        }),
        "uint"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "u16".to_string(),
            generics: vec![]
        }),
        "ushort"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "u8".to_string(),
            generics: vec![]
        }),
        "uchar"
    );
}

#[test]
fn test_metal_type_float_types() {
    let gen = MetalGenerator::new();
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "f64".to_string(),
            generics: vec![]
        }),
        "double"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "f32".to_string(),
            generics: vec![]
        }),
        "float"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "f16".to_string(),
            generics: vec![]
        }),
        "half"
    );
}

#[test]
fn test_metal_type_bool() {
    let gen = MetalGenerator::new();
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "bool".to_string(),
            generics: vec![]
        }),
        "bool"
    );
}

#[test]
fn test_metal_type_void() {
    let gen = MetalGenerator::new();
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "unit".to_string(),
            generics: vec![]
        }),
        "void"
    );
}

#[test]
fn test_metal_vector_float_types() {
    let gen = MetalGenerator::new();
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "float2".to_string(),
            generics: vec![]
        }),
        "float2"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "float3".to_string(),
            generics: vec![]
        }),
        "float3"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "float4".to_string(),
            generics: vec![]
        }),
        "float4"
    );
}

#[test]
fn test_metal_vector_vais_aliases() {
    let gen = MetalGenerator::new();
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "Vec2f32".to_string(),
            generics: vec![]
        }),
        "float2"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "Vec3f32".to_string(),
            generics: vec![]
        }),
        "float3"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "Vec4f32".to_string(),
            generics: vec![]
        }),
        "float4"
    );
}

#[test]
fn test_metal_vector_int_types() {
    let gen = MetalGenerator::new();
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "int2".to_string(),
            generics: vec![]
        }),
        "int2"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "int3".to_string(),
            generics: vec![]
        }),
        "int3"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "int4".to_string(),
            generics: vec![]
        }),
        "int4"
    );
}

#[test]
fn test_metal_vector_uint_types() {
    let gen = MetalGenerator::new();
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "uint2".to_string(),
            generics: vec![]
        }),
        "uint2"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "uint3".to_string(),
            generics: vec![]
        }),
        "uint3"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "uint4".to_string(),
            generics: vec![]
        }),
        "uint4"
    );
}

#[test]
fn test_metal_vector_half_types() {
    let gen = MetalGenerator::new();
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "half2".to_string(),
            generics: vec![]
        }),
        "half2"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "half4".to_string(),
            generics: vec![]
        }),
        "half4"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "Vec2f16".to_string(),
            generics: vec![]
        }),
        "half2"
    );
    assert_eq!(
        gen.type_to_metal(&Type::Named {
            name: "Vec4f16".to_string(),
            generics: vec![]
        }),
        "half4"
    );
}

#[test]
fn test_metal_type_pointer() {
    let gen = MetalGenerator::new();
    let inner = Box::new(vais_ast::Spanned::new(
        Type::Named {
            name: "f32".to_string(),
            generics: vec![],
        },
        Default::default(),
    ));
    assert_eq!(gen.type_to_metal(&Type::Pointer(inner)), "device float*");
}

#[test]
fn test_metal_type_const_array() {
    let gen = MetalGenerator::new();
    let elem = Box::new(vais_ast::Spanned::new(
        Type::Named {
            name: "i32".to_string(),
            generics: vec![],
        },
        Default::default(),
    ));
    assert_eq!(
        gen.type_to_metal(&Type::ConstArray {
            element: elem,
            size: 16
        }),
        "int[16]"
    );
}

#[test]
fn test_metal_type_fallback() {
    let gen = MetalGenerator::new();
    assert_eq!(gen.type_to_metal(&Type::Tuple(vec![])), "void");
}

// ── type_to_metal_base tests ──

#[test]
fn test_metal_type_base_primitives() {
    let gen = MetalGenerator::new();
    assert_eq!(
        gen.type_to_metal_base(&Type::Named {
            name: "i64".to_string(),
            generics: vec![]
        }),
        "int64_t"
    );
    assert_eq!(
        gen.type_to_metal_base(&Type::Named {
            name: "i32".to_string(),
            generics: vec![]
        }),
        "int"
    );
    assert_eq!(
        gen.type_to_metal_base(&Type::Named {
            name: "f64".to_string(),
            generics: vec![]
        }),
        "double"
    );
    assert_eq!(
        gen.type_to_metal_base(&Type::Named {
            name: "f32".to_string(),
            generics: vec![]
        }),
        "float"
    );
    assert_eq!(
        gen.type_to_metal_base(&Type::Named {
            name: "bool".to_string(),
            generics: vec![]
        }),
        "bool"
    );
}

// ── vais_type_to_gpu tests ──

#[test]
fn test_metal_vais_type_to_gpu() {
    let gen = MetalGenerator::new();
    assert_eq!(
        gen.vais_type_to_gpu(&Type::Named {
            name: "i32".to_string(),
            generics: vec![]
        }),
        GpuType::I32
    );
    assert_eq!(
        gen.vais_type_to_gpu(&Type::Named {
            name: "i64".to_string(),
            generics: vec![]
        }),
        GpuType::I64
    );
    assert_eq!(
        gen.vais_type_to_gpu(&Type::Named {
            name: "f32".to_string(),
            generics: vec![]
        }),
        GpuType::F32
    );
    assert_eq!(
        gen.vais_type_to_gpu(&Type::Named {
            name: "f64".to_string(),
            generics: vec![]
        }),
        GpuType::F64
    );
    assert_eq!(
        gen.vais_type_to_gpu(&Type::Named {
            name: "bool".to_string(),
            generics: vec![]
        }),
        GpuType::Bool
    );
    assert_eq!(
        gen.vais_type_to_gpu(&Type::Named {
            name: "MyType".to_string(),
            generics: vec![]
        }),
        GpuType::Void
    );
}

// ── extract_block_size tests ──

#[test]
fn test_extract_block_size_default() {
    let gen = MetalGenerator::new();
    let attrs: Vec<vais_ast::Attribute> = vec![];
    assert_eq!(gen.extract_block_size(&attrs), (256, 1, 1));
}

#[test]
fn test_extract_block_size_from_attribute() {
    let gen = MetalGenerator::new();
    let attrs = vec![vais_ast::Attribute {
        name: "thread_block_size".to_string(),
        args: vec!["128".to_string(), "4".to_string(), "2".to_string()],
    }];
    assert_eq!(gen.extract_block_size(&attrs), (128, 4, 2));
}

#[test]
fn test_extract_block_size_single_arg() {
    let gen = MetalGenerator::new();
    let attrs = vec![vais_ast::Attribute {
        name: "threads_per_threadgroup".to_string(),
        args: vec!["512".to_string()],
    }];
    assert_eq!(gen.extract_block_size(&attrs), (512, 1, 1));
}

// ── extract_shared_memory tests ──

#[test]
fn test_extract_shared_memory_default() {
    let gen = MetalGenerator::new();
    let attrs: Vec<vais_ast::Attribute> = vec![];
    assert_eq!(gen.extract_shared_memory(&attrs), 0);
}

#[test]
fn test_extract_shared_memory_from_attribute() {
    let gen = MetalGenerator::new();
    let attrs = vec![vais_ast::Attribute {
        name: "shared_memory".to_string(),
        args: vec!["4096".to_string()],
    }];
    assert_eq!(gen.extract_shared_memory(&attrs), 4096);
}

#[test]
fn test_extract_shared_memory_threadgroup_alias() {
    let gen = MetalGenerator::new();
    let attrs = vec![vais_ast::Attribute {
        name: "threadgroup_memory".to_string(),
        args: vec!["2048".to_string()],
    }];
    assert_eq!(gen.extract_shared_memory(&attrs), 2048);
}

// ── generate_host_code tests ──

#[test]
fn test_metal_host_code_empty() {
    let code = generate_host_code(&[], "Test");
    assert!(code.contains("Metal Host Code (Swift)"));
    assert!(code.contains("import Metal"));
    assert!(code.contains("import MetalKit"));
    assert!(code.contains("class TestKernels"));
    assert!(code.contains("MTLDevice"));
    assert!(code.contains("MTLCommandQueue"));
}

#[test]
fn test_metal_host_code_single_kernel() {
    let kernels = vec![GpuKernel {
        name: "vector_add".to_string(),
        params: vec![
            ("a".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
            ("b".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
            ("c".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
        ],
        shared_memory: 0,
        block_size: (256, 1, 1),
    }];
    let code = generate_host_code(&kernels, "VectorOps");
    assert!(code.contains("class VectorOpsKernels"));
    assert!(code.contains("func launch_vector_add"));
    assert!(code.contains("MTLSize(width: 256, height: 1, depth: 1)"));
    assert!(code.contains("a: MTLBuffer"));
    assert!(code.contains("b: MTLBuffer"));
    assert!(code.contains("c: MTLBuffer"));
    assert!(code.contains("encoder.setBuffer(a, offset: 0, index: 0)"));
    assert!(code.contains("encoder.setBuffer(b, offset: 0, index: 1)"));
    assert!(code.contains("encoder.setBuffer(c, offset: 0, index: 2)"));
}

#[test]
fn test_metal_host_code_custom_block_size() {
    let kernels = vec![GpuKernel {
        name: "matmul".to_string(),
        params: vec![],
        shared_memory: 0,
        block_size: (16, 16, 1),
    }];
    let code = generate_host_code(&kernels, "MatMul");
    assert!(code.contains("MTLSize(width: 16, height: 16, depth: 1)"));
}

#[test]
fn test_metal_host_code_multiple_kernels() {
    let kernels = vec![
        GpuKernel {
            name: "k1".to_string(),
            params: vec![],
            shared_memory: 0,
            block_size: (256, 1, 1),
        },
        GpuKernel {
            name: "k2".to_string(),
            params: vec![],
            shared_memory: 0,
            block_size: (128, 1, 1),
        },
    ];
    let code = generate_host_code(&kernels, "Multi");
    assert!(code.contains("func launch_k1"));
    assert!(code.contains("func launch_k2"));
}

// ── Metal has more builtins than CUDA/OpenCL/WGSL ──

#[test]
fn test_metal_has_extra_atomics() {
    // Metal has atomic_and, atomic_or, atomic_xor, atomic_exch not in other backends
    assert!(MetalBuiltins::builtin("atomic_and").is_some());
    assert!(MetalBuiltins::builtin("atomic_or").is_some());
    assert!(MetalBuiltins::builtin("atomic_xor").is_some());
    assert!(MetalBuiltins::builtin("atomic_exch").is_some());
}

#[test]
fn test_metal_has_extra_math() {
    // Metal has rsqrt, fract, clamp, mix, step, smoothstep, fma not in common builtins
    assert!(MetalBuiltins::builtin("rsqrt").is_some());
    assert!(MetalBuiltins::builtin("fract").is_some());
    assert!(MetalBuiltins::builtin("clamp").is_some());
    assert!(MetalBuiltins::builtin("mix").is_some());
    assert!(MetalBuiltins::builtin("step").is_some());
    assert!(MetalBuiltins::builtin("smoothstep").is_some());
    assert!(MetalBuiltins::builtin("fma").is_some());
}

#[test]
fn test_metal_has_simd_group_ops() {
    // Metal-specific SIMD group operations
    assert!(MetalBuiltins::builtin("simd_sum").is_some());
    assert!(MetalBuiltins::builtin("simd_shuffle").is_some());
    assert!(MetalBuiltins::builtin("warp_all").is_some());
    assert!(MetalBuiltins::builtin("warp_any").is_some());
}
