use std::{fmt::{self, Display, Formatter}};
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

static IG_USER_ID: Lazy<String> = Lazy::new(|| {
    std::env::var("IG_USER_ID").expect("Set IG_USER_ID env-var")
});

#[derive(Debug, Deserialize)]
pub enum Field {
    #[serde(rename = "messages")]
    Messages
}

#[derive(Debug, Deserialize, Clone)]
pub struct Sender {
    id: String
}

impl Sender {
    pub fn as_recipient(&self) -> Recipient {
        Recipient::new(self.id.as_str())
    }
}

impl Display for Sender {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Recipient {
    id: String
}

impl Recipient {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string()
        }
    }
}

impl Display for Recipient {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IncomingMessage {
    mid: String,
    text: String
}

impl IncomingMessage {
    pub fn from_bot(&self, ig_user_id: &str, sender_id: &str) -> bool {
        sender_id.eq(ig_user_id)
    }
}

impl Display for IncomingMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\" (mid: {})", self.text, self.mid)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct MessagesValue {
    sender: Sender,
    recipient: Recipient,
    timestamp: u64,
    message: IncomingMessage
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
    id: String,
    time: u64,
    messaging: Vec<MessagesValue>
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
    Instagram
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebhookPayload {
    object: MetaObject,
    entry: Vec<Entry>
}

impl WebhookPayload {
    fn first_msg(&self) -> Option<&MessagesValue> {
        self.entry.iter().flat_map(|e| &e.messaging).next()
    }

    pub fn primary_sender(&self) -> Option<&Sender> {
        self.first_msg().map(|m| &m.sender)
    }

    pub fn is_bot_echo(&self) -> bool {
        self.first_msg()
            .map(|m| m.message.from_bot(&IG_USER_ID, &m.sender.id))
            .unwrap_or(false)
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

#[derive(Debug, Serialize)]
pub struct OutgoingMessage<'a> {
    text: &'a str,
}

impl<'a> From<&'a str> for OutgoingMessage<'a> {
    fn from(s: &'a str) -> Self {
        Self { text: s }
    }
}

#[derive(Debug, Serialize)]
pub struct SendBody<'a> {
    recipient: Recipient,
    message  : OutgoingMessage<'a>,
}

impl<'a> SendBody<'a> {
    pub fn new(recipient: Recipient, message: OutgoingMessage) -> SendBody {
        SendBody { recipient, message }
    }
}