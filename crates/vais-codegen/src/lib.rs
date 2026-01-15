//! Vais 2.0 LLVM Code Generator
//!
//! Generates LLVM IR from typed AST for native code generation.
//!
//! Note: This is a placeholder structure. Full LLVM integration requires
//! the inkwell crate and LLVM installation.

use std::collections::HashMap;
use thiserror::Error;
use vais_ast::*;
use vais_types::ResolvedType;

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("Undefined variable: {0}")]
    UndefinedVar(String),

    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("LLVM error: {0}")]
    LlvmError(String),

    #[error("Unsupported feature: {0}")]
    Unsupported(String),
}

type CodegenResult<T> = Result<T, CodegenError>;

/// LLVM IR Builder placeholder
///
/// In a full implementation, this would use inkwell to generate LLVM IR.
/// For now, we provide the structure and will implement with inkwell later.
pub struct CodeGenerator {
    // Module name
    module_name: String,

    // Function signatures for lookup
    functions: HashMap<String, FunctionInfo>,

    // Struct definitions
    structs: HashMap<String, StructInfo>,

    // Current function being compiled
    current_function: Option<String>,

    // Local variables in current function
    locals: HashMap<String, LocalVar>,
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    name: String,
    params: Vec<(String, ResolvedType)>,
    ret_type: ResolvedType,
    is_extern: bool,
}

#[derive(Debug, Clone)]
struct StructInfo {
    #[allow(dead_code)]
    name: String,
    fields: Vec<(String, ResolvedType)>,
}

#[derive(Debug, Clone)]
struct LocalVar {
    ty: ResolvedType,
    /// True if this is a function parameter (SSA value), false if alloca'd
    is_param: bool,
}

impl CodeGenerator {
    pub fn new(module_name: &str) -> Self {
        Self {
            module_name: module_name.to_string(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            current_function: None,
            locals: HashMap::new(),
        }
    }

    /// Generate LLVM IR for a module
    pub fn generate_module(&mut self, module: &Module) -> CodegenResult<String> {
        let mut ir = String::new();

        // Header
        ir.push_str(&format!("; ModuleID = '{}'\n", self.module_name));
        ir.push_str("source_filename = \"<vais>\"\n");

        // Note: target triple and data layout are omitted to let clang auto-detect
        ir.push('\n');

        // First pass: collect declarations
        for item in &module.items {
            match &item.node {
                Item::Function(f) => self.register_function(f)?,
                Item::Struct(s) => self.register_struct(s)?,
                _ => {}
            }
        }

        // Generate struct types
        for (name, info) in &self.structs {
            ir.push_str(&self.generate_struct_type(name, info));
            ir.push('\n');
        }

        // Generate function declarations
        for (_name, info) in &self.functions {
            if info.is_extern {
                ir.push_str(&self.generate_extern_decl(info));
                ir.push('\n');
            }
        }

        // Second pass: generate function bodies
        for item in &module.items {
            if let Item::Function(f) = &item.node {
                ir.push_str(&self.generate_function(f)?);
                ir.push('\n');
            }
        }

        Ok(ir)
    }

    fn register_function(&mut self, f: &Function) -> CodegenResult<()> {
        let params: Vec<_> = f
            .params
            .iter()
            .map(|p| {
                let ty = self.ast_type_to_resolved(&p.ty.node);
                (p.name.node.clone(), ty)
            })
            .collect();

        let ret_type = f
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);

        self.functions.insert(
            f.name.node.clone(),
            FunctionInfo {
                name: f.name.node.clone(),
                params,
                ret_type,
                is_extern: false,
            },
        );

        Ok(())
    }

    fn register_struct(&mut self, s: &Struct) -> CodegenResult<()> {
        let fields: Vec<_> = s
            .fields
            .iter()
            .map(|f| {
                let ty = self.ast_type_to_resolved(&f.ty.node);
                (f.name.node.clone(), ty)
            })
            .collect();

        self.structs.insert(
            s.name.node.clone(),
            StructInfo {
                name: s.name.node.clone(),
                fields,
            },
        );

        Ok(())
    }

