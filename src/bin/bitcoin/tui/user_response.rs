use super::{error_tui::ErrorTUI, menu, menu_option::MenuOption};

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

pub fn response_handle(
    receiver_broadcasting: Receiver<MessageResponse>,
    wallet: Wallet,
    utxo_set: UTXOSet,
    block_chain: BlockChain,
    logger: LoggerSender,
) -> JoinHandle<(BlockChain, Wallet)> {
    thread::spawn(move || {
        for message in receiver_broadcasting {
            response(message, &wallet, &utxo_set, &block_chain, logger.clone());
        }

        (block_chain, wallet)
    })
}

fn response(
    message: MessageResponse,
    wallet: &Wallet,
    utxo_set: &UTXOSet,
    block_chain: &BlockChain,
    logger: LoggerSender,
) {
    match message {
        MessageResponse::Block(block) => todo!(),
        MessageResponse::Transaction(transaction) => todo!(),
        MessageResponse::CreateAccount => todo!(),
        MessageResponse::ChangeAccount => todo!(),
        MessageResponse::SendTransaction => todo!(),
        MessageResponse::ShowAccounts => todo!(),
        MessageResponse::ShowBalance => todo!(),
        MessageResponse::LastTransactions => todo!(),
        MessageResponse::Exit => {}
    }
}
