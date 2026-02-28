#![allow(dead_code)] // GPU common utilities reserved for backend use
//! Common utilities for GPU code generation

use vais_ast::{BinOp, UnaryOp};

/// Convert Vais binary operator to GPU operator string
pub fn binary_op_str(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Mod => "%",
        BinOp::And => "&&",
        BinOp::Or => "||",
        BinOp::BitAnd => "&",
        BinOp::BitOr => "|",
        BinOp::BitXor => "^",
        BinOp::Shl => "<<",
        BinOp::Shr => ">>",
        BinOp::Eq => "==",
        BinOp::Neq => "!=",
        BinOp::Lt => "<",
        BinOp::Lte => "<=",
        BinOp::Gt => ">",
        BinOp::Gte => ">=",
    }
}

/// Convert Vais unary operator to GPU operator string
pub fn unary_op_str(op: &UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "-",
        UnaryOp::Not => "!",
        UnaryOp::BitNot => "~",
    }
}

/// GPU built-in functions mapping
pub struct GpuBuiltins;

impl GpuBuiltins {
    /// Get CUDA equivalent of a built-in function
    pub fn cuda_builtin(name: &str) -> Option<&'static str> {
        match name {
            // Math functions
            "sqrt" => Some("sqrt"),
            "sin" => Some("sin"),
            "cos" => Some("cos"),
            "tan" => Some("tan"),
            "exp" => Some("exp"),
            "log" => Some("log"),
            "pow" => Some("pow"),
            "abs" => Some("abs"),
            "floor" => Some("floor"),
            "ceil" => Some("ceil"),
            "round" => Some("round"),
            "min" => Some("min"),
            "max" => Some("max"),
            // Atomic operations
            "atomic_add" => Some("atomicAdd"),
            "atomic_sub" => Some("atomicSub"),
            "atomic_min" => Some("atomicMin"),
            "atomic_max" => Some("atomicMax"),
            "atomic_cas" => Some("atomicCAS"),
            // Synchronization
            "sync_threads" => Some("__syncthreads"),
            "thread_fence" => Some("__threadfence"),
            // Thread indexing
            "thread_idx_x" => Some("threadIdx.x"),
            "thread_idx_y" => Some("threadIdx.y"),
            "thread_idx_z" => Some("threadIdx.z"),
            "block_idx_x" => Some("blockIdx.x"),
            "block_idx_y" => Some("blockIdx.y"),
            "block_idx_z" => Some("blockIdx.z"),
            "block_dim_x" => Some("blockDim.x"),
            "block_dim_y" => Some("blockDim.y"),
            "block_dim_z" => Some("blockDim.z"),
            "grid_dim_x" => Some("gridDim.x"),
            "grid_dim_y" => Some("gridDim.y"),
            "grid_dim_z" => Some("gridDim.z"),
            "global_idx" => Some("(blockIdx.x * blockDim.x + threadIdx.x)"),
            _ => None,
        }
    }

    /// Get OpenCL equivalent of a built-in function
    pub fn opencl_builtin(name: &str) -> Option<&'static str> {
        match name {
            // Math functions
            "sqrt" => Some("sqrt"),
            "sin" => Some("sin"),
            "cos" => Some("cos"),
            "tan" => Some("tan"),
            "exp" => Some("exp"),
            "log" => Some("log"),
            "pow" => Some("pow"),
            "abs" => Some("fabs"),
            "floor" => Some("floor"),
            "ceil" => Some("ceil"),
            "round" => Some("round"),
            "min" => Some("min"),
            "max" => Some("max"),
            // Atomic operations
            "atomic_add" => Some("atomic_add"),
            "atomic_sub" => Some("atomic_sub"),
            "atomic_min" => Some("atomic_min"),
            "atomic_max" => Some("atomic_max"),
            "atomic_cas" => Some("atomic_cmpxchg"),
            // Synchronization
            "sync_threads" => Some("barrier(CLK_LOCAL_MEM_FENCE)"),
            "thread_fence" => Some("mem_fence(CLK_GLOBAL_MEM_FENCE)"),
            // Thread indexing
            "thread_idx_x" => Some("get_local_id(0)"),
            "thread_idx_y" => Some("get_local_id(1)"),
            "thread_idx_z" => Some("get_local_id(2)"),
            "block_idx_x" => Some("get_group_id(0)"),
            "block_idx_y" => Some("get_group_id(1)"),
            "block_idx_z" => Some("get_group_id(2)"),
            "block_dim_x" => Some("get_local_size(0)"),
            "block_dim_y" => Some("get_local_size(1)"),
            "block_dim_z" => Some("get_local_size(2)"),
            "grid_dim_x" => Some("get_num_groups(0)"),
            "grid_dim_y" => Some("get_num_groups(1)"),
            "grid_dim_z" => Some("get_num_groups(2)"),
            "global_idx" => Some("get_global_id(0)"),
            _ => None,
        }
    }

    /// Get WGSL equivalent of a built-in function
    pub fn wgsl_builtin(name: &str) -> Option<&'static str> {
        match name {
            // Math functions
            "sqrt" => Some("sqrt"),
            "sin" => Some("sin"),
            "cos" => Some("cos"),
            "tan" => Some("tan"),
            "exp" => Some("exp"),
            "log" => Some("log"),
            "pow" => Some("pow"),
            "abs" => Some("abs"),
            "floor" => Some("floor"),
            "ceil" => Some("ceil"),
            "round" => Some("round"),
            "min" => Some("min"),
            "max" => Some("max"),
            // Atomic operations
            "atomic_add" => Some("atomicAdd"),
            "atomic_sub" => Some("atomicSub"),
            "atomic_min" => Some("atomicMin"),
            "atomic_max" => Some("atomicMax"),
            "atomic_cas" => Some("atomicCompareExchangeWeak"),
            // Synchronization
            "sync_threads" => Some("workgroupBarrier()"),
            "thread_fence" => Some("storageBarrier()"),
            // Thread indexing
            "thread_idx_x" => Some("local_invocation_id.x"),
            "thread_idx_y" => Some("local_invocation_id.y"),
            "thread_idx_z" => Some("local_invocation_id.z"),
            "block_idx_x" => Some("workgroup_id.x"),
            "block_idx_y" => Some("workgroup_id.y"),
            "block_idx_z" => Some("workgroup_id.z"),
            "block_dim_x" => Some("workgroup_size.x"),
            "block_dim_y" => Some("workgroup_size.y"),
            "block_dim_z" => Some("workgroup_size.z"),
            "grid_dim_x" => Some("num_workgroups.x"),
            "grid_dim_y" => Some("num_workgroups.y"),
            "grid_dim_z" => Some("num_workgroups.z"),
            "global_idx" => Some("global_invocation_id.x"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── binary_op_str tests ──

    #[test]
    fn test_binary_op_str_arithmetic() {
        assert_eq!(binary_op_str(&BinOp::Add), "+");
        assert_eq!(binary_op_str(&BinOp::Sub), "-");
        assert_eq!(binary_op_str(&BinOp::Mul), "*");
        assert_eq!(binary_op_str(&BinOp::Div), "/");
        assert_eq!(binary_op_str(&BinOp::Mod), "%");
    }

    #[test]
    fn test_binary_op_str_logical() {
        assert_eq!(binary_op_str(&BinOp::And), "&&");
        assert_eq!(binary_op_str(&BinOp::Or), "||");
    }

    #[test]
    fn test_binary_op_str_bitwise() {
        assert_eq!(binary_op_str(&BinOp::BitAnd), "&");
        assert_eq!(binary_op_str(&BinOp::BitOr), "|");
        assert_eq!(binary_op_str(&BinOp::BitXor), "^");
        assert_eq!(binary_op_str(&BinOp::Shl), "<<");
        assert_eq!(binary_op_str(&BinOp::Shr), ">>");
    }

    #[test]
    fn test_binary_op_str_comparison() {
        assert_eq!(binary_op_str(&BinOp::Eq), "==");
        assert_eq!(binary_op_str(&BinOp::Neq), "!=");
        assert_eq!(binary_op_str(&BinOp::Lt), "<");
        assert_eq!(binary_op_str(&BinOp::Lte), "<=");
        assert_eq!(binary_op_str(&BinOp::Gt), ">");
        assert_eq!(binary_op_str(&BinOp::Gte), ">=");
    }

    // ── unary_op_str tests ──

    #[test]
    fn test_unary_op_str_all() {
        assert_eq!(unary_op_str(&UnaryOp::Neg), "-");
        assert_eq!(unary_op_str(&UnaryOp::Not), "!");
        assert_eq!(unary_op_str(&UnaryOp::BitNot), "~");
    }

    // ── CUDA builtins tests ──

    #[test]
    fn test_cuda_builtins_math() {
        assert_eq!(GpuBuiltins::cuda_builtin("sqrt"), Some("sqrt"));
        assert_eq!(GpuBuiltins::cuda_builtin("sin"), Some("sin"));
        assert_eq!(GpuBuiltins::cuda_builtin("cos"), Some("cos"));
        assert_eq!(GpuBuiltins::cuda_builtin("tan"), Some("tan"));
        assert_eq!(GpuBuiltins::cuda_builtin("exp"), Some("exp"));
        assert_eq!(GpuBuiltins::cuda_builtin("log"), Some("log"));
        assert_eq!(GpuBuiltins::cuda_builtin("pow"), Some("pow"));
        assert_eq!(GpuBuiltins::cuda_builtin("abs"), Some("abs"));
        assert_eq!(GpuBuiltins::cuda_builtin("floor"), Some("floor"));
        assert_eq!(GpuBuiltins::cuda_builtin("ceil"), Some("ceil"));
        assert_eq!(GpuBuiltins::cuda_builtin("round"), Some("round"));
        assert_eq!(GpuBuiltins::cuda_builtin("min"), Some("min"));
        assert_eq!(GpuBuiltins::cuda_builtin("max"), Some("max"));
    }

    #[test]
    fn test_cuda_builtins_atomics() {
        assert_eq!(GpuBuiltins::cuda_builtin("atomic_add"), Some("atomicAdd"));
        assert_eq!(GpuBuiltins::cuda_builtin("atomic_sub"), Some("atomicSub"));
        assert_eq!(GpuBuiltins::cuda_builtin("atomic_min"), Some("atomicMin"));
        assert_eq!(GpuBuiltins::cuda_builtin("atomic_max"), Some("atomicMax"));
        assert_eq!(GpuBuiltins::cuda_builtin("atomic_cas"), Some("atomicCAS"));
    }

    #[test]
    fn test_cuda_builtins_sync() {
        assert_eq!(
            GpuBuiltins::cuda_builtin("sync_threads"),
            Some("__syncthreads")
        );
        assert_eq!(
            GpuBuiltins::cuda_builtin("thread_fence"),
            Some("__threadfence")
        );
    }

    #[test]
    fn test_cuda_builtins_thread_indexing() {
        assert_eq!(
            GpuBuiltins::cuda_builtin("thread_idx_x"),
            Some("threadIdx.x")
        );
        assert_eq!(
            GpuBuiltins::cuda_builtin("thread_idx_y"),
            Some("threadIdx.y")
        );
        assert_eq!(
            GpuBuiltins::cuda_builtin("thread_idx_z"),
            Some("threadIdx.z")
        );
        assert_eq!(
            GpuBuiltins::cuda_builtin("block_idx_x"),
            Some("blockIdx.x")
        );
        assert_eq!(
            GpuBuiltins::cuda_builtin("block_idx_y"),
            Some("blockIdx.y")
        );
        assert_eq!(
            GpuBuiltins::cuda_builtin("block_idx_z"),
            Some("blockIdx.z")
        );
        assert_eq!(
            GpuBuiltins::cuda_builtin("block_dim_x"),
            Some("blockDim.x")
        );
        assert_eq!(
            GpuBuiltins::cuda_builtin("block_dim_y"),
            Some("blockDim.y")
        );
        assert_eq!(
            GpuBuiltins::cuda_builtin("block_dim_z"),
            Some("blockDim.z")
        );
        assert_eq!(GpuBuiltins::cuda_builtin("grid_dim_x"), Some("gridDim.x"));
        assert_eq!(GpuBuiltins::cuda_builtin("grid_dim_y"), Some("gridDim.y"));
        assert_eq!(GpuBuiltins::cuda_builtin("grid_dim_z"), Some("gridDim.z"));
    }

    #[test]
    fn test_cuda_builtins_global_idx() {
        assert_eq!(
            GpuBuiltins::cuda_builtin("global_idx"),
            Some("(blockIdx.x * blockDim.x + threadIdx.x)")
        );
    }

    #[test]
    fn test_cuda_builtins_unknown_returns_none() {
        assert_eq!(GpuBuiltins::cuda_builtin("nonexistent"), None);
        assert_eq!(GpuBuiltins::cuda_builtin(""), None);
        assert_eq!(GpuBuiltins::cuda_builtin("SQRT"), None);
    }

    // ── OpenCL builtins tests ──

    #[test]
    fn test_opencl_builtins_math() {
        assert_eq!(GpuBuiltins::opencl_builtin("sqrt"), Some("sqrt"));
        assert_eq!(GpuBuiltins::opencl_builtin("sin"), Some("sin"));
        assert_eq!(GpuBuiltins::opencl_builtin("cos"), Some("cos"));
        assert_eq!(GpuBuiltins::opencl_builtin("abs"), Some("fabs"));
        assert_eq!(GpuBuiltins::opencl_builtin("min"), Some("min"));
        assert_eq!(GpuBuiltins::opencl_builtin("max"), Some("max"));
    }

    #[test]
    fn test_opencl_builtins_atomics() {
        assert_eq!(
            GpuBuiltins::opencl_builtin("atomic_add"),
            Some("atomic_add")
        );
        assert_eq!(
            GpuBuiltins::opencl_builtin("atomic_sub"),
            Some("atomic_sub")
        );
        assert_eq!(
            GpuBuiltins::opencl_builtin("atomic_cas"),
            Some("atomic_cmpxchg")
        );
    }

    #[test]
    fn test_opencl_builtins_sync() {
        assert_eq!(
            GpuBuiltins::opencl_builtin("sync_threads"),
            Some("barrier(CLK_LOCAL_MEM_FENCE)")
        );
        assert_eq!(
            GpuBuiltins::opencl_builtin("thread_fence"),
            Some("mem_fence(CLK_GLOBAL_MEM_FENCE)")
        );
    }

    #[test]
    fn test_opencl_builtins_thread_indexing() {
        assert_eq!(
            GpuBuiltins::opencl_builtin("thread_idx_x"),
            Some("get_local_id(0)")
        );
        assert_eq!(
            GpuBuiltins::opencl_builtin("thread_idx_y"),
            Some("get_local_id(1)")
        );
        assert_eq!(
            GpuBuiltins::opencl_builtin("block_idx_x"),
            Some("get_group_id(0)")
        );
        assert_eq!(
            GpuBuiltins::opencl_builtin("block_dim_x"),
            Some("get_local_size(0)")
        );
        assert_eq!(
            GpuBuiltins::opencl_builtin("grid_dim_x"),
            Some("get_num_groups(0)")
        );
        assert_eq!(
            GpuBuiltins::opencl_builtin("global_idx"),
            Some("get_global_id(0)")
        );
    }

    #[test]
    fn test_opencl_builtins_unknown_returns_none() {
        assert_eq!(GpuBuiltins::opencl_builtin("nonexistent"), None);
        assert_eq!(GpuBuiltins::opencl_builtin(""), None);
    }

    // ── WGSL builtins tests ──

    #[test]
    fn test_wgsl_builtins_math() {
        assert_eq!(GpuBuiltins::wgsl_builtin("sqrt"), Some("sqrt"));
        assert_eq!(GpuBuiltins::wgsl_builtin("sin"), Some("sin"));
        assert_eq!(GpuBuiltins::wgsl_builtin("cos"), Some("cos"));
        assert_eq!(GpuBuiltins::wgsl_builtin("abs"), Some("abs"));
        assert_eq!(GpuBuiltins::wgsl_builtin("min"), Some("min"));
        assert_eq!(GpuBuiltins::wgsl_builtin("max"), Some("max"));
    }

    #[test]
    fn test_wgsl_builtins_atomics() {
        assert_eq!(GpuBuiltins::wgsl_builtin("atomic_add"), Some("atomicAdd"));
        assert_eq!(GpuBuiltins::wgsl_builtin("atomic_sub"), Some("atomicSub"));
        assert_eq!(
            GpuBuiltins::wgsl_builtin("atomic_cas"),
            Some("atomicCompareExchangeWeak")
        );
    }

    #[test]
    fn test_wgsl_builtins_sync() {
        assert_eq!(
            GpuBuiltins::wgsl_builtin("sync_threads"),
            Some("workgroupBarrier()")
        );
        assert_eq!(
            GpuBuiltins::wgsl_builtin("thread_fence"),
            Some("storageBarrier()")
        );
    }

    #[test]
    fn test_wgsl_builtins_thread_indexing() {
        assert_eq!(
            GpuBuiltins::wgsl_builtin("thread_idx_x"),
            Some("local_invocation_id.x")
        );
        assert_eq!(
            GpuBuiltins::wgsl_builtin("block_idx_x"),
            Some("workgroup_id.x")
        );
        assert_eq!(
            GpuBuiltins::wgsl_builtin("block_dim_x"),
            Some("workgroup_size.x")
        );
        assert_eq!(
            GpuBuiltins::wgsl_builtin("grid_dim_x"),
            Some("num_workgroups.x")
        );
        assert_eq!(
            GpuBuiltins::wgsl_builtin("global_idx"),
            Some("global_invocation_id.x")
        );
    }

    #[test]
    fn test_wgsl_builtins_unknown_returns_none() {
        assert_eq!(GpuBuiltins::wgsl_builtin("nonexistent"), None);
        assert_eq!(GpuBuiltins::wgsl_builtin(""), None);
    }

    // ── Cross-backend consistency tests ──

    #[test]
    fn test_all_backends_have_same_math_functions() {
        let math_funcs = [
            "sqrt", "sin", "cos", "tan", "exp", "log", "pow", "abs", "floor", "ceil", "round",
            "min", "max",
        ];
        for func in &math_funcs {
            assert!(
                GpuBuiltins::cuda_builtin(func).is_some(),
                "CUDA missing: {}",
                func
            );
            assert!(
                GpuBuiltins::opencl_builtin(func).is_some(),
                "OpenCL missing: {}",
                func
            );
            assert!(
                GpuBuiltins::wgsl_builtin(func).is_some(),
                "WGSL missing: {}",
                func
            );
        }
    }

    #[test]
    fn test_all_backends_have_same_atomic_functions() {
        let atomic_funcs = [
            "atomic_add",
            "atomic_sub",
            "atomic_min",
            "atomic_max",
            "atomic_cas",
        ];
        for func in &atomic_funcs {
            assert!(
                GpuBuiltins::cuda_builtin(func).is_some(),
                "CUDA missing: {}",
                func
            );
            assert!(
                GpuBuiltins::opencl_builtin(func).is_some(),
                "OpenCL missing: {}",
                func
            );
            assert!(
                GpuBuiltins::wgsl_builtin(func).is_some(),
                "WGSL missing: {}",
                func
            );
        }
    }

    #[test]
    fn test_all_backends_have_same_sync_functions() {
        let sync_funcs = ["sync_threads", "thread_fence"];
        for func in &sync_funcs {
            assert!(
                GpuBuiltins::cuda_builtin(func).is_some(),
                "CUDA missing: {}",
                func
            );
            assert!(
                GpuBuiltins::opencl_builtin(func).is_some(),
                "OpenCL missing: {}",
                func
            );
            assert!(
                GpuBuiltins::wgsl_builtin(func).is_some(),
                "WGSL missing: {}",
                func
            );
        }
    }

    #[test]
    fn test_all_backends_have_same_indexing_functions() {
        let idx_funcs = [
            "thread_idx_x",
            "thread_idx_y",
            "thread_idx_z",
            "block_idx_x",
            "block_idx_y",
            "block_idx_z",
            "block_dim_x",
            "block_dim_y",
            "block_dim_z",
            "grid_dim_x",
            "grid_dim_y",
            "grid_dim_z",
            "global_idx",
        ];
        for func in &idx_funcs {
            assert!(
                GpuBuiltins::cuda_builtin(func).is_some(),
                "CUDA missing: {}",
                func
            );
            assert!(
                GpuBuiltins::opencl_builtin(func).is_some(),
                "OpenCL missing: {}",
                func
            );
            assert!(
                GpuBuiltins::wgsl_builtin(func).is_some(),
                "WGSL missing: {}",
                func
            );
        }
    }

    #[test]
    fn test_opencl_abs_maps_to_fabs() {
        // OpenCL uses fabs for floating point absolute value
        assert_eq!(GpuBuiltins::opencl_builtin("abs"), Some("fabs"));
        // While CUDA and WGSL use plain abs
        assert_eq!(GpuBuiltins::cuda_builtin("abs"), Some("abs"));
        assert_eq!(GpuBuiltins::wgsl_builtin("abs"), Some("abs"));
    }
}
