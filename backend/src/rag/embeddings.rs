use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

pub struct EmbeddingGenerator {
    model: TextEmbedding,
}

impl EmbeddingGenerator {
    pub async fn new() -> Result<Self> {
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::BGESmallENV15)
                .with_show_download_progress(true)
        )?;
        
        Ok(Self { model })
    }

    pub fn generate(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let embeddings = self.model.embed(texts, None)?;
        Ok(embeddings)
    }

    pub fn generate_single(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.generate(vec![text.to_string()])?;
        Ok(embeddings.into_iter().next().unwrap())
    }
}
