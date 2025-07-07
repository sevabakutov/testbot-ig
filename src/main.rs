use std::env;

use actix_web::{http::{Method, StatusCode}, web, App, HttpRequest, HttpResponse, HttpServer};

async fn instagram_webhook(req: HttpRequest, _body: web::Bytes) -> HttpResponse {
    let verify_token = env::var("TOKEN").expect("Failed to get TOKEN variable");

    match *req.method() {
        Method::GET => {
            let qs = req.query_string();

            let mut mode = String::new();
            let mut token = String::new();
            let mut challenge = String::new();

            for (key, value) in url::form_urlencoded::parse(qs.as_bytes()) {
                match key.as_ref() {
                    "hub.mode"         => mode      = value.into_owned(),
                    "hub.verify_token" => token     = value.into_owned(),
                    "hub.challenge"    => challenge = value.into_owned(),
                    _ => {}
                }
            }

            if mode == "subscribe" && token == verify_token {
                println!("✅ Instagram webhook verified");
                HttpResponse::Ok().body(challenge)
            } else {
                println!("❌ Verification failed: token mismatch");
                HttpResponse::build(StatusCode::FORBIDDEN).finish()
            }
        }

        // Method::POST => {
        //     let json: Value = match serde_json::from_slice(&body) {
        //         Ok(v) => v,
        //         Err(e) => {
        //             println!("⚠️  Bad JSON: {}", e);
        //             return HttpResponse::BadRequest().finish();
        //         }
        //     };

        //     println!("📨 New Instagram event: {:#}", json);

        //     HttpResponse::Ok().finish()
        // }

        _ => HttpResponse::MethodNotAllowed().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/app")
                    .route("/instagram", web::to(instagram_webhook))
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
