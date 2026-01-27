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

pub mod cuda;
pub mod opencl;
pub mod webgpu;
pub mod metal;
pub mod simd;
mod common;

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
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cuda" | "ptx" | "nvidia" => Some(Self::Cuda),
            "opencl" | "cl" => Some(Self::OpenCL),
            "webgpu" | "wgsl" => Some(Self::WebGPU),
            "metal" | "msl" | "apple" => Some(Self::Metal),
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
    pub fn generate(&mut self, module: &Module) -> GpuResult<String> {
        match self.target {
            GpuTarget::Cuda => cuda::generate(module, &mut self.kernels),
            GpuTarget::OpenCL => opencl::generate(module, &mut self.kernels),
            GpuTarget::WebGPU => webgpu::generate(module, &mut self.kernels),
            GpuTarget::Metal => metal::generate(module, &mut self.kernels),
        }
    }

    /// Generate host code for launching kernels
    pub fn generate_host_code(&self) -> String {
        match self.target {
            GpuTarget::Cuda => cuda::generate_host_code(&self.kernels),
            GpuTarget::OpenCL => opencl::generate_host_code(&self.kernels),
            GpuTarget::WebGPU => webgpu::generate_host_code(&self.kernels, "Main"),
            GpuTarget::Metal => metal::generate_host_code(&self.kernels, "Main"),
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

    #[test]
    fn test_gpu_target_from_str() {
        assert_eq!(GpuTarget::from_str("cuda"), Some(GpuTarget::Cuda));
        assert_eq!(GpuTarget::from_str("CUDA"), Some(GpuTarget::Cuda));
        assert_eq!(GpuTarget::from_str("opencl"), Some(GpuTarget::OpenCL));
        assert_eq!(GpuTarget::from_str("webgpu"), Some(GpuTarget::WebGPU));
        assert_eq!(GpuTarget::from_str("wgsl"), Some(GpuTarget::WebGPU));
        assert_eq!(GpuTarget::from_str("metal"), Some(GpuTarget::Metal));
        assert_eq!(GpuTarget::from_str("msl"), Some(GpuTarget::Metal));
        assert_eq!(GpuTarget::from_str("unknown"), None);
    }

    #[test]
    fn test_gpu_type_names() {
        let i64_ty = GpuType::I64;
        assert_eq!(i64_ty.cuda_name(), "long long");
        assert_eq!(i64_ty.opencl_name(), "long");
        assert_eq!(i64_ty.wgsl_name(), "i64");

        let ptr_ty = GpuType::Ptr(Box::new(GpuType::F32));
        assert_eq!(ptr_ty.cuda_name(), "float*");
        assert_eq!(ptr_ty.opencl_name(), "__global float*");
    }

    #[test]
    fn test_gpu_target_extension() {
        assert_eq!(GpuTarget::Cuda.extension(), "cu");
        assert_eq!(GpuTarget::OpenCL.extension(), "cl");
        assert_eq!(GpuTarget::WebGPU.extension(), "wgsl");
        assert_eq!(GpuTarget::Metal.extension(), "metal");
    }

    #[test]
    fn test_gpu_target_methods() {
        assert!(GpuTarget::Metal.is_metal());
        assert!(!GpuTarget::Cuda.is_metal());
        assert!(GpuTarget::Cuda.is_cuda());
        assert!(!GpuTarget::Metal.is_cuda());
    }

    #[test]
    fn test_gpu_target_shared_memory() {
        assert_eq!(GpuTarget::Cuda.default_shared_memory(), 48 * 1024);
        assert_eq!(GpuTarget::Metal.default_shared_memory(), 32 * 1024);
    }
}
