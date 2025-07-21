use actix_web::{http::Method, web, HttpRequest, HttpResponse};
use anyhow::Result;
use crate::*;
use api::{meta::stream_by_paragraph, openai::OpenAIClient};
use debouncer::Debouncer;
use memory::{InMemoryStore, MemoryStore};
use models::{Recipient, WebhookPayload};
use utils::{is_escalated, verify_challenge, verify_signature};
use vector_store::VectorStore;

/// Функция обработки сообщения из буфера. 
/// 
/// Идентифецирует язык => вызывает эмбидинг сообщения пользователя => поиск эмбидинга с фильтром языка в векторной бд => открытие стрима.
pub async fn process_merged_message(
    openai_client: &OpenAIClient,
    memory: &InMemoryStore,
    vstore: &VectorStore,
    recipient: Recipient,
    user_text: String,
) -> Result<()> {
    let chat_id = recipient.id();
    memory.push_user(chat_id, &user_text).await;

    let lang = openai_client.identify_language(user_text.as_str()).await?;
    let query_vec = openai_client.embed(user_text.clone()).await?;

    let snippets = vstore
        .search_with_limit(query_vec, 3, lang.as_str())
        .await?
        .iter()
        .enumerate()
        .map(|(i,s)| format!("{}. {}", i+1, s))
        .collect::<Vec<_>>()
        .join("\n\n");
        

    stream_by_paragraph(openai_client, memory, recipient, &user_text, &snippets).await
}

/// Инстаграм вэб хук.
/// 
/// Обрабатывает входящие сообщения и пушит их в дебаунсер. Возвращает ошибку при любой ошибке парсинга. 
/// чат уже эскалирован => Ок. 
/// эхо бота => Ок.
/// пустой пейлоад => Ок.
pub async fn instagram_dm_webhook(
    req: HttpRequest,
    body: web::Bytes,
    debounce: web::Data<Debouncer>,
) -> HttpResponse {
    match *req.method() {
        Method::GET => verify_challenge(&req),

        Method::POST => {
            if let Err(resp) = verify_signature(&req, &body) {
                return resp;
            }

            let payload: WebhookPayload = match serde_json::from_slice(&body) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("⚠️  Bad JSON: {e}\nPayload: {body:?}");
                    return HttpResponse::BadRequest().finish();
                }
            };
            println!("📨 {payload:#}");

            let sender = match payload.sender() {
                Some(s) => s,
                None => return HttpResponse::BadRequest().finish(),
            };
            let recipient = sender.as_recipient();
            let chat_id = sender.id();

            if is_escalated(&sender) {
                println!("🚫 Chat {chat_id} escalated, skip bot");
                return HttpResponse::Ok().finish();
            }
            if payload.is_bot_echo() {
                return HttpResponse::Ok().finish();
            }

            let incoming_text = match payload.text() {
                Some(t) => t,
                None => return HttpResponse::Ok().finish(),
            };

            debounce.push(recipient.id(), incoming_text).await;

            HttpResponse::Ok().finish()
        }

        _ => HttpResponse::MethodNotAllowed().finish(),
    }
}