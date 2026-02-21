//! Tensor operations — matmul, rmsnorm, softmax, silu.
//!
//! Pure Rust implementations with future SIMD acceleration.

/// RMS normalization (Root Mean Square Layer Normalization).
/// Used in LLaMA instead of LayerNorm.
pub fn rmsnorm(output: &mut [f32], input: &[f32], weight: &[f32], eps: f32) {
    let n = input.len();
    debug_assert_eq!(output.len(), n);
    debug_assert_eq!(weight.len(), n);

    // Compute RMS
    let ss: f32 = input.iter().map(|&x| x * x).sum::<f32>() / n as f32;
    let rms = (ss + eps).sqrt();
    let inv_rms = 1.0 / rms;

    // Normalize and scale
    for i in 0..n {
        output[i] = input[i] * inv_rms * weight[i];
    }
}

/// Matrix-vector multiply: output = mat * vec.
/// mat is [rows x cols] in row-major order.
pub fn matmul(output: &mut [f32], mat: &[f32], vec: &[f32], rows: usize, cols: usize) {
    debug_assert_eq!(mat.len(), rows * cols);
    debug_assert_eq!(vec.len(), cols);
    debug_assert_eq!(output.len(), rows);

    for i in 0..rows {
        let row = &mat[i * cols..(i + 1) * cols];
        output[i] = dot_product(row, vec);
    }
}

/// Dot product of two vectors.
#[inline]
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum()
}

/// Softmax — converts logits to probabilities.
pub fn softmax(values: &mut [f32]) {
    if values.is_empty() {
        return;
    }

    let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0f32;
    for v in values.iter_mut() {
        *v = (*v - max).exp();
        sum += *v;
    }
    let inv_sum = 1.0 / sum;
    for v in values.iter_mut() {
        *v *= inv_sum;
    }
}

/// SiLU (Swish) activation: silu(x) = x * sigmoid(x) = x / (1 + exp(-x))
pub fn silu(values: &mut [f32]) {
    for v in values.iter_mut() {
        *v = *v / (1.0 + (-*v).exp());
    }
}

/// Element-wise multiply: a[i] *= b[i]
pub fn elementwise_mul(a: &mut [f32], b: &[f32]) {
    debug_assert_eq!(a.len(), b.len());
    for (x, &y) in a.iter_mut().zip(b.iter()) {
        *x *= y;
    }
}

/// Element-wise add: a[i] += b[i]
pub fn elementwise_add(a: &mut [f32], b: &[f32]) {
    debug_assert_eq!(a.len(), b.len());
    for (x, &y) in a.iter_mut().zip(b.iter()) {
        *x += y;
    }
}

/// Copy values from src to dst.
pub fn copy(dst: &mut [f32], src: &[f32]) {
    debug_assert_eq!(dst.len(), src.len());
    dst.copy_from_slice(src);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_softmax() {
        let mut v = vec![1.0, 2.0, 3.0];
        softmax(&mut v);
        let sum: f32 = v.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
        assert!(v[2] > v[1] && v[1] > v[0]);
    }

    #[test]
    fn test_rmsnorm() {
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let weight = vec![1.0, 1.0, 1.0, 1.0];
        let mut output = vec![0.0; 4];
        rmsnorm(&mut output, &input, &weight, 1e-6);
        // Output should be normalized
        assert!(output.iter().all(|&v| v.is_finite()));
    }

    #[test]
    fn test_matmul() {
        // 2x3 matrix * 3-vector
        let mat = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let vec_in = vec![1.0, 1.0, 1.0];
        let mut output = vec![0.0; 2];
        matmul(&mut output, &mat, &vec_in, 2, 3);
        assert!((output[0] - 6.0).abs() < 1e-6); // 1+2+3
        assert!((output[1] - 15.0).abs() < 1e-6); // 4+5+6
    }

    #[test]
    fn test_silu() {
        let mut v = vec![0.0, 1.0, -1.0];
        silu(&mut v);
        assert!((v[0] - 0.0).abs() < 1e-6);
        assert!(v[1] > 0.0);
        assert!(v[2] < 0.0);
    }
}
