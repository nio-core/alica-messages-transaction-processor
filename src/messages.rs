use mockall;
use crate::messages::AlicaMessageValidationError::{InvalidFormat, MissingField};

pub mod json_validation {
    use crate::messages::AlicaMessageValidationError::{InvalidFormat, MissingField};
    use crate::messages::{AlicaMessageValidationResult, CapnZeroIdValidator, AlicaMessageJsonValidator};

    pub fn validate_string_field(container: &json::object::Object, field: &str) -> AlicaMessageValidationResult {
        let value = container.get(field).ok_or_else(|| MissingField(field.to_string()))?;
        value.as_str().ok_or_else(|| InvalidFormat(format!("{} is no string", field)))?;
        Ok(())
    }

    pub fn validate_integer_field(container: &json::object::Object, field: &str) -> AlicaMessageValidationResult {
        let value = container.get(field).ok_or_else(|| MissingField(field.to_string()))?;
        value.as_i64().ok_or_else(|| InvalidFormat(format!("{} is no integer", field)))?;
        Ok(())
    }

    pub fn validate_capnzero_id_field(container: &json::object::Object, field: &str) -> AlicaMessageValidationResult {
        match container.get(field) {
            Some(id) => CapnZeroIdValidator::new().parse_alica_message(id.dump().as_bytes()),
            None => Err(MissingField(field.to_string()))
        }
    }

    pub fn validate_capnzero_id_list_field(container: &json::object::Object, field: &str) -> AlicaMessageValidationResult {
        let validator = CapnZeroIdValidator::new();
        match container.get(field) {
            Some(field_json) => match field_json {
                json::JsonValue::Array(id_array_json) => {
                    id_array_json.iter()
                        .map(|id| validator.parse_alica_message(id.dump().as_bytes()))
                        .collect()
                }
                _ => Err(InvalidFormat(format!("{} is no array", field)))
            },
            None => Err(MissingField(field.to_string()))
        }
    }
}

pub mod json_helper {
    use crate::messages::AlicaMessageValidationError::{self, InvalidFormat};

    pub fn parse_object(data: &[u8]) -> Result<json::object::Object, AlicaMessageValidationError> {
        let raw_message = String::from_utf8(data.to_vec())
            .map_err(|_| InvalidFormat("Message is no UTF-8 string".to_string()))?;

        let root_value = json::parse(&raw_message)
            .map_err(|_| InvalidFormat("Message is no JSON structure".to_string()))?;

        match root_value {
            json::JsonValue::Object(root_object) => Ok(root_object),
            _ => Err(InvalidFormat("Root of message is no object".to_string()))
        }
    }
}

pub enum AlicaMessageValidationError {
    InvalidFormat(String),
    MissingField(String)
}

impl Into<String> for AlicaMessageValidationError {
    fn into(self) -> String {
        match self {
            InvalidFormat(message) => message,
            MissingField(field) => format!("Required field missing: {}", field)
        }
    }
}

pub type AlicaMessageValidationResult = Result<(), AlicaMessageValidationError>;

#[mockall::automock]
pub trait AlicaMessageJsonValidator {
    fn parse_alica_message(&self, message: &[u8]) -> AlicaMessageValidationResult;
}

pub struct CapnZeroIdValidator {}

impl CapnZeroIdValidator {
    pub fn new() -> Self {
        CapnZeroIdValidator {}
    }
}

impl AlicaMessageJsonValidator for CapnZeroIdValidator {
    fn parse_alica_message(&self, message: &[u8]) -> AlicaMessageValidationResult {
        let capnzero_id_root = json_helper::parse_object(message)?;

        json_validation::validate_integer_field(&capnzero_id_root, "type")?;
        json_validation::validate_string_field(&capnzero_id_root, "value")?;

        Ok(())
    }
}

pub struct AlicaEngineInfoValidator {}

impl AlicaEngineInfoValidator {
    pub fn new() -> Self {
        AlicaEngineInfoValidator {}
    }
}

impl AlicaMessageJsonValidator for AlicaEngineInfoValidator {
    fn parse_alica_message(&self, message: &[u8]) -> AlicaMessageValidationResult {
        let engine_info_root = json_helper::parse_object(message)?;

        json_validation::validate_capnzero_id_field(&engine_info_root, "senderId")?;
        json_validation::validate_string_field(&engine_info_root, "masterPlan")?;
        json_validation::validate_string_field(&engine_info_root, "currentPlan")?;
        json_validation::validate_string_field(&engine_info_root, "currentState")?;
        json_validation::validate_string_field(&engine_info_root, "currentRole")?;
        json_validation::validate_string_field(&engine_info_root, "currentTask")?;
        json_validation::validate_capnzero_id_list_field(&engine_info_root, "agentIdsWithMe")?;

        Ok(())
    }
}

