//! GPU code generation benchmarks
//!
//! Measures code generation performance for matrix multiplication and vector operations
//! across CUDA, OpenCL, Metal, and WebGPU backends.
//!
//! NOTE: These benchmarks measure code generation time only, not actual GPU execution.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use vais_ast::{
    Attribute, Expr, Function, FunctionBody, Item, Module, Param, Span, Spanned, Stmt, Type,
};
use vais_gpu::{GpuCodeGenerator, GpuKernel, GpuTarget, GpuType};

/// Create a matrix multiplication kernel for benchmarking
#[allow(dead_code)]
fn create_matmul_kernel(_size: usize) -> GpuKernel {
    GpuKernel {
        name: "matmul".to_string(),
        params: vec![
            ("a".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
            ("b".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
            ("c".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
            ("n".to_string(), GpuType::I32),
        ],
        shared_memory: 48 * 1024, // 48KB shared memory
        block_size: (16, 16, 1),
    }
}

/// Create a vector addition kernel for benchmarking
#[allow(dead_code)]
fn create_vector_add_kernel(_size: usize) -> GpuKernel {
    GpuKernel {
        name: "vector_add".to_string(),
        params: vec![
            ("a".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
            ("b".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
            ("c".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
            ("n".to_string(), GpuType::I32),
        ],
        shared_memory: 0,
        block_size: (256, 1, 1),
    }
}

/// Create a reduction kernel for benchmarking
#[allow(dead_code)]
fn create_reduction_kernel(_size: usize) -> GpuKernel {
    GpuKernel {
        name: "reduce_sum".to_string(),
        params: vec![
            ("input".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
            ("output".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
            ("n".to_string(), GpuType::I32),
        ],
        shared_memory: 16 * 1024, // 16KB for reduction tree
        block_size: (256, 1, 1),
    }
}

/// Create a convolution kernel for benchmarking
#[allow(dead_code)]
fn create_conv2d_kernel(_size: usize) -> GpuKernel {
    GpuKernel {
        name: "conv2d".to_string(),
        params: vec![
            ("input".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
            ("kernel".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
            ("output".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
            ("width".to_string(), GpuType::I32),
            ("height".to_string(), GpuType::I32),
            ("kernel_size".to_string(), GpuType::I32),
        ],
        shared_memory: 32 * 1024, // 32KB for tile
        block_size: (16, 16, 1),
    }
}

/// Create a simple Vais module with a GPU kernel function
fn create_vais_gpu_module(kernel_name: &str) -> Module {
    let dummy_span = Span::new(0, 0);

    // Helper to create f32 type
    let f32_type = Type::Named {
        name: "f32".to_string(),
        generics: vec![],
    };

    // Helper to create i32 type
    let i32_type = Type::Named {
        name: "i32".to_string(),
        generics: vec![],
    };

    let func = Function {
        name: Spanned::new(kernel_name.to_string(), dummy_span),
        generics: vec![],
        params: vec![
            Param {
                name: Spanned::new("a".to_string(), dummy_span),
                ty: Spanned::new(
                    Type::Pointer(Box::new(Spanned::new(f32_type.clone(), dummy_span))),
                    dummy_span,
                ),
                is_mut: false,
                is_vararg: false,
                ownership: vais_ast::Ownership::Regular,
                default_value: None,
            },
            Param {
                name: Spanned::new("b".to_string(), dummy_span),
                ty: Spanned::new(
                    Type::Pointer(Box::new(Spanned::new(f32_type.clone(), dummy_span))),
                    dummy_span,
                ),
                is_mut: false,
                is_vararg: false,
                ownership: vais_ast::Ownership::Regular,
                default_value: None,
            },
            Param {
                name: Spanned::new("c".to_string(), dummy_span),
                ty: Spanned::new(
                    Type::Pointer(Box::new(Spanned::new(f32_type.clone(), dummy_span))),
                    dummy_span,
                ),
                is_mut: false,
                is_vararg: false,
                ownership: vais_ast::Ownership::Regular,
                default_value: None,
            },
            Param {
                name: Spanned::new("n".to_string(), dummy_span),
                ty: Spanned::new(i32_type, dummy_span),
                is_mut: false,
                is_vararg: false,
                ownership: vais_ast::Ownership::Regular,
                default_value: None,
            },
        ],
        ret_type: Some(Spanned::new(Type::Unit, dummy_span)),
        body: FunctionBody::Block(vec![Spanned::new(
            Stmt::Expr(Box::new(Spanned::new(Expr::Int(0), dummy_span))),
            dummy_span,
        )]),
        is_pub: false,
        is_async: false,
        attributes: vec![Attribute {
            name: "gpu".to_string(),
            args: vec![],
            expr: None,
        }],
    };

    Module {
        items: vec![Spanned::new(Item::Function(func), dummy_span)],
        modules_map: None,
    }
}

/// Benchmark: Matrix multiplication code generation across all GPU targets
fn bench_matmul_codegen(c: &mut Criterion) {
    let mut group = c.benchmark_group("matmul_codegen");

    let sizes = [64, 256, 1024];
    let targets = [
        GpuTarget::Cuda,
        GpuTarget::OpenCL,
        GpuTarget::WebGPU,
        GpuTarget::Metal,
    ];

    for target in targets {
        for &size in &sizes {
            let module = create_vais_gpu_module("matmul");

            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(
                BenchmarkId::new(target.name(), format!("{}x{}", size, size)),
                &(target, size),
                |b, &(target, _size)| {
                    b.iter(|| {
                        let mut gen = GpuCodeGenerator::new(target);
                        gen.generate(black_box(&module))
                    })
                },
            );
        }
    }

    group.finish();
}

/// Benchmark: Vector operations code generation
fn bench_vector_add_codegen(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_add_codegen");

    let sizes = [1024, 65536, 1048576]; // 1K, 64K, 1M elements
    let targets = [
        GpuTarget::Cuda,
        GpuTarget::OpenCL,
        GpuTarget::WebGPU,
        GpuTarget::Metal,
    ];

    for target in targets {
        for &size in &sizes {
            let module = create_vais_gpu_module("vector_add");

            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(
                BenchmarkId::new(target.name(), size),
                &(target, size),
                |b, &(target, _size)| {
                    b.iter(|| {
                        let mut gen = GpuCodeGenerator::new(target);
                        gen.generate(black_box(&module))
                    })
                },
            );
        }
    }

    group.finish();
}

/// Benchmark: Reduction kernel code generation
fn bench_reduction_codegen(c: &mut Criterion) {
    let mut group = c.benchmark_group("reduction_codegen");

    let sizes = [1024, 65536, 1048576];
    let targets = [
        GpuTarget::Cuda,
        GpuTarget::OpenCL,
        GpuTarget::WebGPU,
        GpuTarget::Metal,
    ];

    for target in targets {
        for &size in &sizes {
            let module = create_vais_gpu_module("reduce_sum");

            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(
                BenchmarkId::new(target.name(), size),
                &(target, size),
                |b, &(target, _size)| {
                    b.iter(|| {
                        let mut gen = GpuCodeGenerator::new(target);
                        gen.generate(black_box(&module))
                    })
                },
            );
        }
    }

    group.finish();
}

/// Benchmark: Convolution kernel code generation
fn bench_conv2d_codegen(c: &mut Criterion) {
    let mut group = c.benchmark_group("conv2d_codegen");

    let sizes = [32, 128, 512]; // Image sizes (32x32, 128x128, 512x512)
    let targets = [
        GpuTarget::Cuda,
        GpuTarget::OpenCL,
        GpuTarget::WebGPU,
        GpuTarget::Metal,
    ];

    for target in targets {
        for &size in &sizes {
            let module = create_vais_gpu_module("conv2d");

            group.throughput(Throughput::Elements((size * size) as u64));
            group.bench_with_input(
                BenchmarkId::new(target.name(), format!("{}x{}", size, size)),
                &(target, size),
                |b, &(target, _size)| {
                    b.iter(|| {
                        let mut gen = GpuCodeGenerator::new(target);
                        gen.generate(black_box(&module))
                    })
                },
            );
        }
    }

    group.finish();
}

/// Benchmark: Memory operation code generation
fn bench_memory_ops_codegen(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_ops");

    let targets = [
        GpuTarget::Cuda,
        GpuTarget::OpenCL,
        GpuTarget::WebGPU,
        GpuTarget::Metal,
    ];

    for target in targets {
        // Simple memory copy kernel
        let module = create_vais_gpu_module("memcpy");

        group.bench_with_input(
            BenchmarkId::new("memcpy", target.name()),
            &target,
            |b, &target| {
                b.iter(|| {
                    let mut gen = GpuCodeGenerator::new(target);
                    gen.generate(black_box(&module))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Host code generation (kernel launch wrappers)
fn bench_host_code_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("host_code_gen");

    let targets = [
        GpuTarget::Cuda,
        GpuTarget::OpenCL,
        GpuTarget::WebGPU,
        GpuTarget::Metal,
    ];

    for target in targets {
        let module = create_vais_gpu_module("matmul");

        group.bench_with_input(
            BenchmarkId::new("host_wrapper", target.name()),
            &target,
            |b, &target| {
                b.iter(|| {
                    let mut gen = GpuCodeGenerator::new(target);
                    // Generate module first to populate kernels
                    let _ = gen.generate(&module);
                    // Then generate host code
                    gen.generate_host_code()
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Type conversion overhead
fn bench_type_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("type_conversion");

    let types = vec![
        ("f32", GpuType::F32),
        ("f64", GpuType::F64),
        ("i32", GpuType::I32),
        ("i64", GpuType::I64),
        ("ptr_f32", GpuType::Ptr(Box::new(GpuType::F32))),
        (
            "array_f32_1024",
            GpuType::Array(Box::new(GpuType::F32), 1024),
        ),
        ("vec4_f32", GpuType::Vec(Box::new(GpuType::F32), 4)),
    ];

    let targets = [
        GpuTarget::Cuda,
        GpuTarget::OpenCL,
        GpuTarget::WebGPU,
        GpuTarget::Metal,
    ];

    for (type_name, gpu_type) in &types {
        for target in &targets {
            group.bench_with_input(
                BenchmarkId::new(target.name().to_string(), type_name),
                &(gpu_type, target),
                |b, (ty, target)| {
                    b.iter(|| match target {
                        GpuTarget::Cuda => black_box(ty).cuda_name(),
                        GpuTarget::OpenCL => black_box(ty).opencl_name(),
                        GpuTarget::WebGPU | GpuTarget::Metal => black_box(ty).wgsl_name(),
                    })
                },
            );
        }
    }

    group.finish();
}

/// Benchmark: Full compilation pipeline (module generation)
fn bench_full_gpu_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");

    let targets = [
        GpuTarget::Cuda,
        GpuTarget::OpenCL,
        GpuTarget::WebGPU,
        GpuTarget::Metal,
    ];

    // Create a more complex module with multiple kernels
    let complex_module = {
        let dummy_span = Span::new(0, 0);
        let f32_type = Type::Named {
            name: "f32".to_string(),
            generics: vec![],
        };
        let i32_type = Type::Named {
            name: "i32".to_string(),
            generics: vec![],
        };

        let mut items = Vec::new();

        // Add 5 different kernel functions
        for kernel_name in ["matmul", "vector_add", "reduce", "transpose", "conv2d"] {
            let func = Function {
                name: Spanned::new(kernel_name.to_string(), dummy_span),
                generics: vec![],
                params: vec![
                    Param {
                        name: Spanned::new("input".to_string(), dummy_span),
                        ty: Spanned::new(
                            Type::Pointer(Box::new(Spanned::new(f32_type.clone(), dummy_span))),
                            dummy_span,
                        ),
                        is_mut: false,
                        is_vararg: false,
                        ownership: vais_ast::Ownership::Regular,
                        default_value: None,
                    },
                    Param {
                        name: Spanned::new("output".to_string(), dummy_span),
                        ty: Spanned::new(
                            Type::Pointer(Box::new(Spanned::new(f32_type.clone(), dummy_span))),
                            dummy_span,
                        ),
                        is_mut: false,
                        is_vararg: false,
                        ownership: vais_ast::Ownership::Regular,
                        default_value: None,
                    },
                    Param {
                        name: Spanned::new("n".to_string(), dummy_span),
                        ty: Spanned::new(i32_type.clone(), dummy_span),
                        is_mut: false,
                        is_vararg: false,
                        ownership: vais_ast::Ownership::Regular,
                        default_value: None,
                    },
                ],
                ret_type: Some(Spanned::new(Type::Unit, dummy_span)),
                body: FunctionBody::Block(vec![Spanned::new(
                    Stmt::Expr(Box::new(Spanned::new(Expr::Int(0), dummy_span))),
                    dummy_span,
                )]),
                is_pub: false,
                is_async: false,
                attributes: vec![Attribute {
                    name: "gpu".to_string(),
                    args: vec![],
                    expr: None,
                }],
            };
            items.push(Spanned::new(Item::Function(func), dummy_span));
        }

        Module {
            items,
            modules_map: None,
        }
    };

    for target in targets {
        group.bench_with_input(
            BenchmarkId::new("multi_kernel", target.name()),
            &target,
            |b, &target| {
                b.iter(|| {
                    let mut gen = GpuCodeGenerator::new(target);
                    gen.generate(black_box(&complex_module))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Backend comparison (same kernel, all backends)
fn bench_backend_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("backend_comparison");

    let module = create_vais_gpu_module("matmul");
    let targets = [
        GpuTarget::Cuda,
        GpuTarget::OpenCL,
        GpuTarget::WebGPU,
        GpuTarget::Metal,
    ];

    // Benchmark each backend for the same kernel
    for target in targets {
        group.bench_function(target.name(), |b| {
            b.iter(|| {
                let mut gen = GpuCodeGenerator::new(target);
                gen.generate(black_box(&module))
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_matmul_codegen,
    bench_vector_add_codegen,
    bench_reduction_codegen,
    bench_conv2d_codegen,
    bench_memory_ops_codegen,
    bench_host_code_generation,
    bench_type_conversion,
    bench_full_gpu_pipeline,
    bench_backend_comparison,
);

criterion_main!(benches);
