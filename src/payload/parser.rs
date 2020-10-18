use super::{
    Parser, ParsingError, ParsingResult, PayloadValidator, TransactionPayload, ValidationResult,
};

pub struct PipeSeparatedPayloadParser {
    validator: Box<dyn PayloadValidator>,
}

impl PipeSeparatedPayloadParser {
    pub fn new() -> Self {
        PipeSeparatedPayloadParser {
            validator: Box::from(PipeSeparatedPayloadValidator::new()),
        }
    }
}

impl Parser for PipeSeparatedPayloadParser {
    fn parse(&self, bytes: &[u8]) -> ParsingResult<TransactionPayload> {
        self.validator.validate(&bytes)?;

        let payload = String::from_utf8(bytes.to_vec())
            .expect("This cannot happen due to previous validation");

        let mut content = payload.split("|");
        let agent_id = content.next().unwrap();
        let message_type = content.next().unwrap();
        let message_bytes = content.next().unwrap().as_bytes();
        let timestamp = content
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ParsingError::InvalidTimestamp)?;

        Ok(TransactionPayload::new(
            agent_id,
            message_type,
            message_bytes,
            timestamp,
        ))
    }
}

pub struct PipeSeparatedPayloadValidator {}

impl PipeSeparatedPayloadValidator {
    pub const REQUIRED_PAYLOAD_PART_COUNT: u8 = 4;

    pub fn new() -> Self {
        PipeSeparatedPayloadValidator {}
    }
}

impl PayloadValidator for PipeSeparatedPayloadValidator {
    fn validate(&self, payload_bytes: &[u8]) -> ValidationResult {
        let payload = String::from_utf8(payload_bytes.to_vec())
            .map_err(|_| ParsingError::InvalidPayload("Payload is no string".to_string()))?;

        let content = payload.split("|");
        let part_count = content.count() as u8;
        if part_count != super::parser::PipeSeparatedPayloadValidator::REQUIRED_PAYLOAD_PART_COUNT {
            return Err(ParsingError::InvalidPayload(format!(
                "Payload needs to have exactly {} parts",
                super::parser::PipeSeparatedPayloadValidator::REQUIRED_PAYLOAD_PART_COUNT
            )));
        }

        Ok(())
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
