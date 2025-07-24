use std::sync::Arc;
use anyhow::{Context, Result};
use qdrant_client::{qdrant::{Condition, Filter, SearchPointsBuilder}, Qdrant};
use crate::constants::{QDRANT_API_KEY, QDRANT_URL};

#[derive(Clone)]
pub struct VectorStore {
    client: Arc<Qdrant>,
}

impl VectorStore {
    pub fn new() -> Result<Self> {
        let client = Qdrant::from_url(QDRANT_URL)
            .api_key(QDRANT_API_KEY.as_str())
            .build()
            .context("Failed to build qdrant client")?;

        Ok(Self {
            client: Arc::new(client)
        })
    }

    pub async fn search_addresses(&self, vector: Vec<f32>, lang: String, country: String) -> Result<Vec<String>> {
        let request = SearchPointsBuilder::new("addresses", vector, u64::MAX)
            .filter(Filter::must(vec![
                Condition::matches("lang", lang),
                Condition::matches("country", country)
            ]))
            .with_payload(true)
            .build();

        let response = self.client.search_points(request).await?;

        let addresses = response
            .result
            .into_iter()
            .filter_map(|sp| {
                sp
                    .payload
                    .get("answer_text")
                    .and_then(|v| v.as_str().map(ToString::to_string))
            })
            .collect();

        Ok(addresses)
    }

    pub async fn search_lazer_hair_removal_info(&self, vector: Vec<f32>, lang: String) -> Result<Vec<String>> {
        let request = SearchPointsBuilder::new("lazer_hair_removal", vector, 3)
            .filter(Filter::must(vec![
                Condition::matches("topic", "lazer hair removal".to_string()),
                Condition::matches("lang", lang),
            ]))
            .with_payload(true)
            .build();

        let response  = self.client.search_points(request).await?;

        let infos = response
            .result
            .into_iter()
            .filter_map(|sp| {
                sp
                    .payload
                    .get("answer_text")
                    .and_then(|v| v.as_str().map(ToString::to_string))
            })
            .collect();

        Ok(infos)
    }
}