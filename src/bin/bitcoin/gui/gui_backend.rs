use super::{
    signal_to_front::SignalToFront,
    signal_to_back::SignalToBack, error_gui::ErrorGUI,
};

use gtk::glib;
use std::thread;

use crate::{
    error_execution::ErrorExecution,
    process::{
        download, handshake,
        load_system::LoadSystem, 
        save_system::SaveSystem,
    },
};


use cargosos_bitcoin::configurations::{
    connection_config::ConnectionConfig, download_config::DownloadConfig,
    save_config::SaveConfig,
};

use cargosos_bitcoin::{
    logs::logger_sender::LoggerSender,
    block_structure::{
        block_chain::BlockChain,
        utxo_set::UTXOSet,
        transaction::Transaction,
        block::Block,
        error_block::ErrorBlock,
    },
    connections::ibd_methods::IBDMethod,
    wallet_structure::{
        wallet::Wallet,
        account::Account,
        private_key::PrivateKey,
        public_key::PublicKey,
        address::Address,
        error_wallet::ErrorWallet,
    },
    node_structure::{
        broadcasting::Broadcasting, 
        message_response::MessageResponse,
        error_node::ErrorNode,
    }
};

use std::{
    net::{SocketAddr, TcpStream},
    sync::{Arc, Mutex, MutexGuard},
    thread::JoinHandle,
    sync::mpsc::{
        Receiver,
        Sender,
    },
};

use std::sync::mpsc;

type MutArc<T> = Arc<Mutex<T>>;

fn get_reference<'t, T>(reference: &'t MutArc<T>) -> Result<MutexGuard<'t, T>, ErrorGUI> {
    match reference.lock() {
        Ok(reference) => Ok(reference),
        Err(_) => Err(ErrorGUI::CannotUnwrapArc),
    }
}

/// Get the value of a mutable reference given by Arc<Mutex<T>>
///
/// ### Error
///  * `ErrorTUI::CannotGetInner`: It will appear when we try to get the inner value of a mutex
///  * `ErrorTUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
fn get_inner<T>(reference: Arc<Mutex<T>>) -> Result<T, ErrorGUI> {
    match Arc::try_unwrap(reference) {
        Ok(reference_unwrap) => match reference_unwrap.into_inner() {
            Ok(reference) => Ok(reference),
            Err(_) => Err(ErrorGUI::CannotGetInner),
        },
        Err(_) => Err(ErrorGUI::CannotUnwrapArc),
    }
}

fn get_potential_peers(
    connection_config: ConnectionConfig,
    logger: LoggerSender,
) -> Result<Vec<SocketAddr>, ErrorExecution> {
    logger.log_connection("Getting potential peers with dns seeder".to_string())?;

    let potential_peers = connection_config.dns_seeder.discover_peers()?;

    let peer_count_max = std::cmp::min(connection_config.peer_count_max, potential_peers.len());

    let potential_peers = potential_peers[0..peer_count_max].to_vec();

    for potential_peer in &potential_peers {
        logger.log_connection(format!("Potential peer: {:?}", potential_peer))?;
    }

    Ok(potential_peers)
}

fn get_block_chain(
    peer_streams: Vec<TcpStream>,
    block_chain: &mut BlockChain,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    logger: LoggerSender,
) -> Result<Vec<TcpStream>, ErrorExecution> {
    logger.log_connection("Getting block chain".to_string())?;

    Ok(match connection_config.ibd_method {
        IBDMethod::HeaderFirst => {
            download::headers_first(
                peer_streams,
                block_chain,
                connection_config,
                download_config,
                logger,
            )?
        }
        IBDMethod::BlocksFirst => {
            download::blocks_first::<TcpStream>()
        }
    })
}

/// Creates the UTXO set from the given block chain
fn get_utxo_set(block_chain: &BlockChain, logger: LoggerSender) -> UTXOSet {
    let _ = logger.log_wallet("Creating the UTXO set".to_string());

    let utxo_set = UTXOSet::from_blockchain(&block_chain);

    let _ = logger.log_wallet("UTXO set finished successfully".to_string());
    utxo_set
}

