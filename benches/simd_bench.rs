//! SIMD Vector Operations Benchmark
//!
//! Benchmarks 1536-dim f32 vector distance calculations:
//! - Dot product (scalar vs SIMD)
//! - Cosine distance (scalar vs SIMD)
//! - L2 (Euclidean) distance (scalar vs SIMD)
//!
//! Target: > 5x speedup with SIMD over scalar implementation.
//! Validates Vais SIMD codegen patterns using equivalent Rust SIMD.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

const DIM: usize = 1536;
const BATCH_SIZE: usize = 1000;

// ============================================================================
// Vector generation helpers
// ============================================================================

fn generate_vector(seed: u64) -> Vec<f32> {
    let mut v = Vec::with_capacity(DIM);
    let mut x = seed as f32 * 0.001;
    for _ in 0..DIM {
        v.push(x.sin());
        x += 0.7;
    }
    // Normalize
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for val in &mut v {
            *val /= norm;
        }
    }
    v
}

fn generate_vectors(n: usize) -> Vec<Vec<f32>> {
    (0..n).map(|i| generate_vector(i as u64)).collect()
}

// ============================================================================
// Scalar implementations (baseline)
// ============================================================================

#[inline(never)]
fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for i in 0..a.len() {
        sum += a[i] * b[i];
    }
    sum
}

#[inline(never)]
fn cosine_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom > 0.0 {
        1.0 - dot / denom
    } else {
        1.0
    }
}

#[inline(never)]
fn l2_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for i in 0..a.len() {
        let d = a[i] - b[i];
        sum += d * d;
    }
    sum.sqrt()
}

// ============================================================================
// SIMD implementations (matches Vais LLVM IR patterns)
// ============================================================================

// Uses explicit 4-wide f32 operations matching Vais vec4f32 codegen
#[inline(never)]
fn dot_product_simd4(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len();
    let chunks = n / 4;
    let mut sum0 = 0.0f32;
    let mut sum1 = 0.0f32;
    let mut sum2 = 0.0f32;
    let mut sum3 = 0.0f32;

    for i in 0..chunks {
        let base = i * 4;
        sum0 += a[base] * b[base];
        sum1 += a[base + 1] * b[base + 1];
        sum2 += a[base + 2] * b[base + 2];
        sum3 += a[base + 3] * b[base + 3];
    }

    let mut total = sum0 + sum1 + sum2 + sum3;
    // Handle remainder
    for i in (chunks * 4)..n {
        total += a[i] * b[i];
    }
    total
}

// Uses 8-wide f32 operations matching Vais vec8f32 / AVX2 codegen
#[inline(never)]
fn dot_product_simd8(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len();
    let chunks = n / 8;
    let mut acc = [0.0f32; 8];

    for i in 0..chunks {
        let base = i * 8;
        for j in 0..8 {
            acc[j] += a[base + j] * b[base + j];
        }
    }

    let mut total: f32 = acc.iter().sum();
    for i in (chunks * 8)..n {
        total += a[i] * b[i];
    }
    total
}

