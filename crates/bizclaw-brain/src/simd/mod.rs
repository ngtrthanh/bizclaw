//! SIMD acceleration module.
//!
//! Platform-specific SIMD intrinsics for accelerated math operations.
//! Falls back to scalar Rust code when SIMD is unavailable.

pub mod neon;
pub mod sse2;
pub mod avx2;

/// Accelerated dot product — dispatches to SIMD when available.
pub fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());

    // For now, use scalar implementation
    // TODO: Add #[cfg(target_arch)] dispatch to NEON/SSE2/AVX2
    crate::tensor::dot_product(a, b)
}
