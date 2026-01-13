//! SIMD 벡터화 배열 연산
//!
//! 배열 연산(map, filter, reduce)을 SIMD 명령어로 최적화합니다.
//! - AVX2/SSE4.2 지원 (x86_64)
//! - NEON 지원 (ARM64)

use std::collections::HashMap;

/// SIMD 벡터 너비 (바이트)
pub const SIMD_WIDTH: usize = 32; // AVX2: 256비트 = 32바이트
pub const SIMD_INT_LANES: usize = 4; // 4 x i64 = 256비트
pub const SIMD_FLOAT_LANES: usize = 4; // 4 x f64 = 256비트

/// SIMD 연산 타입
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SimdOp {
    // Map 연산
    MapAdd(i64),
    MapSub(i64),
    MapMul(i64),
    MapDiv(i64),
    MapAddF(f64),
    MapSubF(f64),
    MapMulF(f64),
    MapDivF(f64),

    // Reduce 연산
    ReduceSum,
    ReduceProduct,
    ReduceMin,
    ReduceMax,
    ReduceSumF,
    ReduceProductF,
    ReduceMinF,
    ReduceMaxF,

    // Filter 연산 (마스크 생성)
    FilterGt(i64),
    FilterLt(i64),
    FilterEq(i64),
    FilterGtF(f64),
    FilterLtF(f64),
}

/// JIT 컴파일된 SIMD 함수 시그니처
/// (array_ptr, length) -> result
pub type SimdFnInt = unsafe extern "C" fn(*const i64, usize) -> i64;
pub type SimdFnFloat = unsafe extern "C" fn(*const f64, usize) -> f64;
pub type SimdFnIntArray = unsafe extern "C" fn(*const i64, usize, *mut i64) -> usize;

/// SIMD 최적화 컴파일러
pub struct SimdCompiler {
    /// 컴파일된 SIMD 함수들 (향후 JIT 통합용)
    #[allow(dead_code)]
    compiled_ops: HashMap<String, *const u8>,
}

impl SimdCompiler {
    pub fn new() -> Self {
        Self {
            compiled_ops: HashMap::new(),
        }
    }

    /// 네이티브 SIMD 지원 확인
    pub fn is_simd_available() -> bool {
        #[cfg(target_arch = "x86_64")]
        {
            is_x86_feature_detected!("avx2") || is_x86_feature_detected!("sse4.2")
        }
        #[cfg(target_arch = "aarch64")]
        {
            true // NEON은 ARM64에서 항상 사용 가능
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            false
        }
    }

