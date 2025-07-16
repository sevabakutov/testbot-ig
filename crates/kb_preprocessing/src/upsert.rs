use qdrant_client::{qdrant::{CreateCollectionBuilder, Distance, PointStruct, UpsertPointsBuilder, VectorParamsBuilder}, Qdrant};
use crate::constants::{DIM_SIZE, QDRANT_API_KEY, QDRANT_URL};

pub struct VectorStore {
    client: Qdrant,
    collection: String,
}

impl VectorStore {
    pub async fn new(collection: &str) -> anyhow::Result<Self> {
        let client = Qdrant::from_url(QDRANT_URL)
            .api_key(QDRANT_API_KEY.as_str())
            .build()?;

        if !client.collection_exists(collection).await? {
            client
                .create_collection(
                    CreateCollectionBuilder::new(collection)
                        .vectors_config(VectorParamsBuilder::new(DIM_SIZE, Distance::Cosine))
                )
                .await?;
        }

        Ok(Self { client, collection: collection.to_string() })
    }

    pub async fn upsert(&self, points: Vec<PointStruct>) -> anyhow::Result<()> {
        if points.len() as u64 != DIM_SIZE {
            anyhow::bail!("embedding dim mismatch: expected {} got {}", DIM_SIZE, points.len());
        }

        self.client
            .upsert_points(
                UpsertPointsBuilder::new(self.collection.as_str(), points)
                .wait(true)
            )
            .await?;

        Ok(())
    }
}