//! GPU code generation tests

use vais_gpu::{GpuCodeGenerator, GpuTarget, GpuType, GpuError};

#[test]
fn test_gpu_target_from_str() {
    assert_eq!(GpuTarget::from_str("cuda"), Some(GpuTarget::Cuda));
    assert_eq!(GpuTarget::from_str("CUDA"), Some(GpuTarget::Cuda));
    assert_eq!(GpuTarget::from_str("ptx"), Some(GpuTarget::Cuda));
    assert_eq!(GpuTarget::from_str("nvidia"), Some(GpuTarget::Cuda));

    assert_eq!(GpuTarget::from_str("opencl"), Some(GpuTarget::OpenCL));
    assert_eq!(GpuTarget::from_str("cl"), Some(GpuTarget::OpenCL));

    assert_eq!(GpuTarget::from_str("webgpu"), Some(GpuTarget::WebGPU));
    assert_eq!(GpuTarget::from_str("wgsl"), Some(GpuTarget::WebGPU));

    assert_eq!(GpuTarget::from_str("unknown"), None);
    assert_eq!(GpuTarget::from_str(""), None);
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

mod common_tests {
    use vais_gpu::cuda;
    use vais_gpu::opencl;
    use vais_gpu::webgpu;

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
}
