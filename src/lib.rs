use crate::payload::ParsingError::{InvalidPayload, InvalidTimestamp};

pub mod handler;

pub mod payload {
    use std::fmt::{Display, Formatter, Result};

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
}

#[derive(Debug)]
pub struct AlicaMessagePayload {
    agent_id: String,
    message_type: String,
    message_bytes: Vec<u8>,
    timestamp: u64,
}

impl AlicaMessagePayload {
    // payload syntax: agent_id|message_type|message|timestamp
    pub fn from(bytes: Vec<u8>) -> Result<AlicaMessagePayload, payload::ParsingError> {
        let payload = String::from_utf8(bytes)
            .map_err(|_| { InvalidPayload("Payload is no string".to_string()) })?;

        let mut content = payload.split("|");
        let part_count = content.clone().count();

        if part_count != 4 {
            Err(InvalidPayload("Payload needs to have exactly 4 parts".to_string()))
        } else {
            let agent_id = content.next().unwrap().to_string();
            let message_type = content.next().unwrap().to_string();
            let message_bytes = content.next().unwrap().as_bytes().to_vec();
            let timestamp = content.next().unwrap().parse::<u64>()
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

        let payload = AlicaMessagePayload::from(payload_bytes).expect("Error parsing payload");

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

        AlicaMessagePayload::from(payload_bytes).unwrap_err();
    }

    #[test]
    fn the_payload_is_not_valid_if_the_message_is_missing() {
        let id = "id";
        let message_type = "type";
        let timestamp = 6849849849u64;

        let payload_bytes = format!("{}|{}|{}", id, message_type, timestamp).as_bytes().to_vec();

        AlicaMessagePayload::from(payload_bytes).unwrap_err();
    }

    #[test]
    fn the_payload_is_not_valid_if_the_message_type_is_missing(
    ) {
        let id = "id";
        let message = "message";
        let timestamp = 9819849484984u64;

        let payload_bytes = format!("{}|{}|{}", id, message, timestamp).as_bytes().to_vec();

        AlicaMessagePayload::from(payload_bytes).unwrap_err();
    }

    #[test]
    fn the_payload_is_valid_if_the_agent_id_is_missing() {
        let message_type = "type";
        let message_text = "msg";
        let timestamp = 649494894984u64;

        let payload_bytes = format!("{}|{}|{}", message_type, message_text, timestamp)
            .as_bytes()
            .to_vec();

        AlicaMessagePayload::from(payload_bytes).unwrap_err();
    }


    #[test]
    fn empty_message_is_not_parsed() {
        let payload_bytes = "".as_bytes().to_vec();

        AlicaMessagePayload::from(payload_bytes).unwrap_err();
    }
}
