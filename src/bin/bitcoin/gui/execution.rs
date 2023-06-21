use super::{
    error_gui::ErrorGUI,
    signal_to_front::SignalToFront,
    signal_to_back::SignalToBack,
};

use gtk::glib::subclass::Signal;
use gtk::{prelude::*, glib::Object, Button, Entry, Application, Builder, Window, ComboBoxText, TreeView, TreeViewColumn, CellRendererText, TreeSelection, Image, Label};
use gtk::glib;
use std::thread;

use crate::{
    error_execution::ErrorExecution,
    process::{
        download, handshake,
        save_system::SaveSystem, 
        load_system::LoadSystem, 
    }
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
    },
    connections::ibd_methods::IBDMethod,
    wallet_structure::address::Address,
};

use std::{
    net::{SocketAddr, TcpStream},
    sync::{Arc, Mutex},
};

use cargosos_bitcoin::wallet_structure::{private_key, public_key};

use std::sync::mpsc;

pub trait VecOwnExt {
    fn search_by_name(&self, name: &str) -> Object;
    fn search_window_named(&self, name: &str) -> Window;
    fn search_button_named(&self, name: &str) -> Button;
    fn search_entry_named(&self, name: &str) -> Entry;
    fn search_combo_box_named(&self, name: &str) -> ComboBoxText;
    fn search_tree_view_named(&self, name: &str) -> TreeView;
    fn search_tree_view_column_named(&self, name: &str) -> TreeViewColumn;
    fn search_cell_renderer_named(&self, name: &str) -> CellRendererText;
    fn search_tree_view_selection_named(&self, name: &str) -> TreeSelection;
    fn search_image_named(&self, name: &str) -> Image;
    fn search_label_named(&self, name: &str) -> Label;
}

pub trait ObjectOwnExt {
    fn is_named(&self, name: &str) -> bool;
}

impl ObjectOwnExt for Object {
    fn is_named(&self, name: &str) -> bool {
        self.property_value("name").get::<String>().unwrap() == *name
    }
}

impl VecOwnExt for Vec<Object> {
    fn search_by_name(&self, name: &str) -> Object {
        let found = self.iter().find(|&object| object.is_named(name));
        if let Some(found) = found {
            (*found).clone()
        } else {
            (*found.unwrap()).clone()
        }
    }
    fn search_button_named(&self, name: &str) -> Button {
        self.search_by_name(name)
            .downcast_ref::<gtk::Button>()
            .unwrap()
            .clone()
    }
    fn search_entry_named(&self, name: &str) -> Entry {
        self.search_by_name(name)
            .downcast_ref::<gtk::Entry>()
            .unwrap()
            .clone()
    }
    fn search_window_named(&self, name: &str) -> Window {
        self.search_by_name(name)
            .downcast_ref::<gtk::Window>()
            .unwrap()
            .clone()
    }
    fn search_combo_box_named(&self, name: &str) -> ComboBoxText {
        self.search_by_name(name)
            .downcast_ref::<gtk::ComboBoxText>()
            .unwrap()
            .clone()
    }
    fn search_tree_view_named(&self, name: &str) -> TreeView {
        self.search_by_name(name)
            .downcast_ref::<gtk::TreeView>()
            .unwrap()
            .clone()
    }
    fn search_tree_view_column_named(&self, name: &str) -> TreeViewColumn {
        self.search_by_name(name)
            .downcast_ref::<gtk::TreeViewColumn>()
            .unwrap()
            .clone()
    }
    fn search_cell_renderer_named(&self, name: &str) -> CellRendererText {
        self.search_by_name(name)
            .downcast_ref::<gtk::CellRendererText>()
            .unwrap()
            .clone()
    }
    fn search_tree_view_selection_named(&self, name: &str) -> TreeSelection {
        self.search_by_name(name)
            .downcast_ref::<gtk::TreeSelection>()
            .unwrap()
            .clone()
    }

    fn search_image_named(&self, name: &str) -> Image {
        self.search_by_name(name)
            .downcast_ref::<gtk::Image>()
            .unwrap()
            .clone()
    }

    fn search_label_named(&self, name: &str) -> Label {
        self.search_by_name(name)
            .downcast_ref::<gtk::Label>()
            .unwrap()
            .clone()
    }

}

fn login_main_window(application: &gtk::Application, objects: &Vec<Object>) {

    let window = objects.search_window_named("MainWindow");
    window.set_application(Some(application));

    let combo_box = objects.search_combo_box_named("WalletsComboBox");

    let account_registration_button = objects.search_button_named("AccountRegistrationButton");
    let cloned_objects = objects.clone();
    
    account_registration_button.connect_clicked(move |_| {
        let account_registration_window = cloned_objects.search_window_named("AccountRegistrationWindow");
        account_registration_window.set_visible(true);
    });

    window.show_all();
}

fn login_registration_window(objects: &Vec<Object>) {
    let account_registration_window = objects.search_window_named("AccountRegistrationWindow");
    let cloned_objects = objects.clone();
    let save_wallet_button = objects.search_button_named("SaveWalletButton");
    save_wallet_button.connect_clicked(move |_| {
        account_registration_window.set_visible(false);
        
        let private_key_entry = cloned_objects.search_entry_named("PrivateKeyEntry");
        let public_key_entry = cloned_objects.search_entry_named("PublicKeyEntry");
        let address_entry = cloned_objects.search_entry_named("AddressEntry");
        let name_entry = cloned_objects.search_entry_named("NameEntry");

        let combo_box = cloned_objects.search_combo_box_named("WalletsComboBox");
        combo_box.append_text(name_entry.text().as_str());

        println!("{:?} {:?} {:?} {:?}", private_key_entry.text(), public_key_entry.text(), address_entry.text(), name_entry.text());

        private_key_entry.set_text("");
        public_key_entry.set_text("");
        address_entry.set_text("");
        name_entry.set_text("");            
    });
}

