use sawtooth_sdk::messages::processor::TpProcessRequest;
use sawtooth_sdk::processor::handler::{ApplyError, TransactionContext};
use sha2::Digest;

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

#[derive(Debug)]
pub struct Handler {
    family_name: String,
    family_versions: Vec<String>,
    family_namespaces: Vec<String>,
}

impl Handler {
    pub fn new() -> Self {
        let family_name = "alica_messages";
        let mut hasher = sha2::Sha512::new();
        hasher.update(family_name);
        let result = hasher.finalize();

        let namespace = data_encoding::HEXLOWER.encode(&result[..6]);

        Handler {
            family_name: String::from(family_name),
            family_versions: vec![String::from("0.1.0")],
            family_namespaces: vec![namespace],
        }
    }

    fn state_address_for(&self, family_name: &str, message: &Message) -> String {
        let mut hasher = sha2::Sha512::new();
        hasher.update(format!(
            "{}{}{}",
            &message.agent_id, &message.message_type, &message.timestamp
        ));
        let namespace_part = data_encoding::HEXLOWER.encode(&hasher.finalize());

        let mut hasher = sha2::Sha512::new();
        hasher.update(family_name);
        let payload_part = data_encoding::HEXLOWER.encode(&hasher.finalize()[..]);

        format!("{}{}", &namespace_part[..6], &payload_part[..64])
    }
}

impl sawtooth_sdk::processor::handler::TransactionHandler for Handler {
    fn family_name(&self) -> String {
        self.family_name.clone()
    }

    fn family_versions(&self) -> Vec<String> {
        self.family_versions.clone()
    }

    fn namespaces(&self) -> Vec<String> {
        self.family_namespaces.clone()
    }

    fn apply(
        &self,
        request: &TpProcessRequest,
        context: &mut dyn TransactionContext,
    ) -> Result<(), ApplyError> {
        println!(
            "Transaction received from {}!",
            &request.get_header().get_signer_public_key()[..6]
        );

        let message = match Message::from(request.get_payload().to_vec()) {
            Ok(m) => m,
            Err(e) => return Err(e),
        };

        let address = self.state_address_for(&self.family_name(), &message);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod messsage {
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

    mod handler {
        use super::*;
        use sawtooth_sdk::messages::processor::TpProcessRequest;
        use sawtooth_sdk::processor::handler::{
            ContextError, TransactionContext, TransactionHandler,
        };

        mockall::mock! {
            pub Context {}

            trait TransactionContext {
                fn get_state_entries(&self, address: &[String]) -> Result<Vec<(String, Vec<u8>)>, ContextError>;
                fn set_state_entries(&self, entries: Vec<(String, Vec<u8>)>) -> Result<(), ContextError>;
                fn delete_state_entries(&self, address: &[String]) -> Result<Vec<String>, ContextError>;
                fn add_receipt_data(&self, data: &[u8]) -> Result<(), ContextError>;
                fn add_event(&self, address: String, entries: Vec<(String, String)>, data: &[u8]) -> Result<(), ContextError>;
            }
        }

        #[test]
        fn apply_with_invalid_utf8_payload_fails_with_apply_error() {
            let handler = Handler::new();

            let mut request = TpProcessRequest::new();
            let mut context = MockContext::new();

            let mut header = sawtooth_sdk::messages::transaction::TransactionHeader::new();
            header.set_signer_public_key(String::from("980490840984984"));
            request.set_header(header);
            request.set_payload(vec![0xff, 0xff]);

            handler.apply(&request, &mut context).unwrap_err();
        }

        #[test]
        fn apply_with_validly_structured_payload_succeeds() {
            let handler = Handler::new();

            let mut request = TpProcessRequest::new();
            let mut context = MockContext::new();

            let mut header = sawtooth_sdk::messages::transaction::TransactionHeader::new();
            header.set_signer_public_key(String::from("980490840984984"));
            request.set_header(header);
            request.set_payload("id|type|msg|ts".as_bytes().to_vec());

            handler.apply(&request, &mut context).unwrap();
        }

        #[test]
        fn generated_address_is_70_bytes_in_size() {
            let handler = Handler::new();
            let message = Message {
                agent_id: String::from("id"),
                message_type: String::from("type"),
                message: String::from("message").as_bytes().to_vec(),
                timestamp: String::from("6876984987987989"),
            };

            let address = handler.state_address_for("alica_messages", &message);

            assert_eq!(address.as_bytes().len(), 70);
        }

        #[test]
        fn generated_address_for_empty_message_is_70_bytes_in_size() {
            let handler = Handler::new();
            let message = Message {
                agent_id: String::from("id"),
                message_type: String::from("type"),
                message: String::from("").as_bytes().to_vec(),
                timestamp: String::from("684984984984"),
            };

            let address = handler.state_address_for("alica_messages", &message);

            assert_eq!(address.as_bytes().len(), 70);
        }
    }
}
