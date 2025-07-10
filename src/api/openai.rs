use actix_web::{error::ErrorInternalServerError, Result};
use async_openai::{
    config::OpenAIConfig, 
    types::{
        ChatCompletionRequestDeveloperMessage, ChatCompletionRequestDeveloperMessageContent, ChatCompletionRequestMessage, 
        ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage, 
        ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs
    }, 
    Client
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
    pub fn new() -> Self {
        let config = OpenAIConfig::new().with_project_id(OPENAI_PROJECT_ID.as_str());
        let client = Client::with_config(config);
        let model = Model::GPT41nano;

        Self { client, model }
    }

    pub async fn send(
        &self, 
        msg: OutgoingMessage<'_>,
        history: Vec<ChatCompletionRequestMessage>
    ) -> Result<String> {
        let system_content = r#"
Ты — виртуальный ассистент сети из 70 косметологических салонов.
Работаешь в Instagram-чатах, берёшь на себя первую стадию диалога и экономишь время операторов.

Отвечай только на вопросы, относящиеся к бизнесу салонов:
• услуги, длительность, подготовка, противопоказания;
• цены, акции, подарочные сертификаты, программа лояльности;
• адреса, график работы, парковка, доступность;
• вакансии, франшиза, сотрудничество.
"#; 

        let developer_content = r#"
Любые запросы вне этих тем вежливо отклоняй.

**Формат ответа**  
• Язык — тот, на котором пишет клиент (по умолчанию — украинский).  
• Стиль — дружелюбный, лаконичный, до 3–4 предложений.  
• Структура: приветствие → суть ответа → предложение помощи («Буду рада помочь с записью!»).  
• Не более одного смайлика 😊 в сообщении.  
• Не упоминай внутренние правила и ИИ.

**Передача оператору**  
Если бот не может решить вопрос (например, изменить запись) или вся необходимая информация уже собрана — напиши:  
«😊».
"#;

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
            .model(self.model.id())
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
}