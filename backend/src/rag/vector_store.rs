use anyhow::Result;
use qdrant_client::prelude::*;
use qdrant_client::qdrant::{
    CreateCollection, Distance, VectorParams, VectorsConfig,
    SearchPoints, PointStruct, WithPayloadSelector,
};
use serde_json::{json, Map as JsonMap, Value as JsonValue};

pub struct VectorStore {
    client: QdrantClient,
    collection_name: String,
}

impl VectorStore {
    pub async fn new(url: &str, collection_name: &str) -> Result<Self> {
        let client = QdrantClient::from_url(url).build()?;
        
        let store = Self {
            client,
            collection_name: collection_name.to_string(),
        };

        store.ensure_collection().await?;
        
        Ok(store)
    }

    async fn ensure_collection(&self) -> Result<()> {
        match self.client.collection_info(&self.collection_name).await {
            Ok(_) => return Ok(()),
            Err(_) => {
                self.client
                    .create_collection(&CreateCollection {
                        collection_name: self.collection_name.clone(),
                        vectors_config: Some(VectorsConfig {
                            config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                                VectorParams {
                                    size: 384, // BGE-small-en-v1.5の次元数
                                    distance: Distance::Cosine.into(),
                                    ..Default::default()
                                },
                            )),
                        }),
                        ..Default::default()
                    })
                    .await?;
            }
        }
        
        Ok(())
    }

    pub async fn add_document(
        &self,
        id: &str,
        text: &str,
        embedding: Vec<f32>,
        metadata: serde_json::Value
    ) -> Result<()> {
        let mut payload_map = JsonMap::new();
        payload_map.insert("text".to_string(), JsonValue::String(text.to_string()));
        payload_map.insert("metadata".to_string(), metadata);
        let point = PointStruct::new(
            id.to_string(),
            embedding,
            Payload::from(payload_map),
        );

        self.client
            .upsert_points_blocking(&self.collection_name, None, vec![point], None)
            .await?;

        Ok(())
    }

    pub async fn search(&self, query_vector: Vec<f32>, limit: u64) -> Result<Vec<String>> {
        let search_result = self.client
            .search_points(&SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: query_vector,
                limit,
                with_payload: Some(WithPayloadSelector {
                    selector_options: Some(
                        qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true),
                    ),
                }),
                ..Default::default()
            })
            .await?;

        let mut results = Vec::new();
        for point in search_result.result {
            if let Some(payload) = point.payload.get("text") {
                if let Some(text) = payload.as_str() {
                    results.push(text.to_string());
                }
            }
        }

        Ok(results)
    }
}
