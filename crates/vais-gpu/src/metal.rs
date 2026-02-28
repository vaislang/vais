//! Metal Shading Language (MSL) code generator
//!
//! Generates Metal compute shaders from Vais AST for Apple GPUs.
//!
//! # Supported Features
//!
//! - Compute kernels (`kernel` function)
//! - Threadgroup (shared) memory
//! - Thread/threadgroup indexing
//! - Atomic operations
//! - SIMD types (float2, float4, etc.)

use crate::common::{binary_op_str, unary_op_str};
use crate::{GpuError, GpuKernel, GpuResult, GpuType};
use vais_ast::{Expr, Function, FunctionBody, Item, Module, Stmt, Type};

/// Generate Metal code from a Vais module
pub fn generate(module: &Module, kernels: &mut Vec<GpuKernel>) -> GpuResult<String> {
    let mut generator = MetalGenerator::new();
    generator.generate_module(module, kernels)
}

/// Metal built-in function mappings
pub struct MetalBuiltins;

impl MetalBuiltins {
    /// Get Metal equivalent of a Vais built-in function
    pub fn builtin(name: &str) -> Option<&'static str> {
        match name {
            // Math functions
            "sqrt" => Some("sqrt"),
            "rsqrt" => Some("rsqrt"),
            "sin" => Some("sin"),
            "cos" => Some("cos"),
            "tan" => Some("tan"),
            "asin" => Some("asin"),
            "acos" => Some("acos"),
            "atan" => Some("atan"),
            "atan2" => Some("atan2"),
            "sinh" => Some("sinh"),
            "cosh" => Some("cosh"),
            "tanh" => Some("tanh"),
            "exp" => Some("exp"),
            "exp2" => Some("exp2"),
            "log" => Some("log"),
            "log2" => Some("log2"),
            "log10" => Some("log10"),
            "pow" => Some("pow"),
            "abs" => Some("abs"),
            "fabs" => Some("fabs"),
            "floor" => Some("floor"),
            "ceil" => Some("ceil"),
            "round" => Some("round"),
            "trunc" => Some("trunc"),
            "fract" => Some("fract"),
            "min" => Some("min"),
            "max" => Some("max"),
            "clamp" => Some("clamp"),
            "mix" => Some("mix"),
            "step" => Some("step"),
            "smoothstep" => Some("smoothstep"),
            "fma" => Some("fma"),

            // Atomic operations
            "atomic_add" => Some("atomic_fetch_add_explicit"),
            "atomic_sub" => Some("atomic_fetch_sub_explicit"),
            "atomic_min" => Some("atomic_fetch_min_explicit"),
            "atomic_max" => Some("atomic_fetch_max_explicit"),
            "atomic_and" => Some("atomic_fetch_and_explicit"),
            "atomic_or" => Some("atomic_fetch_or_explicit"),
            "atomic_xor" => Some("atomic_fetch_xor_explicit"),
            "atomic_cas" => Some("atomic_compare_exchange_weak_explicit"),
            "atomic_exch" => Some("atomic_exchange_explicit"),

            // Synchronization
            "sync_threads" => Some("threadgroup_barrier(mem_flags::mem_threadgroup)"),
            "thread_fence" => Some("threadgroup_barrier(mem_flags::mem_device)"),
            "thread_fence_block" => Some("threadgroup_barrier(mem_flags::mem_threadgroup)"),

            // Thread indexing
            "thread_idx_x" => Some("thread_position_in_threadgroup.x"),
            "thread_idx_y" => Some("thread_position_in_threadgroup.y"),
            "thread_idx_z" => Some("thread_position_in_threadgroup.z"),
            "block_idx_x" => Some("threadgroup_position_in_grid.x"),
            "block_idx_y" => Some("threadgroup_position_in_grid.y"),
            "block_idx_z" => Some("threadgroup_position_in_grid.z"),
            "block_dim_x" => Some("threads_per_threadgroup.x"),
            "block_dim_y" => Some("threads_per_threadgroup.y"),
            "block_dim_z" => Some("threads_per_threadgroup.z"),
            "grid_dim_x" => Some("threadgroups_per_grid.x"),
            "grid_dim_y" => Some("threadgroups_per_grid.y"),
            "grid_dim_z" => Some("threadgroups_per_grid.z"),
            "global_idx" => Some("thread_position_in_grid.x"),
            "global_idx_x" => Some("thread_position_in_grid.x"),
            "global_idx_y" => Some("thread_position_in_grid.y"),
            "global_idx_z" => Some("thread_position_in_grid.z"),
            "lane_id" => Some("simd_lane_id"),

            // SIMD operations
            "simd_sum" => Some("simd_sum"),
            "simd_min" => Some("simd_min"),
            "simd_max" => Some("simd_max"),
            "simd_broadcast" => Some("simd_broadcast"),
            "simd_shuffle" => Some("simd_shuffle"),
            "simd_shuffle_down" => Some("simd_shuffle_down"),
            "simd_shuffle_up" => Some("simd_shuffle_up"),
            "simd_shuffle_xor" => Some("simd_shuffle_xor"),

            // Warp vote (SIMD group)
            "warp_all" => Some("simd_all"),
            "warp_any" => Some("simd_any"),
            "warp_ballot" => Some("simd_ballot"),

            _ => None,
        }
    }
}

