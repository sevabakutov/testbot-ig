mod chunking;
mod normalize;
mod embedding;
mod upsert;

use embedding::Embedder;
use qdrant_client::{qdrant::PointStruct, Payload};
use serde_json::json;
use upsert::VectorStore;
use crate::{kb_preprocessing::{chunking::split_answer, normalize::normalize}, models::Model};

pub struct KBPipeline {
    embedder: Embedder,
    store: VectorStore,
    topic: String,
    lang: String,
    city: String,
}

impl KBPipeline {
    pub async fn new(collection: &str, topic: &str, lang: &str, city: &str) -> anyhow::Result<Self> {
        Ok(Self {
            embedder: Embedder::new(Model::TextEmbedding3Large),
            store: VectorStore::new(collection).await?,
            topic: topic.to_string(),
            lang: lang.to_string(),
            city: city.to_string(),
        })
    }

    // pub async fn process_row(&self, answer: &str, row_idx: usize) -> anyhow::Result<()> {
    //     for (chunk_idx, chunk) in split_answer(answer).into_iter().enumerate() {
    //         let clean = normalize(&chunk);
    //         let id = format!("{}_{}_{}", self.topic, row_idx, chunk_idx);
    //         let vector = self.embedder.embed(&clean).await?;
    //         let payload = Payload::try_from(json!({
    //             "topic": self.topic,
    //             "lang": self.lang,
    //             "city": self.city,
    //             "answer": clean,
    //             "embedding_model": self.embedder.model()
    //         }))
    //         .unwrap();
    //         let points = PointStruct::new(id, vector, payload);

    //         self.store.upsert(vector).await?;
    //     }
    //     Ok(())
    // }
}