//! C Code Generator
//!
//! Generates C code from compiled AOEL functions

use std::fmt::Write;

use aoel_ir::{OpCode, Value};
use aoel_lowering::CompiledFunction;

use crate::error::CodegenResult;

/// C 코드 생성기
pub struct CCodeGenerator {
    output: String,
    indent: usize,
}

impl CCodeGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
        }
    }

    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }

    fn writeln(&mut self, s: &str) {
        let _ = writeln!(self.output, "{}{}", self.indent_str(), s);
    }

    /// 전체 프로그램 생성
    pub fn generate(&mut self, functions: &[CompiledFunction]) -> CodegenResult<String> {
        self.generate_header();
        self.generate_runtime();

        for func in functions {
            self.generate_function(func)?;
        }

        self.generate_main(functions);

        Ok(self.output.clone())
    }

    fn generate_header(&mut self) {
        self.writeln("#include <stdio.h>");
        self.writeln("#include <stdlib.h>");
        self.writeln("#include <string.h>");
        self.writeln("#include <stdbool.h>");
        self.writeln("#include <stdint.h>");
        self.writeln("#include <math.h>");
        self.writeln("");
    }

    fn generate_runtime(&mut self) {
        // Value 타입 정의
        self.writeln("// AOEL Runtime Types");
        self.writeln("typedef enum { VAL_INT, VAL_FLOAT, VAL_BOOL, VAL_STRING, VAL_ARRAY, VAL_VOID } ValueType;");
        self.writeln("");
        self.writeln("typedef struct Value {");
        self.indent += 1;
        self.writeln("ValueType type;");
        self.writeln("union {");
        self.indent += 1;
        self.writeln("int64_t i;");
        self.writeln("double f;");
        self.writeln("bool b;");
        self.writeln("char* s;");
        self.writeln("struct { struct Value* items; size_t len; size_t cap; } arr;");
        self.indent -= 1;
        self.writeln("} data;");
        self.indent -= 1;
        self.writeln("} Value;");
        self.writeln("");

        // 헬퍼 함수들
        self.writeln("// Value constructors");
        self.writeln("static Value val_int(int64_t i) { Value v; v.type = VAL_INT; v.data.i = i; return v; }");
        self.writeln("static Value val_float(double f) { Value v; v.type = VAL_FLOAT; v.data.f = f; return v; }");
        self.writeln("static Value val_bool(bool b) { Value v; v.type = VAL_BOOL; v.data.b = b; return v; }");
        self.writeln("static Value val_void(void) { Value v; v.type = VAL_VOID; return v; }");
        self.writeln("");

        // 산술 연산
        self.writeln("// Arithmetic operations");
        self.writeln("static Value val_add(Value a, Value b) {");
        self.indent += 1;
        self.writeln("if (a.type == VAL_INT && b.type == VAL_INT) return val_int(a.data.i + b.data.i);");
        self.writeln("if (a.type == VAL_FLOAT || b.type == VAL_FLOAT) {");
        self.indent += 1;
        self.writeln("double af = a.type == VAL_FLOAT ? a.data.f : (double)a.data.i;");
        self.writeln("double bf = b.type == VAL_FLOAT ? b.data.f : (double)b.data.i;");
        self.writeln("return val_float(af + bf);");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("return val_void();");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("");

        self.writeln("static Value val_sub(Value a, Value b) {");
        self.indent += 1;
        self.writeln("if (a.type == VAL_INT && b.type == VAL_INT) return val_int(a.data.i - b.data.i);");
        self.writeln("if (a.type == VAL_FLOAT || b.type == VAL_FLOAT) {");
        self.indent += 1;
        self.writeln("double af = a.type == VAL_FLOAT ? a.data.f : (double)a.data.i;");
        self.writeln("double bf = b.type == VAL_FLOAT ? b.data.f : (double)b.data.i;");
        self.writeln("return val_float(af - bf);");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("return val_void();");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("");

        self.writeln("static Value val_mul(Value a, Value b) {");
        self.indent += 1;
        self.writeln("if (a.type == VAL_INT && b.type == VAL_INT) return val_int(a.data.i * b.data.i);");
        self.writeln("if (a.type == VAL_FLOAT || b.type == VAL_FLOAT) {");
        self.indent += 1;
        self.writeln("double af = a.type == VAL_FLOAT ? a.data.f : (double)a.data.i;");
        self.writeln("double bf = b.type == VAL_FLOAT ? b.data.f : (double)b.data.i;");
        self.writeln("return val_float(af * bf);");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("return val_void();");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("");

        self.writeln("static Value val_div(Value a, Value b) {");
        self.indent += 1;
        self.writeln("if (a.type == VAL_INT && b.type == VAL_INT) return val_int(a.data.i / b.data.i);");
        self.writeln("if (a.type == VAL_FLOAT || b.type == VAL_FLOAT) {");
        self.indent += 1;
        self.writeln("double af = a.type == VAL_FLOAT ? a.data.f : (double)a.data.i;");
        self.writeln("double bf = b.type == VAL_FLOAT ? b.data.f : (double)b.data.i;");
        self.writeln("return val_float(af / bf);");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("return val_void();");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("");

        // 비교 연산
        self.writeln("// Comparison operations");
        self.writeln("static Value val_lt(Value a, Value b) {");
        self.indent += 1;
        self.writeln("if (a.type == VAL_INT && b.type == VAL_INT) return val_bool(a.data.i < b.data.i);");
        self.writeln("if (a.type == VAL_FLOAT || b.type == VAL_FLOAT) {");
        self.indent += 1;
        self.writeln("double af = a.type == VAL_FLOAT ? a.data.f : (double)a.data.i;");
        self.writeln("double bf = b.type == VAL_FLOAT ? b.data.f : (double)b.data.i;");
        self.writeln("return val_bool(af < bf);");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("return val_bool(false);");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("");

        self.writeln("static Value val_eq(Value a, Value b) {");
        self.indent += 1;
        self.writeln("if (a.type != b.type) return val_bool(false);");
        self.writeln("if (a.type == VAL_INT) return val_bool(a.data.i == b.data.i);");
        self.writeln("if (a.type == VAL_FLOAT) return val_bool(a.data.f == b.data.f);");
        self.writeln("if (a.type == VAL_BOOL) return val_bool(a.data.b == b.data.b);");
        self.writeln("return val_bool(false);");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("");

        self.writeln("static Value val_ne(Value a, Value b) {");
        self.indent += 1;
        self.writeln("return val_bool(!val_eq(a, b).data.b);");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("");

        self.writeln("static Value val_lte(Value a, Value b) {");
        self.indent += 1;
        self.writeln("if (a.type == VAL_INT && b.type == VAL_INT) return val_bool(a.data.i <= b.data.i);");
        self.writeln("if (a.type == VAL_FLOAT || b.type == VAL_FLOAT) {");
        self.indent += 1;
        self.writeln("double af = a.type == VAL_FLOAT ? a.data.f : (double)a.data.i;");
        self.writeln("double bf = b.type == VAL_FLOAT ? b.data.f : (double)b.data.i;");
        self.writeln("return val_bool(af <= bf);");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("return val_bool(false);");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("");

        self.writeln("static Value val_gte(Value a, Value b) {");
        self.indent += 1;
        self.writeln("if (a.type == VAL_INT && b.type == VAL_INT) return val_bool(a.data.i >= b.data.i);");
        self.writeln("if (a.type == VAL_FLOAT || b.type == VAL_FLOAT) {");
        self.indent += 1;
        self.writeln("double af = a.type == VAL_FLOAT ? a.data.f : (double)a.data.i;");
        self.writeln("double bf = b.type == VAL_FLOAT ? b.data.f : (double)b.data.i;");
        self.writeln("return val_bool(af >= bf);");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("return val_bool(false);");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("");

        // Print
        self.writeln("// Print");
        self.writeln("static void val_print(Value v) {");
        self.indent += 1;
        self.writeln("switch (v.type) {");
        self.indent += 1;
        self.writeln("case VAL_INT: printf(\"%lld\", (long long)v.data.i); break;");
        self.writeln("case VAL_FLOAT: printf(\"%g\", v.data.f); break;");
        self.writeln("case VAL_BOOL: printf(\"%s\", v.data.b ? \"true\" : \"false\"); break;");
        self.writeln("case VAL_STRING: printf(\"%s\", v.data.s); break;");
        self.writeln("case VAL_VOID: printf(\"()\"); break;");
        self.writeln("case VAL_ARRAY: {");
        self.indent += 1;
        self.writeln("printf(\"[\");");
        self.writeln("for (size_t i = 0; i < v.data.arr.len; i++) {");
        self.indent += 1;
        self.writeln("if (i > 0) printf(\", \");");
        self.writeln("val_print(v.data.arr.items[i]);");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("printf(\"]\");");
        self.writeln("break;");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("default: printf(\"<value>\"); break;");
        self.indent -= 1;
        self.writeln("}");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("");

        // Array creation
        self.writeln("// Array operations");
        self.writeln("static Value val_array_new(size_t cap) {");
        self.indent += 1;
        self.writeln("Value v;");
        self.writeln("v.type = VAL_ARRAY;");
        self.writeln("v.data.arr.items = (Value*)malloc(cap * sizeof(Value));");
        self.writeln("v.data.arr.len = 0;");
        self.writeln("v.data.arr.cap = cap;");
        self.writeln("return v;");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("");

        self.writeln("static void val_array_push(Value* arr, Value elem) {");
        self.indent += 1;
        self.writeln("if (arr->data.arr.len >= arr->data.arr.cap) {");
        self.indent += 1;
        self.writeln("arr->data.arr.cap = arr->data.arr.cap == 0 ? 8 : arr->data.arr.cap * 2;");
        self.writeln("arr->data.arr.items = (Value*)realloc(arr->data.arr.items, arr->data.arr.cap * sizeof(Value));");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("arr->data.arr.items[arr->data.arr.len++] = elem;");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("");

        self.writeln("static Value val_array_get(Value arr, int64_t idx) {");
        self.indent += 1;
        self.writeln("if (idx < 0) idx += (int64_t)arr.data.arr.len;");
        self.writeln("if (idx < 0 || (size_t)idx >= arr.data.arr.len) return val_void();");
        self.writeln("return arr.data.arr.items[idx];");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("");

        self.writeln("static int64_t val_array_len(Value arr) {");
        self.indent += 1;
        self.writeln("return (int64_t)arr.data.arr.len;");
        self.indent -= 1;
        self.writeln("}");
        self.writeln("");
    }

    fn generate_function(&mut self, func: &CompiledFunction) -> CodegenResult<()> {
        // 함수 시그니처
        let params = func.params.iter()
            .map(|p| format!("Value {}", self.sanitize_name(p)))
            .collect::<Vec<_>>()
            .join(", ");

        if params.is_empty() {
            self.writeln(&format!("static Value {}(void) {{", self.sanitize_name(&func.name)));
        } else {
            self.writeln(&format!("static Value {}({}) {{", self.sanitize_name(&func.name), params));
        }
        self.indent += 1;

        // 스택 변수
        self.writeln("Value _stack[256];");
        self.writeln("int _sp = 0;");
        self.writeln("");

        // 명령어 생성
        self.generate_instructions(&func.instructions, &func.name)?;

        self.indent -= 1;
        self.writeln("}");
        self.writeln("");

        Ok(())
    }

    fn generate_instructions(&mut self, instructions: &[aoel_ir::Instruction], func_name: &str) -> CodegenResult<()> {
        let mut i = 0;
        while i < instructions.len() {
            let instr = &instructions[i];

            // 레이블
            self.writeln(&format!("L{}: ;", i));

            match &instr.opcode {
                OpCode::Const(value) => {
                    let val_str = self.value_to_c(value);
                    self.writeln(&format!("_stack[_sp++] = {};", val_str));
                }
                OpCode::Load(name) => {
                    self.writeln(&format!("_stack[_sp++] = {};", self.sanitize_name(name)));
                }
                OpCode::Store(name) => {
                    self.writeln(&format!("{} = _stack[--_sp];", self.sanitize_name(name)));
                }
                OpCode::Add => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_add(a, b); }");
                }
                OpCode::Sub => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_sub(a, b); }");
                }
                OpCode::Mul => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_mul(a, b); }");
                }
                OpCode::Div => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_div(a, b); }");
                }
                OpCode::Lt => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_lt(a, b); }");
                }
                OpCode::Gt => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_lt(b, a); }");
                }
                OpCode::Eq => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_eq(a, b); }");
                }
                OpCode::Neq => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_ne(a, b); }");
                }
                OpCode::Lte => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_lte(a, b); }");
                }
                OpCode::Gte => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_gte(a, b); }");
                }
                OpCode::And => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_bool(a.data.b && b.data.b); }");
                }
                OpCode::Or => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_bool(a.data.b || b.data.b); }");
                }
                OpCode::Not => {
                    self.writeln("{ Value a = _stack[--_sp]; _stack[_sp++] = val_bool(!a.data.b); }");
                }
                OpCode::Neg => {
                    self.writeln("{ Value a = _stack[--_sp]; _stack[_sp++] = a.type == VAL_INT ? val_int(-a.data.i) : val_float(-a.data.f); }");
                }
                OpCode::Jump(offset) => {
                    let target = (i as i32 + offset + 1) as usize;
                    self.writeln(&format!("goto L{};", target));
                }
                OpCode::JumpIf(offset) => {
                    let target = (i as i32 + offset + 1) as usize;
                    self.writeln(&format!("if (_stack[--_sp].data.b) goto L{};", target));
                }
                OpCode::JumpIfNot(offset) => {
                    let target = (i as i32 + offset + 1) as usize;
                    self.writeln(&format!("if (!_stack[--_sp].data.b) goto L{};", target));
                }
                OpCode::Call(name, arg_count) => {
                    // 먼저 인자 준비 (스택 포인터 조정 전에)
                    let func_c_name = self.sanitize_name(name);
                    if *arg_count > 0 {
                        // 블록 스코프로 감싸서 변수 충돌 방지
                        self.writeln("{");
                        self.indent += 1;
                        for j in 0..*arg_count {
                            self.writeln(&format!("Value _arg{} = _stack[_sp - {} - 1];", j, arg_count - j - 1));
                        }
                        self.writeln(&format!("_sp -= {};", arg_count));
                        let args = (0..*arg_count)
                            .map(|j| format!("_arg{}", j))
                            .collect::<Vec<_>>()
                            .join(", ");
                        self.writeln(&format!("_stack[_sp++] = {}({});", func_c_name, args));
                        self.indent -= 1;
                        self.writeln("}");
                    } else {
                        self.writeln(&format!("_stack[_sp++] = {}();", func_c_name));
                    }
                }
                OpCode::SelfCall(arg_count) => {
                    // 먼저 인자 준비 (스택 포인터 조정 전에)
                    let func_c_name = self.sanitize_name(func_name);
                    if *arg_count > 0 {
                        // 블록 스코프로 감싸서 변수 충돌 방지
                        self.writeln("{");
                        self.indent += 1;
                        for j in 0..*arg_count {
                            self.writeln(&format!("Value _arg{} = _stack[_sp - {} - 1];", j, arg_count - j - 1));
                        }
                        self.writeln(&format!("_sp -= {};", arg_count));
                        let args = (0..*arg_count)
                            .map(|j| format!("_arg{}", j))
                            .collect::<Vec<_>>()
                            .join(", ");
                        self.writeln(&format!("_stack[_sp++] = {}({});", func_c_name, args));
                        self.indent -= 1;
                        self.writeln("}");
                    } else {
                        self.writeln(&format!("_stack[_sp++] = {}();", func_c_name));
                    }
                }
                OpCode::Return => {
                    self.writeln("return _stack[--_sp];");
                }
                OpCode::Pop => {
                    self.writeln("_sp--;");
                }
                OpCode::Dup => {
                    self.writeln("_stack[_sp] = _stack[_sp - 1]; _sp++;");
                }
                OpCode::Nop => {
                    self.writeln("/* nop */");
                }
                OpCode::MakeArray(count) => {
                    self.writeln("{");
                    self.indent += 1;
                    self.writeln(&format!("Value _arr = val_array_new({});", count));
                    // 스택에서 역순으로 요소 추가
                    self.writeln(&format!("_sp -= {};", count));
                    for j in 0..*count {
                        self.writeln(&format!("val_array_push(&_arr, _stack[_sp + {}]);", j));
                    }
                    self.writeln("_stack[_sp++] = _arr;");
                    self.indent -= 1;
                    self.writeln("}");
                }
                OpCode::Len => {
                    self.writeln("{ Value _a = _stack[--_sp]; _stack[_sp++] = val_int(val_array_len(_a)); }");
                }
                OpCode::Index => {
                    self.writeln("{ Value _idx = _stack[--_sp]; Value _arr = _stack[--_sp]; _stack[_sp++] = val_array_get(_arr, _idx.data.i); }");
                }
                OpCode::Map(transform_instrs) => {
                    // Map: 배열의 각 요소에 transform 적용
                    self.writeln("{");
                    self.indent += 1;
                    self.writeln("Value _src_arr = _stack[--_sp];");
                    self.writeln("Value _result_arr = val_array_new(_src_arr.data.arr.len);");
                    self.writeln("for (size_t _i = 0; _i < _src_arr.data.arr.len; _i++) {");
                    self.indent += 1;
                    self.writeln("Value _ = _src_arr.data.arr.items[_i];"); // _ 변수 바인딩
                    self.writeln("_stack[_sp++] = _;"); // 스택에 현재 요소 푸시
                    // transform 명령어 생성 (인라인)
                    self.generate_inline_instructions(transform_instrs, func_name)?;
                    self.writeln("Value _mapped = _stack[--_sp];");
                    self.writeln("val_array_push(&_result_arr, _mapped);");
                    self.indent -= 1;
                    self.writeln("}");
                    self.writeln("_stack[_sp++] = _result_arr;");
                    self.indent -= 1;
                    self.writeln("}");
                }
                OpCode::Filter(pred_instrs) => {
                    // Filter: 배열에서 predicate가 true인 요소만 유지
                    self.writeln("{");
                    self.indent += 1;
                    self.writeln("Value _src_arr = _stack[--_sp];");
                    self.writeln("Value _result_arr = val_array_new(_src_arr.data.arr.len);");
                    self.writeln("for (size_t _i = 0; _i < _src_arr.data.arr.len; _i++) {");
                    self.indent += 1;
                    self.writeln("Value _ = _src_arr.data.arr.items[_i];");
                    self.writeln("_stack[_sp++] = _;");
                    // predicate 명령어 생성 (인라인)
                    self.generate_inline_instructions(pred_instrs, func_name)?;
                    self.writeln("Value _pred_result = _stack[--_sp];");
                    self.writeln("if (_pred_result.data.b) {");
                    self.indent += 1;
                    self.writeln("val_array_push(&_result_arr, _);");
                    self.indent -= 1;
                    self.writeln("}");
                    self.indent -= 1;
                    self.writeln("}");
                    self.writeln("_stack[_sp++] = _result_arr;");
                    self.indent -= 1;
                    self.writeln("}");
                }
                OpCode::Reduce(reduce_op, init_value) => {
                    // Reduce: 배열을 단일 값으로 축약
                    self.writeln("{");
                    self.indent += 1;
                    self.writeln("Value _src_arr = _stack[--_sp];");

                    match reduce_op {
                        aoel_ir::ReduceOp::Sum => {
                            self.writeln(&format!("Value _acc = {};", self.value_to_c(init_value)));
                            self.writeln("for (size_t _i = 0; _i < _src_arr.data.arr.len; _i++) {");
                            self.indent += 1;
                            self.writeln("_acc = val_add(_acc, _src_arr.data.arr.items[_i]);");
                            self.indent -= 1;
                            self.writeln("}");
                            self.writeln("_stack[_sp++] = _acc;");
                        }
                        aoel_ir::ReduceOp::Product => {
                            self.writeln(&format!("Value _acc = {};", self.value_to_c(init_value)));
                            self.writeln("for (size_t _i = 0; _i < _src_arr.data.arr.len; _i++) {");
                            self.indent += 1;
                            self.writeln("_acc = val_mul(_acc, _src_arr.data.arr.items[_i]);");
                            self.indent -= 1;
                            self.writeln("}");
                            self.writeln("_stack[_sp++] = _acc;");
                        }
                        aoel_ir::ReduceOp::Min => {
                            self.writeln("if (_src_arr.data.arr.len == 0) { _stack[_sp++] = val_void(); }");
                            self.writeln("else {");
                            self.indent += 1;
                            self.writeln("Value _acc = _src_arr.data.arr.items[0];");
                            self.writeln("for (size_t _i = 1; _i < _src_arr.data.arr.len; _i++) {");
                            self.indent += 1;
                            self.writeln("if (val_lt(_src_arr.data.arr.items[_i], _acc).data.b) _acc = _src_arr.data.arr.items[_i];");
                            self.indent -= 1;
                            self.writeln("}");
                            self.writeln("_stack[_sp++] = _acc;");
                            self.indent -= 1;
                            self.writeln("}");
                        }
                        aoel_ir::ReduceOp::Max => {
                            self.writeln("if (_src_arr.data.arr.len == 0) { _stack[_sp++] = val_void(); }");
                            self.writeln("else {");
                            self.indent += 1;
                            self.writeln("Value _acc = _src_arr.data.arr.items[0];");
                            self.writeln("for (size_t _i = 1; _i < _src_arr.data.arr.len; _i++) {");
                            self.indent += 1;
                            self.writeln("if (val_lt(_acc, _src_arr.data.arr.items[_i]).data.b) _acc = _src_arr.data.arr.items[_i];");
                            self.indent -= 1;
                            self.writeln("}");
                            self.writeln("_stack[_sp++] = _acc;");
                            self.indent -= 1;
                            self.writeln("}");
                        }
                        aoel_ir::ReduceOp::All => {
                            self.writeln("bool _acc = true;");
                            self.writeln("for (size_t _i = 0; _i < _src_arr.data.arr.len; _i++) {");
                            self.indent += 1;
                            self.writeln("if (!_src_arr.data.arr.items[_i].data.b) { _acc = false; break; }");
                            self.indent -= 1;
                            self.writeln("}");
                            self.writeln("_stack[_sp++] = val_bool(_acc);");
                        }
                        aoel_ir::ReduceOp::Any => {
                            self.writeln("bool _acc = false;");
                            self.writeln("for (size_t _i = 0; _i < _src_arr.data.arr.len; _i++) {");
                            self.indent += 1;
                            self.writeln("if (_src_arr.data.arr.items[_i].data.b) { _acc = true; break; }");
                            self.indent -= 1;
                            self.writeln("}");
                            self.writeln("_stack[_sp++] = val_bool(_acc);");
                        }
                        _ => {
                            self.writeln("/* unsupported reduce operation */");
                            self.writeln("_stack[_sp++] = val_void();");
                        }
                    }

                    self.indent -= 1;
                    self.writeln("}");
                }
                OpCode::CallBuiltin(name, arg_count) => {
                    self.writeln("{");
                    self.indent += 1;
                    // 인자 가져오기
                    for j in 0..*arg_count {
                        self.writeln(&format!("Value _arg{} = _stack[_sp - {} - 1];", j, arg_count - j - 1));
                    }
                    self.writeln(&format!("_sp -= {};", arg_count));

                    // 빌트인 함수 호출
                    match name.to_uppercase().as_str() {
                        "SQRT" => {
                            self.writeln("double _v = _arg0.type == VAL_INT ? (double)_arg0.data.i : _arg0.data.f;");
                            self.writeln("_stack[_sp++] = val_float(sqrt(_v));");
                        }
                        "ABS" => {
                            self.writeln("if (_arg0.type == VAL_INT) _stack[_sp++] = val_int(llabs(_arg0.data.i));");
                            self.writeln("else _stack[_sp++] = val_float(fabs(_arg0.data.f));");
                        }
                        "POW" => {
                            self.writeln("double _base = _arg0.type == VAL_INT ? (double)_arg0.data.i : _arg0.data.f;");
                            self.writeln("double _exp = _arg1.type == VAL_INT ? (double)_arg1.data.i : _arg1.data.f;");
                            self.writeln("_stack[_sp++] = val_float(pow(_base, _exp));");
                        }
                        "SIN" => {
                            self.writeln("double _v = _arg0.type == VAL_INT ? (double)_arg0.data.i : _arg0.data.f;");
                            self.writeln("_stack[_sp++] = val_float(sin(_v));");
                        }
                        "COS" => {
                            self.writeln("double _v = _arg0.type == VAL_INT ? (double)_arg0.data.i : _arg0.data.f;");
                            self.writeln("_stack[_sp++] = val_float(cos(_v));");
                        }
                        "TAN" => {
                            self.writeln("double _v = _arg0.type == VAL_INT ? (double)_arg0.data.i : _arg0.data.f;");
                            self.writeln("_stack[_sp++] = val_float(tan(_v));");
                        }
                        "LOG" => {
                            self.writeln("double _v = _arg0.type == VAL_INT ? (double)_arg0.data.i : _arg0.data.f;");
                            self.writeln("_stack[_sp++] = val_float(log(_v));");
                        }
                        "LOG10" => {
                            self.writeln("double _v = _arg0.type == VAL_INT ? (double)_arg0.data.i : _arg0.data.f;");
                            self.writeln("_stack[_sp++] = val_float(log10(_v));");
                        }
                        "FLOOR" => {
                            self.writeln("double _v = _arg0.type == VAL_INT ? (double)_arg0.data.i : _arg0.data.f;");
                            self.writeln("_stack[_sp++] = val_int((int64_t)floor(_v));");
                        }
                        "CEIL" => {
                            self.writeln("double _v = _arg0.type == VAL_INT ? (double)_arg0.data.i : _arg0.data.f;");
                            self.writeln("_stack[_sp++] = val_int((int64_t)ceil(_v));");
                        }
                        "ROUND" => {
                            self.writeln("double _v = _arg0.type == VAL_INT ? (double)_arg0.data.i : _arg0.data.f;");
                            self.writeln("_stack[_sp++] = val_int((int64_t)round(_v));");
                        }
                        "MIN" => {
                            self.writeln("if (_arg0.type == VAL_INT && _arg1.type == VAL_INT)");
                            self.writeln("    _stack[_sp++] = val_int(_arg0.data.i < _arg1.data.i ? _arg0.data.i : _arg1.data.i);");
                            self.writeln("else {");
                            self.writeln("    double a = _arg0.type == VAL_INT ? (double)_arg0.data.i : _arg0.data.f;");
                            self.writeln("    double b = _arg1.type == VAL_INT ? (double)_arg1.data.i : _arg1.data.f;");
                            self.writeln("    _stack[_sp++] = val_float(a < b ? a : b);");
                            self.writeln("}");
                        }
                        "MAX" => {
                            self.writeln("if (_arg0.type == VAL_INT && _arg1.type == VAL_INT)");
                            self.writeln("    _stack[_sp++] = val_int(_arg0.data.i > _arg1.data.i ? _arg0.data.i : _arg1.data.i);");
                            self.writeln("else {");
                            self.writeln("    double a = _arg0.type == VAL_INT ? (double)_arg0.data.i : _arg0.data.f;");
                            self.writeln("    double b = _arg1.type == VAL_INT ? (double)_arg1.data.i : _arg1.data.f;");
                            self.writeln("    _stack[_sp++] = val_float(a > b ? a : b);");
                            self.writeln("}");
                        }
                        "LEN" => {
                            self.writeln("_stack[_sp++] = val_int(val_array_len(_arg0));");
                        }
                        "PRINT" | "PRINTLN" => {
                            for j in 0..*arg_count {
                                self.writeln(&format!("val_print(_arg{});", j));
                            }
                            self.writeln("printf(\"\\n\");");
                            self.writeln("_stack[_sp++] = val_void();");
                        }
                        _ => {
                            self.writeln(&format!("/* unsupported builtin: {} */", name));
                            self.writeln("_stack[_sp++] = val_void();");
                        }
                    }
                    self.indent -= 1;
                    self.writeln("}");
                }
                OpCode::MakeClosure(_, _) => {
                    // Closures are not supported in C codegen yet
                    self.writeln("/* closure not supported in C codegen */");
                    self.writeln("_stack[_sp++] = val_void();");
                }
                OpCode::CallClosure(_) => {
                    // Closures are not supported in C codegen yet
                    self.writeln("/* closure call not supported in C codegen */");
                    self.writeln("_stack[_sp++] = val_void();");
                }
                _ => {
                    self.writeln(&format!("/* unsupported: {:?} */", instr.opcode));
                }
            }

            i += 1;
        }

        // 함수 끝에 도달하면 스택에 값이 있으면 반환, 없으면 void 반환
        self.writeln("if (_sp > 0) return _stack[--_sp];");
        self.writeln("return val_void();");

        Ok(())
    }

    fn generate_main(&mut self, functions: &[CompiledFunction]) {
        self.writeln("int main(int argc, char* argv[]) {");
        self.indent += 1;

        // __main__ 함수가 있으면 호출
        if functions.iter().any(|f| f.name == "__main__") {
            let main_func_name = self.sanitize_name("__main__");
            self.writeln(&format!("Value result = {}();", main_func_name));
            self.writeln("val_print(result);");
            self.writeln("printf(\"\\n\");");
        }

        self.writeln("return 0;");
        self.indent -= 1;
        self.writeln("}");
    }

    fn value_to_c(&self, value: &Value) -> String {
        match value {
            Value::Int(i) => format!("val_int({}LL)", i),
            Value::Float(f) => format!("val_float({})", f),
            Value::Bool(b) => format!("val_bool({})", if *b { "true" } else { "false" }),
            Value::String(s) => format!("(Value){{ .type = VAL_STRING, .data.s = \"{}\" }}", s.replace('\\', "\\\\").replace('"', "\\\"")),
            Value::Void => "val_void()".to_string(),
            _ => "val_void()".to_string(),
        }
    }

    fn sanitize_name(&self, name: &str) -> String {
        // C 키워드 충돌 방지 및 특수문자 처리
        let sanitized = name.replace(['-', '.'], "_");
        format!("aoel_{}", sanitized)
    }

    /// 인라인 명령어 생성 (Map/Filter 내부용)
    /// 레이블 없이 명령어만 생성
    fn generate_inline_instructions(&mut self, instructions: &[aoel_ir::Instruction], _func_name: &str) -> CodegenResult<()> {
        for instr in instructions {
            match &instr.opcode {
                OpCode::Const(value) => {
                    let val_str = self.value_to_c(value);
                    self.writeln(&format!("_stack[_sp++] = {};", val_str));
                }
                OpCode::Load(name) => {
                    // _ 변수는 현재 요소를 가리킴
                    if name == "_" {
                        self.writeln("_stack[_sp++] = _;");
                    } else {
                        self.writeln(&format!("_stack[_sp++] = {};", self.sanitize_name(name)));
                    }
                }
                OpCode::Add => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_add(a, b); }");
                }
                OpCode::Sub => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_sub(a, b); }");
                }
                OpCode::Mul => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_mul(a, b); }");
                }
                OpCode::Div => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_div(a, b); }");
                }
                OpCode::Lt => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_lt(a, b); }");
                }
                OpCode::Gt => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_lt(b, a); }");
                }
                OpCode::Eq => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_eq(a, b); }");
                }
                OpCode::Neq => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_ne(a, b); }");
                }
                OpCode::Lte => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_lte(a, b); }");
                }
                OpCode::Gte => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_gte(a, b); }");
                }
                OpCode::And => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_bool(a.data.b && b.data.b); }");
                }
                OpCode::Or => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_bool(a.data.b || b.data.b); }");
                }
                OpCode::Not => {
                    self.writeln("{ Value a = _stack[--_sp]; _stack[_sp++] = val_bool(!a.data.b); }");
                }
                OpCode::Neg => {
                    self.writeln("{ Value a = _stack[--_sp]; _stack[_sp++] = a.type == VAL_INT ? val_int(-a.data.i) : val_float(-a.data.f); }");
                }
                OpCode::Mod => {
                    self.writeln("{ Value b = _stack[--_sp]; Value a = _stack[--_sp]; _stack[_sp++] = val_int(a.data.i % b.data.i); }");
                }
                OpCode::Pop => {
                    self.writeln("_sp--;");
                }
                _ => {
                    self.writeln(&format!("/* inline unsupported: {:?} */", instr.opcode));
                }
            }
        }
        Ok(())
    }
}

