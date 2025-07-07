use actix_web::{http::StatusCode, web, App, HttpRequest, HttpResponse, HttpServer};

async fn receive_messages(req: HttpRequest) -> HttpResponse {
    println!("Received HttpRequest: {:?}", req);
    HttpResponse::new(StatusCode::OK)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/app")
                    .route("/receive_messages", web::post().to(receive_messages))
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
