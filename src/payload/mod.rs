pub mod parser;

use std::fmt::{Debug, Display, Formatter, Result};
use mockall;

pub type ParsingResult = std::result::Result<TransactionPayload, ParsingError>;

#[mockall::automock]
pub trait Parser {
    fn parse(&self, bytes: &[u8]) -> ParsingResult;
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

pub struct TransactionPayload {
    pub agent_id: String,
    pub message_type: String,
    pub message_bytes: Vec<u8>,
    pub timestamp: u64,
}

impl TransactionPayload {
    pub fn new(agent_id: &str, message_type: &str, message_bytes: &[u8], timestamp: u64) -> Self {
        TransactionPayload {
            agent_id: agent_id.to_string(),
            message_type: message_type.to_string(),
            message_bytes: message_bytes.to_vec(),
            timestamp,
        }
    }
}

impl Default for TransactionPayload {
    fn default() -> Self {
        TransactionPayload {
            agent_id: "".to_string(),
            message_type: "".to_string(),
            message_bytes: "".as_bytes().to_vec(),
            timestamp: 0
        }
    }
}
