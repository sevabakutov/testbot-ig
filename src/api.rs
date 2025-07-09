use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError},
    Result,
};
use reqwest::Client;
use std::env;

use crate::models::{OutgoingMessage, Recipient, SendBody};

pub async fn send_dm<'a>(recipient: Recipient, message: OutgoingMessage<'a>) -> Result<()> {
    let token = env::var("ACCESS_TOKEN")
        .map_err(|_| ErrorBadRequest("ACCESS_TOKEN env-var is missing"))?;

    let url  = "https://graph.instagram.com/v21.0/me/messages";
    let body = SendBody::new(recipient, message);

    Client::new()
        .post(url)
        .query(&[("access_token", token)])
        .json(&body)
        .send()
        .await
        .map_err(ErrorInternalServerError)?
        .error_for_status()
        .map_err(ErrorInternalServerError)?;

    Ok(())
}
