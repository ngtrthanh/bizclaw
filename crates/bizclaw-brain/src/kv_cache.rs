//! KV Cache for key-value pairs across attention layers.
//!
//! Stores and manages key/value vectors for all layers and heads,
//! enabling auto-regressive generation without recomputing past tokens.

/// KV Cache for transformer inference.
pub struct KvCache {
    /// Key cache: [n_layers x max_seq_len x n_kv_heads x head_dim]
    key_cache: Vec<f32>,
    /// Value cache: [n_layers x max_seq_len x n_kv_heads x head_dim]
    value_cache: Vec<f32>,
    /// Number of layers.
    n_layers: usize,
    /// Maximum sequence length.
    max_seq_len: usize,
    /// Key/value dimension per layer (n_kv_heads * head_dim).
    kv_dim: usize,
    /// Current sequence position.
    pos: usize,
}

impl KvCache {
    /// Create a new KV cache.
    pub fn new(n_layers: usize, max_seq_len: usize, n_kv_heads: usize, head_dim: usize) -> Self {
        let kv_dim = n_kv_heads * head_dim;
        let total = n_layers * max_seq_len * kv_dim;
        Self {
            key_cache: vec![0.0; total],
            value_cache: vec![0.0; total],
            n_layers,
            max_seq_len,
            kv_dim,
            pos: 0,
        }
    }

    /// Get a mutable slice for writing a key vector at the current position.
    pub fn key_at_mut(&mut self, layer: usize, pos: usize) -> &mut [f32] {
        let offset = (layer * self.max_seq_len + pos) * self.kv_dim;
        &mut self.key_cache[offset..offset + self.kv_dim]
    }

    /// Get a mutable slice for writing a value vector at the current position.
    pub fn value_at_mut(&mut self, layer: usize, pos: usize) -> &mut [f32] {
        let offset = (layer * self.max_seq_len + pos) * self.kv_dim;
        &mut self.value_cache[offset..offset + self.kv_dim]
    }

    /// Get all key vectors for a layer up to seq_len.
    pub fn keys(&self, layer: usize, seq_len: usize) -> &[f32] {
        let offset = layer * self.max_seq_len * self.kv_dim;
        &self.key_cache[offset..offset + seq_len * self.kv_dim]
    }

    /// Get all value vectors for a layer up to seq_len.
    pub fn values(&self, layer: usize, seq_len: usize) -> &[f32] {
        let offset = layer * self.max_seq_len * self.kv_dim;
        &self.value_cache[offset..offset + seq_len * self.kv_dim]
    }

    /// Advance the position counter.
    pub fn advance(&mut self) {
        self.pos += 1;
    }

    /// Get current position.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Reset cache (clear all KV entries).
    pub fn reset(&mut self) {
        self.key_cache.fill(0.0);
        self.value_cache.fill(0.0);
        self.pos = 0;
    }

    /// Memory usage in bytes.
    pub fn memory_usage(&self) -> usize {
        (self.key_cache.len() + self.value_cache.len()) * std::mem::size_of::<f32>()
    }
}
