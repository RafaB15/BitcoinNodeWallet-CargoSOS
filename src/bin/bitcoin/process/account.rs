use crate::error_execution::ErrorExecution;

use cargosos_bitcoin::{
    logs::logger_sender::LoggerSender,
    wallet_structure::{
        account::Account,
        private_key::PrivateKey,
        public_key::PublicKey,
        address::Address,
    },
};

use std::io::stdin;

pub fn get_private_key(logger_sender: LoggerSender) -> Result<PrivateKey, ErrorExecution> {
    let mut private_key: String = String::new();

    print!("Enter the private key: ");
    let mut private_key_result = match stdin().read_line(&mut private_key) {
        Ok(_) => private_key.trim().to_string(),
        Err(_) => return Err(ErrorExecution::TerminalReadFail),
    };

    loop {
        let _: PrivateKey = match PrivateKey::try_from(private_key_result) {
            Ok(result) => return Ok(result),
            Err(error) => {
                logger_sender.log_wallet(format!(
                    "Put an invalid private key, with error: {:?}",
                    error
                ))?;

                print!("Error, please enter a valid private key:");
                private_key_result = match stdin().read_line(&mut private_key) {
                    Ok(_) => private_key.trim().to_string(),
                    Err(_) => return Err(ErrorExecution::TerminalReadFail),
                };
                continue;
            }
        };
    }
}

pub fn get_public_key(logger_sender: LoggerSender) -> Result<PublicKey, ErrorExecution> {
    let mut public_key: String = String::new();

    print!("Enter the public key: ");
    let mut public_key_result = match stdin().read_line(&mut public_key) {
        Ok(_) => public_key.trim().to_string(),
        Err(_) => return Err(ErrorExecution::TerminalReadFail),
    };

    loop {
        let _: PublicKey = match PublicKey::try_from(public_key_result) {
            Ok(result) => return Ok(result),
            Err(error) => {
                logger_sender.log_wallet(format!(
                    "Put an invalid public key, with error: {:?}",
                    error
                ))?;

                print!("Error, please enter a valid public key:");
                public_key_result = match stdin().read_line(&mut public_key) {
                    Ok(_) => public_key.trim().to_string(),
                    Err(_) => return Err(ErrorExecution::TerminalReadFail),
                };
                continue;
            }
        };
    }
}

pub fn get_address(logger_sender: LoggerSender) -> Result<Address, ErrorExecution> {
    let mut address: String = String::new();
    
    print!("Enter the address: ");
    let mut address_result = match stdin().read_line(&mut address) {
        Ok(_) => address.trim(),
        Err(_) => return Err(ErrorExecution::TerminalReadFail),
    };

    loop {
        match Address::new(address_result) {
            Ok(result) => return Ok(result),
            Err(error) => {
                logger_sender.log_wallet(format!(
                    "Put an invalid public key, with error: {:?}",
                    error
                ))?;

                print!("Error, please enter a valid address:");
                address_result = match stdin().read_line(&mut address) {
                    Ok(_) => address.trim(),
                    Err(_) => return Err(ErrorExecution::TerminalReadFail),
                };
                continue;
            }
        };
    }
}

pub fn get_account_name() -> Result<String, ErrorExecution> {
    let mut name: String = String::new();

    print!("Enter the name: ");
    match stdin().read_line(&mut name) {
        Ok(_) => Ok(name.trim().to_string()),
        Err(_) => Err(ErrorExecution::TerminalReadFail),
    }
}

pub fn add_account(logger_sender: LoggerSender) -> Result<Account, ErrorExecution> {
    Ok(Account { 
        private_key: get_private_key(logger_sender.clone())?, 
        public_key: get_public_key(logger_sender.clone())?, 
        address: get_address(logger_sender)?,
        account_name: get_account_name()?, 
    })
}