/// Creates the broadcasting
fn get_broadcasting(
    peer_streams: Vec<TcpStream>,
    sender_response: Sender<MessageResponse>,
    connection_config: ConnectionConfig,
    logger: LoggerSender,
) -> Broadcasting<TcpStream> {
    let _ = logger.log_node("Broadcasting".to_string());
    Broadcasting::new(peer_streams, sender_response, connection_config, logger)
}

fn receive_transaction(
    wallet: &MutArc<Wallet>,
    transaction: Transaction,
    pending_transactions: MutArc<Vec<Transaction>>,
    logger: LoggerSender,
    tx_to_front: glib::Sender<SignalToFront>,
) -> Result<(), ErrorGUI> {
    let mut transaction_owned_by_account = false;

    let mut receiving_account_name: String = "".to_string();

    for account in get_reference(&wallet)?.get_accounts() {
        if account.verify_transaction_ownership(&(transaction.clone())) {
            let _ = logger.log_wallet(format!(
                "Transaction {transaction} is owned by account {account}",
                transaction = transaction,
                account = account
            ));
            receiving_account_name = account.account_name.clone();
            transaction_owned_by_account = true;
        }
    }

    let mut pending_transaction_reference = get_reference(&pending_transactions)?;
    if transaction_owned_by_account {
        if pending_transaction_reference.contains(&transaction) {
            let _ = logger.log_wallet(format!(
                "Transaction {transaction} is already in the list of transactions seen so far",
            ));
            return Ok(());
        }
        pending_transaction_reference.push(transaction);
        if tx_to_front.send(SignalToFront::Update).is_err() || 
           tx_to_front.send(SignalToFront::TransactionOfAccountReceived(receiving_account_name)).is_err() {
            return Err(ErrorGUI::FailedSignalToFront("TransactionReceived".to_string()));
        }
    }

    Ok(())
}

fn receive_block(
    utxo_set: &MutArc<UTXOSet>,
    block_chain: &MutArc<BlockChain>,
    block: Block,
    pending_transactions: MutArc<Vec<Transaction>>,
    logger: LoggerSender,
    tx_to_front: glib::Sender<SignalToFront>
) -> Result<(), ErrorGUI> {
    get_reference(&pending_transactions)?.retain(|transaction| {
        if block.transactions.contains(transaction) {
            let _ = logger.log_wallet(format!(
                "Removing transaction from list of transaction seen so far"
            ));
            if tx_to_front.send(SignalToFront::BlockWithUnconfirmedTransactionReceived).is_err() {
                println!("Error sending signal to front")
            }
            return false;
        }
        true
    });

    let mut utxo_set = get_reference(&utxo_set)?;
    utxo_set.update_utxo_with_block(&block);
    if tx_to_front.send(SignalToFront::Update).is_err() {
        return Err(ErrorGUI::FailedSignalToFront("TransactionInBlock".to_string()));
    }

    let mut block_chain = get_reference(&block_chain)?;
    match block_chain.append_block(block) {
        Ok(_) | Err(ErrorBlock::TransactionAlreadyInBlock) => Ok(()),
        _ => Err(ErrorGUI::ErrorWriting(
            "Error appending block to blockchain".to_string(),
        )),
    }
}

pub fn handle_peers(
    tx_to_front: glib::Sender<SignalToFront>,
    receiver_broadcasting: Receiver<MessageResponse>,
    wallet: MutArc<Wallet>,
    utxo_set: MutArc<UTXOSet>,
    pending_transactions: MutArc<Vec<Transaction>>,
    block_chain: MutArc<BlockChain>,
    logger: LoggerSender,
) -> JoinHandle<Result<(), ErrorGUI>> {
    thread::spawn(move || {
        for message in receiver_broadcasting {
            match message {
                MessageResponse::Block(block) => {
                    receive_block(
                        &utxo_set,
                        &block_chain,
                        block,
                        pending_transactions.clone(),
                        logger.clone(),
                        tx_to_front.clone(),
                    )?;
                }
                MessageResponse::Transaction(transaction) => {
                    receive_transaction(
                        &wallet, 
                        transaction, 
                        pending_transactions.clone(), 
                        logger.clone(), 
                        tx_to_front.clone()
                    )?;
                }
            }
        };

        Ok(())
    })
}

