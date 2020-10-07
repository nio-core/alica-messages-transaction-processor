use sawtooth_sdk::processor::handler::ApplyError;

pub mod handler;

#[derive(Debug)]
pub struct Message {
    agent_id: String,
    message_type: String,
    message: Vec<u8>,
    timestamp: String,
}

impl Message {
    // payload syntax: agent_id|message_type|message|timestamp
    pub fn from(bytes: Vec<u8>) -> Result<Message, ApplyError> {
        let payload = match String::from_utf8(bytes) {
            Ok(payload) => payload,
            Err(_e) => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Failed to decode payload in UTF8",
                )))
            }
        };

        let mut content = payload.split("|");
        let agent_id = match content.next() {
            Some(id) => id,
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "No agent ID supplied in payload!",
                )))
            }
        };

        let message_type = match content.next() {
            Some(t) => t,
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "No message type suppliied in payload!",
                )))
            }
        };

        let message = match content.next() {
            Some(m) => m,
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "No message supplied in payload!",
                )))
            }
        };

        let timestamp = match content.next() {
            Some(t) => t,
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "No timestamp supplied in payload!",
                )))
            }
        };

        Ok(Message {
            agent_id: String::from(agent_id),
            message_type: String::from(message_type),
            message: message.as_bytes().to_vec(),
            timestamp: String::from(timestamp),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn message_is_parsed_if_payload_is_valid() {
        let id = "id";
        let message_type = "type";
        let message_text = "msg";
        let timestamp = "ts";

        let message_bytes = format!("{}|{}|{}|{}", id, message_type, message_text, timestamp)
            .as_bytes()
            .to_vec();

        let message = Message::from(message_bytes).expect("Error parsing payload");

        assert_eq!(message.agent_id, id);
        assert_eq!(message.message_type, message_type);
        assert_eq!(message.message, message_text.as_bytes().to_vec());
        assert_eq!(message.timestamp, timestamp);
    }

    #[test]
    fn message_is_not_parsed_if_payload_is_missing_a_timestamp() {
        let id = "id";
        let message_type = "type";
        let message_text = "msg";

        let message_bytes = format!("{}|{}|{}", id, message_type, message_text)
            .as_bytes()
            .to_vec();

        Message::from(message_bytes).unwrap_err();
    }

    #[test]
    fn message_is_not_parsed_if_payload_is_missing_a_timestamp_and_a_message() {
        let id = "id";
        let message_type = "type";

        let message_bytes = format!("{}|{}", id, message_type).as_bytes().to_vec();

        Message::from(message_bytes).unwrap_err();
    }

    #[test]
    fn message_is_not_parsed_if_payload_is_missing_a_timestamp_and_a_message_and_a_message_type(
    ) {
        let id = "id";

        let message_bytes = format!("{}", id,).as_bytes().to_vec();

        Message::from(message_bytes).unwrap_err();
    }

    #[test]
    fn empty_message_is_not_parsed() {
        let message_bytes = "".as_bytes().to_vec();

        Message::from(message_bytes).unwrap_err();
    }
}
