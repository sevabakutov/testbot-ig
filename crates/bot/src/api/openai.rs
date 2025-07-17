use std::sync::Arc;

use actix_web::{error::ErrorInternalServerError, Result};
use async_openai::{
    config::OpenAIConfig,
    types::*,
    Client,
    Embeddings,
};
use tokio_stream::StreamExt;

use crate::{
    constants::OPENAI_PROJECT_ID,
    models::{Model, OutgoingMessage},
};

#[derive(Clone)]
pub struct OpenAIClient {
    client: Arc<Client<OpenAIConfig>>, // shared HTTP pool
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

    /// Доступ к «сырым» методам клиента (нужно для внешнего стрима).
    pub fn client(&self) -> &Client<OpenAIConfig> { &self.client }

    pub fn embeddings(&self) -> Embeddings<'_, OpenAIConfig> { self.client.embeddings() }
    pub fn model(&self) -> &str { &self.model.id() }

    // -------- обычный не‑стриминговый вызов
    pub async fn send(
        &self,
        msg: OutgoingMessage<'_>,
        mut history: Vec<ChatCompletionRequestMessage>,
    ) -> Result<String> {
        self.inject_system_messages(&mut history);
        history.push(ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::from(msg.text()),
            name: None,
        }));

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

    // -------- стриминг: готовим запрос, возвращаем итоговую строку --------
    pub async fn send_stream(
        &self,
        msg: OutgoingMessage<'_>,
        history: Vec<ChatCompletionRequestMessage>,
    ) -> Result<String> {
        let request = self.prepare_stream_request(msg.text(), history)?;
        let mut stream = self
            .client
            .chat()
            .create_stream(request)
            .await
            .map_err(ErrorInternalServerError)?;

        let mut answer = String::new();
        while let Some(chunk) = stream.next().await.transpose().map_err(ErrorInternalServerError)? {
            if let Some(delta) = chunk.choices[0].delta.content.clone() {
                answer.push_str(&delta);
            }
        }
        Ok(answer)
    }

    // -------- helper‑ы --------
    pub fn prepare_stream_request(
        &self,
        user_text: &str,
        mut history: Vec<ChatCompletionRequestMessage>,
    ) -> Result<CreateChatCompletionRequest, actix_web::Error> {
        self.inject_system_messages(&mut history);
        history.push(ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::from(user_text),
            name: None,
        }));

        Ok(CreateChatCompletionRequestArgs::default()
            .model(self.model())
            .messages(history)
            .stream(true)
            .build()
            .unwrap()
        )
    }

    fn inject_system_messages(&self, history: &mut Vec<ChatCompletionRequestMessage>) {
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
    }
}