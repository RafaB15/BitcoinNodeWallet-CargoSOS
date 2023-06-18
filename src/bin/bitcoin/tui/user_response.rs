use super::{account, error_tui::ErrorTUI, menu, menu_option::MenuOption, transaction};

use crate::process::{broadcasting::Broadcasting, message_response::MessageResponse};

use cargosos_bitcoin::{
    block_structure::{
        block::Block, block_chain::BlockChain, transaction::Transaction, utxo_set::UTXOSet,
    },
    logs::logger_sender::LoggerSender,
    wallet_structure::wallet::Wallet,
};

use std::{
    net::TcpStream,
    sync::mpsc::Receiver,
    sync::{Arc, Mutex, MutexGuard},
    thread::{self, JoinHandle},
};

type MutArc<T> = Arc<Mutex<T>>;

fn get_reference<'t, T>(reference: &'t MutArc<T>) -> Result<MutexGuard<'t, T>, ErrorTUI> {
    match reference.lock() {
        Ok(reference) => Ok(reference),
        Err(_) => todo!(),
    }
}

pub fn user_input(
    broadcasting: &mut Broadcasting<TcpStream>,
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
            MenuOption::SendTransaction => {
                let transaction = transaction::create_transaction();
                let _ = logger.log_transaction("Sending transaction".to_string());
                broadcasting.send_transaction(transaction);
            }
            MenuOption::ShowAccounts => account::show_accounts(&wallet, logger.clone()),
            MenuOption::ShowBalance => {
                let account = wallet.get_selected_account();
                match account {
                    Some(account) => {
                        let balance = utxo_set.get_balance_in_satoshis(&account.address);
                        let message_output =
                            format!("Account: {:?} has balance of {balance}", account.account_name);

                        println!("{message_output}");
                        let _ = logger.log_wallet(message_output);
                    }
                    None => {
                        let _ = logger.log_wallet("No account selected".to_string());
                    }
                }
            }
            MenuOption::LastTransactions => {
                let selected_timestamp = transaction::select_option(logger.clone())?;
                let timestamp = selected_timestamp.get_timestamps_from_now();

                let _ = logger.log_transaction(format!(
                    "Selected timestamp: {selected_timestamp}, and it's corresponding timestamp: {timestamp}"
                ));

                let blocks = match block_chain.get_blocks_after_timestamp(timestamp as u32) {
                    Ok(blocks) => blocks,
                    Err(_error) => todo!(),
                };

                for block in blocks {
                    for transaction in block.transactions {
                        println!("{transaction}");
                    }
                }
            }
            MenuOption::Exit => break,
        }
    }

    Ok(())
}

pub fn handle_peers(
    receiver_broadcasting: Receiver<MessageResponse>,
    wallet: MutArc<Wallet>,
    utxo_set: MutArc<UTXOSet>,
    block_chain: MutArc<BlockChain>,
    logger: LoggerSender,
) -> JoinHandle<Result<(), ErrorTUI>> {
    thread::spawn(move || {
        let mut transactions: Vec<Transaction> = Vec::new();

        for message in receiver_broadcasting {
            match message {
                MessageResponse::Block(block) => {
                    receive_block(
                        &utxo_set,
                        &block_chain,
                        block,
                        &mut transactions,
                        logger.clone(),
                    )?;
                }
                MessageResponse::Transaction(transaction) => {
                    receive_transaction(&wallet, transaction, &mut transactions, logger.clone())?;
                }
            }
        }

        Ok(())
    })
}

fn receive_transaction(
    wallet: &MutArc<Wallet>,
    transaction: Transaction,
    transactions: &mut Vec<Transaction>,
    logger: LoggerSender,
) -> Result<(), ErrorTUI> {
    let _ = logger.log_wallet(format!("Receive transaction: {:?}", transaction));

    if let Some(account) = get_reference(&wallet)?.get_selected_account() {
        if account.verify_transaction_ownership(&transaction) {
            println!("{transaction} is valid and has not been added to the blockchain yet");
            let _ = logger.log_wallet(format!(
                "Adding transaction {transaction} to list of transaction seen so far"
            ));
            transactions.push(transaction);
        }
    }

    Ok(())
}

fn receive_block(
    utxo_set: &MutArc<UTXOSet>,
    block_chain: &MutArc<BlockChain>,
    block: Block,
    transactions: &mut Vec<Transaction>,
    logger: LoggerSender,
) -> Result<(), ErrorTUI> {
    let _ = logger.log_wallet(format!("Receive block: {:?}", block));

    transactions.retain(|transaction| {
        if block.transactions.contains(transaction) {
            println!("{transaction} has been added to the blockchain");
            let _ = logger.log_wallet(format!(
                "Removing transaction {transaction} from list of transaction seen so far"
            ));
            false
        } else {
            true
        }
    });

    let mut utxo_set = get_reference(&utxo_set)?;
    let mut block_chain = get_reference(&block_chain)?;

    utxo_set.update_utxo_with_block(&block);
    if block_chain.append_block(block).is_err() {
        todo!()
    }

    Ok(())
}
