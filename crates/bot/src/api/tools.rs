use anyhow::{anyhow, Result};
use serde_json::json;
use crate::vector_store::VectorStore;

pub async fn get_salons(
    vstore: &VectorStore,
    vector: Vec<f32>,
    lang: String,
    country: String,
    city: String,
) -> Result<String> {
    Ok(vstore
        .search_addresses(vector, lang, country, city)
        .await?
        .into_iter()
        .enumerate()
        .map(|(i, s)| format!("{}. {}", i + 1, s))
        .collect::<Vec<_>>()
        .join("\n\n"))
}

pub async fn get_lazer_hair_removal_info(
    vstore: &VectorStore,
    vector: Vec<f32>,
    lang: String,
    hint: String
) -> Result<String> {
    Ok(vstore
        .search_lazer_hair_removal_info(vector, lang, hint)
        .await?
        .into_iter()
        .enumerate()
        .map(|(i, info)| format!("{}. {}", i + 1, info))
        .collect::<Vec<_>>()
        .join("\n\n"))
}

pub async fn call_fn(vstore: &VectorStore, name: &str, args: &str) -> Result<serde_json::Value> {
    let function_args: serde_json::Value = serde_json::from_str(args)?;

    match name {
        "get_salons" => {
            let vector: Vec<f32> = serde_json::from_value(function_args["vector"].clone())?;
            let lang: String = serde_json::from_value(function_args["lang"].clone())?;
            let country: String = serde_json::from_value(function_args["country"].clone())?;
            let city: String = serde_json::from_value(function_args["city"].clone())?;
            let result = get_salons(vstore, vector, lang, country, city).await?;

            Ok(json!(result))
        }
        "get_lazer_hair_removal_info" => {
            let vector: Vec<f32> = serde_json::from_value(function_args["vector"].clone())?;
            let lang: String = serde_json::from_value(function_args["lang"].clone())?;
            let hint: String = serde_json::from_value(function_args["hint"].clone())?;
            let result = get_lazer_hair_removal_info(vstore, vector, lang, hint).await?;

            Ok(json!(result))
        }
        _ => Err(anyhow!("Unknown function: {}", name)),
    }
}