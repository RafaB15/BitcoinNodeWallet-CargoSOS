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


use cargosos_bitcoin::{configurations::{
    connection_config::ConnectionConfig, download_config::DownloadConfig,
    save_config::SaveConfig,
}, block_structure::utxo_set};

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
    wallet_structure::wallet::Wallet,
    node_structure::{broadcasting::Broadcasting, message_response::MessageResponse}
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
    transactions: &mut Vec<Transaction>,
    logger: LoggerSender,
) -> Result<(), ErrorGUI> {
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
) -> Result<(), ErrorGUI> {
    transactions.retain(|transaction| {
        if block.transactions.contains(transaction) {
            println!("{transaction} has been added to the blockchain");
            let _ = logger.log_wallet(format!(
                "Removing transaction {transaction} from list of transaction seen so far"
            ));
            return false;
        }
        true
    });

    let mut utxo_set = get_reference(&utxo_set)?;
    let mut block_chain = get_reference(&block_chain)?;

    utxo_set.update_utxo_with_block(&block);

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
    block_chain: MutArc<BlockChain>,
    logger: LoggerSender,
) -> JoinHandle<Result<(), ErrorGUI>> {
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
            if tx_to_front.send(SignalToFront::Update).is_err() {
                return Err(ErrorGUI::FailedSignalToFront(
                    "Failed to send update signal to front".to_string(),
                ));
            }
        }

        Ok(())
    })
}

pub fn spawn_frontend_handler(
    rx_from_front: Receiver<SignalToBack>,
    tx_to_front: glib::Sender<SignalToFront>,
    broadcasting: &mut Broadcasting<TcpStream>,
    wallet: MutArc<Wallet>,
    utxo_set: MutArc<UTXOSet>,
    block_chain: MutArc<BlockChain>,
    logger: LoggerSender,
) -> Result<(), ErrorGUI> {
    for rx in rx_from_front {
        match rx {
            SignalToBack::GetAccountBalance => {
                give_account_balance(wallet.clone(), utxo_set.clone(), tx_to_front.clone())?;
            },
            SignalToBack::ChangeSelectedAccount(account_name) => {
                change_selected_account(account_name, wallet.clone(), tx_to_front.clone())?;
            },
            SignalToBack::ExitProgram => {
                break;
            },
            _ => {},
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

    wallet_reference.selected_account = Some(account_to_select);

    if tx_to_front.send(SignalToFront::Update).is_err() {
        return Err(ErrorGUI::FailedSignalToFront(
            "Failed to send update signal to front".to_string(),
        ));
    }

    Ok(())
}

pub fn give_account_balance(
    wallet: MutArc<Wallet>,
    utxo_set: MutArc<UTXOSet>,
    tx_to_front: glib::Sender<SignalToFront>,
) -> Result<(), ErrorGUI> {
    let wallet_reference = get_reference(&wallet)?;
    let utxo_set_reference = get_reference(&utxo_set)?;

    let account_to_check = match wallet_reference.selected_account.clone() {
        Some(account) => account,
        None => {
            return Err(ErrorGUI::ErrorReading(
                "No account selected".to_string(),
            ))
        }
    };
    let balance = utxo_set_reference.get_balance_in_tbtc(&account_to_check.address);
    let pending = 0.0;
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

    let handle = handle_peers(
        tx_to_front.clone(),
        receiver_response,
        wallet.clone(),
        utxo_set.clone(),
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
        block_chain.clone(),
        logger.clone(),
    )?;

    broadcasting.destroy()?;

    println!("HOliwis");

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

    let mut wallet = load_system.get_wallet()?;
    for account in wallet.accounts.iter() {
        tx_to_front.send(SignalToFront::RegisterWallet(account.account_name.clone())).unwrap();
    }

    tx_to_front.send(SignalToFront::LoadBlockChain).unwrap();

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