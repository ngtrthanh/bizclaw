//! ARM NEON SIMD intrinsics for aarch64.
//!
//! Accelerates dot product, matmul, and other tensor ops
//! on ARM64 processors (Apple Silicon, Raspberry Pi 4/5).

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

/// NEON-accelerated dot product (4 floats per iteration).
#[cfg(target_arch = "aarch64")]
pub fn dot_product_neon(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let n = a.len();

    unsafe {
        let mut sum_vec = vdupq_n_f32(0.0);
        let chunks = n / 4;

        for i in 0..chunks {
            let offset = i * 4;
            let va = vld1q_f32(a.as_ptr().add(offset));
            let vb = vld1q_f32(b.as_ptr().add(offset));
            sum_vec = vfmaq_f32(sum_vec, va, vb); // fused multiply-add
        }

        // Horizontal sum of 4 lanes
        let mut sum = vaddvq_f32(sum_vec);

        // Handle remaining elements
        for i in (chunks * 4)..n {
            sum += a[i] * b[i];
        }

        sum
    }
}

/// Scalar fallback for non-aarch64.
#[cfg(not(target_arch = "aarch64"))]
pub fn dot_product_neon(a: &[f32], b: &[f32]) -> f32 {
    crate::tensor::dot_product(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neon_dot_product() {
        let a: Vec<f32> = (1..=16).map(|x| x as f32).collect();
        let b: Vec<f32> = vec![1.0; 16];
        let result = dot_product_neon(&a, &b);
        let expected: f32 = (1..=16).map(|x| x as f32).sum();
        assert!(
            (result - expected).abs() < 1e-3,
            "got {result}, expected {expected}"
        );
    }

    #[test]
    fn test_neon_dot_product_odd_length() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0]; // 5 elements (not multiple of 4)
        let b = vec![1.0; 5];
        let result = dot_product_neon(&a, &b);
        assert!((result - 15.0).abs() < 1e-3);
    }
}
