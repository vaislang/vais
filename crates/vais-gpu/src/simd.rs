//! CPU SIMD code generation
//!
//! Generates SIMD-optimized C code for various CPU instruction sets:
//! - AVX-512 (Intel/AMD)
//! - AVX2 (Intel/AMD)
//! - SSE4 (Intel/AMD)
//! - NEON (ARM)
//!
//! # Usage
//!
//! ```ignore
//! use vais_gpu::simd::{SimdTarget, generate_simd_code};
//!
//! let code = generate_simd_code(&module, SimdTarget::Avx512)?;
//! ```

use crate::common::binary_op_str;
use crate::GpuResult;
use vais_ast::{Expr, Function, FunctionBody, Item, Module, Stmt, Type};

/// CPU SIMD target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimdTarget {
    /// Intel AVX-512 (512-bit vectors)
    Avx512,
    /// Intel/AMD AVX2 (256-bit vectors)
    Avx2,
    /// Intel SSE4 (128-bit vectors)
    Sse4,
    /// ARM NEON (128-bit vectors)
    Neon,
    /// ARM SVE (Scalable Vector Extension)
    Sve,
}

impl SimdTarget {
    /// Parse target from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "avx512" | "avx-512" => Some(Self::Avx512),
            "avx2" | "avx-2" => Some(Self::Avx2),
            "sse4" | "sse4.2" | "sse" => Some(Self::Sse4),
            "neon" | "arm-neon" => Some(Self::Neon),
            "sve" | "arm-sve" => Some(Self::Sve),
            _ => None,
        }
    }

    /// Get vector width in bits
    pub fn vector_bits(&self) -> usize {
        match self {
            Self::Avx512 => 512,
            Self::Avx2 => 256,
            Self::Sse4 => 128,
            Self::Neon => 128,
            Self::Sve => 512, // Minimum SVE, can be larger
        }
    }

    /// Get number of f32 elements per vector
    pub fn f32_lanes(&self) -> usize {
        self.vector_bits() / 32
    }

    /// Get number of f64 elements per vector
    pub fn f64_lanes(&self) -> usize {
        self.vector_bits() / 64
    }

    /// Get number of i32 elements per vector
    pub fn i32_lanes(&self) -> usize {
        self.vector_bits() / 32
    }

    /// Get required compiler flags
    pub fn compiler_flags(&self) -> &'static str {
        match self {
            Self::Avx512 => "-mavx512f -mavx512dq -mavx512vl",
            Self::Avx2 => "-mavx2 -mfma",
            Self::Sse4 => "-msse4.2",
            Self::Neon => "-mfpu=neon",
            Self::Sve => "-march=armv8-a+sve",
        }
    }

    /// Get include headers
    pub fn headers(&self) -> &'static str {
        match self {
            Self::Avx512 | Self::Avx2 | Self::Sse4 => "#include <immintrin.h>",
            Self::Neon => "#include <arm_neon.h>",
            Self::Sve => "#include <arm_sve.h>",
        }
    }

    /// Get target name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Avx512 => "AVX-512",
            Self::Avx2 => "AVX2",
            Self::Sse4 => "SSE4",
            Self::Neon => "NEON",
            Self::Sve => "SVE",
        }
    }
}

/// SIMD vector type
#[derive(Debug, Clone, PartialEq)]
pub enum SimdVectorType {
    /// f32 vector (float)
    F32(usize),
    /// f64 vector (double)
    F64(usize),
    /// i32 vector (int)
    I32(usize),
    /// i64 vector (long long)
    I64(usize),
}

impl SimdVectorType {
    /// Get type name for target
    pub fn type_name(&self, target: SimdTarget) -> String {
        match target {
            SimdTarget::Avx512 => self.avx512_type(),
            SimdTarget::Avx2 => self.avx2_type(),
            SimdTarget::Sse4 => self.sse4_type(),
            SimdTarget::Neon => self.neon_type(),
            SimdTarget::Sve => self.sve_type(),
        }
    }

    fn avx512_type(&self) -> String {
        match self {
            SimdVectorType::F32(16) => "__m512".to_string(),
            SimdVectorType::F64(8) => "__m512d".to_string(),
            SimdVectorType::I32(16) => "__m512i".to_string(),
            SimdVectorType::I64(8) => "__m512i".to_string(),
            SimdVectorType::F32(8) => "__m256".to_string(),
            SimdVectorType::F64(4) => "__m256d".to_string(),
            SimdVectorType::I32(8) => "__m256i".to_string(),
            SimdVectorType::I64(4) => "__m256i".to_string(),
            _ => "void".to_string(),
        }
    }

    fn avx2_type(&self) -> String {
        match self {
            SimdVectorType::F32(8) => "__m256".to_string(),
            SimdVectorType::F64(4) => "__m256d".to_string(),
            SimdVectorType::I32(8) => "__m256i".to_string(),
            SimdVectorType::I64(4) => "__m256i".to_string(),
            SimdVectorType::F32(4) => "__m128".to_string(),
            SimdVectorType::F64(2) => "__m128d".to_string(),
            SimdVectorType::I32(4) => "__m128i".to_string(),
            SimdVectorType::I64(2) => "__m128i".to_string(),
            _ => "void".to_string(),
        }
    }

    fn sse4_type(&self) -> String {
        match self {
            SimdVectorType::F32(4) => "__m128".to_string(),
            SimdVectorType::F64(2) => "__m128d".to_string(),
            SimdVectorType::I32(4) => "__m128i".to_string(),
            SimdVectorType::I64(2) => "__m128i".to_string(),
            _ => "void".to_string(),
        }
    }

    fn neon_type(&self) -> String {
        match self {
            SimdVectorType::F32(4) => "float32x4_t".to_string(),
            SimdVectorType::F32(2) => "float32x2_t".to_string(),
            SimdVectorType::F64(2) => "float64x2_t".to_string(),
            SimdVectorType::F64(1) => "float64x1_t".to_string(),
            SimdVectorType::I32(4) => "int32x4_t".to_string(),
            SimdVectorType::I32(2) => "int32x2_t".to_string(),
            SimdVectorType::I64(2) => "int64x2_t".to_string(),
            SimdVectorType::I64(1) => "int64x1_t".to_string(),
            _ => "void".to_string(),
        }
    }

    fn sve_type(&self) -> String {
        match self {
            SimdVectorType::F32(_) => "svfloat32_t".to_string(),
            SimdVectorType::F64(_) => "svfloat64_t".to_string(),
            SimdVectorType::I32(_) => "svint32_t".to_string(),
            SimdVectorType::I64(_) => "svint64_t".to_string(),
        }
    }
}

/// SIMD intrinsic mappings
pub struct SimdIntrinsics;

