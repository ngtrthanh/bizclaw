//! BPE (Byte Pair Encoding) tokenizer for LLaMA models.
//!
//! Reads vocabulary and merge rules from GGUF metadata and converts
//! text to/from token IDs.

use crate::gguf::GgufValue;
use bizclaw_core::error::{BizClawError, Result};
use std::collections::HashMap;

/// BPE tokenizer for LLaMA-family models.
pub struct BpeTokenizer {
    /// Token ID → string mapping.
    vocab: Vec<String>,
    /// String → Token ID mapping.
    token_to_id: HashMap<String, u32>,
    /// Token scores (used for BPE merge priority).
    scores: Vec<f32>,
    /// Special token IDs.
    pub bos_id: u32,
    pub eos_id: u32,
    pub pad_id: u32,
}

impl BpeTokenizer {
    /// Create a tokenizer from GGUF metadata.
    pub fn from_gguf(metadata: &HashMap<String, GgufValue>) -> Result<Self> {
        // Extract vocabulary tokens
        let tokens = metadata
            .get("tokenizer.ggml.tokens")
            .and_then(|v| match v {
                GgufValue::Array(arr) => Some(arr),
                _ => None,
            })
            .ok_or_else(|| BizClawError::Brain("Missing tokenizer.ggml.tokens".into()))?;

        let vocab: Vec<String> = tokens
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        if vocab.is_empty() {
            return Err(BizClawError::Brain("Empty vocabulary".into()));
        }

        // Extract scores
        let scores: Vec<f32> = metadata
            .get("tokenizer.ggml.scores")
            .and_then(|v| match v {
                GgufValue::Array(arr) => Some(arr.iter().filter_map(|v| v.as_f32()).collect()),
                _ => None,
            })
            .unwrap_or_else(|| vec![0.0; vocab.len()]);

        // Build reverse mapping
        let token_to_id: HashMap<String, u32> = vocab
            .iter()
            .enumerate()
            .map(|(i, t)| (t.clone(), i as u32))
            .collect();

        // Extract special tokens
        let bos_id = metadata
            .get("tokenizer.ggml.bos_token_id")
            .and_then(|v| v.as_u32())
            .unwrap_or(1);
        let eos_id = metadata
            .get("tokenizer.ggml.eos_token_id")
            .and_then(|v| v.as_u32())
            .unwrap_or(2);
        let pad_id = metadata
            .get("tokenizer.ggml.padding_token_id")
            .and_then(|v| v.as_u32())
            .unwrap_or(0);

        tracing::info!(
            "Tokenizer loaded: vocab_size={}, bos={}, eos={}",
            vocab.len(),
            bos_id,
            eos_id
        );

        Ok(Self {
            vocab,
            token_to_id,
            scores,
            bos_id,
            eos_id,
            pad_id,
        })
    }

    /// Create a simple fallback tokenizer (for testing without a model).
    pub fn fallback() -> Self {
        let vocab: Vec<String> = vec!["<pad>".into(), "<bos>".into(), "<eos>".into(), " ".into()];
        let token_to_id: HashMap<String, u32> = vocab
            .iter()
            .enumerate()
            .map(|(i, t)| (t.clone(), i as u32))
            .collect();
        Self {
            scores: vec![0.0; vocab.len()],
            vocab,
            token_to_id,
            bos_id: 1,
            eos_id: 2,
            pad_id: 0,
        }
    }

    /// Encode text into token IDs using BPE.
    pub fn encode(&self, text: &str) -> Vec<u32> {
        if text.is_empty() {
            return vec![];
        }

        // Step 1: UTF-8 byte-level encoding — each byte becomes a token
        let mut tokens: Vec<u32> = Vec::new();
        for byte in text.bytes() {
            // Try to find the byte as a token
            let byte_str = format!("{}", byte as char);
            if let Some(&id) = self.token_to_id.get(&byte_str) {
                tokens.push(id);
            } else {
                // Fallback: try hex representation
                let hex = format!("<0x{:02X}>", byte);
                if let Some(&id) = self.token_to_id.get(&hex) {
                    tokens.push(id);
                } else {
                    // Last resort: use pad token
                    tokens.push(self.pad_id);
                }
            }
        }

        // Step 2: BPE merges — iteratively merge the best pair
        loop {
            if tokens.len() < 2 {
                break;
            }

            // Find the best merge (highest score pair)
            let mut best_score = f32::NEG_INFINITY;
            let mut best_idx = usize::MAX;
            let mut best_id = 0u32;

            for i in 0..tokens.len() - 1 {
                let merged = format!(
                    "{}{}",
                    self.decode_token(tokens[i]),
                    self.decode_token(tokens[i + 1])
                );
                if let Some(&id) = self.token_to_id.get(&merged) {
                    let score = if (id as usize) < self.scores.len() {
                        self.scores[id as usize]
                    } else {
                        0.0
                    };
                    if score > best_score {
                        best_score = score;
                        best_idx = i;
                        best_id = id;
                    }
                }
            }

            if best_idx == usize::MAX {
                break; // No more merges possible
            }

            // Apply the merge
            tokens[best_idx] = best_id;
            tokens.remove(best_idx + 1);
        }

        tokens
    }

    /// Decode a single token ID to string.
    pub fn decode_token(&self, id: u32) -> &str {
        self.vocab
            .get(id as usize)
            .map(|s| s.as_str())
            .unwrap_or("<unk>")
    }

    /// Decode a sequence of token IDs to text.
    pub fn decode(&self, tokens: &[u32]) -> String {
        tokens
            .iter()
            .map(|&id| self.decode_token(id))
            .collect::<Vec<&str>>()
            .join("")
    }

    /// Get vocabulary size.
    pub fn vocab_size(&self) -> usize {
        self.vocab.len()
    }

    /// Check if a token is a special token.
    pub fn is_special(&self, id: u32) -> bool {
        id == self.bos_id || id == self.eos_id || id == self.pad_id
    }
}
