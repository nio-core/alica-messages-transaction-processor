use crate::payload;
use crate::payload::ParsingError::{InvalidPayload, InvalidTimestamp};
use std::fmt::{Debug, Display, Formatter, Result};

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

pub type ParsingResult<T> = std::result::Result<T, payload::ParsingError>;

#[derive(Debug)]
pub struct AlicaMessagePayload {
    pub agent_id: String,
    pub message_type: String,
    pub message_bytes: Vec<u8>,
    pub timestamp: u64,
}

impl AlicaMessagePayload {
    // payload syntax: agent_id|message_type|message|timestamp
    const REQUIRED_PAYLOAD_PART_COUNT: i32 = 4;

    pub fn from(bytes: &[u8]) -> ParsingResult<AlicaMessagePayload> {
        let payload = String::from_utf8(bytes.to_vec())
            .map_err(|_| InvalidPayload("Payload is no string".to_string()))?;

        let mut content = payload.split("|");
        let part_count = content.clone().count() as i32;

        if part_count != AlicaMessagePayload::REQUIRED_PAYLOAD_PART_COUNT {
            Err(InvalidPayload(format!(
                "Payload needs to have exactly {} parts",
                AlicaMessagePayload::REQUIRED_PAYLOAD_PART_COUNT
            )))
        } else {
            let agent_id = content.next().unwrap().to_string();
            let message_type = content.next().unwrap().to_string();
            let message_bytes = content.next().unwrap().as_bytes().to_vec();
            let timestamp = content
                .next()
                .unwrap()
                .parse::<u64>()
                .map_err(|_| InvalidTimestamp)?;

            Ok(AlicaMessagePayload {
                agent_id,
                message_type,
                message_bytes,
                timestamp,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_payload_is_valid_if_it_is_structured_properly() {
        let id = "id";
        let message_type = "type";
        let message_text = "msg";
        let timestamp = 684948894984u64;

        let payload_bytes = format!("{}|{}|{}|{}", id, message_type, message_text, timestamp)
            .as_bytes()
            .to_vec();

        let payload = AlicaMessagePayload::from(&payload_bytes).expect("Error parsing payload");

        assert_eq!(payload.agent_id, id);
        assert_eq!(payload.message_type, message_type);
        assert_eq!(payload.message_bytes, message_text.as_bytes().to_vec());
        assert_eq!(payload.timestamp, timestamp);
    }

    #[test]
    fn the_payload_is_not_valid_if_the_timestamp_is_missing() {
        let id = "id";
        let message_type = "type";
        let message_text = "msg";

        let payload_bytes = format!("{}|{}|{}", id, message_type, message_text)
            .as_bytes()
            .to_vec();

        AlicaMessagePayload::from(&payload_bytes).unwrap_err();
    }

    #[test]
    fn the_payload_is_not_valid_if_the_message_is_missing() {
        let id = "id";
        let message_type = "type";
        let timestamp = 6849849849u64;

        let payload_bytes = format!("{}|{}|{}", id, message_type, timestamp)
            .as_bytes()
            .to_vec();

        AlicaMessagePayload::from(&payload_bytes).unwrap_err();
    }

    #[test]
    fn the_payload_is_not_valid_if_the_message_type_is_missing() {
        let id = "id";
        let message = "message";
        let timestamp = 9819849484984u64;

        let payload_bytes = format!("{}|{}|{}", id, message, timestamp)
            .as_bytes()
            .to_vec();

        AlicaMessagePayload::from(&payload_bytes).unwrap_err();
    }

    #[test]
    fn the_payload_is_valid_if_the_agent_id_is_missing() {
        let message_type = "type";
        let message_text = "msg";
        let timestamp = 649494894984u64;

        let payload_bytes = format!("{}|{}|{}", message_type, message_text, timestamp)
            .as_bytes()
            .to_vec();

        AlicaMessagePayload::from(&payload_bytes).unwrap_err();
    }

    #[test]
    fn empty_message_is_not_parsed() {
        let payload_bytes = "".as_bytes();
        AlicaMessagePayload::from(payload_bytes).unwrap_err();
    }
}
