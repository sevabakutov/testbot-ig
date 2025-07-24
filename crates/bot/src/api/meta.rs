use anyhow::{Context, Result};
use reqwest::Client;
use crate::{
    constants::{
        ACCESS_TOKEN, 
        DM_URL, IG_LIMIT
    }, 
    models::{
        OutgoingMessage, 
        Recipient, 
        SendBody
    }, 
    utils::mark_escalated
};

pub async fn send_to_ig_by_paragraphs(recipient: Recipient, text: &str) -> Result<()> {
    let mut dm_buf = String::new();
    let mut para_buf = String::new();

    for line in text.split_inclusive('\n') {
        let mut paragraph = line.trim_end_matches('\n').to_string();
        if paragraph.ends_with('\r') {
            paragraph.pop();
        }
        para_buf.push_str(&paragraph);

        if line.ends_with('\n') {
            send_paragraph(&mut dm_buf, &para_buf, &recipient).await?;
            para_buf.clear();
        }
    }
    if !para_buf.is_empty() {
        send_paragraph(&mut dm_buf, &para_buf, &recipient).await?;
    }
    if !dm_buf.is_empty() {
        send_dm(recipient.clone(), OutgoingMessage::from(dm_buf.as_str())).await?;
    }
    Ok(())
}

async fn send_paragraph(
    dm_buf: &mut String,
    paragraph: &str,
    recipient: &Recipient,
) -> Result<()> {
    if paragraph.len() > IG_LIMIT {
        if !dm_buf.is_empty() {
            send_dm(recipient.clone(), OutgoingMessage::from(dm_buf.as_str())).await?;
            dm_buf.clear();
        }
        for slice in paragraph.as_bytes().chunks(IG_LIMIT) {
            let txt = String::from_utf8_lossy(slice);
            send_dm(recipient.clone(), OutgoingMessage::from(txt.to_string().as_str())).await?;
        }
        return Ok(());
    }

    if dm_buf.len() + 1 + paragraph.len() > IG_LIMIT {
        send_dm(recipient.clone(), OutgoingMessage::from(dm_buf.as_str())).await?;
        dm_buf.clear();
    }

    if !dm_buf.is_empty() {
        dm_buf.push('\n');
    }
    dm_buf.push_str(paragraph);

    Ok(())
}

pub async fn send_dm<'a>(recipient: Recipient, message: OutgoingMessage<'a>) -> Result<()> {
    let body = SendBody::new(recipient, message);

    let resp = Client::new()
        .post(DM_URL)
        .query(&[("access_token", ACCESS_TOKEN.as_str())])
        .json(&body)
        .send()
        .await
        .context("Failed to send DM")?;

    // <<< debug-блок
    if !resp.status().is_success() {
        let status = resp.status();
        let text   = resp.text().await.unwrap_or_default();
        eprintln!("❗ send_dm failed: {status} — {text}");
        return Ok(());
    }
    // >>>

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
        .context("Failed to send DM")?
        .error_for_status()
        .context("Failed to get status")?;

    mark_escalated(&sender);

    Ok(())
}