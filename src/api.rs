use actix_web::{
    error::ErrorInternalServerError,
    Result,
};
use reqwest::Client;

use crate::{constants::{ACCESS_TOKEN, DM_URL}, models::{OutgoingMessage, Recipient, SendBody}};

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
