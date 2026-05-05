//! Embedding generation for semantic similarity search.
//!
//! Behind the `ml` feature flag (opt-in). The default lean build does not
//! include any ML dependency. When the feature is enabled, fastembed-rs
//! provides ONNX-based BGE-small-en-v1.5 (384 dim) embeddings.
//!
//! Models auto-download to `~/.cache/first-plan/models/` on first use.

use anyhow::Result;

/// Dimensionality of the embedding vectors. BGE-small produces 384-dim.
pub const EMBEDDING_DIM: usize = 384;

/// Trait abstracting the embedding backend so callers don't depend on
/// fastembed directly.
pub trait EmbeddingProvider: Send + Sync {
    fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    fn dim(&self) -> usize;
    fn model_name(&self) -> &str;
}

#[cfg(feature = "ml")]
pub use ml::FastEmbedProvider;

#[cfg(not(feature = "ml"))]
pub fn make_default_provider() -> Result<Box<dyn EmbeddingProvider>> {
    anyhow::bail!(
        "embedding provider not available - this binary was built without --features=ml. \
         Install the ML-enabled variant from the project releases."
    )
}

#[cfg(feature = "ml")]
pub fn make_default_provider() -> Result<Box<dyn EmbeddingProvider>> {
    Ok(Box::new(ml::FastEmbedProvider::new()?))
}

/// Cosine similarity between two equal-length vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let mut dot = 0.0_f32;
    let mut norm_a = 0.0_f32;
    let mut norm_b = 0.0_f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a.sqrt() * norm_b.sqrt())
}

/// Convert a slice of f32 to bytes for SQLite BLOB storage.
pub fn f32_slice_to_bytes(vec: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vec.len() * 4);
    for v in vec {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes
}

/// Convert bytes from SQLite BLOB back to a Vec<f32>.
pub fn bytes_to_f32_vec(bytes: &[u8]) -> Vec<f32> {
    if !bytes.len().is_multiple_of(4) {
        return Vec::new();
    }
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

#[cfg(feature = "ml")]
mod ml {
    use super::{EmbeddingProvider, EMBEDDING_DIM};
    use anyhow::{Context, Result};
    use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

    pub struct FastEmbedProvider {
        model: TextEmbedding,
        model_name: String,
    }

    impl FastEmbedProvider {
        pub fn new() -> Result<Self> {
            let cache_dir = dirs::cache_dir()
                .map(|d| d.join("first-plan").join("models"))
                .context("could not determine cache dir")?;
            std::fs::create_dir_all(&cache_dir).ok();

            let opts = InitOptions::new(EmbeddingModel::BGESmallENV15)
                .with_cache_dir(cache_dir)
                .with_show_download_progress(false);
            let model = TextEmbedding::try_new(opts).context("init fastembed model")?;
            Ok(Self {
                model,
                model_name: "BGE-small-en-v1.5".into(),
            })
        }
    }

    impl EmbeddingProvider for FastEmbedProvider {
        fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            if texts.is_empty() {
                return Ok(Vec::new());
            }
            let owned: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
            let embeddings = self
                .model
                .embed(owned, None)
                .context("fastembed inference")?;
            Ok(embeddings)
        }

        fn dim(&self) -> usize {
            EMBEDDING_DIM
        }

        fn model_name(&self) -> &str {
            &self.model_name
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_identical_vectors() {
        let v = vec![1.0, 2.0, 3.0];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_orthogonal_vectors() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b)).abs() < 1e-6);
    }

    #[test]
    fn cosine_zero_vector() {
        let zero = vec![0.0_f32; 3];
        let v = vec![1.0_f32, 2.0, 3.0];
        assert_eq!(cosine_similarity(&zero, &v), 0.0);
    }

    #[test]
    fn bytes_roundtrip() {
        let v = vec![1.5_f32, -2.3, 0.0, 99.99];
        let bytes = f32_slice_to_bytes(&v);
        let restored = bytes_to_f32_vec(&bytes);
        for (a, b) in v.iter().zip(restored.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }
}
