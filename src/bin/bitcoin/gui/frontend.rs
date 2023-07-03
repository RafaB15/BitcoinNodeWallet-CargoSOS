use super::{signal_to_back::SignalToBack, signal_to_front::SignalToFront};

use crate::{
    process::{
        reference::{self, MutArc},
        transaction,
    },
    ui::{account, error_ui::ErrorUI, from_hexa},
};

use cargosos_bitcoin::{
    notifications::{notification::Notification, notifier::Notifier},
    wallet_structure::{private_key::PrivateKey, public_key::PublicKey, wallet::Wallet},
    node_structure::connection_id::ConnectionId,
    logs::logger_sender::LoggerSender,
    block_structure::{
        hash::HashType,
        block_chain::BlockChain,
    },
};

use gtk::{
    glib, prelude::*, Builder, Button, ComboBoxText, Entry, Image, Label, ProgressBar, SpinButton,
    TreeStore, Window,
};

use glib::GString;

use std::sync::mpsc::Sender;

use chrono::{DateTime, NaiveDateTime, Utc};

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

/// This function sets up the main window
fn login_main_window(
    application: &gtk::Application,
    builder: &Builder,
    tx_to_back: Sender<SignalToBack>,
) -> Result<(), ErrorUI> {
    let window: Window = match builder.object("MainWindow") {
        Some(window) => window,
        None => return Err(ErrorUI::MissingElement("MainWindow".to_string())),
    };
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
            return Err(ErrorUI::MissingElement(
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
    tx_to_back: Sender<SignalToBack>,
) -> Result<(), ErrorUI> {
    let account_registration_window: Window = match builder.object("AccountRegistrationWindow") {
        Some(account_registration_window) => account_registration_window,
        None => {
            return Err(ErrorUI::MissingElement(
                "AccountRegistrationWindow".to_string(),
            ))
        }
    };
    account_registration_window.set_application(Some(application));

    let cloned_builder = builder.clone();

    let save_wallet_button: Button = match builder.object("SaveWalletButton") {
        Some(save_wallet_button) => save_wallet_button,
        None => return Err(ErrorUI::MissingElement("SaveWalletButton".to_string())),
    };
    save_wallet_button.connect_clicked(move |_| {
        account_registration_window.set_visible(false);

        let private_key_entry: Entry = match cloned_builder.object("PrivateKeyEntry") {
            Some(entry) => entry,
            None => {
                println!("Error: Missing element PrivateKeyEntry");
                Entry::new()
            }
        };
        let public_key_entry: Entry = match cloned_builder.object("PublicKeyEntry") {
            Some(entry) => entry,
            None => {
                println!("Error: Missing element PublicKeyEntry");
                Entry::new()
            }
        };
        let name_entry: Entry = match cloned_builder.object("NameEntry") {
            Some(entry) => entry,
            None => {
                println!("Error: Missing element NameEntry");
                Entry::new()
            }
        };

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

/// This function sets up the combo box
fn login_combo_box(builder: &Builder, tx_to_back: Sender<SignalToBack>) -> Result<(), ErrorUI> {
    let combo_box: ComboBoxText = match builder.object("WalletsComboBox") {
        Some(combo_box) => combo_box,
        None => return Err(ErrorUI::MissingElement("WalletsComboBox".to_string())),
    };
    let cloned_builder = builder.clone();
    combo_box.connect_changed(move |_| {
        let combo_box_cloned: ComboBoxText = match cloned_builder.object("WalletsComboBox") {
            Some(combo_box) => combo_box,
            None => {
                println!("Error: Missing element WalletsComboBox");
                ComboBoxText::new()
            }
        };
        let selected_wallet = match combo_box_cloned.active_text() {
            Some(selected_wallet) => selected_wallet,
            None => {
                println!("Error: Missing element WalletsComboBox");
                GString::new()
            }
        };
        if let Err(error) = tx_to_back.send(SignalToBack::ChangeSelectedAccount(
            selected_wallet.to_string(),
        )) {
            println!("Error sending change selected account signal: {}", error);
        }
        if let Err(error) = tx_to_back.send(SignalToBack::GetAccountBalance) {
            println!("Error sending get account balance signal: {}", error);
        };
        if let Err(error) = tx_to_back.send(SignalToBack::GetAccountTransactions) {
            println!("Error sending get account transactions signal: {}", error);
        };
    });
    Ok(())
}

/// This function sets up the error window
fn login_transaction_error_window(builder: &Builder) -> Result<(), ErrorUI> {
    let transaction_error_window: Window = match builder.object("TransactionErrorWindow") {
        Some(transaction_error_window) => transaction_error_window,
        None => {
            return Err(ErrorUI::MissingElement(
                "TransactionErrorWindow".to_string(),
            ))
        }
    };
    let transaction_error_button: Button = match builder.object("OkErrorButton") {
        Some(transaction_error_button) => transaction_error_button,
        None => return Err(ErrorUI::MissingElement("OkErrorButton".to_string())),
    };
    transaction_error_button.connect_clicked(move |_| {
        transaction_error_window.set_visible(false);
    });
    Ok(())
}

/// This function sets up the error window
fn login_merkle_error_window(builder: &Builder) -> Result<(), ErrorUI> {
    let merkle_error_window: Window = match builder.object("MerkleProofErrorWindow") {
        Some(merkle_error_window) => merkle_error_window,
        None => {
            return Err(ErrorUI::MissingElement(
                "MerkleProofErrorWindow".to_string(),
            ))
        }
    };
    let merkle_error_button: Button = match builder.object("OKMerkleProofErrorButton") {
        Some(merkle_error_button) => merkle_error_button,
        None => return Err(ErrorUI::MissingElement("OKMerkleProofErrorButton".to_string())),
    };
    merkle_error_button.connect_clicked(move |_| {
        merkle_error_window.set_visible(false);
    });
    Ok(())
}

/// Function that makes the error window for the merkle proof of inclusion visible
fn show_merkle_error_window(builder: &Builder, error: String) -> Result<(), ErrorUI>{
    let merkle_error_window: Window = match builder.object("MerkleProofErrorWindow") {
        Some(merkle_error_window) => merkle_error_window,
        None => {
            return Err(ErrorUI::MissingElement(
                "MerkleProofErrorWindow".to_string(),
            ))
        }
    };
    let merkle_error_label: Label = match builder.object("MerkleProofErrorLabel") {
        Some(merkle_error_label) => merkle_error_label,
        None => return Err(ErrorUI::MissingElement("MerkleProofErrorLabel".to_string())),
    };
    merkle_error_label.set_text(&error);
    merkle_error_window.set_visible(true);
    Ok(())
}

/// Turns a hashtype to a string
fn from_hashtype_to_string(hash: HashType) -> String {
    let mut hash_string = "".to_string();
    for byte in hash.iter() {
        hash_string.push_str(&format!("{:02x}", byte));
    }
    hash_string
}

fn show_merkle_proof_success_window(builder: &Builder, merkle_path: Vec<HashType>, root: HashType) -> Result<(), ErrorUI>{
    let merkle_success_window: Window = match builder.object("MerkleProofSuccessfulWindow") {
        Some(merkle_success_window) => merkle_success_window,
        None => {
            return Err(ErrorUI::MissingElement(
                "MerkleProofSuccessfulWindow".to_string(),
            ))
        }
    };
    let merkle_success_label: Label = match builder.object("MerkleProofNotificationfulLabel") {
        Some(merkle_success_label) => merkle_success_label,
        None => return Err(ErrorUI::MissingElement("MerkleProofNotificationfulLabel".to_string())),
    };

    
    let mut message_path = "".to_string();
    
    for hash in merkle_path.clone() {
        message_path.push_str(&format!("{}\n", from_hashtype_to_string(hash)));
    }
    
    let message = format!("Merkle root: \n{}\n Merkle path:\n{}", from_hashtype_to_string(root), message_path);

    merkle_success_label.set_text(&message);
    merkle_success_window.set_visible(true);
    Ok(())
}

/// This function sets up the error window
fn login_merkle_proof_successful_window(builder: &Builder) -> Result<(), ErrorUI> {
    let merkle_success_window: Window = match builder.object("MerkleProofSuccessfulWindow") {
        Some(merkle_success_window) => merkle_success_window,
        None => {
            return Err(ErrorUI::MissingElement(
                "MerkleProofSuccessfulWindow".to_string(),
            ))
        }
    };
    let merkle_success_button: Button = match builder.object("OkMerkleProofNotificationButton") {
        Some(merkle_success_button) => merkle_success_button,
        None => return Err(ErrorUI::MissingElement("OkMerkleProofNotificationButton".to_string())),
    };
    merkle_success_button.connect_clicked(move |_| {
        merkle_success_window.set_visible(false);
    });
    Ok(())
}

/// This function sets up the notification window for transactions
fn login_transaction_notification_window(builder: &Builder) -> Result<(), ErrorUI> {
    let transaction_notification_window: Window =
        match builder.object("TransactionNotificationWindow") {
            Some(transaction_notification_window) => transaction_notification_window,
            None => {
                return Err(ErrorUI::MissingElement(
                    "TransactionNotificationWindow".to_string(),
                ))
            }
        };
    let transaction_notification_button: Button = match builder.object("OkNotificationButton") {
        Some(transaction_notification_button) => transaction_notification_button,
        None => return Err(ErrorUI::MissingElement("OkNotificationButton".to_string())),
    };
    transaction_notification_button.connect_clicked(move |_| {
        transaction_notification_window.set_visible(false);
    });
    Ok(())
}

/// This function sets up the notification window for blocks
fn login_block_notification_window(builder: &Builder) -> Result<(), ErrorUI> {
    let block_notification_window: Window = match builder.object("BlockNotificationWindow") {
        Some(block_notification_window) => block_notification_window,
        None => {
            return Err(ErrorUI::MissingElement(
                "BlockNotificationWindow".to_string(),
            ))
        }
    };
    let block_notification_button: Button = match builder.object("OkBlockNotificationButton") {
        Some(block_notification_button) => block_notification_button,
        None => {
            return Err(ErrorUI::MissingElement(
                "OkBlockNotificationButton".to_string(),
            ))
        }
    };
    block_notification_button.connect_clicked(move |_| {
        block_notification_window.set_visible(false);
    });
    Ok(())
}

/// Requests the merkle proof of a transaction with the data entered by the user
///
/// ### Error
///  * `ErrorUI::FailedSignalToFront`: It will appear when the sender fails
pub fn request_merkle_proof<N: Notifier>(
    block_chain: &BlockChain,
    block_hash: &str,
    transaction_id: &str,
    notifier: N,
    logger: LoggerSender,

) -> Result<(), ErrorUI> {
    let block_hash_bytes: HashType = match from_hexa::from(block_hash.to_string())?.try_into() {
        Ok(block_hash_bytes) => block_hash_bytes,
        Err(_) => {
            let _ = logger.log_error("Error reading block hash".to_string());
            return Err(ErrorUI::ErrorReading("Block hash".to_string()));
        }
    };

    let transaction_id_bytes: HashType = match from_hexa::from(transaction_id.to_string())?.try_into() {
        Ok(transaction_id_bytes) => transaction_id_bytes,
        Err(_) => {
            let _ = logger.log_error("Error reading block hash".to_string());
            return Err(ErrorUI::ErrorReading("Block hash".to_string()));
        }
    };

    transaction::verify_transaction_merkle_proof_of_inclusion(
        block_chain,
        block_hash_bytes,
        transaction_id_bytes,
        notifier,
        logger
    );

    Ok(())
}


/// This function sets up the notification window for merkle proof
fn login_markle_proof_window(builder: &Builder, tx_to_back: Sender<SignalToBack>) -> Result<(), ErrorUI> {

    let validation_button: Button = match builder.object("MerkleProofValidateButton") {
        Some(validation_button) => validation_button,
        None => {
            return Err(ErrorUI::MissingElement(
                "MerkleProofValidateButton".to_string(),
            ))
        }
    };
    let cloned_builder = builder.clone();
    validation_button.connect_clicked(move |_|{
        let block_hash: Entry = match cloned_builder.object("BlockHeaderHashEntry") {
            Some(entry) => entry,
            None => {
                println!("Error: Missing element BlockHeaderHashEntry");
                Entry::new()
            }
        };
        let transaction_id: Entry = match cloned_builder.object("TransactionIDEntry") {
            Some(entry) => entry,
            None => {
                println!("Error: Missing element TransactionIDEntry");
                Entry::new()
            }
        };

        if tx_to_back.send(SignalToBack::RequestMerkleProof(block_hash.text().to_string(), transaction_id.text().to_string())).is_err() {
            println!("Error sending merkle proof signal");
        }

    });

    Ok(())

}

/// This function makes the error window visible and sets the error message
fn show_window_with_error(builder: &Builder, error: &str) -> Result<(), ErrorUI> {
    let transaction_error_window: Window = match builder.object("TransactionErrorWindow") {
        Some(transaction_error_window) => transaction_error_window,
        None => {
            return Err(ErrorUI::MissingElement(
                "TransactionErrorWindow".to_string(),
            ))
        }
    };
    let error_label: Label = match builder.object("ErrorLabel") {
        Some(error_label) => error_label,
        None => return Err(ErrorUI::MissingElement("ErrorLabel".to_string())),
    };
    error_label.set_text(error);
    transaction_error_window.set_visible(true);
    Ok(())
}

/// This function makes the notification window visible and sets the notification message
fn show_new_transaction_notification(
    builder: &Builder,
    account_name: String,
) -> Result<(), ErrorUI> {
    let transaction_notification_window: Window =
        match builder.object("TransactionNotificationWindow") {
            Some(transaction_notification_window) => transaction_notification_window,
            None => {
                return Err(ErrorUI::MissingElement(
                    "TransactionNotificationWindow".to_string(),
                ))
            }
        };
    let notification_label: Label = match builder.object("TransactionNotificationLabel") {
        Some(notification_label) => notification_label,
        None => {
            return Err(ErrorUI::MissingElement(
                "TransactionNotificationLabel".to_string(),
            ))
        }
    };
    notification_label.set_text(format!("New transaction for account {}", account_name).as_str());
    transaction_notification_window.set_visible(true);
    Ok(())
}

/// This function makes the notification window visible and sets the notification message
fn show_new_block_notification(builder: &Builder) -> Result<(), ErrorUI> {
    let block_notification_window: Window = match builder.object("BlockNotificationWindow") {
        Some(block_notification_window) => block_notification_window,
        None => {
            return Err(ErrorUI::MissingElement(
                "BlockNotificationWindow".to_string(),
            ))
        }
    };
    block_notification_window.set_visible(true);
    Ok(())
}

/// This function adds an account to the combo box
fn add_account_to_combo_box(builder: &Builder, account_name: &str) -> Result<(), ErrorUI> {
    let combo_box: ComboBoxText = match builder.object("WalletsComboBox") {
        Some(combo_box) => combo_box,
        None => return Err(ErrorUI::MissingElement("WalletsComboBox".to_string())),
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
fn login_send_page(builder: &Builder, tx_to_back: Sender<SignalToBack>) -> Result<(), ErrorUI> {
    let transaction_clear_all_button: Button = match builder.object("TransactionClearAllButton") {
        Some(button) => button,
        None => {
            return Err(ErrorUI::MissingElement(
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
        None => return Err(ErrorUI::MissingElement("TransactionSendButton".to_string())),
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
fn from_timestamp_to_string(timestamp: &u32) -> Result<String, ErrorUI> {
    let naive = match NaiveDateTime::from_timestamp_opt(*timestamp as i64, 0) {
        Some(naive) => naive,
        None => return Err(ErrorUI::ErrorReading("Error reading timestamp".to_string())),
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
) -> Result<(), ErrorUI> {
    let transactions_tree_store: TreeStore = match builder.object("TransactionTreeStore") {
        Some(list_store) => list_store,
        None => return Err(ErrorUI::MissingElement("TransactionTreeStore".to_string())),
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

/// Function that displays the tree view with the connections
fn show_connections_in_tree_view(builder: &Builder, connection: ConnectionId ) -> Result<(), ErrorUI> {
    let connections_tree_store: TreeStore = match builder.object("ConnectionsTreeStore") {
        Some(list_store) => list_store,
        None => return Err(ErrorUI::MissingElement("ConnectionsTreeStore".to_string())),
    };

    let ip_address = connection.address.ip().to_string();
    let port = connection.address.port().to_string();

    let tree_iter = connections_tree_store.append(None);
    connections_tree_store.set_value(&tree_iter, 0, &glib::Value::from(connection.connection_type.to_string()));
    connections_tree_store.set_value(&tree_iter, 1, &glib::Value::from(ip_address));
    connections_tree_store.set_value(&tree_iter, 2, &glib::Value::from(port));

    Ok(())
}

/// This functions sets up the behaviour of the GUI when it receives a signal from the backend
fn spawn_local_handler(
    builder: &Builder,
    rx_from_back: glib::Receiver<SignalToFront>,
    tx_to_back: Sender<SignalToBack>,
) {
    let cloned_builder = builder.clone();

    rx_from_back.attach(None, move |signal| {
        match signal {
            SignalToFront::RegisterAccount(wallet_name) => {
                if let Err(error) = add_account_to_combo_box(&cloned_builder, wallet_name.as_str())
                {
                    println!("Error adding account to combo box, with error {:?}", error);
                };
            }
            SignalToFront::LoadAvailableBalance(balance) => {
                let balance_label: Label = match cloned_builder.object("AvailableBalanceLabel") {
                    Some(label) => label,
                    None => {
                        println!("Error: Missing element AvailableBalanceLabel");
                        Label::new(None)
                    }
                };
                let pending_label: Label = match cloned_builder.object("PendingBalanceLabel") {
                    Some(label) => label,
                    None => {
                        println!("Error: Missing element PendingBalanceLabel");
                        Label::new(None)
                    }
                };
                let total_label: Label = match cloned_builder.object("TotalBalanceLabel") {
                    Some(label) => label,
                    None => {
                        println!("Error: Missing element TotalBalanceLabel");
                        Label::new(None)
                    }
                };

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
            SignalToFront::UpdateBlockchainProgressBar(to_update, total) => {
                let progress_label = match cloned_builder.object("ProgressLabel") {
                    Some(progress_label) => progress_label,
                    None => {
                        println!("Error: Missing element ProgressLabel");
                        Label::new(None)
                    }
                };
                progress_label.set_text("Blockchain Update Progress");
                let progress_bar: ProgressBar = match cloned_builder.object("ProgressBar") {
                    Some(progress_bar) => progress_bar,
                    None => {
                        println!("Error: Missing element ProgressBar");
                        ProgressBar::new()
                    }
                };
                progress_bar.set_fraction(to_update as f64 / total as f64);
            }
            SignalToFront::UpdateBlockProgressBar(downloaded, total) => {
                let progress_label = match cloned_builder.object("ProgressLabel") {
                    Some(progress_label) => progress_label,
                    None => {
                        println!("Error: Missing element ProgressLabel");
                        Label::new(None)
                    }
                };
                progress_label.set_text("Block Download Progress");
                let progress_bar: ProgressBar = match cloned_builder.object("ProgressBar") {
                    Some(progress_bar) => progress_bar,
                    None => {
                        println!("Error: Missing element ProgressBar");
                        ProgressBar::new()
                    }
                };
                progress_bar.set_fraction(downloaded as f64 / total as f64);
            }
            SignalToFront::UpdateConnection(connection) => {
                if let Err(error) = show_connections_in_tree_view(&cloned_builder, connection) {
                    println!(
                        "Error showing connections in tree view, with error {:?}",
                        error
                    );
                };
            }
            SignalToFront::ErrorInMerkleProof(error) => {
                show_merkle_error_window(&cloned_builder, error);
            },
            SignalToFront::DisplayMerklePath(_, _) => todo!(),
        }
        glib::Continue(true)
    });
}

/// Function that sets up all the elemeents in the ui
pub fn build_ui(
    tx_to_back: Sender<SignalToBack>,
    rx_from_back: Option<glib::Receiver<SignalToFront>>,
    application: &gtk::Application,
    glade_src: &str,
) -> Result<(), ErrorUI> {
    let rx_from_back = match rx_from_back {
        Some(rx) => rx,
        None => {
            return Err(ErrorUI::MissingReceiver);
        }
    };

    let builder: Builder = Builder::from_string(glade_src);

    spawn_local_handler(&builder, rx_from_back, tx_to_back.clone());

    login_main_window(application, &builder, tx_to_back.clone())?;

    login_registration_window(&builder, application, tx_to_back.clone())?;

    login_combo_box(&builder, tx_to_back)?;

    login_transaction_error_window(&builder)?;
    login_merkle_error_window(&builder)?;
    login_transaction_notification_window(&builder)?;

    Ok(())
}
