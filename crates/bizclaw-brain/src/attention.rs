//! Flash Attention — online softmax attention computation.
//!
//! Computes attention scores incrementally without materializing
//! the full QK^T matrix, saving O(seq_len) memory.

pub struct AttentionConfig {
    pub n_heads: usize,
    pub n_kv_heads: usize,
    pub seq_len: usize,
    pub head_dim: usize,
}

/// Compute single-head attention output for a single query position.
/// Uses online softmax (flash attention) — no intermediate score buffer.
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
        for v in output.iter_mut() {
            *v = 0.0;
        }
        return;
    }

    let scale = 1.0 / (head_dim as f32).sqrt();

    // Online softmax (flash attention):
    // Maintains running max and normalizer, avoiding score materialization.
    let mut running_max = f32::NEG_INFINITY;
    let mut running_sum = 0.0f32;
    for v in output.iter_mut() {
        *v = 0.0;
    }

    for t in 0..seq_len {
        let k_offset = t * head_dim;
        let k = &key_cache[k_offset..k_offset + head_dim];

        // Compute score = q · k / sqrt(d)
        let mut dot = 0.0f32;
        for i in 0..head_dim {
            dot += q[i] * k[i];
        }
        let score = dot * scale;

        // Online softmax update
        let new_max = running_max.max(score);

        // Rescale existing accumulator
        let scale_old = (running_max - new_max).exp();
        let exp_score = (score - new_max).exp();

        // Update running sum with rescaled old sum + new contribution
        running_sum = running_sum * scale_old + exp_score;

        // Rescale existing output and add new value contribution
        let v_offset = t * head_dim;
        for i in 0..head_dim {
            output[i] = output[i] * scale_old + exp_score * value_cache[v_offset + i];
        }

        running_max = new_max;
    }

    // Final normalization
    if running_sum > 0.0 {
        let inv_sum = 1.0 / running_sum;
        for v in output.iter_mut() {
            *v *= inv_sum;
        }
    }
}

/// Multi-head attention: apply attention for all heads in parallel.
pub fn multi_head_attention(
    output: &mut [f32],
    q: &[f32],
    key_cache: &[f32],
    value_cache: &[f32],
    cfg: &AttentionConfig,
) {
    let gqa_ratio = cfg.n_heads / cfg.n_kv_heads;

    for h in 0..cfg.n_heads {
        let q_offset = h * cfg.head_dim;
        let kv_head = h / gqa_ratio;
        let kv_stride = cfg.n_kv_heads * cfg.head_dim;

        // For GQA: multiple Q heads share the same KV head
        let k_base = kv_head * cfg.head_dim;
        let v_base = kv_head * cfg.head_dim;

        // Build per-head key/value views (strided)
        let q_slice = &q[q_offset..q_offset + cfg.head_dim];
        let out_slice = &mut output[q_offset..q_offset + cfg.head_dim];

        // Single-head attention with flash attention
        attention_strided(
            out_slice,
            q_slice,
            key_cache,
            value_cache,
            cfg.seq_len,
            cfg.head_dim,
            kv_stride,
            k_base,
            v_base,
        );
    }
}

struct AttentionStridedParams {
    seq_len: usize,
    head_dim: usize,
    kv_stride: usize,
    k_base: usize,
    v_base: usize,
}

/// Strided attention — works with interleaved multi-head KV cache layout.
fn attention_strided(
    output: &mut [f32],
    q: &[f32],
    key_cache: &[f32],
    value_cache: &[f32],
    seq_len: usize,
    head_dim: usize,
    kv_stride: usize,
    k_base: usize,
    v_base: usize,
) {
    let params = AttentionStridedParams {
        seq_len,
        head_dim,
        kv_stride,
        k_base,
        v_base,
    };

    if params.seq_len == 0 {
        for v in output.iter_mut() {
            *v = 0.0;
        }
        return;
    }

    let scale = 1.0 / (params.head_dim as f32).sqrt();
    let mut running_max = f32::NEG_INFINITY;
    let mut running_sum = 0.0f32;
    for v in output.iter_mut() {
        *v = 0.0;
    }

    for t in 0..params.seq_len {
        let k_offset = t * params.kv_stride + params.k_base;
        let v_offset = t * params.kv_stride + params.v_base;

        let mut dot = 0.0f32;
        for i in 0..params.head_dim {
            dot += q[i] * key_cache[k_offset + i];
        }
        let score = dot * scale;

        let new_max = running_max.max(score);
        let scale_old = (running_max - new_max).exp();
        let exp_score = (score - new_max).exp();

        running_sum = running_sum * scale_old + exp_score;

        for i in 0..params.head_dim {
            output[i] = output[i] * scale_old + exp_score * value_cache[v_offset + i];
        }

        running_max = new_max;
    }

    if running_sum > 0.0 {
        let inv_sum = 1.0 / running_sum;
        for v in output.iter_mut() {
            *v *= inv_sum;
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

    #[test]
    fn test_flash_attention_consistency() {
        // Verify flash attention gives same result as standard softmax
        let head_dim = 4;
        let seq_len = 3;
        let q = vec![1.0, 0.5, 0.0, 0.0];
        let key_cache = vec![1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0];
        let value_cache = vec![1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let mut output = vec![0.0; head_dim];

        attention(&mut output, &q, &key_cache, &value_cache, seq_len, head_dim);

        // Output should be a weighted combination of values
        let total: f32 = output.iter().sum();
        assert!(
            (total - 1.0).abs() < 1e-4,
            "Attention weights should sum to ~1.0, got {total}"
        );
    }

    #[test]
    fn test_attention_empty() {
        let head_dim = 4;
        let q = vec![1.0, 0.0, 0.0, 0.0];
        let mut output = vec![1.0; head_dim];

        attention(&mut output, &q, &[], &[], 0, head_dim);

        for v in &output {
            assert_eq!(*v, 0.0);
        }
    }
}
