use super::{error_tui::ErrorTUI, timestamp::Timestamp, account::get_address};

use cargosos_bitcoin::{
    block_structure::{transaction::Transaction, outpoint::Outpoint, utxo_set::UTXOSet, transaction_output::TransactionOutput}, 
    logs::logger_sender::LoggerSender,
    wallet_structure::{wallet::Wallet, account::Account}, serialization::serializable_internal_order::SerializableInternalOrder,
};

use std::{
    io::stdin,
    sync::MutexGuard,
    collections::HashMap,
};

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

fn get_amount(logger: LoggerSender) -> Result<i64, ErrorTUI> {
    Ok(100_000)
}

pub fn create_transaction<'t>(
    utxo_set: &MutexGuard<'t, UTXOSet>,
    account: &Account,
    logger: LoggerSender,
) -> Result<Transaction, ErrorTUI> {    
    let address = get_address(logger.clone())?;
    let amount = get_amount(logger.clone())?;
    let fee = amount / 10;

    let available_outputs = utxo_set.get_utxo_list_with_outpoints(Some(&address));

    match account.create_transaction_with_available_outputs(
        address, 
        amount, 
        fee, 
        available_outputs
    ) {
        Ok(transaction) => {
            let mut stream: Vec<u8> = Vec::new();
            let _ = transaction.io_serialize(&mut stream);
            println!("Transaction created successfully: {:02X?}", stream);
            Ok(transaction)
        },
        Err(error) => {
            let _ = logger.log_wallet(format!(
                "Error creating transaction, with error: {:?}",
                error
            ));
            Err(ErrorTUI::TransactionCreationFail)
        }
    }
}