struct MetalGenerator {
    output: String,
    indent_level: usize,
}

impl MetalGenerator {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
        }
    }

    fn generate_module(
        &mut self,
        module: &Module,
        kernels: &mut Vec<GpuKernel>,
    ) -> GpuResult<String> {
        // Header
        self.emit_line("// Generated by Vais GPU Compiler");
        self.emit_line("// Target: Metal (Apple GPU)");
        self.emit_line("");
        self.emit_line("#include <metal_stdlib>");
        self.emit_line("#include <simd/simd.h>");
        self.emit_line("using namespace metal;");
        self.emit_line("");

        // Generate items
        for item in &module.items {
            self.generate_item(&item.node, kernels)?;
        }

        Ok(self.output.clone())
    }

    fn generate_item(&mut self, item: &Item, kernels: &mut Vec<GpuKernel>) -> GpuResult<()> {
        match item {
            Item::Function(func) => {
                // Check if this is a GPU kernel
                let is_kernel = func
                    .attributes
                    .iter()
                    .any(|attr| attr.name == "gpu" || attr.name == "kernel");

                // Extract thread_block_size attribute
                let block_size = self.extract_block_size(&func.attributes);

                // Extract shared_memory attribute
                let shared_memory = self.extract_shared_memory(&func.attributes);

                if is_kernel {
                    self.generate_kernel(func, kernels, block_size, shared_memory)?;
                } else {
                    self.generate_device_function(func)?;
                }
            }
            Item::Struct(s) => {
                self.generate_struct(&s.name.node, &s.fields)?;
            }
            _ => {
                // Skip other items
            }
        }
        Ok(())
    }

    fn extract_block_size(&self, attrs: &[vais_ast::Attribute]) -> (usize, usize, usize) {
        for attr in attrs {
            if (attr.name == "thread_block_size" || attr.name == "threads_per_threadgroup")
                && !attr.args.is_empty()
            {
                // Parse (x, y, z) or just x from args vector
                let x = attr
                    .args
                    .first()
                    .and_then(|s| s.trim().parse::<usize>().ok())
                    .unwrap_or(256);
                let y = attr
                    .args
                    .get(1)
                    .and_then(|s| s.trim().parse::<usize>().ok())
                    .unwrap_or(1);
                let z = attr
                    .args
                    .get(2)
                    .and_then(|s| s.trim().parse::<usize>().ok())
                    .unwrap_or(1);
                return (x, y, z);
            }
        }
        (256, 1, 1) // Default
    }

    fn extract_shared_memory(&self, attrs: &[vais_ast::Attribute]) -> usize {
        for attr in attrs {
            if attr.name == "shared_memory" || attr.name == "threadgroup_memory" {
                if let Some(arg) = attr.args.first() {
                    return arg.trim().parse().unwrap_or(0);
                }
            }
        }
        0
    }

    fn generate_kernel(
        &mut self,
        func: &Function,
        kernels: &mut Vec<GpuKernel>,
        block_size: (usize, usize, usize),
        shared_memory: usize,
    ) -> GpuResult<()> {
        let name = &func.name.node;

        // Emit kernel function signature
        self.emit("kernel void ");
        self.emit(name);
        self.emit("(\n");
        self.indent_level += 1;

        // Parameters with buffer bindings
        let param_types: Vec<(String, GpuType)> = func
            .params
            .iter()
            .map(|p| (p.name.node.clone(), self.vais_type_to_gpu(&p.ty.node)))
            .collect();

        for (i, param) in func.params.iter().enumerate() {
            self.emit_indent();
            let metal_type = self.type_to_metal(&param.ty.node);

            // Determine address space based on type
            if matches!(param.ty.node, Type::Pointer(_)) {
                self.emit(&format!("device {} [[buffer({})]]", metal_type, i));
            } else {
                self.emit(&format!(
                    "constant {}& {} [[buffer({})]]",
                    self.type_to_metal_base(&param.ty.node),
                    param.name.node,
                    i
                ));
            }

            if i < func.params.len() - 1 {
                self.emit(",\n");
            }
        }

        // Thread position parameters
        self.emit(",\n");
        self.emit_indent();
        self.emit("uint3 thread_position_in_grid [[thread_position_in_grid]],\n");
        self.emit_indent();
        self.emit("uint3 thread_position_in_threadgroup [[thread_position_in_threadgroup]],\n");
        self.emit_indent();
        self.emit("uint3 threads_per_threadgroup [[threads_per_threadgroup]],\n");
        self.emit_indent();
        self.emit("uint3 threadgroup_position_in_grid [[threadgroup_position_in_grid]],\n");
        self.emit_indent();
        self.emit("uint3 threadgroups_per_grid [[threadgroups_per_grid]],\n");
        self.emit_indent();
        self.emit("uint simd_lane_id [[thread_index_in_simdgroup]]");

        self.indent_level -= 1;
        self.emit_line("\n) {");
        self.indent_level += 1;

        // Shared memory allocation if needed
        if shared_memory > 0 {
            self.emit_indent();
            self.emit_line(&format!(
                "threadgroup float shared_mem[{}];",
                shared_memory / 4
            ));
        }

        // Generate body
        self.generate_function_body(&func.body)?;

        self.indent_level -= 1;
        self.emit_line("}");
        self.emit_line("");

        // Record kernel metadata
        kernels.push(GpuKernel {
            name: name.to_string(),
            params: param_types,
            shared_memory,
            block_size,
        });

        Ok(())
    }

    fn generate_device_function(&mut self, func: &Function) -> GpuResult<()> {
        let name = &func.name.node;

        let ret_str = func
            .ret_type
            .as_ref()
            .map(|t| self.type_to_metal(&t.node))
            .unwrap_or_else(|| "void".to_string());

        self.emit(&format!("inline {} {}(", ret_str, name));

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                self.emit(", ");
            }
            self.emit(&format!(
                "{} {}",
                self.type_to_metal(&param.ty.node),
                param.name.node
            ));
        }

        self.emit_line(") {");
        self.indent_level += 1;

        // Generate body
        if func.ret_type.is_some() {
            self.emit_indent();
            self.emit("return ");
            match &func.body {
                FunctionBody::Expr(expr) => {
                    self.generate_expr(&expr.node)?;
                }
                FunctionBody::Block(stmts) => {
                    for stmt in stmts {
                        self.generate_stmt(&stmt.node)?;
                    }
                }
            }
            self.emit_line(";");
        } else {
            self.generate_function_body(&func.body)?;
        }

        self.indent_level -= 1;
        self.emit_line("}");
        self.emit_line("");

        Ok(())
    }

    fn generate_function_body(&mut self, body: &FunctionBody) -> GpuResult<()> {
        match body {
            FunctionBody::Expr(expr) => {
                self.emit_indent();
                self.generate_expr(&expr.node)?;
                self.emit_line(";");
            }
            FunctionBody::Block(stmts) => {
                for stmt in stmts {
                    self.generate_stmt(&stmt.node)?;
                }
            }
        }
        Ok(())
    }

    fn generate_struct(&mut self, name: &str, fields: &[vais_ast::Field]) -> GpuResult<()> {
        self.emit_line(&format!("struct {} {{", name));
        self.indent_level += 1;

        for field in fields {
            self.emit_indent();
            self.emit_line(&format!(
                "{} {};",
                self.type_to_metal(&field.ty.node),
                field.name.node
            ));
        }

        self.indent_level -= 1;
        self.emit_line("};");
        self.emit_line("");

        Ok(())
    }

    fn generate_expr(&mut self, expr: &Expr) -> GpuResult<()> {
        match expr {
            Expr::Int(n) => self.emit(&n.to_string()),
            Expr::Float(f) => self.emit(&format!("{:.6}f", f)),
            Expr::Bool(b) => self.emit(if *b { "true" } else { "false" }),
            Expr::Ident(name) => {
                // Check for Metal built-in
                if let Some(builtin) = MetalBuiltins::builtin(name) {
                    self.emit(builtin);
                } else {
                    self.emit(name);
                }
            }
            Expr::Binary { op, left, right } => {
                self.emit("(");
                self.generate_expr(&left.node)?;
                self.emit(&format!(" {} ", binary_op_str(op)));
                self.generate_expr(&right.node)?;
                self.emit(")");
            }
            Expr::Unary { op, expr } => {
                self.emit(unary_op_str(op));
                self.generate_expr(&expr.node)?;
            }
            Expr::Call { func, args } => {
                // Check if it's a built-in
                if let Expr::Ident(name) = &func.node {
                    if let Some(builtin) = MetalBuiltins::builtin(name) {
                        self.emit(builtin);
                        if !builtin.contains('.') && !builtin.contains('(') {
                            self.emit("(");
                            for (i, arg) in args.iter().enumerate() {
                                if i > 0 {
                                    self.emit(", ");
                                }
                                self.generate_expr(&arg.node)?;
                            }
                            self.emit(")");
                        }
                        return Ok(());
                    }
                }

                self.generate_expr(&func.node)?;
                self.emit("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.emit(", ");
                    }
                    self.generate_expr(&arg.node)?;
                }
                self.emit(")");
            }
            Expr::Index { expr, index } => {
                self.generate_expr(&expr.node)?;
                self.emit("[");
                self.generate_expr(&index.node)?;
                self.emit("]");
            }
            Expr::Field { expr, field } => {
                self.generate_expr(&expr.node)?;
                self.emit(&format!(".{}", field.node));
            }
            Expr::If { cond, then, else_ } => {
                self.emit_indent();
                self.emit("if (");
                self.generate_expr(&cond.node)?;
                self.emit_line(") {");
                self.indent_level += 1;
                for stmt in then {
                    self.generate_stmt(&stmt.node)?;
                }
                self.indent_level -= 1;
                self.emit_indent();
                if let Some(else_branch) = else_ {
                    self.emit("} else ");
                    self.generate_if_else(else_branch)?;
                } else {
                    self.emit_line("}");
                }
            }
            Expr::Block(stmts) => {
                for stmt in stmts {
                    self.generate_stmt(&stmt.node)?;
                }
            }
            Expr::Assign { target, value } => {
                self.emit_indent();
                self.generate_expr(&target.node)?;
                self.emit(" = ");
                self.generate_expr(&value.node)?;
                self.emit_line(";");
            }
            Expr::Loop { body, .. } => {
                self.emit_indent();
                self.emit_line("while (true) {");
                self.indent_level += 1;
                for stmt in body {
                    self.generate_stmt(&stmt.node)?;
                }
                self.indent_level -= 1;
                self.emit_indent();
                self.emit_line("}");
            }
            Expr::Range { start, end, .. } => {
                self.emit_indent();
                self.emit("for (int i = ");
                if let Some(s) = start {
                    self.generate_expr(&s.node)?;
                } else {
                    self.emit("0");
                }
                self.emit("; i < ");
                if let Some(e) = end {
                    self.generate_expr(&e.node)?;
                } else {
                    self.emit("INT_MAX");
                }
                self.emit_line("; i++) {");
            }
            _ => {
                return Err(GpuError::UnsupportedOperation(format!(
                    "Expression not supported in Metal: {:?}",
                    std::mem::discriminant(expr)
                )));
            }
        }
        Ok(())
    }

    fn generate_if_else(&mut self, else_branch: &vais_ast::IfElse) -> GpuResult<()> {
        match else_branch {
            vais_ast::IfElse::ElseIf(cond, then, next) => {
                self.emit("if (");
                self.generate_expr(&cond.node)?;
                self.emit_line(") {");
                self.indent_level += 1;
                for stmt in then {
                    self.generate_stmt(&stmt.node)?;
                }
                self.indent_level -= 1;
                self.emit_indent();
                if let Some(next_else) = next {
                    self.emit("} else ");
                    self.generate_if_else(next_else)?;
                } else {
                    self.emit_line("}");
                }
            }
            vais_ast::IfElse::Else(stmts) => {
                self.emit_line("{");
                self.indent_level += 1;
                for stmt in stmts {
                    self.generate_stmt(&stmt.node)?;
                }
                self.indent_level -= 1;
                self.emit_indent();
                self.emit_line("}");
            }
        }
        Ok(())
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> GpuResult<()> {
        match stmt {
            Stmt::Let {
                name, ty, value, ..
            } => {
                self.emit_indent();
                if let Some(t) = ty {
                    self.emit(&format!("{} {} = ", self.type_to_metal(&t.node), name.node));
                } else {
                    self.emit(&format!("auto {} = ", name.node));
                }
                self.generate_expr(&value.node)?;
                self.emit_line(";");
            }
            Stmt::Expr(expr) => {
                self.emit_indent();
                self.generate_expr(&expr.node)?;
                self.emit_line(";");
            }
            Stmt::Return(expr) => {
                self.emit_indent();
                self.emit("return");
                if let Some(e) = expr {
                    self.emit(" ");
                    self.generate_expr(&e.node)?;
                }
                self.emit_line(";");
            }
            Stmt::Break(_) => {
                self.emit_indent();
                self.emit_line("break;");
            }
            Stmt::Continue => {
                self.emit_indent();
                self.emit_line("continue;");
            }
            _ => {}
        }
        Ok(())
    }

    fn type_to_metal(&self, ty: &Type) -> String {
        match ty {
            Type::Named { name, .. } => {
                match name.as_str() {
                    "i64" => "int64_t".to_string(),
                    "i32" => "int".to_string(),
                    "i16" => "short".to_string(),
                    "i8" => "char".to_string(),
                    "u64" => "uint64_t".to_string(),
                    "u32" => "uint".to_string(),
                    "u16" => "ushort".to_string(),
                    "u8" => "uchar".to_string(),
                    "f64" => "double".to_string(),
                    "f32" => "float".to_string(),
                    "f16" => "half".to_string(),
                    "bool" => "bool".to_string(),
                    "unit" | "()" => "void".to_string(),
                    // Vector types
                    "float2" | "Vec2f32" => "float2".to_string(),
                    "float3" | "Vec3f32" => "float3".to_string(),
                    "float4" | "Vec4f32" => "float4".to_string(),
                    "int2" | "Vec2i32" => "int2".to_string(),
                    "int3" | "Vec3i32" => "int3".to_string(),
                    "int4" | "Vec4i32" => "int4".to_string(),
                    "uint2" | "Vec2u32" => "uint2".to_string(),
                    "uint3" | "Vec3u32" => "uint3".to_string(),
                    "uint4" | "Vec4u32" => "uint4".to_string(),
                    "half2" | "Vec2f16" => "half2".to_string(),
                    "half4" | "Vec4f16" => "half4".to_string(),
                    _ => name.clone(),
                }
            }
            Type::Pointer(inner) => format!("device {}*", self.type_to_metal(&inner.node)),
            Type::ConstArray { element, size } => {
                format!("{}[{}]", self.type_to_metal(&element.node), size)
            }
            _ => "void".to_string(),
        }
    }

    fn type_to_metal_base(&self, ty: &Type) -> String {
        // Same as type_to_metal but without pointer decoration
        match ty {
            Type::Named { name, .. } => match name.as_str() {
                "i64" => "int64_t".to_string(),
                "i32" => "int".to_string(),
                "f64" => "double".to_string(),
                "f32" => "float".to_string(),
                "bool" => "bool".to_string(),
                _ => name.clone(),
            },
            _ => "void".to_string(),
        }
    }

    fn vais_type_to_gpu(&self, ty: &Type) -> GpuType {
        match ty {
            Type::Named { name, .. } => match name.as_str() {
                "i32" => GpuType::I32,
                "i64" => GpuType::I64,
                "f32" => GpuType::F32,
                "f64" => GpuType::F64,
                "bool" => GpuType::Bool,
                _ => GpuType::Void,
            },
            Type::Pointer(inner) => GpuType::Ptr(Box::new(self.vais_type_to_gpu(&inner.node))),
            _ => GpuType::Void,
        }
    }

    fn emit(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn emit_line(&mut self, s: &str) {
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn emit_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }
}

/// Generate Metal host code for launching kernels (Swift/Objective-C)
pub fn generate_host_code(kernels: &[GpuKernel], library_name: &str) -> String {
    let mut code = String::new();

    code.push_str("// Metal Host Code (Swift)\n");
    code.push_str("// Generated by Vais GPU Compiler\n\n");
    code.push_str("import Metal\nimport MetalKit\n\n");
    code.push_str(&format!("class {}Kernels {{\n", library_name));
    code.push_str("    let device: MTLDevice\n");
    code.push_str("    let commandQueue: MTLCommandQueue\n");
    code.push_str("    let library: MTLLibrary\n\n");

    // Constructor
    code.push_str("    init?() {\n");
    code.push_str("        guard let device = MTLCreateSystemDefaultDevice(),\n");
    code.push_str("              let commandQueue = device.makeCommandQueue(),\n");
    code.push_str("              let library = try? device.makeDefaultLibrary() else {\n");
    code.push_str("            return nil\n");
    code.push_str("        }\n");
    code.push_str("        self.device = device\n");
    code.push_str("        self.commandQueue = commandQueue\n");
    code.push_str("        self.library = library\n");
    code.push_str("    }\n\n");

    // Generate launch methods for each kernel
    for kernel in kernels {
        let (bx, by, bz) = kernel.block_size;

        code.push_str(&format!("    /// Launch {} kernel\n", kernel.name));
        code.push_str(&format!("    func launch_{}(\n", kernel.name));
        code.push_str("        gridSize: MTLSize,\n");
        code.push_str(&format!(
            "        threadgroupSize: MTLSize = MTLSize(width: {}, height: {}, depth: {})",
            bx, by, bz
        ));

        for (pname, _ptype) in &kernel.params {
            code.push_str(&format!(",\n        {}: MTLBuffer", pname));
        }

        code.push_str("\n    ) {\n");
        code.push_str(&format!(
            "        guard let function = library.makeFunction(name: \"{}\"),\n",
            kernel.name
        ));
        code.push_str("              let pipeline = try? device.makeComputePipelineState(function: function),\n");
        code.push_str("              let commandBuffer = commandQueue.makeCommandBuffer(),\n");
        code.push_str(
            "              let encoder = commandBuffer.makeComputeCommandEncoder() else {\n",
        );
        code.push_str("            return\n");
        code.push_str("        }\n\n");

        code.push_str("        encoder.setComputePipelineState(pipeline)\n");

        for (i, (pname, _)) in kernel.params.iter().enumerate() {
            code.push_str(&format!(
                "        encoder.setBuffer({}, offset: 0, index: {})\n",
                pname, i
            ));
        }

        code.push_str("        encoder.dispatchThreadgroups(gridSize, threadsPerThreadgroup: threadgroupSize)\n");
        code.push_str("        encoder.endEncoding()\n");
        code.push_str("        commandBuffer.commit()\n");
        code.push_str("        commandBuffer.waitUntilCompleted()\n");
        code.push_str("    }\n\n");
    }

    code.push_str("}\n");

    code
}

#[cfg(test)]
mod tests {
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
        assert_eq!(
            MetalBuiltins::builtin("lane_id"),
            Some("simd_lane_id")
        );
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
        assert_eq!(
            MetalBuiltins::builtin("simd_shuffle"),
            Some("simd_shuffle")
        );
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
        assert_eq!(
            MetalBuiltins::builtin("warp_ballot"),
            Some("simd_ballot")
        );
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
}
