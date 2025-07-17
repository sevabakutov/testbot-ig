use actix_web::{
    error::ErrorInternalServerError
};
use reqwest::Client;
use tokio_stream::StreamExt;

use crate::{api::openai::OpenAIClient, constants::{ACCESS_TOKEN, DM_URL}, memory::{InMemoryStore, MemoryStore}, models::{OutgoingMessage, Recipient, SendBody}, utils::mark_escalated};

const IG_LIMIT: usize = 950;

pub async fn process_merged_message(
    openai_client: &OpenAIClient,
    memory: &InMemoryStore,
    recipient: Recipient,
    user_text: String,
) {
    let chat_id = recipient.id();
    memory.push_user(chat_id, &user_text).await;

    stream_by_paragraph(openai_client, memory, recipient.clone(), &user_text).await;
}

/// Стримим ответ модели, режем по абзацам и шлём DM ≤950. Сохраняем полный ответ в историю.
pub async fn stream_by_paragraph<M: MemoryStore>(
    openai: &OpenAIClient,
    memory: &M,
    recipient: Recipient,
    user_text: &str,
) {
    let chat_id = recipient.id();
    let history = memory.get(chat_id).await;
    let request = openai.prepare_stream_request(user_text, history).unwrap();

    let mut stream = openai
        .client()
        .chat()
        .create_stream(request)
        .await
        .map_err(ErrorInternalServerError)
        .unwrap();

    let mut para_buf = String::new();
    let mut dm_buf = String::new();
    let mut full_answer = String::new();

    while let Some(chunk) = stream.next().await.transpose().map_err(ErrorInternalServerError).unwrap() {
        if let Some(delta) = chunk.choices[0].delta.content.clone() {
            para_buf.push_str(&delta);
            full_answer.push_str(&delta);

            // ищем абзац‑разделители
            while let Some(pos) = para_buf.find('\n') {
                let mut paragraph = para_buf[..pos].to_string();

                if paragraph.ends_with('\r') { 
                    paragraph.pop(); 
                }
                
                para_buf.drain(..=pos);
                
                send_paragraph(&mut dm_buf, &paragraph, &recipient).await;
            }
        }
    }

    if !para_buf.is_empty() {
        send_paragraph(&mut dm_buf, &para_buf, &recipient).await;
    }
    if !dm_buf.is_empty() {
        send_dm(recipient.clone(), OutgoingMessage::from(dm_buf.as_str())).await
    }

    // *Эскалация* по триггеру «😊». Если сработала — прерываемся.
    if full_answer.trim() == "😊" {
        escalate(recipient.clone()).await;

        println!("🛎️ Model escalated chat {chat_id}");

        return;
    }

    // сохраняем полный ответ ассистента
    memory.push_assistant(chat_id, &full_answer).await;
}

async fn send_paragraph(
    dm_buf: &mut String,
    paragraph: &str,
    recipient: &Recipient,
) {
    if paragraph.len() > IG_LIMIT {
        if !dm_buf.is_empty() {
            send_dm(recipient.clone(), OutgoingMessage::from(dm_buf.as_str())).await;
            dm_buf.clear();
        }
        for slice in paragraph.as_bytes().chunks(IG_LIMIT) {
            let txt = String::from_utf8_lossy(slice);
            send_dm(recipient.clone(), OutgoingMessage::from(txt.to_string().as_str())).await;
        }
        return;
    }

    if dm_buf.len() + 1 + paragraph.len() > IG_LIMIT {
        send_dm(recipient.clone(), OutgoingMessage::from(dm_buf.as_str())).await;
        dm_buf.clear();
    }

    if !dm_buf.is_empty() {
        dm_buf.push('\n');
    }
    dm_buf.push_str(paragraph);
}

pub async fn send_dm<'a>(recipient: Recipient, message: OutgoingMessage<'a>) {
    let body = SendBody::new(recipient, message);

    let resp = Client::new()
        .post(DM_URL)
        .query(&[("access_token", ACCESS_TOKEN.as_str())])
        .json(&body)
        .send()
        .await
        .map_err(ErrorInternalServerError)
        .unwrap();

    // <<< debug-блок
    if !resp.status().is_success() {
        let status = resp.status();
        let text   = resp.text().await.unwrap_or_default();
        eprintln!("❗ send_dm failed: {status} — {text}");
        return;
    }
    // >>>
}

pub async fn escalate(recipient: Recipient) {
    let sender = recipient.as_sender();
    let message = OutgoingMessage::from("😊");
    let body = SendBody::new(recipient, message);

    Client::new()
        .post(DM_URL)
        .query(&[("access_token", ACCESS_TOKEN.as_str())])
        .json(&body)
        .send()
        .await
        .map_err(ErrorInternalServerError)
        .unwrap()
        .error_for_status()
        .map_err(ErrorInternalServerError)
        .unwrap();

    mark_escalated(&sender);
}