impl SimdIntrinsics {
    /// Get load intrinsic
    pub fn load(target: SimdTarget, elem_type: &str) -> &'static str {
        match (target, elem_type) {
            (SimdTarget::Avx512, "f32") => "_mm512_loadu_ps",
            (SimdTarget::Avx512, "f64") => "_mm512_loadu_pd",
            (SimdTarget::Avx512, "i32") => "_mm512_loadu_si512",
            (SimdTarget::Avx2, "f32") => "_mm256_loadu_ps",
            (SimdTarget::Avx2, "f64") => "_mm256_loadu_pd",
            (SimdTarget::Avx2, "i32") => "_mm256_loadu_si256",
            (SimdTarget::Sse4, "f32") => "_mm_loadu_ps",
            (SimdTarget::Sse4, "f64") => "_mm_loadu_pd",
            (SimdTarget::Sse4, "i32") => "_mm_loadu_si128",
            (SimdTarget::Neon, "f32") => "vld1q_f32",
            (SimdTarget::Neon, "f64") => "vld1q_f64",
            (SimdTarget::Neon, "i32") => "vld1q_s32",
            (SimdTarget::Sve, "f32") => "svld1_f32",
            (SimdTarget::Sve, "f64") => "svld1_f64",
            (SimdTarget::Sve, "i32") => "svld1_s32",
            _ => "unknown_load",
        }
    }

    /// Get store intrinsic
    pub fn store(target: SimdTarget, elem_type: &str) -> &'static str {
        match (target, elem_type) {
            (SimdTarget::Avx512, "f32") => "_mm512_storeu_ps",
            (SimdTarget::Avx512, "f64") => "_mm512_storeu_pd",
            (SimdTarget::Avx512, "i32") => "_mm512_storeu_si512",
            (SimdTarget::Avx2, "f32") => "_mm256_storeu_ps",
            (SimdTarget::Avx2, "f64") => "_mm256_storeu_pd",
            (SimdTarget::Avx2, "i32") => "_mm256_storeu_si256",
            (SimdTarget::Sse4, "f32") => "_mm_storeu_ps",
            (SimdTarget::Sse4, "f64") => "_mm_storeu_pd",
            (SimdTarget::Sse4, "i32") => "_mm_storeu_si128",
            (SimdTarget::Neon, "f32") => "vst1q_f32",
            (SimdTarget::Neon, "f64") => "vst1q_f64",
            (SimdTarget::Neon, "i32") => "vst1q_s32",
            (SimdTarget::Sve, "f32") => "svst1_f32",
            (SimdTarget::Sve, "f64") => "svst1_f64",
            (SimdTarget::Sve, "i32") => "svst1_s32",
            _ => "unknown_store",
        }
    }

    /// Get add intrinsic
    pub fn add(target: SimdTarget, elem_type: &str) -> &'static str {
        match (target, elem_type) {
            (SimdTarget::Avx512, "f32") => "_mm512_add_ps",
            (SimdTarget::Avx512, "f64") => "_mm512_add_pd",
            (SimdTarget::Avx512, "i32") => "_mm512_add_epi32",
            (SimdTarget::Avx2, "f32") => "_mm256_add_ps",
            (SimdTarget::Avx2, "f64") => "_mm256_add_pd",
            (SimdTarget::Avx2, "i32") => "_mm256_add_epi32",
            (SimdTarget::Sse4, "f32") => "_mm_add_ps",
            (SimdTarget::Sse4, "f64") => "_mm_add_pd",
            (SimdTarget::Sse4, "i32") => "_mm_add_epi32",
            (SimdTarget::Neon, "f32") => "vaddq_f32",
            (SimdTarget::Neon, "f64") => "vaddq_f64",
            (SimdTarget::Neon, "i32") => "vaddq_s32",
            (SimdTarget::Sve, "f32") => "svadd_f32_x",
            (SimdTarget::Sve, "f64") => "svadd_f64_x",
            (SimdTarget::Sve, "i32") => "svadd_s32_x",
            _ => "unknown_add",
        }
    }

    /// Get subtract intrinsic
    pub fn sub(target: SimdTarget, elem_type: &str) -> &'static str {
        match (target, elem_type) {
            (SimdTarget::Avx512, "f32") => "_mm512_sub_ps",
            (SimdTarget::Avx512, "f64") => "_mm512_sub_pd",
            (SimdTarget::Avx512, "i32") => "_mm512_sub_epi32",
            (SimdTarget::Avx2, "f32") => "_mm256_sub_ps",
            (SimdTarget::Avx2, "f64") => "_mm256_sub_pd",
            (SimdTarget::Avx2, "i32") => "_mm256_sub_epi32",
            (SimdTarget::Sse4, "f32") => "_mm_sub_ps",
            (SimdTarget::Sse4, "f64") => "_mm_sub_pd",
            (SimdTarget::Sse4, "i32") => "_mm_sub_epi32",
            (SimdTarget::Neon, "f32") => "vsubq_f32",
            (SimdTarget::Neon, "f64") => "vsubq_f64",
            (SimdTarget::Neon, "i32") => "vsubq_s32",
            (SimdTarget::Sve, "f32") => "svsub_f32_x",
            (SimdTarget::Sve, "f64") => "svsub_f64_x",
            (SimdTarget::Sve, "i32") => "svsub_s32_x",
            _ => "unknown_sub",
        }
    }

    /// Get multiply intrinsic
    pub fn mul(target: SimdTarget, elem_type: &str) -> &'static str {
        match (target, elem_type) {
            (SimdTarget::Avx512, "f32") => "_mm512_mul_ps",
            (SimdTarget::Avx512, "f64") => "_mm512_mul_pd",
            (SimdTarget::Avx512, "i32") => "_mm512_mullo_epi32",
            (SimdTarget::Avx2, "f32") => "_mm256_mul_ps",
            (SimdTarget::Avx2, "f64") => "_mm256_mul_pd",
            (SimdTarget::Avx2, "i32") => "_mm256_mullo_epi32",
            (SimdTarget::Sse4, "f32") => "_mm_mul_ps",
            (SimdTarget::Sse4, "f64") => "_mm_mul_pd",
            (SimdTarget::Sse4, "i32") => "_mm_mullo_epi32",
            (SimdTarget::Neon, "f32") => "vmulq_f32",
            (SimdTarget::Neon, "f64") => "vmulq_f64",
            (SimdTarget::Neon, "i32") => "vmulq_s32",
            (SimdTarget::Sve, "f32") => "svmul_f32_x",
            (SimdTarget::Sve, "f64") => "svmul_f64_x",
            (SimdTarget::Sve, "i32") => "svmul_s32_x",
            _ => "unknown_mul",
        }
    }

    /// Get divide intrinsic
    pub fn div(target: SimdTarget, elem_type: &str) -> &'static str {
        match (target, elem_type) {
            (SimdTarget::Avx512, "f32") => "_mm512_div_ps",
            (SimdTarget::Avx512, "f64") => "_mm512_div_pd",
            (SimdTarget::Avx2, "f32") => "_mm256_div_ps",
            (SimdTarget::Avx2, "f64") => "_mm256_div_pd",
            (SimdTarget::Sse4, "f32") => "_mm_div_ps",
            (SimdTarget::Sse4, "f64") => "_mm_div_pd",
            (SimdTarget::Neon, "f32") => "vdivq_f32",
            (SimdTarget::Neon, "f64") => "vdivq_f64",
            (SimdTarget::Sve, "f32") => "svdiv_f32_x",
            (SimdTarget::Sve, "f64") => "svdiv_f64_x",
            _ => "unknown_div",
        }
    }

    /// Get fused multiply-add intrinsic
    pub fn fma(target: SimdTarget, elem_type: &str) -> &'static str {
        match (target, elem_type) {
            (SimdTarget::Avx512, "f32") => "_mm512_fmadd_ps",
            (SimdTarget::Avx512, "f64") => "_mm512_fmadd_pd",
            (SimdTarget::Avx2, "f32") => "_mm256_fmadd_ps",
            (SimdTarget::Avx2, "f64") => "_mm256_fmadd_pd",
            (SimdTarget::Sse4, "f32") => "_mm_fmadd_ps", // Requires FMA
            (SimdTarget::Sse4, "f64") => "_mm_fmadd_pd",
            (SimdTarget::Neon, "f32") => "vfmaq_f32",
            (SimdTarget::Neon, "f64") => "vfmaq_f64",
            (SimdTarget::Sve, "f32") => "svmla_f32_x",
            (SimdTarget::Sve, "f64") => "svmla_f64_x",
            _ => "unknown_fma",
        }
    }

    /// Get sqrt intrinsic
    pub fn sqrt(target: SimdTarget, elem_type: &str) -> &'static str {
        match (target, elem_type) {
            (SimdTarget::Avx512, "f32") => "_mm512_sqrt_ps",
            (SimdTarget::Avx512, "f64") => "_mm512_sqrt_pd",
            (SimdTarget::Avx2, "f32") => "_mm256_sqrt_ps",
            (SimdTarget::Avx2, "f64") => "_mm256_sqrt_pd",
            (SimdTarget::Sse4, "f32") => "_mm_sqrt_ps",
            (SimdTarget::Sse4, "f64") => "_mm_sqrt_pd",
            (SimdTarget::Neon, "f32") => "vsqrtq_f32",
            (SimdTarget::Neon, "f64") => "vsqrtq_f64",
            (SimdTarget::Sve, "f32") => "svsqrt_f32_x",
            (SimdTarget::Sve, "f64") => "svsqrt_f64_x",
            _ => "unknown_sqrt",
        }
    }

    /// Get horizontal sum intrinsic
    pub fn reduce_add(target: SimdTarget, elem_type: &str) -> &'static str {
        match (target, elem_type) {
            (SimdTarget::Avx512, "f32") => "_mm512_reduce_add_ps",
            (SimdTarget::Avx512, "f64") => "_mm512_reduce_add_pd",
            (SimdTarget::Avx512, "i32") => "_mm512_reduce_add_epi32",
            (SimdTarget::Neon, "f32") => "vaddvq_f32",
            (SimdTarget::Neon, "f64") => "vaddvq_f64",
            (SimdTarget::Neon, "i32") => "vaddvq_s32",
            (SimdTarget::Sve, "f32") => "svaddv_f32",
            (SimdTarget::Sve, "f64") => "svaddv_f64",
            (SimdTarget::Sve, "i32") => "svaddv_s32",
            _ => "unknown_reduce_add",
        }
    }

    /// Get broadcast intrinsic
    pub fn broadcast(target: SimdTarget, elem_type: &str) -> &'static str {
        match (target, elem_type) {
            (SimdTarget::Avx512, "f32") => "_mm512_set1_ps",
            (SimdTarget::Avx512, "f64") => "_mm512_set1_pd",
            (SimdTarget::Avx512, "i32") => "_mm512_set1_epi32",
            (SimdTarget::Avx2, "f32") => "_mm256_set1_ps",
            (SimdTarget::Avx2, "f64") => "_mm256_set1_pd",
            (SimdTarget::Avx2, "i32") => "_mm256_set1_epi32",
            (SimdTarget::Sse4, "f32") => "_mm_set1_ps",
            (SimdTarget::Sse4, "f64") => "_mm_set1_pd",
            (SimdTarget::Sse4, "i32") => "_mm_set1_epi32",
            (SimdTarget::Neon, "f32") => "vdupq_n_f32",
            (SimdTarget::Neon, "f64") => "vdupq_n_f64",
            (SimdTarget::Neon, "i32") => "vdupq_n_s32",
            (SimdTarget::Sve, "f32") => "svdup_f32",
            (SimdTarget::Sve, "f64") => "svdup_f64",
            (SimdTarget::Sve, "i32") => "svdup_s32",
            _ => "unknown_broadcast",
        }
    }

    /// Get min intrinsic
    pub fn min(target: SimdTarget, elem_type: &str) -> &'static str {
        match (target, elem_type) {
            (SimdTarget::Avx512, "f32") => "_mm512_min_ps",
            (SimdTarget::Avx512, "f64") => "_mm512_min_pd",
            (SimdTarget::Avx512, "i32") => "_mm512_min_epi32",
            (SimdTarget::Avx2, "f32") => "_mm256_min_ps",
            (SimdTarget::Avx2, "f64") => "_mm256_min_pd",
            (SimdTarget::Avx2, "i32") => "_mm256_min_epi32",
            (SimdTarget::Sse4, "f32") => "_mm_min_ps",
            (SimdTarget::Sse4, "f64") => "_mm_min_pd",
            (SimdTarget::Sse4, "i32") => "_mm_min_epi32",
            (SimdTarget::Neon, "f32") => "vminq_f32",
            (SimdTarget::Neon, "f64") => "vminq_f64",
            (SimdTarget::Neon, "i32") => "vminq_s32",
            (SimdTarget::Sve, "f32") => "svmin_f32_x",
            (SimdTarget::Sve, "f64") => "svmin_f64_x",
            (SimdTarget::Sve, "i32") => "svmin_s32_x",
            _ => "unknown_min",
        }
    }

    /// Get max intrinsic
    pub fn max(target: SimdTarget, elem_type: &str) -> &'static str {
        match (target, elem_type) {
            (SimdTarget::Avx512, "f32") => "_mm512_max_ps",
            (SimdTarget::Avx512, "f64") => "_mm512_max_pd",
            (SimdTarget::Avx512, "i32") => "_mm512_max_epi32",
            (SimdTarget::Avx2, "f32") => "_mm256_max_ps",
            (SimdTarget::Avx2, "f64") => "_mm256_max_pd",
            (SimdTarget::Avx2, "i32") => "_mm256_max_epi32",
            (SimdTarget::Sse4, "f32") => "_mm_max_ps",
            (SimdTarget::Sse4, "f64") => "_mm_max_pd",
            (SimdTarget::Sse4, "i32") => "_mm_max_epi32",
            (SimdTarget::Neon, "f32") => "vmaxq_f32",
            (SimdTarget::Neon, "f64") => "vmaxq_f64",
            (SimdTarget::Neon, "i32") => "vmaxq_s32",
            (SimdTarget::Sve, "f32") => "svmax_f32_x",
            (SimdTarget::Sve, "f64") => "svmax_f64_x",
            (SimdTarget::Sve, "i32") => "svmax_s32_x",
            _ => "unknown_max",
        }
    }
}

