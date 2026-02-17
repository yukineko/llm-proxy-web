use anyhow::Result;
use fastembed::{TextEmbedding, UserDefinedEmbeddingModel, TokenizerFiles, InitOptionsUserDefined};
use std::path::Path;

const MODEL_DIR: &str = "/app/models/bge-small-en-v1.5";

pub struct EmbeddingGenerator {
    model: TextEmbedding,
}

impl EmbeddingGenerator {
    pub async fn new() -> Result<Self> {
        tracing::info!("Initializing embedding model from local files...");

        let model_dir = Path::new(MODEL_DIR);
        if !model_dir.exists() {
            anyhow::bail!("Model directory not found: {}", MODEL_DIR);
        }

        let onnx_file = std::fs::read(model_dir.join("model.onnx"))
            .map_err(|e| anyhow::anyhow!("Failed to read model.onnx: {}", e))?;
        let tokenizer_file = std::fs::read(model_dir.join("tokenizer.json"))
            .map_err(|e| anyhow::anyhow!("Failed to read tokenizer.json: {}", e))?;
        let config_file = std::fs::read(model_dir.join("config.json"))
            .map_err(|e| anyhow::anyhow!("Failed to read config.json: {}", e))?;
        let special_tokens_map_file = std::fs::read(model_dir.join("special_tokens_map.json"))
            .map_err(|e| anyhow::anyhow!("Failed to read special_tokens_map.json: {}", e))?;
        let tokenizer_config_file = std::fs::read(model_dir.join("tokenizer_config.json"))
            .map_err(|e| anyhow::anyhow!("Failed to read tokenizer_config.json: {}", e))?;

        tracing::info!("Model files loaded, creating embedding model...");

        let user_model = UserDefinedEmbeddingModel {
            onnx_file,
            tokenizer_files: TokenizerFiles {
                tokenizer_file,
                config_file,
                special_tokens_map_file,
                tokenizer_config_file,
            },
        };

        let model = TextEmbedding::try_new_from_user_defined(user_model, InitOptionsUserDefined::default())
            .map_err(|e| anyhow::anyhow!("Failed to initialize embedding model: {}", e))?;

        tracing::info!("Embedding model initialized successfully");
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
