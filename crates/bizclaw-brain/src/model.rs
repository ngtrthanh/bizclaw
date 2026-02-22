//! LLaMA model forward pass.
//!
//! Implements the transformer architecture for LLaMA-family models.
//! Reads weights from mmap, dequantizes on-the-fly, and computes
//! the forward pass producing logits for the next token.

/// Model hyperparameters extracted from GGUF metadata.
#[derive(Debug, Clone)]
pub struct ModelParams {
    pub vocab_size: u32,
    pub dim: u32,        // embedding dimension
    pub hidden_dim: u32, // FFN hidden dimension
    pub n_layers: u32,
    pub n_heads: u32,
    pub n_kv_heads: u32, // for GQA (Grouped Query Attention)
    pub head_dim: u32,   // dim / n_heads
    pub max_seq_len: u32,
    pub rope_theta: f32,
    pub rms_norm_eps: f32,
}

impl Default for ModelParams {
    fn default() -> Self {
        // TinyLlama 1.1B defaults
        Self {
            vocab_size: 32000,
            dim: 2048,
            hidden_dim: 5632,
            n_layers: 22,
            n_heads: 32,
            n_kv_heads: 4,
            head_dim: 64,
            max_seq_len: 2048,
            rope_theta: 10000.0,
            rms_norm_eps: 1e-5,
        }
    }
}

impl ModelParams {
    /// Extract model parameters from GGUF metadata.
    pub fn from_gguf(gguf: &crate::gguf::GgufFile) -> Self {
        let arch = gguf.architecture().unwrap_or("llama");
        let prefix = format!("{arch}.");

        let dim = gguf
            .get_u32(&format!("{prefix}embedding_length"))
            .unwrap_or(2048);
        let n_heads = gguf
            .get_u32(&format!("{prefix}attention.head_count"))
            .unwrap_or(32);
        let n_kv_heads = gguf
            .get_u32(&format!("{prefix}attention.head_count_kv"))
            .unwrap_or(n_heads);

        Self {
            vocab_size: gguf
                .get_u32(&format!("{prefix}vocab_size"))
                .or_else(|| {
                    // Count from tokenizer tokens
                    gguf.metadata
                        .get("tokenizer.ggml.tokens")
                        .and_then(|v| match v {
                            crate::gguf::GgufValue::Array(arr) => Some(arr.len() as u32),
                            _ => None,
                        })
                })
                .unwrap_or(32000),
            dim,
            hidden_dim: gguf
                .get_u32(&format!("{prefix}feed_forward_length"))
                .unwrap_or(5632),
            n_layers: gguf.get_u32(&format!("{prefix}block_count")).unwrap_or(22),
            n_heads,
            n_kv_heads,
            head_dim: dim / n_heads,
            max_seq_len: gguf
                .get_u32(&format!("{prefix}context_length"))
                .unwrap_or(2048),
            rope_theta: gguf
                .get_f32(&format!("{prefix}rope.freq_base"))
                .unwrap_or(10000.0),
            rms_norm_eps: gguf
                .get_f32(&format!("{prefix}attention.layer_norm_rms_epsilon"))
                .unwrap_or(1e-5),
        }
    }
}
