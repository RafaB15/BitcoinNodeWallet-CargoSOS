use super::{signal_to_back::SignalToBack};


use crate::{
    error_execution::ErrorExecution,
    process::{
        broadcasting::{get_broadcasting, handle_peers},
        download, handshake,
        load_system::LoadSystem,
        reference::{get_inner, MutArc},
        save_system::SaveSystem,
        connection::get_potential_peers,
        transaction,
    },
    ui::{
        account,
        error_ui::ErrorUI,
    }, 
};

use cargosos_bitcoin::configurations::{
    connection_config::ConnectionConfig, download_config::DownloadConfig, mode_config::ModeConfig,
    save_config::SaveConfig,
};

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, utxo_set::UTXOSet},
    logs::logger_sender::LoggerSender,
    node_structure::{
        broadcasting::Broadcasting, message_response::MessageResponse,
    },
    notifications::{
        notification::Notification,
        notifier::Notifier,
    },
    wallet_structure::{
        private_key::PrivateKey, public_key::PublicKey,
        wallet::Wallet, address::Address,
    },
};

use std::{
    net::{IpAddr, SocketAddr, TcpStream},
    sync::mpsc::{Receiver, channel},
    sync::{Arc, Mutex},
    thread,
};

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

    account::create_account(
        wallet.clone(),
        account_name,
        private_key,
        public_key,
        notifier.clone(),
    )
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
                account::give_account_balance(wallet.clone(), utxo_set.clone(), notifier.clone())?;
            }
            SignalToBack::ChangeSelectedAccount(account_name) => {
                account::change_selected_account(account_name, wallet.clone(), notifier.clone())?;
            }
            SignalToBack::CreateTransaction(address_string, amount, fee) => {
                let address = match Address::new(&address_string) {
                    Ok(address) => address,
                    Err(_) => {
                        notifier.notify(Notification::InvalidAddressEnter);
                        return Ok(());
                    }
                };
                
                transaction::sending_transaction(
                    broadcasting,
                    &wallet,
                    &utxo_set,
                    address,
                    (amount, fee),
                    notifier.clone(),
                    logger.clone(),
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
                account::give_account_transactions(
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

    let peer_streams = download::get_block_chain(
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
    let utxo_set = Arc::new(Mutex::new(download::get_utxo_set(&block_chain, logger.clone())));
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
