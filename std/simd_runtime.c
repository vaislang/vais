#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

// Platform detection
#if defined(__x86_64__) || defined(_M_X64)
  #include <immintrin.h>
  #define VAIS_X86_64 1
#elif defined(__aarch64__) || defined(_M_ARM64)
  #include <arm_neon.h>
  #define VAIS_AARCH64 1
#endif

// Memory alignment utilities
int64_t simd_alloc_aligned(int64_t size, int64_t alignment) {
    void* ptr = NULL;
#if defined(_WIN32)
    ptr = _aligned_malloc((size_t)size, (size_t)alignment);
#else
    if (posix_memalign(&ptr, (size_t)alignment, (size_t)size) != 0) {
        return 0;
    }
#endif
    if (ptr) {
        memset(ptr, 0, (size_t)size);
    }
    return (int64_t)ptr;
}

int64_t simd_free_aligned(int64_t ptr) {
    if (ptr == 0) {
        return 0;
    }
#if defined(_WIN32)
    _aligned_free((void*)ptr);
#else
    free((void*)ptr);
#endif
    return 1;
}

// Capability detection
int64_t simd_has_sse2(void) {
#ifdef VAIS_X86_64
    return 1;  // x86_64 always has SSE2
#else
    return 0;
#endif
}

int64_t simd_has_avx2(void) {
#ifdef VAIS_X86_64
    #ifdef __GNUC__
    return __builtin_cpu_supports("avx2") ? 1 : 0;
    #else
    return 0;
    #endif
#else
    return 0;
#endif
}

int64_t simd_has_neon(void) {
#ifdef VAIS_AARCH64
    return 1;  // AArch64 always has NEON
#else
    return 0;
#endif
}

// SSE2 float operations (128-bit, 4 x f32)
int64_t simd_add_f32x4(int64_t dst, int64_t a, int64_t b) {
    if (!dst || !a || !b) return 0;
#ifdef VAIS_X86_64
    __m128 va = _mm_loadu_ps((const float*)a);
    __m128 vb = _mm_loadu_ps((const float*)b);
    __m128 vr = _mm_add_ps(va, vb);
    _mm_storeu_ps((float*)dst, vr);
#elif defined(VAIS_AARCH64)
    float32x4_t va = vld1q_f32((const float*)a);
    float32x4_t vb = vld1q_f32((const float*)b);
    float32x4_t vr = vaddq_f32(va, vb);
    vst1q_f32((float*)dst, vr);
#else
    // Scalar fallback
    float* d = (float*)dst;
    const float* pa = (const float*)a;
    const float* pb = (const float*)b;
    for (int i = 0; i < 4; i++) {
        d[i] = pa[i] + pb[i];
    }
#endif
    return 1;
}

int64_t simd_sub_f32x4(int64_t dst, int64_t a, int64_t b) {
    if (!dst || !a || !b) return 0;
#ifdef VAIS_X86_64
    __m128 va = _mm_loadu_ps((const float*)a);
    __m128 vb = _mm_loadu_ps((const float*)b);
    __m128 vr = _mm_sub_ps(va, vb);
    _mm_storeu_ps((float*)dst, vr);
#elif defined(VAIS_AARCH64)
    float32x4_t va = vld1q_f32((const float*)a);
    float32x4_t vb = vld1q_f32((const float*)b);
    float32x4_t vr = vsubq_f32(va, vb);
    vst1q_f32((float*)dst, vr);
#else
    float* d = (float*)dst;
    const float* pa = (const float*)a;
    const float* pb = (const float*)b;
    for (int i = 0; i < 4; i++) {
        d[i] = pa[i] - pb[i];
    }
#endif
    return 1;
}

