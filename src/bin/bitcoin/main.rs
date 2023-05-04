mod initialization_error;

use initialization_error::InitializationError;
use std::io::BufReader;
use std::fs::File;
use cargosos_bitcoin::configurations::configuration::config;

const DECLARATION_LITTLE_CONFIG: &str = "-c";
const DECLARATION_CONFIG: &str = "--config";
const DECLARATION_BIG_CONFIG: &str = "--configuration";

/// Finds the position of the declaration of the configuration given by  `-c`, `--config` or `--configuration`
/// 
///  * `ErrorNoGivenFile`: It will appear when there is not `-c`, `--config` or `--configuration` in the arguments
fn find_config_name(arguments: &Vec<String>) -> Result<usize, InitializationError> {
    
    let config_declarations = &[DECLARATION_LITTLE_CONFIG.to_string(), 
        DECLARATION_CONFIG.to_string(), 
        DECLARATION_BIG_CONFIG.to_string()
    ];

    for (index, argument) in arguments.iter().enumerate() {
        if config_declarations.contains(argument) {
            return Ok(index);
        }
    }

    Err(InitializationError::ErrorNoGivenFile)
}

/// Get the configuration name given the arguments 
/// 
/// ### Errors
///  * `ErrorNoGivenFile`: It will appear when there is not `-c`, `--config` or `--configuration` in the arguments or there is not argument pass that configuration declaration
fn get_config_name(arguments: Vec<String>) -> Result<String, InitializationError> {
    
    let config_position: usize = find_config_name(&arguments)?;

    let config_name: String = match arguments.get(config_position + 1) {
        Some(config_name) => config_name.to_owned(),
        None => return Err(InitializationError::ErrorNoGivenFile),
    };

    Ok(config_name)
}

/// Get the file given by its name
/// 
/// ### Errors
/// * `ErrorFileNotExist`: It will appear when the file does not exist
fn open_config_file(config_name: String) -> Result<BufReader<File>, InitializationError> {
    let config_file = match File::open(config_name) {
        Ok(config_file) => config_file,
        Err(_) => return Err(InitializationError::ErrorFileNotExist),
    };

    Ok(BufReader::new(config_file))   
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();

    println!("Initialization");

    println!("Reading the configuration file");

    let config_name: String = match get_config_name(arguments) {
        Ok(config_name) => config_name,
        Err(err) => return println!("{err}"),
    };

    let config_file = match open_config_file(config_name) {
        Ok(config_file) => config_file,
        Err(err) => return println!("{err}"),
    };

    let (log_config, connection_config) = match config::new(config_file) {
        Ok(configuration) => configuration,
        Err(err) => {
            let error: InitializationError = err.into();
            return println!("{error}");
        },
    };

    println!("Creating the logs system");


    
}