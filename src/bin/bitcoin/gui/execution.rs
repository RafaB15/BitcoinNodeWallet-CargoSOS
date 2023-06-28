use super::{
    error_gui::ErrorGUI, gui_backend::spawn_backend_handler, signal_to_back::SignalToBack,
    signal_to_front::SignalToFront,
};

use gtk::glib;
use gtk::{
    prelude::*, Application, Builder, Button, ComboBoxText, Entry, Image, Label, SpinButton,
    TreeStore, Window,
};

use crate::{error_execution::ErrorExecution, process::save_system::SaveSystem};

use cargosos_bitcoin::configurations::{
    connection_config::ConnectionConfig, download_config::DownloadConfig, save_config::SaveConfig, mode_config::ModeConfig,
};

use cargosos_bitcoin::logs::logger_sender::LoggerSender;

use std::{cell::Cell, sync::mpsc};

use chrono::{DateTime, NaiveDateTime, Utc};

/// This function sets up the main window
fn login_main_window(
    application: &gtk::Application,
    builder: &Builder,
    tx_to_back: mpsc::Sender<SignalToBack>,
) -> Result<(), ErrorGUI> {
    let window: Window = builder.object("MainWindow").unwrap();
    window.set_application(Some(application));

    let application_clone = application.clone();
    let tx_to_back_clone = tx_to_back.clone();
    window.connect_destroy(move |_| {
        application_clone.quit();
        if tx_to_back_clone.send(SignalToBack::ExitProgram).is_err() {
            println!("Error sending exit program signal");
        };
    });

    let account_registration_button: Button = match builder.object("AccountRegistrationButton") {
        Some(account_registration_button) => account_registration_button,
        None => {
            return Err(ErrorGUI::MissingElement(
                "AccountRegistrationButton".to_string(),
            ))
        }
    };

    let cloned_builer = builder.clone();

    account_registration_button.connect_clicked(move |_| {
        let account_registration_window: Window =
            match cloned_builer.object("AccountRegistrationWindow") {
                Some(account_registration_window) => account_registration_window,
                None => {
                    println!("Error getting account registration window");
                    Window::new(gtk::WindowType::Toplevel)
                }
            };
        account_registration_window.set_visible(true);
    });

    login_send_page(builder, tx_to_back)?;
    login_block_notification_window(builder)?;
    window.show_all();
    Ok(())
}

/// This function sets up the registration window
fn login_registration_window(
    builder: &Builder,
    application: &gtk::Application,
    tx_to_back: mpsc::Sender<SignalToBack>,
) -> Result<(), ErrorGUI> {
    let account_registration_window: Window = builder.object("AccountRegistrationWindow").unwrap();
    account_registration_window.set_application(Some(application));

    let cloned_builder = builder.clone();

    let save_wallet_button: Button = builder.object("SaveWalletButton").unwrap();
    save_wallet_button.connect_clicked(move |_| {
        account_registration_window.set_visible(false);

        let private_key_entry: Entry = cloned_builder.object("PrivateKeyEntry").unwrap();
        let public_key_entry: Entry = cloned_builder.object("PublicKeyEntry").unwrap();
        let name_entry: Entry = cloned_builder.object("NameEntry").unwrap();

        if tx_to_back
            .send(SignalToBack::CreateAccount(
                name_entry.text().to_string(),
                private_key_entry.text().to_string(),
                public_key_entry.text().to_string(),
            ))
            .is_err()
        {
            println!("Error sending create account signal");
        }

        private_key_entry.set_text("");
        public_key_entry.set_text("");
        name_entry.set_text("");
    });
    Ok(())
}

/// THis function sets up the combo box
fn login_combo_box(builder: &Builder, tx_to_back: mpsc::Sender<SignalToBack>) {
    let combo_box: ComboBoxText = builder.object("WalletsComboBox").unwrap();
    let cloned_builder = builder.clone();
    combo_box.connect_changed(move |_| {
        let combo_box_cloned: ComboBoxText = cloned_builder.object("WalletsComboBox").unwrap();
        let selected_wallet = combo_box_cloned.active_text().unwrap();
        let _ = tx_to_back.send(SignalToBack::ChangeSelectedAccount(
            selected_wallet.to_string(),
        ));
        let _ = tx_to_back.send(SignalToBack::GetAccountBalance);
        let _ = tx_to_back.send(SignalToBack::GetAccountTransactions);
    });
}

/// This function sets up the error window
fn login_transaction_error_window(builder: &Builder) -> Result<(), ErrorGUI> {
    let transaction_error_window: Window = match builder.object("TransactionErrorWindow") {
        Some(transaction_error_window) => transaction_error_window,
        None => {
            return Err(ErrorGUI::MissingElement(
                "TransactionErrorWindow".to_string(),
            ))
        }
    };
    let transaction_error_button: Button = match builder.object("OkErrorButton") {
        Some(transaction_error_button) => transaction_error_button,
        None => return Err(ErrorGUI::MissingElement("OkErrorButton".to_string())),
    };
    transaction_error_button.connect_clicked(move |_| {
        transaction_error_window.set_visible(false);
    });
    Ok(())
}

