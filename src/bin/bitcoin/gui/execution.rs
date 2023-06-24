use super::{
    error_gui::ErrorGUI,
    signal_to_front::SignalToFront,
    signal_to_back::SignalToBack,
    gui_backend::spawn_backend_handler,
};

use gtk::{prelude::*, Button, Entry, Application, Builder, Window, ComboBoxText, Image, Label, SpinButton};
use gtk::glib::{self, Sender};

use crate::{
    process::save_system::SaveSystem,
    error_execution::ErrorExecution,
};

use cargosos_bitcoin::configurations::{
    connection_config::ConnectionConfig, download_config::DownloadConfig,
    save_config::SaveConfig,
};

use cargosos_bitcoin::{
    logs::logger_sender::LoggerSender,
};

use std::{
    sync::mpsc, 
    cell::Cell,
};

fn login_main_window(application: &gtk::Application, builder: &Builder, tx_to_back: mpsc::Sender<SignalToBack>) -> Result<(), ErrorGUI>{

    let window: Window = builder.object("MainWindow").unwrap();
    window.set_application(Some(application));

    let application_clone = application.clone();
    let tx_to_back_clone = tx_to_back.clone();
    window.connect_destroy(move |_| {
        application_clone.quit();
        if tx_to_back_clone.send(SignalToBack::ExitProgram).is_err(){
            println!("Error sending exit program signal");
        };
    });

    let account_registration_button: Button = match builder.object("AccountRegistrationButton") {
        Some(account_registration_button) => account_registration_button,
        None => return Err(ErrorGUI::MissingElement("AccountRegistrationButton".to_string())),
    };
    
    let cloned_builer = builder.clone();
    
    account_registration_button.connect_clicked(move |_| {
        let account_registration_window: Window = match cloned_builer.object("AccountRegistrationWindow"){
            Some(account_registration_window) => account_registration_window,
            None => {
                println!("Error getting account registration window");
                Window::new(gtk::WindowType::Toplevel)
            },
        };
        account_registration_window.set_visible(true);
    });

    login_send_page(&builder, tx_to_back.clone())?;

    window.show_all();
    Ok(())
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
        let _ = tx_to_back.send(SignalToBack::ChangeSelectedAccount(selected_wallet.to_string()));
        let _ = tx_to_back.send(SignalToBack::GetAccountBalance);
    });
}


