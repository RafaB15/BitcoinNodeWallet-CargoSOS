use crate::{error_execution::ErrorExecution, error_initialization::ErrorInitialization};

use cargosos_bitcoin::{
    block_structure::block_chain::BlockChain,
    logs::logger_sender::LoggerSender,
    serialization::serializable_internal_order::SerializableInternalOrder,
};

use std::fs::OpenOptions;

pub struct SaveSystem {
    block_chain: BlockChain,
    logger: LoggerSender,
}

impl SaveSystem {
    pub fn new(block_chain: BlockChain, logger: LoggerSender) -> SaveSystem {
        SaveSystem { block_chain, logger }
    }

    pub fn save_to_files(
        self,
        posible_path: Option<String>,
    ) -> Result<(), ErrorExecution> {
        save_block_chain(&self.block_chain, posible_path, self.logger)
    }
}

/// Saves the blockchain to a file
///
/// ### Error
///  * `ErrorInitialization::BlockchainFileDoesntExist`: It will appear when the file could not be created
///  * `ErrorSerialization::ErrorInSerialization`: It will appear when the serialization of the blockchain fails
fn save_block_chain(
    block_chain: &BlockChain,
    posible_path: Option<String>,
    logger: LoggerSender,
) -> Result<(), ErrorExecution> {
    let path = match posible_path {
        Some(path) => path,
        None => {
            let _ = logger.log_connection("No path to save the blockchain".to_string());
            return Ok(());
        }
    };

    let mut file = match OpenOptions::new().create(true).write(true).open(path) {
        Ok(file) => file,
        _ => return Err(ErrorInitialization::BlockchainFileDoesntExist.into()),
    };

    let _ = logger.log_connection("Writing the blockchain to file".to_string());

    block_chain.io_serialize(&mut file)?;

    Ok(())
}