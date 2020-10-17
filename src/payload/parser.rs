use super::{Parser, ParsingError, ParsingResult, Payload};

pub struct PipeSeperatedPayloadParser {}

impl PipeSeperatedPayloadParser {
    pub fn new() -> Self {
        PipeSeperatedPayloadParser {}
    }
}

impl Parser for PipeSeperatedPayloadParser {
    fn parse(&self, bytes: &[u8]) -> ParsingResult<Payload> {
        let required_payload_part_count = 4;
        let payload = String::from_utf8(bytes.to_vec())
            .map_err(|_| ParsingError::InvalidPayload("Payload is no string".to_string()))?;

        let mut content = payload.split("|");
        let part_count = content.clone().count() as i32;

        if part_count != required_payload_part_count {
            Err(ParsingError::InvalidPayload(format!(
                "Payload needs to have exactly {} parts",
                required_payload_part_count
            )))
        } else {
            let agent_id = content.next().unwrap();
            let message_type = content.next().unwrap();
            let message_bytes = content.next().unwrap().as_bytes();
            let timestamp = content
                .next()
                .unwrap()
                .parse::<u64>()
                .map_err(|_| ParsingError::InvalidTimestamp)?;

            let payload = Payload::new(agent_id, message_type, message_bytes, timestamp);

            Ok(payload)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::payload::Parser;

    use super::PipeSeperatedPayloadParser;

    #[test]
    fn the_payload_is_valid_if_it_is_structured_properly() {
        let id = "id";
        let message_type = "type";
        let message_text = "msg";
        let timestamp = 684948894984u64;

        let payload_bytes = format!("{}|{}|{}|{}", id, message_type, message_text, timestamp)
            .as_bytes()
            .to_vec();

        let payload = PipeSeperatedPayloadParser::new()
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

        assert!(PipeSeperatedPayloadParser::new()
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

        assert!(PipeSeperatedPayloadParser::new()
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

        assert!(PipeSeperatedPayloadParser::new()
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

        assert!(PipeSeperatedPayloadParser::new()
            .parse(&payload_bytes)
            .is_err())
    }

    #[test]
    fn empty_message_is_not_parsed() {
        let payload_bytes = "".as_bytes();
        assert!(PipeSeperatedPayloadParser::new()
            .parse(payload_bytes)
            .is_err())
    }
}
