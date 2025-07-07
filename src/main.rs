    use std::env;

    use actix_web::{http::StatusCode, web, App, HttpRequest, HttpResponse, HttpServer};

    async fn receive_messages(req: HttpRequest) -> HttpResponse {
        println!("Received HttpRequest: {:?}", req);
        HttpResponse::new(StatusCode::OK)
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
                        .route("/receive_messages", web::post().to(receive_messages))
                )
        })
        .bind(("0.0.0.0", port))?
        .run()
        .await
    }
