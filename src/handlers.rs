use actix_web::{http::Method, web, HttpRequest, HttpResponse};

use crate::{
    api::{
        meta::{escalate, send_dm},
        openai::OpenAIClient,
    },
    memory::{InMemoryStore, MemoryStore},
    models::{OutgoingMessage, WebhookPayload},
    utils::{is_escalated, verify_challenge, verify_signature},
};

pub async fn instagram_dm_webhook(req: HttpRequest, body: web::Bytes) -> HttpResponse {
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
                None    => return HttpResponse::BadRequest().finish(),
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
                None    => return HttpResponse::Ok().finish(),
            };

            let memory = InMemoryStore;
            memory.push_user(chat_id, incoming_text).await;
            let history = memory.get(chat_id).await;

            if payload.wants_human() {
                if let Err(err) = escalate(recipient.clone()).await {
                    eprintln!("{err}");
                    return HttpResponse::InternalServerError().finish();
                }
                println!("🛎️ User requested human, escalated chat {chat_id}");
                return HttpResponse::Ok().finish();
            }

            let openai_client = OpenAIClient::new();
            let reply = match openai_client
                .send(OutgoingMessage::from(incoming_text), history)
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("OpenAI error: {e}");
                    return HttpResponse::InternalServerError().finish();
                }
            };

            if reply.trim() == "😊" {
                if let Err(err) = escalate(recipient.clone()).await {
                    eprintln!("{err}");
                    return HttpResponse::InternalServerError().finish();
                }
                println!("🛎️ Model escalated chat {chat_id}");
                return HttpResponse::Ok().finish();
            }

            println!("🤖 OpenAI reply: {reply}");
            if let Err(e) = send_dm(recipient.clone(), OutgoingMessage::from(reply.as_str())).await {
                eprintln!("{e}");
                return HttpResponse::InternalServerError().finish();
            }

            memory.push_assistant(chat_id, &reply).await;

            HttpResponse::Ok().finish()
        }

        _ => HttpResponse::MethodNotAllowed().finish(),
    }
}
