use async_openai::types::CreateEmbeddingRequestArgs;
use crate::{api::openai::OpenAIClient, models::Model};

pub struct Embedder {
    client: OpenAIClient,
}

impl Embedder {
    pub fn new(model: Model) -> Self {
        Self {
            client: OpenAIClient::new(model),
        }
    }

    /// Возвращает 1 536‑мерный вектор для переданного текста.
    pub async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let req = CreateEmbeddingRequestArgs::default()
            .model(self.client.model())
            .input(text)
            .build()?;

        let resp = self.client
            .embeddings()
            .create(req)
            .await?;
        
        Ok(resp.data[0].embedding.clone())
    }

    pub fn model(&self) -> &str {
        self.client.model()
    }
}