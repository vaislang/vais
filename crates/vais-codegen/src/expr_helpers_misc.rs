//! Miscellaneous expression helpers for CodeGenerator
//!
//! Contains lambda, await, try/unwrap, and SIMD intrinsic generation.

use crate::types::ClosureInfo;
use crate::{CodeGenerator, CodegenError, CodegenResult, LocalVar};
use vais_ast::{Expr, Param, Spanned};
use vais_types::ResolvedType;

impl CodeGenerator {
    /// Extract the poll function name from an expression AST node (static analysis).
    /// Used for direct await codegen. For Let bindings that involve Spawn,
    /// prefer resolve_poll_func_name which can check is_async on the inner call.
    #[allow(dead_code)]
    pub(crate) fn extract_poll_func_name_from_expr(expr: &Expr) -> Option<String> {
        match expr {
            Expr::Call { func, .. } => {
                if let Expr::Ident(name) = &func.node {
                    Some(format!("{}__poll", name))
                } else {
                    None
                }
            }
            Expr::MethodCall { method, .. } => Some(format!("{}__poll", method.node)),
            Expr::Spawn(_) => {
                // Spawn requires runtime is_async check (via self.types.functions),
                // which static analysis cannot provide. Return None here;
                // callers with &self access should use resolve_poll_func_name instead.
                None
            }
            Expr::SelfCall => {
                // @ operator — self-recursion calls the current function
                None // caller must resolve from current_async_function
            }
            _ => None,
        }
    }

