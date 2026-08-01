use anyhow::Result;
use std::path::Path;

use crate::embeddings::Embedder;

pub struct Doc {
    pub source: String,
    pub text: String,
    pub embedding: Vec<f32>,
}

pub struct DocStore {
    docs: Vec<Doc>,
}

impl DocStore {
    /// Loads every `.txt` file in `dir` (one file = one document; no chunking —
    /// see the note in README about chunking longer files for real use) and
    /// embeds them all up front.
    pub fn load_dir(dir: impl AsRef<Path>, embedder: &Embedder) -> Result<Self> {
        let dir = dir.as_ref();
        let mut sources = Vec::new();
        let mut texts = Vec::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("txt") {
                let text = std::fs::read_to_string(&path)?;
                sources.push(path.file_name().unwrap().to_string_lossy().to_string());
                texts.push(text);
            }
        }

        if texts.is_empty() {
            anyhow::bail!("no .txt files found in {}", dir.display());
        }

        let embeddings = embedder.embed_passages(&texts)?;

        let docs = sources
            .into_iter()
            .zip(texts)
            .zip(embeddings)
            .map(|((source, text), embedding)| Doc { source, text, embedding })
            .collect();

        Ok(Self { docs })
    }

    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<(&Doc, f32)> {
        let mut scored: Vec<(&Doc, f32)> = self
            .docs
            .iter()
            .map(|d| (d, cosine_similarity(query_embedding, &d.embedding)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored.truncate(top_k);
        scored
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}
