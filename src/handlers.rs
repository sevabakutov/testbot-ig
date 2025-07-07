use actix_web::{http::Method, web, HttpRequest, HttpResponse};
use crate::{models::WebhookPayload, utils::{verify_challenge, verify_signature}};

pub async fn instagram_webhook(req: HttpRequest, body: web::Bytes) -> HttpResponse {
    match *req.method() {
        Method::GET => verify_challenge(&req),
        Method::POST => {
            if let Err(resp) = verify_signature(&req, &body) {
                return resp;
            }

            let json = match serde_json::from_slice::<WebhookPayload>(&body) {
                Ok(v) => v,
                Err(e) => {
                    println!("⚠️  Bad JSON: {}", e);
                    return HttpResponse::BadRequest().finish();
                }
            };

            println!("📨 New Instagram event: {:#}", json);

            HttpResponse::Ok().finish()
        }

        _ => HttpResponse::MethodNotAllowed().finish(),
    }
}