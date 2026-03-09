//! SIMD built-in function definitions for inkwell code generator.

use super::*;

/// Define SIMD built-in functions with actual implementations.
/// Uses heap-allocated arrays behind i8* pointers to represent SIMD vectors,
/// matching the text backend's approach.
pub(super) fn define_simd_builtins<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let builder = context.create_builder();
    let i8_ptr = context.i8_type().ptr_type(AddressSpace::default());
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let f32_type = context.f32_type();
    let f64_type = context.f64_type();

    let malloc_fn = module.get_function("malloc").unwrap_or_else(|| {
        module.add_function("malloc", i8_ptr.fn_type(&[i64_type.into()], false), None)
    });

    // Helper: define a vec constructor that allocates N elements and stores args
    // vec4i32(a, b, c, d) -> i8* (heap-allocated [i32 x 4])
    {
        let fn_type = i8_ptr.fn_type(
            &[
                i32_type.into(),
                i32_type.into(),
                i32_type.into(),
                i32_type.into(),
            ],
            false,
        );
        let func = module.add_function("vec4i32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        // malloc(16) for 4 x i32
        let ptr = builder
            .build_call(malloc_fn, &[i64_type.const_int(16, false).into()], "ptr")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            .try_as_basic_value()
            .left()
            .ok_or("ICE: inkwell builtins: call returned void")?
            .into_pointer_value();
        let i32_ptr = builder
            .build_pointer_cast(ptr, i32_type.ptr_type(AddressSpace::default()), "i32ptr")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        for i in 0..4u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        i32_type,
                        i32_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            builder
                .build_store(
                    gep,
                    func.get_nth_param(i)
                        .ok_or("ICE: builtin function missing parameter")?
                        .into_int_value(),
                )
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        }
        builder
            .build_return(Some(&ptr))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }

    // vec4f32(a, b, c, d) -> i8* (heap-allocated [float x 4])
    {
        let fn_type = i8_ptr.fn_type(
            &[
                f64_type.into(),
                f64_type.into(),
                f64_type.into(),
                f64_type.into(),
            ],
            false,
        );
        let func = module.add_function("vec4f32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let ptr = builder
            .build_call(malloc_fn, &[i64_type.const_int(16, false).into()], "ptr")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            .try_as_basic_value()
            .left()
            .ok_or("ICE: inkwell builtins: call returned void")?
            .into_pointer_value();
        let f32_ptr = builder
            .build_pointer_cast(ptr, f32_type.ptr_type(AddressSpace::default()), "f32ptr")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        for i in 0..4u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        f32_type,
                        f32_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            // Truncate f64 param to f32
            let param = func
                .get_nth_param(i)
                .ok_or("ICE: builtin function missing parameter")?
                .into_float_value();
            let f32_val = builder
                .build_float_trunc(param, f32_type, "trunc")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
            builder
                .build_store(gep, f32_val)
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        }
        builder
            .build_return(Some(&ptr))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }

    // vec2i64(a, b) -> i8*
    {
        let fn_type = i8_ptr.fn_type(&[i64_type.into(), i64_type.into()], false);
        let func = module.add_function("vec2i64", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let ptr = builder
            .build_call(malloc_fn, &[i64_type.const_int(16, false).into()], "ptr")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            .try_as_basic_value()
            .left()
            .ok_or("ICE: inkwell builtins: call returned void")?
            .into_pointer_value();
        let i64_ptr = builder
            .build_pointer_cast(ptr, i64_type.ptr_type(AddressSpace::default()), "i64ptr")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        for i in 0..2u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        i64_type,
                        i64_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            builder
                .build_store(
                    gep,
                    func.get_nth_param(i)
                        .ok_or("ICE: builtin function missing parameter")?
                        .into_int_value(),
                )
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        }
        builder
            .build_return(Some(&ptr))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }

    // Helper macro for SIMD binary operations on i32x4
    // pattern: load 4 elements from each, do op, store into new alloc, return
    for (name, op) in &[
        ("simd_add_vec4i32", "add"),
        ("simd_sub_vec4i32", "sub"),
        ("simd_mul_vec4i32", "mul"),
        ("simd_div_vec4i32", "sdiv"),
    ] {
        let fn_type = i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false);
        let func = module.add_function(name, fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);

        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0)
                    .ok_or("ICE: builtin function missing parameter")?
                    .into_pointer_value(),
                i32_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        let b_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(1)
                    .ok_or("ICE: builtin function missing parameter")?
                    .into_pointer_value(),
                i32_type.ptr_type(AddressSpace::default()),
                "b",
            )
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        let out = builder
            .build_call(malloc_fn, &[i64_type.const_int(16, false).into()], "out")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            .try_as_basic_value()
            .left()
            .ok_or("ICE: inkwell builtins: call returned void")?
            .into_pointer_value();
        let out_i32 = builder
            .build_pointer_cast(out, i32_type.ptr_type(AddressSpace::default()), "out_i32")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;

        for i in 0..4u32 {
            let idx = i32_type.const_int(i as u64, false);
            let a_gep = unsafe {
                builder
                    .build_gep(i32_type, a_ptr, &[idx], "a_gep")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            let b_gep = unsafe {
                builder
                    .build_gep(i32_type, b_ptr, &[idx], "b_gep")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            let a_val = builder
                .build_load(i32_type, a_gep, "a_val")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
                .into_int_value();
            let b_val = builder
                .build_load(i32_type, b_gep, "b_val")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
                .into_int_value();
            let result = match *op {
                "add" => builder
                    .build_int_add(a_val, b_val, "r")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?,
                "sub" => builder
                    .build_int_sub(a_val, b_val, "r")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?,
                "mul" => builder
                    .build_int_mul(a_val, b_val, "r")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?,
                "sdiv" => builder
                    .build_int_signed_div(a_val, b_val, "r")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?,
                _ => {
                    return Err(format!(
                        "ICE: unexpected SIMD operation in vec4i32: {op}"
                    ));
                }
            };
            let o_gep = unsafe {
                builder
                    .build_gep(i32_type, out_i32, &[idx], "o_gep")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            builder
                .build_store(o_gep, result)
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        }
        builder
            .build_return(Some(&out))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }

    // simd_reduce_add_vec4i32: sum all 4 i32 elements -> i64
    {
        let fn_type = i64_type.fn_type(&[i8_ptr.into()], false);
        let func = module.add_function("simd_reduce_add_vec4i32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0)
                    .ok_or("ICE: builtin function missing parameter")?
                    .into_pointer_value(),
                i32_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        let mut sum = i64_type.const_int(0, false);
        for i in 0..4u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        i32_type,
                        a_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            let val = builder
                .build_load(i32_type, gep, "val")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
                .into_int_value();
            let ext = builder
                .build_int_s_extend(val, i64_type, "ext")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
            sum = builder
                .build_int_add(sum, ext, "sum")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        }
        builder
            .build_return(Some(&sum))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }

    // simd_reduce_mul_vec4i32
    {
        let fn_type = i64_type.fn_type(&[i8_ptr.into()], false);
        let func = module.add_function("simd_reduce_mul_vec4i32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0)
                    .ok_or("ICE: builtin function missing parameter")?
                    .into_pointer_value(),
                i32_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        let mut prod = i64_type.const_int(1, false);
        for i in 0..4u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        i32_type,
                        a_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            let val = builder
                .build_load(i32_type, gep, "val")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
                .into_int_value();
            let ext = builder
                .build_int_s_extend(val, i64_type, "ext")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
            prod = builder
                .build_int_mul(prod, ext, "prod")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        }
        builder
            .build_return(Some(&prod))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }

    // SIMD f32x4 binary ops
    for (name, op) in &[
        ("simd_add_vec4f32", "fadd"),
        ("simd_sub_vec4f32", "fsub"),
        ("simd_mul_vec4f32", "fmul"),
        ("simd_div_vec4f32", "fdiv"),
    ] {
        let fn_type = i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false);
        let func = module.add_function(name, fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);

        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0)
                    .ok_or("ICE: builtin function missing parameter")?
                    .into_pointer_value(),
                f32_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        let b_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(1)
                    .ok_or("ICE: builtin function missing parameter")?
                    .into_pointer_value(),
                f32_type.ptr_type(AddressSpace::default()),
                "b",
            )
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        let out = builder
            .build_call(malloc_fn, &[i64_type.const_int(16, false).into()], "out")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            .try_as_basic_value()
            .left()
            .ok_or("ICE: inkwell builtins: call returned void")?
            .into_pointer_value();
        let out_f32 = builder
            .build_pointer_cast(out, f32_type.ptr_type(AddressSpace::default()), "out_f32")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;

        for i in 0..4u32 {
            let idx = i32_type.const_int(i as u64, false);
            let a_gep = unsafe {
                builder
                    .build_gep(f32_type, a_ptr, &[idx], "a_gep")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            let b_gep = unsafe {
                builder
                    .build_gep(f32_type, b_ptr, &[idx], "b_gep")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            let a_val = builder
                .build_load(f32_type, a_gep, "a_val")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
                .into_float_value();
            let b_val = builder
                .build_load(f32_type, b_gep, "b_val")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
                .into_float_value();
            let result = match *op {
                "fadd" => builder
                    .build_float_add(a_val, b_val, "r")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?,
                "fsub" => builder
                    .build_float_sub(a_val, b_val, "r")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?,
                "fmul" => builder
                    .build_float_mul(a_val, b_val, "r")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?,
                "fdiv" => builder
                    .build_float_div(a_val, b_val, "r")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?,
                _ => {
                    return Err(format!(
                        "ICE: unexpected SIMD operation in vec4f32: {op}"
                    ));
                }
            };
            let o_gep = unsafe {
                builder
                    .build_gep(f32_type, out_f32, &[idx], "o_gep")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            builder
                .build_store(o_gep, result)
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        }
        builder
            .build_return(Some(&out))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }

    // simd_reduce_add_vec4f32: sum all 4 f32 elements -> f64
    {
        let fn_type = f64_type.fn_type(&[i8_ptr.into()], false);
        let func = module.add_function("simd_reduce_add_vec4f32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0)
                    .ok_or("ICE: builtin function missing parameter")?
                    .into_pointer_value(),
                f32_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        let mut sum = f64_type.const_float(0.0);
        for i in 0..4u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        f32_type,
                        a_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            let val = builder
                .build_load(f32_type, gep, "val")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
                .into_float_value();
            let ext = builder
                .build_float_ext(val, f64_type, "ext")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
            sum = builder
                .build_float_add(sum, ext, "sum")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        }
        builder
            .build_return(Some(&sum))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }

    // simd_reduce_mul_vec4f32
    {
        let fn_type = f64_type.fn_type(&[i8_ptr.into()], false);
        let func = module.add_function("simd_reduce_mul_vec4f32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0)
                    .ok_or("ICE: builtin function missing parameter")?
                    .into_pointer_value(),
                f32_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        let mut prod = f64_type.const_float(1.0);
        for i in 0..4u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        f32_type,
                        a_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            let val = builder
                .build_load(f32_type, gep, "val")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
                .into_float_value();
            let ext = builder
                .build_float_ext(val, f64_type, "ext")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
            prod = builder
                .build_float_mul(prod, ext, "prod")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        }
        builder
            .build_return(Some(&prod))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }

    // simd_dot_vec4f32(a, b) -> f64: sum of element-wise products
    {
        let fn_type = f64_type.fn_type(&[i8_ptr.into(), i8_ptr.into()], false);
        let func = module.add_function("simd_dot_vec4f32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0)
                    .ok_or("ICE: builtin function missing parameter")?
                    .into_pointer_value(),
                f32_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        let b_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(1)
                    .ok_or("ICE: builtin function missing parameter")?
                    .into_pointer_value(),
                f32_type.ptr_type(AddressSpace::default()),
                "b",
            )
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        let mut sum = f64_type.const_float(0.0);
        for i in 0..4u32 {
            let idx = i32_type.const_int(i as u64, false);
            let a_gep = unsafe {
                builder
                    .build_gep(f32_type, a_ptr, &[idx], "a_gep")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            let b_gep = unsafe {
                builder
                    .build_gep(f32_type, b_ptr, &[idx], "b_gep")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            let a_val = builder
                .build_load(f32_type, a_gep, "a_val")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
                .into_float_value();
            let b_val = builder
                .build_load(f32_type, b_gep, "b_val")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
                .into_float_value();
            let prod = builder
                .build_float_mul(a_val, b_val, "prod")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
            let ext = builder
                .build_float_ext(prod, f64_type, "ext")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
            sum = builder
                .build_float_add(sum, ext, "sum")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        }
        builder
            .build_return(Some(&sum))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }

    // simd_create_vec4f32 (same as vec4f32)
    {
        let fn_type = i8_ptr.fn_type(
            &[
                f64_type.into(),
                f64_type.into(),
                f64_type.into(),
                f64_type.into(),
            ],
            false,
        );
        let func = module.add_function("simd_create_vec4f32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let ptr = builder
            .build_call(malloc_fn, &[i64_type.const_int(16, false).into()], "ptr")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            .try_as_basic_value()
            .left()
            .ok_or("ICE: inkwell builtins: call returned void")?
            .into_pointer_value();
        let f32_ptr = builder
            .build_pointer_cast(ptr, f32_type.ptr_type(AddressSpace::default()), "f32ptr")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        for i in 0..4u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        f32_type,
                        f32_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            let param = func
                .get_nth_param(i)
                .ok_or("ICE: builtin function missing parameter")?
                .into_float_value();
            let f32_val = builder
                .build_float_trunc(param, f32_type, "trunc")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
            builder
                .build_store(gep, f32_val)
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        }
        builder
            .build_return(Some(&ptr))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }

    // simd_get_vec4f32(ptr, idx) -> f64
    {
        let fn_type = f64_type.fn_type(&[i8_ptr.into(), i32_type.into()], false);
        let func = module.add_function("simd_get_vec4f32", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0)
                    .ok_or("ICE: builtin function missing parameter")?
                    .into_pointer_value(),
                f32_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        let idx = func
            .get_nth_param(1)
            .ok_or("ICE: builtin function missing parameter")?
            .into_int_value();
        let gep = unsafe {
            builder
                .build_gep(f32_type, a_ptr, &[idx], "gep")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
        };
        let val = builder
            .build_load(f32_type, gep, "val")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            .into_float_value();
        let ext = builder
            .build_float_ext(val, f64_type, "ext")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        builder
            .build_return(Some(&ext))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }

    // Vec2i64 binary ops
    {
        let fn_type = i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false);
        let func = module.add_function("simd_add_vec2i64", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0)
                    .ok_or("ICE: builtin function missing parameter")?
                    .into_pointer_value(),
                i64_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        let b_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(1)
                    .ok_or("ICE: builtin function missing parameter")?
                    .into_pointer_value(),
                i64_type.ptr_type(AddressSpace::default()),
                "b",
            )
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        let out = builder
            .build_call(malloc_fn, &[i64_type.const_int(16, false).into()], "out")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            .try_as_basic_value()
            .left()
            .ok_or("ICE: inkwell builtins: call returned void")?
            .into_pointer_value();
        let out_i64 = builder
            .build_pointer_cast(out, i64_type.ptr_type(AddressSpace::default()), "out_i64")
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        for i in 0..2u32 {
            let idx = i32_type.const_int(i as u64, false);
            let a_gep = unsafe {
                builder
                    .build_gep(i64_type, a_ptr, &[idx], "a_gep")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            let b_gep = unsafe {
                builder
                    .build_gep(i64_type, b_ptr, &[idx], "b_gep")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            let a_val = builder
                .build_load(i64_type, a_gep, "a_val")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
                .into_int_value();
            let b_val = builder
                .build_load(i64_type, b_gep, "b_val")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
                .into_int_value();
            let result = builder
                .build_int_add(a_val, b_val, "r")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
            let o_gep = unsafe {
                builder
                    .build_gep(i64_type, out_i64, &[idx], "o_gep")
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            builder
                .build_store(o_gep, result)
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        }
        builder
            .build_return(Some(&out))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }

    // simd_reduce_add_vec2i64
    {
        let fn_type = i64_type.fn_type(&[i8_ptr.into()], false);
        let func = module.add_function("simd_reduce_add_vec2i64", fn_type, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);
        let a_ptr = builder
            .build_pointer_cast(
                func.get_nth_param(0)
                    .ok_or("ICE: builtin function missing parameter")?
                    .into_pointer_value(),
                i64_type.ptr_type(AddressSpace::default()),
                "a",
            )
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        let mut sum = i64_type.const_int(0, false);
        for i in 0..2u32 {
            let gep = unsafe {
                builder
                    .build_gep(
                        i64_type,
                        a_ptr,
                        &[i32_type.const_int(i as u64, false)],
                        "gep",
                    )
                    .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
            };
            let val = builder
                .build_load(i64_type, gep, "val")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?
                .into_int_value();
            sum = builder
                .build_int_add(sum, val, "sum")
                .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
        }
        builder
            .build_return(Some(&sum))
            .map_err(|e| format!("ICE: inkwell builtins: {e}"))?;
    }

    Ok(())
}
