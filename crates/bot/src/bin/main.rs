use std::{env, time::Duration};

use actix_web::{web, App, HttpServer};

use bot::{
    api::{
        meta::{stream_by_paragraph},
        openai::OpenAIClient,
    },
    debouncer::Debouncer,
    handlers::instagram_dm_webhook,
    memory::{InMemoryStore, MemoryStore},
    models::{Model, Recipient},
};

/// Сколько ждать после последнего входящего символа, прежде чем "слить" буфер.
const WINDOW_MS: u64 = 4500;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    let debounce = Debouncer::new(Duration::from_millis(WINDOW_MS));
    let openai_client = OpenAIClient::new(Model::GPT41MiniFineTuned);
    let memory = InMemoryStore;

    // --- фоновая задача, которая склеивает сообщения и зовёт модель ---
    {
        let debounce_clone = debounce.clone();
        let openai_clone = openai_client.clone();
        actix_web::rt::spawn(async move {
            debounce_clone
                .run(move |recipient, merged_text| {
                    let openai = openai_clone.clone();
                    let memory = memory.clone();
                    actix_web::rt::spawn(async move {
                        process_merged_message(&openai, &memory, recipient, merged_text).await;
                    });
                })
                .await;
        });
    }

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(debounce.clone()))
            .route("/app/instagram", web::to(instagram_dm_webhook))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

async fn process_merged_message(
    openai_client: &OpenAIClient,
    memory: &InMemoryStore,
    recipient: Recipient,
    user_text: String,
) {
    let chat_id = recipient.id();
    memory.push_user(chat_id, &user_text).await;

    if let Err(e) = stream_by_paragraph(openai_client, memory, recipient.clone(), &user_text).await {
        eprintln!("stream‑by‑paragraph error: {e}");
    }
}
// async fn process_merged_message(
//     openai_client: &OpenAIClient,
//     memory: &InMemoryStore,
//     recipient: Recipient,
//     user_text: String,
// ) {
//     let chat_id = recipient.id();

//     // 1. Обновляем историю
//     memory.push_user(chat_id, &user_text).await;
//     let history = memory.get(chat_id).await;

//     // 2. Вызов модели
//     let reply = match openai_client
//         .send(OutgoingMessage::from(user_text.as_str()), history)
//         .await
//     {
//         Ok(r) => r,
//         Err(e) => {
//             eprintln!("OpenAI error: {e}");
//             return;
//         }
//     };

//     // 3. Эскалация по триггеру
//     if reply.trim() == "😊" {
//         if let Err(err) = escalate(recipient.clone()).await {
//             eprintln!("{err}");
//         }
//         println!("🛎️ Model escalated chat {chat_id}");
//         return;
//     }

//     // 4. Отправляем ответ пользователю
//     if let Err(e) = send_dm(recipient.clone(), OutgoingMessage::from(reply.replace("\"", "").as_str())).await {
//         eprintln!("{e}");
//     }

//     // 5. Записываем ответ ассистента в память
//     memory.push_assistant(chat_id, &reply).await;
// }