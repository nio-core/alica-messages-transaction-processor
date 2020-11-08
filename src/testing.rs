use mockall;
use sawtooth_sdk::processor::handler::{ContextError, TransactionContext};

mockall::mock! {
    pub TransactionContext {}

    trait TransactionContext {
        fn get_state_entries(&self, addresses: &[String]) -> Result<Vec<(String, Vec<u8>)>, ContextError>;
        fn set_state_entries(&self, entries: Vec<(String, Vec<u8>)>) -> Result<(), ContextError>;
        fn delete_state_entries(&self, addresses: &[String]) -> Result<Vec<String>, ContextError>;
        fn add_receipt_data(&self, data: &[u8]) -> Result<(), ContextError>;
        fn add_event(&self, address: String, entries: Vec<(String, String)>, data: &[u8]) -> Result<(), ContextError>;
    }
}