int64_t simd_mul_f32x4(int64_t dst, int64_t a, int64_t b) {
    if (!dst || !a || !b) return 0;
#ifdef VAIS_X86_64
    __m128 va = _mm_loadu_ps((const float*)a);
    __m128 vb = _mm_loadu_ps((const float*)b);
    __m128 vr = _mm_mul_ps(va, vb);
    _mm_storeu_ps((float*)dst, vr);
#elif defined(VAIS_AARCH64)
    float32x4_t va = vld1q_f32((const float*)a);
    float32x4_t vb = vld1q_f32((const float*)b);
    float32x4_t vr = vmulq_f32(va, vb);
    vst1q_f32((float*)dst, vr);
#else
    float* d = (float*)dst;
    const float* pa = (const float*)a;
    const float* pb = (const float*)b;
    for (int i = 0; i < 4; i++) {
        d[i] = pa[i] * pb[i];
    }
#endif
    return 1;
}

int64_t simd_div_f32x4(int64_t dst, int64_t a, int64_t b) {
    if (!dst || !a || !b) return 0;
#ifdef VAIS_X86_64
    __m128 va = _mm_loadu_ps((const float*)a);
    __m128 vb = _mm_loadu_ps((const float*)b);
    __m128 vr = _mm_div_ps(va, vb);
    _mm_storeu_ps((float*)dst, vr);
#elif defined(VAIS_AARCH64)
    float32x4_t va = vld1q_f32((const float*)a);
    float32x4_t vb = vld1q_f32((const float*)b);
    float32x4_t vr = vdivq_f32(va, vb);
    vst1q_f32((float*)dst, vr);
#else
    float* d = (float*)dst;
    const float* pa = (const float*)a;
    const float* pb = (const float*)b;
    for (int i = 0; i < 4; i++) {
        d[i] = pa[i] / pb[i];
    }
#endif
    return 1;
}

// SSE2 double operations (128-bit, 2 x f64)
int64_t simd_add_f64x2(int64_t dst, int64_t a, int64_t b) {
    if (!dst || !a || !b) return 0;
#ifdef VAIS_X86_64
    __m128d va = _mm_loadu_pd((const double*)a);
    __m128d vb = _mm_loadu_pd((const double*)b);
    __m128d vr = _mm_add_pd(va, vb);
    _mm_storeu_pd((double*)dst, vr);
#elif defined(VAIS_AARCH64)
    float64x2_t va = vld1q_f64((const double*)a);
    float64x2_t vb = vld1q_f64((const double*)b);
    float64x2_t vr = vaddq_f64(va, vb);
    vst1q_f64((double*)dst, vr);
#else
    double* d = (double*)dst;
    const double* pa = (const double*)a;
    const double* pb = (const double*)b;
    for (int i = 0; i < 2; i++) {
        d[i] = pa[i] + pb[i];
    }
#endif
    return 1;
}

int64_t simd_sub_f64x2(int64_t dst, int64_t a, int64_t b) {
    if (!dst || !a || !b) return 0;
#ifdef VAIS_X86_64
    __m128d va = _mm_loadu_pd((const double*)a);
    __m128d vb = _mm_loadu_pd((const double*)b);
    __m128d vr = _mm_sub_pd(va, vb);
    _mm_storeu_pd((double*)dst, vr);
#elif defined(VAIS_AARCH64)
    float64x2_t va = vld1q_f64((const double*)a);
    float64x2_t vb = vld1q_f64((const double*)b);
    float64x2_t vr = vsubq_f64(va, vb);
    vst1q_f64((double*)dst, vr);
#else
    double* d = (double*)dst;
    const double* pa = (const double*)a;
    const double* pb = (const double*)b;
    for (int i = 0; i < 2; i++) {
        d[i] = pa[i] - pb[i];
    }
#endif
    return 1;
}

int64_t simd_mul_f64x2(int64_t dst, int64_t a, int64_t b) {
    if (!dst || !a || !b) return 0;
#ifdef VAIS_X86_64
    __m128d va = _mm_loadu_pd((const double*)a);
    __m128d vb = _mm_loadu_pd((const double*)b);
    __m128d vr = _mm_mul_pd(va, vb);
    _mm_storeu_pd((double*)dst, vr);
#elif defined(VAIS_AARCH64)
    float64x2_t va = vld1q_f64((const double*)a);
    float64x2_t vb = vld1q_f64((const double*)b);
    float64x2_t vr = vmulq_f64(va, vb);
    vst1q_f64((double*)dst, vr);
#else
    double* d = (double*)dst;
    const double* pa = (const double*)a;
    const double* pb = (const double*)b;
    for (int i = 0; i < 2; i++) {
        d[i] = pa[i] * pb[i];
    }
#endif
    return 1;
}

