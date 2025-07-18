use std::sync::Arc;
use anyhow::{anyhow, Context, Result};
use async_openai::{
    config::OpenAIConfig,
    types::*,
    Client,
};
use crate::{
    constants::{DIM_SIZE, OPENAI_PROJECT_ID},
    models::{
        Model, 
        OutgoingMessage
    },
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

    pub fn client(&self) -> &Client<OpenAIConfig> { &self.client }
    pub fn model(&self) -> &str { &self.model.id() }

    pub async fn embed(&self, text: String) -> Result<Vec<f32>> {
        let response = self
            .client
            .embeddings()
            .create(
                CreateEmbeddingRequestArgs::default()
                    .model("text-embedding-3-large")
                    .dimensions(DIM_SIZE as u32)
                    .input(EmbeddingInput::String(text))
                    .build()
                    .context("Failed to build embedding request")?
            )
            .await
            .context("Failed to embed")?;

        Ok(response.data[0].embedding.clone())
    }

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
            .context("Failed to build request")?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .context("Failed to send request")?;

        Ok(response
            .choices
            .get(0)
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| anyhow!("empty response"))?
        )
    }

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

    pub fn prepare_stream_request_with_memory(
        &self,
        user_text: &str,
        mut history: Vec<ChatCompletionRequestMessage>,
        rag_memory: &str,
    ) -> Result<CreateChatCompletionRequest> {
        self.inject_system_messages(&mut history);

        /* Вставляем память как отдельный System‑message */
        history.insert(2, ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::from(
                    format!("MEMORY:\n{}", rag_memory)
                ),
                name: None,
            }
        ));

        history.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::from(user_text),
                name: None,
            },
        ));

        Ok(CreateChatCompletionRequestArgs::default()
            .model(self.model())
            .messages(history)
            .stream(true)
            .build()
            .context("Failed to build stream")?
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