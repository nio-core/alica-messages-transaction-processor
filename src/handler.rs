use crate::{sawtooth, util};
use sawtooth_sdk::messages::processor::TpProcessRequest;
use sawtooth_sdk::processor::handler::ApplyError::InvalidTransaction;
use sawtooth_sdk::processor::handler::{ApplyError, TransactionContext, TransactionHandler};
use sawtooth_alica_message_transaction_payload::messages::AlicaMessageJsonValidator;

use std::collections::HashMap;
use sawtooth_alica_message_transaction_payload::payloads;

pub struct AlicaMessageTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    family_namespaces: Vec<String>,
    transaction_payload_parser: Box<dyn payloads::Parser>,
    alica_message_validators: HashMap<String, Box<dyn AlicaMessageJsonValidator>>
}

impl AlicaMessageTransactionHandler {
    pub fn new(transaction_payload_parser: Box<dyn payloads::Parser>) -> Self {
        let family_name = "alica_messages";
        let family_name_hash = util::hash(family_name);

        AlicaMessageTransactionHandler {
            family_name: String::from(family_name),
            family_versions: vec![String::from("0.1.0")],
            family_namespaces: vec![String::from(&family_name_hash[..6])],
            transaction_payload_parser,
            alica_message_validators: HashMap::new()
        }
    }

    pub fn with_validator_for(&mut self, message_type: &str, message_parser: Box<dyn AlicaMessageJsonValidator>) -> &mut Self {
        self.alica_message_validators.insert(message_type.to_string(), message_parser);
        self
    }

    fn parse_pipe_separated(&self, transaction_payload_bytes: &[u8]) -> Result<payloads::TransactionPayload, ApplyError> {
        println!("Parsing received payload");
        let parsing_result = self.transaction_payload_parser
            .parse(transaction_payload_bytes)
            .map_err(|e| InvalidTransaction(format!("Error parsing payload: {}", e)));
        println!("-> Payload format valid: Pipe separated");
        parsing_result
    }

