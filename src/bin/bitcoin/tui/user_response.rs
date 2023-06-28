use super::{
    account, error_tui::ErrorTUI, menu, menu_option::MenuOption, notify::notify, transaction,
};

use cargosos_bitcoin::{
    block_structure::{
        block::Block, block_chain::BlockChain, error_block::ErrorBlock, transaction::Transaction,
        utxo_set::UTXOSet,
    },
    logs::logger_sender::LoggerSender,
    node_structure::{
        broadcasting::Broadcasting, error_node::ErrorNode, message_response::MessageResponse,
    },
    wallet_structure::{account::Account, wallet::Wallet},
    notifications::notification::{Notification, NotificationSender},
};

use std::{
    net::TcpStream,
    sync::mpsc::Receiver,
    sync::{Arc, Mutex, MutexGuard},
    thread::{self, JoinHandle},
};

type MutArc<T> = Arc<Mutex<T>>;

/// Get a mutable guard to use the value inside the Arc<Mutex<T>>
///
/// ### Error
///  * `ErrorTUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
fn get_reference<T>(reference: &MutArc<T>) -> Result<MutexGuard<'_, T>, ErrorTUI> {
    match reference.lock() {
        Ok(reference) => Ok(reference),
        Err(_) => Err(ErrorTUI::CannotUnwrapArc),
    }
}

/// It will responde to the user input
///
/// ### Error
///  * `ErrorTUI::TerminalReadFail`: It will appear when the terminal read fails
///  * `ErrorTUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
///  * `ErrorTUI::ErrorFromPeer`: It will appear when a conextion with a peer fails
pub fn user_input(
    broadcasting: &mut Broadcasting<TcpStream>,
    wallet: MutArc<Wallet>,
    utxo_set: MutArc<UTXOSet>,
    block_chain: MutArc<BlockChain>,
    logger: LoggerSender,
) -> Result<(), ErrorTUI> {
    loop {
        match menu::select_option(logger.clone())? {
            MenuOption::CreateAccount => creating_accout(&wallet, logger.clone())?,
            MenuOption::ChangeAccount => changing_account(&wallet, logger.clone())?,
            MenuOption::RemoveAccount => removing_account(&wallet, logger.clone())?,
            MenuOption::SendTransaction => {
                sending_transaction(broadcasting, &wallet, &utxo_set, logger.clone())?
            }
            MenuOption::ShowAccounts => {
                let wallet_ref = get_reference(&wallet)?;
                account::show_accounts(&wallet_ref, logger.clone());
            }
            MenuOption::ShowBalance => showing_balance(
                &wallet,
                &utxo_set,
                logger.clone(),
            )?,
            MenuOption::LastTransactions => latest_transactions(&block_chain, logger.clone())?,
            MenuOption::Exit => break,
        }
    }

    Ok(())
}

/// Appends an new account to the wallet created by the user
///
/// ### Error
///  * `ErrorTUI::TerminalReadFail`: It will appear when the terminal read fails
///  * `ErrorTUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
fn creating_accout(wallet: &MutArc<Wallet>, logger: LoggerSender) -> Result<(), ErrorTUI> {
    let mut wallet = get_reference(wallet)?;
    let account = account::create_account(logger)?;
    wallet.add_account(account);

    Ok(())
}

/// Change the selected account to the one selected by the user
///
/// ### Error
///  * `ErrorTUI::TerminalReadFail`: It will appear when the terminal read fails
///  * `ErrorTUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
fn changing_account(wallet: &MutArc<Wallet>, logger: LoggerSender) -> Result<(), ErrorTUI> {
    let mut wallet = get_reference(wallet)?;
    let account = account::select_account(&wallet, logger)?;
    wallet.change_account(account);

    Ok(())
}

/// Delete the selected account selected by the user
///
/// ### Error
///  * `ErrorTUI::TerminalReadFail`: It will appear when the terminal read fails
///  * `ErrorTUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
fn removing_account(wallet: &MutArc<Wallet>, logger: LoggerSender) -> Result<(), ErrorTUI> {
    let mut wallet = get_reference(wallet)?;
    let account = account::select_account(&wallet, logger)?;
    wallet.remove_account(account);

    Ok(())
}

/// Broadcast the transaction created by the user to the peers from the selected account in the wallet
///
/// ### Error
///  * `ErrorTUI::TerminalReadFail`: It will appear when the terminal read fails
///  * `ErrorTUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
///  * `ErrorTUI::ErrorFromPeer`: It will appear when a conextion with a peer fails
fn sending_transaction(
    broadcasting: &mut Broadcasting<TcpStream>,
    wallet: &MutArc<Wallet>,
    utxo_set: &MutArc<UTXOSet>,
    logger: LoggerSender,
) -> Result<(), ErrorTUI> {
    let wallet = get_reference(wallet)?;
    let account = match wallet.get_selected_account() {
        Some(account) => account,
        None => {
            let message = "No account selected can't send transaction";
            println!("{message}");
            let _ = logger.log_wallet(message.to_string());
            return Ok(());
        }
    };
    let mut utxo_set = get_reference(utxo_set)?;

    let transaction = match transaction::create_transaction(&utxo_set, account, logger.clone()) {
        Ok(transaction) => transaction,
        Err(ErrorTUI::TransactionWithoutSufficientFunds) => {
            let message = "Transaction without sufficient funds";
            println!("{message}");
            let _ = logger.log_transaction(message.to_string());
            return Ok(());
        }
        Err(error) => return Err(error),
    };
    let _ = logger.log_transaction("Sending transaction".to_string());

    utxo_set.append_pending_transaction(transaction.clone());

    match broadcasting.send_transaction(transaction) {
        Ok(()) => Ok(()),
        Err(ErrorNode::WhileSendingMessage(message)) => Err(ErrorTUI::ErrorFromPeer(format!(
            "While sending message {message}"
        ))),
        _ => Err(ErrorTUI::ErrorFromPeer(
            "While sending transaction".to_string(),
        )),
    }
}

