use super::{signal_to_back::SignalToBack};


use crate::{
    error_execution::ErrorExecution,
    process::{
        broadcasting::{create_transaction, get_broadcasting, handle_peers},
        download, handshake,
        load_system::LoadSystem,
        reference::{get_inner, get_reference, MutArc},
        save_system::SaveSystem,
    },
    ui::{
        account::{give_account_balance, give_account_transactions},
        error_ui::ErrorUI,
    }, 
};

use cargosos_bitcoin::configurations::{
    connection_config::ConnectionConfig, download_config::DownloadConfig, mode_config::ModeConfig,
    save_config::SaveConfig, server_config::ServerConfig,
};

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, utxo_set::UTXOSet},
    connections::ibd_methods::IBDMethod,
    logs::logger_sender::LoggerSender,
    node_structure::{
        broadcasting::Broadcasting, error_node::ErrorNode, message_response::MessageResponse,
    },
    notifications::{
        notification::Notification,
        notifier::Notifier,
    },
    wallet_structure::{
        account::Account, address::Address, private_key::PrivateKey, public_key::PublicKey,
        wallet::Wallet,
    },
};

use std::{
    net::{IpAddr, SocketAddr, TcpStream},
    sync::mpsc::{Receiver, channel},
    sync::{Arc, Mutex},
    thread,
};

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
fn sending_transaction<N : Notifier>(
    broadcasting: &mut Broadcasting<TcpStream>,
    wallet: &MutArc<Wallet>,
    utxo_set: &MutArc<UTXOSet>,
    logger: LoggerSender,
    address_string: &str,
    amount_fee: (f64, f64),
    notifier: N,
) -> Result<(), ErrorUI> {
    let amount = amount_fee.0;
    let fee = amount_fee.1;
    let address = match Address::new(address_string) {
        Ok(address) => address,
        Err(_) => {
            notifier.notify(Notification::InvalidAddressEnter);
            return Ok(());
        }
    };

    let wallet = get_reference(wallet)?;
    let account = match wallet.get_selected_account() {
        Some(account) => account,
        None => {
            let _ = logger.log_wallet("No account selected cannot send transaction".to_string());
            notifier.notify(Notification::AccountNotSelected);
            return Ok(());
        }
    };

    let transaction =
        match create_transaction(&utxo_set, account, logger.clone(), &address, amount, fee) {
            Ok(transaction) => transaction,
            Err(error) => {
                notifier.notify(Notification::NotEnoughFunds);
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
pub fn create_account<N : Notifier>(
    wallet: MutArc<Wallet>,
    account_name: &str,
    private_key_string: &str,
    public_key_string: &str,
    notifier: N,
    logger: LoggerSender,
) -> Result<(), ErrorUI> {
    let mut wallet = get_reference(&wallet)?;

    let private_key = match PrivateKey::try_from(private_key_string) {
        Ok(private_key) => private_key,
        Err(_) => {
            notifier.notify(Notification::InvalidPrivateKeyEnter);
            return Ok(());
        }
    };

    let public_key = match PublicKey::try_from(public_key_string.to_string()) {
        Ok(public_key) => public_key,
        Err(_) => {
            notifier.notify(Notification::InvalidPublicKeyEnter);
            return Ok(());
        }
    };

    let account = match Account::from_keys(
        account_name,
        private_key,
        public_key,
    ) {
        Ok(account) => account,
        _ => {
            notifier.notify(Notification::AccountCreationFail);
            return Ok(());
        }
    };

    wallet.add_account(account.clone());
    notifier.notify(Notification::RegisterWalletAccount(account));

    Ok(())
}

/// Function that handles the signals from the front end
pub fn spawn_frontend_handler<N : Notifier>(
    rx_from_front: Receiver<SignalToBack>,
    broadcasting: &mut Broadcasting<TcpStream>,
    data: (MutArc<Wallet>, MutArc<UTXOSet>, MutArc<BlockChain>),
    notifier: N,
    logger: LoggerSender,
) -> Result<(), ErrorUI> {
    let wallet: MutArc<Wallet> = data.0;
    let utxo_set: MutArc<UTXOSet> = data.1;
    let block_chain: MutArc<BlockChain> = data.2;
    for rx in rx_from_front {
        match rx {
            SignalToBack::GetAccountBalance => {
                give_account_balance(wallet.clone(), utxo_set.clone(), notifier.clone())?;
            }
            SignalToBack::ChangeSelectedAccount(account_name) => {
                change_selected_account(account_name, wallet.clone(), notifier.clone())?;
            }
            SignalToBack::CreateTransaction(address_string, amount, fee) => {
                sending_transaction(
                    broadcasting,
                    &wallet,
                    &utxo_set,
                    logger.clone(),
                    &address_string,
                    (amount, fee),
                    notifier.clone(),
                )?;
            }
            SignalToBack::CreateAccount(name, private_key, public_key) => {
                create_account(
                    wallet.clone(),
                    &name,
                    &private_key,
                    &public_key,
                    notifier.clone(),
                    logger.clone(),
                )?;
            }
            SignalToBack::GetAccountTransactions => {
                give_account_transactions(
                    wallet.clone(),
                    block_chain.clone(),
                    logger.clone(),
                    notifier.clone(),
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
pub fn change_selected_account<N : Notifier>(
    account_name: String,
    wallet: MutArc<Wallet>,
    notifier: N,
) -> Result<(), ErrorUI> {
    let mut wallet_reference = get_reference(&wallet)?;

    let account_to_select = match wallet_reference.get_account_with_name(&account_name) {
        Some(account) => account.clone(),
        None => return Err(ErrorUI::ErrorReading("Account does not exist".to_string())),
    };

    wallet_reference.change_account(account_to_select.clone());

    notifier.notify(Notification::UpdatedSelectedAccount(account_to_select));

    Ok(())
}

/// Broadcasting blocks and transactions from and to the given peers
fn broadcasting<N : Notifier>(
    rx_from_front: Receiver<SignalToBack>,
    peer_streams: Vec<TcpStream>,
    data: (MutArc<Wallet>, MutArc<UTXOSet>, MutArc<BlockChain>),
    connection_config: ConnectionConfig,
    notifier: N,
    logger: LoggerSender,
) -> Result<(), ErrorExecution> {
    let wallet: Arc<Mutex<Wallet>> = data.0;
    let utxo_set: Arc<Mutex<UTXOSet>> = data.1;
    let block_chain: Arc<Mutex<BlockChain>> = data.2;
    let (sender_response, receiver_response) = channel::<MessageResponse>();

    let handle = handle_peers(
        receiver_response,
        wallet.clone(),
        utxo_set.clone(),
        block_chain.clone(),
        notifier.clone(),
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
        &mut broadcasting,
        (wallet, utxo_set, block_chain),
        notifier,
        logger,
    )?;

    broadcasting.destroy()?;

    match handle.join() {
        Ok(_) => Ok(()),
        Err(_) => Err(ErrorUI::ErrorFromPeer("Failed to remove notifications".to_string()).into()),
    }
}

/// Function that performs the backend execution
pub fn backend_execution<N : Notifier>(
    mode_config: ModeConfig,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    load_system: LoadSystem,
    rx_from_front: Receiver<SignalToBack>,
    notifier: N,
    logger: LoggerSender,
) -> Result<SaveSystem, ErrorExecution> {
    let mut load_system = load_system;

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
        notifier.clone(),
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
        notifier.notify(Notification::RegisterWalletAccount(account.clone()));
    }

    notifier.notify(Notification::NotifyBlockchainIsReady);

    let wallet = Arc::new(Mutex::new(wallet));
    let utxo_set = Arc::new(Mutex::new(get_utxo_set(&block_chain, logger.clone())));
    let block_chain = Arc::new(Mutex::new(block_chain));

    broadcasting(
        rx_from_front,
        peer_streams,
        (wallet.clone(), utxo_set, block_chain.clone()),
        connection_config,
        notifier,
        logger.clone(),
    )?;

    Ok(SaveSystem::new(
        get_inner(block_chain)?,
        get_inner(wallet)?,
        logger,
    ))
}

/// Function that spawns the backend handler thread
pub fn spawn_backend_handler<N : Notifier>(
    mode_config: ModeConfig,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    save_config: SaveConfig,
    rx_from_front: Receiver<SignalToBack>,
    notifier: N,
    logger: LoggerSender,
) -> thread::JoinHandle<Result<SaveSystem, ErrorExecution>> {
    thread::spawn(move || {
        let load_system = LoadSystem::new(save_config.clone(), logger.clone());
        backend_execution(
            mode_config,
            connection_config,
            download_config,
            load_system,
            rx_from_front,
            notifier,
            logger,
        )
    })
}
