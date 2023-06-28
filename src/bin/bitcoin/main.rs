mod error_execution;
mod error_initialization;
mod gui;
mod process;
mod tui;
mod ui;

use std::{
    fs::{File, OpenOptions},
    io::BufReader,
    path::Path,
    thread::{self, JoinHandle},
};

use error_execution::ErrorExecution;
use error_initialization::ErrorInitialization;
use process::{configuration::Configuration, load_system::LoadSystem, save_system::SaveSystem};

use cargosos_bitcoin::{
    configurations::{interface::Interface, log_config::LogConfig, save_config::SaveConfig},
    logs::{error_log::ErrorLog, logger, logger_sender::LoggerSender},
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
    let (logger, logger_receiver) = logger::initialize_logger(log_file, log_config.show_console);

    let handle = thread::spawn(move || logger_receiver.receive_log());

    logger.log_configuration("Logs are already configured".to_string())?;

    Ok((handle, logger))
}

fn end_program(
    save_system: SaveSystem,
    save_config: SaveConfig,
    logger: LoggerSender,
) -> Result<(), ErrorExecution> {
    save_system.save_to_files(save_config)?;

    let _ = logger.log_configuration("Closing program".to_string());

    Ok(())
}

fn main() -> Result<(), ErrorExecution> {
    let arguments: Vec<String> = std::env::args().collect();

    println!("\tInitialization");
    println!("Reading the configuration file");

    let config_name: String = get_config_name(arguments)?;
    let config_file = open_config_file(config_name)?;

    let configuration = Configuration::new(config_file)?;
    let (log_config, connection_config, download_config, save_config, ui_config, mode_config) =
        configuration.separate();

    let (handle, logger) = initialize_logs(log_config)?;

    let save_system = match ui_config.interface {
        Interface::Tui => {
            let mut load_system = LoadSystem::new(save_config.clone(), logger.clone());
            tui::execution::program_execution(
                mode_config,
                connection_config,
                download_config,
                &mut load_system,
                logger.clone(),
            )?
        }
        Interface::Gui => gui::execution::program_execution(
            mode_config,
            connection_config,
            download_config,
            save_config.clone(),
            logger.clone(),
        )?,
    };

    end_program(save_system, save_config, logger)?;

    match handle.join() {
        Ok(result) => result?,
        _ => return Err(ErrorExecution::FailThread),
    }

    Ok(())
}
