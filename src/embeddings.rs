use anyhow::Result;
use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};

/// Wraps a local ONNX embedding model (via fastembed/ort). Runs entirely on
/// CPU, no network calls after the model is downloaded/cached the first time.
///
/// NOTE ON PREFIXES: BGE-family models are trained to expect a "query: " or
/// "passage: " prefix depending on which side of the search you're embedding.
/// Skipping this still works but retrieval quality drops noticeably — keep it.
pub struct Embedder {
    model: std::sync::Mutex<TextEmbedding>,
}

impl Embedder {
    pub fn new() -> Result<Self> {
        let model = TextEmbedding::try_new(
            TextInitOptions::new(EmbeddingModel::BGESmallENV15)
                .with_show_download_progress(true),
        )?;
        Ok(Self { model: std::sync::Mutex::new(model) })
    }

    pub fn embed_passages(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let prefixed: Vec<String> = texts.iter().map(|t| format!("passage: {t}")).collect();
        let mut model = self.model.lock().unwrap();
        Ok(model.embed(prefixed, None)?)
    }

    pub fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        let mut model = self.model.lock().unwrap();
        let mut out = model.embed(vec![format!("query: {text}")], None)?;
        Ok(out.remove(0))
    }
}