    /// Resolve the poll function name for an await expression, using `self` to
    /// check whether the called function is actually async. This is critical for
    /// Spawn expressions: if the inner call targets a non-async function, Spawn
    /// codegen wraps the result in a sync wrapper struct and we must use
    /// `__sync_spawn__poll` (not the inner function's `__poll`).
    pub(crate) fn resolve_poll_func_name(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Call { func, .. } => {
                if let Expr::Ident(name) = &func.node {
                    Some(format!("{}__poll", name))
                } else if let Expr::SelfCall = &func.node {
                    // @(args) — self-recursive call to current function
                    self.fn_ctx
                        .current_function
                        .as_ref()
                        .map(|fn_name| {
                            // current_function inside async poll is "name__poll",
                            // but the base name is what we need for the poll function
                            let base = fn_name.trim_end_matches("__poll");
                            format!("{}__poll", base)
                        })
                } else {
                    None
                }
            }
            Expr::MethodCall { method, .. } => Some(format!("{}__poll", method.node)),
            Expr::Spawn(inner) => {
                // Check whether the inner call is to an async function.
                // This must mirror generate_expr(Spawn)'s passthrough condition:
                // async calls produce a Future and are passed through, so we use
                // the inner function's __poll. Non-async calls get wrapped in a
                // sync struct, so we must use __sync_spawn__poll.
                if let Expr::Call { func, .. } = &inner.node {
                    if let Expr::Ident(name) = &func.node {
                        let is_async = self
                            .types
                            .functions
                            .get(name.as_str())
                            .is_some_and(|info| info.signature.is_async);
                        if is_async {
                            return Some(format!("{}__poll", name));
                        }
                    }
                }
                // For non-async or non-Call inner, Spawn creates a sync wrapper
                Some("__sync_spawn__poll".to_string())
            }
            Expr::SelfCall => None,
            _ => None,
        }
    }

    pub(crate) fn generate_await_expr(
        &mut self,
        inner: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (future_ptr, future_ir) = self.generate_expr(inner, counter)?;
        let mut ir = future_ir;

        // Determine the poll function to call:
        // 1. Try static AST analysis (direct call/spawn expressions)
        // 2. For Ident expressions, look up in future_poll_fns (variable-based await)
        // 3. Fall back to __sync_spawn__poll
        //
        // IMPORTANT: For Spawn(Call{fn}) expressions, the poll function must match
        // the pointer layout that Spawn codegen produced. Spawn codegen checks
        // is_async on the called function to decide between passthrough (async state
        // ptr → fn__poll) and sync wrapper ({state=-1, result} → __sync_spawn__poll).
        // extract_poll_func_name_from_expr must use the same is_async check to stay
        // consistent; otherwise we may call fn__poll with a sync wrapper pointer,
        // causing an infinite loop (state=-1 doesn't match any valid poll state).
        let poll_func = if let Some(name) = self.resolve_poll_func_name(&inner.node) {
            name
        } else if let Expr::Ident(var_name) = &inner.node {
            // Variable-based await: look up the recorded poll function
            self.fn_ctx
                .future_poll_fns
                .get(var_name)
                .cloned()
                .unwrap_or_else(|| "__sync_spawn__poll".to_string())
        } else {
            "__sync_spawn__poll".to_string()
        };

        // Ensure the __sync_spawn__poll function is emitted if referenced
        if poll_func == "__sync_spawn__poll" {
            self.needs_sync_spawn_poll = true;
        }

        // Determine the poll result type: { i64 status, <ret_llvm> value }
        // For async functions, the poll function returns the function's actual return type.
        // For __sync_spawn__poll, it always returns { i64, i64 }.
        let inner_ret_llvm = if poll_func == "__sync_spawn__poll" {
            "i64".to_string()
        } else {
            // Extract the async function name from the poll function name
            let async_fn_name = poll_func.trim_end_matches("__poll");
            // Look up the function's return type in the registry
            self.types
                .functions
                .get(async_fn_name)
                .map(|info| self.type_to_llvm(&info.signature.ret))
                .unwrap_or_else(|| {
                    // Also check resolved_function_sigs from the type checker
                    self.types
                        .resolved_function_sigs
                        .get(async_fn_name)
                        .map(|sig| self.type_to_llvm(&sig.ret))
                        .unwrap_or_else(|| "i64".to_string())
                })
        };
        let poll_ret_ty = format!("{{ i64, {} }}", inner_ret_llvm);

        let poll_start = self.next_label("await_poll");
        let poll_ready = self.next_label("await_ready");
        let poll_pending = self.next_label("await_pending");

        ir.push_str(&format!("  br label %{}\n\n", poll_start));
        ir.push_str(&format!("{}:\n", poll_start));

        let poll_result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call {} @{}(i64 {})\n",
            poll_result, poll_ret_ty, poll_func, future_ptr
        ));

        let status = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = extractvalue {} {}, 0\n",
            status, poll_ret_ty, poll_result
        ));

        let is_ready = self.next_temp(counter);
        ir.push_str(&format!("  {} = icmp eq i64 {}, 1\n", is_ready, status));
        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n\n",
            is_ready, poll_ready, poll_pending
        ));

        ir.push_str(&format!("{}:\n", poll_pending));
        // Yield CPU cooperatively instead of busy-waiting
        ir.push_str("  call i32 @sched_yield()\n");
        ir.push_str(&format!("  br label %{}\n\n", poll_start));

        ir.push_str(&format!("{}:\n", poll_ready));
        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = extractvalue {} {}, 1\n",
            result, poll_ret_ty, poll_result
        ));

        Ok((result, ir))
    }

    /// Generate lambda expression
    pub(crate) fn generate_lambda_expr(
        &mut self,
        params: &[Param],
        body: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let lambda_name = format!("__lambda_{}", self.fn_ctx.label_counter);
        self.fn_ctx.label_counter += 1;

        let capture_names = self.find_lambda_captures(params, body);

        let mut captured_vars: Vec<(String, ResolvedType, String)> = Vec::new();
        let mut capture_ir = String::new();

        for cap_name in &capture_names {
            if let Some(local) = self.fn_ctx.locals.get(cap_name) {
                let ty = local.ty.clone();
                if local.is_param() {
                    captured_vars.push((cap_name.clone(), ty, format!("%{}", local.llvm_name)));
                } else if local.is_ssa() {
                    // SSA values are already the value itself, use directly
                    captured_vars.push((cap_name.clone(), ty, local.llvm_name.clone()));
                } else {
                    let tmp = self.next_temp(counter);
                    let llvm_ty = self.type_to_llvm(&ty);
                    capture_ir.push_str(&format!(
                        "  {} = load {}, {}* %{}\n",
                        tmp, llvm_ty, llvm_ty, local.llvm_name
                    ));
                    captured_vars.push((cap_name.clone(), ty, tmp));
                }
            }
        }

        let mut param_strs = Vec::new();
        let mut param_types = Vec::new();

        for (cap_name, cap_ty, _) in &captured_vars {
            let llvm_ty = self.type_to_llvm(cap_ty);
            param_strs.push(format!("{} %__cap_{}", llvm_ty, cap_name));
            param_types.push(llvm_ty);
        }

        for p in params {
            let ty = self.ast_type_to_resolved(&p.ty.node);
            let llvm_ty = self.type_to_llvm(&ty);
            param_strs.push(format!("{} %{}", llvm_ty, p.name.node));
            param_types.push(llvm_ty);
        }

        // SAFETY: if generate_expr below returns Err, the entire codegen aborts,
        // so empty self.fn_ctx.locals after take is acceptable (never accessed post-error).
        let saved_function = self.fn_ctx.current_function.take();
        let saved_locals = std::mem::take(&mut self.fn_ctx.locals);

        self.fn_ctx.current_function = Some(lambda_name.clone());

        for (cap_name, cap_ty, _) in &captured_vars {
            self.fn_ctx.locals.insert(
                cap_name.clone(),
                LocalVar::param(cap_ty.clone(), format!("__cap_{}", cap_name)),
            );
        }

        for p in params {
            let ty = self.ast_type_to_resolved(&p.ty.node);
            self.fn_ctx.locals.insert(
                p.name.node.clone(),
                LocalVar::param(ty, p.name.node.clone()),
            );
        }

        let mut lambda_counter = 0;
        let (body_val, body_ir) = self.generate_expr(body, &mut lambda_counter)?;

        let mut lambda_ir = format!(
            "define i64 @{}({}) {{\nentry:\n",
            lambda_name,
            param_strs.join(", ")
        );
        lambda_ir.push_str(&body_ir);
        lambda_ir.push_str(&format!("  ret i64 {}\n}}\n", body_val));

        self.lambdas.generated_ir.push(lambda_ir);

        self.fn_ctx.current_function = saved_function;
        self.fn_ctx.locals = saved_locals;

        // Emit ptrtoint as a proper instruction (not a constant expression)
        // so the result is a clean SSA temp that can be used anywhere
        let fn_ptr_tmp = self.next_temp(counter);
        capture_ir.push_str(&format!(
            "  {} = ptrtoint i64 ({})* @{} to i64\n",
            fn_ptr_tmp,
            param_types.join(", "),
            lambda_name
        ));

        if captured_vars.is_empty() {
            self.lambdas.last_lambda_info = None;
            Ok((fn_ptr_tmp, capture_ir))
        } else {
            self.lambdas.last_lambda_info = Some(ClosureInfo {
                func_name: lambda_name.clone(),
                captures: captured_vars
                    .iter()
                    .map(|(name, _, val)| (name.clone(), val.clone()))
                    .collect(),
                is_ref_capture: false,
            });
            Ok((fn_ptr_tmp, capture_ir))
        }
    }

    /// Generate try expression — delegates to the canonical aggregate extractvalue implementation
    pub(crate) fn generate_try_expr(
        &mut self,
        inner: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let inner_type = self.infer_expr_type(inner);
        let llvm_type = self.type_to_llvm(&inner_type);

        let (inner_val, inner_ir) = self.generate_expr(inner, counter)?;
        let mut ir = inner_ir;

        ir.push_str("  ; Try expression (?)\n");

        // Determine the tag type based on the inner type:
        // - Optional/Result use i8 tag: { i8, T }
        // - User-defined enums use i32 tag: { i32, { T } }
        let (tag_type, extract_payload) = match &inner_type {
            ResolvedType::Optional(_) | ResolvedType::Result(_, _) => {
                ("i8", false)
            }
            _ => {
                ("i32", true)
            }
        };

        // Extract tag (field 0)
        let tag = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = extractvalue {} {}, 0\n",
            tag, llvm_type, inner_val
        ));

        // Check if Err (tag != 0)
        let is_err = self.next_temp(counter);
        let err_label = self.next_label("try_err");
        let ok_label = self.next_label("try_ok");
        let merge_label = self.next_label("try_merge");

        ir.push_str(&format!("  {} = icmp ne {} {}, 0\n", is_err, tag_type, tag));
        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n\n",
            is_err, err_label, ok_label
        ));

        // Err branch: early return
        ir.push_str(&format!("{}:\n", err_label));
        ir.push_str(&format!(
            "  ret {} {}  ; early return on Err\n\n",
            llvm_type, inner_val
        ));

        // Ok branch: extract payload value
        ir.push_str(&format!("{}:\n", ok_label));
        let value = self.next_temp(counter);
        if extract_payload {
            ir.push_str(&format!(
                "  {} = extractvalue {} {}, 1, 0\n",
                value, llvm_type, inner_val
            ));
        } else {
            ir.push_str(&format!(
                "  {} = extractvalue {} {}, 1\n",
                value, llvm_type, inner_val
            ));
        }
        ir.push_str(&format!("  br label %{}\n\n", merge_label));

        // Merge block
        ir.push_str(&format!("{}:\n", merge_label));

        Ok((value, ir))
    }

    /// Generate unwrap expression — delegates to the canonical aggregate extractvalue implementation
    pub(crate) fn generate_unwrap_expr(
        &mut self,
        inner: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let inner_type = self.infer_expr_type(inner);
        let llvm_type = self.type_to_llvm(&inner_type);

        let (inner_val, inner_ir) = self.generate_expr(inner, counter)?;
        let mut ir = inner_ir;

        ir.push_str("  ; Unwrap expression\n");

        // Determine the tag type based on the inner type:
        // - Optional/Result use i8 tag: { i8, T }
        // - User-defined enums use i32 tag: { i32, { T } }
        let (tag_type, extract_payload) = match &inner_type {
            ResolvedType::Optional(_) | ResolvedType::Result(_, _) => {
                ("i8", false)
            }
            _ => {
                ("i32", true)
            }
        };

        // Extract tag (field 0)
        let tag = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = extractvalue {} {}, 0\n",
            tag, llvm_type, inner_val
        ));

        // Check if Err/None (tag != 0)
        let is_err = self.next_temp(counter);
        let err_label = self.next_label("unwrap_err");
        let ok_label = self.next_label("unwrap_ok");

        ir.push_str(&format!("  {} = icmp ne {} {}, 0\n", is_err, tag_type, tag));
        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n\n",
            is_err, err_label, ok_label
        ));

        // Err branch: panic/abort
        ir.push_str(&format!("{}:\n", err_label));
        ir.push_str("  call i32 @puts(ptr getelementptr ([22 x i8], ptr @.unwrap_panic_msg, i64 0, i64 0))\n");
        ir.push_str("  call void @abort()\n");
        ir.push_str("  unreachable\n\n");

        // Ok branch: extract value
        ir.push_str(&format!("{}:\n", ok_label));
        let value = self.next_temp(counter);
        if extract_payload {
            ir.push_str(&format!(
                "  {} = extractvalue {} {}, 1, 0\n",
                value, llvm_type, inner_val
            ));
        } else {
            ir.push_str(&format!(
                "  {} = extractvalue {} {}, 1\n",
                value, llvm_type, inner_val
            ));
        }

        self.needs_unwrap_panic = true;

        Ok((value, ir))
    }

    // === SIMD Intrinsic Support ===

    /// Check if a function name is a SIMD intrinsic
    #[inline]
    pub(crate) fn is_simd_intrinsic(name: &str) -> bool {
        name.starts_with("vec")
            && (name.ends_with("f32")
                || name.ends_with("f64")
                || name.ends_with("i32")
                || name.ends_with("i64"))
            || name.starts_with("simd_")
    }

    /// Generate SIMD intrinsic call
    pub(crate) fn generate_simd_intrinsic(
        &mut self,
        fn_name: &str,
        args: &[Spanned<Expr>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let mut ir = String::new();
        let mut arg_vals = Vec::new();

        // Evaluate all arguments first
        for arg in args {
            let (val, arg_ir) = self.generate_expr(arg, counter)?;
            ir.push_str(&arg_ir);
            arg_vals.push(val);
        }

        // Handle vector constructors
        if fn_name.starts_with("vec") && !fn_name.starts_with("vec_") {
            return self.generate_vector_constructor(fn_name, &arg_vals, counter, ir);
        }

        // Handle SIMD binary operations
        if fn_name.starts_with("simd_add_")
            || fn_name.starts_with("simd_sub_")
            || fn_name.starts_with("simd_mul_")
            || fn_name.starts_with("simd_div_")
        {
            return self.generate_simd_binop(fn_name, &arg_vals, counter, ir);
        }

        // Handle SIMD reduce operations
        if fn_name.starts_with("simd_reduce_add_") {
            return self.generate_simd_reduce_add(fn_name, &arg_vals, counter, ir);
        }

        Err(CodegenError::Unsupported(format!(
            "Unknown SIMD intrinsic: {}",
            fn_name
        )))
    }

    /// Generate vector constructor (e.g., vec4f32(x, y, z, w))
    fn generate_vector_constructor(
        &mut self,
        fn_name: &str,
        arg_vals: &[String],
        counter: &mut usize,
        mut ir: String,
    ) -> CodegenResult<(String, String)> {
        // Parse vector type from name (e.g., "vec4f32" -> lanes=4, element="float")
        let (lanes, elem_ty) = self.parse_vector_type_name(fn_name)?;

        // Build vector using insertelement instructions
        // Start with undef and insert each element
        let vec_ty = format!("<{} x {}>", lanes, elem_ty);
        let mut current_vec = "undef".to_string();

        for (i, val) in arg_vals.iter().enumerate() {
            let next_vec = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = insertelement {} {}, {} {}, i32 {}\n",
                next_vec, vec_ty, current_vec, elem_ty, val, i
            ));
            current_vec = next_vec;
        }

        Ok((current_vec, ir))
    }

    /// Generate SIMD binary operation (add, sub, mul, div)
    fn generate_simd_binop(
        &mut self,
        fn_name: &str,
        arg_vals: &[String],
        counter: &mut usize,
        mut ir: String,
    ) -> CodegenResult<(String, String)> {
        if arg_vals.len() != 2 {
            return Err(CodegenError::TypeError(format!(
                "SIMD binary operation {} requires 2 arguments",
                fn_name
            )));
        }

        // Parse operation and type from name (e.g., "simd_add_vec4f32")
        let (op, vec_suffix) = if let Some(suffix) = fn_name.strip_prefix("simd_add_") {
            ("add", suffix)
        } else if let Some(suffix) = fn_name.strip_prefix("simd_sub_") {
            ("sub", suffix)
        } else if let Some(suffix) = fn_name.strip_prefix("simd_mul_") {
            ("mul", suffix)
        } else if let Some(suffix) = fn_name.strip_prefix("simd_div_") {
            ("div", suffix)
        } else {
            return Err(CodegenError::Unsupported(format!(
                "Unknown SIMD op: {}",
                fn_name
            )));
        };

        let (lanes, elem_ty) = self.parse_vector_type_name(vec_suffix)?;
        let vec_ty = format!("<{} x {}>", lanes, elem_ty);

        // Determine LLVM instruction based on element type
        let llvm_op = match (op, elem_ty.as_str()) {
            ("add", "float") | ("add", "double") => "fadd",
            ("sub", "float") | ("sub", "double") => "fsub",
            ("mul", "float") | ("mul", "double") => "fmul",
            ("div", "float") | ("div", "double") => "fdiv",
            ("add", _) => "add",
            ("sub", _) => "sub",
            ("mul", _) => "mul",
            ("div", _) => "sdiv",
            _ => return Err(CodegenError::Unsupported(format!("Unknown op: {}", op))),
        };

        let result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = {} {} {}, {}\n",
            result, llvm_op, vec_ty, arg_vals[0], arg_vals[1]
        ));

        Ok((result, ir))
    }

    /// Generate SIMD reduce add operation
    fn generate_simd_reduce_add(
        &mut self,
        fn_name: &str,
        arg_vals: &[String],
        counter: &mut usize,
        mut ir: String,
    ) -> CodegenResult<(String, String)> {
        if arg_vals.len() != 1 {
            return Err(CodegenError::TypeError(format!(
                "SIMD reduce operation {} requires 1 argument",
                fn_name
            )));
        }

        // Parse type from name (e.g., "simd_reduce_add_vec4f32")
        let vec_suffix = &fn_name[16..]; // Skip "simd_reduce_add_"
        let (lanes, elem_ty) = self.parse_vector_type_name(vec_suffix)?;

        // Use LLVM vector reduce intrinsics
        let intrinsic = match elem_ty.as_str() {
            "float" => format!("@llvm.vector.reduce.fadd.v{}f32", lanes),
            "double" => format!("@llvm.vector.reduce.fadd.v{}f64", lanes),
            "i32" => format!("@llvm.vector.reduce.add.v{}i32", lanes),
            "i64" => format!("@llvm.vector.reduce.add.v{}i64", lanes),
            _ => {
                return Err(CodegenError::Unsupported(format!(
                    "Unknown element type: {}",
                    elem_ty
                )))
            }
        };

        let vec_ty = format!("<{} x {}>", lanes, elem_ty);
        let result = self.next_temp(counter);

        // For float/double, we need an initial value for ordered reduction
        if elem_ty == "float" || elem_ty == "double" {
            let zero = "0.0";
            ir.push_str(&format!(
                "  {} = call {} {}({} {}, {} {})\n",
                result, elem_ty, intrinsic, elem_ty, zero, vec_ty, arg_vals[0]
            ));
        } else {
            ir.push_str(&format!(
                "  {} = call {} {}({} {})\n",
                result, elem_ty, intrinsic, vec_ty, arg_vals[0]
            ));
        }

        Ok((result, ir))
    }

    /// Parse vector type name to get lanes and element type
    fn parse_vector_type_name(&self, name: &str) -> CodegenResult<(u32, String)> {
        // e.g., "vec4f32" -> (4, "float"), "vec2i64" -> (2, "i64")
        let (lanes, elem) = if let Some(rest) = name.strip_prefix("vec") {
            // Remove "vec" prefix
            if let Some(lanes_str) = rest.strip_suffix("f32") {
                (lanes_str.parse::<u32>().unwrap_or(4), "float".to_string())
            } else if let Some(lanes_str) = rest.strip_suffix("f64") {
                (lanes_str.parse::<u32>().unwrap_or(2), "double".to_string())
            } else if let Some(lanes_str) = rest.strip_suffix("i32") {
                (lanes_str.parse::<u32>().unwrap_or(4), "i32".to_string())
            } else if let Some(lanes_str) = rest.strip_suffix("i64") {
                (lanes_str.parse::<u32>().unwrap_or(2), "i64".to_string())
            } else {
                return Err(CodegenError::Unsupported(format!(
                    "Unknown vector type: {}",
                    name
                )));
            }
        } else {
            return Err(CodegenError::Unsupported(format!(
                "Invalid vector type name: {}",
                name
            )));
        };

        Ok((lanes, elem))
    }
}
