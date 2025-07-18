use std::{env, time::Duration};

use actix_web::{web, App, HttpServer};

use bot::{
    api::openai::OpenAIClient,
    debouncer::Debouncer,
    handlers::{instagram_dm_webhook, process_merged_message},
    memory::InMemoryStore,
    models::Model,
};

/// Сколько ждать после последнего входящего символа, прежде чем "слить" буфер.
const WINDOW_MS: u64 = 4500;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    // let qdrant = VectorStore::new("http://localhost:6334")
    //     .await
    //     .expect("Qdrant init failed");

    let debounce = Debouncer::new(Duration::from_millis(WINDOW_MS));
    let openai_client = OpenAIClient::new(Model::GPT41mini);
    let memory = InMemoryStore;

    // --- фоновая задача, которая склеивает сообщения и зовёт модель ---
    {
        let debounce_clone = debounce.clone();
        let openai_clone = openai_client.clone();
        tokio::spawn(async move {
            debounce_clone
                .run(move |recipient, merged_text| {
                    let openai = openai_clone.clone();
                    let memory = memory.clone();
                    // let vstore  = vstore.clone();
                    tokio::spawn(async move {
                        process_merged_message(&openai, &memory, recipient, merged_text).await
                        // process_merged_message(&openai, &memory, &vstore, recipient, merged_text).await
                    });
                })
                .await;
        });
    }

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(debounce.clone()))
            // .app_data(web::Data::new(qdrant.clone()))
            .route("/app/instagram", web::to(instagram_dm_webhook))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

