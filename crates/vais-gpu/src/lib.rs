//! Vais GPU Code Generator
//!
//! Generates GPU compute shader code from Vais AST for various GPU backends.
//!
//! # Supported Backends
//!
//! - **CUDA**: NVIDIA GPU via PTX/CUDA C
//! - **OpenCL**: Cross-platform GPU via OpenCL C
//! - **WebGPU**: Browser-based GPU via WGSL
//!
//! # Usage
//!
//! ```ignore
//! use vais_gpu::{GpuCodeGenerator, GpuTarget};
//!
//! let gen = GpuCodeGenerator::new(GpuTarget::Cuda);
//! let ptx = gen.generate(&module)?;
//! ```

mod common;
#[cfg(feature = "cuda")]
pub mod cuda;
#[cfg(feature = "metal")]
pub mod metal;
#[cfg(feature = "opencl")]
pub mod opencl;
pub mod simd;
#[cfg(feature = "webgpu")]
pub mod webgpu;

use thiserror::Error;
use vais_ast::Module;
use vais_types::ResolvedType;

/// GPU target backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuTarget {
    /// NVIDIA CUDA (generates PTX or CUDA C)
    Cuda,
    /// OpenCL C (cross-platform)
    OpenCL,
    /// WebGPU WGSL (browser-based)
    WebGPU,
    /// Apple Metal Shading Language
    Metal,
}

impl GpuTarget {
    /// Parse target from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cuda" | "ptx" | "nvidia" => {
                #[cfg(feature = "cuda")]
                return Some(Self::Cuda);
                #[cfg(not(feature = "cuda"))]
                return None;
            }
            "opencl" | "cl" => {
                #[cfg(feature = "opencl")]
                return Some(Self::OpenCL);
                #[cfg(not(feature = "opencl"))]
                return None;
            }
            "webgpu" | "wgsl" => {
                #[cfg(feature = "webgpu")]
                return Some(Self::WebGPU);
                #[cfg(not(feature = "webgpu"))]
                return None;
            }
            "metal" | "msl" | "apple" => {
                #[cfg(feature = "metal")]
                return Some(Self::Metal);
                #[cfg(not(feature = "metal"))]
                return None;
            }
            _ => None,
        }
    }

    /// Get file extension for generated code
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Cuda => "cu",
            Self::OpenCL => "cl",
            Self::WebGPU => "wgsl",
            Self::Metal => "metal",
        }
    }

    /// Get target name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Cuda => "CUDA",
            Self::OpenCL => "OpenCL",
            Self::WebGPU => "WebGPU",
            Self::Metal => "Metal",
        }
    }

    /// Check if target is Apple Metal
    pub fn is_metal(&self) -> bool {
        matches!(self, Self::Metal)
    }

    /// Check if target is NVIDIA CUDA
    pub fn is_cuda(&self) -> bool {
        matches!(self, Self::Cuda)
    }

    /// Get recommended shared memory size for target
    pub fn default_shared_memory(&self) -> usize {
        match self {
            Self::Cuda => 48 * 1024,  // 48KB shared memory
            Self::Metal => 32 * 1024, // 32KB threadgroup memory
            Self::OpenCL => 32 * 1024,
            Self::WebGPU => 16 * 1024,
        }
    }
}

/// GPU code generation error
#[derive(Debug, Error)]
pub enum GpuError {
    #[error("Unsupported type for GPU: {0}")]
    UnsupportedType(String),

    #[error("Unsupported operation for GPU: {0}")]
    UnsupportedOperation(String),

    #[error("GPU kernel error: {0}")]
    KernelError(String),

    #[error("Memory error: {0}")]
    MemoryError(String),

    #[error("Backend error: {0}")]
    BackendError(String),
}

pub type GpuResult<T> = Result<T, GpuError>;

/// GPU kernel metadata
#[derive(Debug, Clone)]
pub struct GpuKernel {
    /// Kernel function name
    pub name: String,
    /// Parameter types
    pub params: Vec<(String, GpuType)>,
    /// Local (shared) memory size in bytes
    pub shared_memory: usize,
    /// Suggested block size (threads per block)
    pub block_size: (usize, usize, usize),
}

