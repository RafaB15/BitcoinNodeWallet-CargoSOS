use super::error_ui::ErrorUI;

use cargosos_bitcoin::{
    wallet_structure::{
        account::Account,
        wallet::Wallet, private_key::PrivateKey, public_key::PublicKey,
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
    wallet: &Wallet,
    utxo_set: &UTXOSet,
    notifier: N,
) -> Result<(), ErrorUI> {
    let account_to_check = match wallet.get_selected_account() {
        Some(account) => account,
        None => return Err(ErrorUI::ErrorReading("No account selected".to_string())),
    };
    let balance = utxo_set.get_balance_in_tbtc(&account_to_check.address);
    let pending = utxo_set.get_pending_in_tbtc(&account_to_check.address);

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

/// Function that changes the selected account of the address
pub fn change_selected_account<N : Notifier>(
    account_name: String,
    wallet: &Wallet,
    notifier: N,
) -> Result<(), ErrorUI> {
    let account_to_select = match wallet.get_account_with_name(&account_name) {
        Some(account) => account.clone(),
        None => return Err(ErrorUI::ErrorReading("Account does not exist".to_string())),
    };

    wallet.change_account(account_to_select.clone());

    notifier.notify(Notification::UpdatedSelectedAccount(account_to_select));

    Ok(())
}

pub fn create_account<N : Notifier>(
    wallet: &Wallet,
    account_name: &str,
    private_key: PrivateKey,
    public_key: PublicKey,
    notifier: N,
) -> Result<(), ErrorUI> {
    let account = match Account::from_keys(
        account_name,
        private_key,
        public_key,
    ) {
        Ok(account) => account,
        _ => {
            notifier.notify(Notification::AccountCreationFail);
            return Ok(());
        }
    };

    wallet.add_account(account.clone());
    notifier.notify(Notification::RegisterWalletAccount(account));

    Ok(())
}

/// Function that gets the information of the transactions of the selected account
/// and sends it to the front
pub fn give_account_transactions<N : Notifier>(
    wallet: &Wallet,
    blockchain: &BlockChain,
    logger: LoggerSender,
    notifier: N,
) -> Result<(), ErrorUI> {
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