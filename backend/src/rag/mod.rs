pub mod embeddings;
pub mod vector_store;
pub mod index_manager;
pub mod versioning;

use std::sync::Arc;
use anyhow::Result;
use self::embeddings::EmbeddingGenerator;
use self::vector_store::VectorStore;

pub struct RAGEngine {
    pub embeddings: Arc<EmbeddingGenerator>,
    pub vector_store: Arc<VectorStore>,
}

impl RAGEngine {
    pub async fn new(qdrant_url: &str, collection_name: &str) -> Result<Self> {
        let embeddings = Arc::new(EmbeddingGenerator::new().await?);
        let vector_store = Arc::new(VectorStore::new(qdrant_url, collection_name).await?);

        Ok(Self {
            embeddings,
            vector_store,
        })
    }

    pub async fn add_document(
        &self,
        id: &str,
        text: &str,
        metadata: serde_json::Value
    ) -> Result<()> {
        let embedding = self.embeddings.generate_single(text)?;
        self.vector_store.add_document(id, text, embedding, metadata).await?;
        Ok(())
    }

    pub async fn retrieve_context(&self, query: &str, top_k: u64) -> Result<String> {
        let query_embedding = self.embeddings.generate_single(query)?;
        let results = self.vector_store.search(query_embedding, top_k).await?;

        if results.is_empty() {
            return Ok(String::new());
        }

        let context = results.join("\n\n");
        Ok(format!("関連情報:\n{}\n\n", context))
    }
}