/// GPU-compatible type
#[derive(Debug, Clone, PartialEq)]
pub enum GpuType {
    /// 32-bit integer
    I32,
    /// 64-bit integer
    I64,
    /// 32-bit float
    F32,
    /// 64-bit float
    F64,
    /// Boolean
    Bool,
    /// Void
    Void,
    /// Pointer to GPU memory
    Ptr(Box<GpuType>),
    /// Fixed-size array
    Array(Box<GpuType>, usize),
    /// Vector type (SIMD)
    Vec(Box<GpuType>, usize),
}

impl GpuType {
    /// Convert from Vais resolved type
    pub fn from_resolved(ty: &ResolvedType) -> GpuResult<Self> {
        match ty {
            ResolvedType::I32 => Ok(GpuType::I32),
            ResolvedType::I64 => Ok(GpuType::I64),
            ResolvedType::F32 => Ok(GpuType::F32),
            ResolvedType::F64 => Ok(GpuType::F64),
            ResolvedType::Bool => Ok(GpuType::Bool),
            ResolvedType::Unit => Ok(GpuType::Void),
            ResolvedType::Pointer(inner) => {
                let inner_ty = GpuType::from_resolved(inner)?;
                Ok(GpuType::Ptr(Box::new(inner_ty)))
            }
            ResolvedType::ConstArray { element, size } => {
                let elem = GpuType::from_resolved(element)?;
                // Try to extract size from ResolvedConst
                if let Some(n) = size.try_evaluate() {
                    Ok(GpuType::Array(Box::new(elem), n as usize))
                } else {
                    Err(GpuError::UnsupportedType("Dynamic array size".to_string()))
                }
            }
            ResolvedType::Vector { element, lanes } => {
                let elem = GpuType::from_resolved(element)?;
                Ok(GpuType::Vec(Box::new(elem), *lanes as usize))
            }
            _ => Err(GpuError::UnsupportedType(format!("{:?}", ty))),
        }
    }

    /// Get type name for CUDA
    pub fn cuda_name(&self) -> String {
        match self {
            GpuType::I32 => "int".to_string(),
            GpuType::I64 => "long long".to_string(),
            GpuType::F32 => "float".to_string(),
            GpuType::F64 => "double".to_string(),
            GpuType::Bool => "bool".to_string(),
            GpuType::Void => "void".to_string(),
            GpuType::Ptr(inner) => format!("{}*", inner.cuda_name()),
            GpuType::Array(elem, size) => format!("{}[{}]", elem.cuda_name(), size),
            GpuType::Vec(elem, n) => format!("{}{}", elem.cuda_name(), n),
        }
    }

    /// Get type name for OpenCL
    pub fn opencl_name(&self) -> String {
        match self {
            GpuType::I32 => "int".to_string(),
            GpuType::I64 => "long".to_string(),
            GpuType::F32 => "float".to_string(),
            GpuType::F64 => "double".to_string(),
            GpuType::Bool => "bool".to_string(),
            GpuType::Void => "void".to_string(),
            GpuType::Ptr(inner) => format!("__global {}*", inner.opencl_name()),
            GpuType::Array(elem, size) => format!("{}[{}]", elem.opencl_name(), size),
            GpuType::Vec(elem, n) => format!("{}{}", elem.opencl_name(), n),
        }
    }

    /// Get type name for WGSL
    pub fn wgsl_name(&self) -> String {
        match self {
            GpuType::I32 => "i32".to_string(),
            GpuType::I64 => "i64".to_string(), // Note: limited support
            GpuType::F32 => "f32".to_string(),
            GpuType::F64 => "f64".to_string(), // Note: limited support
            GpuType::Bool => "bool".to_string(),
            GpuType::Void => "".to_string(),
            GpuType::Ptr(inner) => format!("ptr<storage, {}>", inner.wgsl_name()),
            GpuType::Array(elem, size) => format!("array<{}, {}>", elem.wgsl_name(), size),
            GpuType::Vec(elem, n) => format!("vec{}<{}>", n, elem.wgsl_name()),
        }
    }
}