#[inline(never)]
fn cosine_distance_simd8(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len();
    let chunks = n / 8;
    let mut dot_acc = [0.0f32; 8];
    let mut na_acc = [0.0f32; 8];
    let mut nb_acc = [0.0f32; 8];

    for i in 0..chunks {
        let base = i * 8;
        for j in 0..8 {
            let av = a[base + j];
            let bv = b[base + j];
            dot_acc[j] += av * bv;
            na_acc[j] += av * av;
            nb_acc[j] += bv * bv;
        }
    }

    let mut dot: f32 = dot_acc.iter().sum();
    let mut norm_a: f32 = na_acc.iter().sum();
    let mut norm_b: f32 = nb_acc.iter().sum();

    for i in (chunks * 8)..n {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom > 0.0 {
        1.0 - dot / denom
    } else {
        1.0
    }
}

#[inline(never)]
fn l2_distance_simd8(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len();
    let chunks = n / 8;
    let mut acc = [0.0f32; 8];

    for i in 0..chunks {
        let base = i * 8;
        for j in 0..8 {
            let d = a[base + j] - b[base + j];
            acc[j] += d * d;
        }
    }

    let mut total: f32 = acc.iter().sum();
    for i in (chunks * 8)..n {
        let d = a[i] - b[i];
        total += d * d;
    }
    total.sqrt()
}

// ============================================================================
// Platform-specific SIMD using std::arch
// ============================================================================

#[cfg(target_arch = "x86_64")]
mod x86_simd {
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    #[inline(never)]
    #[target_feature(enable = "avx2")]
    pub unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {
        let n = a.len();
        let chunks = n / 8;
        let mut sum = _mm256_setzero_ps();

        for i in 0..chunks {
            let base = i * 8;
            let va = _mm256_loadu_ps(a.as_ptr().add(base));
            let vb = _mm256_loadu_ps(b.as_ptr().add(base));
            sum = _mm256_fmadd_ps(va, vb, sum);
        }

        // Horizontal sum
        let hi = _mm256_extractf128_ps(sum, 1);
        let lo = _mm256_castps256_ps128(sum);
        let sum128 = _mm_add_ps(lo, hi);
        let shuf = _mm_movehdup_ps(sum128);
        let sums = _mm_add_ps(sum128, shuf);
        let shuf2 = _mm_movehl_ps(sums, sums);
        let result = _mm_add_ss(sums, shuf2);
        let mut total = _mm_cvtss_f32(result);

        for i in (chunks * 8)..n {
            total += a[i] * b[i];
        }
        total
    }

    #[inline(never)]
    #[target_feature(enable = "avx2")]
    pub unsafe fn cosine_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
        let n = a.len();
        let chunks = n / 8;
        let mut dot_sum = _mm256_setzero_ps();
        let mut na_sum = _mm256_setzero_ps();
        let mut nb_sum = _mm256_setzero_ps();

        for i in 0..chunks {
            let base = i * 8;
            let va = _mm256_loadu_ps(a.as_ptr().add(base));
            let vb = _mm256_loadu_ps(b.as_ptr().add(base));
            dot_sum = _mm256_fmadd_ps(va, vb, dot_sum);
            na_sum = _mm256_fmadd_ps(va, va, na_sum);
            nb_sum = _mm256_fmadd_ps(vb, vb, nb_sum);
        }

        let hsum = |v: __m256| -> f32 {
            let hi = _mm256_extractf128_ps(v, 1);
            let lo = _mm256_castps256_ps128(v);
            let sum128 = _mm_add_ps(lo, hi);
            let shuf = _mm_movehdup_ps(sum128);
            let sums = _mm_add_ps(sum128, shuf);
            let shuf2 = _mm_movehl_ps(sums, sums);
            _mm_cvtss_f32(_mm_add_ss(sums, shuf2))
        };

        let mut dot = hsum(dot_sum);
        let mut norm_a = hsum(na_sum);
        let mut norm_b = hsum(nb_sum);

        for i in (chunks * 8)..n {
            dot += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }

        let denom = norm_a.sqrt() * norm_b.sqrt();
        if denom > 0.0 {
            1.0 - dot / denom
        } else {
            1.0
        }
    }

    #[inline(never)]
    #[target_feature(enable = "avx2")]
    pub unsafe fn l2_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
        let n = a.len();
        let chunks = n / 8;
        let mut sum = _mm256_setzero_ps();

        for i in 0..chunks {
            let base = i * 8;
            let va = _mm256_loadu_ps(a.as_ptr().add(base));
            let vb = _mm256_loadu_ps(b.as_ptr().add(base));
            let diff = _mm256_sub_ps(va, vb);
            sum = _mm256_fmadd_ps(diff, diff, sum);
        }

        let hi = _mm256_extractf128_ps(sum, 1);
        let lo = _mm256_castps256_ps128(sum);
        let sum128 = _mm_add_ps(lo, hi);
        let shuf = _mm_movehdup_ps(sum128);
        let sums = _mm_add_ps(sum128, shuf);
        let shuf2 = _mm_movehl_ps(sums, sums);
        let mut total = _mm_cvtss_f32(_mm_add_ss(sums, shuf2));

        for i in (chunks * 8)..n {
            let d = a[i] - b[i];
            total += d * d;
        }
        total.sqrt()
    }
}

