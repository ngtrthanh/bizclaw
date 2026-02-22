//! x86 SSE2 SIMD intrinsics for dot product.
//!
//! Available on all x86_64 CPUs (SSE2 is baseline for x86_64).

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// SSE2-accelerated dot product (4 floats per iteration).
#[cfg(target_arch = "x86_64")]
pub fn dot_product_sse2(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let n = a.len();

    unsafe {
        let mut sum_vec = _mm_setzero_ps();
        let chunks = n / 4;

        for i in 0..chunks {
            let offset = i * 4;
            let va = _mm_loadu_ps(a.as_ptr().add(offset));
            let vb = _mm_loadu_ps(b.as_ptr().add(offset));
            let prod = _mm_mul_ps(va, vb);
            sum_vec = _mm_add_ps(sum_vec, prod);
        }

        // Horizontal sum: [a, b, c, d] → a+b+c+d
        let hi = _mm_movehl_ps(sum_vec, sum_vec); // [c, d, c, d]
        let sum2 = _mm_add_ps(sum_vec, hi); // [a+c, b+d, ...]
        let hi2 = _mm_shuffle_ps(sum2, sum2, 1); // [b+d, ...]
        let sum_scalar = _mm_add_ss(sum2, hi2); // [a+b+c+d]
        let mut sum = _mm_cvtss_f32(sum_scalar);

        // Tail
        for i in (chunks * 4)..n {
            sum += a[i] * b[i];
        }

        sum
    }
}

/// Scalar fallback.
#[cfg(not(target_arch = "x86_64"))]
pub fn dot_product_sse2(a: &[f32], b: &[f32]) -> f32 {
    crate::tensor::dot_product(a, b)
}
