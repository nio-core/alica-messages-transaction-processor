pub mod parser;

use std::fmt::{Debug, Display, Formatter, Result};

pub type ParsingResult<T> = std::result::Result<T, ParsingError>;

pub trait Parser {
    fn parse(&self, bytes: &[u8]) -> ParsingResult<Payload>;
}

#[derive(Debug)]
pub enum ParsingError {
    InvalidPayload(String),
    InvalidTimestamp,
}

impl Display for ParsingError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        let message = match self {
            ParsingError::InvalidPayload(message) => message,
            ParsingError::InvalidTimestamp => "Payload contains invalid timestamp",
        };

        write!(formatter, "{}", message)
    }
}

pub struct Payload {
    pub agent_id: String,
    pub message_type: String,
    pub message_bytes: Vec<u8>,
    pub timestamp: u64,
}

impl Payload {
    pub fn new(agent_id: &str, message_type: &str, message_bytes: &[u8], timestamp: u64) -> Self {
        Payload {
            agent_id: agent_id.to_string(),
            message_type: message_type.to_string(),
            message_bytes: message_bytes.to_vec(),
            timestamp,
        }
    }
}