#[cfg(target_arch = "aarch64")]
mod neon_simd {
    use std::arch::aarch64::*;

    #[inline(never)]
    pub unsafe fn dot_product_neon(a: &[f32], b: &[f32]) -> f32 {
        let n = a.len();
        let chunks = n / 4;
        let mut sum = vdupq_n_f32(0.0);

        for i in 0..chunks {
            let base = i * 4;
            let va = vld1q_f32(a.as_ptr().add(base));
            let vb = vld1q_f32(b.as_ptr().add(base));
            sum = vfmaq_f32(sum, va, vb);
        }

        let mut total = vaddvq_f32(sum);
        for i in (chunks * 4)..n {
            total += a[i] * b[i];
        }
        total
    }

    #[inline(never)]
    pub unsafe fn cosine_distance_neon(a: &[f32], b: &[f32]) -> f32 {
        let n = a.len();
        let chunks = n / 4;
        let mut dot_sum = vdupq_n_f32(0.0);
        let mut na_sum = vdupq_n_f32(0.0);
        let mut nb_sum = vdupq_n_f32(0.0);

        for i in 0..chunks {
            let base = i * 4;
            let va = vld1q_f32(a.as_ptr().add(base));
            let vb = vld1q_f32(b.as_ptr().add(base));
            dot_sum = vfmaq_f32(dot_sum, va, vb);
            na_sum = vfmaq_f32(na_sum, va, va);
            nb_sum = vfmaq_f32(nb_sum, vb, vb);
        }

        let mut dot = vaddvq_f32(dot_sum);
        let mut norm_a = vaddvq_f32(na_sum);
        let mut norm_b = vaddvq_f32(nb_sum);

        for i in (chunks * 4)..n {
            dot += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }

        let denom = norm_a.sqrt() * norm_b.sqrt();
        if denom > 0.0 {
            1.0 - dot / denom
        } else {
            1.0
        }
    }

    #[inline(never)]
    pub unsafe fn l2_distance_neon(a: &[f32], b: &[f32]) -> f32 {
        let n = a.len();
        let chunks = n / 4;
        let mut sum = vdupq_n_f32(0.0);

        for i in 0..chunks {
            let base = i * 4;
            let va = vld1q_f32(a.as_ptr().add(base));
            let vb = vld1q_f32(b.as_ptr().add(base));
            let diff = vsubq_f32(va, vb);
            sum = vfmaq_f32(sum, diff, diff);
        }

        let mut total = vaddvq_f32(sum);
        for i in (chunks * 4)..n {
            let d = a[i] - b[i];
            total += d * d;
        }
        total.sqrt()
    }
}

// ============================================================================
// Benchmarks
// ============================================================================

fn bench_dot_product(c: &mut Criterion) {
    let mut group = c.benchmark_group("dot_product_1536dim");
    let a = generate_vector(42);
    let b = generate_vector(99);

    group.bench_function("scalar", |bench| {
        bench.iter(|| dot_product_scalar(black_box(&a), black_box(&b)))
    });

    group.bench_function("simd4_unrolled", |bench| {
        bench.iter(|| dot_product_simd4(black_box(&a), black_box(&b)))
    });

    group.bench_function("simd8_unrolled", |bench| {
        bench.iter(|| dot_product_simd8(black_box(&a), black_box(&b)))
    });

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            group.bench_function("avx2_fma", |bench| {
                bench.iter(|| unsafe {
                    x86_simd::dot_product_avx2(black_box(&a), black_box(&b))
                })
            });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        group.bench_function("neon_fma", |bench| {
            bench.iter(|| unsafe {
                neon_simd::dot_product_neon(black_box(&a), black_box(&b))
            })
        });
    }

    group.finish();
}

