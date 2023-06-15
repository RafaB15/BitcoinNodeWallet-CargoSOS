use crate::error_execution::ErrorExecution;

use cargosos_bitcoin::{
    logs::logger_sender::LoggerSender,
    wallet_structure::{
        account::Account, address::Address, private_key::PrivateKey, public_key::PublicKey,
        wallet::Wallet,
    },
};

use std::io::stdin;

/// Get the private key from the terminal
///
/// ### Error
///  * `ErrorExecution::TerminalReadFail`: It will appear when the terminal read fails
fn get_private_key(logger_sender: LoggerSender) -> Result<PrivateKey, ErrorExecution> {
    let mut private_key: String = String::new();

    println!("Enter the private key: ");
    if stdin().read_line(&mut private_key).is_err() {
        return Err(ErrorExecution::TerminalReadFail);
    }

    loop {
        let _: PrivateKey = match PrivateKey::try_from(private_key.trim().to_string()) {
            Ok(result) => return Ok(result),
            Err(error) => {
                logger_sender.log_wallet(format!(
                    "Put an invalid private key, with error: {:?}",
                    error
                ))?;

                private_key.clear();
                println!("Error, please enter a valid private key:");
                if stdin().read_line(&mut private_key).is_err() {
                    return Err(ErrorExecution::TerminalReadFail);
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
fn get_public_key(logger_sender: LoggerSender) -> Result<PublicKey, ErrorExecution> {
    let mut public_key: String = String::new();

    println!("Enter the public key: ");
    if stdin().read_line(&mut public_key).is_err() {
        return Err(ErrorExecution::TerminalReadFail);
    }

    loop {
        let _: PublicKey = match PublicKey::try_from(public_key.trim().to_string()) {
            Ok(result) => return Ok(result),
            Err(error) => {
                logger_sender.log_wallet(format!(
                    "Put an invalid public key, with error: {:?}",
                    error
                ))?;

                public_key.clear();
                println!("Error, please enter a valid public key:");
                if stdin().read_line(&mut public_key).is_err() {
                    return Err(ErrorExecution::TerminalReadFail);
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
fn get_address(logger_sender: LoggerSender) -> Result<Address, ErrorExecution> {
    let mut address: String = String::new();

    println!("Enter the address: ");
    if stdin().read_line(&mut address).is_err() {
        return Err(ErrorExecution::TerminalReadFail);
    }

    loop {
        match Address::new(address.trim()) {
            Ok(result) => return Ok(result),
            Err(error) => {
                logger_sender.log_wallet(format!(
                    "Put an invalid public key, with error: {:?}",
                    error
                ))?;

                address.clear();
                println!("Error, please enter a valid address:");
                if stdin().read_line(&mut address).is_err() {
                    return Err(ErrorExecution::TerminalReadFail);
                }
                continue;
            }
        };
    }
}

/// Get the account name from the terminal
fn get_account_name() -> Result<String, ErrorExecution> {
    let mut name: String = String::new();

    println!("Enter the name: ");
    match stdin().read_line(&mut name) {
        Ok(_) => Ok(name.trim().to_string()),
        Err(_) => Err(ErrorExecution::TerminalReadFail),
    }
}

/// Creates a new account with the data entered by the user
///
/// ### Error
///  * `ErrorExecution::TerminalReadFail`: It will appear when the terminal read fails
pub fn create_account(logger: LoggerSender) -> Result<Account, ErrorExecution> {
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

/// Select an account from the wallet
pub fn select_account(wallet: &Wallet, logger: LoggerSender) -> Account {
    todo!()
}

/// Show all accounts from the wallet
pub fn show_accounts(wallet: &Wallet, logger: LoggerSender) {

}
