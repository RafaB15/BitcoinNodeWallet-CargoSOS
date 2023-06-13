use super::error_gui::ErrorGUI;

use gtk::{prelude::*, glib::Object, Button, Entry, Application, Builder, Window};

use crate::{
    error_execution::ErrorExecution,
    process::{
        download, handshake, account,
        save_system::SaveSystem, 
        load_system::LoadSystem, 
    },
};

use cargosos_bitcoin::{configurations::{
    connection_config::ConnectionConfig, 
    download_config::DownloadConfig,
}, wallet_structure::{private_key, public_key}};

use cargosos_bitcoin::logs::{
    logger_sender::LoggerSender,
};

use std::sync::mpsc;

pub trait VecOwnExt {
    fn search_by_name(&self, name: &str) -> Object;
    fn search_window_named(&self, name: &str) -> Window;
    fn search_button_named(&self, name: &str) -> Button;
    fn search_entry_named(&self, name: &str) -> Entry;
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
}

fn build_ui(application: &gtk::Application, glade_src: &str) {
    let builder: Builder = Builder::from_string(glade_src);

    let objects = builder.objects();
    
    let window = objects.search_window_named("MainWindow");
    
    window.set_application(Some(application));

    window.show_all();
    println!("hola");
    let account_registration_button = objects.search_button_named("AccountRegistrationButton");
    println!("adios");
    let obj_cl = objects.clone();
    account_registration_button.connect_clicked(move |_| {
        let account_registration_window = obj_cl.search_window_named("AccountRegistrationWindow");
        account_registration_window.set_visible(true);
        
        let save_wallet_button = objects.search_button_named("SaveWalletButton");
        let obj_cl_2 = obj_cl.clone();

        save_wallet_button.connect_clicked(move |_| {
            account_registration_window.set_visible(false);
            
            let private_key_entry = obj_cl_2.search_entry_named("PrivateKeyEntry");
            let public_key_entry = obj_cl_2.search_entry_named("PublicKeyEntry");
            let address_entry = obj_cl_2.search_entry_named("AddressEntry");
            let name_entry = obj_cl_2.search_entry_named("NameEntry");

            println!("{:?} {:?} {:?} {:?}", private_key_entry.text(), public_key_entry.text(), address_entry.text(), name_entry.text());

            private_key_entry.set_text("");
            public_key_entry.set_text("");
            address_entry.set_text("");
            name_entry.set_text("");            
        });
    });
}
/* 
pub fn backend_execution(
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    load_system: &mut LoadSystem,
    logger: LoggerSender,
) -> Result<SaveSystem, ErrorExecution> {

}
*/

pub fn program_execution(
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    load_system: &mut LoadSystem,
    logger: LoggerSender,
) -> Result<SaveSystem, ErrorGUI> {

    let (tx_to_back, rx_from_front) = mpsc::channel::<String>();
    let (tx_to_front, rx_from_back) = mpsc::channel::<String>();

    let glade_src = include_str!("WindowNotebook.glade");

    let application = Application::builder().build();

    application.connect_activate(move |app| build_ui(app, glade_src));
    let vector: Vec<String> = Vec::new();
    application.run_with_args(&vector);

    Err(ErrorGUI::TODO)
}