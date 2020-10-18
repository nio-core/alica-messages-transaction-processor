use crate::{payload::TransactionPayload, sawtooth, util};
use sawtooth_sdk::messages::processor::TpProcessRequest;
use sawtooth_sdk::processor::handler::ApplyError::InvalidTransaction;
use sawtooth_sdk::processor::handler::{ApplyError, TransactionContext, TransactionHandler};

use crate::payload::{parser::PipeSeparatedPayloadParser, Parser};

pub struct AlicaMessageTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    family_namespaces: Vec<String>,
}

impl AlicaMessageTransactionHandler {
    pub fn new() -> Self {
        let family_name = "alica_messages";
        let family_name_hash = util::hash(family_name);

        AlicaMessageTransactionHandler {
            family_name: String::from(family_name),
            family_versions: vec![String::from("0.1.0")],
            family_namespaces: vec![String::from(&family_name_hash[..6])],
        }
    }

    fn parse_pipe_separated(
        &self,
        transaction_payload_bytes: &[u8],
    ) -> Result<TransactionPayload, ApplyError> {
        PipeSeparatedPayloadParser::new()
            .parse(transaction_payload_bytes)
            .map_err(|e| InvalidTransaction(format!("Error parsing payload: {}", e)))
    }

    fn state_address_for(&self, transaction_payload: &TransactionPayload) -> String {
        let payload_part_of_state_address = format!(
            "{}{}{}",
            transaction_payload.agent_id,
            transaction_payload.message_type,
            transaction_payload.timestamp
        );
        let payload_checksum = util::hash(&payload_part_of_state_address);

        let first_64_bytes_of_payload_checksum = &payload_checksum[..64];
        let first_6_bytes_of_namespace_checksum = &self.family_namespaces[0];

        format!(
            "{}{}",
            first_6_bytes_of_namespace_checksum, first_64_bytes_of_payload_checksum
        )
    }
}

impl TransactionHandler for AlicaMessageTransactionHandler {
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

        let transaction_payload_bytes = request.get_payload();
        let transaction_payload = self.parse_pipe_separated(transaction_payload_bytes)?;

        let transaction_address = self.state_address_for(&transaction_payload);
        let sawtooth_interactor = sawtooth::Interactor::new(context);
        sawtooth_interactor.create_state_entry(&transaction_address,
                                               &transaction_payload.message_bytes)
    }
}

#[cfg(test)]
mod test {
    use crate::payload::TransactionPayload;
    use crate::{handler::AlicaMessageTransactionHandler, util};

    use sawtooth_sdk::messages;
    use sawtooth_sdk::messages::processor::TpProcessRequest;
    use sawtooth_sdk::messages::transaction::TransactionHeader;
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
    fn apply_with_invalid_utf8_payload_fails_with_apply_error() {
        let handler = AlicaMessageTransactionHandler::new();

        let mut request = TpProcessRequest::new();
        let mut context = MockContext::new();

        let mut header = TransactionHeader::new();
        header.set_signer_public_key(String::from("980490840984984"));
        request.set_header(header);
        request.set_payload(vec![0xff, 0xff]);

        handler.apply(&request, &mut context).unwrap_err();
    }

    #[test]
    fn apply_with_validly_structured_payload_succeeds() {
        let handler = AlicaMessageTransactionHandler::new();

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

        let mut header = TransactionHeader::new();
        header.set_signer_public_key(String::from("980490840984984"));
        request.set_header(header);
        request.set_payload("id|type|msg|64984494984".as_bytes().to_vec());

        handler.apply(&request, &mut context).unwrap();
    }

    #[test]
    fn generated_address_is_70_bytes_in_size() {
        let handler = AlicaMessageTransactionHandler::new();
        let payload = TransactionPayload::new("id", "type", "message".as_bytes(), 6876984987987989);

        let address = handler.state_address_for(&payload);

        assert_eq!(address.as_bytes().len(), 70);
    }

    #[test]
    fn generated_address_for_empty_message_is_70_bytes_in_size() {
        let handler = AlicaMessageTransactionHandler::new();
        let payload = TransactionPayload::new("id", "type", "".as_bytes(), 6876984987987989);

        let address = handler.state_address_for(&payload);

        assert_eq!(address.as_bytes().len(), 70);
    }

    #[test]
    fn generated_address_starts_with_transaction_family_namespace() {
        let handler = AlicaMessageTransactionHandler::new();
        let payload = TransactionPayload::new("id", "type", "message".as_bytes(), 6876984987987989);

        let address = handler.state_address_for(&payload);

        let namespace = util::hash(&handler.family_name());
        assert!(address.starts_with(&namespace[..6]))
    }

    #[test]
    fn apply_adds_non_existing_entry() {
        let handler = AlicaMessageTransactionHandler::new();
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

        let mut header = messages::transaction::TransactionHeader::new();
        header.set_signer_public_key(String::from("980490840984984"));
        request.set_header(header);
        request.set_payload("id|type|msg|498498498".as_bytes().to_vec());

        handler.apply(&request, &mut context).unwrap();
    }

    #[test]
    fn apply_fails_with_existing_entry() {
        let handler = AlicaMessageTransactionHandler::new();

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

        let mut header = messages::transaction::TransactionHeader::new();
        header.set_signer_public_key(String::from("980490840984984"));
        request.set_header(header);
        request.set_payload("id|type|msg|65494894949".as_bytes().to_vec());

        handler.apply(&request, &mut context).unwrap_err();
    }

    #[test]
    fn apply_fails_if_multiple_entries_exist() {
        let handler = AlicaMessageTransactionHandler::new();
        let mut request = TpProcessRequest::new();
        let mut context = MockContext::new();
        context
            .expect_get_state_entries()
            .times(1)
            .returning(|addresses| {
                let mut entries = Vec::new();
                for address in addresses {
                    entries.push((address.clone(), vec![0x0]));
                    entries.push((address.clone(), vec![0x1]));
                }

                Ok(entries)
            });

        let mut header = messages::transaction::TransactionHeader::new();
        header.set_signer_public_key(String::from("980490840984984"));
        request.set_header(header);
        request.set_payload("id|type|msg|89891819".as_bytes().to_vec());

        handler.apply(&request, &mut context).unwrap_err();
    }
}
