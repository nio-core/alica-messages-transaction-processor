use mockall;

use crate::messages::AlicaMessageValidationError::{InvalidFormat, MissingField};

pub mod json_validation {
    use crate::messages::{AlicaMessageJsonValidator, AlicaMessageValidationResult, CapnZeroIdValidator};
    use crate::messages::AlicaMessageValidationError::{InvalidFormat, MissingField};

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

    pub fn validate_integer_list_field(container: &json::object::Object, field: &str) -> AlicaMessageValidationResult {
        match container.get(field) {
            Some(field_json) => match field_json {
                json::JsonValue::Array(array_json) => {
                    array_json.iter()
                        .map(|array_entry| match array_entry.as_i64() {
                            Some(_) => Ok(()),
                            None => Err(InvalidFormat(format!("{} contains a non integer entry", field)))
                        })
                        .collect()
                },
                _ => Err(InvalidFormat(format!("{} is no array", field)))
            },
            None => Err(MissingField(field.to_string()))
        }
    }

    pub fn validate_list_field_with_complex_components(container: &json::object::Object, field: &str, validator: &dyn AlicaMessageJsonValidator)
                                                       -> AlicaMessageValidationResult {
        match container.get(field) {
            Some(field_json) => match field_json {
                json::JsonValue::Array(array_json) => {
                    array_json.iter()
                        .map(|array_entry| validator.parse_alica_message(array_entry.dump().as_bytes()))
                        .collect()
                },
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
        json_validation::validate_list_field_with_complex_components(&engine_info_root, "agentIdsWithMe", &CapnZeroIdValidator::new())?;

        Ok(())
    }
}

pub struct AllocationAuthorityInfoValidator {}

impl AllocationAuthorityInfoValidator {
    pub fn new() -> Self {
        AllocationAuthorityInfoValidator {}
    }
}

impl AlicaMessageJsonValidator for AllocationAuthorityInfoValidator {
    fn parse_alica_message(&self, message: &[u8]) -> AlicaMessageValidationResult {
        let allocation_authority_info_root = json_helper::parse_object(message)?;

        json_validation::validate_capnzero_id_field(&allocation_authority_info_root, "senderId")?;
        json_validation::validate_integer_field(&allocation_authority_info_root, "planId")?;
        json_validation::validate_integer_field(&allocation_authority_info_root, "parentState")?;
        json_validation::validate_integer_field(&allocation_authority_info_root, "planType")?;
        json_validation::validate_capnzero_id_field(&allocation_authority_info_root, "authority")?;
        json_validation::validate_list_field_with_complex_components(&allocation_authority_info_root, "entrypointRobots", &EntryPointRobotValidator::new())?;

        Ok(())
    }
}

pub struct EntryPointRobotValidator {}

impl EntryPointRobotValidator {
    pub fn new() -> Self {
        EntryPointRobotValidator {}
    }
}

impl AlicaMessageJsonValidator for EntryPointRobotValidator {
    fn parse_alica_message(&self, message: &[u8]) -> AlicaMessageValidationResult {
        let entry_point_robot = json_helper::parse_object(message)?;
        json_validation::validate_integer_field(&entry_point_robot, "entrypoint")?;
        json_validation::validate_list_field_with_complex_components(&entry_point_robot, "robots", &CapnZeroIdValidator::new())?;
        Ok(())
    }
}

pub struct PlanTreeInfoValidator {}

impl PlanTreeInfoValidator {
    pub fn new() -> Self {
        PlanTreeInfoValidator {}
    }
}

impl AlicaMessageJsonValidator for PlanTreeInfoValidator {
    fn parse_alica_message(&self, message: &[u8]) -> AlicaMessageValidationResult {
        let plan_tree_info = json_helper::parse_object(message)?;
        json_validation::validate_capnzero_id_field(&plan_tree_info, "senderId")?;
        json_validation::validate_integer_list_field(&plan_tree_info, "stateIds")?;
        json_validation::validate_integer_list_field(&plan_tree_info, "succeededEps")?;
        Ok(())
    }
}

pub struct RoleSwitchValidator {}

impl RoleSwitchValidator {
    pub fn new() -> Self {
        RoleSwitchValidator {}
    }
}

impl AlicaMessageJsonValidator for RoleSwitchValidator {
    fn parse_alica_message(&self, message: &[u8]) -> AlicaMessageValidationResult {
        let role_switch = json_helper::parse_object(message)?;
        json_validation::validate_capnzero_id_field(&role_switch, "senderId")?;
        json_validation::validate_integer_field(&role_switch, "roleId")?;
        Ok(())
    }
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

mod test {
    mod alica_engine_info {
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
            }.dump();

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(engine_info.as_bytes());

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
            let engine_info = json::object!{}.dump();

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(engine_info.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_alica_engine_info_with_missing_master_plan_invalid() {
            let engine_info = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                }
            }.dump();

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(engine_info.as_bytes());

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
            }.dump();

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(engine_info.as_bytes());

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
            }.dump();

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(engine_info.as_bytes());

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
            }.dump();

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(engine_info.as_bytes());

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
            }.dump();

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(engine_info.as_bytes());

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
            }.dump();

            let validation_result = AlicaEngineInfoValidator::new().parse_alica_message(engine_info.as_bytes());

            assert!(validation_result.is_err())
        }
    }

    mod allocation_authority_info {
        use crate::messages::{AlicaMessageJsonValidator, AllocationAuthorityInfoValidator};

        #[test]
        fn it_considers_a_complete_allocation_authority_info_valid() {
            let allocation_authority_info = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                },
                planId: 1,
                parentState: 2,
                planType: 3,
                authority: {
                    type: 1,
                    value: "authority id"
                },
                entrypointRobots: [

                ]
            }.dump();

            let validation_result = AllocationAuthorityInfoValidator::new().parse_alica_message(allocation_authority_info.as_bytes());

            assert!(validation_result.is_ok())
        }

        #[test]
        fn it_considers_a_non_utf8_message_invalid() {
            let message = vec![0x0];

            let validation_result = AllocationAuthorityInfoValidator::new().parse_alica_message(&message);

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_a_non_json_message_invalid() {
            let message = "";

            let validation_result = AllocationAuthorityInfoValidator::new().parse_alica_message(message.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_allocation_authority_info_without_a_sender_id_invalid() {
            let allocation_authority_info = json::object!{}.dump();

            let validation_result = AllocationAuthorityInfoValidator::new().parse_alica_message(allocation_authority_info.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_allocation_authority_without_a_plan_id_invalid() {
            let allocation_authority_info = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                }
            }.dump();

            let validation_result = AllocationAuthorityInfoValidator::new().parse_alica_message(allocation_authority_info.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_allocation_authority_info_without_a_parent_state_invalid() {
            let allocation_authority_info = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                },
                planId: 1
            }.dump();

            let validation_result = AllocationAuthorityInfoValidator::new().parse_alica_message(allocation_authority_info.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_allocation_authority_info_without_a_plan_type_invalid() {
            let allocation_authority_info = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                },
                planId: 1,
                parentState: 2
            }.dump();

            let validation_result = AllocationAuthorityInfoValidator::new().parse_alica_message(allocation_authority_info.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_allocation_authority_info_without_an_authority_invalid() {
            let allocation_authority_info = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                },
                planId: 1,
                parentState: 2,
                planType: 3
            }.dump();

            let validation_result = AllocationAuthorityInfoValidator::new().parse_alica_message(allocation_authority_info.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_allocation_authority_info_without_a_list_of_entrypoint_robots_invalid() {
            let allocation_authority_info = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                },
                planId: 1,
                parentState: 2,
                planType: 3,
                authority: {
                    type: 1,
                    value: "authority id"
                }
            }.dump();

            let validation_result = AllocationAuthorityInfoValidator::new().parse_alica_message(allocation_authority_info.as_bytes());

            assert!(validation_result.is_err())
        }
    }

    mod entry_point_robot {
        use crate::messages::{AlicaMessageJsonValidator, EntryPointRobotValidator};

        #[test]
        fn it_considers_a_complete_entry_point_robot_valid() {
            let entry_point_robot = json::object!{
                entrypoint: 0,
                robots: [
                    {
                        type: 1,
                        value: "id1"
                    },
                    {
                        type: 1,
                        value: "id2"
                    }
                ]
            }.dump();

            let validation_result = EntryPointRobotValidator::new().parse_alica_message(entry_point_robot.as_bytes());

            assert!(validation_result.is_ok())
        }

        #[test]
        fn it_considers_a_non_utf8_message_invalid() {
            let message = vec![0x0];

            let validation_result = EntryPointRobotValidator::new().parse_alica_message(&message);

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_a_non_json_message_invalid() {
            let message = "";

            let validation_result = EntryPointRobotValidator::new().parse_alica_message(message.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_entry_point_robot_without_entrypoint_invalid() {
            let entry_point_robot = json::object!{}.dump();

            let validation_result = EntryPointRobotValidator::new().parse_alica_message(entry_point_robot.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_entry_point_robot_without_robots_invalid() {
            let entry_point_robot = json::object!{
                entrypoint: 0
            }.dump();

            let validation_result = EntryPointRobotValidator::new().parse_alica_message(entry_point_robot.as_bytes());

            assert!(validation_result.is_err())
        }
    }

    mod plan_tree_info {
        use crate::messages::{PlanTreeInfoValidator, AlicaMessageJsonValidator};

        #[test]
        fn it_considers_a_complete_plan_tree_info_valid() {
            let plan_tree_info = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                },
                stateIds: [1, 2, 3],
                succeededEps: [4, 5, 6]
            }.dump();

            let validation_result = PlanTreeInfoValidator::new().parse_alica_message(plan_tree_info.as_bytes());

            assert!(validation_result.is_ok())
        }

        #[test]
        fn it_considers_a_non_utf8_message_invalid() {
            let message = vec![0x0];

            let validation_result = PlanTreeInfoValidator::new().parse_alica_message(&message);

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_a_non_json_message_invalid() {
            let message = "";

            let validation_result = PlanTreeInfoValidator::new().parse_alica_message(message.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_a_plan_tree_info_without_a_sender_id_invalid() {
            let plan_tree_info = json::object!{}.dump();

            let validation_result = PlanTreeInfoValidator::new().parse_alica_message(plan_tree_info.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_a_plan_tree_info_without_state_ids_invalid() {
            let plan_tree_info = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                }
            }.dump();

            let validation_result = PlanTreeInfoValidator::new().parse_alica_message(plan_tree_info.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_a_plan_tree_info_without_succeeded_eps_invalid() {
            let plan_tree_info = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                },
                stateIds: [1, 2, 3]
            }.dump();

            let validation_result = PlanTreeInfoValidator::new().parse_alica_message(plan_tree_info.as_bytes());

            assert!(validation_result.is_err())
        }
    }

    mod role_switch {
        use crate::messages::{RoleSwitchValidator, AlicaMessageJsonValidator};

        #[test]
        fn it_considers_a_complete_role_switch_valid() {
            let role_switch = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                },
                roleId: 1
            }.dump();

            let validation_result = RoleSwitchValidator::new().parse_alica_message(role_switch.as_bytes());

            assert!(validation_result.is_ok())
        }

        #[test]
        fn it_considers_a_non_utf8_message_invalid() {
            let message = vec![0x0];

            let validation_result = RoleSwitchValidator::new().parse_alica_message(&message);

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_a_non_json_message_invalid() {
            let message = "";

            let validation_result = RoleSwitchValidator::new().parse_alica_message(message.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_a_role_switch_without_sender_id_invalid() {
            let role_switch = json::object!{}.dump();

            let validation_result = RoleSwitchValidator::new().parse_alica_message(role_switch.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_a_role_switch_wihtout_a_role_id_invalid() {
            let role_switch = json::object!{
                senderId: {
                    type: 0,
                    value: "id"
                }
            }.dump();

            let validation_result = RoleSwitchValidator::new().parse_alica_message(role_switch.as_bytes());

            assert!(validation_result.is_err())
        }
    }

    mod capnzero_id {
        use crate::messages::{AlicaMessageJsonValidator, CapnZeroIdValidator};

        #[test]
        fn it_considers_a_complete_capnzero_id_valid() {
            let capnzero_id = json::object!{
                type: 0,
                value: "id"
            }.dump();

            let validation_result = CapnZeroIdValidator::new().parse_alica_message(capnzero_id.as_bytes());

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
            let capnzero_id = json::object!{}.dump();

            let validation_result = CapnZeroIdValidator::new().parse_alica_message(capnzero_id.as_bytes());

            assert!(validation_result.is_err())
        }

        #[test]
        fn it_considers_an_id_without_a_value_invalid() {
            let capnzero_id = json::object!{
                type: 0
            }.dump();

            let validation_result = CapnZeroIdValidator::new().parse_alica_message(capnzero_id.as_bytes());

            assert!(validation_result.is_err())
        }
    }
}