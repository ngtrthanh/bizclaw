//! Rotary Position Embeddings (RoPE).
//!
//! Applied to query and key vectors to encode position information.

/// Apply RoPE to a vector in-place.
/// `pos` is the token position, `dim` is the embedding dimension,
/// `head_dim` is the dimension per attention head.
pub fn apply_rope(vec: &mut [f32], pos: usize, head_dim: usize, rope_theta: f32) {
    let half_dim = head_dim / 2;
    for i in 0..half_dim {
        let freq = 1.0 / rope_theta.powf(2.0 * i as f32 / head_dim as f32);
        let angle = pos as f32 * freq;
        let cos = angle.cos();
        let sin = angle.sin();

        let x0 = vec[i];
        let x1 = vec[i + half_dim];
        vec[i] = x0 * cos - x1 * sin;
        vec[i + half_dim] = x0 * sin + x1 * cos;
    }
}

/// Apply RoPE to all heads in a layer.
pub fn apply_rope_multi_head(
    vec: &mut [f32],
    pos: usize,
    n_heads: usize,
    head_dim: usize,
    rope_theta: f32,
) {
    for h in 0..n_heads {
        let start = h * head_dim;
        let end = start + head_dim;
        apply_rope(&mut vec[start..end], pos, head_dim, rope_theta);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rope_position_0() {
        // At position 0, RoPE should be identity (cos(0)=1, sin(0)=0)
        let mut vec = vec![1.0, 2.0, 3.0, 4.0];
        let original = vec.clone();
        apply_rope(&mut vec, 0, 4, 10000.0);
        for (a, b) in vec.iter().zip(original.iter()) {
            assert!((a - b).abs() < 1e-5);
        }
    }
}
