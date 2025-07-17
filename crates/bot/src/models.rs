use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::constants::IG_USER_ID;

/* ---------- Core wrappers ---------- */

#[derive(Debug, Deserialize, Clone)]
pub struct Sender {
    id: String,
}

impl Sender {
    pub fn id(&self) -> &str { &self.id }

    pub fn as_recipient(&self) -> Recipient { Recipient::new(self.id()) }

    pub fn new(id: &str) -> Self { Self { id: id.into() } }
}

impl Display for Sender {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { write!(f, "{}", self.id()) }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Recipient {
    id: String,
}

impl Recipient {
    pub fn id(&self) -> &str { &self.id }

    pub fn new<S: Into<String>>(id: S) -> Self { Self { id: id.into() } }

    pub fn as_sender(&self) -> Sender { Sender::new(self.id()) }
}

impl Display for Recipient {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { write!(f, "{}", self.id()) }
}

/* ---------- Message payloads ---------- */

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IncomingMessage {
    mid:  String,
    text: String,
}

impl IncomingMessage {
    pub fn text(&self) -> &str { &self.text }

    pub fn from_bot(&self, ig_user_id: &str, sender_id: &str) -> bool {
        sender_id == ig_user_id
    }
}

impl Display for IncomingMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\" (mid: {})", self.text, self.mid)
    }
}

/* ---------- Webhook hierarchy ---------- */

#[derive(Debug, Deserialize, Clone)]
pub struct MessagesValue {
    sender:    Sender,
    recipient: Recipient,
    timestamp: u64,
    message:   IncomingMessage,
}

impl MessagesValue {
    pub fn sender(&self) -> &Sender              { &self.sender }
    pub fn recipient(&self) -> &Recipient        { &self.recipient }
    pub fn message(&self) -> &IncomingMessage    { &self.message }
}

impl Display for MessagesValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[IG] {} → {} @ {} : {}",
            self.sender,
            self.recipient,
            self.timestamp,
            self.message
        )
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Entry {
    id:        String,
    time:      u64,
    messaging: Vec<MessagesValue>,
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "• Entry {} @ {}", self.id, self.time)?;
        for ch in &self.messaging {
            writeln!(f, "    {}", ch)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MetaObject {
    Instagram,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebhookPayload {
    object: MetaObject,
    entry:  Vec<Entry>,
}

/* ---------- Helpers used в хэндлере ---------- */

impl WebhookPayload {
    /// Первая `messaging`-запись во всём payload
    fn first_msg(&self) -> Option<&MessagesValue> {
        self.entry.iter().flat_map(|e| &e.messaging).next()
    }

    /// ID пользователя, приславшего сообщение (primary sender)
    pub fn sender(&self) -> Option<&Sender> {
        self.first_msg().map(|m| m.sender())
    }

    /// Текст входящего сообщения
    pub fn text(&self) -> Option<&str> {
        self.first_msg().map(|m| m.message().text())
    }

    /// Проверка, что событие — echo от нашего же бота
    pub fn is_bot_echo(&self) -> bool {
        self.first_msg()
            .map(|m| m.message().from_bot(IG_USER_ID.as_str(), m.sender().id()))
            .unwrap_or(false)
    }

    /// Клиент попросил человека
    pub fn wants_human(&self) -> bool {
        self.text().map(|t| t.trim() == "human agent").unwrap_or(false)
    }

    /// Чат-идентификатор: используем ID отправителя
    pub fn chat_id(&self) -> Option<&str> {
        self.sender().map(|s| s.id())
    }

    /// Recipient бизнес-аккаунта
    pub fn recipient(&self) -> Option<&Recipient> {
        self.first_msg().map(|m| m.recipient())
    }
}

impl Display for WebhookPayload {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "=== Webhook object: {:?} | {} entr{} ===",
            self.object,
            self.entry.len(),
            if self.entry.len() == 1 { "y" } else { "ies" }
        )?;
        for e in &self.entry {
            writeln!(f, "{}", e)?;
        }
        Ok(())
    }
}

/* ---------- Outgoing wrappers ---------- */

#[derive(Debug, Serialize)]
pub struct OutgoingMessage<'a> {
    text: &'a str,
}

impl OutgoingMessage<'_> {
    pub fn text(&self) -> &str { self.text }
}

impl<'a> From<&'a str> for OutgoingMessage<'a> {
    fn from(s: &'a str) -> Self { Self { text: s } }
}

#[derive(Debug, Serialize)]
pub struct SendBody<'a> {
    recipient: Recipient,
    message:   OutgoingMessage<'a>,
}

impl<'a> SendBody<'a> {
    pub fn new(recipient: Recipient, message: OutgoingMessage<'a>) -> Self {
        Self { recipient, message }
    }
}

/* ---------- OpenAI model enum ---------- */
#[derive(Debug, Clone)]
pub enum Model {
    GPT41nano,
    GPT41mini,
    TextEmbedding3Large,
    GPT41MiniFineTuned
}

impl Model {
    pub fn id(&self) -> &str {
        match self {
            Self::GPT41mini           => "gpt-4.1-mini",
            Self::GPT41nano           => "gpt-4.1-nano",
            Self::TextEmbedding3Large => "text-embedding-3-large",
            Self::GPT41MiniFineTuned  => "ft:gpt-4.1-mini-2025-04-14:personal:test-ig-bot:Btvegwb1"
        }
    }
}
