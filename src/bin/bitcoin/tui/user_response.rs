use super::{account, error_tui::ErrorTUI, menu, menu_option::MenuOption};

use crate::process::message_response::MessageResponse;

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, utxo_set::UTXOSet},
    logs::logger_sender::LoggerSender,
    wallet_structure::wallet::Wallet,
};

use std::{
    sync::mpsc::{Receiver, Sender},
    sync::{Arc, Mutex, MutexGuard},
    thread::{self, JoinHandle},
    time::Duration,
};

type MutArc<T> = Arc<Mutex<T>>;

fn get_reference<'t, T>(reference: &'t MutArc<T>) -> Result<MutexGuard<'t, T>, ErrorTUI> {
    match reference.lock() {
        Ok(reference) => Ok(reference),
        Err(_) => todo!(),
    }
}

pub fn user_input(
    sender: Sender<MessageResponse>,
    wallet_ref: MutArc<Wallet>,
    utxo_set_ref: MutArc<UTXOSet>,
    block_chain_ref: MutArc<BlockChain>,
    logger: LoggerSender,
) -> Result<(), ErrorTUI> {
    loop {
        let mut wallet = get_reference(&wallet_ref)?;
        let utxo_set = get_reference(&utxo_set_ref)?;
        let block_chain = get_reference(&block_chain_ref)?;

        match menu::select_option(logger.clone())? {
            MenuOption::CreateAccount => {
                let account = account::create_account(logger.clone())?;
                wallet.add_account(account);
            }
            MenuOption::ChangeAccount => {
                let account = account::select_account(&wallet, logger.clone())?;
                wallet.change_account(account);
            }
            MenuOption::RemoveAccount => {
                let account = account::select_account(&wallet, logger.clone())?;
                wallet.remove_account(account);
            }
            MenuOption::SendTransaction => todo!(),
            MenuOption::ShowAccounts => account::show_accounts(&wallet, logger.clone()),
            MenuOption::ShowBalance => {
                let account = wallet.get_selected_account();
                match account {
                    Some(account) => {
                        let balance = utxo_set.get_balance_in_satoshis(&account.address);
                        let message_output =
                            format!("Account: {:?} has balance of {balance}", account);

                        println!("{message_output}");
                        let _ = logger.log_wallet(message_output);
                    }
                    None => {
                        let _ = logger.log_wallet("No account selected".to_string());
                    }
                }
            }
            MenuOption::LastTransactions => {}
            MenuOption::Exit => break,
        }

        thread::sleep(Duration::from_millis(200));
    }

    Ok(())
}

fn send_menu_option(
    sender: Sender<MessageResponse>,
    message: MessageResponse,
    logger: LoggerSender,
) -> Result<(), ErrorTUI> {
    match sender.send(message.clone()) {
        Ok(_) => {
            let _ = logger.log_interface(format!("sending message: {:?}", message));
            Ok(())
        }
        Err(_) => todo!(),
    }
}

pub fn handle_response(
    receiver_broadcasting: Receiver<MessageResponse>,
    wallet: MutArc<Wallet>,
    utxo_set: MutArc<UTXOSet>,
    block_chain: MutArc<BlockChain>,
    logger: LoggerSender,
) -> JoinHandle<()> {
    thread::spawn(move || {
        for message in receiver_broadcasting {
            if message == MessageResponse::Exit {
                break;
            }

            response(message, &wallet, &utxo_set, &block_chain, logger.clone());
        }
    })
}

fn response(
    message: MessageResponse,
    wallet: &MutArc<Wallet>,
    utxo_set: &MutArc<UTXOSet>,
    block_chain: &MutArc<BlockChain>,
    logger: LoggerSender,
) -> Result<(), ErrorTUI> {
    match message {
        MessageResponse::Block(block) => todo!(),
        MessageResponse::Transaction(transaction) => {
            let _ = logger.log_wallet(format!("Receive transaction: {:?}", transaction));

            if let Some(account) = get_reference(&wallet)?.get_selected_account() {
                if account.verify_transaction_ownership(&transaction) {
                    let message_output = format!(
                        "Transaction: {:?} is valid and has not been added to the blockchain yet",
                        transaction
                    );

                    println!("{message_output}");
                    let _ = logger.log_wallet(message_output);
                }
            }
        }
        MessageResponse::Exit => {}
    }

    Ok(())
}
