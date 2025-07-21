use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::{sleep, Instant}};
use crate::models::Recipient;

struct Pending {
    buf: String,
    last: Instant,
}

#[derive(Clone)]
pub struct Debouncer {
    inner: Arc<Mutex<HashMap<String, Pending>>>,
    window: Duration,
}

impl Debouncer {
    pub fn new(window: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            window,
        }
    }

    /// Добавление нового сообщения в цикл. Если айди чата уже в буфере, обновляеться время добавления последнего сообщения для этого чата, тем самым происходит склеивание разных сообщений в одно.
    pub async fn push(&self, chat_id: &str, text: &str) {
        let mut map = self.inner.lock().await;
        let entry = map.entry(chat_id.to_string()).or_insert_with(|| Pending {
            buf: String::new(),
            last: Instant::now(),
        });

        if !entry.buf.is_empty() {
            entry.buf.push(' ');
        }
        entry.buf.push_str(text.trim());
        entry.last = Instant::now();
    }

    /// Функция запуска фоновой задачи обработки входящих сообщений и очистка буферов для разных чатов. 
    pub async fn run<F>(self, mut on_ready: F) -> !
    where
        F: FnMut(Recipient /*chat_id*/, String /*merged text*/) + Send + 'static,
    {
        loop {
            sleep(Duration::from_millis(500)).await;

            let mut ready = Vec::new();
            {
                let mut map = self.inner.lock().await;
                map.retain(|chat, pending| {
                    if pending.last.elapsed() >= self.window {
                        ready.push((chat.clone(), pending.buf.clone()));
                        false
                    } else {
                        true
                    }
                });
            }

            for (chat, text) in ready.drain(..) {
                on_ready(Recipient::new(chat), text);
            }
        }
    }
}