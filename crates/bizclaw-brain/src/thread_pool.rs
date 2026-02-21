//! Multi-threaded matrix multiply using rayon.

use rayon::prelude::*;

/// Parallel matrix-vector multiply: output = mat * vec.
/// mat is [rows x cols] in row-major order.
/// Splits rows across threads for parallel computation.
pub fn matmul_parallel(output: &mut [f32], mat: &[f32], vec_in: &[f32], rows: usize, cols: usize) {
    debug_assert_eq!(mat.len(), rows * cols);
    debug_assert_eq!(vec_in.len(), cols);
    debug_assert_eq!(output.len(), rows);

    output.par_iter_mut().enumerate().for_each(|(i, out)| {
        let row = &mat[i * cols..(i + 1) * cols];
        *out = crate::tensor::dot_product(row, vec_in);
    });
}

/// Get the number of available threads.
pub fn num_threads() -> usize {
    rayon::current_num_threads()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matmul_parallel() {
        let mat = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let vec_in = vec![1.0, 1.0, 1.0];
        let mut output = vec![0.0; 2];
        matmul_parallel(&mut output, &mat, &vec_in, 2, 3);
        assert!((output[0] - 6.0).abs() < 1e-6);
        assert!((output[1] - 15.0).abs() < 1e-6);
    }
}
