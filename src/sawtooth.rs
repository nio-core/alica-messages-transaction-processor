use sawtooth_sdk::processor::handler::{
    ApplyError::{self, InternalError},
    TransactionContext,
};

pub struct TransactionApplicator<'a> {
    context: &'a mut dyn TransactionContext,
}

impl<'a> TransactionApplicator<'a> {
    pub fn new(context: &'a mut dyn TransactionContext) -> Self {
        TransactionApplicator { context }
    }

    pub fn fetch_state_entries(
        &self,
        state_address: &str,
    ) -> Result<Vec<(String, Vec<u8>)>, ApplyError> {
        self.context
            .get_state_entries(&vec![state_address.to_string()])
            .map_err(|e| {
                InternalError(format!(
                    "Internal error while trying to access state address {}. Error was {}",
                    &state_address, e
                ))
            })
    }

    pub fn create_state_entry(&self, state_address: &str, data: &[u8]) -> Result<(), ApplyError> {
        let state_entries = self.fetch_state_entries(&state_address)?;

        let state_entry_count = state_entries.len();
        match state_entry_count {
            0 => self.store_state_entry((state_address, data)),
            1 => Err(InternalError(format!(
                "Message with address {} already exists",
                state_address
            ))),
            _ => Err(InternalError(format!(
                "Inconsistent state detected: address {} refers to {} entries",
                state_address, state_entry_count
            ))),
        }
    }

    fn store_state_entry(&self, state_entry: (&str, &[u8])) -> Result<(), ApplyError> {
        let destination_address = String::from(state_entry.0);
        let data = state_entry.1.to_vec();
        self.context
            .set_state_entries(vec![(destination_address, data)])
            .map_err(|e| {
                ApplyError::InternalError(format!(
                    "Internal error while trying to access state address {}. Error was {}",
                    state_entry.0, e
                ))
            })
    }
}

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

        assert!(transaction_applicator.create_state_entry(address, value).is_ok())
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

        assert!(transaction_applicator.create_state_entry(address, value).is_err())
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

        assert!(transaction_applicator.create_state_entry(address, value).is_err())
    }
}