int64_t simd_div_f64x2(int64_t dst, int64_t a, int64_t b) {
    if (!dst || !a || !b) return 0;
#ifdef VAIS_X86_64
    __m128d va = _mm_loadu_pd((const double*)a);
    __m128d vb = _mm_loadu_pd((const double*)b);
    __m128d vr = _mm_div_pd(va, vb);
    _mm_storeu_pd((double*)dst, vr);
#elif defined(VAIS_AARCH64)
    float64x2_t va = vld1q_f64((const double*)a);
    float64x2_t vb = vld1q_f64((const double*)b);
    float64x2_t vr = vdivq_f64(va, vb);
    vst1q_f64((double*)dst, vr);
#else
    double* d = (double*)dst;
    const double* pa = (const double*)a;
    const double* pb = (const double*)b;
    for (int i = 0; i < 2; i++) {
        d[i] = pa[i] / pb[i];
    }
#endif
    return 1;
}

// SSE2 int operations (128-bit, 4 x i32)
int64_t simd_add_i32x4(int64_t dst, int64_t a, int64_t b) {
    if (!dst || !a || !b) return 0;
#ifdef VAIS_X86_64
    __m128i va = _mm_loadu_si128((const __m128i*)a);
    __m128i vb = _mm_loadu_si128((const __m128i*)b);
    __m128i vr = _mm_add_epi32(va, vb);
    _mm_storeu_si128((__m128i*)dst, vr);
#elif defined(VAIS_AARCH64)
    int32x4_t va = vld1q_s32((const int32_t*)a);
    int32x4_t vb = vld1q_s32((const int32_t*)b);
    int32x4_t vr = vaddq_s32(va, vb);
    vst1q_s32((int32_t*)dst, vr);
#else
    int32_t* d = (int32_t*)dst;
    const int32_t* pa = (const int32_t*)a;
    const int32_t* pb = (const int32_t*)b;
    for (int i = 0; i < 4; i++) {
        d[i] = pa[i] + pb[i];
    }
#endif
    return 1;
}

int64_t simd_sub_i32x4(int64_t dst, int64_t a, int64_t b) {
    if (!dst || !a || !b) return 0;
#ifdef VAIS_X86_64
    __m128i va = _mm_loadu_si128((const __m128i*)a);
    __m128i vb = _mm_loadu_si128((const __m128i*)b);
    __m128i vr = _mm_sub_epi32(va, vb);
    _mm_storeu_si128((__m128i*)dst, vr);
#elif defined(VAIS_AARCH64)
    int32x4_t va = vld1q_s32((const int32_t*)a);
    int32x4_t vb = vld1q_s32((const int32_t*)b);
    int32x4_t vr = vsubq_s32(va, vb);
    vst1q_s32((int32_t*)dst, vr);
#else
    int32_t* d = (int32_t*)dst;
    const int32_t* pa = (const int32_t*)a;
    const int32_t* pb = (const int32_t*)b;
    for (int i = 0; i < 4; i++) {
        d[i] = pa[i] - pb[i];
    }
#endif
    return 1;
}

int64_t simd_mul_i32x4(int64_t dst, int64_t a, int64_t b) {
    if (!dst || !a || !b) return 0;
#ifdef VAIS_X86_64
    __m128i va = _mm_loadu_si128((const __m128i*)a);
    __m128i vb = _mm_loadu_si128((const __m128i*)b);
    __m128i vr = _mm_mullo_epi32(va, vb);
    _mm_storeu_si128((__m128i*)dst, vr);
#elif defined(VAIS_AARCH64)
    int32x4_t va = vld1q_s32((const int32_t*)a);
    int32x4_t vb = vld1q_s32((const int32_t*)b);
    int32x4_t vr = vmulq_s32(va, vb);
    vst1q_s32((int32_t*)dst, vr);
#else
    int32_t* d = (int32_t*)dst;
    const int32_t* pa = (const int32_t*)a;
    const int32_t* pb = (const int32_t*)b;
    for (int i = 0; i < 4; i++) {
        d[i] = pa[i] * pb[i];
    }
#endif
    return 1;
}

