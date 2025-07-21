use std::{env, time::Duration};
use actix_web::{web, App, HttpServer};
use anyhow::{Context, Result};
use bot::{
    api::openai::OpenAIClient, 
    constants::WINDOW_MS, 
    debouncer::Debouncer, 
    handlers::{
        instagram_dm_webhook, 
        process_merged_message
    }, 
    memory::InMemoryStore, 
    vector_store::VectorStore
};

#[tokio::main]
async fn main() -> Result<()> {
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    let vstore = VectorStore::new().context("Faled to build vector store")?;
    let debounce = Debouncer::new(Duration::from_millis(WINDOW_MS));
    let openai_client = OpenAIClient::new();
    let memory = InMemoryStore;

    // --- фоновая задача, которая склеивает сообщения и зовёт модель ---
    {
        let debounce_clone = debounce.clone();
        let openai_clone = openai_client.clone();
        let vstore_clone = vstore.clone();
        tokio::spawn(async move {
            debounce_clone
                .run(move |recipient, merged_text| {
                    let openai = openai_clone.clone();
                    let memory = memory.clone();
                    let vstore = vstore_clone.clone();
                    tokio::spawn(async move {
                        process_merged_message(&openai, &memory, &vstore, recipient, merged_text).await
                    });
                })
                .await;
        });
    }

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(debounce.clone()))
            // .app_data(web::Data::new(vstore.clone()))
            .route("/app/instagram", web::to(instagram_dm_webhook))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}