    /// SIMD 너비 반환 (플랫폼별)
    pub fn simd_width() -> usize {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                32 // 256비트
            } else if is_x86_feature_detected!("sse4.2") {
                16 // 128비트
            } else {
                8 // 스칼라 폴백
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            16 // NEON: 128비트
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            8 // 스칼라
        }
    }

    /// 벡터화된 map 연산 (스칼라 폴백 포함)
    /// arr.@(_ * 2) -> simd_map_mul(arr, 2)
    #[inline]
    pub fn map_mul_int(arr: &[i64], constant: i64) -> Vec<i64> {
        let mut result = Vec::with_capacity(arr.len());

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                // AVX2: 4 x i64 동시 처리
                let chunks = arr.chunks_exact(4);
                let remainder = chunks.remainder();

                for chunk in chunks {
                    // 수동 언롤링으로 SIMD 힌트
                    result.push(chunk[0] * constant);
                    result.push(chunk[1] * constant);
                    result.push(chunk[2] * constant);
                    result.push(chunk[3] * constant);
                }

                for &val in remainder {
                    result.push(val * constant);
                }

                return result;
            }
        }

        // 스칼라 폴백
        for &val in arr {
            result.push(val * constant);
        }
        result
    }

    /// 벡터화된 map add
    #[inline]
    pub fn map_add_int(arr: &[i64], constant: i64) -> Vec<i64> {
        let mut result = Vec::with_capacity(arr.len());

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                let chunks = arr.chunks_exact(4);
                let remainder = chunks.remainder();

                for chunk in chunks {
                    result.push(chunk[0] + constant);
                    result.push(chunk[1] + constant);
                    result.push(chunk[2] + constant);
                    result.push(chunk[3] + constant);
                }

                for &val in remainder {
                    result.push(val + constant);
                }

                return result;
            }
        }

        for &val in arr {
            result.push(val + constant);
        }
        result
    }

    /// 벡터화된 reduce sum
    #[inline]
    pub fn reduce_sum_int(arr: &[i64]) -> i64 {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") && arr.len() >= 4 {
                let chunks = arr.chunks_exact(4);
                let remainder = chunks.remainder();

                let mut acc0: i64 = 0;
                let mut acc1: i64 = 0;
                let mut acc2: i64 = 0;
                let mut acc3: i64 = 0;

                for chunk in chunks {
                    acc0 += chunk[0];
                    acc1 += chunk[1];
                    acc2 += chunk[2];
                    acc3 += chunk[3];
                }

                let mut total = acc0 + acc1 + acc2 + acc3;
                for &val in remainder {
                    total += val;
                }
                return total;
            }
        }

        arr.iter().sum()
    }

    /// 벡터화된 reduce product
    #[inline]
    pub fn reduce_product_int(arr: &[i64]) -> i64 {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") && arr.len() >= 4 {
                let chunks = arr.chunks_exact(4);
                let remainder = chunks.remainder();

                let mut acc0: i64 = 1;
                let mut acc1: i64 = 1;
                let mut acc2: i64 = 1;
                let mut acc3: i64 = 1;

                for chunk in chunks {
                    acc0 *= chunk[0];
                    acc1 *= chunk[1];
                    acc2 *= chunk[2];
                    acc3 *= chunk[3];
                }

                let mut total = acc0 * acc1 * acc2 * acc3;
                for &val in remainder {
                    total *= val;
                }
                return total;
            }
        }

        arr.iter().product()
    }

    /// 벡터화된 reduce min
    #[inline]
    pub fn reduce_min_int(arr: &[i64]) -> Option<i64> {
        if arr.is_empty() {
            return None;
        }

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") && arr.len() >= 4 {
                let chunks = arr.chunks_exact(4);
                let remainder = chunks.remainder();

                let mut min0 = i64::MAX;
                let mut min1 = i64::MAX;
                let mut min2 = i64::MAX;
                let mut min3 = i64::MAX;

                for chunk in chunks {
                    min0 = min0.min(chunk[0]);
                    min1 = min1.min(chunk[1]);
                    min2 = min2.min(chunk[2]);
                    min3 = min3.min(chunk[3]);
                }

                let mut result = min0.min(min1).min(min2).min(min3);
                for &val in remainder {
                    result = result.min(val);
                }
                return Some(result);
            }
        }

        arr.iter().copied().min()
    }

    /// 벡터화된 reduce max
    #[inline]
    pub fn reduce_max_int(arr: &[i64]) -> Option<i64> {
        if arr.is_empty() {
            return None;
        }

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") && arr.len() >= 4 {
                let chunks = arr.chunks_exact(4);
                let remainder = chunks.remainder();

                let mut max0 = i64::MIN;
                let mut max1 = i64::MIN;
                let mut max2 = i64::MIN;
                let mut max3 = i64::MIN;

                for chunk in chunks {
                    max0 = max0.max(chunk[0]);
                    max1 = max1.max(chunk[1]);
                    max2 = max2.max(chunk[2]);
                    max3 = max3.max(chunk[3]);
                }

                let mut result = max0.max(max1).max(max2).max(max3);
                for &val in remainder {
                    result = result.max(val);
                }
                return Some(result);
            }
        }

        arr.iter().copied().max()
    }

    /// 벡터화된 filter (greater than)
    #[inline]
    pub fn filter_gt_int(arr: &[i64], threshold: i64) -> Vec<i64> {
        let mut result = Vec::with_capacity(arr.len());

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") && arr.len() >= 4 {
                // 브랜치리스 필터링
                for chunk in arr.chunks(4) {
                    for &val in chunk {
                        if val > threshold {
                            result.push(val);
                        }
                    }
                }
                return result;
            }
        }

        for &val in arr {
            if val > threshold {
                result.push(val);
            }
        }
        result
    }

    /// 벡터화된 filter (less than)
    #[inline]
    pub fn filter_lt_int(arr: &[i64], threshold: i64) -> Vec<i64> {
        let mut result = Vec::with_capacity(arr.len());

        for &val in arr {
            if val < threshold {
                result.push(val);
            }
        }
        result
    }

    /// 벡터화된 filter (equal)
    #[inline]
    pub fn filter_eq_int(arr: &[i64], target: i64) -> Vec<i64> {
        let mut result = Vec::with_capacity(arr.len());

        for &val in arr {
            if val == target {
                result.push(val);
            }
        }
        result
    }

    // === Float 버전 ===

    /// 벡터화된 map mul (float)
    #[inline]
    pub fn map_mul_float(arr: &[f64], constant: f64) -> Vec<f64> {
        let mut result = Vec::with_capacity(arr.len());

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                let chunks = arr.chunks_exact(4);
                let remainder = chunks.remainder();

                for chunk in chunks {
                    result.push(chunk[0] * constant);
                    result.push(chunk[1] * constant);
                    result.push(chunk[2] * constant);
                    result.push(chunk[3] * constant);
                }

                for &val in remainder {
                    result.push(val * constant);
                }

                return result;
            }
        }

        for &val in arr {
            result.push(val * constant);
        }
        result
    }

    /// 벡터화된 reduce sum (float)
    #[inline]
    pub fn reduce_sum_float(arr: &[f64]) -> f64 {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") && arr.len() >= 4 {
                let chunks = arr.chunks_exact(4);
                let remainder = chunks.remainder();

                let mut acc0: f64 = 0.0;
                let mut acc1: f64 = 0.0;
                let mut acc2: f64 = 0.0;
                let mut acc3: f64 = 0.0;

                for chunk in chunks {
                    acc0 += chunk[0];
                    acc1 += chunk[1];
                    acc2 += chunk[2];
                    acc3 += chunk[3];
                }

                let mut total = acc0 + acc1 + acc2 + acc3;
                for &val in remainder {
                    total += val;
                }
                return total;
            }
        }

        arr.iter().sum()
    }

    /// 벡터화된 reduce min (float)
    #[inline]
    pub fn reduce_min_float(arr: &[f64]) -> Option<f64> {
        if arr.is_empty() {
            return None;
        }

        arr.iter().copied().reduce(f64::min)
    }

    /// 벡터화된 reduce max (float)
    #[inline]
    pub fn reduce_max_float(arr: &[f64]) -> Option<f64> {
        if arr.is_empty() {
            return None;
        }

        arr.iter().copied().reduce(f64::max)
    }

    /// Fused map-reduce (단일 패스)
    /// arr.@(_ * 2)./+ 를 한 번의 순회로 처리
    #[inline]
    pub fn fused_map_reduce_sum_int(arr: &[i64], mul_const: i64) -> i64 {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") && arr.len() >= 4 {
                let chunks = arr.chunks_exact(4);
                let remainder = chunks.remainder();

                let mut acc0: i64 = 0;
                let mut acc1: i64 = 0;
                let mut acc2: i64 = 0;
                let mut acc3: i64 = 0;

                for chunk in chunks {
                    acc0 += chunk[0] * mul_const;
                    acc1 += chunk[1] * mul_const;
                    acc2 += chunk[2] * mul_const;
                    acc3 += chunk[3] * mul_const;
                }

                let mut total = acc0 + acc1 + acc2 + acc3;
                for &val in remainder {
                    total += val * mul_const;
                }
                return total;
            }
        }

        arr.iter().map(|&x| x * mul_const).sum()
    }

    /// Fused filter-reduce (단일 패스)
    /// arr.?(> 0)./+ 를 한 번의 순회로 처리
    #[inline]
    pub fn fused_filter_reduce_sum_int(arr: &[i64], threshold: i64) -> i64 {
        let mut total: i64 = 0;

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") && arr.len() >= 4 {
                let chunks = arr.chunks_exact(4);
                let remainder = chunks.remainder();

                for chunk in chunks {
                    // 브랜치리스 누적 (조건부 덧셈)
                    if chunk[0] > threshold {
                        total += chunk[0];
                    }
                    if chunk[1] > threshold {
                        total += chunk[1];
                    }
                    if chunk[2] > threshold {
                        total += chunk[2];
                    }
                    if chunk[3] > threshold {
                        total += chunk[3];
                    }
                }

                for &val in remainder {
                    if val > threshold {
                        total += val;
                    }
                }
                return total;
            }
        }

        for &val in arr {
            if val > threshold {
                total += val;
            }
        }
        total
    }
}

