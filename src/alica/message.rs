use sawtooth_sdk::messages::processor::TpProcessRequest;
use sawtooth_sdk::processor::handler::{ApplyError, TransactionContext};
use sha2::Digest;

pub struct Handler {
    family_name: String,
    family_versions: Vec<String>,
    family_namespaces: Vec<String>,
}

impl Handler {
    pub fn new() -> Self {
        let family_name = "alica_messages";
        let mut hasher = sha2::Sha512::new();
        hasher.input(family_name);
        let result = hasher.result();

        let namespace = data_encoding::HEXLOWER.encode(&result[..6]);

        Handler {
            family_name: String::from(family_name),
            family_versions: vec![String::from("0.1.0")],
            family_namespaces: vec![namespace],
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

        let payload = match String::from_utf8(request.get_payload().to_vec()) {
            Ok(payload) => payload,
            Err(_e) => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Failed to decode payload in UTF8",
                )))
            }
        };

        /*
            Messages:
            {
                agentId: String,
                type: MessageTypeEnum,
                Timestamp: Time,
                message: Bytes
            }
        */

        let message = Message::from(payload);
        let address = address_for(&message);
        match context.get_state_entry(address) {
            Ok()
        };

        // Addresses: AgentId|MessageType|Timestamp

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sawtooth_sdk::messages::processor::TpProcessRequest;
    use sawtooth_sdk::processor::handler::{ContextError, TransactionContext, TransactionHandler};

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
    fn apply_with_valid_utf8_payload_succeeds() {
        let handler = Handler::new();

        let mut request = TpProcessRequest::new();
        let mut context = MockContext::new();

        let mut header = sawtooth_sdk::messages::transaction::TransactionHeader::new();
        header.set_signer_public_key(String::from("980490840984984"));
        request.set_header(header);
        request.set_payload(vec![0x0]);

        handler.apply(&request, &mut context).unwrap();
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
        request.set_payload(String::from("id1|mt|202006162222").as_bytes().to_vec());

        handler.apply(&request, &mut context).unwrap();
    }
}
