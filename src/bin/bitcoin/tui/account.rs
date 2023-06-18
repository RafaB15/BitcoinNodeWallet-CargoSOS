use super::error_tui::ErrorTUI;

use cargosos_bitcoin::{
    logs::logger_sender::LoggerSender,
    wallet_structure::{
        account::Account, address::Address, private_key::PrivateKey, public_key::PublicKey,
        wallet::Wallet,
    },
};

use std::{io::stdin, sync::MutexGuard};

/// Get the private key from the terminal
///
/// ### Error
///  * `ErrorExecution::TerminalReadFail`: It will appear when the terminal read fails
fn get_private_key(logger: LoggerSender) -> Result<PrivateKey, ErrorTUI> {
    let mut private_key: String = String::new();

    println!("Enter the private key: ");
    if stdin().read_line(&mut private_key).is_err() {
        return Err(ErrorTUI::TerminalReadFail);
    }

    loop {
        let _: PrivateKey = match PrivateKey::try_from(private_key.trim().to_string()) {
            Ok(result) => return Ok(result),
            Err(error) => {
                let _ = logger.log_wallet(format!(
                    "Put an invalid private key, with error: {:?}",
                    error
                ));

                private_key.clear();
                println!("Error, please enter a valid private key:");
                if stdin().read_line(&mut private_key).is_err() {
                    return Err(ErrorTUI::TerminalReadFail);
                }
                continue;
            }
        };
    }
}

/// Get the public key from the terminal
///
/// ### Error
///  * `ErrorExecution::TerminalReadFail`: It will appear when the terminal read fails
fn get_public_key(logger: LoggerSender) -> Result<PublicKey, ErrorTUI> {
    let mut public_key: String = String::new();

    println!("Enter the public key: ");
    if stdin().read_line(&mut public_key).is_err() {
        return Err(ErrorTUI::TerminalReadFail);
    }

    loop {
        let _: PublicKey = match PublicKey::try_from(public_key.trim().to_string()) {
            Ok(result) => return Ok(result),
            Err(error) => {
                let _ = logger.log_wallet(format!(
                    "Put an invalid public key, with error: {:?}",
                    error
                ));

                public_key.clear();
                println!("Error, please enter a valid public key:");
                if stdin().read_line(&mut public_key).is_err() {
                    return Err(ErrorTUI::TerminalReadFail);
                }

                continue;
            }
        };
    }
}

/// Get the address from the terminal
///
/// ### Error
///  * `ErrorExecution::TerminalReadFail`: It will appear when the terminal read fails
fn get_address(logger: LoggerSender) -> Result<Address, ErrorTUI> {
    let mut address: String = String::new();

    println!("Enter the address: ");
    if stdin().read_line(&mut address).is_err() {
        return Err(ErrorTUI::TerminalReadFail);
    }

    loop {
        match Address::new(address.trim()) {
            Ok(result) => return Ok(result),
            Err(error) => {
                let _ = logger.log_wallet(format!(
                    "Put an invalid public key, with error: {:?}",
                    error
                ));

                address.clear();
                println!("Error, please enter a valid address:");
                if stdin().read_line(&mut address).is_err() {
                    return Err(ErrorTUI::TerminalReadFail);
                }
                continue;
            }
        };
    }
}

/// Get the account name from the terminal
fn get_account_name() -> Result<String, ErrorTUI> {
    let mut name: String = String::new();

    println!("Enter the name: ");
    match stdin().read_line(&mut name) {
        Ok(_) => Ok(name.trim().to_string()),
        Err(_) => Err(ErrorTUI::TerminalReadFail),
    }
}

/// Creates a new account with the data entered by the user
///
/// ### Error
///  * `ErrorExecution::TerminalReadFail`: It will appear when the terminal read fails
pub fn create_account(logger: LoggerSender) -> Result<Account, ErrorTUI> {
    let _ = logger.log_wallet("Creating a new account".to_string());

    let account = Account {
        private_key: get_private_key(logger.clone())?,
        public_key: get_public_key(logger.clone())?,
        address: get_address(logger.clone())?,
        account_name: get_account_name()?,
    };

    let _ = logger.log_wallet("Account created successfully!".to_string());

    Ok(account)
}

/// get an account from the wallet with the corresponding name
fn get_account_from_name<'t>(
    account_name: &str,
    wallet: &MutexGuard<'t, Wallet>,
) -> Option<Account> {
    for account in wallet.accounts.iter() {
        if account.account_name == account_name {
            return Some(account.clone());
        }
    }

    None
}

/// Select an account from the wallet
pub fn select_account<'t>(
    wallet: &MutexGuard<'t, Wallet>,
    logger: LoggerSender,
) -> Result<Account, ErrorTUI> {
    let _ = logger.log_wallet("Selecting an account".to_string());

    println!("Possible accounts: ");
    show_accounts(wallet, logger.clone());

    let mut account_name: String = String::new();

    println!("Enter the address: ");
    if stdin().read_line(&mut account_name).is_err() {
        return Err(ErrorTUI::TerminalReadFail);
    }

    loop {
        match get_account_from_name(account_name.trim(), wallet) {
            Some(account) => return Ok(account),
            None => {
                let _ = logger.log_wallet(format!("Put an invalid account name"));

                account_name.clear();
                println!("Error, please enter a valid account name:");
                if stdin().read_line(&mut account_name).is_err() {
                    return Err(ErrorTUI::TerminalReadFail);
                }
                continue;
            }
        };
    }
}

/// Show all accounts from the wallet
pub fn show_accounts<'t>(wallet: &MutexGuard<'t, Wallet>, logger: LoggerSender) {
    let _ = logger.log_wallet("Showing accounts".to_string());

    wallet
        .accounts
        .iter()
        .for_each(|account| println!("{account}"));
}
