use super::error_gui::ErrorGUI;

use gtk::{prelude::*, glib::Object, Button, Entry, Application, Builder, Window, ComboBoxText};
use gtk::glib;
use std::thread;

use crate::{
    error_execution::ErrorExecution,
    process::{
        download, handshake, account,
        save_system::SaveSystem, 
        load_system::LoadSystem, 
    },
};

use cargosos_bitcoin::configurations::{
    connection_config::ConnectionConfig, download_config::DownloadConfig
};

use cargosos_bitcoin::{
    logs::logger_sender::LoggerSender,
    block_structure::{
        block_chain::BlockChain,
        utxo_set::UTXOSet,
    },
    connections::ibd_methods::IBDMethod,
};

use std::net::{SocketAddr, TcpStream};

use cargosos_bitcoin::wallet_structure::{private_key, public_key};


use std::sync::mpsc;

pub trait VecOwnExt {
    fn search_by_name(&self, name: &str) -> Object;
    fn search_window_named(&self, name: &str) -> Window;
    fn search_button_named(&self, name: &str) -> Button;
    fn search_entry_named(&self, name: &str) -> Entry;
    fn search_combo_box_named(&self, name: &str) -> ComboBoxText;
    //fn search_radio_button_named(&self, name: &str) -> RadioButton;

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
    
}

fn build_ui(application: &gtk::Application, glade_src: &str) {
    let builder: Builder = Builder::from_string(glade_src);

    let objects = builder.objects();
    
    let window = objects.search_window_named("MainWindow");
    
    window.set_application(Some(application));

    window.show_all();
 
    //Combo Box
    let account_registration_window = objects.search_window_named("AccountRegistrationWindow");
    let combo_box = objects.search_combo_box_named("WalletsComboBox");
    /* 
    combo_box.append_text("Add address");
    combo_box.append_text("Tu vieja");
    */
    combo_box.connect_changed(move |combo_box| {
        if let Some(active_text) = combo_box.active_text() {
            if active_text == "Add address" {
                account_registration_window.set_visible(true);
                println!("Add address it is then!");
            } else if active_text == "Tu vieja" {
                println!("Tu vieja it is then");
            }
        }
    });
    

    //Add address button
    let account_registration_button = objects.search_button_named("AccountRegistrationButton");

    let obj_cl = objects.clone();

    let account_registration_window = obj_cl.search_window_named("AccountRegistrationWindow");
    account_registration_button.connect_clicked(move |_| {
        account_registration_window.set_visible(true);
    });

    let account_registration_window = obj_cl.search_window_named("AccountRegistrationWindow");
    let obj_cl = objects.clone();
    let save_wallet_button = objects.search_button_named("SaveWalletButton");
    save_wallet_button.connect_clicked(move |_| {
        account_registration_window.set_visible(false);
        
        let private_key_entry = obj_cl.search_entry_named("PrivateKeyEntry");
        let public_key_entry = obj_cl.search_entry_named("PublicKeyEntry");
        let address_entry = obj_cl.search_entry_named("AddressEntry");
        let name_entry = obj_cl.search_entry_named("NameEntry");

        let combo_box = objects.search_combo_box_named("WalletsComboBox");
        combo_box.append_text(name_entry.text().as_str());

        println!("{:?} {:?} {:?} {:?}", private_key_entry.text(), public_key_entry.text(), address_entry.text(), name_entry.text());

        private_key_entry.set_text("");
        public_key_entry.set_text("");
        address_entry.set_text("");
        name_entry.set_text("");            
    });

    
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
            download::blocks_first();
        }
    }

    Ok(())
}

pub fn backend_execution(
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    load_system: LoadSystem,
    logger: LoggerSender,
    tx_to_front: glib::Sender<String>,
    rx_from_front: mpsc::Receiver<String>,
) -> Result<(), ErrorExecution> {
    
    let mut load_system = load_system;

    let potential_peers = get_potential_peers(connection_config.clone(), logger.clone())?;

    let peer_streams = handshake::connect_to_peers(
        potential_peers,
        connection_config.clone(),
        logger.clone(),
    );

    let mut block_chain = load_system.get_block_chain()?;
    let mut wallet = load_system.get_wallet()?;

    println!("Wallet: {:?}", wallet);

    get_block_chain(
        peer_streams,
        &mut block_chain,
        connection_config,
        download_config,
        logger.clone(),
    )?;

    Ok(())
}


pub fn program_execution(
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    load_system: LoadSystem,
    logger: LoggerSender,
) -> Result<SaveSystem, ErrorGUI> {
    let (tx_to_back, rx_from_front) = mpsc::channel::<String>();
    let (tx_to_front, rx_from_back) = glib::MainContext::channel::<String>(glib::PRIORITY_DEFAULT);

    let backend_initialization_handler = thread::spawn(move || {
        let _ = backend_execution(connection_config, download_config, load_system, logger, tx_to_front, rx_from_front);
    });

    //let result = backend_initialization_handler.join();

    let glade_src = include_str!("WindowNotebook.glade");

    let application = Application::builder().build();

    application.connect_activate(move |app| build_ui(app, glade_src));
    let vector: Vec<String> = Vec::new();
    application.run_with_args(&vector);

    Err(ErrorGUI::TODO)
}