pub fn fron_tbtc_to_satoshi(tbtc: f64) -> i64 {
    (tbtc * 100_000_000.0) as i64
}

/// Creates a transaction via terminal given the user user_input
///
/// ### Error
///  * `ErrorTUI::TransactionWithoutSufficientFunds`: It will appear when the user does not have enough funds to make the transaction
///  * `ErrorTUI::TransactionCreationFail`: It will appear when the transaction fail to create the signature script
pub fn create_transaction<'t>(
    utxo_set: &MutexGuard<'t, UTXOSet>,
    account: &Account,
    logger: LoggerSender,
    address: &Address,
    amount: f64,
    fee: f64,
) -> Result<Transaction, ErrorGUI> {
    let available_outputs = utxo_set.get_utxo_list_with_outpoints(Some(&account.address));

    match account.create_transaction_with_available_outputs(address.clone(), fron_tbtc_to_satoshi(amount), fron_tbtc_to_satoshi(fee), available_outputs)
    {
        Ok(transaction) => Ok(transaction),
        Err(ErrorWallet::NotEnoughFunds(error_string)) => {
            let _ = logger.log_wallet(format!(
                "Error creating transaction, with error: {:?}",
                ErrorWallet::NotEnoughFunds(error_string))
            );
            Err(ErrorGUI::ErrorInTransaction("Not enough funds".to_string()))
        },
        Err(error) => {
            let _ = logger.log_wallet(format!(
                "Error creating transaction, with error: {:?}",
                error
            ));
            Err(ErrorGUI::ErrorInTransaction("Transaction creation failed".to_string()))
        }
    }
}

fn sending_transaction(
    broadcasting: &mut Broadcasting<TcpStream>,
    wallet: &MutArc<Wallet>,
    utxo_set: &MutArc<UTXOSet>,
    logger: LoggerSender,
    address_string: &str,
    amount: f64,
    fee: f64,
    tx_to_front: glib::Sender<SignalToFront>,
) -> Result<(), ErrorGUI> {

    let address = match Address::new(address_string) {
        Ok(address) => address,
        Err(_) => {
            let message = "Invalid address";
            let _ = logger.log_wallet(message.to_string());
            if tx_to_front.send(SignalToFront::ErrorInTransaction(message.to_string())).is_err() {
                return Err(ErrorGUI::FailedSignalToFront(
                    "Failed to send error signal to front".to_string(),
                ));
            }
            return Ok(());
        }
    };

    let wallet = get_reference(wallet)?;
    let account = match wallet.get_selected_account() {
        Some(account) => account,
        None => {
            let message = "No account selected can't send transaction";
            let _ = logger.log_wallet(message.to_string());
            if tx_to_front.send(SignalToFront::ErrorInTransaction(message.to_string())).is_err() {
                return Err(ErrorGUI::FailedSignalToFront(
                    "Failed to send error signal to front".to_string(),
                ));
            }
            return Ok(());
        }
    };
    let utxo_set = get_reference(utxo_set)?;

    let transaction = match create_transaction(&utxo_set, account, logger.clone(), &address, amount, fee) {
        Ok(transaction) => transaction,
        Err(error) => {
            if tx_to_front.send(SignalToFront::ErrorInTransaction("Error creating the transaction".to_string())).is_err() {
                return Err(ErrorGUI::FailedSignalToFront(
                    "Failed to send error signal to front".to_string(),
                ));
            };
            return Err(error)
        },
    };
    let _ = logger.log_transaction("Sending transaction".to_string());

    match broadcasting.send_transaction(transaction) {
        Ok(()) => Ok(()),
        Err(ErrorNode::WhileSendingMessage(message)) => Err(ErrorGUI::ErrorFromPeer(message)),
        _ => Err(ErrorGUI::ErrorFromPeer(
            "While sending transaction".to_string(),
        )),
    }
}

