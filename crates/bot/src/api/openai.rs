use std::sync::Arc;

use actix_web::{error::ErrorInternalServerError, Result};
use async_openai::{
    config::OpenAIConfig,
    types::*,
    Client,
    Embeddings,
};

use crate::{
    constants::OPENAI_PROJECT_ID,
    models::{Model, OutgoingMessage},
};

#[derive(Clone)]
pub struct OpenAIClient {
    client: Arc<Client<OpenAIConfig>>,
    model: Model,
}

impl OpenAIClient {
    pub fn new(model: Model) -> Self {
        let config = OpenAIConfig::new().with_project_id(OPENAI_PROJECT_ID.as_str());
        let client = Client::with_config(config);
        Self {
            client: Arc::new(client),
            model,
        }
    }

    /// Доступ к embeddings‑эндпоинту.
    pub fn embeddings(&self) -> Embeddings<'_, OpenAIConfig> {
        self.client.embeddings()
    }

    /// Короткий геттер id модели.
    pub fn model(&self) -> &str {
        &self.model.id()
    }

    /// Обычный (не‑стриминговый) вызов Chat Completions.
    pub async fn send(
        &self,
        msg: OutgoingMessage<'_>,
        mut history: Vec<ChatCompletionRequestMessage>,
    ) -> Result<String> {
        // ——— системные промпты ———
        const SYSTEM_CONTENT: &str = include_str!("prompts/system.txt");
        const DEV_CONTENT: &str = include_str!("prompts/developer.txt");

        history.splice(
            0..0,
            [
                ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::from(SYSTEM_CONTENT),
                    name: None,
                }),
                ChatCompletionRequestMessage::Developer(ChatCompletionRequestDeveloperMessage {
                    content: ChatCompletionRequestDeveloperMessageContent::from(DEV_CONTENT),
                    name: None,
                }),
            ],
        );

        history.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::from(msg.text()),
                name: None,
            },
        ));

        let request = CreateChatCompletionRequestArgs::default()
            .model(self.model())
            .messages(history)
            .build()
            .unwrap();

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(ErrorInternalServerError)?;

        Ok(response
            .choices
            .get(0)
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| ErrorInternalServerError("empty response"))?)
    }

    // /// Вариант со стримингом (SSE). Возвращает итоговую строку,
    // /// но при желании можно передавать чанки наружу в замыкание‑callback.
    // pub async fn send_stream(
    //     &self,
    //     msg: OutgoingMessage<'_>,
    //     mut history: Vec<ChatCompletionRequestMessage>,
    // ) -> Result<String> {
    //     history.splice(
    //         0..0,
    //         [
    //             ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
    //                 content: ChatCompletionRequestSystemMessageContent::from(
    //                     include_str!("prompts/system.txt"),
    //                 ),
    //                 name: None,
    //             }),
    //             ChatCompletionRequestMessage::Developer(ChatCompletionRequestDeveloperMessage {
    //                 content: ChatCompletionRequestDeveloperMessageContent::from(
    //                     include_str!("prompts/developer.txt"),
    //                 ),
    //                 name: None,
    //             }),
    //         ],
    //     );
    //     history.push(ChatCompletionRequestMessage::User(
    //         ChatCompletionRequestUserMessage {
    //             content: ChatCompletionRequestUserMessageContent::from(msg.text()),
    //             name: None,
    //         },
    //     ));

    //     let request = CreateChatCompletionRequestArgs::default()
    //         .model(self.model())
    //         .messages(history)
    //         .stream(true)
    //         .build()
    //         .unwrap();

    //     let mut stream = self
    //         .client
    //         .chat()
    //         .create_stream(request)
    //         .await
    //         .map_err(ErrorInternalServerError)?;

    //     let mut answer = String::new();
    //     while let Some(chunk) = stream.
        
    //     .await.transpose().map_err(ErrorInternalServerError)? {
    //         if let Some(delta) = chunk.choices[0].delta.content.clone() {
    //             answer.push_str(&delta);
    //         }
    //     }
    //     Ok(answer)
    // }
}