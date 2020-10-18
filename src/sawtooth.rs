use sawtooth_sdk::processor::handler::{
    ApplyError::{self, InternalError},
    TransactionContext,
};

pub struct Interactor<'a> {
    context: &'a mut dyn TransactionContext,
}

impl<'a> Interactor<'a> {
    pub fn new(context: &'a mut dyn TransactionContext) -> Self {
        Interactor { context }
    }

    pub fn fetch_state_entries_for(
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
        let state_entries = self.fetch_state_entries_for(&state_address)?;

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
