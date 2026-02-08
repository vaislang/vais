//! MIR → LLVM IR text emission.
//!
//! Converts optimized MIR to textual LLVM IR (.ll format).
//! This provides an alternative path from MIR directly to LLVM IR
//! without going through the AST-based codegen.

use crate::types::*;
use std::collections::HashMap;

/// Emit a MIR module as LLVM IR text.
pub fn emit_llvm_ir(module: &MirModule, target_triple: &str) -> String {
    let mut emitter = LlvmEmitter::new(target_triple);
    emitter.emit_module(module)
}

struct LlvmEmitter {
    target_triple: String,
    output: String,
    local_names: HashMap<Local, String>,
    next_unnamed: u32,
}

impl LlvmEmitter {
    fn new(target_triple: &str) -> Self {
        Self {
            target_triple: target_triple.to_string(),
            output: String::new(),
            local_names: HashMap::new(),
            next_unnamed: 0,
        }
    }

    fn emit_module(&mut self, module: &MirModule) -> String {
        // Module preamble
        self.output
            .push_str(&format!("; ModuleID = '{}'\n", module.name));
        self.output
            .push_str(&format!("target triple = \"{}\"\n\n", self.target_triple));

        // Extern declarations for common builtins
        self.output.push_str("declare i32 @putchar(i32)\n");
        self.output.push_str("declare i32 @printf(i8*, ...)\n\n");

        // Struct type definitions
        for (name, fields) in &module.structs {
            self.output.push_str(&format!("%{} = type {{ ", name));
            for (i, (_, ty)) in fields.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                self.output.push_str(&self.mir_type_to_llvm(ty));
            }
            self.output.push_str(" }\n");
        }
        if !module.structs.is_empty() {
            self.output.push('\n');
        }

        // Function bodies
        for body in &module.bodies {
            self.emit_body(body);
            self.output.push('\n');
        }

