use mockall;

pub struct AlicaMessageValidationError {
    message_type: String,
    missing_fields: Vec<String>
}

impl AlicaMessageValidationError {
    pub fn new(message_type: &str) -> Self {
        AlicaMessageValidationError {
            message_type: message_type.to_string(),
            missing_fields: Vec::new()
        }
    }
}

impl Into<String> for AlicaMessageValidationError {
    fn into(self) -> String {
        format!("Validation of {} failed. The following fields were missing: {}", self.message_type, self.missing_fields.join(", "))
    }
}

pub type AlicaMessageValidationResult = Result<(), AlicaMessageValidationError>;

#[mockall::automock]
pub trait AlicaMessageJsonValidator {
    fn parse_alica_message(&self, message: &[u8]) -> AlicaMessageValidationResult;
}

pub struct AlicaEngineInfoValidator {}

impl AlicaEngineInfoValidator {
    pub fn new() -> Self {
        AlicaEngineInfoValidator {}
    }
}

impl AlicaMessageJsonValidator for AlicaEngineInfoValidator {
    fn parse_alica_message(&self, message: &[u8]) -> AlicaMessageValidationResult {
        Ok(())
    }
}