pub fn create_account(
    wallet: MutArc<Wallet>,
    account_name: &str,
    private_key_string: &str,
    public_key_string: &str,
    tx_to_front: glib::Sender<SignalToFront>,
    logger: LoggerSender,
) -> Result<(), ErrorGUI> {
    
    let mut wallet = get_reference(&wallet)?;

    let private_key = match PrivateKey::try_from(private_key_string) {
        Ok(private_key) => private_key,
        Err(_) => {
            let message = "Invalid private key";
            let _ = logger.log_wallet(message.to_string());
            if tx_to_front.send(SignalToFront::ErrorInAccountCreation(message.to_string())).is_err() {
                return Err(ErrorGUI::FailedSignalToFront(
                    "Failed to send error signal to front".to_string(),
                ));
            }
            return Ok(());
        }
    };

    let public_key = match PublicKey::try_from(public_key_string.to_string()) {
        Ok(public_key) => public_key,
        Err(_) => {
            let message = "Invalid public key";
            let _ = logger.log_wallet(message.to_string());
            if tx_to_front.send(SignalToFront::ErrorInAccountCreation(message.to_string())).is_err() {
                return Err(ErrorGUI::FailedSignalToFront(
                    "Failed to send error signal to front".to_string(),
                ));
            }
            return Ok(());
        }
    };

    let address = match Address::from_public_key(&public_key) {
        Ok(result) => result,
        Err(error) => {
            let message = format!("Error creating address: {:?}", error);
            let _ = logger.log_wallet(message.to_string());
            if tx_to_front.send(SignalToFront::ErrorInAccountCreation(message.to_string())).is_err() {
                return Err(ErrorGUI::FailedSignalToFront(
                    "Failed to send error signal to front".to_string(),
                ));
            }
            return Ok(());
        }
    };

    let account = Account {
        account_name: account_name.to_string(),
        private_key,
        public_key,
        address,
    };

    wallet.add_account(account);
    if tx_to_front.send(SignalToFront::RegisterWallet(account_name.to_string())).is_err() {
        return Err(ErrorGUI::FailedSignalToFront(
            "Failed to send account created signal to front".to_string(),
        ));
    }

    Ok(())
}

pub fn get_account_transactions_information(account: &Account, blockchain: &BlockChain) -> Vec<(u32, [u8;32], i64)> {
    let mut transactions: Vec<Transaction> = Vec::new();
    let blocks = blockchain.get_all_blocks();
    for block in blocks {
        for transaction in block.transactions {
            if account.verify_transaction_ownership(&transaction) {
                transactions.push(transaction);
            }
        }
    }
    let filtered_transactions = transactions.iter().filter_map(|transaction| {
        let timestamp = transaction.time;
        let label = match transaction.get_tx_id(){
            Ok(txid) => txid,
            Err(_) => return None,
        };
        let mut amount: i64 = 0;
        for utxo in transaction.tx_out.clone() {
            if account.verify_transaction_output_ownership(&utxo) {
                amount += utxo.value;
            }
        }
        Some((timestamp, label, amount))
    }).collect();
    filtered_transactions
}

fn give_account_transactions(
    wallet: MutArc<Wallet>,
    blockchain: MutArc<BlockChain>,
    logger: LoggerSender,
    tx_to_front: glib::Sender<SignalToFront>
) -> Result<(), ErrorGUI>{
    let wallet = get_reference(&wallet).unwrap();
    let blockchain = get_reference(&blockchain).unwrap();

    let account = match wallet.get_selected_account() {
        Some(account) => account,
        None => {
            let message = "No account selected cannot get transactions";
            let _ = logger.log_wallet(message.to_string());
            if tx_to_front.send(SignalToFront::ErrorInTransaction(message.to_string())).is_err() {
                return Err(ErrorGUI::FailedSignalToFront(
                    "Failed to send error signal to front".to_string(),
                ));
            }
            return Ok(());
        }
    };

    let transactions = get_account_transactions_information(&account, &blockchain);
    if tx_to_front.send(SignalToFront::AccountTransactions(transactions)).is_err() {
        if tx_to_front.send(SignalToFront::ErrorInTransaction("Failed to send transactions to front".to_string())).is_err() {
            return Err(ErrorGUI::FailedSignalToFront(
                "Failed to send error signal to front".to_string(),
            ));
        };
        return Err(ErrorGUI::FailedSignalToFront(
            "Failed to send error signal to front".to_string(),
        ));
    }

    Ok(())
}

