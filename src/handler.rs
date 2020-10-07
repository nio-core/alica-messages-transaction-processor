use sawtooth_sdk::messages::processor::TpProcessRequest;
use sha2::{Sha512, Digest};
use crate::Message;
use sawtooth_sdk::processor::handler::{ApplyError, TransactionContext};
use sawtooth_sdk::processor::handler::ApplyError::{InvalidTransaction, InternalError};

#[derive(Debug)]
pub struct Handler {
    family_name: String,
    family_versions: Vec<String>,
    family_namespaces: Vec<String>,
}

impl Handler {
    pub fn new() -> Self {
        let family_name = "alica_messages";
        let mut hasher = Sha512::new();
        hasher.update(family_name);
        let result = hasher.finalize();
        let encoded_result = data_encoding::HEXLOWER.encode(&result);

        Handler {
            family_name: String::from(family_name),
            family_versions: vec![String::from("0.1.0")],
            family_namespaces: vec![String::from(&encoded_result[..6])],
        }
    }

    fn state_address_for(&self, message: &Message) -> String {
        let mut hasher = Sha512::new();
        hasher.update(format!("{}{}{}", &message.agent_id, &message.message_type, &message.timestamp));
        let payload_part = data_encoding::HEXLOWER.encode(&hasher.finalize_reset());

        let namespace_part = self.family_namespaces[0].clone();
        format!("{}{}", namespace_part, &payload_part[..64])
    }

    fn store_message_at(&self, message: &[u8], state_address: &str, context: &mut dyn TransactionContext)
        -> Result<(), ApplyError> {
        let destination_address = String::from(state_address);
        let message_bytes = message.to_vec();
        match context.set_state_entries(vec![(destination_address, message_bytes)]) {
            Ok(()) => Ok(()),
            Err(e) => Err(ApplyError::InternalError(format!(
                "Internal error while trying to access state address {}. Error was {}",
                state_address, e
            )))
        }
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

        let payload_bytes = request.get_payload().to_vec();
        let message = match Message::from(payload_bytes) {
            Ok(m) => m,
            Err(e) => return Err(e),
        };

        let transaction_address = self.state_address_for(&message);
        let state_entries = match context.get_state_entries(&vec![transaction_address.clone()][..]) {
            Ok(entries) => entries,
            Err(e) => return Err(InternalError(
                    format!("Internal error while trying to access state address {}. Error was {}",
                            &transaction_address, e)))
        };

        let state_entry_count = state_entries.len();
        match state_entry_count {
            0 => self.store_message_at(
                &message.message,
                transaction_address.as_str(),
                context
            ),
            1 => Err(InvalidTransaction(format!("Message with address {} already exists", &transaction_address))),
            _ => Err(InternalError(format!("Inconsistent state detected: address {} refers to {} entries",
                                           &transaction_address, state_entry_count)))
        }
    }
}

#[cfg(test)]
mod test {
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

        context
            .expect_get_state_entries()
            .times(1)
            .returning(|_addresses| Ok(vec![]));
        context
            .expect_set_state_entries()
            .times(1)
            .returning(|_entries| Ok(()));

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

        let address = handler.state_address_for(&message);

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

        let address = handler.state_address_for(&message);

        assert_eq!(address.as_bytes().len(), 70);
    }

    #[test]
    fn generated_address_starts_with_transaction_family_namespace() {
        let handler = Handler::new();
        let message = Message {
            agent_id: String::from("id"),
            message_type: String::from("type"),
            message: String::from("").as_bytes().to_vec(),
            timestamp: String::from("684984984984"),
        };

        let address = handler.state_address_for(&message);

        let mut hasher = sha2::Sha512::new();
        hasher.update(handler.family_name());
        let namespace = data_encoding::HEXLOWER.encode(&hasher.finalize()[..]);

        assert!(address.starts_with(&namespace[..6]))
    }

    #[test]
    fn apply_adds_non_existing_entry() {
        let handler = Handler::new();

        let mut request = TpProcessRequest::new();

        let mut context = MockContext::new();
        context
            .expect_get_state_entries()
            .times(1)
            .returning(|_addresses| Ok(vec![]));
        context
            .expect_set_state_entries()
            .times(1)
            .returning(|_entries| Ok(()));

        let mut header = sawtooth_sdk::messages::transaction::TransactionHeader::new();
        header.set_signer_public_key(String::from("980490840984984"));
        request.set_header(header);
        request.set_payload("id|type|msg|ts".as_bytes().to_vec());

        handler.apply(&request, &mut context).unwrap();
    }

    #[test]
    fn apply_fails_with_existing_entry() {
        let handler = Handler::new();

        let mut request = TpProcessRequest::new();
        let mut context = MockContext::new();
        context
            .expect_get_state_entries()
            .times(1)
            .returning(|addresses| {
                let mut entries = Vec::new();
                for addr in addresses {
                    entries.push((addr.clone(), vec![0x0]));
                }

                Ok(entries)
            });
        context.expect_set_state_entries().times(0);

        let mut header = sawtooth_sdk::messages::transaction::TransactionHeader::new();
        header.set_signer_public_key(String::from("980490840984984"));
        request.set_header(header);
        request.set_payload("id|type|msg|ts".as_bytes().to_vec());

        handler.apply(&request, &mut context).unwrap_err();
    }

    #[test]
    fn apply_fails_if_multiple_entries_exist() {
        let handler = Handler::new();

        let mut request = TpProcessRequest::new();

        let mut context = MockContext::new();
        context
            .expect_get_state_entries()
            .times(1)
            .returning(|addresses| {
                let mut entries = Vec::new();
                for addr in addresses {
                    entries.push((addr.clone(), vec![0x0]));
                    entries.push((addr.clone(), vec![0x1]));
                }

                Ok(entries)
            });

        let mut header = sawtooth_sdk::messages::transaction::TransactionHeader::new();
        header.set_signer_public_key(String::from("980490840984984"));
        request.set_header(header);
        request.set_payload("id|type|msg|ts".as_bytes().to_vec());

        handler.apply(&request, &mut context).unwrap_err();
    }
}