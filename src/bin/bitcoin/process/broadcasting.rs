use super::{
    error_process::ErrorProcess,
    reference::{get_reference, MutArc},
};

use cargosos_bitcoin::{
    block_structure::{
        block::Block, block_chain::BlockChain, error_block::ErrorBlock, transaction::Transaction,
        utxo_set::UTXOSet,
    },
    logs::logger_sender::LoggerSender,
    node_structure::{
        broadcasting::Broadcasting, connection_id::ConnectionId, message_response::MessageResponse,
        message_to_peer::MessageToPeer, peer_manager::PeerManager,
    },
    notifications::{notification::Notification, notifier::Notifier},
    wallet_structure::wallet::Wallet,
};

use std::{
    io::{Read, Write},
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
};

/// Gives the broadcasting the peers to broadcast the blocks and transactions
pub fn add_peer_to_broadcasting<N: Notifier + 'static, RW: Read + Write + Send + 'static>(
    broadcasting: &mut Broadcasting<RW>,
    connection: (RW, ConnectionId),
    sender_response: Sender<MessageResponse>,
    magic_numbers: [u8; 4],
    notifier: N,
    logger: LoggerSender,
) {
    let peer_manager = create_peer_manager(
        connection,
        sender_response.clone(),
        magic_numbers,
        notifier.clone(),
        logger.clone(),
    );

    let (sender, receiver) = channel::<MessageToPeer>();

    broadcasting.add_connection(peer_manager, (sender, receiver));

    let _ = logger.log_node("Adding new peer to bradcasting".to_string());
}

/// Creates a peer manager to manege the message of this peer
fn create_peer_manager<N: Notifier + 'static, RW: Read + Write + Send + 'static>(
    connection: (RW, ConnectionId),
    sender_response: Sender<MessageResponse>,
    magic_numbers: [u8; 4],
    notifier: N,
    logger: LoggerSender,
) -> PeerManager<RW, N> {
    let _ = logger.log_node("Creating new Peer manager".to_string());
    PeerManager::<RW, N>::new(
        connection.1,
        connection.0,
        sender_response,
        magic_numbers,
        notifier,
        logger,
    )
}

/// Create a thread for handling the blocks and transactions received
pub fn handle_peers<N: Notifier + 'static>(
    receiver_broadcasting: Receiver<MessageResponse>,
    wallet: MutArc<Wallet>,
    utxo_set: MutArc<UTXOSet>,
    block_chain: MutArc<BlockChain>,
    notifier: N,
    logger: LoggerSender,
) -> JoinHandle<Result<(), ErrorProcess>> {
    thread::spawn(move || {
        for message in receiver_broadcasting {
            match message {
                MessageResponse::Block(block) => {
                    receive_block(&utxo_set, &wallet, &block_chain, block, notifier.clone())?;
                }
                MessageResponse::Transaction(transaction) => {
                    receive_transaction(
                        &wallet,
                        transaction,
                        &utxo_set,
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
///  * `ErrorUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
fn receive_transaction<N: Notifier>(
    wallet: &MutArc<Wallet>,
    transaction: Transaction,
    utxo_set: &MutArc<UTXOSet>,
    logger: LoggerSender,
    notifier: N,
) -> Result<(), ErrorProcess> {
    let mut utxo_set = get_reference(utxo_set)?;

    if utxo_set.is_transaction_pending(&transaction) {
        let _ = logger.log_wallet(format!(
            "Transaction {transaction} is already in the list of transactions seen so far",
        ));
        return Ok(());
    }

    let mut involved_accounts = Vec::new();
    for account in get_reference(wallet)?.get_accounts() {
        if account.verify_transaction_ownership(&(transaction.clone())) {
            let _ = logger.log_wallet(format!(
                "Transaction {transaction} is owned by account {account}"
            ));
            involved_accounts.push(account.clone());
        }
    }

    if !involved_accounts.is_empty() {
        notifier.notify(Notification::TransactionOfAccountReceived(
            involved_accounts,
            transaction.clone(),
        ));
    }

    utxo_set.append_pending_transaction(transaction);
    Ok(())
}

/// Manage receiving a block by updating the block chain and the utxo set
///
/// ### Error
///  * `ErrorUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
///  * `ErrorUI::ErrorWriting`: It will appear when writing to the block chain
fn receive_block<N: Notifier>(
    utxo_set: &MutArc<UTXOSet>,
    wallet: &MutArc<Wallet>,
    block_chain: &MutArc<BlockChain>,
    block: Block,
    notifier: N,
) -> Result<(), ErrorProcess> {
    let mut utxo_set = get_reference(utxo_set)?;
    let wallet = get_reference(wallet)?;

    for transaction in utxo_set.pending_transactions() {
        if block.transactions.contains(transaction)
            && wallet
                .get_accounts()
                .iter()
                .any(|account| account.verify_transaction_ownership(transaction))
        {
            notifier.notify(Notification::TransactionOfAccountInNewBlock(
                transaction.clone(),
            ));
        }
    }

    utxo_set.update_utxo_with_block(&block);

    notifier.notify(Notification::NewBlockAddedToTheBlockchain(block.clone()));

    match get_reference(block_chain)?.append_block(block) {
        Ok(_) | Err(ErrorBlock::TransactionAlreadyInBlock) => Ok(()),
        _ => Err(ErrorProcess::ErrorWriting),
    }
}