/// Main GPU code generator
pub struct GpuCodeGenerator {
    target: GpuTarget,
    kernels: Vec<GpuKernel>,
}

impl GpuCodeGenerator {
    /// Create a new GPU code generator for the specified target
    pub fn new(target: GpuTarget) -> Self {
        Self {
            target,
            kernels: Vec::new(),
        }
    }

    /// Get the target backend
    pub fn target(&self) -> GpuTarget {
        self.target
    }

    /// Generate GPU code from a Vais module
    #[allow(unused_variables)]
    pub fn generate(&mut self, module: &Module) -> GpuResult<String> {
        match self.target {
            GpuTarget::Cuda => {
                #[cfg(feature = "cuda")]
                return cuda::generate(module, &mut self.kernels);
                #[cfg(not(feature = "cuda"))]
                return Err(GpuError::BackendError(
                    "CUDA backend not enabled. Enable the 'cuda' feature".to_string(),
                ));
            }
            GpuTarget::OpenCL => {
                #[cfg(feature = "opencl")]
                return opencl::generate(module, &mut self.kernels);
                #[cfg(not(feature = "opencl"))]
                return Err(GpuError::BackendError(
                    "OpenCL backend not enabled. Enable the 'opencl' feature".to_string(),
                ));
            }
            GpuTarget::WebGPU => {
                #[cfg(feature = "webgpu")]
                return webgpu::generate(module, &mut self.kernels);
                #[cfg(not(feature = "webgpu"))]
                return Err(GpuError::BackendError(
                    "WebGPU backend not enabled. Enable the 'webgpu' feature".to_string(),
                ));
            }
            GpuTarget::Metal => {
                #[cfg(feature = "metal")]
                return metal::generate(module, &mut self.kernels);
                #[cfg(not(feature = "metal"))]
                return Err(GpuError::BackendError(
                    "Metal backend not enabled. Enable the 'metal' feature".to_string(),
                ));
            }
        }
    }

    /// Generate host code for launching kernels
    pub fn generate_host_code(&self) -> GpuResult<String> {
        match self.target {
            GpuTarget::Cuda => {
                #[cfg(feature = "cuda")]
                return Ok(cuda::generate_host_code(&self.kernels));
                #[cfg(not(feature = "cuda"))]
                return Err(GpuError::BackendError(
                    "CUDA backend not enabled. Enable the 'cuda' feature".to_string(),
                ));
            }
            GpuTarget::OpenCL => {
                #[cfg(feature = "opencl")]
                return Ok(opencl::generate_host_code(&self.kernels));
                #[cfg(not(feature = "opencl"))]
                return Err(GpuError::BackendError(
                    "OpenCL backend not enabled. Enable the 'opencl' feature".to_string(),
                ));
            }
            GpuTarget::WebGPU => {
                #[cfg(feature = "webgpu")]
                return Ok(webgpu::generate_host_code(&self.kernels, "Main"));
                #[cfg(not(feature = "webgpu"))]
                return Err(GpuError::BackendError(
                    "WebGPU backend not enabled. Enable the 'webgpu' feature".to_string(),
                ));
            }
            GpuTarget::Metal => {
                #[cfg(feature = "metal")]
                return Ok(metal::generate_host_code(&self.kernels, "Main"));
                #[cfg(not(feature = "metal"))]
                return Err(GpuError::BackendError(
                    "Metal backend not enabled. Enable the 'metal' feature".to_string(),
                ));
            }
        }
    }

