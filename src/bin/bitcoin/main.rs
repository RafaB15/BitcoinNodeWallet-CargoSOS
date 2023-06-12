mod error_execution;
mod error_initialization;
mod process;
mod tui;
mod gui;

use std::{
    fs::{File, OpenOptions},
    io::BufReader,
    path::Path,
    thread::{self, JoinHandle},
};

use error_execution::ErrorExecution;
use error_initialization::ErrorInitialization;
use process::{
    configuration::Configuration, 
    load_system::LoadSystem, 
};

use cargosos_bitcoin::{
    logs::{error_log::ErrorLog, logger, logger_sender::LoggerSender},
    configurations::{
        log_config::LogConfig,
        interface::Interface,
    },
    block_structure::block_chain::BlockChain,
};

/// Get the configuration name given the arguments
///
/// ### Errors
///  * `ErrorNoGivenFile`: It will appear when there is not argument pass that configuration declaration
fn get_config_name(arguments: Vec<String>) -> Result<String, ErrorInitialization> {
    let config_name: String = match arguments.get(1) {
        Some(config_name) => config_name.to_owned(),
        None => return Err(ErrorInitialization::NoGivenConfigurationFile),
    };

    Ok(config_name)
}

/// Get the file given by its name
///
/// ### Errors
/// * `ErrorFileNotExist`: It will appear when the file does not exist
fn open_config_file(config_name: String) -> Result<BufReader<File>, ErrorInitialization> {
    let config_file = match File::open(config_name) {
        Ok(config_file) => config_file,
        Err(_) => return Err(ErrorInitialization::ConfigurationFileDoesntExist),
    };

    Ok(BufReader::new(config_file))
}

/// Get the file given by its path. If the file does not exist, it will be created. Evrytime the file is opened, it will be truncated to set the file size to 0 and overwrite the previous content
///
/// ### Errors
/// * `ErrorFileNotExist`: It will appear when the file does not exist
/// * `CouldNotTruncateFile`: It will appear when the file could not be truncated
fn open_log_file(log_path: &Path) -> Result<File, ErrorInitialization> {
    let log_file = match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
    {
        Ok(log_file) => log_file,
        _ => return Err(ErrorInitialization::LogFileDoesntExist),
    };

    Ok(log_file)
}

/// Initialize the logs ready for ejecution
///
/// ### Errors
///  * `ErrorCouldNotFindReceiver`: No se encontro el receiver
///  * `ReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
///  * `ErrorLogFileDoesNotExist`: No se encontro el archivo de logs
///  * `CouldNotTruncateFile`: No se pudo truncar el archivo de logs
fn initialize_logs(
    log_config: LogConfig,
) -> Result<(JoinHandle<Result<(), ErrorLog>>, LoggerSender), ErrorExecution> {
    println!("Creating the logs system");

    let filepath_log = Path::new(&log_config.filepath_log);
    let log_file = open_log_file(filepath_log)?;
    let (logger, logger_receiver) =
        logger::initialize_logger(log_file, log_config.show_console)?;

    let handle = thread::spawn(move || logger_receiver.receive_log());

    logger.log_configuration("Logs are already configured".to_string())?;

    Ok((handle, logger))
}





fn _show_merkle_path(
    block_chain: &BlockChain,
    logger: LoggerSender,
) -> Result<(), ErrorExecution> {
    let latest = block_chain.latest();

    let last_block = match latest.last() {
        Some(last_block) => last_block,
        None => {
            return Err(ErrorExecution::ErrorBlock(
                "Last block not found".to_string(),
            ))
        }
    };

    logger.log_connection(format!(
        "With the block with header: \n{:?}",
        last_block.header,
    ))?;

    let transaction_position =
        std::cmp::min::<u64>(6, last_block.header.transaction_count.value - 1);

    let transaction = match last_block.transactions.get(transaction_position as usize) {
        Some(transaction) => transaction,
        None => {
            return Err(ErrorExecution::ErrorBlock(
                "Transaction not found".to_string(),
            ))
        }
    };

    logger.log_connection(format!("And transaction: \n{:?}", transaction,))?;

    let merkle_path = last_block.get_merkle_path(transaction)?;

    let mut path: String = "\n".to_string();
    for hash in merkle_path {
        path = format!("{path}\t{:?}\n", hash);
    }

    logger.log_connection(format!("We get the merkle path: {path}"))?;

    Ok(())
}

fn main() -> Result<(), ErrorExecution> {
    let arguments: Vec<String> = std::env::args().collect();

    println!("\tInitialization");
    println!("Reading the configuration file");

    let config_name: String = get_config_name(arguments)?;
    let config_file = open_config_file(config_name)?;

    let configuration = Configuration::new(config_file)?;
    let (log_config, 
        connection_config, 
        download_config, 
        save_config,
        ui_config,
    ) = configuration.separate();

    let (handle, logger) = initialize_logs(log_config)?;

    {
        let mut load_system = LoadSystem::new(
            save_config.clone(),
            logger.clone(),
        );

        let save_system = match ui_config.interface {
            Interface::Tui => {
                tui::execution::program_execution(
                    connection_config,
                    download_config,
                    &mut load_system,
                    logger.clone(),
                )?
            },
            Interface::Gui => {
                gui::execution::program_execution()?
            },
        };     
    
        save_system.save_to_files(save_config)?;
    }

    logger.log_configuration("Closing program".to_string())?;

    std::mem::drop(logger);
    match handle.join() {
        Ok(result) => result?,
        _ => return Err(ErrorExecution::FailThread),
    }

    Ok(())
}
