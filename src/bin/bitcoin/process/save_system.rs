use crate::{error_execution::ErrorExecution, error_initialization::ErrorInitialization};

use cargosos_bitcoin::{
    block_structure::{block::Block, block_chain::BlockChain, block_header::BlockHeader},
    logs::logger_sender::LoggerSender,
    serialization::{
        deserializable_internal_order::DeserializableInternalOrder,
        serializable_internal_order::SerializableInternalOrder,
    },
};

use std::{
    fs::OpenOptions,
    io::BufReader,
    thread::{self, JoinHandle},
};

/// Saves the blockchain to a file
///
/// ### Error
///  * `ErrorInitialization::BlockchainFileDoesntExist`: It will appear when the file could not be created
///  * `ErrorSerialization::ErrorInSerialization`: It will appear when the serialization of the blockchain fails
pub fn save_block_chain(
    block_chain: &BlockChain,
    posible_path: Option<String>,
    logger_sender: LoggerSender,
) -> Result<(), ErrorExecution> {
    let path = match posible_path {
        Some(path) => path,
        None => {
            let _ = logger_sender.log_connection("No path to save the blockchain".to_string());
            return Ok(());
        }
    };

    let mut file = match OpenOptions::new().create(true).write(true).open(path) {
        Ok(file) => file,
        _ => return Err(ErrorInitialization::BlockchainFileDoesntExist.into()),
    };

    let _ = logger_sender.log_connection("Writing the blockchain to file".to_string());

    block_chain.io_serialize(&mut file)?;

    Ok(())
}

/// Loads the blockchain from a file and returns a handle of the thread loading the blockchain
///
/// ### Error
///  * `ErrorBlock::CouldNotHash`: It will appear when the hash of the block is not correct while creating the block chain
///  * `ErrorSerialization::ErrorInDeserialization`: It will appear when the deserialization of the blockchain fails
pub fn load_block_chain(
    posible_path: Option<String>,
    logger_sender: LoggerSender,
) -> JoinHandle<Result<BlockChain, ErrorExecution>> {
    thread::spawn(move || {
        if let Some(path) = posible_path {
            if let Ok(file) = OpenOptions::new().read(true).open(path) {
                let mut file = BufReader::new(file);

                let _ =
                    logger_sender.log_connection("Reading the blockchain from file".to_string());

                let block_chain = BlockChain::io_deserialize(&mut file)?;

                let _ = logger_sender.log_connection("Blockchain loaded from file".to_string());

                return Ok(block_chain);
            }

            let _ = logger_sender.log_connection("Could not open file".to_string());
        }

        let genesis_header: BlockHeader = BlockHeader::generate_genesis_block_header();
        let genesis_block: Block = Block::new(genesis_header);

        let _ =
            logger_sender.log_connection("Initializing blockchain from genesis block".to_string());

        Ok(BlockChain::new(genesis_block)?)
    })
}
