use std::sync::Arc;
use anyhow::{anyhow, Context, Result};
use async_openai::{
    config::OpenAIConfig,
    types::*,
    Client,
};
use serde_json::json;
use crate::{
    constants::{DIM_SIZE, OPENAI_PROJECT_ID}, memory::get_chat_summary, models::{LanguageResponse, OutgoingMessage}
};

#[derive(Clone)]
pub struct OpenAIClient {
    client: Arc<Client<OpenAIConfig>>
}

impl OpenAIClient {
    pub fn new() -> Self {
        let config = OpenAIConfig::new().with_project_id(OPENAI_PROJECT_ID.as_str());
        let client = Client::with_config(config);
        Self {
            client: Arc::new(client),
        }
    }

    pub fn client(&self) -> &Client<OpenAIConfig> { 
        &self.client 
    }

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
            .model("gpt-4.1-mini")
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

    pub async fn identify_language(&self, text: &str) -> Result<String> {
        let system_prompt = r#"
Ты — модель, задача которой — определить язык входного текста и вернуть результат в строгом структурированном формате.  
1. Прочитай входящий текст.  
2. Определи, на каком языке он написан:  
- Русский  → код "ru"  
- Украинский → код "ua"  
- Польский   → код "pl"  
- Любой другой → код "unknown"  
3. Ответь **только** JSON-объектом со следующим полем:  
- **language**: строка, код языка.  
Никаких дополнительных полей и никакого свободного текста!  
"#;

        let json_schema = ResponseFormatJsonSchema {
            description: Some("Schema to get right language".to_string()),
            name: "Language detector".to_string(),
            schema: Some(json!({
                "type": "object",
                "properties": {
                    "language": { "type": "string" }
                },
                "required": ["language"]
            })),
            strict: Some(true),
        };

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4.1-nano")
            .messages([
                ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage { 
                    content: ChatCompletionRequestSystemMessageContent::from(system_prompt), 
                    name: None 
                }),
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::from(text),
                    name: None
                })
            ])
            .response_format(ResponseFormat::JsonSchema { json_schema })
            .build()?;

        let response = self.client
            .chat()
            .create(request)
            .await?;

        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .unwrap_or_default();

        let language = serde_json::from_str::<LanguageResponse>(&content)
            .map(|resp| resp.language)
            .unwrap_or_else(|_| "unknown".to_string());

        Ok(language)
    }

    pub async fn summarize_chat_memory(&self, history: &[ChatCompletionRequestMessage]) -> Result<String> {
        let system_prompt = r#"
Сожми следующие сообщения из переписки в краткое (не более 100 слов) резюме, 
выдели ключевые факты: имя клиента, упомянутые услуги, даты и любые договорённости.
"#;

        let mut messages: Vec<ChatCompletionRequestMessage> = Vec::with_capacity(history.len() + 1);

        messages.push(ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::from(system_prompt),
                name: None,
            },
        ));

        for msg in history.iter() {
            messages.push(msg.clone());
        }

        let request = async_openai::types::CreateChatCompletionRequestArgs::default()
            .model("gpt-4.1-mini")
            .messages(messages)
            .build()
            .context("Failed to build summarize_chat_memory request")?;

        let response = self
            .client()
            .chat()
            .create(request)
            .await
            .context("OpenAI chat.create failed")?;

        let summary = response
            .choices
            .get(0)
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| anyhow!("Empty summary response"))?;

        Ok(summary)

    }

    // pub fn prepare_stream_request(
    //     &self,
    //     user_text: &str,
    //     mut history: Vec<ChatCompletionRequestMessage>,
    // ) -> Result<CreateChatCompletionRequest, actix_web::Error> {
    //     self.inject_system_messages(&mut history);
    //     history.push(ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
    //         content: ChatCompletionRequestUserMessageContent::from(user_text),
    //         name: None,
    //     }));

    //     Ok(CreateChatCompletionRequestArgs::default()
    //         .model("gpt-4.1-mini")
    //         .messages(history)
    //         .stream(true)
    //         .build()
    //         .unwrap()
    //     )
    // }

    pub fn prepare_stream_request_with_memory(
        &self,
        user_text: &str,
        mut history: Vec<ChatCompletionRequestMessage>,
        snippets: &str,
        chat_id: &str
    ) -> Result<CreateChatCompletionRequest> {
        self.inject_system_messages(&mut history);

        history.insert(2, ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::from(
                    format!("MEMORY:\n{}", snippets)
                ),
                name: None,
            }
        ));

        history.insert(3, ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessage { 
                content: ChatCompletionRequestSystemMessageContent::from(
                    format!("CHAT SUMMARIZE:\n{}", get_chat_summary(chat_id))
                ), 
                name: None
            }
        ));

        history.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::from(user_text),
                name: None,
            },
        ));

        println!("MESSAGES: {:#?}", history.clone());

        Ok(CreateChatCompletionRequestArgs::default()
            .model("gpt-4.1-mini")
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