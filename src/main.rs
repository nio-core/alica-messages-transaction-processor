use alica_messages_tp::handler::AlicaMessageTransactionHandler;
use sawtooth_sdk::processor::TransactionProcessor;
use sawtooth_alica_payload::messages::json::{AlicaEngineInfoValidator,
                                                                 AllocationAuthorityInfoValidator,
                                                                 PlanTreeInfoValidator, SolverResultValidator,
                                                                 RoleSwitchValidator, SyncReadyValidator,
                                                                 SyncTalkValidator};
use sawtooth_alica_payload::{payloads, TransactionFamily};

fn main() {
    let args = clap::App::new("alica-messages-tp")
        .version("0.1.0")
        .about("Transaction Processor for ALICA task allocation messages")
        .author("Sven Starcke")
        .arg(
            clap::Arg::with_name("connect")
                .short("C")
                .long("connect")
                .takes_value(true)
                .help("Address of the validator to connect to"),
        )
        .get_matches();

    let validator_url = args.value_of("connect").expect("Missing validator address!");

    let transaction_family = TransactionFamily::new("alica_messages", &vec!["0.1.0".to_string()]);
    let payload_format = Box::from(payloads::pipe_separated::Format::default());
    let mut transaction_handler = AlicaMessageTransactionHandler::new(transaction_family, payload_format);
    transaction_handler
        .with_validator_for("ALICA_ENGINE_INFO", Box::from(AlicaEngineInfoValidator::new()))
        .with_validator_for("ALLOCATION_AUTHORITY_INFO", Box::from(AllocationAuthorityInfoValidator::new()))
        .with_validator_for("PLAN_TREE_INFO", Box::from(PlanTreeInfoValidator::new()))
        .with_validator_for("SOLVER_RESULT", Box::from(SolverResultValidator::new()))
        .with_validator_for("ROLE_SWITCH", Box::from(RoleSwitchValidator::new()))
        .with_validator_for("SYNC_READY", Box::from(SyncReadyValidator::new()))
        .with_validator_for("SYNC_TALK", Box::from(SyncTalkValidator::new()));

    let mut processor = TransactionProcessor::new(validator_url);
    processor.add_handler(&transaction_handler);
    processor.start();
}
