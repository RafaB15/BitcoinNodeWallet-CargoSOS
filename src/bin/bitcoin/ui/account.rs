use super::error_ui::ErrorUI;

use crate::process::reference::{MutArc, get_reference};

use cargosos_bitcoin::{
    wallet_structure::{
        account::Account,
        wallet::Wallet,
    },
    block_structure::{
        block_chain::BlockChain,
        transaction::Transaction,
        utxo_set::UTXOSet,
    },
    notifications::{
        notification::Notification,
        notifier::Notifier,
    },
    logs::logger_sender::LoggerSender,
};

/// Function that obtains the balance of the selected account and sends it to the front
pub fn give_account_balance<N : Notifier>(
    wallet: MutArc<Wallet>,
    utxo_set: MutArc<UTXOSet>,
    notifier: N,
) -> Result<(), ErrorUI> {
    let wallet_reference = get_reference(&wallet)?;
    let utxo_set_reference = get_reference(&utxo_set)?;

    let account_to_check = match wallet_reference.get_selected_account() {
        Some(account) => account,
        None => return Err(ErrorUI::ErrorReading("No account selected".to_string())),
    };
    let balance = utxo_set_reference.get_balance_in_tbtc(&account_to_check.address);
    let pending = utxo_set_reference.get_pending_in_tbtc(&account_to_check.address);

    notifier.notify(Notification::LoadAvailableBalance((balance, pending)));    

    Ok(())
}

/// Function that obtains and return the transactions of an account
fn get_account_transactions(
    account: &Account,
    blockchain: &BlockChain,
) -> Vec<Transaction> {
    let mut transactions: Vec<Transaction> = Vec::new();
    let blocks = blockchain.get_all_blocks();
    for block in blocks {
        for transaction in block.transactions {
            if account.verify_transaction_ownership(&transaction) {
                transactions.push(transaction);
            }
        }
    }
    transactions
}

/// Function that gets the information of the transactions of the selected account
/// and sends it to the front
pub fn give_account_transactions<N : Notifier>(
    wallet: MutArc<Wallet>,
    blockchain: MutArc<BlockChain>,
    logger: LoggerSender,
    notifier: N,
) -> Result<(), ErrorUI> {
    let wallet = get_reference(&wallet).unwrap();
    let blockchain = get_reference(&blockchain).unwrap();

    let account = match wallet.get_selected_account() {
        Some(account) => *account,
        None => {
            let _ = logger.log_wallet("No account selected cannot get transactions".to_string());
            notifier.notify(Notification::AccountNotSelected);
            return Ok(());
        }
    };

    let transactions = get_account_transactions(&account, &blockchain);
    notifier.notify(Notification::AccountTransactions((account, transactions)));

    Ok(())
}