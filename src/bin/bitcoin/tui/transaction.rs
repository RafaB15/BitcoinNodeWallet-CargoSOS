use super::{account::get_address, timestamp::Timestamp};

use crate::ui::error_ui::ErrorUI;

use cargosos_bitcoin::{
    block_structure::{transaction::Transaction, utxo_set::UTXOSet},
    logs::logger_sender::LoggerSender,
    wallet_structure::{account::Account, error_wallet::ErrorWallet},
};

use std::{io::stdin, sync::MutexGuard};

/// Get the timestamp from the user via terminal
///
/// ### Error
///  * `ErrorUI::TerminalReadFail`: It will appear when the terminal read fails
pub fn select_option(logger: LoggerSender) -> Result<Timestamp, ErrorUI> {
    println!("Select an option:");
    Timestamp::print_all();

    let mut option: String = String::new();
    if stdin().read_line(&mut option).is_err() {
        return Err(ErrorUI::TerminalReadFail);
    }

    loop {
        let _: Timestamp = match Timestamp::try_from(option.trim()) {
            Ok(result) => {
                let _ = logger.log_wallet("Valid option entered".to_string());
                return Ok(result);
            }
            Err(error) => {
                let _ =
                    logger.log_wallet(format!("Invalid option entered, with error: {:?}", error));

                option.clear();
                println!("Error, please enter a valid option:");
                Timestamp::print_all();
                if stdin().read_line(&mut option).is_err() {
                    return Err(ErrorUI::TerminalReadFail);
                }
                continue;
            }
        };
    }
}

/// Get the amount for the transaction from the terminal
///
/// ### Error
///  * `ErrorUI::TerminalReadFail`: It will appear when the terminal read fails
fn get_amount(logger: LoggerSender) -> Result<i64, ErrorUI> {
    let mut amount: String = String::new();

    println!("Enter an amount: ");
    if stdin().read_line(&mut amount).is_err() {
        return Err(ErrorUI::TerminalReadFail);
    }

    loop {
        match amount.trim().parse::<u32>() {
            Ok(result) => {
                let _ = logger.log_wallet("Valid amount entered".to_string());
                return Ok(result as i64);
            }
            Err(error) => {
                let _ =
                    logger.log_wallet(format!("Invalid amount entered, with error: {:?}", error));

                amount.clear();
                println!("Error, please enter a valid amount:");
                if stdin().read_line(&mut amount).is_err() {
                    return Err(ErrorUI::TerminalReadFail);
                }

                continue;
            }
        };
    }
}

/// Get the fee for the transaction from the terminal
///
/// ### Error
///  * `ErrorUI::TerminalReadFail`: It will appear when the terminal read fails
fn get_fee(logger: LoggerSender) -> Result<i64, ErrorUI> {
    let mut fee: String = String::new();

    println!("Enter a fee: ");
    if stdin().read_line(&mut fee).is_err() {
        return Err(ErrorUI::TerminalReadFail);
    }

    loop {
        match fee.trim().parse::<u32>() {
            Ok(result) => {
                let _ = logger.log_wallet("Valid fee entered".to_string());
                return Ok(result as i64);
            }
            Err(error) => {
                let _ = logger.log_wallet(format!("Invalid fee entered, with error: {:?}", error));

                fee.clear();
                println!("Error, please enter a valid fee:");
                if stdin().read_line(&mut fee).is_err() {
                    return Err(ErrorUI::TerminalReadFail);
                }

                continue;
            }
        };
    }
}

/// Creates a transaction via terminal given the user user_input
///
/// ### Error
///  * `ErrorUI::TransactionWithoutSufficientFunds`: It will appear when the user does not have enough funds to make the transaction
///  * `ErrorUI::TransactionCreationFail`: It will appear when the transaction fail to create the signature script
pub fn create_transaction(
    utxo_set: &MutexGuard<'_, UTXOSet>,
    account: &Account,
    logger: LoggerSender,
) -> Result<Transaction, ErrorUI> {
    let address = get_address(logger.clone())?;
    let amount = get_amount(logger.clone())?;
    let fee = get_fee(logger.clone())?;

    let available_outputs = utxo_set.get_utxo_list_with_outpoints(Some(&account.address));

    match account.create_transaction_with_available_outputs(address, amount, fee, available_outputs)
    {
        Ok(transaction) => Ok(transaction),
        Err(ErrorWallet::NotEnoughFunds(_)) => Err(ErrorUI::TransactionWithoutSufficientFunds),
        Err(error) => {
            let _ = logger.log_wallet(format!(
                "Error creating transaction, with error: {:?}",
                error
            ));
            Err(ErrorUI::TransactionCreationFail)
        }
    }
}
