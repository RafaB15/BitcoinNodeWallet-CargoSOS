use super::{error_tui::ErrorTUI, timestamp::Timestamp};

use cargosos_bitcoin::{
    block_structure::transaction::Transaction, logs::logger_sender::LoggerSender,
};

use std::io::stdin;

fn print_timestamp() {
    let options: &[Timestamp] = &[
        Timestamp::Day,
        Timestamp::Week,
        Timestamp::Month,
        Timestamp::Year,
    ];

    for option in options {
        let option_id: char = (*option).into();
        println!("{option} [{option_id}]");
    }
}

pub fn select_option(logger: LoggerSender) -> Result<Timestamp, ErrorTUI> {
    println!("Select an option:");
    print_timestamp();

    let mut option: String = String::new();
    if stdin().read_line(&mut option).is_err() {
        return Err(ErrorTUI::TerminalReadFail);
    }

    loop {
        let _: Timestamp = match Timestamp::try_from(option.trim()) {
            Ok(result) => {
                let _ = logger.log_wallet(format!("Valid option entered"));
                return Ok(result);
            }
            Err(error) => {
                let _ =
                    logger.log_wallet(format!("Invalid option entered, with error: {:?}", error));

                option.clear();
                println!("Error, please enter a valid option:");
                print_timestamp();
                if stdin().read_line(&mut option).is_err() {
                    return Err(ErrorTUI::TerminalReadFail);
                }
                continue;
            }
        };
    }
}

pub fn create_transaction() -> Transaction {
    todo!()
}
