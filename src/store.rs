use anyhow::{anyhow, Result};
use pulldown_cmark::{Event, Parser, TagEnd};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;

use std::io::Read;
use std::path::Path;

use crate::embeddings::Embedder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doc {
    pub source: String,
    pub chunk_index: usize,
    pub text: String,
    pub embedding: Vec<f32>,
}

#[derive(Serialize, Deserialize)]
struct VectorCache {
    hash_digest: String,
    docs: Vec<Doc>,
}

pub struct DocStore {
    docs: Vec<Doc>,
}

impl DocStore {
    /// Loads documents from `dir` supporting `.txt`, `.md`, `.pdf`, `.csv`, and `.json`.
    /// Applies sliding-window chunking (300 words with 50-word overlap) and
    /// caches pre-computed embeddings to `.vector_cache.bin` on disk for instant startup.
    pub fn load_dir(dir: impl AsRef<Path>, embedder: &Embedder) -> Result<Self> {
        let dir = dir.as_ref();
        let cache_path = dir.join(".vector_cache.bin");

        // 1. Gather all supported files and compute total hash digest
        let mut supported_files = Vec::new();
        let mut hasher = Sha256::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                if matches!(ext.as_str(), "txt" | "md" | "pdf" | "csv" | "json") {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        hasher.update(path.to_string_lossy().as_bytes());
                        hasher.update(metadata.len().to_le_bytes());
                        hasher.update(
                            metadata
                                .modified()
                                .ok()
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs())
                                .unwrap_or(0)
                                .to_le_bytes(),
                        );
                    }
                    supported_files.push(path);
                }
            }
        }

        supported_files.sort();

        if supported_files.is_empty() {
            anyhow::bail!("no supported documents (.txt, .md, .pdf, .csv, .json) found in {}", dir.display());
        }

        let current_digest = format!("{:x}", hasher.finalize());

        // 2. Try loading pre-computed cache from binary file
        if cache_path.exists() {
            if let Ok(mut file) = File::open(&cache_path) {
                let mut buffer = Vec::new();
                if file.read_to_end(&mut buffer).is_ok() {
                    if let Ok(cache) = bincode::deserialize::<VectorCache>(&buffer) {
                        if cache.hash_digest == current_digest {
                            eprintln!("[cache] Loaded {} cached passages from {}", cache.docs.len(), cache_path.display());
                            return Ok(Self { docs: cache.docs });
                        }
                    }
                }
            }
        }

        // 3. Parse files, chunk passages, and generate embeddings
        let mut sources = Vec::new();
        let mut chunk_indices = Vec::new();
        let mut chunk_texts = Vec::new();

        for path in supported_files {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            let raw_text = match extract_text_from_file(&path) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[store] Warning: Failed to extract text from {filename}: {e}");
                    continue;
                }
            };

            let chunks = chunk_text(&raw_text, 300, 50);
            for (idx, chunk) in chunks.into_iter().enumerate() {
                sources.push(filename.clone());
                chunk_indices.push(idx);
                chunk_texts.push(chunk);
            }
        }

        if chunk_texts.is_empty() {
            anyhow::bail!("no readable text content extracted from documents in {}", dir.display());
        }

        eprintln!("[store] Embedding {} document chunks...", chunk_texts.len());
        let embeddings = embedder.embed_passages(&chunk_texts)?;

        let docs: Vec<Doc> = sources
            .into_iter()
            .zip(chunk_indices)
            .zip(chunk_texts)
            .zip(embeddings)
            .map(|(((source, chunk_index), text), embedding)| Doc {
                source,
                chunk_index,
                text,
                embedding,
            })
            .collect();

        // 4. Save pre-computed embeddings to disk cache
        let cache = VectorCache {
            hash_digest: current_digest,
            docs: docs.clone(),
        };
        if let Ok(encoded) = bincode::serialize(&cache) {
            let _ = std::fs::write(&cache_path, encoded);
        }

        Ok(Self { docs })
    }

    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<(&Doc, f32)> {
        let mut scored: Vec<(&Doc, f32)> = self
            .docs
            .iter()
            .map(|d| (d, cosine_similarity(query_embedding, &d.embedding)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    pub fn get_document_titles(&self) -> Vec<String> {
        let mut titles: Vec<String> = self.docs.iter().map(|d| d.source.clone()).collect();
        titles.sort();
        titles.dedup();
        titles
    }
}

/// Extract text from `.txt`, `.md`, `.pdf`, `.csv`, or `.json` files.
fn extract_text_from_file(path: &Path) -> Result<String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "txt" => Ok(std::fs::read_to_string(path)?),
        "md" => {
            let raw = std::fs::read_to_string(path)?;
            let parser = Parser::new(&raw);
            let mut plain_text = String::new();
            for event in parser {
                match event {
                    Event::Text(text) | Event::Code(text) => plain_text.push_str(&text),
                    Event::SoftBreak | Event::HardBreak => plain_text.push('\n'),
                    Event::End(TagEnd::Paragraph | TagEnd::Heading(_)) => plain_text.push_str("\n\n"),
                    _ => {}
                }
            }
            Ok(plain_text)
        }
        "pdf" => {
            let bytes = std::fs::read(path)?;
            pdf_extract::extract_text_from_mem(&bytes).map_err(|e| anyhow!("PDF extraction error: {e}"))
        }
        "csv" => {
            let mut rdr = csv::Reader::from_path(path)?;
            let headers = rdr.headers()?.clone();
            let mut formatted = String::new();
            for record in rdr.records() {
                let record = record?;
                let mut line = Vec::new();
                for (header, val) in headers.iter().zip(record.iter()) {
                    if !val.trim().is_empty() {
                        line.push(format!("{header}: {val}"));
                    }
                }
                formatted.push_str(&line.join(", "));
                formatted.push('\n');
            }
            Ok(formatted)
        }
        "json" => {
            let raw = std::fs::read_to_string(path)?;
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw) {
                Ok(serde_json::to_string_pretty(&parsed)?)
            } else {
                Ok(raw)
            }
        }
        _ => Ok(std::fs::read_to_string(path)?),
    }
}

/// Sliding-window text chunker (target chunk_words count with overlap_words overlap).
fn chunk_text(text: &str, chunk_words: usize, overlap_words: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() <= chunk_words {
        return vec![text.trim().to_string()];
    }

    let mut chunks = Vec::new();
    let step = chunk_words.saturating_sub(overlap_words).max(1);

    let mut start = 0;
    while start < words.len() {
        let end = (start + chunk_words).min(words.len());
        let chunk_str = words[start..end].join(" ");
        if !chunk_str.trim().is_empty() {
            chunks.push(chunk_str);
        }
        if end == words.len() {
            break;
        }
        start += step;
    }

    chunks
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
