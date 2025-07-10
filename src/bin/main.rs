use std::env;
use actix_web::{web, App, HttpServer};
use testbot::handlers::instagram_dm_webhook;

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
                    .route("/instagram", web::to(instagram_dm_webhook))
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