    fn generate_struct_type(&self, name: &str, info: &StructInfo) -> String {
        let fields: Vec<_> = info
            .fields
            .iter()
            .map(|(_, ty)| self.type_to_llvm(ty))
            .collect();

        format!("%{} = type {{ {} }}", name, fields.join(", "))
    }

    fn generate_extern_decl(&self, info: &FunctionInfo) -> String {
        let params: Vec<_> = info
            .params
            .iter()
            .map(|(_, ty)| self.type_to_llvm(ty))
            .collect();

        let ret = self.type_to_llvm(&info.ret_type);

        format!("declare {} @{}({})", ret, info.name, params.join(", "))
    }

    fn generate_function(&mut self, f: &Function) -> CodegenResult<String> {
        self.current_function = Some(f.name.node.clone());
        self.locals.clear();

        let params: Vec<_> = f
            .params
            .iter()
            .map(|p| {
                let ty = self.ast_type_to_resolved(&p.ty.node);
                let llvm_ty = self.type_to_llvm(&ty);

                // Register parameter as local (SSA value, not alloca)
                self.locals.insert(
                    p.name.node.clone(),
                    LocalVar {
                        ty: ty.clone(),
                        is_param: true,
                    },
                );

                format!("{} %{}", llvm_ty, p.name.node)
            })
            .collect();

        let ret_type = f
            .ret_type
            .as_ref()
            .map(|t| self.ast_type_to_resolved(&t.node))
            .unwrap_or(ResolvedType::Unit);

        let ret_llvm = self.type_to_llvm(&ret_type);

        let mut ir = format!(
            "define {} @{}({}) {{\n",
            ret_llvm,
            f.name.node,
            params.join(", ")
        );

        ir.push_str("entry:\n");

        // Generate body
        match &f.body {
            FunctionBody::Expr(expr) => {
                let (value, expr_ir) = self.generate_expr(expr, &mut 0)?;
                ir.push_str(&expr_ir);
                if ret_type == ResolvedType::Unit {
                    ir.push_str("  ret void\n");
                } else {
                    ir.push_str(&format!("  ret {} {}\n", ret_llvm, value));
                }
            }
            FunctionBody::Block(stmts) => {
                let (value, block_ir) = self.generate_block(stmts, &mut 0)?;
                ir.push_str(&block_ir);
                if ret_type == ResolvedType::Unit {
                    ir.push_str("  ret void\n");
                } else {
                    ir.push_str(&format!("  ret {} {}\n", ret_llvm, value));
                }
            }
        }

        ir.push_str("}\n");

        self.current_function = None;
        Ok(ir)
    }

    fn generate_block(
        &mut self,
        stmts: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();
        let mut last_value = "void".to_string();

        for stmt in stmts {
            let (value, stmt_ir) = self.generate_stmt(stmt, counter)?;
            ir.push_str(&stmt_ir);
            last_value = value;
        }

        Ok((last_value, ir))
    }

