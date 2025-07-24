use std::{collections::HashMap, sync::{Arc, Mutex}};
use anyhow::{anyhow, Context, Result};
use async_openai::{
    config::OpenAIConfig,
    types::*,
    Client,
};
use serde_json::{json, Value};
use tokio_stream::StreamExt;
use crate::*;
use api::{
    meta::send_to_ig_by_paragraphs, 
    tools::call_fn
};
use constants::{DIM_SIZE, OPENAI_PROJECT_ID};
use memory::{get_chat_summary, MemoryStore};
use models::Recipient;
use vector_store::VectorStore;


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

    
    
    pub async fn stream_by_paragraph_with_tools<M: MemoryStore>(
        &self,
        memory: &M,
        recipient: Recipient,
        user_text: &str,
        vstore: &VectorStore,
        // rag_memory: &str,
    ) -> Result<()> {
        let chat_id = recipient.id();
        let mut history = memory.get(chat_id);
        
        // 1. Вставляем два систменых пропта с папки prompts/
        // 2. Вставляем информацию из векторной базы данных
        // 3. Вставляем резюме чата
        // 4. Вставляем запрос пользователя
        self.inject_system_messages(&mut history);

        // history.insert(
        //     2,
        //     ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
        //         content: ChatCompletionRequestSystemMessageContent::from(format!(
        //             "ИНФОРМАЦИЯ С ВЕКТОРНОЙ БАЗЫ ДАННЫХ ВЗЯТАЯ ВЫЗОВОМ ФУНКЦИЙ:\n{}",
        //             rag_memory
        //         )),
        //         name: None,
        //     }),
        // );

        history.insert(
            2,
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::from(format!(
                    "РЕЗЮМЕ ЧАТА (recipient_id: {}):\n{}",
                    recipient.id(),
                    get_chat_summary(chat_id)
                )),
                name: None,
            }),
        );

        history.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: user_text.into(),
                name: None,
            },
        ));
        
        // крутимся, пока модель не закончит tool calls
        let final_answer = self
            .complete_with_tools(history, vstore, )
            .await?;

        println!("✅ Финальный ответ модели ({} символов):\n{}", final_answer.len(), final_answer);

        // режем и отправляем в IG
        send_to_ig_by_paragraphs(recipient.clone(), &final_answer).await?;

        Ok(())
    }

    async fn complete_with_tools(
        &self,
        mut history: Vec<ChatCompletionRequestMessage>,
        vstore: &VectorStore
    ) -> Result<String> {
        loop {
            match self
                .run_with_tools_once(history.clone(), vstore)
                .await?
            {
                RunWithToolsOutcome::NeedMore { assistant_and_tool_messages } => {
                    println!("🔄 Модель попросила вызвать {} инструмент(а/ов). Запускаем следующий круг", assistant_and_tool_messages.len() - 1);
                    history.extend(assistant_and_tool_messages);
                    continue;
                }
                RunWithToolsOutcome::Done { final_text } => {
                    return Ok(final_text);
                }
            }
        }
    }

    async fn run_with_tools_once(
        &self,
        history: Vec<ChatCompletionRequestMessage>,
        vstore: &VectorStore
    ) -> Result<RunWithToolsOutcome> {
        println!("\n=== 📤 Отправляем запрос в OpenAI (messages: {}) ===", history.len());
        let request = CreateChatCompletionRequestArgs::default()
            .model("o4-mini")
            .messages(history)
            .tools(vec![
                ChatCompletionTool {
                    r#type: ChatCompletionToolType::Function,
                    function: FunctionObjectArgs::default()
                        .name("get_salons")
                        .description("Ищет салоны по фильтрам в векторной БД и возвращает их виде строки")
                        .parameters(json!({
                            "type": "object",
                            "properties": {
                                "lang": {
                                    "type": "string",
                                    "description": "задача — определить язык входного текста и вернуть результат в строгом структурированном формате. Русский → \"ru\"; Украинский → \"ua\"; Польский → \"pl\"; Остальное → \"unknown\""
                                },
                                "country": { 
                                    "type": "string",
                                    "description": "Страна для фильтрации салонов. если страна Польша -> Poland, украина -> Ukraine"
                                },
                            },
                            "required": ["lang", "country"]
                        }))
                        .build()?,
                },
                ChatCompletionTool {
                    r#type: ChatCompletionToolType::Function,
                    function: FunctionObjectArgs::default()
                        .name("get_lazer_hair_removal_info")
                        .description("Ищет информацию о лазерной эпиляции в векторной базе данных и возвращает её строкой")
                        .parameters(json!({
                            "type": "object",
                            "properties": {
                                "lang": {
                                    "type": "string",
                                    "description": "задача — определить язык входного текста и вернуть результат в строгом структурированном формате. Русский → \"ru\"; Украинский → \"ua\"; Польский → \"pl\"; Остальное → \"unknown\""
                                }
                            },
                            "required": ["lang"]
                        }))
                        .build()?,
                },
                ChatCompletionTool {
                    r#type: ChatCompletionToolType::Function,
                    function: FunctionObjectArgs::default()
                        .name("escalate")
                        .description("Эскалация чата оператору.")
                        .parameters(json!({
                            "type": "object",
                            "properties": {
                                "recipient_id": {
                                    "type": "string",
                                    "description": "Идентефикация чата"
                                }
                            },
                            "required": ["recipient_id"]
                        }))
                        .build()?
                }
            ])
            .tool_choice("auto")
            .build()
            .context("Failed to build chat request")?;

        println!("REQUEST:\n {:#?}\n", request.clone());

        let mut stream = self.client.chat().create_stream(request).await?;

        // Аккумулируем tool_calls по частям
        let tool_call_states: Arc<Mutex<HashMap<(u32, u32), ChatCompletionMessageToolCall>>> = Arc::new(Mutex::new(HashMap::new()));

        let mut final_text_buf = String::new();
        let mut finish_reason: Option<FinishReason> = None;
        let function_responses: Arc<Mutex<Vec<(ChatCompletionMessageToolCall, Value)>>> = Arc::new(Mutex::new(Vec::new()));

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            for choice in chunk.choices {
                // контент
                if let Some(content) = choice.delta.content {
                    println!("📝 delta.content: {}", content);
                    final_text_buf.push_str(&content);
                }

                // tool_calls delta
                if let Some(tool_calls) = choice.delta.tool_calls {
                    for tc in tool_calls {
                        println!("🛠️  delta.tool_call: index={} id={:?}", tc.index, tc.id);
                        let key = (choice.index, tc.index);
                        let mut map = tool_call_states.lock().unwrap();

                        let state = map.entry(key).or_insert_with(|| ChatCompletionMessageToolCall {
                            id: tc.id.clone().unwrap_or_default(),
                            r#type: ChatCompletionToolType::Function,
                            function: FunctionCall {
                                name: tc.function.as_ref().and_then(|f| f.name.clone()).unwrap_or_default(),
                                arguments: tc.function.as_ref().and_then(|f| f.arguments.clone()).unwrap_or_default(),
                            },
                        });

                        if let Some(args) = tc.function.and_then(|f| f.arguments) {
                            state.function.arguments.push_str(&args);
                        }
                    }
                }

                if let Some(fr) = &choice.finish_reason {
                    finish_reason = Some(fr.clone());
                }
            }
        }

        println!("🏁 finish_reason: {:?}", finish_reason);

        // Если модель просит вызвать tools
        if matches!(finish_reason, Some(FinishReason::ToolCalls)) {
            // 1) собираем все tool_call'ы
            let calls = {
                let map = tool_call_states.lock().unwrap();
                map.values().cloned().collect::<Vec<_>>()
            };

            // 2) параллельный вызов функций
            let mut handles = Vec::new();
            for call in calls.clone() {
                let vstore = vstore.clone();
                let function_responce = function_responses.clone();

                handles.push(tokio::spawn(async move {
                    let name = call.function.name.clone();
                    let args = call.function.arguments.clone();
                    let result = call_fn(&vstore, &name, &args).await;
                    println!("⚙️  Вызов функции '{}' с аргументами: {}", name, args);

                    match result {
                        Ok(val) => {
                            println!("✅ Функция '{}' вернула: {}", name, val);
                            function_responce.lock().unwrap().push((call.clone(), val));
                        }
                        Err(e) => {
                            println!("❌ Функция '{}' упала: {}", name, e);
                            function_responce.lock().unwrap().push((
                                call.clone(),
                                json!({"error": format!("tool `{}` failed: {}", name, e)}),
                            ));
                        }
                    }
                }));
            }

            for handle in handles {
                handle.await?;
            }

            // 3) строим assistant/tool сообщения для следующего раунда
            let function_responce_lock = function_responses.lock().unwrap();

            let tool_calls_only = function_responce_lock
                .iter()
                .map(|(c, _)| c.clone())
                .collect::<Vec<_>>();

            let assistant_msg = ChatCompletionRequestAssistantMessageArgs::default()
                .tool_calls(tool_calls_only)
                .build()?;

            let mut tool_msgs: Vec<ChatCompletionRequestMessage> = Vec::new();
            for (tool_call, value) in function_responce_lock.iter() {
                tool_msgs.push(
                    ChatCompletionRequestToolMessageArgs::default()
                        .tool_call_id(tool_call.id.clone())
                        .content(value.to_string())
                        .build()?
                        .into(),
                );
            }

            return Ok(RunWithToolsOutcome::NeedMore {
                assistant_and_tool_messages: {
                    let mut v = Vec::with_capacity(1 + tool_msgs.len());
                    v.push(assistant_msg.into());
                    v.extend(tool_msgs);
                    v
                },
            });
        }

        // Иначе — это финальный ответ
        Ok(RunWithToolsOutcome::Done {
            final_text: final_text_buf,
        })
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
            .model("o4-mini")
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

    // pub fn prepare_stream_request_with_memory(
    //     &self,
    //     user_text: &str,
    //     mut history: Vec<ChatCompletionRequestMessage>,
    //     snippets: &str,
    //     chat_id: &str
    // ) -> Result<CreateChatCompletionRequest> {
    //     self.inject_system_messages(&mut history);

    //     history.insert(2, ChatCompletionRequestMessage::System(
    //         ChatCompletionRequestSystemMessage {
    //             content: ChatCompletionRequestSystemMessageContent::from(
    //                 format!("MEMORY:\n{}", snippets)
    //             ),
    //             name: None,
    //         }
    //     ));

    //     history.insert(3, ChatCompletionRequestMessage::System(
    //         ChatCompletionRequestSystemMessage { 
    //             content: ChatCompletionRequestSystemMessageContent::from(
    //                 format!("CHAT SUMMARIZE:\n{}", get_chat_summary(chat_id))
    //             ), 
    //             name: None
    //         }
    //     ));

    //     history.push(ChatCompletionRequestMessage::User(
    //         ChatCompletionRequestUserMessage {
    //             content: ChatCompletionRequestUserMessageContent::from(user_text),
    //             name: None,
    //         },
    //     ));

    //     println!("MESSAGES: {:#?}", history.clone());

    //     Ok(CreateChatCompletionRequestArgs::default()
    //         .model("gpt-4.1-mini")
    //         .messages(history)
    //         .stream(true)
    //         .build()
    //         .context("Failed to build stream")?
    //     )
    // }

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

enum RunWithToolsOutcome {
    /// Модель попросила вызвать инструменты — нужно продолжить цикл,
    /// добавив эти сообщения к истории.
    NeedMore {
        assistant_and_tool_messages: Vec<ChatCompletionRequestMessage>,
    },
    /// Модель закончила и вернула финальный текст.
    Done {
        final_text: String,
    },
}
