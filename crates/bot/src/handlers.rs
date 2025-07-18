use actix_web::{http::Method, web, HttpRequest, HttpResponse};
use anyhow::Result;
use crate::{api::{meta::stream_by_paragraph, openai::OpenAIClient}, debouncer::Debouncer, memory::{InMemoryStore, MemoryStore}, models::{Recipient, WebhookPayload}, utils::{is_escalated, verify_challenge, verify_signature}};

pub async fn process_merged_message(
    openai_client: &OpenAIClient,
    memory: &InMemoryStore,
    recipient: Recipient,
    user_text: String,
) -> Result<()> {
    let chat_id = recipient.id();
    memory.push_user(chat_id, &user_text).await;

    // let query_vec = openai_client.embed(user_text.clone()).await?;
    // let lang = if user_text.contains('ł') || user_text.contains("wizyt") { "pl" } else { "uk" };
    // let snippets = vstore.search(query_vec, lang, 3).await?;  
    // let memory_block = snippets
    //     .iter()
    //     .enumerate()
    //     .map(|(i,s)| format!("{}. {}", i+1, s))
    //     .collect::<Vec<_>>()
    //     .join("\n\n");

    // stream_by_paragraph(openai_client, memory, recipient, &user_text, &memory_block).await
    stream_by_paragraph(openai_client, memory, recipient.clone(), &user_text).await
}

pub async fn instagram_dm_webhook(
    req: HttpRequest,
    body: web::Bytes,
    debounce: web::Data<Debouncer>,
    // vstore: web::Data<VectorStore>,
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