impl Default for CCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// 편의 함수: 컴파일된 함수들을 C 코드로 변환
pub fn generate_c(functions: &[CompiledFunction]) -> CodegenResult<String> {
    let mut gen = CCodeGenerator::new();
    gen.generate(functions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoel_ir::{Instruction, OpCode, Value};

    #[test]
    fn test_simple_function() {
        let func = CompiledFunction {
            name: "add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            instructions: vec![
                Instruction::new(OpCode::Load("a".to_string())),
                Instruction::new(OpCode::Load("b".to_string())),
                Instruction::new(OpCode::Add),
                Instruction::new(OpCode::Return),
            ],
        };

        let code = generate_c(&[func]).unwrap();
        assert!(code.contains("static Value aoel_add"));
        assert!(code.contains("val_add"));
    }

    #[test]
    fn test_factorial() {
        let func = CompiledFunction {
            name: "fact".to_string(),
            params: vec!["n".to_string()],
            instructions: vec![
                Instruction::new(OpCode::Load("n".to_string())),
                Instruction::new(OpCode::Const(Value::Int(2))),
                Instruction::new(OpCode::Lt),
                Instruction::new(OpCode::JumpIfNot(2)),
                Instruction::new(OpCode::Const(Value::Int(1))),
                Instruction::new(OpCode::Return),
                Instruction::new(OpCode::Load("n".to_string())),
                Instruction::new(OpCode::Load("n".to_string())),
                Instruction::new(OpCode::Const(Value::Int(1))),
                Instruction::new(OpCode::Sub),
                Instruction::new(OpCode::SelfCall(1)),
                Instruction::new(OpCode::Mul),
                Instruction::new(OpCode::Return),
            ],
        };

        let code = generate_c(&[func]).unwrap();
        assert!(code.contains("static Value aoel_fact"));
        assert!(code.contains("aoel_fact(")); // recursive call
    }

    #[test]
    fn test_make_array() {
        let func = CompiledFunction {
            name: "test_arr".to_string(),
            params: vec![],
            instructions: vec![
                Instruction::new(OpCode::Const(Value::Int(1))),
                Instruction::new(OpCode::Const(Value::Int(2))),
                Instruction::new(OpCode::Const(Value::Int(3))),
                Instruction::new(OpCode::MakeArray(3)),
                Instruction::new(OpCode::Return),
            ],
        };

        let code = generate_c(&[func]).unwrap();
        assert!(code.contains("val_array_new"));
        assert!(code.contains("val_array_push"));
    }

    #[test]
    fn test_map_operation() {
        let transform = vec![
            Instruction::new(OpCode::Load("_".to_string())),
            Instruction::new(OpCode::Const(Value::Int(2))),
            Instruction::new(OpCode::Mul),
        ];

        let func = CompiledFunction {
            name: "double_all".to_string(),
            params: vec!["arr".to_string()],
            instructions: vec![
                Instruction::new(OpCode::Load("arr".to_string())),
                Instruction::new(OpCode::Map(Box::new(transform))),
                Instruction::new(OpCode::Return),
            ],
        };

        let code = generate_c(&[func]).unwrap();
        assert!(code.contains("_result_arr"));
        assert!(code.contains("val_mul"));
    }

    #[test]
    fn test_reduce_sum() {
        use aoel_ir::ReduceOp;

        let func = CompiledFunction {
            name: "sum_all".to_string(),
            params: vec!["arr".to_string()],
            instructions: vec![
                Instruction::new(OpCode::Load("arr".to_string())),
                Instruction::new(OpCode::Reduce(ReduceOp::Sum, Value::Int(0))),
                Instruction::new(OpCode::Return),
            ],
        };

        let code = generate_c(&[func]).unwrap();
        assert!(code.contains("val_add"));
        assert!(code.contains("_acc"));
    }
}