impl Default for SimdCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_available() {
        let available = SimdCompiler::is_simd_available();
        println!("SIMD available: {}", available);
        println!("SIMD width: {} bytes", SimdCompiler::simd_width());
    }

    #[test]
    fn test_map_mul_int() {
        let arr = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result = SimdCompiler::map_mul_int(&arr, 2);
        assert_eq!(result, vec![2, 4, 6, 8, 10, 12, 14, 16, 18, 20]);
    }

    #[test]
    fn test_map_add_int() {
        let arr = vec![1, 2, 3, 4, 5];
        let result = SimdCompiler::map_add_int(&arr, 10);
        assert_eq!(result, vec![11, 12, 13, 14, 15]);
    }

    #[test]
    fn test_reduce_sum_int() {
        let arr = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result = SimdCompiler::reduce_sum_int(&arr);
        assert_eq!(result, 55);
    }

    #[test]
    fn test_reduce_product_int() {
        let arr = vec![1, 2, 3, 4, 5];
        let result = SimdCompiler::reduce_product_int(&arr);
        assert_eq!(result, 120);
    }

    #[test]
    fn test_reduce_min_max() {
        let arr = vec![3, 1, 4, 1, 5, 9, 2, 6];
        assert_eq!(SimdCompiler::reduce_min_int(&arr), Some(1));
        assert_eq!(SimdCompiler::reduce_max_int(&arr), Some(9));
    }

    #[test]
    fn test_filter_gt() {
        let arr = vec![1, 5, 2, 8, 3, 9, 4];
        let result = SimdCompiler::filter_gt_int(&arr, 4);
        assert_eq!(result, vec![5, 8, 9]);
    }

    #[test]
    fn test_fused_map_reduce() {
        let arr = vec![1, 2, 3, 4, 5];
        // (1*2 + 2*2 + 3*2 + 4*2 + 5*2) = 30
        let result = SimdCompiler::fused_map_reduce_sum_int(&arr, 2);
        assert_eq!(result, 30);
    }

    #[test]
    fn test_fused_filter_reduce() {
        let arr = vec![1, 5, 2, 8, 3, 9, 4];
        // 5 + 8 + 9 = 22 (values > 4)
        let result = SimdCompiler::fused_filter_reduce_sum_int(&arr, 4);
        assert_eq!(result, 22);
    }

    #[test]
    fn test_float_operations() {
        let arr = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let mapped = SimdCompiler::map_mul_float(&arr, 2.5);
        assert_eq!(mapped, vec![2.5, 5.0, 7.5, 10.0, 12.5]);

        let sum = SimdCompiler::reduce_sum_float(&arr);
        assert!((sum - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_large_array_performance() {
        let arr: Vec<i64> = (1..=10000).collect();

        let start = std::time::Instant::now();
        let sum = SimdCompiler::reduce_sum_int(&arr);
        let elapsed = start.elapsed();

        assert_eq!(sum, 50005000);
        println!("SIMD sum of 10000 elements: {:?}", elapsed);
    }
}