/// Generate SIMD-optimized C code
pub fn generate_simd_code(module: &Module, target: SimdTarget) -> GpuResult<String> {
    let mut generator = SimdGenerator::new(target);
    generator.generate_module(module)
}

struct SimdGenerator {
    target: SimdTarget,
    output: String,
    indent_level: usize,
}

impl SimdGenerator {
    fn new(target: SimdTarget) -> Self {
        Self {
            target,
            output: String::new(),
            indent_level: 0,
        }
    }

    fn generate_module(&mut self, module: &Module) -> GpuResult<String> {
        // Header
        self.emit_line("// Generated by Vais SIMD Compiler");
        self.emit_line(&format!("// Target: {}", self.target.name()));
        self.emit_line("");
        self.emit_line(self.target.headers());
        self.emit_line("#include <stdint.h>");
        self.emit_line("#include <stdlib.h>");
        self.emit_line("");

        // Generate items
        for item in &module.items {
            self.generate_item(&item.node)?;
        }

        Ok(self.output.clone())
    }

    fn generate_item(&mut self, item: &Item) -> GpuResult<()> {
        match item {
            Item::Function(func) => {
                // Check for SIMD attributes
                let is_simd = func
                    .attributes
                    .iter()
                    .any(|attr| attr.name == "simd" || attr.name == "vectorize");

                if is_simd {
                    self.generate_simd_function(func)?;
                } else {
                    self.generate_scalar_function(func)?;
                }
            }
            Item::Struct(s) => {
                self.generate_struct(&s.name.node, &s.fields)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn generate_simd_function(&mut self, func: &Function) -> GpuResult<()> {
        let name = &func.name.node;

        // Extract vectorization info from attributes
        let elem_type = self.extract_elem_type(&func.attributes);
        let _lanes = self.target.f32_lanes(); // Default to f32 lanes

        let ret_str = func
            .ret_type
            .as_ref()
            .map(|t| self.type_to_c(&t.node))
            .unwrap_or_else(|| "void".to_string());

        self.emit(&format!("{} {}(", ret_str, name));

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                self.emit(", ");
            }
            self.emit(&format!(
                "{} {}",
                self.type_to_c(&param.ty.node),
                param.name.node
            ));
        }

        self.emit_line(") {");
        self.indent_level += 1;

        // Generate SIMD-optimized body
        self.generate_simd_body(&func.body, &elem_type)?;

        self.indent_level -= 1;
        self.emit_line("}");
        self.emit_line("");

        Ok(())
    }

    fn generate_scalar_function(&mut self, func: &Function) -> GpuResult<()> {
        let name = &func.name.node;

        let ret_str = func
            .ret_type
            .as_ref()
            .map(|t| self.type_to_c(&t.node))
            .unwrap_or_else(|| "void".to_string());

        self.emit(&format!("{} {}(", ret_str, name));

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                self.emit(", ");
            }
            self.emit(&format!(
                "{} {}",
                self.type_to_c(&param.ty.node),
                param.name.node
            ));
        }

