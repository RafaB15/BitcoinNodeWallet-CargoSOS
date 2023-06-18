use super::{account, error_tui::ErrorTUI, menu, menu_option::MenuOption};

use crate::process::message_response::MessageResponse;

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, utxo_set::UTXOSet},
    logs::logger_sender::LoggerSender,
    wallet_structure::wallet::Wallet,
};

use std::{
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

pub fn user_input(sender: Sender<MessageResponse>, logger: LoggerSender) -> Result<(), ErrorTUI> {
    loop {
        let sender_clone = sender.clone();
        let logger_clone = logger.clone();

        match menu::select_option(logger.clone())? {
            MenuOption::CreateAccount => {
                send_menu_option(sender_clone, MessageResponse::CreateAccount, logger_clone)?
            }
            MenuOption::ChangeAccount => {
                send_menu_option(sender_clone, MessageResponse::ChangeAccount, logger_clone)?
            }
            MenuOption::RemoveAccount => {
                send_menu_option(sender_clone, MessageResponse::RemoveAccount, logger_clone)?
            }
            MenuOption::SendTransaction => {
                send_menu_option(sender_clone, MessageResponse::SendTransaction, logger_clone)?
            }
            MenuOption::ShowAccounts => {
                send_menu_option(sender_clone, MessageResponse::ShowAccounts, logger_clone)?
            }
            MenuOption::ShowBalance => {
                send_menu_option(sender_clone, MessageResponse::ShowBalance, logger_clone)?
            }
            MenuOption::LastTransactions => send_menu_option(
                sender_clone,
                MessageResponse::LastTransactions,
                logger_clone,
            )?,
            MenuOption::Exit => break,
        }
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
    mut wallet: Wallet,
    utxo_set: UTXOSet,
    block_chain: BlockChain,
    logger: LoggerSender,
) -> JoinHandle<(BlockChain, Wallet)> {
    thread::spawn(move || {
        //let account = menu::

        for message in receiver_broadcasting {
            if message == MessageResponse::Exit {
                break;
            }

            response(
                message,
                &mut wallet,
                &utxo_set,
                &block_chain,
                logger.clone(),
            );
        }

        (block_chain, wallet)
    })
}

fn response(
    message: MessageResponse,
    wallet: &mut Wallet,
    utxo_set: &UTXOSet,
    block_chain: &BlockChain,
    logger: LoggerSender,
) -> Result<(), ErrorTUI> {
    match message {
        MessageResponse::Block(block) => todo!(),
        MessageResponse::Transaction(transaction) => {
            let _ = logger.log_wallet(format!("Receive transaction: {:?}", transaction));

            if let Some(account) = wallet.get_selected_account() {
                if account.verify_transaction_ownership(&transaction) {
                    let message_output = format!(
                        "Transaction: {:?} is valid and has not been added to the blockchain yet",
                        transaction
                    );

                    println!("{message_output}");
                    let _ = logger.log_wallet(message_output);
                }
            }
        },
        MessageResponse::CreateAccount => {
            let account = account::create_account(logger)?;
            wallet.add_account(account);
        }
        MessageResponse::ChangeAccount => {
            let account = account::select_account(wallet, logger)?;
            wallet.change_account(account);
        }
        MessageResponse::RemoveAccount => {
            let account = account::select_account(wallet, logger)?;
            wallet.remove_account(account);
        }
        MessageResponse::SendTransaction => todo!(),
        MessageResponse::ShowAccounts => account::show_accounts(wallet, logger),
        MessageResponse::ShowBalance => {
            let account = wallet.get_selected_account();
            match account {
                Some(account) => {
                    let balance = utxo_set.get_balance_in_satoshis(&account.address);
                    let message_output = format!("Account: {:?} has balance of {balance}", account);
                    
                    println!("{message_output}");
                    let _ = logger.log_wallet(message_output);
                }
                None => {
                    let _ = logger.log_wallet("No account selected".to_string());
                }
            }
        }
        MessageResponse::LastTransactions => {

        },
        MessageResponse::Exit => {}
    }

    Ok(())
}