        self.output.clone()
    }

    fn emit_body(&mut self, body: &Body) {
        self.local_names.clear();
        self.next_unnamed = 0;

        // Map locals to LLVM names
        for (i, _local) in body.locals.iter().enumerate() {
            let name = format!("%_{}", i);
            self.local_names.insert(Local(i as u32), name);
        }

        // Function signature
        let ret_type = self.mir_type_to_llvm(&body.return_type);
        self.output
            .push_str(&format!("define {} @{}(", ret_type, body.name));

        for (i, param_ty) in body.params.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            let llvm_ty = self.mir_type_to_llvm(param_ty);
            let param_local = Local((i + 1) as u32);
            let name = self
                .local_names
                .get(&param_local)
                .cloned()
                .unwrap_or_else(|| format!("%_arg{}", i));
            self.output.push_str(&format!("{} {}", llvm_ty, name));
        }

        self.output.push_str(") {\n");

        // Allocate local variables (alloca for return place and temporaries)
        for (i, local_decl) in body.locals.iter().enumerate() {
            // Skip parameters (they are function arguments)
            if i == 0 || (i > 0 && i <= body.params.len()) {
                continue;
            }
            let llvm_ty = self.mir_type_to_llvm(&local_decl.ty);
            let name = self
                .local_names
                .get(&Local(i as u32))
                .cloned()
                .unwrap_or_else(|| format!("%_{}", i));
            self.output
                .push_str(&format!("  {}.addr = alloca {}\n", name, llvm_ty));
        }

        // Return place alloca
        if body.return_type != MirType::Unit {
            let ret_llvm = self.mir_type_to_llvm(&body.return_type);
            self.output
                .push_str(&format!("  %_0.addr = alloca {}\n", ret_llvm));
        }

        // Emit basic blocks
        for (i, bb) in body.basic_blocks.iter().enumerate() {
            if i > 0 {
                self.output.push_str(&format!("bb{}:\n", i));
            } else {
                self.output.push_str("entry:\n");
            }

            for stmt in &bb.statements {
                self.emit_statement(stmt, body);
            }

            if let Some(ref term) = bb.terminator {
                self.emit_terminator(term, body);
            }
        }

        self.output.push_str("}\n");
    }

    fn emit_statement(&mut self, stmt: &Statement, body: &Body) {
        match stmt {
            Statement::Assign(place, rvalue) => {
                self.emit_assign(place, rvalue, body);
            }
            Statement::Drop(_place) => {
                // Drop is a no-op in LLVM IR for now
                self.output.push_str("  ; drop\n");
            }
            Statement::Nop => {
                self.output.push_str("  ; nop\n");
            }
        }
    }

    fn emit_assign(&mut self, place: &Place, rvalue: &Rvalue, body: &Body) {
        let dest_name = self.local_name(place.local);
        let dest_ty = self.local_type(place.local, body);
        let llvm_ty = self.mir_type_to_llvm(&dest_ty);

        match rvalue {
            Rvalue::Use(operand) => {
                let val = self.emit_operand(operand, body);
                self.output
                    .push_str(&format!("  {} = add {} 0, {}\n", dest_name, llvm_ty, val));
            }
            Rvalue::BinaryOp(op, lhs, rhs) => {
                let lhs_val = self.emit_operand(lhs, body);
                let rhs_val = self.emit_operand(rhs, body);
                let llvm_op = self.binop_to_llvm(op, &dest_ty);
                self.output.push_str(&format!(
                    "  {} = {} {} {}, {}\n",
                    dest_name, llvm_op, llvm_ty, lhs_val, rhs_val
                ));
            }
            Rvalue::UnaryOp(op, operand) => {
                let val = self.emit_operand(operand, body);
                match op {
                    UnOp::Neg => {
                        self.output
                            .push_str(&format!("  {} = sub {} 0, {}\n", dest_name, llvm_ty, val));
                    }
                    UnOp::Not => {
                        self.output
                            .push_str(&format!("  {} = xor {} {}, -1\n", dest_name, llvm_ty, val));
                    }
                }
            }
            Rvalue::Cast(operand, target_ty) => {
                let val = self.emit_operand(operand, body);
                let src_ty = self.mir_type_to_llvm(&dest_ty);
                let dst_ty = self.mir_type_to_llvm(target_ty);
                // Simple bitcast for now
                if src_ty == dst_ty {
                    self.output
                        .push_str(&format!("  {} = add {} 0, {}\n", dest_name, src_ty, val));
                } else {
                    self.output.push_str(&format!(
                        "  {} = bitcast {} {} to {}\n",
                        dest_name, src_ty, val, dst_ty
                    ));
                }
            }
            Rvalue::Ref(ref_place) => {
                let ref_name = self.local_name(ref_place.local);
                self.output.push_str(&format!(
                    "  {} = getelementptr {}, ptr {}.addr, i32 0\n",
                    dest_name, llvm_ty, ref_name
                ));
            }
            Rvalue::Aggregate(_, _) => {
                // Aggregate construction - simplified
                self.output
                    .push_str(&format!("  ; aggregate construction for {}\n", dest_name));
                self.output
                    .push_str(&format!("  {} = add {} 0, 0\n", dest_name, llvm_ty));
            }
            Rvalue::Discriminant(disc_place) => {
                let disc_name = self.local_name(disc_place.local);
                self.output.push_str(&format!(
                    "  {} = add i64 0, {}  ; discriminant\n",
                    dest_name, disc_name
                ));
            }
            Rvalue::Len(len_place) => {
                let len_name = self.local_name(len_place.local);
                self.output.push_str(&format!(
                    "  {} = add i64 0, {}  ; len\n",
                    dest_name, len_name
                ));
            }
        }
    }

    fn emit_operand(&self, operand: &Operand, _body: &Body) -> String {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => self.local_name(place.local),
            Operand::Constant(c) => match c {
                Constant::Int(v) => v.to_string(),
                Constant::Float(v) => format!("{:.6e}", v),
                Constant::Bool(v) => {
                    if *v {
                        "1".to_string()
                    } else {
                        "0".to_string()
                    }
                }
                Constant::Str(s) => format!("\"{}\"", s),
                Constant::Unit => "0".to_string(),
            },
        }
    }

    fn emit_terminator(&mut self, term: &Terminator, body: &Body) {
        match term {
            Terminator::Goto(bb) => {
                let label = if bb.0 == 0 {
                    "entry".to_string()
                } else {
                    format!("bb{}", bb.0)
                };
                self.output.push_str(&format!("  br label %{}\n", label));
            }
            Terminator::SwitchInt {
                discriminant,
                targets,
                otherwise,
            } => {
                let disc_val = self.emit_operand(discriminant, body);
                if targets.len() == 1 {
                    // Single target = conditional branch
                    let (val, target) = &targets[0];
                    let cmp_name = self.next_temp();
                    self.output.push_str(&format!(
                        "  {} = icmp eq i64 {}, {}\n",
                        cmp_name, disc_val, val
                    ));
                    let t_label = if target.0 == 0 {
                        "entry".to_string()
                    } else {
                        format!("bb{}", target.0)
                    };
                    let f_label = if otherwise.0 == 0 {
                        "entry".to_string()
                    } else {
                        format!("bb{}", otherwise.0)
                    };
                    self.output.push_str(&format!(
                        "  br i1 {}, label %{}, label %{}\n",
                        cmp_name, t_label, f_label
                    ));
                } else {
                    // Multi-target = switch
                    let o_label = if otherwise.0 == 0 {
                        "entry".to_string()
                    } else {
                        format!("bb{}", otherwise.0)
                    };
                    self.output.push_str(&format!(
                        "  switch i64 {}, label %{} [\n",
                        disc_val, o_label
                    ));
                    for (val, target) in targets {
                        let t_label = if target.0 == 0 {
                            "entry".to_string()
                        } else {
                            format!("bb{}", target.0)
                        };
                        self.output
                            .push_str(&format!("    i64 {}, label %{}\n", val, t_label));
                    }
                    self.output.push_str("  ]\n");
                }
            }
            Terminator::Return => {
                let ret_ty = self.mir_type_to_llvm(&body.return_type);
                if body.return_type == MirType::Unit {
                    self.output.push_str("  ret void\n");
                } else {
                    let ret_name = self.local_name(Local(0));
                    self.output
                        .push_str(&format!("  ret {} {}\n", ret_ty, ret_name));
                }
            }
            Terminator::Call {
                func,
                args,
                destination,
                target,
            } => {
                let dest_name = self.local_name(destination.local);
                let dest_ty = self.local_type(destination.local, body);
                let llvm_ty = self.mir_type_to_llvm(&dest_ty);

                self.output
                    .push_str(&format!("  {} = call {} @{}(", dest_name, llvm_ty, func));
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    let val = self.emit_operand(arg, body);
                    // Infer argument type from operand
                    let arg_ty = self.operand_type(arg, body);
                    let arg_llvm = self.mir_type_to_llvm(&arg_ty);
                    self.output.push_str(&format!("{} {}", arg_llvm, val));
                }
                self.output.push_str(")\n");

                let t_label = if target.0 == 0 {
                    "entry".to_string()
                } else {
                    format!("bb{}", target.0)
                };
                self.output.push_str(&format!("  br label %{}\n", t_label));
            }
            Terminator::TailCall { func, args } => {
                let ret_ty = self.mir_type_to_llvm(&body.return_type);
                let ret_name = self.local_name(Local(0));

                self.output.push_str(&format!(
                    "  {} = musttail call {} @{}(",
                    ret_name, ret_ty, func
                ));
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    let val = self.emit_operand(arg, body);
                    let arg_ty = self.operand_type(arg, body);
                    let arg_llvm = self.mir_type_to_llvm(&arg_ty);
                    self.output.push_str(&format!("{} {}", arg_llvm, val));
                }
                self.output.push_str(")\n");
                self.output
                    .push_str(&format!("  ret {} {}\n", ret_ty, ret_name));
            }
            Terminator::Unreachable => {
                self.output.push_str("  unreachable\n");
            }
            Terminator::Assert {
                cond,
                expected,
                msg,
                target,
            } => {
                let cond_val = self.emit_operand(cond, body);
                let cmp_name = self.next_temp();
                let expected_val = if *expected { "1" } else { "0" };
                self.output.push_str(&format!(
                    "  {} = icmp eq i1 {}, {}\n",
                    cmp_name, cond_val, expected_val
                ));
                self.output.push_str(&format!("  ; assert: {}\n", msg));
                let t_label = if target.0 == 0 {
                    "entry".to_string()
                } else {
                    format!("bb{}", target.0)
                };
                self.output.push_str(&format!(
                    "  br i1 {}, label %{}, label %assert_fail\n",
                    cmp_name, t_label
                ));
            }
        }
    }

    fn local_name(&self, local: Local) -> String {
        self.local_names
            .get(&local)
            .cloned()
            .unwrap_or_else(|| format!("%_{}", local.0))
    }

    fn local_type(&self, local: Local, body: &Body) -> MirType {
        let idx = local.0 as usize;
        if idx < body.locals.len() {
            body.locals[idx].ty.clone()
        } else {
            MirType::I64 // fallback
        }
    }

    fn operand_type(&self, operand: &Operand, body: &Body) -> MirType {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => self.local_type(place.local, body),
            Operand::Constant(c) => match c {
                Constant::Int(_) => MirType::I64,
                Constant::Float(_) => MirType::F64,
                Constant::Bool(_) => MirType::Bool,
                Constant::Str(_) => MirType::Str,
                Constant::Unit => MirType::Unit,
            },
        }
    }

    fn next_temp(&mut self) -> String {
        let name = format!("%_tmp{}", self.next_unnamed);
        self.next_unnamed += 1;
        name
    }

    fn mir_type_to_llvm(&self, ty: &MirType) -> String {
        match ty {
            MirType::I8 | MirType::U8 => "i8".to_string(),
            MirType::I16 | MirType::U16 => "i16".to_string(),
            MirType::I32 | MirType::U32 => "i32".to_string(),
            MirType::I64 | MirType::U64 => "i64".to_string(),
            MirType::I128 | MirType::U128 => "i128".to_string(),
            MirType::F32 => "float".to_string(),
            MirType::F64 => "double".to_string(),
            MirType::Bool => "i1".to_string(),
            MirType::Str => "i8*".to_string(),
            MirType::Unit => "void".to_string(),
            MirType::Pointer(inner) => format!("{}*", self.mir_type_to_llvm(inner)),
            MirType::Ref(inner) => format!("{}*", self.mir_type_to_llvm(inner)),
            MirType::Array(elem) => format!("[0 x {}]", self.mir_type_to_llvm(elem)),
            MirType::Tuple(elems) => {
                let parts: Vec<String> = elems.iter().map(|t| self.mir_type_to_llvm(t)).collect();
                format!("{{ {} }}", parts.join(", "))
            }
            MirType::Struct(name) => format!("%{}", name),
            MirType::Enum(name) => format!("%{}", name),
            MirType::Function { params, ret } => {
                let ret_ty = self.mir_type_to_llvm(ret);
                let param_tys: Vec<String> =
                    params.iter().map(|t| self.mir_type_to_llvm(t)).collect();
                format!("{} ({})", ret_ty, param_tys.join(", "))
            }
            MirType::Never => "void".to_string(),
        }
    }

    fn binop_to_llvm(&self, op: &BinOp, ty: &MirType) -> &'static str {
        let is_float = matches!(ty, MirType::F32 | MirType::F64);
        match op {
            BinOp::Add => {
                if is_float {
                    "fadd"
                } else {
                    "add"
                }
            }
            BinOp::Sub => {
                if is_float {
                    "fsub"
                } else {
                    "sub"
                }
            }
            BinOp::Mul => {
                if is_float {
                    "fmul"
                } else {
                    "mul"
                }
            }
            BinOp::Div => {
                if is_float {
                    "fdiv"
                } else {
                    "sdiv"
                }
            }
            BinOp::Rem => {
                if is_float {
                    "frem"
                } else {
                    "srem"
                }
            }
            BinOp::BitAnd => "and",
            BinOp::BitOr => "or",
            BinOp::BitXor => "xor",
            BinOp::Shl => "shl",
            BinOp::Shr => "ashr",
            BinOp::Eq => "icmp eq",
            BinOp::Ne => "icmp ne",
            BinOp::Lt => {
                if is_float {
                    "fcmp olt"
                } else {
                    "icmp slt"
                }
            }
            BinOp::Le => {
                if is_float {
                    "fcmp ole"
                } else {
                    "icmp sle"
                }
            }
            BinOp::Gt => {
                if is_float {
                    "fcmp ogt"
                } else {
                    "icmp sgt"
                }
            }
            BinOp::Ge => {
                if is_float {
                    "fcmp oge"
                } else {
                    "icmp sge"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MirBuilder;

    #[test]
    fn test_emit_simple_function() {
        let mut builder = MirBuilder::new("add", vec![MirType::I64, MirType::I64], MirType::I64);

        let result = builder.new_local(MirType::I64, None);
        let param_a = Operand::Copy(Place::local(builder.param(0)));
        let param_b = Operand::Copy(Place::local(builder.param(1)));

        builder.assign_binop(result, BinOp::Add, param_a, param_b);
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(result))),
        );
        builder.return_();

        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);

        let ir = emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("define i64 @add("));
        assert!(ir.contains("add i64"));
        assert!(ir.contains("ret i64"));
    }

    #[test]
    fn test_emit_with_branch() {
        let mut builder = MirBuilder::new("abs", vec![MirType::I64], MirType::I64);

        let cond = builder.new_local(MirType::Bool, None);
        let bb_neg = builder.new_block();
        let bb_pos = builder.new_block();
        let bb_end = builder.new_block();

        let param = Operand::Copy(Place::local(builder.param(0)));

        // bb0: compare param < 0
        builder.assign(
            Place::local(cond),
            Rvalue::BinaryOp(
                BinOp::Lt,
                param.clone(),
                Operand::Constant(Constant::Int(0)),
            ),
        );
        builder.switch_int(Operand::Copy(Place::local(cond)), vec![(1, bb_neg)], bb_pos);

        // bb1 (neg): return -param
        builder.switch_to_block(bb_neg);
        builder.assign(
            builder.return_place(),
            Rvalue::UnaryOp(UnOp::Neg, param.clone()),
        );
        builder.goto(bb_end);

        // bb2 (pos): return param
        builder.switch_to_block(bb_pos);
        builder.assign(builder.return_place(), Rvalue::Use(param));
        builder.goto(bb_end);

        // bb3 (end): return
        builder.switch_to_block(bb_end);
        builder.return_();

        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);

        let ir = emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("define i64 @abs("));
        assert!(ir.contains("icmp eq i64"));
        assert!(ir.contains("br i1"));
        assert!(ir.contains("sub i64 0,"));
    }

    #[test]
    fn test_emit_float_operations() {
        let mut builder = MirBuilder::new("fmul", vec![MirType::F64, MirType::F64], MirType::F64);

        let result = builder.new_local(MirType::F64, None);
        let param_a = Operand::Copy(Place::local(builder.param(0)));
        let param_b = Operand::Copy(Place::local(builder.param(1)));

        builder.assign_binop(result, BinOp::Mul, param_a, param_b);
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(result))),
        );
        builder.return_();

        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);

        let ir = emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("define double @fmul("));
        assert!(ir.contains("fmul double"));
        assert!(ir.contains("ret double"));
    }

    #[test]
    fn test_emit_struct_type() {
        let mut module = MirModule::new("test");
        module.structs.insert(
            "Point".into(),
            vec![("x".into(), MirType::I64), ("y".into(), MirType::I64)],
        );

        let ir = emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("%Point = type { i64, i64 }"));
    }

    #[test]
    fn test_emit_multi_way_switch() {
        let mut builder = MirBuilder::new("classify", vec![MirType::I64], MirType::I64);

        let bb1 = builder.new_block();
        let bb2 = builder.new_block();
        let bb3 = builder.new_block();
        let bb_default = builder.new_block();

        let param = Operand::Copy(Place::local(builder.param(0)));

        // Switch with 3 cases
        builder.switch_int(param, vec![(0, bb1), (1, bb2), (2, bb3)], bb_default);

        builder.switch_to_block(bb1);
        builder.assign_const(builder.return_place().local, Constant::Int(100));
        builder.return_();

        builder.switch_to_block(bb2);
        builder.assign_const(builder.return_place().local, Constant::Int(200));
        builder.return_();

        builder.switch_to_block(bb3);
        builder.assign_const(builder.return_place().local, Constant::Int(300));
        builder.return_();

        builder.switch_to_block(bb_default);
        builder.assign_const(builder.return_place().local, Constant::Int(0));
        builder.return_();

        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);

        let ir = emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("switch i64"));
        assert!(ir.contains("i64 0, label %bb1"));
        assert!(ir.contains("i64 1, label %bb2"));
        assert!(ir.contains("i64 2, label %bb3"));
    }

    #[test]
    fn test_emit_unary_not() {
        let mut builder = MirBuilder::new("bitflip", vec![MirType::I64], MirType::I64);

        let result = builder.new_local(MirType::I64, None);
        let param = Operand::Copy(Place::local(builder.param(0)));

        builder.assign(Place::local(result), Rvalue::UnaryOp(UnOp::Not, param));
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(result))),
        );
        builder.return_();

        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);

        let ir = emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        assert!(ir.contains("xor i64"));
        assert!(ir.contains(", -1"));
    }

    #[test]
    fn test_emit_comparison_ops() {
        let mut builder =
            MirBuilder::new("compare", vec![MirType::I64, MirType::I64], MirType::I64);

        let result = builder.new_local(MirType::I64, None);
        let param_a = Operand::Copy(Place::local(builder.param(0)));
        let param_b = Operand::Copy(Place::local(builder.param(1)));

        // Test less-than comparison (result stored as i64, not bool)
        builder.assign_binop(result, BinOp::Lt, param_a, param_b);
        builder.assign(
            builder.return_place(),
            Rvalue::Use(Operand::Copy(Place::local(result))),
        );
        builder.return_();

        let body = builder.build();
        let mut module = MirModule::new("test");
        module.bodies.push(body);

        let ir = emit_llvm_ir(&module, "x86_64-unknown-linux-gnu");
        // Comparison ops emit "icmp slt" instruction
        assert!(ir.contains("icmp slt"));
    }
}
