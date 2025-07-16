use actix_web::{error::ErrorInternalServerError, Result};
use async_openai::{
    config::OpenAIConfig, 
    types::*, 
    Client, 
    Embeddings
};
use crate::{
    constants::OPENAI_PROJECT_ID, 
    models::{Model, OutgoingMessage}
};

pub struct OpenAIClient {
    client: Client<OpenAIConfig>,
    model: Model
}

impl OpenAIClient {
    pub fn new(model: Model) -> Self {
        let config = OpenAIConfig::new().with_project_id(OPENAI_PROJECT_ID.as_str());
        let client = Client::with_config(config);

        Self { client, model }
    }

    pub async fn send(
        &self, 
        msg: OutgoingMessage<'_>,
        history: Vec<ChatCompletionRequestMessage>
    ) -> Result<String> {
        let system_content = include_str!("prompts/system.txt"); 
        let developer_content = include_str!("prompts/developer.txt");

        let mut prompts = Vec::new();
        prompts.push(ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::from(system_content),
                name: None,
            },
        ));
        prompts.push(ChatCompletionRequestMessage::Developer(
            ChatCompletionRequestDeveloperMessage {
                content: ChatCompletionRequestDeveloperMessageContent::from(developer_content),
                name: None,
            },
        ));
        prompts.extend(history);

        prompts.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::from(msg.text()),
                name: None,
            },
        ));

        let request = CreateChatCompletionRequestArgs::default()
            .model(self.model())
            .messages(prompts)
            .build()
            .unwrap();

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(ErrorInternalServerError)?;

        Ok(response.choices[0].message.content.clone().unwrap())
    }

    pub fn embeddings(&self) -> Embeddings<'_, OpenAIConfig> {
        self.client.embeddings()
    }

    pub fn model(&self) -> &str {
        &self.model.id()
    }
}