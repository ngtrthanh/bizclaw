//! x86 AVX2 SIMD intrinsics for dot product.
//!
//! Available on Intel Haswell+ (2013), AMD Zen+ (2018).
//! Processes 8 floats per iteration (256-bit vectors).

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// AVX2-accelerated dot product (8 floats per iteration).
#[cfg(target_arch = "x86_64")]
pub fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let n = a.len();

    unsafe {
        let mut sum_vec = _mm256_setzero_ps();
        let chunks = n / 8;

        for i in 0..chunks {
            let offset = i * 8;
            let va = _mm256_loadu_ps(a.as_ptr().add(offset));
            let vb = _mm256_loadu_ps(b.as_ptr().add(offset));
            sum_vec = _mm256_fmadd_ps(va, vb, sum_vec); // fused multiply-add
        }

        // Horizontal sum of 8 lanes
        // [a, b, c, d | e, f, g, h]
        let hi128 = _mm256_extractf128_ps(sum_vec, 1); // [e, f, g, h]
        let lo128 = _mm256_castps256_ps128(sum_vec); // [a, b, c, d]
        let sum128 = _mm_add_ps(lo128, hi128); // [a+e, b+f, c+g, d+h]
        let hi64 = _mm_movehl_ps(sum128, sum128);
        let sum64 = _mm_add_ps(sum128, hi64);
        let hi32 = _mm_shuffle_ps(sum64, sum64, 1);
        let total = _mm_add_ss(sum64, hi32);
        let mut sum = _mm_cvtss_f32(total);

        // Tail
        for i in (chunks * 8)..n {
            sum += a[i] * b[i];
        }

        sum
    }
}

/// Scalar fallback.
#[cfg(not(target_arch = "x86_64"))]
pub fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {
    crate::tensor::dot_product(a, b)
}
