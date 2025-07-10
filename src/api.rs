use actix_web::{
    error::ErrorInternalServerError,
    Result,
};
use reqwest::Client;

use crate::{constants::{ACCESS_TOKEN, DM_URL}, models::{OutgoingMessage, Recipient, SendBody}, utils::mark_escalated};

pub async fn send_dm<'a>(recipient: Recipient, message: OutgoingMessage<'a>) -> Result<()> {
    let body = SendBody::new(recipient, message);

    Client::new()
        .post(DM_URL)
        .query(&[("access_token", ACCESS_TOKEN.as_str())])
        .json(&body)
        .send()
        .await
        .map_err(ErrorInternalServerError)?
        .error_for_status()
        .map_err(ErrorInternalServerError)?;

    Ok(())
}

pub async fn escalate(recipient: Recipient) -> Result<()> {
    let sender = recipient.as_sender();
    let message = OutgoingMessage::from("😊");
    let body = SendBody::new(recipient, message);

    Client::new()
        .post(DM_URL)
        .query(&[("access_token", ACCESS_TOKEN.as_str())])
        .json(&body)
        .send()
        .await
        .map_err(ErrorInternalServerError)?
        .error_for_status()
        .map_err(ErrorInternalServerError)?;

    mark_escalated(&sender);

    Ok(())
}