    fn generate_stmt(
        &mut self,
        stmt: &Spanned<Stmt>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        match &stmt.node {
            Stmt::Let {
                name,
                ty,
                value,
                is_mut: _,
            } => {
                let (val, val_ir) = self.generate_expr(value, counter)?;

                let resolved_ty = ty
                    .as_ref()
                    .map(|t| self.ast_type_to_resolved(&t.node))
                    .unwrap_or(ResolvedType::I64); // Default to i64

                self.locals.insert(
                    name.node.clone(),
                    LocalVar {
                        ty: resolved_ty.clone(),
                        is_param: false, // alloca'd variable
                    },
                );

                let llvm_ty = self.type_to_llvm(&resolved_ty);
                let mut ir = val_ir;

                // Allocate and store
                ir.push_str(&format!(
                    "  %{} = alloca {}\n",
                    name.node, llvm_ty
                ));
                ir.push_str(&format!(
                    "  store {} {}, {}* %{}\n",
                    llvm_ty, val, llvm_ty, name.node
                ));

                Ok(("void".to_string(), ir))
            }
            Stmt::Expr(expr) => self.generate_expr(expr, counter),
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    let (val, ir) = self.generate_expr(expr, counter)?;
                    Ok((val, ir))
                } else {
                    Ok(("void".to_string(), String::new()))
                }
            }
            Stmt::Break(_) | Stmt::Continue => {
                // TODO: Implement loop control flow
                Ok(("void".to_string(), String::new()))
            }
        }
    }

    fn generate_expr(
        &mut self,
        expr: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        match &expr.node {
            Expr::Int(n) => Ok((n.to_string(), String::new())),
            Expr::Float(n) => Ok((format!("{:e}", n), String::new())),
            Expr::Bool(b) => Ok((if *b { "1" } else { "0" }.to_string(), String::new())),
            Expr::String(s) => {
                // TODO: Proper string handling
                Ok((format!("\"{}\"", s), String::new()))
            }
            Expr::Unit => Ok(("void".to_string(), String::new())),

            Expr::Ident(name) => {
                if let Some(local) = self.locals.get(name) {
                    if local.is_param {
                        // Parameters are SSA values, use directly
                        Ok((format!("%{}", name), String::new()))
                    } else {
                        // Local variables need to be loaded from alloca
                        let tmp = self.next_temp(counter);
                        let llvm_ty = self.type_to_llvm(&local.ty);
                        let ir = format!(
                            "  {} = load {}, {}* %{}\n",
                            tmp, llvm_ty, llvm_ty, name
                        );
                        Ok((tmp, ir))
                    }
                } else if name == "self" {
                    // Handle self reference
                    Ok(("%self".to_string(), String::new()))
                } else {
                    // Might be a function reference
                    Ok((format!("@{}", name), String::new()))
                }
            }

            Expr::SelfCall => {
                // @ refers to current function
                if let Some(fn_name) = &self.current_function {
                    Ok((format!("@{}", fn_name), String::new()))
                } else {
                    Err(CodegenError::UndefinedFunction("@".to_string()))
                }
            }

            Expr::Binary { op, left, right } => {
                let (left_val, left_ir) = self.generate_expr(left, counter)?;
                let (right_val, right_ir) = self.generate_expr(right, counter)?;

                let tmp = self.next_temp(counter);
                let op_str = match op {
                    BinOp::Add => "add",
                    BinOp::Sub => "sub",
                    BinOp::Mul => "mul",
                    BinOp::Div => "sdiv",
                    BinOp::Mod => "srem",
                    BinOp::Lt => "icmp slt",
                    BinOp::Lte => "icmp sle",
                    BinOp::Gt => "icmp sgt",
                    BinOp::Gte => "icmp sge",
                    BinOp::Eq => "icmp eq",
                    BinOp::Neq => "icmp ne",
                    BinOp::And => "and",
                    BinOp::Or => "or",
                    BinOp::BitAnd => "and",
                    BinOp::BitOr => "or",
                    BinOp::BitXor => "xor",
                    BinOp::Shl => "shl",
                    BinOp::Shr => "ashr",
                };

                let mut ir = left_ir;
                ir.push_str(&right_ir);
                ir.push_str(&format!(
                    "  {} = {} i64 {}, {}\n",
                    tmp, op_str, left_val, right_val
                ));

                Ok((tmp, ir))
            }

            Expr::Unary { op, expr: inner } => {
                let (val, val_ir) = self.generate_expr(inner, counter)?;
                let tmp = self.next_temp(counter);

                let mut ir = val_ir;
                match op {
                    UnaryOp::Neg => {
                        ir.push_str(&format!("  {} = sub i64 0, {}\n", tmp, val));
                    }
                    UnaryOp::Not => {
                        ir.push_str(&format!("  {} = xor i1 {}, 1\n", tmp, val));
                    }
                    UnaryOp::BitNot => {
                        ir.push_str(&format!("  {} = xor i64 {}, -1\n", tmp, val));
                    }
                }

                Ok((tmp, ir))
            }

            Expr::Ternary { cond, then, else_ } => {
                let (cond_val, cond_ir) = self.generate_expr(cond, counter)?;
                let (then_val, then_ir) = self.generate_expr(then, counter)?;
                let (else_val, else_ir) = self.generate_expr(else_, counter)?;

                let tmp = self.next_temp(counter);
                let mut ir = cond_ir;
                ir.push_str(&then_ir);
                ir.push_str(&else_ir);
                ir.push_str(&format!(
                    "  {} = select i1 {}, i64 {}, i64 {}\n",
                    tmp, cond_val, then_val, else_val
                ));

                Ok((tmp, ir))
            }

            Expr::Call { func, args } => {
                let mut ir = String::new();
                let mut arg_vals = Vec::new();

                for arg in args {
                    let (val, arg_ir) = self.generate_expr(arg, counter)?;
                    ir.push_str(&arg_ir);
                    arg_vals.push(format!("i64 {}", val));
                }

                let fn_name = if let Expr::Ident(name) = &func.node {
                    name.clone()
                } else if let Expr::SelfCall = &func.node {
                    self.current_function.clone().unwrap_or_default()
                } else {
                    return Err(CodegenError::Unsupported("indirect call".to_string()));
                };

                let tmp = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call i64 @{}({})\n",
                    tmp,
                    fn_name,
                    arg_vals.join(", ")
                ));

                Ok((tmp, ir))
            }

            // TODO: Implement remaining expression types
            _ => Err(CodegenError::Unsupported(format!("{:?}", expr.node))),
        }
    }

    fn next_temp(&self, counter: &mut usize) -> String {
        let tmp = format!("%{}", counter);
        *counter += 1;
        tmp
    }

    fn type_to_llvm(&self, ty: &ResolvedType) -> String {
        match ty {
            ResolvedType::I8 => "i8".to_string(),
            ResolvedType::I16 => "i16".to_string(),
            ResolvedType::I32 => "i32".to_string(),
            ResolvedType::I64 => "i64".to_string(),
            ResolvedType::I128 => "i128".to_string(),
            ResolvedType::U8 => "i8".to_string(),
            ResolvedType::U16 => "i16".to_string(),
            ResolvedType::U32 => "i32".to_string(),
            ResolvedType::U64 => "i64".to_string(),
            ResolvedType::U128 => "i128".to_string(),
            ResolvedType::F32 => "float".to_string(),
            ResolvedType::F64 => "double".to_string(),
            ResolvedType::Bool => "i1".to_string(),
            ResolvedType::Str => "i8*".to_string(),
            ResolvedType::Unit => "void".to_string(),
            ResolvedType::Array(inner) => format!("{}*", self.type_to_llvm(inner)),
            ResolvedType::Pointer(inner) => format!("{}*", self.type_to_llvm(inner)),
            ResolvedType::Ref(inner) => format!("{}*", self.type_to_llvm(inner)),
            ResolvedType::RefMut(inner) => format!("{}*", self.type_to_llvm(inner)),
            ResolvedType::Named { name, .. } => format!("%{}*", name),
            _ => "i64".to_string(), // Default fallback
        }
    }

    fn ast_type_to_resolved(&self, ty: &Type) -> ResolvedType {
        match ty {
            Type::Named { name, generics } => match name.as_str() {
                "i8" => ResolvedType::I8,
                "i16" => ResolvedType::I16,
                "i32" => ResolvedType::I32,
                "i64" => ResolvedType::I64,
                "i128" => ResolvedType::I128,
                "u8" => ResolvedType::U8,
                "u16" => ResolvedType::U16,
                "u32" => ResolvedType::U32,
                "u64" => ResolvedType::U64,
                "u128" => ResolvedType::U128,
                "f32" => ResolvedType::F32,
                "f64" => ResolvedType::F64,
                "bool" => ResolvedType::Bool,
                "str" => ResolvedType::Str,
                _ => ResolvedType::Named {
                    name: name.clone(),
                    generics: generics
                        .iter()
                        .map(|g| self.ast_type_to_resolved(&g.node))
                        .collect(),
                },
            },
            Type::Array(inner) => {
                ResolvedType::Array(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Pointer(inner) => {
                ResolvedType::Pointer(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Ref(inner) => ResolvedType::Ref(Box::new(self.ast_type_to_resolved(&inner.node))),
            Type::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(self.ast_type_to_resolved(&inner.node)))
            }
            Type::Unit => ResolvedType::Unit,
            _ => ResolvedType::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vais_parser::parse;

    #[test]
    fn test_simple_function() {
        let source = "F add(a:i64,b:i64)->i64=a+b";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @add"));
        assert!(ir.contains("add i64"));
    }

    #[test]
    fn test_fibonacci() {
        let source = "F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @fib"));
        assert!(ir.contains("call i64 @fib"));
    }
}
