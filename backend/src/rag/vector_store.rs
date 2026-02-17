use anyhow::Result;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, VectorParamsBuilder,
    PointStruct, SearchPointsBuilder,
    ScrollPointsBuilder, PointsIdsList,
    point_id::PointIdOptions, DeletePointsBuilder,
};
use serde_json::{Map as JsonMap, Value as JsonValue};

pub struct VectorStore {
    client: Qdrant,
    collection_name: String,
}

impl VectorStore {
    pub async fn new(url: &str, collection_name: &str) -> Result<Self> {
        tracing::info!("Building Qdrant client for URL: {}", url);
        let client = match Qdrant::from_url(url).build() {
            Ok(c) => {
                tracing::info!("Qdrant client built successfully");
                c
            }
            Err(e) => {
                tracing::error!("Qdrant client build failed: {:?}", e);
                anyhow::bail!("Qdrant client build failed: {}", e);
            }
        };

        let store = Self {
            client,
            collection_name: collection_name.to_string(),
        };

        tracing::info!("Checking Qdrant collection...");
        if let Err(e) = store.ensure_collection().await {
            tracing::error!("Qdrant ensure_collection failed: {:?}", e);
            return Err(e);
        }
        tracing::info!("Qdrant collection ready");

        Ok(store)
    }

    async fn ensure_collection(&self) -> Result<()> {
        if !self.client.collection_exists(&self.collection_name).await? {
            self.client
                .create_collection(
                    CreateCollectionBuilder::new(&self.collection_name)
                        .vectors_config(VectorParamsBuilder::new(384, Distance::Cosine)),
                )
                .await?;
        }
        Ok(())
    }

    pub async fn add_document(
        &self,
        id: &str,
        text: &str,
        embedding: Vec<f32>,
        metadata: serde_json::Value,
    ) -> Result<()> {
        let mut payload_map = JsonMap::new();
        payload_map.insert("text".to_string(), JsonValue::String(text.to_string()));
        payload_map.insert("metadata".to_string(), metadata);
        let point = PointStruct::new(id.to_string(), embedding, payload_map);

        self.client
            .upsert_points(
                qdrant_client::qdrant::UpsertPointsBuilder::new(&self.collection_name, vec![point]),
            )
            .await?;

        Ok(())
    }

    pub async fn search(&self, query_vector: Vec<f32>, limit: u64) -> Result<Vec<String>> {
        let search_result = self
            .client
            .search_points(
                SearchPointsBuilder::new(&self.collection_name, query_vector, limit)
                    .with_payload(true),
            )
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

    pub async fn scroll_all_point_ids(&self) -> Result<Vec<String>> {
        let mut all_ids = Vec::new();
        let mut offset: Option<qdrant_client::qdrant::PointId> = None;

        loop {
            let mut builder = ScrollPointsBuilder::new(&self.collection_name)
                .limit(100)
                .with_payload(false);

            if let Some(ref off) = offset {
                builder = builder.offset(off.clone());
            }

            let result = self.client.scroll(builder).await?;

            for point in &result.result {
                if let Some(ref id) = point.id {
                    if let Some(ref id_options) = id.point_id_options {
                        match id_options {
                            PointIdOptions::Uuid(uuid) => all_ids.push(uuid.clone()),
                            PointIdOptions::Num(num) => all_ids.push(num.to_string()),
                        }
                    }
                }
            }

            offset = result.next_page_offset;
            if offset.is_none() {
                break;
            }
        }

        Ok(all_ids)
    }

    pub async fn delete_points(&self, ids: Vec<String>) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let point_ids: Vec<qdrant_client::qdrant::PointId> = ids
            .into_iter()
            .map(|id| qdrant_client::qdrant::PointId {
                point_id_options: Some(PointIdOptions::Uuid(id)),
            })
            .collect();

        self.client
            .delete_points(
                DeletePointsBuilder::new(&self.collection_name)
                    .points(PointsIdsList { ids: point_ids }),
            )
            .await?;

        Ok(())
    }
}
