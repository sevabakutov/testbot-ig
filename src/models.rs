use std::fmt::{self, Display, Formatter};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum Field {
    #[serde(rename = "messages")]
    Messages
}

#[derive(Debug, Deserialize)]
pub struct Sender {
    id: String
}

impl Display for Sender {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug, Deserialize)]
pub struct Recipient {
    id: String
}

impl Display for Recipient {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug, Deserialize)]
pub struct Message {
    mid: String,
    text: String
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\" (mid: {})", self.text, self.mid)
    }
}

#[derive(Debug, Deserialize)]
pub struct MessagesValue {
    sender: Sender,
    recipient: Recipient,
    timestamp: String,
    message: Message
}

#[derive(Debug, Deserialize)]
pub struct Change {
    field: Field,
    value: MessagesValue
}

impl Display for Change {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.field {
            Field::Messages => write!(
                f,
                "[IG] {} → {} @ {} : {}",
                self.value.sender,
                self.value.recipient,
                self.value.timestamp,
                self.value.message
            ),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    id: String,
    time: u64,
    changes: Vec<Change>
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "• Entry {} @ {}", self.id, self.time)?;
        for ch in &self.changes {
            writeln!(f, "    {}", ch)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetaObject {
    Instagram
}

#[derive(Debug, Deserialize)]
pub struct WebhookPayload {
    object: MetaObject,
    entry: Vec<Entry>
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