    fn state_address_for(&self, transaction_payload: &payloads::TransactionPayload) -> String {
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

    fn validate_contained_message(&self, payload: &payloads::TransactionPayload) -> Result<(), ApplyError> {
        println!("Validating message for type {}", payload.message_type);
        let message_validator = self.alica_message_validators.get(&payload.message_type)
            .ok_or_else(|| InvalidTransaction(format!("No matching message validator for {} available", &payload.message_type)))?;
        let validation_result = message_validator.validate(&payload.message_bytes)
            .map_err(|e| InvalidTransaction(e.into()));
        println!("-> Validation successful");
        validation_result
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

    fn apply(&self, request: &TpProcessRequest, context: &mut dyn TransactionContext) -> Result<(), ApplyError> {
        let transaction_payload_bytes = request.get_payload();
        let transaction_signer = request.get_header().get_signer_public_key();
        println!(
            "Transaction received from {}!\n-> Payload is {}", &transaction_signer[..6],
            String::from_utf8(transaction_payload_bytes.to_vec()).map_err(|e| InvalidTransaction(format!("Invalid transaction,  error was {}", e)))?
        );

        let transaction_payload = self.parse_pipe_separated(transaction_payload_bytes)?;

        self.validate_contained_message(&transaction_payload)?;

        let transaction_address = self.state_address_for(&transaction_payload);
        let transaction_applicator = sawtooth::TransactionApplicator::new(context);
        println!("Trying to create state entry for address {}", &transaction_address);
        transaction_applicator.create_at(&transaction_payload.message_bytes, &transaction_address)?;
        println!("-> State entry created successfully");

        Ok(())
    }
}

#[cfg(test)]
mod test {
    mod state_address_generation {
        use crate::util;
        use crate::handler::AlicaMessageTransactionHandler;
        use sawtooth_alica_message_transaction_payload::payloads;

        fn transaction_handler() -> AlicaMessageTransactionHandler {
            let transaction_payload_parser: Box<dyn payloads::Parser> = Box::new(payloads::MockParser::new());
            AlicaMessageTransactionHandler::new(transaction_payload_parser)
        }

        #[test]
        fn generated_address_is_70_bytes_in_size() {
            let parsed_payload = payloads::TransactionPayload::default();

            let state_address = transaction_handler().state_address_for(&parsed_payload);

            assert_eq!(state_address.len(), 70)
        }

        #[test]
        fn generated_address_starts_with_transaction_family_namespace() {
            let parsed_payload = payloads::TransactionPayload::default();

            let state_address = transaction_handler().state_address_for(&parsed_payload);

            let expected_namespace = &transaction_handler().family_namespaces[0];
            assert!(state_address.starts_with(expected_namespace))
        }

        #[test]
        fn generated_address_ends_with_a_hash_built_from_the_transaction_payload_meta_data() {
            let parsed_payload = payloads::TransactionPayload::default();

            let state_address = transaction_handler().state_address_for(&parsed_payload);

            let expected_transaction_part = format!("{}{}{}", parsed_payload.agent_id, parsed_payload.message_type, parsed_payload.timestamp);
            let transaction_part = &util::hash(&expected_transaction_part)[..64];
            assert!(state_address.ends_with(transaction_part))
        }
    }

    mod transaction_application {
        use crate::handler::AlicaMessageTransactionHandler;
        use crate::testing;
        use sawtooth_sdk::processor::handler::TransactionHandler;
        use sawtooth_sdk::messages::processor::TpProcessRequest;
        use sawtooth_sdk::messages::transaction::TransactionHeader;
        use sawtooth_alica_message_transaction_payload::messages::{AlicaMessageValidationError,
                                                                   MockAlicaMessageJsonValidator};
        use sawtooth_alica_message_transaction_payload::payloads::{MockParser, TransactionPayload, ParsingError};

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

            let mut alica_message_parser = MockAlicaMessageJsonValidator::new();
            alica_message_parser.expect_validate().times(1).returning(|_| Ok(()));

            let mut transaction_handler = AlicaMessageTransactionHandler::new(transaction_payload_parser);
            transaction_handler.with_validator_for("", Box::from(alica_message_parser));

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

            let mut alica_message_parser = MockAlicaMessageJsonValidator::new();
            alica_message_parser.expect_validate().times(0).returning(|_| Ok(()));

            let mut transaction_handler = AlicaMessageTransactionHandler::new(transaction_payload_parser);
            transaction_handler.with_validator_for("", Box::from(alica_message_parser));

            let request = transaction_processing_request();
            let mut context = testing::MockTransactionContext::new();
            context.expect_get_state_entries().times(0);
            context.expect_set_state_entries().times(0);

            let transaction_application_result = transaction_handler.apply(&request, &mut context);

            assert!(transaction_application_result.is_err())
        }

        #[test]
        fn apply_does_not_add_transaction_its_contained_message_is_not_valid() {
            let mut transaction_payload_parser = Box::new(MockParser::new());
            transaction_payload_parser.expect_parse().times(1).returning(|_| Ok(TransactionPayload::default()));

            let mut alica_message_parser = MockAlicaMessageJsonValidator::new();
            alica_message_parser.expect_validate().times(1).returning(|_| Err(AlicaMessageValidationError::InvalidFormat("".to_string())));

            let mut transaction_handler = AlicaMessageTransactionHandler::new(transaction_payload_parser);
            transaction_handler.with_validator_for("", Box::from(alica_message_parser));

            let request = transaction_processing_request();
            let mut context = testing::MockTransactionContext::new();
            context.expect_get_state_entries().times(0).returning(|_| Ok(vec![]));
            context.expect_set_state_entries().times(0).returning(|_| Ok(()));

            let transaction_application_result = transaction_handler.apply(&request, &mut context);

            assert!(transaction_application_result.is_err())
        }

        #[test]
        fn apply_does_not_add_transaction_if_no_validator_for_the_contained_message_type_is_available() {
            let mut transaction_payload_parser = Box::new(MockParser::new());
            transaction_payload_parser.expect_parse().times(1).returning(|_| Ok(TransactionPayload::default()));

            let transaction_handler = AlicaMessageTransactionHandler::new(transaction_payload_parser);

            let request = transaction_processing_request();
            let mut context = testing::MockTransactionContext::new();
            context.expect_get_state_entries().times(0).returning(|_| Ok(vec![]));
            context.expect_set_state_entries().times(0).returning(|_| Ok(()));

            let transaction_application_result = transaction_handler.apply(&request, &mut context);

            assert!(transaction_application_result.is_err())
        }
    }
}
