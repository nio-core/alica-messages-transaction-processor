use crate::payload::ParsingError::InvalidPayload;

pub mod handler;

pub mod payload {
    use std::fmt::{Display, Formatter, Result};
    use crate::payload::ParsingError::InvalidPayload;

    #[derive(Debug)]
    pub enum ParsingError {
        InvalidPayload(String),
    }

    impl Display for ParsingError {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
            match self {
                InvalidPayload(message) => write!(formatter, "{}", message)
            }
        }
    }
}

#[derive(Debug)]
pub struct AlicaMessagePayload {
    agent_id: String,
    message_type: String,
    message: Vec<u8>,
    timestamp: String,
}

impl AlicaMessagePayload {
    // payload syntax: agent_id|message_type|message|timestamp
    pub fn from(bytes: Vec<u8>) -> Result<AlicaMessagePayload, payload::ParsingError> {
        let payload = match String::from_utf8(bytes) {
            Ok(payload) => Ok(payload),
            Err(_) => Err(InvalidPayload(String::from("Failed to decode payload in UTF8")))
        }?;

        let mut content = payload.split("|");
        let agent_id = match content.next() {
            Some(id) => Ok(String::from(id)),
            None => Err(InvalidPayload(String::from("No agent ID supplied in payload!")))
        };

        let message_type = match content.next() {
            Some(t) => Ok(String::from(t)),
            None => Err(InvalidPayload(String::from("No message type supplied in payload!")))
        };

        let message = match content.next() {
            Some(m) => Ok(String::from(m)),
            None => Err(InvalidPayload(String::from("No message supplied in payload!")))
        };

        let timestamp = match content.next() {
            Some(t) => Ok(String::from(t)),
            None => Err(InvalidPayload(String::from("No timestamp supplied in payload!")))
        };

        Ok(AlicaMessagePayload {
            agent_id: agent_id?,
            message_type: message_type?,
            message: message?.as_bytes().to_vec(),
            timestamp: timestamp?,
        })
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
        let timestamp = "ts";

        let payload_bytes = format!("{}|{}|{}|{}", id, message_type, message_text, timestamp)
            .as_bytes()
            .to_vec();

        let payload = AlicaMessagePayload::from(payload_bytes).expect("Error parsing payload");

        assert_eq!(payload.agent_id, id);
        assert_eq!(payload.message_type, message_type);
        assert_eq!(payload.message, message_text.as_bytes().to_vec());
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
        let timestamp = "ts";

        let payload_bytes = format!("{}|{}|{}", id, message_type, timestamp).as_bytes().to_vec();

        AlicaMessagePayload::from(payload_bytes).unwrap_err();
    }

    #[test]
    fn the_payload_is_not_valid_if_the_message_type_is_missing(
    ) {
        let id = "id";
        let message = "message";
        let timestamp = "ts";

        let payload_bytes = format!("{}|{}|{}", id, message, timestamp).as_bytes().to_vec();

        AlicaMessagePayload::from(payload_bytes).unwrap_err();
    }

    #[test]
    fn the_payload_is_valid_if_the_agent_id_is_missing() {
        let message_type = "type";
        let message_text = "msg";
        let timestamp = "ts";

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
