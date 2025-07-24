use std::sync::Arc;
use anyhow::{Context, Result};
use qdrant_client::{qdrant::{Condition, Filter, SearchPointsBuilder}, Qdrant};
use crate::constants::{QDRANT_API_KEY, QDRANT_COLLECTION_NAME, QDRANT_URL};

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

    pub async fn search_addresses(&self, vector: Vec<f32>, lang: String, country: String, city: String) -> Result<Vec<String>> {
        let request = SearchPointsBuilder::new(QDRANT_COLLECTION_NAME, vector, u64::MAX)
            .filter(Filter::must(vec![
                Condition::matches("topic", "address".to_string()),
                Condition::matches("lang", lang),
                Condition::matches("country", country),
                Condition::matches("city", city)
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

    pub async fn search_lazer_hair_removal_info(&self, vector: Vec<f32>, lang: String, hint: String) -> Result<Vec<String>> {
        let request = SearchPointsBuilder::new(QDRANT_COLLECTION_NAME, vector, 3)
            .filter(Filter::must(vec![
                Condition::matches("topic", "laser_epilation".to_string()),
                Condition::matches("lang", lang),
                Condition::matches("q_hint", hint)
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

    // pub async fn search_with_limit(
    //     &self,
    //     vector: Vec<f32>,
    //     limit: u64,
    //     lang: &str,
    // ) -> Result<Vec<String>> {
    //     let request = SearchPointsBuilder::new(QDRANT_COLLECTION_NAME, vector, limit)
    //         .filter(Filter::must(vec![Condition::matches("lang", lang.to_string())]))
    //         .with_payload(true)
    //         .build();

    //     let response = self.client.search_points(request).await?;

    //     let snippets = response
    //         .result
    //         .into_iter()
    //         .filter_map(|pt| {
    //             pt.payload
    //             .get("answer_text")
    //             .and_then(|v| v.as_str().map(ToString::to_string))
    //         })
    //         .collect();

    //     Ok(snippets)
    // }
}