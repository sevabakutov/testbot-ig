use anyhow::{Context, Result};
use async_openai::{config::OpenAIConfig, types::{CreateEmbeddingRequest, EmbeddingInput}, Client};

use crate::constants::VECTOR_SIZE;

pub async fn embed(text: &str) -> Result<Vec<f32>> {
    let config = OpenAIConfig::default();
    let client = Client::with_config(config);

    let response = client.embeddings().create(CreateEmbeddingRequest {
        model: "text-embedding-3-large".to_string(),
        input: EmbeddingInput::String(text.to_string()),
        dimensions: Some(VECTOR_SIZE as u32),
        ..Default::default()
    })
    .await
    .context("Failed to embed")?;

    Ok(response.data[0].embedding.clone())
}