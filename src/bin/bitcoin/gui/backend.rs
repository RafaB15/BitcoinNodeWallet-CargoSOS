use super::signal_to_back::SignalToBack;

use crate::{
    error_execution::ErrorExecution,
    process::{
        broadcasting::{add_peers, handle_peers},
        connection::get_potential_peers,
        download, handshake,
        load_system::LoadSystem,
        reference::{self, get_inner, get_reference, MutArc},
        save_system::SaveSystem,
        transaction,
    },
    ui::{account, error_ui::ErrorUI},
};

use cargosos_bitcoin::{
    block_structure::block,
    configurations::{
        connection_config::ConnectionConfig, download_config::DownloadConfig,
        mode_config::ModeConfig, save_config::SaveConfig,
    },
    node_structure::connection_id::ConnectionId,
};

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, utxo_set::UTXOSet},
    connections::error_connection::ErrorConnection,
    logs::logger_sender::LoggerSender,
    node_structure::{broadcasting::Broadcasting, message_response::MessageResponse},
    notifications::{notification::Notification, notifier::Notifier},
    wallet_structure::{
        address::Address, private_key::PrivateKey, public_key::PublicKey, wallet::Wallet,
    },
};

use std::{
    net::{IpAddr, SocketAddr, TcpStream},
    sync::mpsc::{channel, Receiver},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

/// Creates a new account with the data entered by the user
///
/// ### Error
///  * `ErrorUI::FailedSignalToFront`: It will appear when the sender fails
pub fn create_account<N: Notifier>(
    wallet: MutArc<Wallet>,
    account_name: &str,
    private_key_string: &str,
    public_key_string: &str,
    notifier: N,
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

    let mut wallet = reference::get_reference(&wallet)?;
    account::create_account(
        &mut wallet,
        account_name,
        private_key,
        public_key,
        notifier.clone(),
    )
}

/// Function that handles the signals from the front end
pub fn spawn_frontend_handler<N: Notifier>(
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
        let mut wallet_reference = get_reference(&wallet)?;
        let mut utxo_set_reference = get_reference(&utxo_set)?;
        let block_chain_reference = get_reference(&block_chain)?;

        match rx {
            SignalToBack::GetAccountBalance => {
                account::give_account_balance(
                    &wallet_reference,
                    &utxo_set_reference,
                    notifier.clone(),
                );
            }
            SignalToBack::ChangeSelectedAccount(account_name) => {
                account::change_selected_account(
                    account_name,
                    &mut wallet_reference,
                    notifier.clone(),
                )?;
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
                    &wallet_reference,
                    &mut utxo_set_reference,
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
                )?;
            }
            SignalToBack::GetAccountTransactions => {
                account::give_account_transactions(
                    &wallet_reference,
                    &block_chain_reference,
                    notifier.clone(),
                    logger.clone(),
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
fn broadcasting<N: Notifier + 'static>(
    rx_from_front: Receiver<SignalToBack>,
    connections: Vec<(TcpStream, ConnectionId)>,
    data: (MutArc<Wallet>, MutArc<UTXOSet>, MutArc<BlockChain>),
    connection_config: ConnectionConfig,
    notifier: N,
    logger: LoggerSender,
) -> Result<(), ErrorExecution> {
    let wallet: Arc<Mutex<Wallet>> = data.0;
    let utxo_set: Arc<Mutex<UTXOSet>> = data.1;
    let block_chain: Arc<Mutex<BlockChain>> = data.2;

    let (sender_response, receiver_response) = channel::<MessageResponse>();

    for (stream, _) in connections.iter() {
        if stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .is_err()
        {
            return Err(ErrorConnection::ErrorCannotSetStreamProperties.into());
        };
    }

    let handle = handle_peers(
        receiver_response,
        wallet.clone(),
        utxo_set.clone(),
        block_chain.clone(),
        notifier.clone(),
        logger.clone(),
    );

    let mut broadcasting = Broadcasting::<TcpStream>::new(logger.clone());

    add_peers(
        &mut broadcasting,
        connections,
        sender_response,
        block_chain.clone(),
        connection_config.magic_numbers,
        notifier.clone(),
        logger.clone(),
    );

    spawn_frontend_handler(
        rx_from_front,
        &mut broadcasting,
        (wallet, utxo_set, block_chain),
        notifier.clone(),
        logger,
    )?;

    broadcasting.destroy(notifier)?;

    match handle.join() {
        Ok(_) => Ok(()),
        Err(_) => Err(ErrorUI::ErrorFromPeer("Failed to remove notifications".to_string()).into()),
    }
}

/// Function that performs the backend execution
pub fn backend_execution<N: Notifier + 'static>(
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

    let connections = handshake::connect_to_peers(
        potential_peers,
        connection_config.clone(),
        notifier.clone(),
        logger.clone(),
    );

    let mut block_chain = load_system.get_block_chain()?;

    let connections = download::update_block_chain(
        connections,
        &mut block_chain,
        connection_config.clone(),
        download_config,
        notifier.clone(),
        logger.clone(),
    )?;

    let wallet = load_system.get_wallet()?;
    for account in wallet.get_accounts().iter() {
        notifier.notify(Notification::RegisterWalletAccount(account.clone()));
    }

    notifier.notify(Notification::NotifyBlockchainIsReady);

    let wallet = Arc::new(Mutex::new(wallet));
    let utxo_set = Arc::new(Mutex::new(download::get_utxo_set(
        &block_chain,
        logger.clone(),
    )));
    let block_chain = Arc::new(Mutex::new(block_chain));

    broadcasting(
        rx_from_front,
        connections,
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
pub fn spawn_backend_handler<N: Notifier + 'static>(
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