    /// Get discovered kernels
    pub fn kernels(&self) -> &[GpuKernel] {
        &self.kernels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── GpuTarget::parse tests ──

    #[test]
    fn test_gpu_target_from_str() {
        #[cfg(feature = "cuda")]
        {
            assert_eq!(GpuTarget::parse("cuda"), Some(GpuTarget::Cuda));
            assert_eq!(GpuTarget::parse("CUDA"), Some(GpuTarget::Cuda));
        }
        #[cfg(feature = "opencl")]
        {
            assert_eq!(GpuTarget::parse("opencl"), Some(GpuTarget::OpenCL));
        }
        #[cfg(feature = "webgpu")]
        {
            assert_eq!(GpuTarget::parse("webgpu"), Some(GpuTarget::WebGPU));
            assert_eq!(GpuTarget::parse("wgsl"), Some(GpuTarget::WebGPU));
        }
        #[cfg(feature = "metal")]
        {
            assert_eq!(GpuTarget::parse("metal"), Some(GpuTarget::Metal));
            assert_eq!(GpuTarget::parse("msl"), Some(GpuTarget::Metal));
        }
        assert_eq!(GpuTarget::parse("unknown"), None);
    }

    #[test]
    fn test_gpu_target_parse_aliases() {
        // CUDA aliases
        #[cfg(feature = "cuda")]
        {
            assert_eq!(GpuTarget::parse("ptx"), Some(GpuTarget::Cuda));
            assert_eq!(GpuTarget::parse("nvidia"), Some(GpuTarget::Cuda));
            assert_eq!(GpuTarget::parse("NVIDIA"), Some(GpuTarget::Cuda));
            assert_eq!(GpuTarget::parse("PTX"), Some(GpuTarget::Cuda));
        }
        // OpenCL aliases
        #[cfg(feature = "opencl")]
        {
            assert_eq!(GpuTarget::parse("cl"), Some(GpuTarget::OpenCL));
            assert_eq!(GpuTarget::parse("CL"), Some(GpuTarget::OpenCL));
        }
        // Metal aliases
        #[cfg(feature = "metal")]
        {
            assert_eq!(GpuTarget::parse("apple"), Some(GpuTarget::Metal));
            assert_eq!(GpuTarget::parse("MSL"), Some(GpuTarget::Metal));
            assert_eq!(GpuTarget::parse("APPLE"), Some(GpuTarget::Metal));
        }
    }

    #[test]
    fn test_gpu_target_parse_unknown_returns_none() {
        assert_eq!(GpuTarget::parse(""), None);
        assert_eq!(GpuTarget::parse("vulkan"), None);
        assert_eq!(GpuTarget::parse("directx"), None);
        assert_eq!(GpuTarget::parse("spirv"), None);
    }

    // ── GpuTarget::extension tests ──

    #[test]
    fn test_gpu_target_all_extensions() {
        assert_eq!(GpuTarget::Cuda.extension(), "cu");
        assert_eq!(GpuTarget::OpenCL.extension(), "cl");
        assert_eq!(GpuTarget::WebGPU.extension(), "wgsl");
        assert_eq!(GpuTarget::Metal.extension(), "metal");
    }

    // ── GpuTarget::name tests ──

    #[test]
    fn test_gpu_target_all_names() {
        assert_eq!(GpuTarget::Cuda.name(), "CUDA");
        assert_eq!(GpuTarget::OpenCL.name(), "OpenCL");
        assert_eq!(GpuTarget::WebGPU.name(), "WebGPU");
        assert_eq!(GpuTarget::Metal.name(), "Metal");
    }

    // ── GpuTarget::is_metal / is_cuda tests ──

    #[test]
    fn test_gpu_target_is_metal() {
        assert!(GpuTarget::Metal.is_metal());
        assert!(!GpuTarget::Cuda.is_metal());
        assert!(!GpuTarget::OpenCL.is_metal());
        assert!(!GpuTarget::WebGPU.is_metal());
    }

    #[test]
    fn test_gpu_target_is_cuda() {
        assert!(GpuTarget::Cuda.is_cuda());
        assert!(!GpuTarget::Metal.is_cuda());
        assert!(!GpuTarget::OpenCL.is_cuda());
        assert!(!GpuTarget::WebGPU.is_cuda());
    }

    // ── GpuTarget::default_shared_memory tests ──

    #[test]
    fn test_gpu_target_shared_memory_all() {
        assert_eq!(GpuTarget::Cuda.default_shared_memory(), 48 * 1024);
        assert_eq!(GpuTarget::Metal.default_shared_memory(), 32 * 1024);
        assert_eq!(GpuTarget::OpenCL.default_shared_memory(), 32 * 1024);
        assert_eq!(GpuTarget::WebGPU.default_shared_memory(), 16 * 1024);
    }

    #[test]
    fn test_gpu_target_cuda_has_largest_shared_memory() {
        let cuda = GpuTarget::Cuda.default_shared_memory();
        assert!(cuda >= GpuTarget::Metal.default_shared_memory());
        assert!(cuda >= GpuTarget::OpenCL.default_shared_memory());
        assert!(cuda >= GpuTarget::WebGPU.default_shared_memory());
    }

    // ── GpuTarget traits ──

    #[test]
    fn test_gpu_target_clone_copy() {
        let target = GpuTarget::Cuda;
        let cloned = target.clone();
        let copied = target;
        assert_eq!(target, cloned);
        assert_eq!(target, copied);
    }

    #[test]
    fn test_gpu_target_equality() {
        assert_eq!(GpuTarget::Cuda, GpuTarget::Cuda);
        assert_ne!(GpuTarget::Cuda, GpuTarget::Metal);
        assert_ne!(GpuTarget::OpenCL, GpuTarget::WebGPU);
    }

    #[test]
    fn test_gpu_target_debug() {
        let s = format!("{:?}", GpuTarget::Cuda);
        assert_eq!(s, "Cuda");
    }

    // ── GpuError tests ──

    #[test]
    fn test_gpu_error_unsupported_type() {
        let err = GpuError::UnsupportedType("HashMap".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Unsupported type for GPU"));
        assert!(msg.contains("HashMap"));
    }

    #[test]
    fn test_gpu_error_unsupported_operation() {
        let err = GpuError::UnsupportedOperation("closures".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Unsupported operation for GPU"));
        assert!(msg.contains("closures"));
    }

    #[test]
    fn test_gpu_error_kernel_error() {
        let err = GpuError::KernelError("too many threads".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("GPU kernel error"));
        assert!(msg.contains("too many threads"));
    }

    #[test]
    fn test_gpu_error_memory_error() {
        let err = GpuError::MemoryError("out of memory".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Memory error"));
        assert!(msg.contains("out of memory"));
    }

    #[test]
    fn test_gpu_error_backend_error() {
        let err = GpuError::BackendError("backend not enabled".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Backend error"));
        assert!(msg.contains("backend not enabled"));
    }

    // ── GpuType tests ──

    #[test]
    fn test_gpu_type_primitive_cuda_names() {
        assert_eq!(GpuType::I32.cuda_name(), "int");
        assert_eq!(GpuType::I64.cuda_name(), "long long");
        assert_eq!(GpuType::F32.cuda_name(), "float");
        assert_eq!(GpuType::F64.cuda_name(), "double");
        assert_eq!(GpuType::Bool.cuda_name(), "bool");
        assert_eq!(GpuType::Void.cuda_name(), "void");
    }

    #[test]
    fn test_gpu_type_primitive_opencl_names() {
        assert_eq!(GpuType::I32.opencl_name(), "int");
        assert_eq!(GpuType::I64.opencl_name(), "long");
        assert_eq!(GpuType::F32.opencl_name(), "float");
        assert_eq!(GpuType::F64.opencl_name(), "double");
        assert_eq!(GpuType::Bool.opencl_name(), "bool");
        assert_eq!(GpuType::Void.opencl_name(), "void");
    }

    #[test]
    fn test_gpu_type_primitive_wgsl_names() {
        assert_eq!(GpuType::I32.wgsl_name(), "i32");
        assert_eq!(GpuType::I64.wgsl_name(), "i64");
        assert_eq!(GpuType::F32.wgsl_name(), "f32");
        assert_eq!(GpuType::F64.wgsl_name(), "f64");
        assert_eq!(GpuType::Bool.wgsl_name(), "bool");
        assert_eq!(GpuType::Void.wgsl_name(), "");
    }

    #[test]
    fn test_gpu_type_ptr_cuda_name() {
        let ptr = GpuType::Ptr(Box::new(GpuType::F32));
        assert_eq!(ptr.cuda_name(), "float*");
    }

    #[test]
    fn test_gpu_type_ptr_opencl_name() {
        let ptr = GpuType::Ptr(Box::new(GpuType::F32));
        assert_eq!(ptr.opencl_name(), "__global float*");
    }

    #[test]
    fn test_gpu_type_ptr_wgsl_name() {
        let ptr = GpuType::Ptr(Box::new(GpuType::F32));
        assert_eq!(ptr.wgsl_name(), "ptr<storage, f32>");
    }

    #[test]
    fn test_gpu_type_nested_ptr() {
        let ptr_ptr = GpuType::Ptr(Box::new(GpuType::Ptr(Box::new(GpuType::I32))));
        assert_eq!(ptr_ptr.cuda_name(), "int**");
    }

    #[test]
    fn test_gpu_type_array_cuda_name() {
        let arr = GpuType::Array(Box::new(GpuType::F32), 16);
        assert_eq!(arr.cuda_name(), "float[16]");
    }

    #[test]
    fn test_gpu_type_array_opencl_name() {
        let arr = GpuType::Array(Box::new(GpuType::I64), 8);
        assert_eq!(arr.opencl_name(), "long[8]");
    }

    #[test]
    fn test_gpu_type_array_wgsl_name() {
        let arr = GpuType::Array(Box::new(GpuType::F32), 4);
        assert_eq!(arr.wgsl_name(), "array<f32, 4>");
    }

    #[test]
    fn test_gpu_type_vec_cuda_name() {
        let vec_ty = GpuType::Vec(Box::new(GpuType::F32), 4);
        assert_eq!(vec_ty.cuda_name(), "float4");
    }

    #[test]
    fn test_gpu_type_vec_opencl_name() {
        let vec_ty = GpuType::Vec(Box::new(GpuType::I32), 8);
        assert_eq!(vec_ty.opencl_name(), "int8");
    }

    #[test]
    fn test_gpu_type_vec_wgsl_name() {
        let vec_ty = GpuType::Vec(Box::new(GpuType::F32), 4);
        assert_eq!(vec_ty.wgsl_name(), "vec4<f32>");
    }

    #[test]
    fn test_gpu_type_from_resolved_primitives() {
        assert_eq!(
            GpuType::from_resolved(&ResolvedType::I32).unwrap(),
            GpuType::I32
        );
        assert_eq!(
            GpuType::from_resolved(&ResolvedType::I64).unwrap(),
            GpuType::I64
        );
        assert_eq!(
            GpuType::from_resolved(&ResolvedType::F32).unwrap(),
            GpuType::F32
        );
        assert_eq!(
            GpuType::from_resolved(&ResolvedType::F64).unwrap(),
            GpuType::F64
        );
        assert_eq!(
            GpuType::from_resolved(&ResolvedType::Bool).unwrap(),
            GpuType::Bool
        );
        assert_eq!(
            GpuType::from_resolved(&ResolvedType::Unit).unwrap(),
            GpuType::Void
        );
    }

    #[test]
    fn test_gpu_type_from_resolved_pointer() {
        let resolved = ResolvedType::Pointer(Box::new(ResolvedType::F32));
        let gpu_type = GpuType::from_resolved(&resolved).unwrap();
        assert_eq!(gpu_type, GpuType::Ptr(Box::new(GpuType::F32)));
    }

    #[test]
    fn test_gpu_type_from_resolved_unsupported() {
        let resolved = ResolvedType::Str;
        let result = GpuType::from_resolved(&resolved);
        assert!(result.is_err());
    }

    #[test]
    fn test_gpu_type_equality() {
        assert_eq!(GpuType::I32, GpuType::I32);
        assert_ne!(GpuType::I32, GpuType::I64);
        assert_ne!(GpuType::F32, GpuType::F64);
    }

    #[test]
    fn test_gpu_type_clone() {
        let ty = GpuType::Ptr(Box::new(GpuType::F32));
        let cloned = ty.clone();
        assert_eq!(ty, cloned);
    }

    // ── GpuKernel tests ──

    #[test]
    fn test_gpu_kernel_construction() {
        let kernel = GpuKernel {
            name: "vector_add".to_string(),
            params: vec![
                ("a".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
                ("b".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
                ("c".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
            ],
            shared_memory: 1024,
            block_size: (256, 1, 1),
        };
        assert_eq!(kernel.name, "vector_add");
        assert_eq!(kernel.params.len(), 3);
        assert_eq!(kernel.shared_memory, 1024);
        assert_eq!(kernel.block_size, (256, 1, 1));
    }

    #[test]
    fn test_gpu_kernel_clone() {
        let kernel = GpuKernel {
            name: "matmul".to_string(),
            params: vec![],
            shared_memory: 0,
            block_size: (16, 16, 1),
        };
        let cloned = kernel.clone();
        assert_eq!(cloned.name, "matmul");
        assert_eq!(cloned.block_size, (16, 16, 1));
    }

    // ── GpuCodeGenerator tests ──

    #[test]
    fn test_gpu_code_generator_new() {
        let gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        assert_eq!(gen.target(), GpuTarget::Cuda);
        assert!(gen.kernels().is_empty());
    }

    #[test]
    fn test_gpu_code_generator_target() {
        let gen = GpuCodeGenerator::new(GpuTarget::Metal);
        assert_eq!(gen.target(), GpuTarget::Metal);
    }

    #[test]
    fn test_gpu_code_generator_empty_kernels() {
        let gen = GpuCodeGenerator::new(GpuTarget::WebGPU);
        assert_eq!(gen.kernels().len(), 0);
    }

    #[test]
    #[cfg(not(feature = "cuda"))]
    fn test_generate_host_code_error_when_backend_disabled() {
        let gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let result = gen.generate_host_code();
        assert!(result.is_err());
        if let Err(GpuError::BackendError(msg)) = result {
            assert!(msg.contains("CUDA backend not enabled"));
            assert!(msg.contains("'cuda' feature"));
        } else {
            panic!("Expected BackendError");
        }
    }

    #[test]
    #[cfg(not(feature = "opencl"))]
    fn test_generate_host_code_opencl_error_when_disabled() {
        let gen = GpuCodeGenerator::new(GpuTarget::OpenCL);
        let result = gen.generate_host_code();
        assert!(result.is_err());
        if let Err(GpuError::BackendError(msg)) = result {
            assert!(msg.contains("OpenCL backend not enabled"));
        } else {
            panic!("Expected BackendError");
        }
    }

    #[test]
    #[cfg(not(feature = "webgpu"))]
    fn test_generate_host_code_webgpu_error_when_disabled() {
        let gen = GpuCodeGenerator::new(GpuTarget::WebGPU);
        let result = gen.generate_host_code();
        assert!(result.is_err());
        if let Err(GpuError::BackendError(msg)) = result {
            assert!(msg.contains("WebGPU backend not enabled"));
        } else {
            panic!("Expected BackendError");
        }
    }

    #[test]
    #[cfg(not(feature = "metal"))]
    fn test_generate_host_code_metal_error_when_disabled() {
        let gen = GpuCodeGenerator::new(GpuTarget::Metal);
        let result = gen.generate_host_code();
        assert!(result.is_err());
        if let Err(GpuError::BackendError(msg)) = result {
            assert!(msg.contains("Metal backend not enabled"));
        } else {
            panic!("Expected BackendError");
        }
    }

    #[test]
    #[cfg(not(feature = "cuda"))]
    fn test_generate_cuda_error_when_disabled() {
        let mut gen = GpuCodeGenerator::new(GpuTarget::Cuda);
        let module = Module {
            items: vec![],
            modules_map: None,
        };
        let result = gen.generate(&module);
        assert!(result.is_err());
    }

    // ── GpuResult tests ──

    #[test]
    fn test_gpu_result_ok() {
        let result: GpuResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_gpu_result_err() {
        let result: GpuResult<i32> = Err(GpuError::UnsupportedType("str".to_string()));
        assert!(result.is_err());
    }
}
