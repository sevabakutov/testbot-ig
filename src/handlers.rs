use actix_web::{http::Method, web, HttpRequest, HttpResponse};
use crate::{api::{escalate, send_dm}, models::{OutgoingMessage, WebhookPayload}, utils::{is_escalated, verify_challenge, verify_signature}};

pub async fn instagram_dm_webhook(req: HttpRequest, body: web::Bytes) -> HttpResponse {
    match *req.method() {
        Method::GET  => verify_challenge(&req),

        Method::POST => {
            if let Err(resp) = verify_signature(&req, &body) {
                return resp;
            }

            let payload: WebhookPayload = match serde_json::from_slice(&body) {
                Ok(v)  => v,
                Err(e) => {
                    eprintln!("⚠️  Bad JSON: {e}\nPayload: {body:?}");
                    return HttpResponse::BadRequest().finish();
                }
            };
            println!("📨 {payload:#}");

            let sender = payload.primary_sender().unwrap();
            if is_escalated(sender) {
                println!("🚫 Chat {} is escalated, skipping bot reply", sender.id());
                return HttpResponse::Ok().finish();
            }

            if payload.is_bot_echo() {
                return HttpResponse::Ok().finish();
            }

            let recipient = match payload.primary_sender() {
                Some(s) => s.as_recipient(),
                None => return HttpResponse::BadRequest().finish(),
            };

            if payload.ready_to_escalate() {
                match escalate(recipient).await {
                    Ok(_) => return HttpResponse::Ok().finish(),
                    Err(err) => {
                        eprintln!("{err}");
                        return HttpResponse::BadRequest().finish();
                    }
                }
            }

            let message = OutgoingMessage::from("Welcome! (constant response)");

            match send_dm(recipient, message).await {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(e) => {
                    eprintln!("{e}");
                    HttpResponse::InternalServerError().finish()
                }
            }
        }

        _ => HttpResponse::MethodNotAllowed().finish(),
    }
}