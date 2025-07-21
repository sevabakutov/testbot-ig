use std::{collections::HashMap, sync::Mutex};

use anyhow::{Context, Result};
use async_openai::types::*;
use async_trait::async_trait;
use once_cell::sync::Lazy;

use crate::api::openai::OpenAIClient;

pub static ESCALATED_CHATS: Lazy<Mutex<HashMap<String, bool>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub static REQUEST_COUNTER: Lazy<Mutex<HashMap<String, usize>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub static MEMORY: Lazy<Mutex<HashMap<String, Vec<ChatCompletionRequestMessage>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub static CHAT_SUMMARY: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

const MAX_MESSAGES: usize = 20;

#[async_trait]
pub trait MemoryStore: Send + Sync + 'static {
    async fn get(&self, chat_id: &str) -> Vec<ChatCompletionRequestMessage>;

    async fn push_user(&self, chat_id: &str, content: &str);

    async fn push_assistant(&self, chat_id: &str, content: &str);

    // async fn push_system(&self, chat_id: &str, content: &str);
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

        let mut counters = REQUEST_COUNTER.lock().unwrap();
        let count = counters.entry(chat_id.to_string()).or_insert(0);
        *count += 1;

        if *count == 1 || *count % 5 == 0 {
            let snapshot = entry.clone();
            let chat_id = chat_id.to_string();
            let client = OpenAIClient::new();

            tokio::spawn(async move {
                if let Err(e) = update_memory_summary(&client, &chat_id, &snapshot).await {
                    eprintln!("update_memory_summary error: {:?}", e);
                }
            });
        }
    }
}

pub fn get_chat_summary(chat_id: &str) -> String {
    let summaries = CHAT_SUMMARY.lock().unwrap();
    summaries
        .get(chat_id)
        .cloned()
        .unwrap_or_else(String::new)
}

async fn update_memory_summary(
    client: &OpenAIClient,
    chat_id: &str,
    history: &[ChatCompletionRequestMessage],
) -> Result<()> {
    let summary = client
        .summarize_chat_memory(history)
        .await
        .context("summarize_chat_memory failed")?;

    let mut chat_summary = CHAT_SUMMARY.lock().unwrap();
    chat_summary.insert(chat_id.to_string(), summary);

    Ok(())
}

fn truncate_if_needed(history: &mut Vec<ChatCompletionRequestMessage>) {
    if history.len() > MAX_MESSAGES {
        let excess = history.len() - MAX_MESSAGES;
        history.drain(0..excess);
    }
}
