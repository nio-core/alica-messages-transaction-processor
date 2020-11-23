use crate::sawtooth;
use sawtooth_sdk::messages::processor::TpProcessRequest;
use sawtooth_sdk::processor::handler::ApplyError::InvalidTransaction;
use sawtooth_sdk::processor::handler::{ApplyError, TransactionContext, TransactionHandler};
use sawtooth_alica_message_transaction_payload::messages::AlicaMessageJsonValidator;

use std::collections::HashMap;
use sawtooth_alica_message_transaction_payload::{payloads, TransactionFamily};
use sawtooth_alica_message_transaction_payload::payloads::TransactionPayload;

pub struct AlicaMessageTransactionHandler {
    family: TransactionFamily,
    payload_format: Box<dyn payloads::Format>,
    alica_message_validators: HashMap<String, Box<dyn AlicaMessageJsonValidator>>
}

impl AlicaMessageTransactionHandler {
    pub fn new(family: TransactionFamily, payload_format: Box<dyn payloads::Format>) -> Self {
        AlicaMessageTransactionHandler {
            family,
            payload_format,
            alica_message_validators: HashMap::new()
        }
    }

    pub fn with_validator_for(&mut self, message_type: &str, message_parser: Box<dyn AlicaMessageJsonValidator>) -> &mut Self {
        self.alica_message_validators.insert(message_type.to_string(), message_parser);
        self
    }

    fn parse_payload(&self, transaction_payload_bytes: &[u8]) -> Result<payloads::TransactionPayload, ApplyError> {
        println!("Parsing received payload");
        let parsing_result = self.payload_format
            .deserialize(transaction_payload_bytes)
            .map_err(|e| InvalidTransaction(format!("Error parsing payload: {}", e)));
        println!("-> Payload format valid: Pipe separated");
        parsing_result
    }

    fn try_create_state_entry_for(&self, context: &mut dyn TransactionContext, payload: &TransactionPayload) -> Result<(), ApplyError> {
        let transaction_address = self.family.calculate_state_address_for(&payload);
        let transaction_applicator = sawtooth::TransactionApplicator::new(context);

        println!("Trying to create state entry for address {}", &transaction_address);
        transaction_applicator.create_at(&payload.message_bytes, &transaction_address)?;
        println!("-> State entry created successfully");

        Ok(())
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
        self.family.name.clone()
    }

    fn family_versions(&self) -> Vec<String> {
        self.family.versions.clone()
    }

    fn namespaces(&self) -> Vec<String> {
        vec![self.family.calculate_namespace()]
    }

    fn apply(&self, request: &TpProcessRequest, context: &mut dyn TransactionContext) -> Result<(), ApplyError> {
        let transaction_payload_bytes = request.get_payload();
        let transaction_signer = request.get_header().get_signer_public_key();
        println!(
            "Transaction received from {}!\n-> Payload is {}", &transaction_signer[..6],
            String::from_utf8(transaction_payload_bytes.to_vec()).map_err(|e| InvalidTransaction(format!("Invalid transaction,  error was {}", e)))?
        );

        let transaction_payload = self.parse_payload(transaction_payload_bytes)?;
        self.validate_contained_message(&transaction_payload)?;
        self.try_create_state_entry_for(context, &transaction_payload)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::handler::AlicaMessageTransactionHandler;
    use crate::testing;
    use sawtooth_sdk::processor::handler::TransactionHandler;
    use sawtooth_sdk::messages::processor::TpProcessRequest;
    use sawtooth_sdk::messages::transaction::TransactionHeader;
    use sawtooth_alica_message_transaction_payload::messages::{AlicaMessageValidationError,
                                                               MockAlicaMessageJsonValidator};
    use sawtooth_alica_message_transaction_payload::{payloads, TransactionFamily};

    fn transaction_processing_request() -> TpProcessRequest {
        let mut transaction_header = TransactionHeader::new();
        transaction_header.set_signer_public_key("SomeKey".to_string());
        let mut request = TpProcessRequest::new();
        request.set_header(transaction_header);
        request.set_payload("".as_bytes().to_vec());
        request
    }

    fn transaction_family() -> TransactionFamily {
        TransactionFamily::new("alica_messages", &vec!["0.1.0".to_string()])
    }

    #[test]
    fn apply_adds_transaction_if_it_is_well_structured() {
        let mut transaction_payload_format = Box::new(payloads::MockFormat::new());
        transaction_payload_format.expect_deserialize().times(1).returning(|_| Ok(payloads::TransactionPayload::default()));

        let mut alica_message_parser = MockAlicaMessageJsonValidator::new();
        alica_message_parser.expect_validate().times(1).returning(|_| Ok(()));

        let mut transaction_handler = AlicaMessageTransactionHandler::new(transaction_family(), transaction_payload_format);
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
        let mut transaction_payload_format = Box::new(payloads::MockFormat::new());
        transaction_payload_format.expect_deserialize().times(1).returning(|_| Err(payloads::Error::InvalidPayload("".to_string())));

        let mut alica_message_parser = MockAlicaMessageJsonValidator::new();
        alica_message_parser.expect_validate().times(0).returning(|_| Ok(()));

        let mut transaction_handler = AlicaMessageTransactionHandler::new(transaction_family(), transaction_payload_format);
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
        let mut transaction_payload_format = Box::new(payloads::MockFormat::new());
        transaction_payload_format.expect_deserialize().times(1).returning(|_| Ok(payloads::TransactionPayload::default()));

        let mut alica_message_parser = MockAlicaMessageJsonValidator::new();
        alica_message_parser.expect_validate().times(1).returning(|_| Err(AlicaMessageValidationError::InvalidFormat("".to_string())));

        let mut transaction_handler = AlicaMessageTransactionHandler::new(transaction_family(), transaction_payload_format);
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
        let mut transaction_payload_format = Box::new(payloads::MockFormat::new());
        transaction_payload_format.expect_deserialize().times(1).returning(|_| Ok(payloads::TransactionPayload::default()));

        let transaction_handler = AlicaMessageTransactionHandler::new(transaction_family(), transaction_payload_format);

        let request = transaction_processing_request();
        let mut context = testing::MockTransactionContext::new();
        context.expect_get_state_entries().times(0).returning(|_| Ok(vec![]));
        context.expect_set_state_entries().times(0).returning(|_| Ok(()));

        let transaction_application_result = transaction_handler.apply(&request, &mut context);

        assert!(transaction_application_result.is_err())
    }
}