mod test {
    mod capnzero_id {
        use json;
        use crate::messages::{AlicaMessageJsonValidator, CapnZeroIdValidator};

        #[test]
        fn it_considers_a_complete_capnzero_id_valid() {
            let capnzero_id = json::object!{
                type: 0,
                value: "id"
            };
            let capnzero_id_json = json::stringify(capnzero_id);

            let validation_result = CapnZeroIdValidator::new().parse_alica_message(capnzero_id_json.as_bytes());

            assert!(validation_result.is_ok())
        }

        #[test]
        fn it_considers_a_non_utf8_message_invalid() {
            let message = vec![0x0];

            let validation_result = CapnZeroIdValidator::new().parse_alica_message(&message);

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_a_non_json_message_invalid() {
            let message = "";

            let validation_result = CapnZeroIdValidator::new().parse_alica_message(message.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_id_without_a_type_invalid() {
            let capnzero_id = json::object!{};
            let capnzero_id_json = json::stringify(capnzero_id);

            let validation_result = CapnZeroIdValidator::new().parse_alica_message(capnzero_id_json.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_id_without_a_value_invalid() {
            let capnzero_id = json::object!{
                type: 0
            };
            let capnzero_id_json = json::stringify(capnzero_id);

            let validation_result = CapnZeroIdValidator::new().parse_alica_message(capnzero_id_json.as_bytes());

            assert!(validation_result.is_err())
        }
    }

    mod alica_engine_info {
        use json;
        use crate::messages::{AlicaEngineInfoValidator, AlicaMessageJsonValidator};

        #[test]
        fn it_considers_a_complete_alica_engine_info_valid() {
            let engine_info = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                },
                masterPlan: "master plan",
                currentPlan: "current plan",
                currentState: "current state",
                currentRole: "current role",
                currentTask: "current task",
                agentIdsWithMe: [
                    {
                        type: 1,
                        value: "other agent"
                    },
                    {
                        type: 1,
                        value: "other other agent"
                    },
                ]
            };
            let engine_info_json = json::stringify(engine_info);

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(engine_info_json.as_bytes());

            assert!(validation_result.is_ok())
        }

        #[test]
        fn it_considers_a_non_utf8_message_invalid() {
            let message = vec![0x0];

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(&message);

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_a_non_json_message_invalid() {
            let message = "";

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(message.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_alica_engine_info_with_missing_sender_id_invalid() {
            let engine_info = json::object!{};
            let engine_info_json = json::stringify(engine_info);

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(engine_info_json.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_alica_engine_info_with_missing_master_plan_invalid() {
            let engine_info = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                }
            };
            let engine_info_json = json::stringify(engine_info);

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(engine_info_json.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_alica_engine_info_with_missing_current_plan_invalid() {
            let engine_info = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                },
                masterPlan: "master plan"
            };
            let engine_info_json = json::stringify(engine_info);

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(engine_info_json.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_alica_engine_info_with_missing_current_state_invalid() {
            let engine_info = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                },
                masterPlan: "master plan",
                currentPlan: "current plan"
            };
            let engine_info_json = json::stringify(engine_info);

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(engine_info_json.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_alica_engine_info_with_missing_current_role_invalid() {
            let engine_info = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                },
                masterPlan: "master plan",
                currentPlan: "current plan",
                currentState: "current state"
            };
            let engine_info_json = json::stringify(engine_info);

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(engine_info_json.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_alica_engine_info_with_missing_current_task_invalid() {
            let engine_info = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                },
                masterPlan: "master plan",
                currentPlan: "current plan",
                currentState: "current state",
                currentRole: "current role"
            };
            let engine_info_json = json::stringify(engine_info);

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(engine_info_json.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_alica_engine_info_with_missing_agent_ids_with_me_invalid() {
            let engine_info = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                },
                masterPlan: "master plan",
                currentPlan: "current plan",
                currentState: "current state",
                currentRole: "current role",
                currentTask: "current task"
            };
            let engine_info_json = json::stringify(engine_info);

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(engine_info_json.as_bytes());

            assert!(validation_result.is_err())
        }
    }

    mod allocation_authority_info {}
}