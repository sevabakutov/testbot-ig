use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde_json::json;
use crate::{constants::{ACCESS_TOKEN, DM_URL}, models::{OutgoingMessage, Recipient, SendBody}, utils::mark_escalated, vector_store::VectorStore};

pub async fn get_salons(
    vstore: &VectorStore,
    vector: Vec<f32>,
    lang: String,
    country: String
) -> Result<String> {
    Ok(vstore
        .search_addresses(vector, lang, country)
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
) -> Result<String> {
    Ok(vstore
        .search_lazer_hair_removal_info(vector, lang)
        .await?
        .into_iter()
        .enumerate()
        .map(|(i, info)| format!("{}. {}", i + 1, info))
        .collect::<Vec<_>>()
        .join("\n\n"))
}

pub async fn escalate(recipient: Recipient) -> Result<()> {
    let sender = recipient.as_sender();
    let message = OutgoingMessage::from("Минуточку...");
    let body = SendBody::new(recipient, message);

    Client::new()
        .post(DM_URL)
        .query(&[("access_token", ACCESS_TOKEN.as_str())])
        .json(&body)
        .send()
        .await
        .context("Failed to send DM")?
        .error_for_status()
        .context("Failed to get status")?;

    mark_escalated(&sender);

    Ok(())
}

pub async fn call_fn(vstore: &VectorStore, name: &str, args: &str) -> Result<serde_json::Value> {
    let function_args: serde_json::Value = serde_json::from_str(args)?;

    match name {
        "get_salons" => {
            let vector: Vec<f32> = serde_json::from_value(function_args["vector"].clone())?;
            let lang: String = serde_json::from_value(function_args["lang"].clone())?;
            let country: String = serde_json::from_value(function_args["country"].clone())?;
            let result = get_salons(vstore, vector, lang, country).await?;

            Ok(json!(result))
        }
        "get_lazer_hair_removal_info" => {
            let vector: Vec<f32> = serde_json::from_value(function_args["vector"].clone())?;
            let lang: String = serde_json::from_value(function_args["lang"].clone())?;
            let result = get_lazer_hair_removal_info(vstore, vector, lang).await?;

            Ok(json!(result))
        }
        "escalate" => {
            let recipient_id: String = serde_json::from_value(function_args["recipient_id"].clone())?;
            let recipient = Recipient::new(recipient_id);
            escalate(recipient).await?;

            Ok(json!({}))
        }
        _ => Err(anyhow!("Unknown function: {}", name)),
    }
}