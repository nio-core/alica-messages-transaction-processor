use ::alica_messages_tp::handler::AlicaMessageTransactionHandler;

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

    let validator_url = match args.value_of("connect") {
        Some(url) => url,
        None => panic!("Missing validator address!"),
    };

    let alica_message_transaction_handler = AlicaMessageTransactionHandler::new();
    let mut processor = sawtooth_sdk::processor::TransactionProcessor::new(validator_url);
    processor.add_handler(&alica_message_transaction_handler);
    processor.start();
}