// Dot product (4 x f32)
int64_t simd_dot_f32x4(int64_t a, int64_t b) {
    if (!a || !b) return 0;
    float result;
#ifdef VAIS_X86_64
    __m128 va = _mm_loadu_ps((const float*)a);
    __m128 vb = _mm_loadu_ps((const float*)b);
    __m128 vr = _mm_mul_ps(va, vb);
    // Horizontal sum
    __m128 shuf = _mm_shuffle_ps(vr, vr, _MM_SHUFFLE(2, 3, 0, 1));
    __m128 sums = _mm_add_ps(vr, shuf);
    shuf = _mm_movehl_ps(shuf, sums);
    sums = _mm_add_ss(sums, shuf);
    _mm_store_ss(&result, sums);
#elif defined(VAIS_AARCH64)
    float32x4_t va = vld1q_f32((const float*)a);
    float32x4_t vb = vld1q_f32((const float*)b);
    float32x4_t vr = vmulq_f32(va, vb);
    result = vaddvq_f32(vr);
#else
    const float* pa = (const float*)a;
    const float* pb = (const float*)b;
    result = 0.0f;
    for (int i = 0; i < 4; i++) {
        result += pa[i] * pb[i];
    }
#endif
    // Convert float to int64_t representation
    union { float f; int32_t i; } u;
    u.f = result;
    return (int64_t)u.i;
}

// Horizontal sum (4 x f32)
int64_t simd_hsum_f32x4(int64_t a) {
    if (!a) return 0;
    float result;
#ifdef VAIS_X86_64
    __m128 va = _mm_loadu_ps((const float*)a);
    __m128 shuf = _mm_shuffle_ps(va, va, _MM_SHUFFLE(2, 3, 0, 1));
    __m128 sums = _mm_add_ps(va, shuf);
    shuf = _mm_movehl_ps(shuf, sums);
    sums = _mm_add_ss(sums, shuf);
    _mm_store_ss(&result, sums);
#elif defined(VAIS_AARCH64)
    float32x4_t va = vld1q_f32((const float*)a);
    result = vaddvq_f32(va);
#else
    const float* pa = (const float*)a;
    result = pa[0] + pa[1] + pa[2] + pa[3];
#endif
    union { float f; int32_t i; } u;
    u.f = result;
    return (int64_t)u.i;
}

// Horizontal sum (2 x f64)
int64_t simd_hsum_f64x2(int64_t a) {
    if (!a) return 0;
    double result;
#ifdef VAIS_X86_64
    __m128d va = _mm_loadu_pd((const double*)a);
    __m128d shuf = _mm_shuffle_pd(va, va, 1);
    __m128d sums = _mm_add_pd(va, shuf);
    _mm_store_sd(&result, sums);
#elif defined(VAIS_AARCH64)
    float64x2_t va = vld1q_f64((const double*)a);
    result = vaddvq_f64(va);
#else
    const double* pa = (const double*)a;
    result = pa[0] + pa[1];
#endif
    union { double d; int64_t i; } u;
    u.d = result;
    return u.i;
}

// Horizontal sum (4 x i32)
int64_t simd_hsum_i32x4(int64_t a) {
    if (!a) return 0;
    int32_t result;
#ifdef VAIS_X86_64
    __m128i va = _mm_loadu_si128((const __m128i*)a);
    __m128i shuf = _mm_shuffle_epi32(va, _MM_SHUFFLE(2, 3, 0, 1));
    __m128i sums = _mm_add_epi32(va, shuf);
    shuf = _mm_shuffle_epi32(sums, _MM_SHUFFLE(1, 0, 3, 2));
    sums = _mm_add_epi32(sums, shuf);
    result = _mm_cvtsi128_si32(sums);
#elif defined(VAIS_AARCH64)
    int32x4_t va = vld1q_s32((const int32_t*)a);
    result = vaddvq_s32(va);
#else
    const int32_t* pa = (const int32_t*)a;
    result = pa[0] + pa[1] + pa[2] + pa[3];
#endif
    return (int64_t)result;
}

