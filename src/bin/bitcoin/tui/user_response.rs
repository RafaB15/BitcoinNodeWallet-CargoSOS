use super::{backend, menu, menu_option::MenuOption, timestamp::Timestamp};

use crate::{
    ui::{
        error_ui::ErrorUI,
    }, 
    process::{
        reference::{get_reference, MutArc},
        transaction,
    },
};


use cargosos_bitcoin::{
    block_structure::{
        block_chain::BlockChain, 
        utxo_set::UTXOSet,
    },
    logs::logger_sender::LoggerSender,
    node_structure::{
        broadcasting::Broadcasting,  
    },
    wallet_structure::wallet::Wallet, notifications::notifier::Notifier,
};

use std::{
    net::TcpStream,
    io::{stdin, Read, Write},
};

/// It will responde to the user input
///
/// ### Error
///  * `ErrorUI::TerminalReadFail`: It will appear when the terminal read fails
///  * `ErrorUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
///  * `ErrorUI::ErrorFromPeer`: It will appear when a conextion with a peer fails
pub fn user_input<N : Notifier>(
    broadcasting: &mut Broadcasting<TcpStream>,
    wallet: MutArc<Wallet>,
    utxo_set: MutArc<UTXOSet>,
    block_chain: MutArc<BlockChain>,
    notifier: N,
    logger: LoggerSender,
) -> Result<(), ErrorUI> {
    loop {
        let wallet_reference = get_reference(&wallet)?;
        let utxo_set_reference = get_reference(&utxo_set)?;

        match menu::select_option(logger.clone())? {
            MenuOption::CreateAccount => backend::create_account(&wallet_reference, notifier, logger.clone())?,
            MenuOption::ChangeAccount => backend::change_account(&wallet_reference, notifier, logger.clone())?,
            MenuOption::RemoveAccount => backend::remove_account(&wallet_reference, logger.clone())?,
            MenuOption::SendTransaction => {
                backend::sending_transaction(broadcasting, &wallet_reference, &utxo_set_reference, notifier, logger.clone())?
            }
            MenuOption::ShowAccounts => {
                backend::show_accounts(&wallet_reference, logger.clone());
            }
            MenuOption::ShowBalance => showing_balance(&wallet, &utxo_set, logger.clone())?,
            MenuOption::LastTransactions => latest_transactions(&block_chain, logger.clone())?,
            MenuOption::Exit => break,
        }
    }

    Ok(())
}


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

/// Show the balance of the selected account in the wallet
///
/// ### Error
///  * `ErrorUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
fn showing_balance(
    wallet: &MutArc<Wallet>,
    utxo_set: &MutArc<UTXOSet>,
    logger: LoggerSender,
) -> Result<(), ErrorUI> {
    let wallet = get_reference(wallet)?;

    for account in wallet.get_accounts() {
        let utxo_set = get_reference(utxo_set)?;
        let balance = utxo_set.get_balance_in_satoshis(&account.address);
        let pending = utxo_set.get_pending_in_satoshis(&account.address);
        let total = balance + pending;

        println!(
            "Account: {} has balance of \n\tAvalable: {balance}\n\tPending: {pending}\n\n\tTotal: {total}",
            account.account_name
        );

        let _ = logger.log_wallet(format!("Account: {} has balance of, avalable: {balance}, pending: {pending} and a total of: {total}", account.account_name));
    }

    Ok(())
}

/// Show the lastest transactions given by the timestamp selected by the user
///
/// ### Error
///  * `ErrorUI::TerminalReadFail`: It will appear when the terminal read fails
///  * `ErrorUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
fn latest_transactions(
    block_chain: &MutArc<BlockChain>,
    logger: LoggerSender,
) -> Result<(), ErrorUI> {
    let selected_timestamp = transaction::select_option(logger.clone())?;
    let timestamp = selected_timestamp.get_timestamps_from_now();

    let _ = logger.log_transaction(format!(
        "Selected timestamp: {selected_timestamp}, and it's corresponding timestamp: {timestamp}"
    ));

    let block_chain = get_reference(block_chain)?;
    let blocks = block_chain.get_blocks_after_timestamp(timestamp as u32);

    for block in blocks {
        for transaction in block.transactions {
            println!("{transaction}");
        }
    }

    Ok(())
}
