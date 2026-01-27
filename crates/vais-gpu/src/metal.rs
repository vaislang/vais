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

use vais_ast::{Module, Item, Stmt, Expr, Type, Function, FunctionBody};
use crate::{GpuError, GpuResult, GpuKernel, GpuType};
use crate::common::{binary_op_str, unary_op_str};

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

    fn generate_module(&mut self, module: &Module, kernels: &mut Vec<GpuKernel>) -> GpuResult<String> {
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
                let is_kernel = func.attributes.iter().any(|attr| {
                    attr.name == "gpu" || attr.name == "kernel"
                });

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
            if attr.name == "thread_block_size" || attr.name == "threads_per_threadgroup" {
                if !attr.args.is_empty() {
                    // Parse (x, y, z) or just x from args vector
                    let x = attr.args.get(0).and_then(|s| s.trim().parse::<usize>().ok()).unwrap_or(256);
                    let y = attr.args.get(1).and_then(|s| s.trim().parse::<usize>().ok()).unwrap_or(1);
                    let z = attr.args.get(2).and_then(|s| s.trim().parse::<usize>().ok()).unwrap_or(1);
                    return (x, y, z);
                }
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
        let param_types: Vec<(String, GpuType)> = func.params
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
                self.emit(&format!("constant {}& {} [[buffer({})]]",
                    self.type_to_metal_base(&param.ty.node), param.name.node, i));
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
            self.emit_line(&format!("threadgroup float shared_mem[{}];", shared_memory / 4));
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

        let ret_str = func.ret_type
            .as_ref()
            .map(|t| self.type_to_metal(&t.node))
            .unwrap_or_else(|| "void".to_string());

        self.emit(&format!("inline {} {}(", ret_str, name));

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                self.emit(", ");
            }
            self.emit(&format!("{} {}", self.type_to_metal(&param.ty.node), param.name.node));
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
            self.emit_line(&format!("{} {};", self.type_to_metal(&field.ty.node), field.name.node));
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
            Stmt::Let { name, ty, value, .. } => {
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
            Type::Named { name, .. } => {
                match name.as_str() {
                    "i64" => "int64_t".to_string(),
                    "i32" => "int".to_string(),
                    "f64" => "double".to_string(),
                    "f32" => "float".to_string(),
                    "bool" => "bool".to_string(),
                    _ => name.clone(),
                }
            }
            _ => "void".to_string(),
        }
    }

    fn vais_type_to_gpu(&self, ty: &Type) -> GpuType {
        match ty {
            Type::Named { name, .. } => {
                match name.as_str() {
                    "i32" => GpuType::I32,
                    "i64" => GpuType::I64,
                    "f32" => GpuType::F32,
                    "f64" => GpuType::F64,
                    "bool" => GpuType::Bool,
                    _ => GpuType::Void,
                }
            }
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
    code.push_str(&format!("              let library = try? device.makeDefaultLibrary() else {{\n"));
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
        code.push_str(&format!("        threadgroupSize: MTLSize = MTLSize(width: {}, height: {}, depth: {})", bx, by, bz));

        for (pname, ptype) in &kernel.params {
            code.push_str(&format!(",\n        {}: MTLBuffer", pname));
        }

        code.push_str("\n    ) {\n");
        code.push_str(&format!("        guard let function = library.makeFunction(name: \"{}\"),\n", kernel.name));
        code.push_str("              let pipeline = try? device.makeComputePipelineState(function: function),\n");
        code.push_str("              let commandBuffer = commandQueue.makeCommandBuffer(),\n");
        code.push_str("              let encoder = commandBuffer.makeComputeCommandEncoder() else {\n");
        code.push_str("            return\n");
        code.push_str("        }\n\n");

        code.push_str("        encoder.setComputePipelineState(pipeline)\n");

        for (i, (pname, _)) in kernel.params.iter().enumerate() {
            code.push_str(&format!("        encoder.setBuffer({}, offset: 0, index: {})\n", pname, i));
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

    #[test]
    fn test_metal_builtins() {
        assert_eq!(MetalBuiltins::builtin("sqrt"), Some("sqrt"));
        assert_eq!(MetalBuiltins::builtin("thread_idx_x"), Some("thread_position_in_threadgroup.x"));
        assert_eq!(MetalBuiltins::builtin("sync_threads"), Some("threadgroup_barrier(mem_flags::mem_threadgroup)"));
        assert_eq!(MetalBuiltins::builtin("global_idx"), Some("thread_position_in_grid.x"));
    }

    #[test]
    fn test_metal_type_names() {
        let gen = MetalGenerator::new();
        assert_eq!(gen.type_to_metal(&Type::Named { name: "i64".to_string(), generics: vec![] }), "int64_t");
        assert_eq!(gen.type_to_metal(&Type::Named { name: "f32".to_string(), generics: vec![] }), "float");
        assert_eq!(gen.type_to_metal(&Type::Named { name: "f16".to_string(), generics: vec![] }), "half");
    }

    #[test]
    fn test_metal_vector_types() {
        let gen = MetalGenerator::new();
        assert_eq!(gen.type_to_metal(&Type::Named { name: "float4".to_string(), generics: vec![] }), "float4");
        assert_eq!(gen.type_to_metal(&Type::Named { name: "Vec4f32".to_string(), generics: vec![] }), "float4");
        assert_eq!(gen.type_to_metal(&Type::Named { name: "half2".to_string(), generics: vec![] }), "half2");
    }

    #[test]
    fn test_generate_host_code_basic() {
        let kernels = vec![
            GpuKernel {
                name: "vector_add".to_string(),
                params: vec![
                    ("a".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
                    ("b".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
                    ("c".to_string(), GpuType::Ptr(Box::new(GpuType::F32))),
                ],
                shared_memory: 0,
                block_size: (256, 1, 1),
            }
        ];

        let code = generate_host_code(&kernels, "VectorOps");
        assert!(code.contains("class VectorOpsKernels"));
        assert!(code.contains("func launch_vector_add"));
        assert!(code.contains("MTLSize(width: 256, height: 1, depth: 1)"));
    }
}
