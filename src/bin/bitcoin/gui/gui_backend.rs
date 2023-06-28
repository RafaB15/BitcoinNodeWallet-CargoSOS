use super::{signal_to_back::SignalToBack, signal_to_front::SignalToFront};

use gtk::glib;
use std::thread;

use crate::{
    error_execution::ErrorExecution,
    process::{
        broadcasting::{create_transaction, get_broadcasting, handle_peers},
        download, handshake,
        load_system::LoadSystem,
        reference::{get_inner, get_reference, MutArc},
        save_system::SaveSystem,
    },
    ui::error_ui::ErrorUI,
};

use cargosos_bitcoin::configurations::{
    connection_config::ConnectionConfig, download_config::DownloadConfig, mode_config::ModeConfig,
    save_config::SaveConfig, server_config::ServerConfig,
};

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, transaction::Transaction, utxo_set::UTXOSet},
    connections::ibd_methods::IBDMethod,
    logs::logger_sender::LoggerSender,
    node_structure::{
        broadcasting::Broadcasting, error_node::ErrorNode, message_response::MessageResponse,
    },
    notifications::notification::{Notification, NotificationSender},
    wallet_structure::{
        account::Account, address::Address, private_key::PrivateKey, public_key::PublicKey,
        wallet::Wallet,
    },
};

use std::{
    net::{IpAddr, SocketAddr, TcpStream},
    sync::mpsc::Receiver,
    sync::{Arc, Mutex},
};

use std::sync::mpsc;

/// Get the peers from the dns seeder
///
/// ### Error
///  * `ErrorUI::ErrorFromPeer`: It will appear when a conextion with a peer fails
fn get_potential_peers(
    server_config: ServerConfig,
    logger: LoggerSender,
) -> Result<Vec<SocketAddr>, ErrorExecution> {
    logger.log_connection("Getting potential peers with dns seeder".to_string())?;

    let potential_peers = server_config.dns_seeder.discover_peers()?;

    let peer_count_max = std::cmp::min(server_config.peer_count_max, potential_peers.len());

    let potential_peers = potential_peers[0..peer_count_max].to_vec();

    for potential_peer in &potential_peers {
        logger.log_connection(format!("Potential peer: {:?}", potential_peer))?;
    }

    Ok(potential_peers)
}

/// Updates the blockchain with the new blocks and returns the TcpStreams that are still connected
fn get_block_chain(
    peer_streams: Vec<TcpStream>,
    block_chain: &mut BlockChain,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    logger: LoggerSender,
) -> Result<Vec<TcpStream>, ErrorExecution> {
    logger.log_connection("Getting block chain".to_string())?;

    Ok(match connection_config.ibd_method {
        IBDMethod::HeaderFirst => download::headers_first(
            peer_streams,
            block_chain,
            connection_config,
            download_config,
            logger,
        )?,
        IBDMethod::BlocksFirst => download::blocks_first::<TcpStream>(),
    })
}

/// Creates the UTXO set from the given block chain
fn get_utxo_set(block_chain: &BlockChain, logger: LoggerSender) -> UTXOSet {
    let _ = logger.log_wallet("Creating the UTXO set".to_string());

    let utxo_set = UTXOSet::from_blockchain(block_chain);

    let _ = logger.log_wallet("UTXO set finished successfully".to_string());
    utxo_set
}