/// This function sets up the notification window for transactions
fn login_transaction_notification_window(builder: &Builder) -> Result<(), ErrorGUI> {
    let transaction_notification_window: Window =
        match builder.object("TransactionNotificationWindow") {
            Some(transaction_notification_window) => transaction_notification_window,
            None => {
                return Err(ErrorGUI::MissingElement(
                    "TransactionNotificationWindow".to_string(),
                ))
            }
        };
    let transaction_notification_button: Button = match builder.object("OkNotificationButton") {
        Some(transaction_notification_button) => transaction_notification_button,
        None => return Err(ErrorGUI::MissingElement("OkNotificationButton".to_string())),
    };
    transaction_notification_button.connect_clicked(move |_| {
        transaction_notification_window.set_visible(false);
    });
    Ok(())
}

/// This function sets up the notification window for blocks
fn login_block_notification_window(builder: &Builder) -> Result<(), ErrorGUI> {
    let block_notification_window: Window = match builder.object("BlockNotificationWindow") {
        Some(block_notification_window) => block_notification_window,
        None => {
            return Err(ErrorGUI::MissingElement(
                "BlockNotificationWindow".to_string(),
            ))
        }
    };
    let block_notification_button: Button = match builder.object("OkBlockNotificationButton") {
        Some(block_notification_button) => block_notification_button,
        None => {
            return Err(ErrorGUI::MissingElement(
                "OkBlockNotificationButton".to_string(),
            ))
        }
    };
    block_notification_button.connect_clicked(move |_| {
        block_notification_window.set_visible(false);
    });
    Ok(())
}

/// This function makes the error window visible and sets the error message
fn show_window_with_error(builder: &Builder, error: &str) -> Result<(), ErrorGUI> {
    let transaction_error_window: Window = match builder.object("TransactionErrorWindow") {
        Some(transaction_error_window) => transaction_error_window,
        None => {
            return Err(ErrorGUI::MissingElement(
                "TransactionErrorWindow".to_string(),
            ))
        }
    };
    let error_label: Label = match builder.object("ErrorLabel") {
        Some(error_label) => error_label,
        None => return Err(ErrorGUI::MissingElement("ErrorLabel".to_string())),
    };
    error_label.set_text(error);
    transaction_error_window.set_visible(true);
    Ok(())
}

/// This function makes the notification window visible and sets the notification message
fn show_new_transaction_notification(
    builder: &Builder,
    account_name: String,
) -> Result<(), ErrorGUI> {
    let transaction_notification_window: Window =
        match builder.object("TransactionNotificationWindow") {
            Some(transaction_notification_window) => transaction_notification_window,
            None => {
                return Err(ErrorGUI::MissingElement(
                    "TransactionNotificationWindow".to_string(),
                ))
            }
        };
    let notification_label: Label = match builder.object("TransactionNotificationLabel") {
        Some(notification_label) => notification_label,
        None => {
            return Err(ErrorGUI::MissingElement(
                "TransactionNotificationLabel".to_string(),
            ))
        }
    };
    notification_label.set_text(format!("New transaction for account {}", account_name).as_str());
    transaction_notification_window.set_visible(true);
    Ok(())
}

/// This function makes the notification window visible and sets the notification message
fn show_new_block_notification(builder: &Builder) -> Result<(), ErrorGUI> {
    let block_notification_window: Window = match builder.object("BlockNotificationWindow") {
        Some(block_notification_window) => block_notification_window,
        None => {
            return Err(ErrorGUI::MissingElement(
                "BlockNotificationWindow".to_string(),
            ))
        }
    };
    block_notification_window.set_visible(true);
    Ok(())
}

/// This function adds an account to the combo box
fn add_account_to_combo_box(builder: &Builder, account_name: &str) -> Result<(), ErrorGUI> {
    let combo_box: ComboBoxText = match builder.object("WalletsComboBox") {
        Some(combo_box) => combo_box,
        None => return Err(ErrorGUI::MissingElement("WalletsComboBox".to_string())),
    };
    combo_box.append_text(account_name);
    Ok(())
}