fn bench_cosine_distance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cosine_distance_1536dim");
    let a = generate_vector(42);
    let b = generate_vector(99);

    group.bench_function("scalar", |bench| {
        bench.iter(|| cosine_distance_scalar(black_box(&a), black_box(&b)))
    });

    group.bench_function("simd8_unrolled", |bench| {
        bench.iter(|| cosine_distance_simd8(black_box(&a), black_box(&b)))
    });

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            group.bench_function("avx2_fma", |bench| {
                bench.iter(|| unsafe {
                    x86_simd::cosine_distance_avx2(black_box(&a), black_box(&b))
                })
            });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        group.bench_function("neon_fma", |bench| {
            bench.iter(|| unsafe {
                neon_simd::cosine_distance_neon(black_box(&a), black_box(&b))
            })
        });
    }

    group.finish();
}

fn bench_l2_distance(c: &mut Criterion) {
    let mut group = c.benchmark_group("l2_distance_1536dim");
    let a = generate_vector(42);
    let b = generate_vector(99);

    group.bench_function("scalar", |bench| {
        bench.iter(|| l2_distance_scalar(black_box(&a), black_box(&b)))
    });

    group.bench_function("simd8_unrolled", |bench| {
        bench.iter(|| l2_distance_simd8(black_box(&a), black_box(&b)))
    });

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            group.bench_function("avx2_fma", |bench| {
                bench.iter(|| unsafe {
                    x86_simd::l2_distance_avx2(black_box(&a), black_box(&b))
                })
            });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        group.bench_function("neon_fma", |bench| {
            bench.iter(|| unsafe {
                neon_simd::l2_distance_neon(black_box(&a), black_box(&b))
            })
        });
    }

    group.finish();
}

fn bench_batch_cosine(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_cosine_distance");
    let query = generate_vector(0);
    let vectors = generate_vectors(BATCH_SIZE);

    group.bench_with_input(
        BenchmarkId::new("scalar", BATCH_SIZE),
        &BATCH_SIZE,
        |bench, _| {
            bench.iter(|| {
                let mut best = f32::MAX;
                for v in &vectors {
                    let d = cosine_distance_scalar(black_box(&query), black_box(v));
                    if d < best {
                        best = d;
                    }
                }
                best
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("simd8", BATCH_SIZE),
        &BATCH_SIZE,
        |bench, _| {
            bench.iter(|| {
                let mut best = f32::MAX;
                for v in &vectors {
                    let d = cosine_distance_simd8(black_box(&query), black_box(v));
                    if d < best {
                        best = d;
                    }
                }
                best
            })
        },
    );

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            group.bench_with_input(
                BenchmarkId::new("avx2_fma", BATCH_SIZE),
                &BATCH_SIZE,
                |bench, _| {
                    bench.iter(|| {
                        let mut best = f32::MAX;
                        for v in &vectors {
                            let d = unsafe {
                                x86_simd::cosine_distance_avx2(
                                    black_box(&query),
                                    black_box(v),
                                )
                            };
                            if d < best {
                                best = d;
                            }
                        }
                        best
                    })
                },
            );
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        group.bench_with_input(
            BenchmarkId::new("neon_fma", BATCH_SIZE),
            &BATCH_SIZE,
            |bench, _| {
                bench.iter(|| {
                    let mut best = f32::MAX;
                    for v in &vectors {
                        let d = unsafe {
                            neon_simd::cosine_distance_neon(
                                black_box(&query),
                                black_box(v),
                            )
                        };
                        if d < best {
                            best = d;
                        }
                    }
                    best
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_dot_product,
    bench_cosine_distance,
    bench_l2_distance,
    bench_batch_cosine,
);
criterion_main!(benches);
