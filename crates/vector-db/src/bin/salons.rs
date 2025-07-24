use anyhow::Result;
use qdrant_client::{qdrant::{CreateCollectionBuilder, Distance, PointStruct, UpsertPointsBuilder, VectorParamsBuilder}, Payload, Qdrant};
use serde_json::json;
use vector_db::{constants::{CONNECTION_STRING, VECTOR_SIZE}, tools::embed};

#[tokio::main]
async fn main() -> Result<()> {
    let collection_name = "addresses";
    dotenv::dotenv().ok();
    let client = Qdrant::from_url(CONNECTION_STRING).build()?;

    if !client.collection_exists(collection_name).await? {
        client.create_collection(
            CreateCollectionBuilder::new(collection_name)
                .vectors_config(VectorParamsBuilder::new(VECTOR_SIZE, Distance::Cosine))   
        )
        .await?;
    }

    // let addresses = vec![
    //     ("Краків", "вул. Mogilska 120B"),
    //     ("Вроцлав", "вул. Ignacego Daszyńskiego 19"),
    //     ("Варшава", "вул. Gizów 3"), 
    //     ("Варшава", "вул. Orzycka 6, LU5H"),
    //     ("Лодзь", "вул. Łagiewnicka 53 LU3.")
    // ];

    // let mut points = Vec::with_capacity(addresses.len());

    // for (i, (city, address)) in addresses.into_iter().enumerate() {
    //     let answer_text = format!("{} {}", city, address);
    //     let vector = embed(&answer_text).await?;
    //     let payload = Payload::try_from(json!({
    //         "lang": "ua",
    //         "topic": "address",
    //         "country": "Poland",
    //         "city": city,
    //         "answer_text": answer_text
    //     }))?;

    //     points.push(PointStruct::new(i as u64, vector, payload));
    // }

    let text_ua = r#"
- Краків:
    - вул. Mogilska 120B

- Вроцлав:
    - вул. Ignacego Daszyńskiego 19

- Варшава:
    - вул. Gizów 3
    - вул. Orzycka 6, LU5H

- Лодзь:
    - вул. Łagiewnicka 53 LU3.
"#;

    let vector = embed("Інформація про адреси в Польщі").await?;
    let paylaod = Payload::try_from(json!({
        "lang": "ua",
        "country": "Poland",
        "answer_text": text_ua
    }))?;

    let points = vec![
        PointStruct::new(2, vector, paylaod)
    ];
    client.upsert_points(UpsertPointsBuilder::new(collection_name, points).wait(true)).await?;

    println!("Адреса успешно сохранены в коллекции '{}'", collection_name);

    Ok(())
}