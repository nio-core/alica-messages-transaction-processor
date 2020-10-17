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

    pub fn store_state_entry(&self, state_entry: (&str, &[u8])) -> Result<(), ApplyError> {
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

    pub fn get_state_entries_for(
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
}
