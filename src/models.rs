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
pub struct InstagramMessagesRequest {
    field: Field,
    value: MessagesValue
}

impl Display for InstagramMessagesRequest {
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