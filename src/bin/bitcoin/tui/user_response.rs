use super::{menu, menu_option::MenuOption, error_tui::ErrorTUI};

use crate::process::message_broadcasting::MessageBroadcasting;

use cargosos_bitcoin::{
    logs::logger_sender::LoggerSender,
    block_structure::{block_chain::BlockChain, utxo_set::UTXOSet},
    wallet_structure::wallet::Wallet,
};

use std::{
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
};

pub fn user_input(logger: LoggerSender) -> Result<(), ErrorTUI> {
    loop {
        match menu::select_option(logger.clone())? {
            MenuOption::CreateAccount => todo!(),
            MenuOption::ChangeAccount => todo!(),
            MenuOption::SendTransaction => todo!(),
            MenuOption::ShowAccounts => todo!(),
            MenuOption::ShowBalance => todo!(),
            MenuOption::LastTransactions => todo!(),
            MenuOption::Exit => break,
        }
    }

    Ok(())
}

pub fn response_handle(
    receiver_broadcasting: Receiver<MessageBroadcasting>,
    wallet: Wallet,
    utxo_set: UTXOSet,
    block_chain: BlockChain,
    logger: LoggerSender,
) -> JoinHandle<(BlockChain, Wallet)> {
    thread::spawn(move || {
        for message in receiver_broadcasting {
            response(
                message, 
                &wallet,
                &utxo_set,
                &block_chain,
                logger.clone()
            );
        }

        (block_chain, wallet)
    })
}

fn response(
    message: MessageBroadcasting, 
    wallet: &Wallet,
    utxo_set: &UTXOSet,
    block_chain: &BlockChain,
    logger: LoggerSender
) {
    match message {
        MessageBroadcasting::Block(block) => todo!(),
        MessageBroadcasting::Transaction(transaction) => todo!(),
        MessageBroadcasting::Exit => todo!(),
    }
}