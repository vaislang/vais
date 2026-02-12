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

    #[test]
    fn test_binary_op_str() {
        assert_eq!(binary_op_str(&BinOp::Add), "+");
        assert_eq!(binary_op_str(&BinOp::Mul), "*");
        assert_eq!(binary_op_str(&BinOp::Eq), "==");
    }

    #[test]
    fn test_cuda_builtins() {
        assert_eq!(GpuBuiltins::cuda_builtin("sqrt"), Some("sqrt"));
        assert_eq!(
            GpuBuiltins::cuda_builtin("thread_idx_x"),
            Some("threadIdx.x")
        );
        assert_eq!(
            GpuBuiltins::cuda_builtin("sync_threads"),
            Some("__syncthreads")
        );
    }
}