/// Show the balance of the selected account in the wallet
///
/// ### Error
///  * `ErrorTUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
fn showing_balance(
    wallet: &MutArc<Wallet>,
    utxo_set: &MutArc<UTXOSet>,
    logger: LoggerSender,
) -> Result<(), ErrorTUI> {
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
///  * `ErrorTUI::TerminalReadFail`: It will appear when the terminal read fails
///  * `ErrorTUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
fn latest_transactions(
    block_chain: &MutArc<BlockChain>,
    logger: LoggerSender,
) -> Result<(), ErrorTUI> {
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

/// Crate a thread for handling the blocks and transactions received
pub fn handle_peers(
    receiver_broadcasting: Receiver<MessageResponse>,
    wallet: MutArc<Wallet>,
    utxo_set: MutArc<UTXOSet>,
    block_chain: MutArc<BlockChain>,
    logger: LoggerSender,
    notifier: NotificationSender,
) -> JoinHandle<Result<(), ErrorTUI>> {
    thread::spawn(move || {
        for message in receiver_broadcasting {
            match message {
                MessageResponse::Block(block) => {
                    receive_block(
                        &utxo_set,
                        &block_chain,
                        block,
                        logger.clone(),
                        notifier.clone(),
                    )?;
                }
                MessageResponse::Transaction(transaction) => {
                    receive_transaction(
                        &wallet,
                        &utxo_set,
                        transaction,
                        logger.clone(),
                        notifier.clone(),
                    )?;
                }
            }
        }

        Ok(())
    })
}

/// Manage receiving a transaction by updating the list of transactions seen so far if the transaction is from the selected account
///
/// ### Error
///  * `ErrorTUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
fn receive_transaction(
    wallet: &MutArc<Wallet>,
    utxo_set: &MutArc<UTXOSet>,
    transaction: Transaction,
    logger: LoggerSender,
    notifier: NotificationSender,
) -> Result<(), ErrorTUI> {
    
    let mut utxo_set = get_reference(utxo_set)?;

    if utxo_set.is_transaction_pending(&transaction) {
        let _ = logger.log_wallet(format!(
            "Transaction {transaction} is already in the list of transactions seen so far",
        ));
        return Ok(());
    }

    let mut involved_accounts = Vec::new();
    for account in get_reference(wallet)?.get_accounts() {
        if account.verify_transaction_ownership(&transaction) {
            notify(
                &format!("New transaction received own by {}", account.account_name),
                &format!(
                    "The transaction: \n{transaction}\n has not been added to the blockchain yet"
                ),
                logger.clone(),
            );
            involved_accounts.push(account.clone());
        }
    }
    if !involved_accounts.is_empty(){
        let _ = notifier.send(Notification::TransactionOfAccountReceived(involved_accounts, transaction.clone()));
    }
    utxo_set.append_pending_transaction(transaction);
    Ok(())
}

/// Manage receiving a block by updating the block chain and the utxo set
///
/// ### Error
///  * `ErrorTUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
///  * `ErrorTUI::ErrorWriting`: It will appear when writing to the block chain
fn receive_block(
    utxo_set: &MutArc<UTXOSet>,
    block_chain: &MutArc<BlockChain>,
    block: Block,
    logger: LoggerSender,
    notifier: NotificationSender,
) -> Result<(), ErrorTUI> {

    let mut utxo_set = get_reference(utxo_set)?;

    for transaction in utxo_set.pending_transactions() {
        if block.transactions.contains(transaction) {
            notify(
                "Transaction added to blockchain",
                &format!("The transaction: \n{transaction}\n has been added to the blockchain"),
                logger.clone(),
            );
            let _ = notifier.send(Notification::TransactionOfAccountInNewBlock(transaction.clone()));
            let _ = logger.log_wallet(
                "Removing transaction from list of transaction seen so far".to_string(),
            );
        }
    }
    
    utxo_set.update_utxo_with_block(&block);
    let _ = notifier.send(Notification::NewBlockAddedToTheBlockchain(block.clone()));
    match get_reference(block_chain)?.append_block(block) {
        Ok(_) | Err(ErrorBlock::TransactionAlreadyInBlock) => Ok(()),
        _ => Err(ErrorTUI::ErrorWriting(
            "Error appending block to blockchain".to_string(),
        )),
    }
}