// Euclidean distance (L2 norm) - f32
int64_t simd_distance_f32(int64_t a, int64_t b, int64_t n) {
    if (!a || !b || n <= 0) return 0;
    const float* pa = (const float*)a;
    const float* pb = (const float*)b;
    float sum_sq = 0.0f;

    int64_t i = 0;
#ifdef VAIS_X86_64
    __m128 vsum = _mm_setzero_ps();
    for (; i + 4 <= n; i += 4) {
        __m128 va = _mm_loadu_ps(&pa[i]);
        __m128 vb = _mm_loadu_ps(&pb[i]);
        __m128 vd = _mm_sub_ps(va, vb);
        vsum = _mm_add_ps(vsum, _mm_mul_ps(vd, vd));
    }
    // Horizontal sum
    __m128 shuf = _mm_shuffle_ps(vsum, vsum, _MM_SHUFFLE(2, 3, 0, 1));
    __m128 sums = _mm_add_ps(vsum, shuf);
    shuf = _mm_movehl_ps(shuf, sums);
    sums = _mm_add_ss(sums, shuf);
    _mm_store_ss(&sum_sq, sums);
#elif defined(VAIS_AARCH64)
    float32x4_t vsum = vdupq_n_f32(0.0f);
    for (; i + 4 <= n; i += 4) {
        float32x4_t va = vld1q_f32(&pa[i]);
        float32x4_t vb = vld1q_f32(&pb[i]);
        float32x4_t vd = vsubq_f32(va, vb);
        vsum = vmlaq_f32(vsum, vd, vd);
    }
    sum_sq = vaddvq_f32(vsum);
#endif

    // Process remaining elements
    for (; i < n; i++) {
        float d = pa[i] - pb[i];
        sum_sq += d * d;
    }

    float result = sqrtf(sum_sq);
    union { float f; int32_t i; } u;
    u.f = result;
    return (int64_t)u.i;
}

// Euclidean distance (L2 norm) - f64
int64_t simd_distance_f64(int64_t a, int64_t b, int64_t n) {
    if (!a || !b || n <= 0) return 0;
    const double* pa = (const double*)a;
    const double* pb = (const double*)b;
    double sum_sq = 0.0;

    int64_t i = 0;
#ifdef VAIS_X86_64
    __m128d vsum = _mm_setzero_pd();
    for (; i + 2 <= n; i += 2) {
        __m128d va = _mm_loadu_pd(&pa[i]);
        __m128d vb = _mm_loadu_pd(&pb[i]);
        __m128d vd = _mm_sub_pd(va, vb);
        vsum = _mm_add_pd(vsum, _mm_mul_pd(vd, vd));
    }
    // Horizontal sum
    __m128d shuf = _mm_shuffle_pd(vsum, vsum, 1);
    __m128d sums = _mm_add_pd(vsum, shuf);
    _mm_store_sd(&sum_sq, sums);
#elif defined(VAIS_AARCH64)
    float64x2_t vsum = vdupq_n_f64(0.0);
    for (; i + 2 <= n; i += 2) {
        float64x2_t va = vld1q_f64(&pa[i]);
        float64x2_t vb = vld1q_f64(&pb[i]);
        float64x2_t vd = vsubq_f64(va, vb);
        vsum = vmlaq_f64(vsum, vd, vd);
    }
    sum_sq = vaddvq_f64(vsum);
#endif

    // Process remaining elements
    for (; i < n; i++) {
        double d = pa[i] - pb[i];
        sum_sq += d * d;
    }

    double result = sqrt(sum_sq);
    union { double d; int64_t i; } u;
    u.d = result;
    return u.i;
}
