use super::{
    error_gui::ErrorGUI,
    signal_to_front::SignalToFront,
    signal_to_back::SignalToBack,
    gui_backend::spawn_backend_handler,
};

use gtk::{prelude::*, Button, Entry, Application, Builder, Window, ComboBoxText, Image, Label};
use gtk::{glib};

use crate::process::save_system::SaveSystem;

use cargosos_bitcoin::configurations::{
    connection_config::ConnectionConfig, download_config::DownloadConfig,
    save_config::SaveConfig,
};

use cargosos_bitcoin::{
    logs::logger_sender::LoggerSender,
};

use std::sync::mpsc;

fn login_main_window(application: &gtk::Application, builder: &Builder) {

    let window: Window = builder.object("MainWindow").unwrap();
    window.set_application(Some(application));

    let application_clone = application.clone();

    window.connect_destroy(move |_| {
        application_clone.quit();
    });

    let account_registration_button: Button = builder.object("AccountRegistrationButton").unwrap();
    
    let cloned_builer = builder.clone();
    
    account_registration_button.connect_clicked(move |_| {
        let account_registration_window: Window = cloned_builer.object("AccountRegistrationWindow").unwrap();
        account_registration_window.set_visible(true);
    });

    window.show_all();
}

fn login_registration_window(builder: &Builder, application: &gtk::Application) {
    let account_registration_window: Window = builder.object("AccountRegistrationWindow").unwrap();
    account_registration_window.set_application(Some(application));

    let cloned_builder = builder.clone();

    let save_wallet_button: Button = builder.object("SaveWalletButton").unwrap();
    save_wallet_button.connect_clicked(move |_| {
        account_registration_window.set_visible(false);
        
        let private_key_entry: Entry = cloned_builder.object("PrivateKeyEntry").unwrap();
        let public_key_entry: Entry = cloned_builder.object("PublicKeyEntry").unwrap();
        let address_entry: Entry = cloned_builder.object("AddressEntry").unwrap();
        let name_entry: Entry = cloned_builder.object("NameEntry").unwrap();

        let combo_box: ComboBoxText= cloned_builder.object("WalletsComboBox").unwrap();
        combo_box.append_text(name_entry.text().as_str());

        println!("{:?} {:?} {:?} {:?}", private_key_entry.text(), public_key_entry.text(), address_entry.text(), name_entry.text());

        private_key_entry.set_text("");
        public_key_entry.set_text("");
        address_entry.set_text("");
        name_entry.set_text("");            
    });
}

fn login_combo_box(builder: &Builder, tx_to_back: mpsc::Sender<SignalToBack>) {
    let combo_box: ComboBoxText = builder.object("WalletsComboBox").unwrap();
    let cloned_builder = builder.clone();
    combo_box.connect_changed(move |_| {
        let combo_box_cloned: ComboBoxText = cloned_builder.object("WalletsComboBox").unwrap();
        let selected_wallet = combo_box_cloned.active_text().unwrap();
        let _ = tx_to_back.send(SignalToBack::GetAccountBalance(selected_wallet.to_string()));
        println!("{}", selected_wallet);
    });
}
/* 
fn correct_entry_information(address: &str, label: &str, amount: &str) -> Result<(),ErrorGUI> {
    todo!()
}

fn transaction_error_window(objects: &Vec<Object>) {
    let transaction_error_window = objects.search_window_named("TransactionErrorWindow");
    let cloned_objects = objects.clone();
    let transaction_error_button = objects.search_button_named("OkButton");
    transaction_error_button.connect_clicked(move |_| {
        transaction_error_window.set_visible(false);
    });
}

fn register_transaction(objects: &Vec<Object>, transaction: Transaction) {
    let send_button = objects.search_button_named("SendButton");
    let cloned_objects = objects.clone();
    send_button.connect_clicked(move |_| {
        let address_entry = cloned_objects.search_entry_named("AddressEntry");
        let label_entry = cloned_objects.search_entry_named("LabelEntry");
        let amount_entry = cloned_objects.search_entry_named("AmountEntry");
    
        if correct_entry_information(address_entry.text().as_str(), label_entry.text().as_str(), amount_entry.text().as_str()).is_err() {
            //let error_label = cloned_objects.search_label_named("ErrorLabel");
            //error_label.set_text("Error ");

            //mostrar una window de error?
            let transaction_error_window = cloned_objects.search_window_named("TransactionErrorWindow");
            transaction_error_window.set_visible(true);

            //let transaction = create_transaction();
        } else {

        };

        println!("{}", transaction);
    });


}
*/
fn spawn_local_handler(builder: &Builder, rx_from_back: glib::Receiver<SignalToFront>) {
    let cloned_builder = builder.clone();

    rx_from_back.attach(None, move |signal| {
        match signal {
            SignalToFront::RegisterWallet(wallet_name) => {
                let combo_box: ComboBoxText = cloned_builder.object("WalletsComboBox").unwrap();
                combo_box.append_text(&wallet_name);
                println!("Registering wallet: {:?}", wallet_name);
            },
            SignalToFront::LoadAvailableBalance(balance) => {
                let balance_label: Label = cloned_builder.object("AvailableBalanceLabel").unwrap();
                balance_label.set_text(balance.to_string().as_str());
            },
            SignalToFront::LoadBlockChain => {
                let signal_blockchain_not_ready: Image = cloned_builder.object("BlockchainNotReadySymbol").unwrap();
                signal_blockchain_not_ready.set_visible(false);
            }
            _ => {}


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

    let (tx_to_back, rx_from_front) = mpsc::channel::<SignalToBack>();
    let (tx_to_front, rx_from_back) = glib::MainContext::channel::<SignalToFront>(glib::PRIORITY_DEFAULT);

    spawn_backend_handler(connection_config, download_config, save_config, logger, tx_to_front, rx_from_front);

    spawn_local_handler(&builder, rx_from_back);

    login_main_window(application, &builder);

    login_registration_window(&builder, application);

    login_combo_box(&builder, tx_to_back.clone());
}

pub fn program_execution(
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    save_config: SaveConfig,
    logger: LoggerSender,
) -> Result<SaveSystem, ErrorGUI> {
    let glade_src = include_str!("WindowNotebook.glade");

    let application = Application::builder()
        .build();

    application.connect_activate(move |app| build_ui(app, glade_src, connection_config.clone(), download_config.clone(), save_config.clone(), logger.clone()));
    let vector: Vec<String> = Vec::new();
    application.run_with_args(&vector);

    Err(ErrorGUI::FailedToInitializeGTK)
}
