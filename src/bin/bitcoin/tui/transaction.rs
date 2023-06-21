use super::{error_tui::ErrorTUI, timestamp::Timestamp, account::get_address};

use cargosos_bitcoin::{
    block_structure::{transaction::Transaction, utxo_set::UTXOSet}, 
    logs::logger_sender::LoggerSender,
    wallet_structure::{account::Account, error_wallet::ErrorWallet}, 
};

use std::{
    io::stdin,
    sync::MutexGuard,
};

pub fn select_option(logger: LoggerSender) -> Result<Timestamp, ErrorTUI> {
    println!("Select an option:");
    Timestamp::print_all();

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
                Timestamp::print_all();
                if stdin().read_line(&mut option).is_err() {
                    return Err(ErrorTUI::TerminalReadFail);
                }
                continue;
            }
        };
    }
}

/// Get the amount for the transaction from the terminal
///
/// ### Error
///  * `ErrorTUI::TerminalReadFail`: It will appear when the terminal read fails
fn get_amount(logger: LoggerSender) -> Result<i64, ErrorTUI> {
    let mut amount: String = String::new();

    println!("Enter an amount: ");
    if stdin().read_line(&mut amount).is_err() {
        return Err(ErrorTUI::TerminalReadFail);
    }

    loop {
        match amount.trim().parse::<u32>() {
            Ok(result) => {
                let _ = logger.log_wallet(format!("Valid amount entered"));
                return Ok(result as i64);
            },
            Err(error) => {
                let _ = logger.log_wallet(format!(
                    "Invalid amount entered, with error: {:?}",
                    error
                ));

                amount.clear();
                println!("Error, please enter a valid amount:");
                if stdin().read_line(&mut amount).is_err() {
                    return Err(ErrorTUI::TerminalReadFail);
                }

                continue;
            }
        };
    }
}

/// Get the fee for the transaction from the terminal
///
/// ### Error
///  * `ErrorTUI::TerminalReadFail`: It will appear when the terminal read fails
fn get_fee(logger: LoggerSender) -> Result<i64, ErrorTUI> {
    let mut fee: String = String::new();

    println!("Enter a fee: ");
    if stdin().read_line(&mut fee).is_err() {
        return Err(ErrorTUI::TerminalReadFail);
    }

    loop {
        match fee.trim().parse::<u32>() {
            Ok(result) => {
                let _ = logger.log_wallet(format!("Valid fee entered"));
                return Ok(result as i64);
            },
            Err(error) => {
                let _ = logger.log_wallet(format!(
                    "Invalid fee entered, with error: {:?}",
                    error
                ));

                fee.clear();
                println!("Error, please enter a valid fee:");
                if stdin().read_line(&mut fee).is_err() {
                    return Err(ErrorTUI::TerminalReadFail);
                }

                continue;
            }
        };
    }
}


pub fn create_transaction<'t>(
    utxo_set: &MutexGuard<'t, UTXOSet>,
    account: &Account,
    logger: LoggerSender,
) -> Result<Transaction, ErrorTUI> {    
    let address = get_address(logger.clone())?;
    let amount = get_amount(logger.clone())?;
    let fee = get_fee(logger.clone())?;

    let available_outputs = utxo_set.get_utxo_list_with_outpoints(Some(&account.address));

    match account.create_transaction_with_available_outputs(
        address, 
        amount, 
        fee, 
        available_outputs
    ) {
        Ok(transaction) => Ok(transaction),
        Err(ErrorWallet::NotEnoughFunds(_)) => Err(ErrorTUI::TransactionWithoutSufficientFunds),
        Err(error) => {
            let _ = logger.log_wallet(format!(
                "Error creating transaction, with error: {:?}",
                error
            ));
            Err(ErrorTUI::TransactionCreationFail)
        }
    }
}
