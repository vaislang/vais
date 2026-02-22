//! Async function code generation

use crate::types::LocalVar;
use crate::{AsyncFunctionInfo, CodeGenerator, CodegenResult};
use vais_ast::{Function, FunctionBody};

impl CodeGenerator {
    /// Generate an async function as a state machine coroutine
    ///
    /// Async functions are transformed into:
    /// 1. A state struct holding local variables and current state
    /// 2. A poll function that implements the state machine
    /// 3. A create function that returns a pointer to the state struct
    pub(crate) fn generate_async_function(&mut self, f: &Function) -> CodegenResult<String> {
        let func_name = &f.name.node;
        let state_struct_name = format!("{}__AsyncState", func_name);

        // Collect parameters for state struct
        let params: Vec<_> = f
            .params
            .iter()
            .map(|p| {
                let ty = self.ast_type_to_resolved(&p.ty.node);
                (p.name.node.to_string(), ty)
            })
            .collect();

        let ret_type = self.resolve_fn_return_type(f, func_name);

        let ret_llvm = self.type_to_llvm(&ret_type);

        // Reset async state tracking
        self.lambdas.async_state_counter = 0;
        self.lambdas.async_await_points.clear();
        self.lambdas.current_async_function = Some(AsyncFunctionInfo {
            name: func_name.to_string(),
            state_struct: state_struct_name.to_string(),
            captured_vars: params.clone(),
            ret_type: ret_type.clone(),
        });

        let mut ir = String::new();

        // 1. Generate state struct type
        // Structure: { i64 state, i64 result, param1, param2, ... }
        ir.push_str(&format!("; Async state struct for {}\n", func_name));
        ir.push_str(&format!(
            "%{} = type {{ i64, {}",
            state_struct_name, ret_llvm
        ));
        for (_, ty) in &params {
            ir.push_str(&format!(", {}", self.type_to_llvm(ty)));
        }
        ir.push_str(" }\n\n");

        // 2. Generate create function: allocates and initializes state
        ir.push_str(&format!("; Create function for async {}\n", func_name));
        let create_params: Vec<_> = params
            .iter()
            .map(|(name, ty)| format!("{} %{}", self.type_to_llvm(ty), name))
            .collect();
        ir.push_str(&format!(
            "define i64 @{}({}) {{\n",
            func_name,
            create_params.join(", ")
        ));
        ir.push_str("entry:\n");

        // Calculate struct size based on actual LLVM type sizes
        let llvm_size = |llvm_ty: &str| -> usize {
            match llvm_ty {
                "i1" | "i8" => 1,
                "i16" => 2,
                "i32" | "float" => 4,
                "i64" | "double" => 8,
                "i128" => 16,
                _ if llvm_ty.ends_with('*') => 8,
                _ => 8, // Default to pointer-sized for structs/unknown
            }
        };
        let struct_size = 8 /* state: i64 */
            + llvm_size(&ret_llvm)
            + params.iter().map(|(_, ty)| llvm_size(&self.type_to_llvm(ty))).sum::<usize>();
        ir.push_str(&format!(
            "  %state_ptr = call i64 @malloc(i64 {})\n",
            struct_size
        ));
        ir.push_str(&format!(
            "  %state = inttoptr i64 %state_ptr to %{}*\n",
            state_struct_name
        ));

        // Initialize state to 0 (start state)
        ir.push_str(&format!(
            "  %state_field = getelementptr %{}, %{}* %state, i32 0, i32 0\n",
            state_struct_name, state_struct_name
        ));
        ir.push_str("  store i64 0, i64* %state_field\n");

        // Store parameters in state struct
        for (i, (name, ty)) in params.iter().enumerate() {
            let field_idx = i + 2; // Skip state and result fields
            let param_llvm_ty = self.type_to_llvm(ty);
            ir.push_str(&format!(
                "  %param_{}_ptr = getelementptr %{}, %{}* %state, i32 0, i32 {}\n",
                name, state_struct_name, state_struct_name, field_idx
            ));
            ir.push_str(&format!(
                "  store {} %{}, {}* %param_{}_ptr\n",
                param_llvm_ty, name, param_llvm_ty, name
            ));
        }

        ir.push_str("  ret i64 %state_ptr\n");
        ir.push_str("}\n\n");

        // 3. Generate poll function: implements state machine
        self.fn_ctx.current_function = Some(format!("{}__poll", func_name));
        self.fn_ctx.locals.clear();
        self.fn_ctx.label_counter = 0;
        self.fn_ctx.loop_stack.clear();

        ir.push_str(&format!("; Poll function for async {}\n", func_name));
        ir.push_str(&format!(
            "define {{ i64, {} }} @{}__poll(i64 %state_ptr) {{\n",
            ret_llvm, func_name
        ));
        ir.push_str("entry:\n");
        ir.push_str(&format!(
            "  %state = inttoptr i64 %state_ptr to %{}*\n",
            state_struct_name
        ));

        // Load current state
        ir.push_str(&format!(
            "  %state_field = getelementptr %{}, %{}* %state, i32 0, i32 0\n",
            state_struct_name, state_struct_name
        ));
        ir.push_str("  %current_state = load i64, i64* %state_field\n");

        // Load parameters from state into locals
        for (i, (name, ty)) in params.iter().enumerate() {
            let field_idx = i + 2;
            let param_llvm_ty = self.type_to_llvm(ty);
            ir.push_str(&format!(
                "  %param_{}_ptr = getelementptr %{}, %{}* %state, i32 0, i32 {}\n",
                name, state_struct_name, state_struct_name, field_idx
            ));
            ir.push_str(&format!(
                "  %{} = load {}, {}* %param_{}_ptr\n",
                name, param_llvm_ty, param_llvm_ty, name
            ));

            self.fn_ctx
                .locals
                .insert(name.clone(), LocalVar::param(ty.clone(), name.clone()));
        }

        // State machine switch
        ir.push_str("  switch i64 %current_state, label %state_invalid [\n");
        ir.push_str("    i64 0, label %state_0\n");
        ir.push_str("  ]\n\n");

        // Generate state_0 (initial state) - execute function body
        ir.push_str("state_0:\n");

        // Set async poll context so Return statements inside the body
        // can generate proper poll result wrapping ({1, result})
        self.fn_ctx.async_poll_context = Some(crate::AsyncPollContext {
            ret_llvm: ret_llvm.clone(),
        });

        let mut counter = 0;
        let body_result = match &f.body {
            FunctionBody::Expr(expr) => self.generate_expr(expr, &mut counter)?,
            FunctionBody::Block(stmts) => self.generate_block(stmts, &mut counter)?,
        };

        ir.push_str(&body_result.1);

        // Store result and return Ready
        ir.push_str(&format!(
            "  %result_ptr = getelementptr %{}, %{}* %state, i32 0, i32 1\n",
            state_struct_name, state_struct_name
        ));
        // The codegen promotes bool comparisons to i64 (zext i1 to i64), but the
        // result field in the state struct uses the actual return type. Truncate
        // i64 back to i1 when the return type is bool.
        let store_val = if ret_llvm == "i1" {
            let trunc = format!("%body_trunc.{}", counter);
            ir.push_str(&format!(
                "  {} = trunc i64 {} to i1\n",
                trunc, body_result.0
            ));
            trunc
        } else {
            body_result.0.clone()
        };
        ir.push_str(&format!(
            "  store {} {}, {}* %result_ptr\n",
            ret_llvm, store_val, ret_llvm
        ));

        // Set state to -1 (completed)
        ir.push_str("  store i64 -1, i64* %state_field\n");

        // Return {1, result} for Ready
        ir.push_str(&format!(
            "  %ret_val = load {}, {}* %result_ptr\n",
            ret_llvm, ret_llvm
        ));
        ir.push_str(&format!(
            "  %ret_0 = insertvalue {{ i64, {} }} undef, i64 1, 0\n",
            ret_llvm
        ));
        ir.push_str(&format!(
            "  %ret_1 = insertvalue {{ i64, {} }} %ret_0, {} %ret_val, 1\n",
            ret_llvm, ret_llvm
        ));
        ir.push_str(&format!("  ret {{ i64, {} }} %ret_1\n\n", ret_llvm));

        // Invalid state handler
        ir.push_str("state_invalid:\n");
        ir.push_str(&format!(
            "  %invalid_ret = insertvalue {{ i64, {} }} undef, i64 0, 0\n",
            ret_llvm
        ));
        ir.push_str(&format!("  ret {{ i64, {} }} %invalid_ret\n", ret_llvm));

        ir.push_str("}\n");

        self.fn_ctx.current_function = None;
        self.lambdas.current_async_function = None;

        Ok(ir)
    }
}