        self.emit_line(") {");
        self.indent_level += 1;

        match &func.body {
            FunctionBody::Expr(expr) => {
                self.emit_indent();
                if func.ret_type.is_some() {
                    self.emit("return ");
                }
                self.generate_expr(&expr.node)?;
                self.emit_line(";");
            }
            FunctionBody::Block(stmts) => {
                for stmt in stmts {
                    self.generate_stmt(&stmt.node)?;
                }
            }
        }

        self.indent_level -= 1;
        self.emit_line("}");
        self.emit_line("");

        Ok(())
    }

    fn generate_simd_body(&mut self, body: &FunctionBody, elem_type: &str) -> GpuResult<()> {
        // For now, generate standard body with SIMD intrinsic hints
        match body {
            FunctionBody::Expr(expr) => {
                self.emit_indent();
                self.generate_simd_expr(&expr.node, elem_type)?;
                self.emit_line(";");
            }
            FunctionBody::Block(stmts) => {
                for stmt in stmts {
                    self.generate_simd_stmt(&stmt.node, elem_type)?;
                }
            }
        }
        Ok(())
    }

    fn generate_simd_expr(&mut self, expr: &Expr, elem_type: &str) -> GpuResult<()> {
        match expr {
            Expr::Call { func, args } => {
                if let Expr::Ident(name) = &func.node {
                    // Map SIMD function calls
                    match name.as_str() {
                        "simd_load" => {
                            self.emit(SimdIntrinsics::load(self.target, elem_type));
                            self.emit("(");
                            if !args.is_empty() {
                                self.generate_expr(&args[0].node)?;
                            }
                            self.emit(")");
                            return Ok(());
                        }
                        "simd_store" => {
                            self.emit(SimdIntrinsics::store(self.target, elem_type));
                            self.emit("(");
                            for (i, arg) in args.iter().enumerate() {
                                if i > 0 {
                                    self.emit(", ");
                                }
                                self.generate_expr(&arg.node)?;
                            }
                            self.emit(")");
                            return Ok(());
                        }
                        "simd_add" => {
                            self.emit(SimdIntrinsics::add(self.target, elem_type));
                            self.emit("(");
                            for (i, arg) in args.iter().enumerate() {
                                if i > 0 {
                                    self.emit(", ");
                                }
                                self.generate_expr(&arg.node)?;
                            }
                            self.emit(")");
                            return Ok(());
                        }
                        "simd_mul" => {
                            self.emit(SimdIntrinsics::mul(self.target, elem_type));
                            self.emit("(");
                            for (i, arg) in args.iter().enumerate() {
                                if i > 0 {
                                    self.emit(", ");
                                }
                                self.generate_expr(&arg.node)?;
                            }
                            self.emit(")");
                            return Ok(());
                        }
                        "simd_fma" => {
                            self.emit(SimdIntrinsics::fma(self.target, elem_type));
                            self.emit("(");
                            for (i, arg) in args.iter().enumerate() {
                                if i > 0 {
                                    self.emit(", ");
                                }
                                self.generate_expr(&arg.node)?;
                            }
                            self.emit(")");
                            return Ok(());
                        }
                        "simd_broadcast" => {
                            self.emit(SimdIntrinsics::broadcast(self.target, elem_type));
                            self.emit("(");
                            if !args.is_empty() {
                                self.generate_expr(&args[0].node)?;
                            }
                            self.emit(")");
                            return Ok(());
                        }
                        "simd_reduce_add" => {
                            self.emit(SimdIntrinsics::reduce_add(self.target, elem_type));
                            self.emit("(");
                            if !args.is_empty() {
                                self.generate_expr(&args[0].node)?;
                            }
                            self.emit(")");
                            return Ok(());
                        }
                        _ => {}
                    }
                }
                // Fall through to normal call
                self.generate_expr(expr)?;
            }
            _ => {
                self.generate_expr(expr)?;
            }
        }
        Ok(())
    }

    fn generate_simd_stmt(&mut self, stmt: &Stmt, elem_type: &str) -> GpuResult<()> {
        match stmt {
            Stmt::Let {
                name, ty, value, ..
            } => {
                self.emit_indent();
                if let Some(t) = ty {
                    self.emit(&format!("{} {} = ", self.type_to_c(&t.node), name.node));
                } else {
                    self.emit(&format!("auto {} = ", name.node));
                }
                self.generate_simd_expr(&value.node, elem_type)?;
                self.emit_line(";");
            }
            Stmt::Expr(expr) => {
                self.emit_indent();
                self.generate_simd_expr(&expr.node, elem_type)?;
                self.emit_line(";");
            }
            _ => {
                self.generate_stmt(stmt)?;
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
                    self.emit(&format!("{} {} = ", self.type_to_c(&t.node), name.node));
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

    fn generate_expr(&mut self, expr: &Expr) -> GpuResult<()> {
        match expr {
            Expr::Int(n) => self.emit(&n.to_string()),
            Expr::Float(f) => self.emit(&format!("{:.6}", f)),
            Expr::Bool(b) => self.emit(if *b { "true" } else { "false" }),
            Expr::Ident(name) => self.emit(name),
            Expr::Binary { op, left, right } => {
                self.emit("(");
                self.generate_expr(&left.node)?;
                self.emit(&format!(" {} ", binary_op_str(op)));
                self.generate_expr(&right.node)?;
                self.emit(")");
            }
            Expr::Call { func, args } => {
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
            _ => {}
        }
        Ok(())
    }

    fn generate_struct(&mut self, name: &str, fields: &[vais_ast::Field]) -> GpuResult<()> {
        self.emit_line(&format!("typedef struct {} {{", name));
        self.indent_level += 1;

        for field in fields {
            self.emit_indent();
            self.emit_line(&format!(
                "{} {};",
                self.type_to_c(&field.ty.node),
                field.name.node
            ));
        }

        self.indent_level -= 1;
        self.emit_line(&format!("}} {};", name));
        self.emit_line("");

        Ok(())
    }

    fn extract_elem_type(&self, attrs: &[vais_ast::Attribute]) -> String {
        for attr in attrs {
            if attr.name == "simd" || attr.name == "vectorize" {
                if let Some(arg) = attr.args.first() {
                    // Parse element type: #[simd(f32)] or #[simd(f64)]
                    return arg.trim().to_string();
                }
            }
        }
        "f32".to_string() // Default
    }

    fn type_to_c(&self, ty: &Type) -> String {
        match ty {
            Type::Named { name, .. } => match name.as_str() {
                "i64" => "int64_t".to_string(),
                "i32" => "int32_t".to_string(),
                "i16" => "int16_t".to_string(),
                "i8" => "int8_t".to_string(),
                "u64" => "uint64_t".to_string(),
                "u32" => "uint32_t".to_string(),
                "u16" => "uint16_t".to_string(),
                "u8" => "uint8_t".to_string(),
                "f64" => "double".to_string(),
                "f32" => "float".to_string(),
                "bool" => "_Bool".to_string(),
                "unit" | "()" => "void".to_string(),
                _ => name.clone(),
            },
            Type::Pointer(inner) => format!("{}*", self.type_to_c(&inner.node)),
            Type::ConstArray { element, size } => {
                format!("{}[{}]", self.type_to_c(&element.node), size)
            }
            _ => "void".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── SimdTarget::parse tests ──

    #[test]
    fn test_simd_target_from_str() {
        assert_eq!(SimdTarget::parse("avx512"), Some(SimdTarget::Avx512));
        assert_eq!(SimdTarget::parse("avx2"), Some(SimdTarget::Avx2));
        assert_eq!(SimdTarget::parse("sse4"), Some(SimdTarget::Sse4));
        assert_eq!(SimdTarget::parse("neon"), Some(SimdTarget::Neon));
        assert_eq!(SimdTarget::parse("sve"), Some(SimdTarget::Sve));
        assert_eq!(SimdTarget::parse("unknown"), None);
    }

    #[test]
    fn test_simd_target_parse_aliases() {
        assert_eq!(SimdTarget::parse("avx-512"), Some(SimdTarget::Avx512));
        assert_eq!(SimdTarget::parse("avx-2"), Some(SimdTarget::Avx2));
        assert_eq!(SimdTarget::parse("sse4.2"), Some(SimdTarget::Sse4));
        assert_eq!(SimdTarget::parse("sse"), Some(SimdTarget::Sse4));
        assert_eq!(SimdTarget::parse("arm-neon"), Some(SimdTarget::Neon));
        assert_eq!(SimdTarget::parse("arm-sve"), Some(SimdTarget::Sve));
    }

    #[test]
    fn test_simd_target_parse_case_insensitive() {
        assert_eq!(SimdTarget::parse("AVX512"), Some(SimdTarget::Avx512));
        assert_eq!(SimdTarget::parse("AVX2"), Some(SimdTarget::Avx2));
        assert_eq!(SimdTarget::parse("SSE4"), Some(SimdTarget::Sse4));
        assert_eq!(SimdTarget::parse("NEON"), Some(SimdTarget::Neon));
        assert_eq!(SimdTarget::parse("SVE"), Some(SimdTarget::Sve));
    }

    #[test]
    fn test_simd_target_parse_unknown() {
        assert_eq!(SimdTarget::parse(""), None);
        assert_eq!(SimdTarget::parse("gpu"), None);
        assert_eq!(SimdTarget::parse("avx1"), None);
    }

    // ── SimdTarget::vector_bits tests ──

    #[test]
    fn test_simd_target_vector_bits() {
        assert_eq!(SimdTarget::Avx512.vector_bits(), 512);
        assert_eq!(SimdTarget::Avx2.vector_bits(), 256);
        assert_eq!(SimdTarget::Sse4.vector_bits(), 128);
        assert_eq!(SimdTarget::Neon.vector_bits(), 128);
        assert_eq!(SimdTarget::Sve.vector_bits(), 512);
    }

    // ── SimdTarget lane count tests ──

    #[test]
    fn test_simd_target_f32_lanes() {
        assert_eq!(SimdTarget::Avx512.f32_lanes(), 16);
        assert_eq!(SimdTarget::Avx2.f32_lanes(), 8);
        assert_eq!(SimdTarget::Sse4.f32_lanes(), 4);
        assert_eq!(SimdTarget::Neon.f32_lanes(), 4);
        assert_eq!(SimdTarget::Sve.f32_lanes(), 16);
    }

    #[test]
    fn test_simd_target_f64_lanes() {
        assert_eq!(SimdTarget::Avx512.f64_lanes(), 8);
        assert_eq!(SimdTarget::Avx2.f64_lanes(), 4);
        assert_eq!(SimdTarget::Sse4.f64_lanes(), 2);
        assert_eq!(SimdTarget::Neon.f64_lanes(), 2);
        assert_eq!(SimdTarget::Sve.f64_lanes(), 8);
    }

    #[test]
    fn test_simd_target_i32_lanes() {
        assert_eq!(SimdTarget::Avx512.i32_lanes(), 16);
        assert_eq!(SimdTarget::Avx2.i32_lanes(), 8);
        assert_eq!(SimdTarget::Sse4.i32_lanes(), 4);
        assert_eq!(SimdTarget::Neon.i32_lanes(), 4);
    }

    #[test]
    fn test_simd_lanes_consistent_with_bits() {
        for target in [
            SimdTarget::Avx512,
            SimdTarget::Avx2,
            SimdTarget::Sse4,
            SimdTarget::Neon,
            SimdTarget::Sve,
        ] {
            assert_eq!(target.f32_lanes(), target.vector_bits() / 32);
            assert_eq!(target.f64_lanes(), target.vector_bits() / 64);
            assert_eq!(target.i32_lanes(), target.vector_bits() / 32);
        }
    }

    // ── SimdTarget::compiler_flags tests ──

    #[test]
    fn test_simd_target_compiler_flags_all() {
        assert!(SimdTarget::Avx512.compiler_flags().contains("-mavx512f"));
        assert!(SimdTarget::Avx512.compiler_flags().contains("-mavx512dq"));
        assert!(SimdTarget::Avx2.compiler_flags().contains("-mavx2"));
        assert!(SimdTarget::Avx2.compiler_flags().contains("-mfma"));
        assert!(SimdTarget::Sse4.compiler_flags().contains("-msse4.2"));
        assert!(SimdTarget::Neon.compiler_flags().contains("-mfpu=neon"));
        assert!(SimdTarget::Sve
            .compiler_flags()
            .contains("-march=armv8-a+sve"));
    }

    // ── SimdTarget::headers tests ──

    #[test]
    fn test_simd_target_headers_all() {
        assert!(SimdTarget::Avx512.headers().contains("immintrin.h"));
        assert!(SimdTarget::Avx2.headers().contains("immintrin.h"));
        assert!(SimdTarget::Sse4.headers().contains("immintrin.h"));
        assert!(SimdTarget::Neon.headers().contains("arm_neon.h"));
        assert!(SimdTarget::Sve.headers().contains("arm_sve.h"));
    }

    #[test]
    fn test_simd_target_intel_share_header() {
        // All Intel SIMD targets share the same header
        assert_eq!(SimdTarget::Avx512.headers(), SimdTarget::Avx2.headers());
        assert_eq!(SimdTarget::Avx2.headers(), SimdTarget::Sse4.headers());
    }

    // ── SimdTarget::name tests ──

    #[test]
    fn test_simd_target_name_all() {
        assert_eq!(SimdTarget::Avx512.name(), "AVX-512");
        assert_eq!(SimdTarget::Avx2.name(), "AVX2");
        assert_eq!(SimdTarget::Sse4.name(), "SSE4");
        assert_eq!(SimdTarget::Neon.name(), "NEON");
        assert_eq!(SimdTarget::Sve.name(), "SVE");
    }

    // ── SimdTarget traits ──

    #[test]
    fn test_simd_target_clone_copy() {
        let t = SimdTarget::Avx512;
        let cloned = t.clone();
        let copied = t;
        assert_eq!(t, cloned);
        assert_eq!(t, copied);
    }

    #[test]
    fn test_simd_target_equality() {
        assert_eq!(SimdTarget::Avx512, SimdTarget::Avx512);
        assert_ne!(SimdTarget::Avx512, SimdTarget::Avx2);
    }

    #[test]
    fn test_simd_target_debug() {
        let s = format!("{:?}", SimdTarget::Neon);
        assert_eq!(s, "Neon");
    }

    // ── SimdVectorType::type_name tests ──

    #[test]
    fn test_simd_vector_type_avx512() {
        assert_eq!(
            SimdVectorType::F32(16).type_name(SimdTarget::Avx512),
            "__m512"
        );
        assert_eq!(
            SimdVectorType::F64(8).type_name(SimdTarget::Avx512),
            "__m512d"
        );
        assert_eq!(
            SimdVectorType::I32(16).type_name(SimdTarget::Avx512),
            "__m512i"
        );
        assert_eq!(
            SimdVectorType::I64(8).type_name(SimdTarget::Avx512),
            "__m512i"
        );
    }

    #[test]
    fn test_simd_vector_type_avx512_256() {
        assert_eq!(
            SimdVectorType::F32(8).type_name(SimdTarget::Avx512),
            "__m256"
        );
        assert_eq!(
            SimdVectorType::F64(4).type_name(SimdTarget::Avx512),
            "__m256d"
        );
    }

    #[test]
    fn test_simd_vector_type_avx2() {
        assert_eq!(SimdVectorType::F32(8).type_name(SimdTarget::Avx2), "__m256");
        assert_eq!(
            SimdVectorType::F64(4).type_name(SimdTarget::Avx2),
            "__m256d"
        );
        assert_eq!(
            SimdVectorType::I32(8).type_name(SimdTarget::Avx2),
            "__m256i"
        );
        assert_eq!(
            SimdVectorType::I64(4).type_name(SimdTarget::Avx2),
            "__m256i"
        );
    }

    #[test]
    fn test_simd_vector_type_avx2_128() {
        assert_eq!(SimdVectorType::F32(4).type_name(SimdTarget::Avx2), "__m128");
        assert_eq!(
            SimdVectorType::F64(2).type_name(SimdTarget::Avx2),
            "__m128d"
        );
        assert_eq!(
            SimdVectorType::I32(4).type_name(SimdTarget::Avx2),
            "__m128i"
        );
        assert_eq!(
            SimdVectorType::I64(2).type_name(SimdTarget::Avx2),
            "__m128i"
        );
    }

    #[test]
    fn test_simd_vector_type_sse4() {
        assert_eq!(SimdVectorType::F32(4).type_name(SimdTarget::Sse4), "__m128");
        assert_eq!(
            SimdVectorType::F64(2).type_name(SimdTarget::Sse4),
            "__m128d"
        );
        assert_eq!(
            SimdVectorType::I32(4).type_name(SimdTarget::Sse4),
            "__m128i"
        );
        assert_eq!(
            SimdVectorType::I64(2).type_name(SimdTarget::Sse4),
            "__m128i"
        );
    }

    #[test]
    fn test_simd_vector_type_neon() {
        assert_eq!(
            SimdVectorType::F32(4).type_name(SimdTarget::Neon),
            "float32x4_t"
        );
        assert_eq!(
            SimdVectorType::F32(2).type_name(SimdTarget::Neon),
            "float32x2_t"
        );
        assert_eq!(
            SimdVectorType::F64(2).type_name(SimdTarget::Neon),
            "float64x2_t"
        );
        assert_eq!(
            SimdVectorType::F64(1).type_name(SimdTarget::Neon),
            "float64x1_t"
        );
        assert_eq!(
            SimdVectorType::I32(4).type_name(SimdTarget::Neon),
            "int32x4_t"
        );
        assert_eq!(
            SimdVectorType::I32(2).type_name(SimdTarget::Neon),
            "int32x2_t"
        );
        assert_eq!(
            SimdVectorType::I64(2).type_name(SimdTarget::Neon),
            "int64x2_t"
        );
        assert_eq!(
            SimdVectorType::I64(1).type_name(SimdTarget::Neon),
            "int64x1_t"
        );
    }

    #[test]
    fn test_simd_vector_type_sve() {
        assert_eq!(
            SimdVectorType::F32(16).type_name(SimdTarget::Sve),
            "svfloat32_t"
        );
        assert_eq!(
            SimdVectorType::F64(8).type_name(SimdTarget::Sve),
            "svfloat64_t"
        );
        assert_eq!(
            SimdVectorType::I32(16).type_name(SimdTarget::Sve),
            "svint32_t"
        );
        assert_eq!(
            SimdVectorType::I64(8).type_name(SimdTarget::Sve),
            "svint64_t"
        );
    }

    #[test]
    fn test_simd_vector_type_unsupported_returns_void() {
        // Unsupported lane counts should return "void" for x86 targets
        assert_eq!(SimdVectorType::F32(3).type_name(SimdTarget::Sse4), "void");
        assert_eq!(SimdVectorType::I32(7).type_name(SimdTarget::Avx2), "void");
    }

    #[test]
    fn test_simd_vector_type_equality() {
        assert_eq!(SimdVectorType::F32(4), SimdVectorType::F32(4));
        assert_ne!(SimdVectorType::F32(4), SimdVectorType::F32(8));
        assert_ne!(SimdVectorType::F32(4), SimdVectorType::F64(4));
    }

    #[test]
    fn test_simd_vector_type_clone() {
        let ty = SimdVectorType::I64(4);
        let cloned = ty.clone();
        assert_eq!(ty, cloned);
    }

    // ── SimdIntrinsics tests ──

    #[test]
    fn test_simd_intrinsics_load_all_targets() {
        // f32
        assert_eq!(
            SimdIntrinsics::load(SimdTarget::Avx512, "f32"),
            "_mm512_loadu_ps"
        );
        assert_eq!(
            SimdIntrinsics::load(SimdTarget::Avx2, "f32"),
            "_mm256_loadu_ps"
        );
        assert_eq!(
            SimdIntrinsics::load(SimdTarget::Sse4, "f32"),
            "_mm_loadu_ps"
        );
        assert_eq!(SimdIntrinsics::load(SimdTarget::Neon, "f32"), "vld1q_f32");
        assert_eq!(SimdIntrinsics::load(SimdTarget::Sve, "f32"), "svld1_f32");
    }

    #[test]
    fn test_simd_intrinsics_load_f64() {
        assert_eq!(
            SimdIntrinsics::load(SimdTarget::Avx512, "f64"),
            "_mm512_loadu_pd"
        );
        assert_eq!(
            SimdIntrinsics::load(SimdTarget::Avx2, "f64"),
            "_mm256_loadu_pd"
        );
        assert_eq!(
            SimdIntrinsics::load(SimdTarget::Sse4, "f64"),
            "_mm_loadu_pd"
        );
        assert_eq!(SimdIntrinsics::load(SimdTarget::Neon, "f64"), "vld1q_f64");
        assert_eq!(SimdIntrinsics::load(SimdTarget::Sve, "f64"), "svld1_f64");
    }

    #[test]
    fn test_simd_intrinsics_load_i32() {
        assert_eq!(
            SimdIntrinsics::load(SimdTarget::Avx512, "i32"),
            "_mm512_loadu_si512"
        );
        assert_eq!(
            SimdIntrinsics::load(SimdTarget::Avx2, "i32"),
            "_mm256_loadu_si256"
        );
        assert_eq!(
            SimdIntrinsics::load(SimdTarget::Sse4, "i32"),
            "_mm_loadu_si128"
        );
        assert_eq!(SimdIntrinsics::load(SimdTarget::Neon, "i32"), "vld1q_s32");
    }

    #[test]
    fn test_simd_intrinsics_load_unknown_type() {
        assert_eq!(
            SimdIntrinsics::load(SimdTarget::Avx512, "i8"),
            "unknown_load"
        );
    }

    #[test]
    fn test_simd_intrinsics_store_all_targets_f32() {
        assert_eq!(
            SimdIntrinsics::store(SimdTarget::Avx512, "f32"),
            "_mm512_storeu_ps"
        );
        assert_eq!(
            SimdIntrinsics::store(SimdTarget::Avx2, "f32"),
            "_mm256_storeu_ps"
        );
        assert_eq!(
            SimdIntrinsics::store(SimdTarget::Sse4, "f32"),
            "_mm_storeu_ps"
        );
        assert_eq!(SimdIntrinsics::store(SimdTarget::Neon, "f32"), "vst1q_f32");
        assert_eq!(SimdIntrinsics::store(SimdTarget::Sve, "f32"), "svst1_f32");
    }

    #[test]
    fn test_simd_intrinsics_add_all_targets_f32() {
        assert_eq!(
            SimdIntrinsics::add(SimdTarget::Avx512, "f32"),
            "_mm512_add_ps"
        );
        assert_eq!(
            SimdIntrinsics::add(SimdTarget::Avx2, "f32"),
            "_mm256_add_ps"
        );
        assert_eq!(SimdIntrinsics::add(SimdTarget::Sse4, "f32"), "_mm_add_ps");
        assert_eq!(SimdIntrinsics::add(SimdTarget::Neon, "f32"), "vaddq_f32");
        assert_eq!(SimdIntrinsics::add(SimdTarget::Sve, "f32"), "svadd_f32_x");
    }

    #[test]
    fn test_simd_intrinsics_sub_all_targets_f32() {
        assert_eq!(
            SimdIntrinsics::sub(SimdTarget::Avx512, "f32"),
            "_mm512_sub_ps"
        );
        assert_eq!(
            SimdIntrinsics::sub(SimdTarget::Avx2, "f32"),
            "_mm256_sub_ps"
        );
        assert_eq!(SimdIntrinsics::sub(SimdTarget::Sse4, "f32"), "_mm_sub_ps");
        assert_eq!(SimdIntrinsics::sub(SimdTarget::Neon, "f32"), "vsubq_f32");
    }

    #[test]
    fn test_simd_intrinsics_mul_all_targets_f32() {
        assert_eq!(
            SimdIntrinsics::mul(SimdTarget::Avx512, "f32"),
            "_mm512_mul_ps"
        );
        assert_eq!(
            SimdIntrinsics::mul(SimdTarget::Avx2, "f32"),
            "_mm256_mul_ps"
        );
        assert_eq!(SimdIntrinsics::mul(SimdTarget::Sse4, "f32"), "_mm_mul_ps");
        assert_eq!(SimdIntrinsics::mul(SimdTarget::Neon, "f32"), "vmulq_f32");
    }

    #[test]
    fn test_simd_intrinsics_div_all_targets_f32() {
        assert_eq!(
            SimdIntrinsics::div(SimdTarget::Avx512, "f32"),
            "_mm512_div_ps"
        );
        assert_eq!(
            SimdIntrinsics::div(SimdTarget::Avx2, "f32"),
            "_mm256_div_ps"
        );
        assert_eq!(SimdIntrinsics::div(SimdTarget::Sse4, "f32"), "_mm_div_ps");
        assert_eq!(SimdIntrinsics::div(SimdTarget::Neon, "f32"), "vdivq_f32");
    }

    #[test]
    fn test_simd_intrinsics_div_no_i32() {
        // Integer division is not natively supported
        assert_eq!(
            SimdIntrinsics::div(SimdTarget::Avx512, "i32"),
            "unknown_div"
        );
        assert_eq!(SimdIntrinsics::div(SimdTarget::Sse4, "i32"), "unknown_div");
    }

    #[test]
    fn test_simd_intrinsics_fma_all_targets() {
        assert_eq!(
            SimdIntrinsics::fma(SimdTarget::Avx512, "f32"),
            "_mm512_fmadd_ps"
        );
        assert_eq!(
            SimdIntrinsics::fma(SimdTarget::Avx512, "f64"),
            "_mm512_fmadd_pd"
        );
        assert_eq!(
            SimdIntrinsics::fma(SimdTarget::Avx2, "f32"),
            "_mm256_fmadd_ps"
        );
        assert_eq!(SimdIntrinsics::fma(SimdTarget::Sse4, "f32"), "_mm_fmadd_ps");
        assert_eq!(SimdIntrinsics::fma(SimdTarget::Neon, "f32"), "vfmaq_f32");
        assert_eq!(SimdIntrinsics::fma(SimdTarget::Sve, "f64"), "svmla_f64_x");
    }

    #[test]
    fn test_simd_intrinsics_fma_no_i32() {
        assert_eq!(
            SimdIntrinsics::fma(SimdTarget::Avx512, "i32"),
            "unknown_fma"
        );
    }

    #[test]
    fn test_simd_intrinsics_sqrt_all_targets() {
        assert_eq!(
            SimdIntrinsics::sqrt(SimdTarget::Avx512, "f32"),
            "_mm512_sqrt_ps"
        );
        assert_eq!(
            SimdIntrinsics::sqrt(SimdTarget::Avx2, "f32"),
            "_mm256_sqrt_ps"
        );
        assert_eq!(SimdIntrinsics::sqrt(SimdTarget::Sse4, "f32"), "_mm_sqrt_ps");
        assert_eq!(SimdIntrinsics::sqrt(SimdTarget::Neon, "f32"), "vsqrtq_f32");
        assert_eq!(SimdIntrinsics::sqrt(SimdTarget::Sve, "f32"), "svsqrt_f32_x");
    }

    #[test]
    fn test_simd_intrinsics_sqrt_no_i32() {
        assert_eq!(
            SimdIntrinsics::sqrt(SimdTarget::Avx512, "i32"),
            "unknown_sqrt"
        );
    }

    #[test]
    fn test_simd_intrinsics_reduce_add() {
        assert_eq!(
            SimdIntrinsics::reduce_add(SimdTarget::Avx512, "f32"),
            "_mm512_reduce_add_ps"
        );
        assert_eq!(
            SimdIntrinsics::reduce_add(SimdTarget::Avx512, "f64"),
            "_mm512_reduce_add_pd"
        );
        assert_eq!(
            SimdIntrinsics::reduce_add(SimdTarget::Avx512, "i32"),
            "_mm512_reduce_add_epi32"
        );
        assert_eq!(
            SimdIntrinsics::reduce_add(SimdTarget::Neon, "f32"),
            "vaddvq_f32"
        );
        assert_eq!(
            SimdIntrinsics::reduce_add(SimdTarget::Sve, "f32"),
            "svaddv_f32"
        );
    }

    #[test]
    fn test_simd_intrinsics_reduce_add_no_avx2() {
        // AVX2 doesn't have native reduce_add
        assert_eq!(
            SimdIntrinsics::reduce_add(SimdTarget::Avx2, "f32"),
            "unknown_reduce_add"
        );
    }

    #[test]
    fn test_simd_intrinsics_broadcast_all_targets() {
        assert_eq!(
            SimdIntrinsics::broadcast(SimdTarget::Avx512, "f32"),
            "_mm512_set1_ps"
        );
        assert_eq!(
            SimdIntrinsics::broadcast(SimdTarget::Avx2, "f32"),
            "_mm256_set1_ps"
        );
        assert_eq!(
            SimdIntrinsics::broadcast(SimdTarget::Sse4, "f32"),
            "_mm_set1_ps"
        );
        assert_eq!(
            SimdIntrinsics::broadcast(SimdTarget::Neon, "f32"),
            "vdupq_n_f32"
        );
        assert_eq!(
            SimdIntrinsics::broadcast(SimdTarget::Sve, "f32"),
            "svdup_f32"
        );
    }

    #[test]
    fn test_simd_intrinsics_min_all_targets() {
        assert_eq!(
            SimdIntrinsics::min(SimdTarget::Avx512, "f32"),
            "_mm512_min_ps"
        );
        assert_eq!(
            SimdIntrinsics::min(SimdTarget::Avx2, "f32"),
            "_mm256_min_ps"
        );
        assert_eq!(SimdIntrinsics::min(SimdTarget::Sse4, "f32"), "_mm_min_ps");
        assert_eq!(SimdIntrinsics::min(SimdTarget::Neon, "f32"), "vminq_f32");
        assert_eq!(SimdIntrinsics::min(SimdTarget::Sve, "f32"), "svmin_f32_x");
    }

    #[test]
    fn test_simd_intrinsics_max_all_targets() {
        assert_eq!(
            SimdIntrinsics::max(SimdTarget::Avx512, "f32"),
            "_mm512_max_ps"
        );
        assert_eq!(
            SimdIntrinsics::max(SimdTarget::Avx2, "f32"),
            "_mm256_max_ps"
        );
        assert_eq!(SimdIntrinsics::max(SimdTarget::Sse4, "f32"), "_mm_max_ps");
        assert_eq!(SimdIntrinsics::max(SimdTarget::Neon, "f32"), "vmaxq_f32");
        assert_eq!(SimdIntrinsics::max(SimdTarget::Sve, "f32"), "svmax_f32_x");
    }

    #[test]
    fn test_simd_intrinsics_i32_arithmetic() {
        assert_eq!(
            SimdIntrinsics::add(SimdTarget::Avx512, "i32"),
            "_mm512_add_epi32"
        );
        assert_eq!(
            SimdIntrinsics::sub(SimdTarget::Avx512, "i32"),
            "_mm512_sub_epi32"
        );
        assert_eq!(
            SimdIntrinsics::mul(SimdTarget::Avx512, "i32"),
            "_mm512_mullo_epi32"
        );
    }

    #[test]
    fn test_simd_intrinsics_neon_i32() {
        assert_eq!(SimdIntrinsics::add(SimdTarget::Neon, "i32"), "vaddq_s32");
        assert_eq!(SimdIntrinsics::sub(SimdTarget::Neon, "i32"), "vsubq_s32");
        assert_eq!(SimdIntrinsics::mul(SimdTarget::Neon, "i32"), "vmulq_s32");
        assert_eq!(SimdIntrinsics::min(SimdTarget::Neon, "i32"), "vminq_s32");
        assert_eq!(SimdIntrinsics::max(SimdTarget::Neon, "i32"), "vmaxq_s32");
    }

    #[test]
    fn test_simd_intrinsics_sve_f64() {
        assert_eq!(SimdIntrinsics::add(SimdTarget::Sve, "f64"), "svadd_f64_x");
        assert_eq!(SimdIntrinsics::sub(SimdTarget::Sve, "f64"), "svsub_f64_x");
        assert_eq!(SimdIntrinsics::mul(SimdTarget::Sve, "f64"), "svmul_f64_x");
        assert_eq!(SimdIntrinsics::div(SimdTarget::Sve, "f64"), "svdiv_f64_x");
    }
}
