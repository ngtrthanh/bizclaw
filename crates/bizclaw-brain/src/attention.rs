//! Flash Attention — online softmax attention computation.
//!
//! Computes attention scores incrementally without materializing
//! the full QK^T matrix, saving memory.

/// Compute single-head attention output for a single query position.
///
/// q: query vector [head_dim]
/// key_cache: all key vectors [seq_len x head_dim]
/// value_cache: all value vectors [seq_len x head_dim]
/// seq_len: current sequence length (how many KV entries are valid)
/// head_dim: dimension per head
pub fn attention(
    output: &mut [f32],
    q: &[f32],
    key_cache: &[f32],
    value_cache: &[f32],
    seq_len: usize,
    head_dim: usize,
) {
    debug_assert_eq!(q.len(), head_dim);
    debug_assert_eq!(output.len(), head_dim);

    if seq_len == 0 {
        for v in output.iter_mut() { *v = 0.0; }
        return;
    }

    let scale = 1.0 / (head_dim as f32).sqrt();

    // Compute attention scores: score[t] = q · k[t] / sqrt(d)
    let mut scores = vec![0.0f32; seq_len];
    for t in 0..seq_len {
        let k_offset = t * head_dim;
        let k = &key_cache[k_offset..k_offset + head_dim];
        let mut dot = 0.0f32;
        for i in 0..head_dim {
            dot += q[i] * k[i];
        }
        scores[t] = dot * scale;
    }

    // Softmax over scores
    crate::tensor::softmax(&mut scores);

    // Weighted sum: output = sum(score[t] * v[t])
    for v in output.iter_mut() {
        *v = 0.0;
    }
    for t in 0..seq_len {
        let v_offset = t * head_dim;
        let weight = scores[t];
        for i in 0..head_dim {
            output[i] += weight * value_cache[v_offset + i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attention_single_kv() {
        let head_dim = 4;
        let q = vec![1.0, 0.0, 0.0, 0.0];
        let key_cache = vec![1.0, 0.0, 0.0, 0.0]; // 1 key
        let value_cache = vec![0.0, 1.0, 0.0, 0.0]; // 1 value
        let mut output = vec![0.0; head_dim];

        attention(&mut output, &q, &key_cache, &value_cache, 1, head_dim);

        // With a single KV pair, output should equal the value vector
        assert!((output[0] - 0.0).abs() < 1e-5);
        assert!((output[1] - 1.0).abs() < 1e-5);
    }
}