pub fn spawn_frontend_handler(
    rx_from_front: Receiver<SignalToBack>,
    tx_to_front: glib::Sender<SignalToFront>,
    broadcasting: &mut Broadcasting<TcpStream>,
    wallet: MutArc<Wallet>,
    utxo_set: MutArc<UTXOSet>,
    pending_transactions: MutArc<Vec<Transaction>>,
    block_chain: MutArc<BlockChain>,
    logger: LoggerSender,
) -> Result<(), ErrorGUI> {
    for rx in rx_from_front {
        match rx {
            SignalToBack::GetAccountBalance => {
                give_account_balance(wallet.clone(), utxo_set.clone(), pending_transactions.clone(), tx_to_front.clone())?;
            },
            SignalToBack::ChangeSelectedAccount(account_name) => {
                change_selected_account(account_name, wallet.clone(), tx_to_front.clone())?;
            },
            SignalToBack::CreateTransaction(address_string, amount, fee) => {
                sending_transaction(broadcasting, &wallet, &utxo_set, logger.clone(), &address_string, amount, fee, tx_to_front.clone())?;
            }
            SignalToBack::CreateAccount(name, private_key, public_key) => {
                create_account(wallet.clone(), &name, &private_key, &public_key, tx_to_front.clone(), logger.clone())?;
            },
            SignalToBack::GetAccountTransactions => {
                give_account_transactions(wallet.clone(), block_chain.clone(), logger.clone(), tx_to_front.clone())?;
            },
            SignalToBack::ExitProgram => {
                break;
            },
        }
    }
    Ok(())
}

pub fn change_selected_account(
    account_name: String,
    wallet: MutArc<Wallet>,
    tx_to_front: glib::Sender<SignalToFront>,
) -> Result<(), ErrorGUI> {
    let mut wallet_reference = get_reference(&wallet)?;

    let account_to_select = match wallet_reference.get_account_with_name(&account_name) {
        Some(account) => account.clone(),
        None => {
            return Err(ErrorGUI::ErrorReading(
                "Account does not exist".to_string(),
            ))
        }
    };

    wallet_reference.change_account(account_to_select.clone());

    if tx_to_front.send(SignalToFront::Update).is_err() {
        return Err(ErrorGUI::FailedSignalToFront(
            "Failed to send update signal to front".to_string(),
        ));
    }

    Ok(())
}

pub fn get_pending_amount(
    pending_transactions: MutArc<Vec<Transaction>>,
    account: &Account,
) -> Result<f64, ErrorGUI> {
    let mut pending: f64 = 0.0;
    for transaction in get_reference(&pending_transactions)?.iter() {
        for transaction_output in transaction.tx_out.iter() {
            if account.verify_transaction_output_ownership(transaction_output) {
                pending += transaction_output.value as f64 / 100_000_000.0;
            }
        }
    }

    Ok(pending)
}

pub fn give_account_balance(
    wallet: MutArc<Wallet>,
    utxo_set: MutArc<UTXOSet>,
    pending_transactions: MutArc<Vec<Transaction>>,
    tx_to_front: glib::Sender<SignalToFront>,
) -> Result<(), ErrorGUI> {
    let wallet_reference = get_reference(&wallet)?;
    let utxo_set_reference = get_reference(&utxo_set)?;

    let account_to_check = match wallet_reference.get_selected_account().clone() {
        Some(account) => account,
        None => {
            return Err(ErrorGUI::ErrorReading(
                "No account selected".to_string(),
            ))
        }
    };
    let balance = utxo_set_reference.get_balance_in_tbtc(&account_to_check.address);
    let pending = get_pending_amount(pending_transactions, &account_to_check)?;
    if tx_to_front.send(SignalToFront::LoadAvailableBalance((balance, pending))).is_err() {
        return Err(ErrorGUI::FailedSignalToFront(
            "Failed to send available balance to front".to_string(),
        ));
    }
    Ok(())
}