fn login_combo_box(objects: &Vec<Object>, tx_to_back: mpsc::Sender<SignalToBack>) {
    let combo_box = objects.search_combo_box_named("WalletsComboBox");
    let cloned_objects = objects.clone();
    combo_box.connect_changed(move |_| {
        let combo_box_cloned = cloned_objects.search_combo_box_named("WalletsComboBox");
        let selected_wallet = combo_box_cloned.active_text().unwrap();
        let _ = tx_to_back.send(SignalToBack::GetAccountBalance(selected_wallet.to_string()));
        println!("{}", selected_wallet);
    });
}

fn spawn_backend_handler(
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    save_config: SaveConfig,
    logger: LoggerSender,
    tx_to_front: glib::Sender<SignalToFront>,
    rx_from_front: mpsc::Receiver<SignalToBack>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let load_system = LoadSystem::new(
            save_config.clone(),
            logger.clone(),
        );
        let _ = backend_initialization(connection_config, download_config, load_system, logger, tx_to_front, rx_from_front);
    })
}

fn spawn_local_handler(objects: &Vec<Object>, rx_from_back: glib::Receiver<SignalToFront>) {
    let cloned_objects = objects.clone();
    rx_from_back.attach(None, move |signal| {
        match signal {
            SignalToFront::RegisterWallet(wallet_name) => {
                let combo_box = cloned_objects.search_combo_box_named("WalletsComboBox");
                combo_box.append_text(&wallet_name);
                println!("Registering wallet: {:?}", wallet_name);
            },
            SignalToFront::LoadAvailableBalance(balance) => {
                let balance_label = cloned_objects.search_label_named("AvailableBalanceLabel");
                balance_label.set_text(balance.to_string().as_str());
            },
            SignalToFront::LoadBlockChain => {
                let signal_blockchain_not_ready = cloned_objects.search_image_named("BlockchainNotReadySymbol");
                signal_blockchain_not_ready.set_visible(false);
            }
            /*
            SignalToFront::LoadRecentTransactions(transactions) => {
                for transaction in transactions {
                    let transactions_list_box = cloned_objects.search_by_name("TransactionsListBox");
                    let transaction_label = gtk::Label::new(Some(&transaction));
                    //transactions_list_box.append_text(&transaction_label);
                }
            }*/
            //recibir la blockchain -> integrarla al load bar
            //obtener transacciones de bloques ->  cargarlas al tree view
        }
        glib::Continue(true)
    });
} 


fn build_ui(
    application: &gtk::Application, 
    glade_src: &str,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    save_config: SaveConfig,
    logger: LoggerSender,
) {
    let builder: Builder = Builder::from_string(glade_src);

    let objects = builder.objects();

    let (tx_to_back, rx_from_front) = mpsc::channel::<SignalToBack>();
    let (tx_to_front, rx_from_back) = glib::MainContext::channel::<SignalToFront>(glib::PRIORITY_DEFAULT);

    spawn_backend_handler(connection_config, download_config, save_config, logger, tx_to_front, rx_from_front);

    spawn_local_handler(&objects, rx_from_back);

    login_main_window(application, &objects);

    login_registration_window(&objects);

    login_combo_box(&objects, tx_to_back.clone());
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
) -> Result<(), ErrorExecution> {
    logger.log_connection("Getting block chain".to_string())?;

    match connection_config.ibd_method {
        IBDMethod::HeaderFirst => {
            download::headers_first(
                peer_streams,
                block_chain,
                connection_config,
                download_config,
                logger,
            )?;
        }
        IBDMethod::BlocksFirst => {
            download::blocks_first::<TcpStream>();
        }
    }

    Ok(())
}

pub fn backend_initialization(
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    load_system: LoadSystem,
    logger: LoggerSender,
    tx_to_front: glib::Sender<SignalToFront>,
    rx_from_front: mpsc::Receiver<SignalToBack>,
) -> Result<(), ErrorExecution> {


    let mut load_system = load_system;

    let potential_peers = get_potential_peers(connection_config.clone(), logger.clone())?;

    let mut block_chain = load_system.get_block_chain()?;

    let peer_streams = handshake::connect_to_peers(
        potential_peers,
        connection_config.clone(),
        logger.clone(),
    );

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

    let utxo_set = UTXOSet::from_blockchain(&block_chain);

    for rx in rx_from_front {
        match rx {
            SignalToBack::GetAccountBalance(account_name) => {
                let balance = utxo_set.get_balance_in_tbtc(&wallet.get_account_with_name(&account_name).unwrap().address);
                tx_to_front.send(SignalToFront::LoadAvailableBalance(balance)).unwrap();
            },
            _ => {}
        }
    }


    //let block_chain = Arc::new(Mutex::new(block_chain));

    //tx_to_front.send(SignalToFront::LoadBlockChain).unwrap();

    //tx_to_front.send(SignalToFront::LoadBlockChain(load_system.get_block_chain()?)).unwrap();

    println!("Wallet: {:?}", wallet);
    Ok(())
}


pub fn program_execution(
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    save_config: SaveConfig,
    logger: LoggerSender,
) -> Result<SaveSystem, ErrorGUI> {
    

    //let result = backend_initialization_handler.join();

    let glade_src = include_str!("WindowNotebook.glade");

    let application = Application::builder()
        .build();

    application.connect_activate(move |app| build_ui(app, glade_src, connection_config.clone(), download_config.clone(), save_config.clone(), logger.clone()));
    let vector: Vec<String> = Vec::new();
    application.run_with_args(&vector);
   //gtk_main_quit();

    Err(ErrorGUI::FailedToInitializeGTK)
}