fn login_transaction_error_window(builder: &Builder, error: &str) {
    let transaction_error_window: Window = builder.object("TransactionErrorWindow").unwrap();
    let error_label: Label = builder.object("ErrorLabel").unwrap();
    error_label.set_text(error);
    let cloned_builder = builder.clone();
    let transaction_error_button: Button = builder.object("OkButton").unwrap();
    transaction_error_button.connect_clicked(move |_| {
        transaction_error_window.set_visible(false);
    });
}
/* 
fn register_transaction(tx_to_back: mpsc::Sender<SignalToBack> ,builder: &Builder) {

    let send_button: Button = builder.object("SendButton").unwrap();
    let cloned_builder: Builder = builder.clone();
    send_button.connect_clicked(move |_| {
        let adress_entry: Label = cloned_builder.object("AddressEntry").unwrap();
        let amount_entry: Label = cloned_builder.object("AmountEntry").unwrap();

        tx_to_back.send(SignalToBack::CreateTransaction(adress_entry.text().to_string(), amount_entry.text().to_string()));
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
                let pending_label: Label = cloned_builder.object("PendingBalanceLabel").unwrap();
                let total_label: Label = cloned_builder.object("TotalBalanceLabel").unwrap();

                balance_label.set_text(balance.0.to_string().as_str());
                if balance.1 != 0.0 {
                    pending_label.set_text(balance.1.to_string().as_str());
                }
                total_label.set_text((balance.0 + balance.1).to_string().as_str());
            },
            SignalToFront::LoadBlockChain => {
                let signal_blockchain_not_ready: Image = cloned_builder.object("BlockchainNotReadySymbol").unwrap();
                signal_blockchain_not_ready.set_visible(false);
            }
            SignalToFront::ErrorInTransaction(error) => {
                login_transaction_error_window(&cloned_builder, error.as_str());
            },
            _ => {}


            //recibir la blockchain -> integrarla al load bar
            //obtener transacciones de bloques ->  cargarlas al tree view
        }
        glib::Continue(true)
    });
} 

fn login_send_page(builder: &Builder, tx_to_back: mpsc::Sender<SignalToBack>) -> Result<(), ErrorGUI>{
    let transaction_clear_all_button: Button = match builder.object("TransactionClearAllButton") {
        Some(button) => button,
        None => return Err(ErrorGUI::MissingElement("TransactionClearAllButton".to_string())),
    };
    let cloned_builder = builder.clone();
    transaction_clear_all_button.connect_clicked(move |_| {
        let bitcoin_address_entry: Entry = match cloned_builder.object("BitcoinAddressEntry"){
            Some(entry) => entry,
            None => {
                println!("Error: Missing element BitcoinAddressEntry");
                Entry::new()
            },
        };
        let amount_spin_button: SpinButton = match cloned_builder.object("AmountSpinButton"){
            Some(entry) => entry,
            None => {
                println!("Error: Missing element AmountSpinButton");
                SpinButton::with_range(0.0, 0.0, 0.0)
            },
        };
        bitcoin_address_entry.set_text("");
        amount_spin_button.set_value(0.0);
    });

    let transaction_send_button: Button = match builder.object("TransactionSendButton") {
        Some(button) => button,
        None => return Err(ErrorGUI::MissingElement("TransactionSendButton".to_string())),
    };

    let cloned_builder = builder.clone();

    transaction_send_button.connect_clicked(move |_| {
        let bitcoin_address_entry: Entry = match cloned_builder.object("BitcoinAddressEntry"){
            Some(entry) => entry,
            None => {
                println!("Error: Missing element BitcoinAddressEntry");
                Entry::new()
            },
        };
        let amount_spin_button: SpinButton = match cloned_builder.object("AmountSpinButton"){
            Some(entry) => entry,
            None => {
                println!("Error: Missing element AmountSpinButton");
                SpinButton::with_range(0.0, 0.0, 0.0)
            },
        };
        let fee_spin_button: SpinButton = match cloned_builder.object("FeeSpinButton"){
            Some(entry) => entry,
            None => {
                println!("Error: Missing element FeeSpinButton");
                SpinButton::with_range(0.0, 0.0, 0.0)
            },
        };
        let _ = tx_to_back.send(SignalToBack::CreateTransaction(bitcoin_address_entry.text().to_string(), amount_spin_button.value(), fee_spin_button.value()));
    });

    Ok(())
}

fn build_ui(
    tx_to_back: mpsc::Sender<SignalToBack>,
    rx_from_back: Option<glib::Receiver<SignalToFront>>,
    application: &gtk::Application, 
    glade_src: &str,
) -> Result<(), ErrorGUI>{

    let rx_from_back = match rx_from_back {
        Some(rx) => rx,
        None => {
            return Err(ErrorGUI::MissingReceiver);
        },
    };

    let builder: Builder = Builder::from_string(glade_src);

    spawn_local_handler(&builder, rx_from_back);

    login_main_window(application, &builder, tx_to_back.clone())?;

    login_registration_window(&builder, application);

    login_combo_box(&builder, tx_to_back.clone());
    Ok(())
}

pub fn program_execution(
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    save_config: SaveConfig,
    logger: LoggerSender,
) -> Result<SaveSystem, ErrorExecution> {

    let (tx_to_back, rx_from_front) = mpsc::channel::<SignalToBack>();
    let (tx_to_front, rx_from_back) = glib::MainContext::channel::<SignalToFront>(glib::PRIORITY_DEFAULT);

    let backend_handler = spawn_backend_handler(connection_config, download_config, save_config, logger, tx_to_front, rx_from_front);

    let glade_src = include_str!("WindowNotebook.glade");

    let application = Application::builder()
        .build();

    let wrapped_rx_to_back: Cell<Option<gtk::glib::Receiver<SignalToFront>>> = Cell::new(Some(rx_from_back));

    application.connect_activate(move |app| {
        if let Err(error) = build_ui(tx_to_back.clone(), wrapped_rx_to_back.take(), app, glade_src) {
            println!("Error: {:?}", error);
        }
    });
    let vector: Vec<String> = Vec::new();
    application.run_with_args(&vector);

    match backend_handler.join() {
        Ok(save_system) => save_system,
        Err(_) => Err(ErrorExecution::FailThread),
    }
}
