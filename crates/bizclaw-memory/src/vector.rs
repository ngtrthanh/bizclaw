//! In-memory vector search engine for semantic memory.
//!
//! Uses cosine similarity for nearest-neighbor search.
//! Phase 3: Will integrate with bizclaw-brain for embeddings.

use bizclaw_core::traits::memory::{MemoryEntry, MemorySearchResult};

/// Simple in-memory vector store using cosine similarity.
pub struct VectorStore {
    entries: Vec<(MemoryEntry, Vec<f32>)>,
}

impl VectorStore {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    /// Add an entry with its embedding vector.
    pub fn add(&mut self, entry: MemoryEntry, embedding: Vec<f32>) {
        self.entries.push((entry, embedding));
    }

    /// Search by cosine similarity against a query embedding.
    pub fn search(&self, query_embedding: &[f32], limit: usize) -> Vec<MemorySearchResult> {
        let mut scored: Vec<(f32, &MemoryEntry)> = self
            .entries
            .iter()
            .map(|(entry, emb)| {
                let score = cosine_similarity(query_embedding, emb);
                (score, entry)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);

        scored
            .into_iter()
            .map(|(score, entry)| MemorySearchResult {
                entry: entry.clone(),
                score,
            })
            .collect()
    }

    /// Number of stored vectors.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if store is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute cosine similarity between two vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 { 0.0 } else { dot / denom }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 1.0];
        let b = vec![1.0, 0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim + 1.0).abs() < 1e-6);
    }
}