fn broadcasting(
    rx_from_front: Receiver<SignalToBack>,
    tx_to_front: glib::Sender<SignalToFront>,
    peer_streams: Vec<TcpStream>,
    wallet: Arc<Mutex<Wallet>>,
    utxo_set: Arc<Mutex<UTXOSet>>,
    block_chain: Arc<Mutex<BlockChain>>,
    connection_config: ConnectionConfig,
    logger: LoggerSender,
) -> Result<(), ErrorExecution> {
    let (sender_response, receiver_response) = mpsc::channel::<MessageResponse>();
    let pending_transactions = Arc::new(Mutex::new(Vec::<Transaction>::new()));

    let handle = handle_peers(
        tx_to_front.clone(),
        receiver_response,
        wallet.clone(),
        utxo_set.clone(),
        pending_transactions.clone(),
        block_chain.clone(),
        logger.clone(),
    );

    let mut broadcasting = get_broadcasting(
        peer_streams,
        sender_response,
        connection_config,
        logger.clone(),
    );

    spawn_frontend_handler(
        rx_from_front,
        tx_to_front.clone(),
        &mut broadcasting,
        wallet.clone(),
        utxo_set,
        pending_transactions,
        block_chain.clone(),
        logger.clone(),
    )?;

    broadcasting.destroy()?;

    match handle.join() {
        Ok(_) => Ok(()),
        Err(_) => Err(ErrorGUI::ErrorFromPeer("Failed to remove notifications".to_string()).into()),
    }
}

pub fn backend_initialization(
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    load_system: LoadSystem,
    logger: LoggerSender,
    tx_to_front: glib::Sender<SignalToFront>,
    rx_from_front: mpsc::Receiver<SignalToBack>,
) -> Result<SaveSystem, ErrorExecution> {

    let mut load_system = load_system;

    let potential_peers = get_potential_peers(connection_config.clone(), logger.clone())?;

    let peer_streams = handshake::connect_to_peers(
        potential_peers,
        connection_config.clone(),
        logger.clone(),
    );

    let mut block_chain = load_system.get_block_chain()?;

    let peer_streams = get_block_chain(
        peer_streams,
        &mut block_chain,
        connection_config.clone(),
        download_config,
        logger.clone(),
    )?;

    let wallet = load_system.get_wallet()?;
    for account in wallet.get_accounts().iter() {
        tx_to_front.send(SignalToFront::RegisterWallet(account.account_name.clone())).unwrap();
    }

    tx_to_front.send(SignalToFront::NotifyBlockchainIsReady).unwrap();

    let wallet = Arc::new(Mutex::new(wallet));
    let utxo_set = Arc::new(Mutex::new(get_utxo_set(&block_chain, logger.clone())));
    let block_chain = Arc::new(Mutex::new(block_chain));

    broadcasting(
        rx_from_front,
        tx_to_front.clone(),
        peer_streams,
        wallet.clone(),
        utxo_set,
        block_chain.clone(),
        connection_config,
        logger.clone(),
    )?;

    Ok(SaveSystem::new(
        get_inner(block_chain)?,
        get_inner(wallet)?,
        logger,
    ))
}

pub fn spawn_backend_handler(
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    save_config: SaveConfig,
    logger: LoggerSender,
    tx_to_front: glib::Sender<SignalToFront>,
    rx_from_front: mpsc::Receiver<SignalToBack>,
) -> thread::JoinHandle<Result<SaveSystem, ErrorExecution>> {
    thread::spawn(move || {
        let load_system = LoadSystem::new(
            save_config.clone(),
            logger.clone(),
        );
        backend_initialization(connection_config, download_config, load_system, logger, tx_to_front, rx_from_front)
    })
}