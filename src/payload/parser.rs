use super::{
    Parser, ParsingResult, TransactionPayload
};
use crate::payload::ParsingError::{InvalidPayload, InvalidTimestamp};

pub struct PipeSeparatedPayloadParser {}

impl PipeSeparatedPayloadParser {
    pub fn new() -> Self {
        PipeSeparatedPayloadParser {}
    }
}

impl Parser for PipeSeparatedPayloadParser {
    fn parse(&self, bytes: &[u8]) -> ParsingResult {
        let payload = String::from_utf8(bytes.to_vec())
            .map_err(|_| InvalidPayload("Payload is not a string".to_string()))?;

        let mut content = payload.split("|");
        let agent_id = content.next()
            .ok_or_else(|| InvalidPayload("Payload contains no agent id".to_string()))?;
        let message_type = content.next()
            .ok_or_else(|| InvalidPayload("Payload contains no message type".to_string()))?;
        let message_bytes = content.next()
            .and_then(|message| Some(message.as_bytes()))
            .ok_or_else(|| InvalidPayload("Payload contains no message".to_string()))?;
        let timestamp = content.next()
            .ok_or_else(|| InvalidPayload("Payload contains no timestamp".to_string()))?
            .parse::<u64>()
            .map_err(|_| InvalidTimestamp)?;

        Ok(TransactionPayload::new(
            agent_id,
            message_type,
            message_bytes,
            timestamp,
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::payload::Parser;
    use super::PipeSeparatedPayloadParser;

    #[test]
    fn the_payload_is_valid_if_it_is_structured_properly() {
        let id = "id";
        let message_type = "type";
        let message_text = "msg";
        let timestamp = 684948894984u64;

        let payload_bytes = format!("{}|{}|{}|{}", id, message_type, message_text, timestamp)
            .as_bytes()
            .to_vec();

        let payload = PipeSeparatedPayloadParser::new()
            .parse(&payload_bytes)
            .expect("Error parsing payload");

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

        assert!(PipeSeparatedPayloadParser::new()
            .parse(&payload_bytes)
            .is_err())
    }

    #[test]
    fn the_payload_is_not_valid_if_the_message_is_missing() {
        let id = "id";
        let message_type = "type";
        let timestamp = 6849849849u64;

        let payload_bytes = format!("{}|{}|{}", id, message_type, timestamp)
            .as_bytes()
            .to_vec();

        assert!(PipeSeparatedPayloadParser::new()
            .parse(&payload_bytes)
            .is_err())
    }

    #[test]
    fn the_payload_is_not_valid_if_the_message_type_is_missing() {
        let id = "id";
        let message = "message";
        let timestamp = 9819849484984u64;

        let payload_bytes = format!("{}|{}|{}", id, message, timestamp)
            .as_bytes()
            .to_vec();

        assert!(PipeSeparatedPayloadParser::new()
            .parse(&payload_bytes)
            .is_err())
    }

    #[test]
    fn the_payload_is_valid_if_the_agent_id_is_missing() {
        let message_type = "type";
        let message_text = "msg";
        let timestamp = 649494894984u64;

        let payload_bytes = format!("{}|{}|{}", message_type, message_text, timestamp)
            .as_bytes()
            .to_vec();

        assert!(PipeSeparatedPayloadParser::new()
            .parse(&payload_bytes)
            .is_err())
    }

    #[test]
    fn empty_message_is_not_parsed() {
        let payload_bytes = "".as_bytes();
        assert!(PipeSeparatedPayloadParser::new()
            .parse(payload_bytes)
            .is_err())
    }
}
