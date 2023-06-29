use super::error_process::ErrorProcess;

use crate::ui::error_ui::ErrorUI;

use cargosos_bitcoin::{
    block_structure::{transaction::Transaction, utxo_set::UTXOSet},
    logs::logger_sender::LoggerSender,
    node_structure::{broadcasting::Broadcasting, error_node::ErrorNode},
    notifications::{notification::Notification, notifier::Notifier},
    wallet_structure::{
        account::Account, address::Address, error_wallet::ErrorWallet, wallet::Wallet,
    },
};

use std::io::{Read, Write};

/// FUnction that converts testnet bitcoins to satoshis
pub fn fron_tbtc_to_satoshi(tbtc: f64) -> i64 {
    (tbtc * 100_000_000.0) as i64
}

/// Creates a transaction given the user user_input
///
/// ### Error
///  * `ErrorUI::ErrorInTransaction`: It will appear when the user does not have enough funds to make the transaction or the transaction is not valid
fn create_transaction(
    utxo_set: &UTXOSet,
    account: &Account,
    logger: LoggerSender,
    address: &Address,
    amount: f64,
    fee: f64,
) -> Result<Transaction, ErrorProcess> {
    match account.create_transaction(
        address.clone(),
        fron_tbtc_to_satoshi(amount),
        fron_tbtc_to_satoshi(fee),
        &utxo_set,
    ) {
        Ok(transaction) => Ok(transaction),
        Err(ErrorWallet::NotEnoughFunds(error_string)) => {
            let _ = logger.log_wallet(format!(
                "Error creating transaction, with error: {:?}",
                ErrorWallet::NotEnoughFunds(error_string)
            ));
            Err(ErrorProcess::TransactionWithoutSufficientFunds)
        }
        Err(error) => {
            let _ = logger.log_wallet(format!(
                "Error creating transaction, with error: {:?}",
                error
            ));
            Err(ErrorProcess::TransactionCreationFail)
        }
    }
}

/// Broadcast the transaction created by the user to the peers from the selected account in the wallet
///
/// ### Error
///  * `ErrorUI::FailedSignalToFront`: It will appear when the sender fails
///  * `ErrorUI::CannotUnwrapArc`: It will appear when we try to unwrap an Arc
///  * `ErrorUI::ErrorFromPeer`: It will appear when a conextion with a peer fails
pub fn sending_transaction<N: Notifier, RW: Read + Write + Send + 'static>(
    broadcasting: &mut Broadcasting<RW>,
    wallet: &Wallet,
    utxo_set: &UTXOSet,
    address: Address,
    amount_fee: (f64, f64),
    notifier: N,
    logger: LoggerSender,
) -> Result<(), ErrorUI> {
    let amount = amount_fee.0;
    let fee = amount_fee.1;

    let account = match wallet.get_selected_account() {
        Some(account) => account,
        None => {
            let _ = logger.log_wallet("No account selected cannot send transaction".to_string());
            notifier.notify(Notification::AccountNotSelected);
            return Ok(());
        }
    };

    let transaction =
        match create_transaction(&utxo_set, account, logger.clone(), &address, amount, fee) {
            Ok(transaction) => transaction,
            Err(error) => {
                notifier.notify(Notification::NotEnoughFunds);
                return Err(error.into());
            }
        };

    let _ = logger.log_transaction("Sending transaction".to_string());
    utxo_set.append_pending_transaction(transaction.clone());

    match broadcasting.send_transaction(transaction) {
        Ok(()) => Ok(()),
        Err(ErrorNode::WhileSendingMessage(message)) => Err(ErrorUI::ErrorFromPeer(message)),
        _ => Err(ErrorUI::ErrorFromPeer(
            "While sending transaction".to_string(),
        )),
    }
}
