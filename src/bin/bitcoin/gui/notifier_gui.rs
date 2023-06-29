use super::signal_to_front::SignalToFront;

use cargosos_bitcoin::{
    block_structure::transaction::Transaction,
    logs::logger_sender::LoggerSender,
    notifications::{notification::Notification, notifier::Notifier},
    wallet_structure::account::Account,
};

use gtk::glib::Sender;

#[derive(Clone)]
pub struct NotifierGUI {
    tx_to_front: Sender<SignalToFront>,
    logger: LoggerSender,
}

impl NotifierGUI {
    pub fn new(tx_to_front: Sender<SignalToFront>, logger: LoggerSender) -> Self {
        Self {
            tx_to_front,
            logger,
        }
    }
}

impl Notifier for NotifierGUI {
    fn notify(&self, notification: Notification) {
        match notification {
            Notification::AttemptingHandshakeWithPeer(peer) => {
                println!("Attempting handshake with peer {}", peer)
            }
            Notification::SuccessfulHandshakeWithPeer(peer) => {
                println!("Successful handshake with peer {}", peer)
            }
            Notification::FailedHandshakeWithPeer(peer) => {
                println!("Failed handshake with peer {}", peer)
            }
            Notification::TransactionOfAccountReceived(accounts, _) => {
                if self.tx_to_front.send(SignalToFront::Update).is_err()
                    || self
                        .tx_to_front
                        .send(SignalToFront::TransactionOfAccountReceived(
                            accounts[0].account_name.clone(),
                        ))
                        .is_err()
                {
                    let _ = self
                        .logger
                        .log_error("Error sending notification".to_string());
                }
            }
            Notification::TransactionOfAccountInNewBlock(_) => {
                if self
                    .tx_to_front
                    .send(SignalToFront::BlockWithUnconfirmedTransactionReceived)
                    .is_err()
                {
                    let _ = self
                        .logger
                        .log_error("Error sending notification".to_string());
                }
            }
            Notification::NewBlockAddedToTheBlockchain(_) => {
                if self.tx_to_front.send(SignalToFront::Update).is_err() {
                    let _ = self
                        .logger
                        .log_error("Failed to send update of new block added".to_string());
                }
            }
            Notification::UpdatedSelectedAccount(_) => {
                if self.tx_to_front.send(SignalToFront::Update).is_err() {
                    let _ = self
                        .logger
                        .log_error("Failed to send update selected account".to_string());
                }
            }
            Notification::RegisterWalletAccount(account) => {
                if self
                    .tx_to_front
                    .send(SignalToFront::RegisterWallet(account.account_name.clone()))
                    .is_err()
                {
                    let _ = self
                        .logger
                        .log_error("Failed to send register wallet account".to_string());
                }
            }
            Notification::NotifyBlockchainIsReady => {
                if self
                    .tx_to_front
                    .send(SignalToFront::NotifyBlockchainIsReady)
                    .is_err()
                {
                    let _ = self
                        .logger
                        .log_error("Failed to signal finish block chain loading".to_string());
                }
            }
            Notification::LoadAvailableBalance((_, balance, pending)) => {
                if self
                    .tx_to_front
                    .send(SignalToFront::LoadAvailableBalance((balance, pending)))
                    .is_err()
                {
                    let _ = self
                        .logger
                        .log_error("Failed to send available balance to front".to_string());
                }
            }
            Notification::AccountNotSelected => {
                let message = "No account selected cannot get transactions";
                if self
                    .tx_to_front
                    .send(SignalToFront::ErrorInTransaction(message.to_string()))
                    .is_err()
                {
                    let _ = self
                        .logger
                        .log_error("Failed to send error signal to front".to_string());
                }
            }
            Notification::AccountTransactions((account, transactions)) => {
                let transactions = get_account_transactions_information(&account, transactions);
                if self
                    .tx_to_front
                    .send(SignalToFront::AccountTransactions(transactions))
                    .is_err()
                {
                    let _ = self.logger.log_error(
                        "Failed to send error signal of transactions from account".to_string(),
                    );
                }
            }
            Notification::InvalidAddressEnter => {
                let message = "Invalid address".to_string();
                let _ = self.logger.log_error(message.clone());
                if self
                    .tx_to_front
                    .send(SignalToFront::ErrorInTransaction(message))
                    .is_err()
                {
                    let _ = self
                        .logger
                        .log_error("Failed to send error signal to front".to_string());
                }
            }
            Notification::NotEnoughFunds => {
                if self
                    .tx_to_front
                    .send(SignalToFront::ErrorInTransaction(
                        "Error creating the transaction".to_string(),
                    ))
                    .is_err()
                {
                    let _ = self
                        .logger
                        .log_error("Failed to send error signal to front".to_string());
                };
            }
            Notification::InvalidPublicKeyEnter => {
                let message = "Invalid public key".to_string();
                let _ = self.logger.log_error(message.clone());
                if self
                    .tx_to_front
                    .send(SignalToFront::ErrorInAccountCreation(message))
                    .is_err()
                {
                    let _ = self.logger.log_error(
                        "Failed to send error signal for an invalid public key".to_string(),
                    );
                }
            }
            Notification::InvalidPrivateKeyEnter => {
                let message = "Invalid private key".to_string();
                let _ = self.logger.log_error(message.clone());
                if self
                    .tx_to_front
                    .send(SignalToFront::ErrorInAccountCreation(message))
                    .is_err()
                {
                    let _ = self.logger.log_error(
                        "Failed to send error signal for an invalid private key".to_string(),
                    );
                }
            }
            Notification::AccountCreationFail => {
                let message = "Error in account creation".to_string();
                let _ = self.logger.log_error(message.clone());
                if self
                    .tx_to_front
                    .send(SignalToFront::ErrorInAccountCreation(message))
                    .is_err()
                {
                    let _ = self.logger.log_error(
                        "Failed to send error signal for an error in creation of an account"
                            .to_string(),
                    );
                }
            }
        }
    }
}

/// Return the information of the transactions of an account
fn get_account_transactions_information(
    account: &Account,
    transactions: Vec<Transaction>,
) -> Vec<(u32, [u8; 32], i64)> {
    transactions
        .iter()
        .filter_map(|transaction| {
            let timestamp = transaction.time;
            let label = match transaction.get_tx_id() {
                Ok(txid) => txid,
                Err(_) => return None,
            };
            let mut amount: i64 = 0;
            for utxo in transaction.tx_out.clone() {
                if account.verify_transaction_output_ownership(&utxo) {
                    amount += utxo.value;
                }
            }
            Some((timestamp, label, amount))
        })
        .collect()
}