///Function that clears the contents of the send transaction window
fn clear_send_transaction_contents(builder: &Builder) {
    let bitcoin_address_entry: Entry = match builder.object("BitcoinAddressEntry") {
        Some(entry) => entry,
        None => {
            println!("Error: Missing element BitcoinAddressEntry");
            Entry::new()
        }
    };
    let amount_spin_button: SpinButton = match builder.object("AmountSpinButton") {
        Some(entry) => entry,
        None => {
            println!("Error: Missing element AmountSpinButton");
            SpinButton::with_range(0.0, 0.0, 0.0)
        }
    };
    let fee_spin_button: SpinButton = match builder.object("FeeSpinButton") {
        Some(entry) => entry,
        None => {
            println!("Error: Missing element FeeSpinButton");
            SpinButton::with_range(0.0, 0.0, 0.0)
        }
    };
    bitcoin_address_entry.set_text("");
    amount_spin_button.set_value(0.0);
    fee_spin_button.set_value(0.0);
}

/// Function that sets up the send transaction page
fn login_send_page(
    builder: &Builder,
    tx_to_back: mpsc::Sender<SignalToBack>,
) -> Result<(), ErrorGUI> {
    let transaction_clear_all_button: Button = match builder.object("TransactionClearAllButton") {
        Some(button) => button,
        None => {
            return Err(ErrorGUI::MissingElement(
                "TransactionClearAllButton".to_string(),
            ))
        }
    };
    let cloned_builder = builder.clone();
    transaction_clear_all_button.connect_clicked(move |_| {
        clear_send_transaction_contents(&cloned_builder);
    });

    let transaction_send_button: Button = match builder.object("TransactionSendButton") {
        Some(button) => button,
        None => {
            return Err(ErrorGUI::MissingElement(
                "TransactionSendButton".to_string(),
            ))
        }
    };

    let cloned_builder = builder.clone();

    transaction_send_button.connect_clicked(move |_| {
        let bitcoin_address_entry: Entry = match cloned_builder.object("BitcoinAddressEntry") {
            Some(entry) => entry,
            None => {
                println!("Error: Missing element BitcoinAddressEntry");
                Entry::new()
            }
        };
        let amount_spin_button: SpinButton = match cloned_builder.object("AmountSpinButton") {
            Some(entry) => entry,
            None => {
                println!("Error: Missing element AmountSpinButton");
                SpinButton::with_range(0.0, 0.0, 0.0)
            }
        };
        let fee_spin_button: SpinButton = match cloned_builder.object("FeeSpinButton") {
            Some(entry) => entry,
            None => {
                println!("Error: Missing element FeeSpinButton");
                SpinButton::with_range(0.0, 0.0, 0.0)
            }
        };
        let _ = tx_to_back.send(SignalToBack::CreateTransaction(
            bitcoin_address_entry.text().to_string(),
            amount_spin_button.value(),
            fee_spin_button.value(),
        ));
        bitcoin_address_entry.set_text("");
        amount_spin_button.set_value(0.0);
        fee_spin_button.set_value(0.0);
    });

    Ok(())
}

