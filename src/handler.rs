use crate::{sawtooth, util, payload};
use sawtooth_sdk::messages::processor::TpProcessRequest;
use sawtooth_sdk::processor::handler::ApplyError::InvalidTransaction;
use sawtooth_sdk::processor::handler::{ApplyError, TransactionContext, TransactionHandler};

pub struct AlicaMessageTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    family_namespaces: Vec<String>,
    transaction_payload_parser: Box<dyn payload::Parser>,

}

impl AlicaMessageTransactionHandler {
    pub fn new(transaction_payload_parser: Box<dyn payload::Parser>) -> Self {
        let family_name = "alica_messages";
        let family_name_hash = util::hash(family_name);

        AlicaMessageTransactionHandler {
            family_name: String::from(family_name),
            family_versions: vec![String::from("0.1.0")],
            family_namespaces: vec![String::from(&family_name_hash[..6])],
            transaction_payload_parser
        }
    }

    fn parse_pipe_separated(
        &self,
        transaction_payload_bytes: &[u8],
    ) -> Result<payload::TransactionPayload, ApplyError> {
        self.transaction_payload_parser
            .parse(transaction_payload_bytes)
            .map_err(|e| InvalidTransaction(format!("Error parsing payload: {}", e)))
    }

    fn state_address_for(&self, transaction_payload: &payload::TransactionPayload) -> String {
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
        let transaction_applicator = sawtooth::TransactionApplicator::new(context);
        transaction_applicator.create_state_entry(&transaction_address,
                                                  &transaction_payload.message_bytes)
    }
}

#[cfg(test)]
mod test {
    mod state_address_generation {
        use crate::{payload, util};
        use crate::handler::AlicaMessageTransactionHandler;

        fn transaction_handler() -> AlicaMessageTransactionHandler {
            let transaction_payload_parser: Box<dyn payload::Parser> = Box::new(payload::MockParser::new());
            AlicaMessageTransactionHandler::new(transaction_payload_parser)
        }

        #[test]
        fn generated_address_is_70_bytes_in_size() {
            let parsed_payload = payload::TransactionPayload::default();

            let state_address = transaction_handler().state_address_for(&parsed_payload);

            assert_eq!(state_address.len(), 70)
        }

        #[test]
        fn generated_address_starts_with_transaction_family_namespace() {
            let parsed_payload = payload::TransactionPayload::default();

            let state_address = transaction_handler().state_address_for(&parsed_payload);

            let expected_namespace = &transaction_handler().family_namespaces[0];
            assert!(state_address.starts_with(expected_namespace))
        }

        #[test]
        fn generated_address_ends_with_a_hash_built_from_the_transaction_payload_meta_data() {
            let parsed_payload = payload::TransactionPayload::default();

            let state_address = transaction_handler().state_address_for(&parsed_payload);

            let expected_transaction_part = format!("{}{}{}", parsed_payload.agent_id, parsed_payload.message_type, parsed_payload.timestamp);
            let transaction_part = &util::hash(&expected_transaction_part)[..64];
            assert!(state_address.ends_with(transaction_part))
        }
    }

    mod transaction_application {
        use crate::handler::AlicaMessageTransactionHandler;
        use crate::payload::{MockParser, TransactionPayload, ParsingError};
        use sawtooth_sdk::processor::handler::TransactionHandler;
        use sawtooth_sdk::messages::processor::TpProcessRequest;
        use sawtooth_sdk::messages::transaction::TransactionHeader;
        use crate::testing;

        fn transaction_processing_request() -> TpProcessRequest {
            let mut transaction_header = TransactionHeader::new();
            transaction_header.set_signer_public_key("SomeKey".to_string());
            let mut request = TpProcessRequest::new();
            request.set_header(transaction_header);
            request.set_payload("".as_bytes().to_vec());
            request
        }

        #[test]
        fn apply_adds_transaction_if_it_is_well_structured() {
            let mut transaction_payload_parser = Box::new(MockParser::new());
            transaction_payload_parser.expect_parse().times(1).returning(|_| Ok(TransactionPayload::default()));
            let transaction_handler = AlicaMessageTransactionHandler::new(transaction_payload_parser);
            let request = transaction_processing_request();
            let mut context = testing::MockTransactionContext::new();
            context.expect_get_state_entries().times(1).returning(|_| Ok(vec![]));
            context.expect_set_state_entries().times(1).returning(|_| Ok(()));

            let transaction_application_result = transaction_handler.apply(&request, &mut context);

            assert!(transaction_application_result.is_ok())
        }

        #[test]
        fn apply_does_not_add_transaction_if_it_is_not_well_structured() {
            let mut transaction_payload_parser = Box::new(MockParser::new());
            transaction_payload_parser.expect_parse().times(1).returning(|_| Err(ParsingError::InvalidPayload("".to_string())));
            let transaction_handler = AlicaMessageTransactionHandler::new(transaction_payload_parser);
            let request = transaction_processing_request();
            let mut context = testing::MockTransactionContext::new();
            context.expect_get_state_entries().times(0);
            context.expect_set_state_entries().times(0);

            let transaction_application_result = transaction_handler.apply(&request, &mut context);

            assert!(transaction_application_result.is_err())
        }
    }
}
