use actix_web::{http::Method, web, HttpRequest, HttpResponse};

use crate::{api::meta::escalate, debouncer::Debouncer, models::WebhookPayload, utils::{is_escalated, verify_challenge, verify_signature}};

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

            // "Нужен человек" — эскалация мгновенно, без модели.
            if payload.wants_human() {
                escalate(recipient.clone()).await;

                println!("🛎️ User requested human, escalated chat {chat_id}");
                
                return HttpResponse::Ok().finish();
            }

            let incoming_text = match payload.text() {
                Some(t) => t,
                None => return HttpResponse::Ok().finish(),
            };

            // Кладём текст в дебоунсер — дальше ответит фоновая задача.
            debounce.push(recipient.id(), incoming_text).await;

            HttpResponse::Ok().finish()
        }

        _ => HttpResponse::MethodNotAllowed().finish(),
    }
}