/// Function that takes a timestamp and turns it into a string of the date
fn from_timestamp_to_string(timestamp: &u32) -> Result<String, ErrorGUI> {
    let naive = match NaiveDateTime::from_timestamp_opt(*timestamp as i64, 0) {
        Some(naive) => naive,
        None => {
            return Err(ErrorGUI::ErrorReading(
                "Error reading timestamp".to_string(),
            ))
        }
    };
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    Ok(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
}

/// Function that takes a vector of u8 and turns it into a string
fn from_vector_to_string(vector: &[u8; 32]) -> String {
    let mut string = String::new();
    for byte in vector.iter() {
        string.push_str(&format!("{:02x}", byte));
    }
    string
}

/// Function that updates the tree vies with the transactions of the current account
fn show_transactions_in_tree_view(
    builder: &Builder,
    transaction_information: Vec<(u32, [u8; 32], i64)>,
) -> Result<(), ErrorGUI> {
    let transactions_tree_store: TreeStore = match builder.object("TransactionTreeStore") {
        Some(list_store) => list_store,
        None => return Err(ErrorGUI::MissingElement("TransactionTreeStore".to_string())),
    };

    transactions_tree_store.clear();

    for (timestamp, label, amount) in transaction_information.iter().rev() {
        let tree_iter = transactions_tree_store.append(None);
        transactions_tree_store.set_value(
            &tree_iter,
            0,
            &glib::Value::from(from_timestamp_to_string(timestamp)?),
        );
        transactions_tree_store.set_value(&tree_iter, 1, &glib::Value::from("Mined".to_string()));
        transactions_tree_store.set_value(
            &tree_iter,
            2,
            &glib::Value::from(from_vector_to_string(label)),
        );
        transactions_tree_store.set_value(&tree_iter, 3, &glib::Value::from(amount.to_string()));
    }
    Ok(())
}

/// This functions sets up the behaviour of the GUI when it receives a signal from the backend
fn spawn_local_handler(
    builder: &Builder,
    rx_from_back: glib::Receiver<SignalToFront>,
    tx_to_back: mpsc::Sender<SignalToBack>,
) {
    let cloned_builder = builder.clone();

    rx_from_back.attach(None, move |signal| {
        match signal {
            SignalToFront::RegisterWallet(wallet_name) => {
                if let Err(error) = add_account_to_combo_box(&cloned_builder, wallet_name.as_str())
                {
                    println!("Error adding account to combo box, with error {:?}", error);
                };
            }
            SignalToFront::LoadAvailableBalance(balance) => {
                let balance_label: Label = cloned_builder.object("AvailableBalanceLabel").unwrap();
                let pending_label: Label = cloned_builder.object("PendingBalanceLabel").unwrap();
                let total_label: Label = cloned_builder.object("TotalBalanceLabel").unwrap();

                let balance_string = format!("{:.8}", balance.0);
                let pending_string = format!("{:.8}", balance.1);
                let total_string = format!("{:.8}", balance.0 + balance.1);

                balance_label.set_text(&balance_string);
                pending_label.set_text(&pending_string);
                total_label.set_text(&total_string);
            }
            SignalToFront::NotifyBlockchainIsReady => {
                let signal_blockchain_not_ready: Image =
                    match cloned_builder.object("BlockchainNotReadySymbol") {
                        Some(image) => image,
                        None => {
                            println!("Error: Missing element BlockchainNotReadySymbol");
                            Image::new()
                        }
                    };
                signal_blockchain_not_ready.set_visible(false);
            }
            SignalToFront::ErrorInTransaction(error) => {
                if let Err(error) = show_window_with_error(&cloned_builder, error.as_str()) {
                    println!("Error showing error window, with error {:?}", error);
                };
            }
            SignalToFront::TransactionOfAccountReceived(account) => {
                if let Err(error) = show_new_transaction_notification(&cloned_builder, account) {
                    println!(
                        "Error showing new transaction notification, with error {:?}",
                        error
                    );
                };
            }
            SignalToFront::BlockWithUnconfirmedTransactionReceived => {
                if let Err(error) = show_new_block_notification(&cloned_builder) {
                    println!(
                        "Error showing new block notification, with error {:?}",
                        error
                    );
                };
            }
            SignalToFront::AccountTransactions(transaction_information) => {
                if let Err(error) =
                    show_transactions_in_tree_view(&cloned_builder, transaction_information)
                {
                    println!(
                        "Error showing transactions in tree view, with error {:?}",
                        error
                    );
                };
            }
            SignalToFront::Update => {
                if tx_to_back.send(SignalToBack::GetAccountBalance).is_err()
                    || tx_to_back
                        .send(SignalToBack::GetAccountTransactions)
                        .is_err()
                {
                    println!("Error sending signal to back");
                };
            }
            SignalToFront::ErrorInAccountCreation(error) => {
                if let Err(error) = show_window_with_error(&cloned_builder, error.as_str()) {
                    println!("Error showing error window, with error {:?}", error);
                };
            }
        }
        glib::Continue(true)
    });
}

/// Function that sets up all the elemeents in the ui
fn build_ui(
    tx_to_back: mpsc::Sender<SignalToBack>,
    rx_from_back: Option<glib::Receiver<SignalToFront>>,
    application: &gtk::Application,
    glade_src: &str,
) -> Result<(), ErrorGUI> {
    let rx_from_back = match rx_from_back {
        Some(rx) => rx,
        None => {
            return Err(ErrorGUI::MissingReceiver);
        }
    };

    let builder: Builder = Builder::from_string(glade_src);

    spawn_local_handler(&builder, rx_from_back, tx_to_back.clone());

    login_main_window(application, &builder, tx_to_back.clone())?;

    login_registration_window(&builder, application, tx_to_back.clone())?;

    login_combo_box(&builder, tx_to_back);

    login_transaction_error_window(&builder)?;
    login_transaction_notification_window(&builder)?;

    Ok(())
}

/// The main function of the program for the graphical interface.
pub fn program_execution(
    mode_config: ModeConfig,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    save_config: SaveConfig,
    logger: LoggerSender,
) -> Result<SaveSystem, ErrorExecution> {
    let (tx_to_back, rx_from_front) = mpsc::channel::<SignalToBack>();
    let (tx_to_front, rx_from_back) =
        glib::MainContext::channel::<SignalToFront>(glib::PRIORITY_DEFAULT);

    let backend_handler = spawn_backend_handler(
        mode_config,
        connection_config,
        download_config,
        save_config,
        logger,
        tx_to_front,
        rx_from_front,
    );

    let glade_src = include_str!("WindowNotebook.glade");

    let application = Application::builder().build();

    let wrapped_rx_to_back: Cell<Option<gtk::glib::Receiver<SignalToFront>>> =
        Cell::new(Some(rx_from_back));

    application.connect_activate(move |app| {
        if let Err(error) = build_ui(
            tx_to_back.clone(),
            wrapped_rx_to_back.take(),
            app,
            glade_src,
        ) {
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
