//! SIMD vector built-in functions

use super::*;

impl TypeChecker {
    pub(super) fn register_simd_builtins(&mut self) {
        // Helper to create vector types
        let vec2f32 = ResolvedType::Vector {
            element: Box::new(ResolvedType::F32),
            lanes: 2,
        };
        let vec4f32 = ResolvedType::Vector {
            element: Box::new(ResolvedType::F32),
            lanes: 4,
        };
        let vec8f32 = ResolvedType::Vector {
            element: Box::new(ResolvedType::F32),
            lanes: 8,
        };
        let vec2f64 = ResolvedType::Vector {
            element: Box::new(ResolvedType::F64),
            lanes: 2,
        };
        let vec4f64 = ResolvedType::Vector {
            element: Box::new(ResolvedType::F64),
            lanes: 4,
        };
        let vec4i32 = ResolvedType::Vector {
            element: Box::new(ResolvedType::I32),
            lanes: 4,
        };
        let vec8i32 = ResolvedType::Vector {
            element: Box::new(ResolvedType::I32),
            lanes: 8,
        };
        let vec2i64 = ResolvedType::Vector {
            element: Box::new(ResolvedType::I64),
            lanes: 2,
        };
        let vec4i64 = ResolvedType::Vector {
            element: Box::new(ResolvedType::I64),
            lanes: 4,
        };

        // === Vector Constructors ===
        // vec2f32(x, y) -> Vec2f32
        self.functions.insert(
            "vec2f32".to_string(),
            FunctionSig {
                name: "vec2f32".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::F32, false),
                    ("y".to_string(), ResolvedType::F32, false),
                ],
                ret: vec2f32.clone(),
                ..Default::default()
            },
        );

        // vec4f32(x, y, z, w) -> Vec4f32
        self.functions.insert(
            "vec4f32".to_string(),
            FunctionSig {
                name: "vec4f32".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::F32, false),
                    ("y".to_string(), ResolvedType::F32, false),
                    ("z".to_string(), ResolvedType::F32, false),
                    ("w".to_string(), ResolvedType::F32, false),
                ],
                ret: vec4f32.clone(),
                ..Default::default()
            },
        );

        // vec8f32(a, b, c, d, e, f, g, h) -> Vec8f32
        self.functions.insert(
            "vec8f32".to_string(),
            FunctionSig {
                name: "vec8f32".to_string(),
                params: vec![
                    ("a".to_string(), ResolvedType::F32, false),
                    ("b".to_string(), ResolvedType::F32, false),
                    ("c".to_string(), ResolvedType::F32, false),
                    ("d".to_string(), ResolvedType::F32, false),
                    ("e".to_string(), ResolvedType::F32, false),
                    ("f".to_string(), ResolvedType::F32, false),
                    ("g".to_string(), ResolvedType::F32, false),
                    ("h".to_string(), ResolvedType::F32, false),
                ],
                ret: vec8f32.clone(),
                ..Default::default()
            },
        );

        // vec2f64(x, y) -> Vec2f64
        self.functions.insert(
            "vec2f64".to_string(),
            FunctionSig {
                name: "vec2f64".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::F64, false),
                    ("y".to_string(), ResolvedType::F64, false),
                ],
                ret: vec2f64.clone(),
                ..Default::default()
            },
        );

        // vec4f64(x, y, z, w) -> Vec4f64
        self.functions.insert(
            "vec4f64".to_string(),
            FunctionSig {
                name: "vec4f64".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::F64, false),
                    ("y".to_string(), ResolvedType::F64, false),
                    ("z".to_string(), ResolvedType::F64, false),
                    ("w".to_string(), ResolvedType::F64, false),
                ],
                ret: vec4f64.clone(),
                ..Default::default()
            },
        );

        // vec4i32(x, y, z, w) -> Vec4i32
        self.functions.insert(
            "vec4i32".to_string(),
            FunctionSig {
                name: "vec4i32".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::I32, false),
                    ("y".to_string(), ResolvedType::I32, false),
                    ("z".to_string(), ResolvedType::I32, false),
                    ("w".to_string(), ResolvedType::I32, false),
                ],
                ret: vec4i32.clone(),
                ..Default::default()
            },
        );

        // vec8i32(a, b, c, d, e, f, g, h) -> Vec8i32
        self.functions.insert(
            "vec8i32".to_string(),
            FunctionSig {
                name: "vec8i32".to_string(),
                params: vec![
                    ("a".to_string(), ResolvedType::I32, false),
                    ("b".to_string(), ResolvedType::I32, false),
                    ("c".to_string(), ResolvedType::I32, false),
                    ("d".to_string(), ResolvedType::I32, false),
                    ("e".to_string(), ResolvedType::I32, false),
                    ("f".to_string(), ResolvedType::I32, false),
                    ("g".to_string(), ResolvedType::I32, false),
                    ("h".to_string(), ResolvedType::I32, false),
                ],
                ret: vec8i32.clone(),
                ..Default::default()
            },
        );

        // vec2i64(x, y) -> Vec2i64
        self.functions.insert(
            "vec2i64".to_string(),
            FunctionSig {
                name: "vec2i64".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::I64, false),
                    ("y".to_string(), ResolvedType::I64, false),
                ],
                ret: vec2i64.clone(),
                ..Default::default()
            },
        );

        // vec4i64(x, y, z, w) -> Vec4i64
        self.functions.insert(
            "vec4i64".to_string(),
            FunctionSig {
                name: "vec4i64".to_string(),
                params: vec![
                    ("x".to_string(), ResolvedType::I64, false),
                    ("y".to_string(), ResolvedType::I64, false),
                    ("z".to_string(), ResolvedType::I64, false),
                    ("w".to_string(), ResolvedType::I64, false),
                ],
                ret: vec4i64.clone(),
                ..Default::default()
            },
        );

        // === SIMD Arithmetic Operations ===
        // Helper macro to register binary SIMD operations
        macro_rules! register_simd_binop {
            ($name:expr, $vec_ty:expr) => {
                self.functions.insert(
                    $name.to_string(),
                    FunctionSig {
                        name: $name.to_string(),
                        params: vec![
                            ("a".to_string(), $vec_ty.clone(), false),
                            ("b".to_string(), $vec_ty.clone(), false),
                        ],
                        ret: $vec_ty.clone(),
                        ..Default::default()
                    },
                );
            };
        }

        // Vec4f32 operations
        register_simd_binop!("simd_add_vec4f32", vec4f32);
        register_simd_binop!("simd_sub_vec4f32", vec4f32);
        register_simd_binop!("simd_mul_vec4f32", vec4f32);
        register_simd_binop!("simd_div_vec4f32", vec4f32);

        // Vec8f32 operations
        register_simd_binop!("simd_add_vec8f32", vec8f32);
        register_simd_binop!("simd_sub_vec8f32", vec8f32);
        register_simd_binop!("simd_mul_vec8f32", vec8f32);
        register_simd_binop!("simd_div_vec8f32", vec8f32);

        // Vec2f64 operations
        register_simd_binop!("simd_add_vec2f64", vec2f64);
        register_simd_binop!("simd_sub_vec2f64", vec2f64);
        register_simd_binop!("simd_mul_vec2f64", vec2f64);
        register_simd_binop!("simd_div_vec2f64", vec2f64);

        // Vec4f64 operations
        register_simd_binop!("simd_add_vec4f64", vec4f64);
        register_simd_binop!("simd_sub_vec4f64", vec4f64);
        register_simd_binop!("simd_mul_vec4f64", vec4f64);
        register_simd_binop!("simd_div_vec4f64", vec4f64);

        // Vec4i32 operations
        register_simd_binop!("simd_add_vec4i32", vec4i32);
        register_simd_binop!("simd_sub_vec4i32", vec4i32);
        register_simd_binop!("simd_mul_vec4i32", vec4i32);

        // Vec8i32 operations
        register_simd_binop!("simd_add_vec8i32", vec8i32);
        register_simd_binop!("simd_sub_vec8i32", vec8i32);
        register_simd_binop!("simd_mul_vec8i32", vec8i32);

        // Vec2i64 operations
        register_simd_binop!("simd_add_vec2i64", vec2i64);
        register_simd_binop!("simd_sub_vec2i64", vec2i64);
        register_simd_binop!("simd_mul_vec2i64", vec2i64);

        // Vec4i64 operations
        register_simd_binop!("simd_add_vec4i64", vec4i64);
        register_simd_binop!("simd_sub_vec4i64", vec4i64);
        register_simd_binop!("simd_mul_vec4i64", vec4i64);

        // === Horizontal Reduction Operations ===
        // simd_reduce_add_vec4f32(v) -> f32
        self.functions.insert(
            "simd_reduce_add_vec4f32".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec4f32".to_string(),
                params: vec![("v".to_string(), vec4f32.clone(), false)],
                ret: ResolvedType::F32,
                ..Default::default()
            },
        );

        // simd_reduce_add_vec8f32(v) -> f32
        self.functions.insert(
            "simd_reduce_add_vec8f32".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec8f32".to_string(),
                params: vec![("v".to_string(), vec8f32, false)],
                ret: ResolvedType::F32,
                ..Default::default()
            },
        );

        // simd_reduce_add_vec2f64(v) -> f64
        self.functions.insert(
            "simd_reduce_add_vec2f64".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec2f64".to_string(),
                params: vec![("v".to_string(), vec2f64, false)],
                ret: ResolvedType::F64,
                ..Default::default()
            },
        );

        // simd_reduce_add_vec4f64(v) -> f64
        self.functions.insert(
            "simd_reduce_add_vec4f64".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec4f64".to_string(),
                params: vec![("v".to_string(), vec4f64, false)],
                ret: ResolvedType::F64,
                ..Default::default()
            },
        );

        // simd_reduce_add_vec4i32(v) -> i32
        self.functions.insert(
            "simd_reduce_add_vec4i32".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec4i32".to_string(),
                params: vec![("v".to_string(), vec4i32, false)],
                ret: ResolvedType::I32,
                ..Default::default()
            },
        );

        // simd_reduce_add_vec8i32(v) -> i32
        self.functions.insert(
            "simd_reduce_add_vec8i32".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec8i32".to_string(),
                params: vec![("v".to_string(), vec8i32, false)],
                ret: ResolvedType::I32,
                ..Default::default()
            },
        );

        // simd_reduce_add_vec2i64(v) -> i64
        self.functions.insert(
            "simd_reduce_add_vec2i64".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec2i64".to_string(),
                params: vec![("v".to_string(), vec2i64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );

        // simd_reduce_add_vec4i64(v) -> i64
        self.functions.insert(
            "simd_reduce_add_vec4i64".to_string(),
            FunctionSig {
                name: "simd_reduce_add_vec4i64".to_string(),
                params: vec![("v".to_string(), vec4i64, false)],
                ret: ResolvedType::I64,
                ..Default::default()
            },
        );
    }
}