/// Broadcast the transaction created by the user to the peers from the selected account in the wallet
///
/// ### Error
///  * `ErrorUI::FailedSignalToFront`: It will appear when the sender fails
///  * `ErrorUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
///  * `ErrorUI::ErrorFromPeer`: It will appear when a conextion with a peer fails
fn sending_transaction(
    broadcasting: &mut Broadcasting<TcpStream>,
    wallet: &MutArc<Wallet>,
    utxo_set: &MutArc<UTXOSet>,
    logger: LoggerSender,
    address_string: &str,
    amount_fee: (f64, f64),
    tx_to_front: glib::Sender<SignalToFront>,
) -> Result<(), ErrorUI> {
    let amount = amount_fee.0;
    let fee = amount_fee.1;
    let address = match Address::new(address_string) {
        Ok(address) => address,
        Err(_) => {
            let message = "Invalid address";
            let _ = logger.log_wallet(message.to_string());
            if tx_to_front
                .send(SignalToFront::ErrorInTransaction(message.to_string()))
                .is_err()
            {
                return Err(ErrorUI::FailedSignalToFront(
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
            if tx_to_front
                .send(SignalToFront::ErrorInTransaction(message.to_string()))
                .is_err()
            {
                return Err(ErrorUI::FailedSignalToFront(
                    "Failed to send error signal to front".to_string(),
                ));
            }
            return Ok(());
        }
    };

    let transaction =
        match create_transaction(&utxo_set, account, logger.clone(), &address, amount, fee) {
            Ok(transaction) => transaction,
            Err(error) => {
                if tx_to_front
                    .send(SignalToFront::ErrorInTransaction(
                        "Error creating the transaction".to_string(),
                    ))
                    .is_err()
                {
                    return Err(ErrorUI::FailedSignalToFront(
                        "Failed to send error signal to front".to_string(),
                    ));
                };
                return Err(error.into());
            }
        };
    let _ = logger.log_transaction("Sending transaction".to_string());

    get_reference(utxo_set)?.append_pending_transaction(transaction.clone());

    match broadcasting.send_transaction(transaction) {
        Ok(()) => Ok(()),
        Err(ErrorNode::WhileSendingMessage(message)) => Err(ErrorUI::ErrorFromPeer(message)),
        _ => Err(ErrorUI::ErrorFromPeer(
            "While sending transaction".to_string(),
        )),
    }
}

/// Creates a new account with the data entered by the user
///
/// ### Error
///  * `ErrorUI::FailedSignalToFront`: It will appear when the sender fails
pub fn create_account(
    wallet: MutArc<Wallet>,
    account_name: &str,
    private_key_string: &str,
    public_key_string: &str,
    tx_to_front: glib::Sender<SignalToFront>,
    logger: LoggerSender,
) -> Result<(), ErrorUI> {
    let mut wallet = get_reference(&wallet)?;

    let private_key = match PrivateKey::try_from(private_key_string) {
        Ok(private_key) => private_key,
        Err(_) => {
            let message = "Invalid private key";
            let _ = logger.log_wallet(message.to_string());
            if tx_to_front
                .send(SignalToFront::ErrorInAccountCreation(message.to_string()))
                .is_err()
            {
                return Err(ErrorUI::FailedSignalToFront(
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
            if tx_to_front
                .send(SignalToFront::ErrorInAccountCreation(message.to_string()))
                .is_err()
            {
                return Err(ErrorUI::FailedSignalToFront(
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
            if tx_to_front
                .send(SignalToFront::ErrorInAccountCreation(message))
                .is_err()
            {
                return Err(ErrorUI::FailedSignalToFront(
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
    if tx_to_front
        .send(SignalToFront::RegisterWallet(account_name.to_string()))
        .is_err()
    {
        return Err(ErrorUI::FailedSignalToFront(
            "Failed to send account created signal to front".to_string(),
        ));
    }

    Ok(())
}

/// Function that obtains and return the information of the transactions of an account
pub fn get_account_transactions_information(
    account: &Account,
    blockchain: &BlockChain,
) -> Vec<(u32, [u8; 32], i64)> {
    let mut transactions: Vec<Transaction> = Vec::new();
    let blocks = blockchain.get_all_blocks();
    for block in blocks {
        for transaction in block.transactions {
            if account.verify_transaction_ownership(&transaction) {
                transactions.push(transaction);
            }
        }
    }
    let filtered_transactions = transactions
        .iter()
        .filter_map(|transaction| {
            let timestamp = transaction.time;
            let label = match transaction.get_tx_id() {
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
        })
        .collect();
    filtered_transactions
}

/// Function that gets the information of the transactions of the selected account
/// and sends it to the front
fn give_account_transactions(
    wallet: MutArc<Wallet>,
    blockchain: MutArc<BlockChain>,
    logger: LoggerSender,
    tx_to_front: glib::Sender<SignalToFront>,
) -> Result<(), ErrorUI> {
    let wallet = get_reference(&wallet).unwrap();
    let blockchain = get_reference(&blockchain).unwrap();

    let account = match wallet.get_selected_account() {
        Some(account) => account,
        None => {
            let message = "No account selected cannot get transactions";
            let _ = logger.log_wallet(message.to_string());
            if tx_to_front
                .send(SignalToFront::ErrorInTransaction(message.to_string()))
                .is_err()
            {
                return Err(ErrorUI::FailedSignalToFront(
                    "Failed to send error signal to front".to_string(),
                ));
            }
            return Ok(());
        }
    };

    let transactions = get_account_transactions_information(account, &blockchain);
    if tx_to_front
        .send(SignalToFront::AccountTransactions(transactions))
        .is_err()
    {
        if tx_to_front
            .send(SignalToFront::ErrorInTransaction(
                "Failed to send transactions to front".to_string(),
            ))
            .is_err()
        {
            return Err(ErrorUI::FailedSignalToFront(
                "Failed to send error signal to front".to_string(),
            ));
        };
        return Err(ErrorUI::FailedSignalToFront(
            "Failed to send error signal to front".to_string(),
        ));
    }

    Ok(())
}

/// Function that handles the signals from the front end
pub fn spawn_frontend_handler(
    rx_from_front: Receiver<SignalToBack>,
    tx_to_front: glib::Sender<SignalToFront>,
    broadcasting: &mut Broadcasting<TcpStream>,
    data: (MutArc<Wallet>, MutArc<UTXOSet>, MutArc<BlockChain>),
    logger: LoggerSender,
) -> Result<(), ErrorUI> {
    let wallet: MutArc<Wallet> = data.0;
    let utxo_set: MutArc<UTXOSet> = data.1;
    let block_chain: MutArc<BlockChain> = data.2;
    for rx in rx_from_front {
        match rx {
            SignalToBack::GetAccountBalance => {
                give_account_balance(wallet.clone(), utxo_set.clone(), tx_to_front.clone())?;
            }
            SignalToBack::ChangeSelectedAccount(account_name) => {
                change_selected_account(account_name, wallet.clone(), tx_to_front.clone())?;
            }
            SignalToBack::CreateTransaction(address_string, amount, fee) => {
                sending_transaction(
                    broadcasting,
                    &wallet,
                    &utxo_set,
                    logger.clone(),
                    &address_string,
                    (amount, fee),
                    tx_to_front.clone(),
                )?;
            }
            SignalToBack::CreateAccount(name, private_key, public_key) => {
                create_account(
                    wallet.clone(),
                    &name,
                    &private_key,
                    &public_key,
                    tx_to_front.clone(),
                    logger.clone(),
                )?;
            }
            SignalToBack::GetAccountTransactions => {
                give_account_transactions(
                    wallet.clone(),
                    block_chain.clone(),
                    logger.clone(),
                    tx_to_front.clone(),
                )?;
            }
            SignalToBack::ExitProgram => {
                break;
            }
        }
    }
    Ok(())
}

/// Function that changes the selected account of the address
pub fn change_selected_account(
    account_name: String,
    wallet: MutArc<Wallet>,
    tx_to_front: glib::Sender<SignalToFront>,
) -> Result<(), ErrorUI> {
    let mut wallet_reference = get_reference(&wallet)?;

    let account_to_select = match wallet_reference.get_account_with_name(&account_name) {
        Some(account) => account.clone(),
        None => return Err(ErrorUI::ErrorReading("Account does not exist".to_string())),
    };

    wallet_reference.change_account(account_to_select);

    if tx_to_front.send(SignalToFront::Update).is_err() {
        return Err(ErrorUI::FailedSignalToFront(
            "Failed to send update signal to front".to_string(),
        ));
    }

    Ok(())
}

/// Function that obtains the balance of the selected account and sends it to the front
pub fn give_account_balance(
    wallet: MutArc<Wallet>,
    utxo_set: MutArc<UTXOSet>,
    tx_to_front: glib::Sender<SignalToFront>,
) -> Result<(), ErrorUI> {
    let wallet_reference = get_reference(&wallet)?;
    let utxo_set_reference = get_reference(&utxo_set)?;

    let account_to_check = match wallet_reference.get_selected_account() {
        Some(account) => account,
        None => return Err(ErrorUI::ErrorReading("No account selected".to_string())),
    };
    let balance = utxo_set_reference.get_balance_in_tbtc(&account_to_check.address);
    let pending = utxo_set_reference.get_pending_in_tbtc(&account_to_check.address);
    if tx_to_front
        .send(SignalToFront::LoadAvailableBalance((balance, pending)))
        .is_err()
    {
        return Err(ErrorUI::FailedSignalToFront(
            "Failed to send available balance to front".to_string(),
        ));
    }
    Ok(())
}

/// Broadcasting blocks and transactions from and to the given peers
fn broadcasting(
    rx_from_front: Receiver<SignalToBack>,
    tx_to_front: glib::Sender<SignalToFront>,
    peer_streams: Vec<TcpStream>,
    data: (MutArc<Wallet>, MutArc<UTXOSet>, MutArc<BlockChain>),
    connection_config: ConnectionConfig,
    logger: LoggerSender,
    notifier: NotificationSender,
) -> Result<(), ErrorExecution> {
    let wallet: Arc<Mutex<Wallet>> = data.0;
    let utxo_set: Arc<Mutex<UTXOSet>> = data.1;
    let block_chain: Arc<Mutex<BlockChain>> = data.2;
    let (sender_response, receiver_response) = mpsc::channel::<MessageResponse>();

    let handle = handle_peers(
        receiver_response,
        wallet.clone(),
        utxo_set.clone(),
        block_chain.clone(),
        logger.clone(),
        notifier.clone(),
    );

    let mut broadcasting = get_broadcasting(
        peer_streams,
        sender_response,
        connection_config,
        logger.clone(),
    );

    spawn_frontend_handler(
        rx_from_front,
        tx_to_front,
        &mut broadcasting,
        (wallet, utxo_set, block_chain),
        logger,
    )?;

    broadcasting.destroy()?;

    match handle.join() {
        Ok(_) => Ok(()),
        Err(_) => Err(ErrorUI::ErrorFromPeer("Failed to remove notifications".to_string()).into()),
    }
}

/// Function that spawns the notification handler thread.
/// It return the notification sender and a handle on the
/// thread.
pub fn spawn_notification_handler(
    tx_to_front: glib::Sender<SignalToFront>,
) -> (NotificationSender, thread::JoinHandle<()>) {
    let (notification_sender, notification_receiver) = mpsc::channel::<Notification>();

    let handle = thread::spawn(move || {
        for notification in notification_receiver {
            match notification {
                Notification::AttemptingHandshakeWithPeer(peer) => {
                    println!("Attempting handshake with peer {}", peer)
                }
                Notification::SuccessfulHandshakeWithPeer(peer) => {
                    println!("Successful handshake with peer {}", peer)
                }
                Notification::FailedHandshakeWithPeer(peer) => {
                    println!("Failed handshake with peer {}", peer)
                }
                Notification::TransactionOfAccountReceived(accounts, _transaction) => {
                    if tx_to_front.send(SignalToFront::Update).is_err()
                        || tx_to_front
                            .send(SignalToFront::TransactionOfAccountReceived(
                                accounts[0].account_name.clone(),
                            ))
                            .is_err()
                    {
                        println!("Failed to send update signal to front");
                    }
                }
                Notification::TransactionOfAccountInNewBlock(_transaction) => {
                    if tx_to_front
                        .send(SignalToFront::BlockWithUnconfirmedTransactionReceived)
                        .is_err()
                    {
                        println!("Error sending signal to front")
                    }
                }
                Notification::NewBlockAddedToTheBlockchain(_block) => {
                    if tx_to_front.send(SignalToFront::Update).is_err() {
                        println!("Error sending signal to front");
                    }
                }
            }
        }
    });

    (notification_sender, handle)
}

/// Function that performs the backend execution
pub fn backend_execution(
    mode_config: ModeConfig,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    load_system: LoadSystem,
    logger: LoggerSender,
    tx_to_front: glib::Sender<SignalToFront>,
    rx_from_front: mpsc::Receiver<SignalToBack>,
) -> Result<SaveSystem, ErrorExecution> {
    let mut load_system = load_system;

    let (notification_sender, _notification_handler) =
        spawn_notification_handler(tx_to_front.clone());

    let potential_peers = match mode_config {
        ModeConfig::Server(server_config) => get_potential_peers(server_config, logger.clone())?,
        ModeConfig::Client(client_config) => vec![SocketAddr::new(
            IpAddr::V4(client_config.address),
            client_config.port,
        )],
    };

    let peer_streams = handshake::connect_to_peers(
        potential_peers,
        connection_config.clone(),
        logger.clone(),
        notification_sender.clone(),
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
        tx_to_front
            .send(SignalToFront::RegisterWallet(account.account_name.clone()))
            .unwrap();
    }

    tx_to_front
        .send(SignalToFront::NotifyBlockchainIsReady)
        .unwrap();

    let wallet = Arc::new(Mutex::new(wallet));
    let utxo_set = Arc::new(Mutex::new(get_utxo_set(&block_chain, logger.clone())));
    let block_chain = Arc::new(Mutex::new(block_chain));

    broadcasting(
        rx_from_front,
        tx_to_front,
        peer_streams,
        (wallet.clone(), utxo_set, block_chain.clone()),
        connection_config,
        logger.clone(),
        notification_sender,
    )?;

    Ok(SaveSystem::new(
        get_inner(block_chain)?,
        get_inner(wallet)?,
        logger,
    ))
}

/// Function that spawns the backend handler thread
pub fn spawn_backend_handler(
    mode_config: ModeConfig,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    save_config: SaveConfig,
    logger: LoggerSender,
    tx_to_front: glib::Sender<SignalToFront>,
    rx_from_front: mpsc::Receiver<SignalToBack>,
) -> thread::JoinHandle<Result<SaveSystem, ErrorExecution>> {
    thread::spawn(move || {
        let load_system = LoadSystem::new(save_config.clone(), logger.clone());
        backend_execution(
            mode_config,
            connection_config,
            download_config,
            load_system,
            logger,
            tx_to_front,
            rx_from_front,
        )
    })
}
