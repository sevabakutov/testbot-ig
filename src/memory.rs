use std::{collections::HashMap, sync::Mutex};

use async_openai::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent,
};
use async_trait::async_trait;
use once_cell::sync::Lazy;

pub static ESCALATED_CHATS: Lazy<Mutex<HashMap<String, bool>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub static MEMORY: Lazy<Mutex<HashMap<String, Vec<ChatCompletionRequestMessage>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

const MAX_MESSAGES: usize = 20;

#[async_trait]
pub trait MemoryStore: Send + Sync + 'static {
    async fn get(&self, chat_id: &str) -> Vec<ChatCompletionRequestMessage>;

    async fn push_user(&self, chat_id: &str, content: &str);

    async fn push_assistant(&self, chat_id: &str, content: &str);
}

#[derive(Clone)]
pub struct InMemoryStore;

#[async_trait]
impl MemoryStore for InMemoryStore {
    async fn get(&self, chat_id: &str) -> Vec<ChatCompletionRequestMessage> {
        MEMORY
            .lock()
            .unwrap()
            .get(chat_id)
            .cloned()
            .unwrap_or_default()
    }

    async fn push_user(&self, chat_id: &str, content: &str) {
        let mut data = MEMORY.lock().unwrap();
        let entry = data.entry(chat_id.to_string()).or_default();
        entry.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::from(content),
                name: None,
            },
        ));
        truncate_if_needed(entry);
    }

    #[allow(deprecated)]
    async fn push_assistant(&self, chat_id: &str, content: &str) {
        let mut data = MEMORY.lock().unwrap();
        let entry = data.entry(chat_id.to_string()).or_default();
        entry.push(ChatCompletionRequestMessage::Assistant(
            ChatCompletionRequestAssistantMessage {
                content: Some(ChatCompletionRequestAssistantMessageContent::from(content)),
                name: None,
                audio: None,
                refusal: None,
                tool_calls: None,
                function_call: None
            },
        ));
        truncate_if_needed(entry);
    }
}

fn truncate_if_needed(history: &mut Vec<ChatCompletionRequestMessage>) {
    if history.len() > MAX_MESSAGES {
        let excess = history.len() - MAX_MESSAGES;
        history.drain(0..excess);
    }
}
