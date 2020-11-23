use sawtooth_sdk::processor::handler::TransactionContext;
use sawtooth_sdk::processor::handler::ApplyError::{self, InvalidTransaction};

pub struct TransactionApplicator<'a> {
    context: &'a mut dyn TransactionContext,
}

impl<'a> TransactionApplicator<'a> {
    pub fn new(context: &'a mut dyn TransactionContext) -> Self {
        TransactionApplicator { context }
    }

    pub fn create_at(&self, data: &[u8], state_address: &str) -> Result<(), ApplyError> {
        let state_entry = self.fetch(&state_address)?;
        match state_entry {
            Some(_) => Err(InvalidTransaction(format!("Message with address {} already exists", state_address))),
            None => self.store_at(data, state_address)
        }
    }

    pub fn fetch(&self, state_address: &str) -> Result<Option<Vec<u8>>, ApplyError> {
        self.context.get_state_entry(state_address)
            .map_err(|error| ApplyError::from(error))
    }

    fn store_at(&self, data: &[u8], address: &str) -> Result<(), ApplyError> {
        self.context
            .set_state_entry(address.to_string(), data.to_vec())
            .map_err(|error| ApplyError::from(error))
    }
}

#[cfg(test)]
mod test {
    use super::TransactionApplicator;
    use crate::testing::MockTransactionContext;

    #[test]
    fn apply_adds_transaction_if_no_state_entry_exists_for_the_address() {
        let address = "addr";
        let value = "value".as_bytes();
        let mut context = MockTransactionContext::new();
        context.expect_get_state_entries().times(1).returning(|_| Ok(vec![]));
        context.expect_set_state_entries().times(1).returning(|_| Ok(()));
        let transaction_applicator = TransactionApplicator::new(&mut context);

        let transaction_application_result = transaction_applicator.create_at(value, address);

        assert!(transaction_application_result.is_ok())
    }

    #[test]
    fn apply_does_not_add_the_transaction_if_a_single_state_entry_exists_for_the_address() {
        let address = "addr";
        let value = "value".as_bytes();
        let mut context = MockTransactionContext::new();
        context.expect_get_state_entries().times(1)
            .returning(move |_| Ok(vec![(String::from(address), value.to_vec())]));
        context.expect_set_state_entries().times(0);
        let transaction_applicator = TransactionApplicator::new(&mut context);

        let transaction_application_result = transaction_applicator.create_at(value, address);

        assert!(transaction_application_result.is_err())
    }

    #[test]
    fn apply_does_not_add_the_transaction_if_multiple_state_entries_exists_for_the_address() {
        let address = "addr";
        let value = "value".as_bytes();
        let mut context = MockTransactionContext::new();
        context.expect_get_state_entries().times(1)
            .returning(move |_| Ok(vec![
                (String::from(address), value.to_vec()),
                (String::from(address), value.to_vec())
            ]));
        context.expect_set_state_entries().times(0);
        let transaction_applicator = TransactionApplicator::new(&mut context);

        let transaction_application_result = transaction_applicator.create_at(value, address);

        assert!(transaction_application_result.is_err())
    }
}
