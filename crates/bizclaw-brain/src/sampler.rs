//! Temperature + Top-p/Top-k sampling for token generation.

use rand::Rng;

/// Sampler configuration.
#[derive(Debug, Clone)]
pub struct SamplerConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: u32,
    pub repeat_penalty: f32,
    pub repeat_last_n: usize,
}

impl Default for SamplerConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
        }
    }
}

/// Token sampler — selects next token from logits.
pub struct Sampler {
    config: SamplerConfig,
}

impl Sampler {
    pub fn new(config: SamplerConfig) -> Self {
        Self { config }
    }

    /// Sample a token from logits.
    pub fn sample(&self, logits: &mut [f32], last_tokens: &[u32]) -> u32 {
        // Apply repeat penalty
        if self.config.repeat_penalty != 1.0 {
            let n = last_tokens.len().min(self.config.repeat_last_n);
            for &token_id in &last_tokens[last_tokens.len().saturating_sub(n)..] {
                let idx = token_id as usize;
                if idx < logits.len() {
                    if logits[idx] > 0.0 {
                        logits[idx] /= self.config.repeat_penalty;
                    } else {
                        logits[idx] *= self.config.repeat_penalty;
                    }
                }
            }
        }

        // Apply temperature
        if self.config.temperature > 0.0 && self.config.temperature != 1.0 {
            let inv_temp = 1.0 / self.config.temperature;
            for logit in logits.iter_mut() {
                *logit *= inv_temp;
            }
        }

        // If temperature is 0, return argmax (greedy)
        if self.config.temperature <= 0.0 {
            return argmax(logits);
        }

        // Create sorted indices
        let mut indices: Vec<(usize, f32)> =
            logits.iter().enumerate().map(|(i, &v)| (i, v)).collect();
        indices.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Top-K filtering
        let top_k = if self.config.top_k > 0 {
            (self.config.top_k as usize).min(indices.len())
        } else {
            indices.len()
        };
        indices.truncate(top_k);

        // Softmax
        let max_logit = indices[0].1;
        let mut probs: Vec<(usize, f32)> = indices
            .iter()
            .map(|&(i, v)| (i, (v - max_logit).exp()))
            .collect();
        let sum: f32 = probs.iter().map(|&(_, p)| p).sum();
        for p in probs.iter_mut() {
            p.1 /= sum;
        }

        // Top-P (nucleus) sampling
        if self.config.top_p < 1.0 {
            let mut cumulative = 0.0;
            let mut cutoff = probs.len();
            for (i, &(_, p)) in probs.iter().enumerate() {
                cumulative += p;
                if cumulative > self.config.top_p {
                    cutoff = i + 1;
                    break;
                }
            }
            probs.truncate(cutoff);

            // Re-normalize
            let sum: f32 = probs.iter().map(|&(_, p)| p).sum();
            for p in probs.iter_mut() {
                p.1 /= sum;
            }
        }

        // Random sampling
        let mut rng = rand::thread_rng();
        let r: f32 = rng.r#gen();
        let mut cumulative = 0.0;
        for &(idx, prob) in &probs {
            cumulative += prob;
            if r < cumulative {
                return idx as u32;
            }
        }

        // Fallback
        probs.last().map(|&(idx, _)| idx as u32).unwrap_or(0)
    }
}

/// Return the index of the maximum value (greedy decoding).
fn argmax(values: &[f32]) -> u32 {
    values
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i as u32)
        .unwrap_or(0)
}
