use crate::error_execution::ErrorExecution;

use cargosos_bitcoin::{
    block_structure::{block::Block, block_chain::BlockChain, block_header::BlockHeader},
    logs::logger_sender::LoggerSender,
    serialization::deserializable_internal_order::DeserializableInternalOrder
};

use std::{
    fs::OpenOptions,
    io::BufReader,
    thread::{self, JoinHandle},
    mem::replace,
};

pub struct LoadSystem {
    block_chain: Option<JoinHandle<Result<BlockChain, ErrorExecution>>>,
}

impl LoadSystem {
    
    pub fn new(
        posible_path: Option<String>,
        logger: LoggerSender,
    ) -> LoadSystem 
    {
        LoadSystem {
            block_chain: Some(Self::load_block_chain(posible_path, logger)),
        }
    }

    pub fn get_block_chain(&mut self) -> Result<BlockChain, ErrorExecution> {
        let block_chain_handle = replace(&mut self.block_chain, None);

        if let Some(block_chain_handle) = block_chain_handle {
            return match block_chain_handle.join() { 
                Ok(block_chain) => block_chain,
                _ => Err(ErrorExecution::FailThread),
            };
        }

        Err(ErrorExecution::FailThread)
    }

    /// Loads the blockchain from a file and returns a handle of the thread loading the blockchain
    ///
    /// ### Error
    ///  * `ErrorBlock::CouldNotHash`: It will appear when the hash of the block is not correct while creating the block chain
    ///  * `ErrorSerialization::ErrorInDeserialization`: It will appear when the deserialization of the blockchain fails
    fn load_block_chain(
        path_block_chain: Option<String>,
        logger: LoggerSender,
    ) -> JoinHandle<Result<BlockChain, ErrorExecution>> {
        thread::spawn(move || {
            if let Some(path) = path_block_chain {
                if let Ok(file) = OpenOptions::new().read(true).open(path) {
                    let mut file = BufReader::new(file);
    
                    let _ =
                        logger.log_connection("Reading the blockchain from file".to_string());
    
                    let block_chain = BlockChain::io_deserialize(&mut file)?;
    
                    let _ = logger.log_connection("Blockchain loaded from file".to_string());
    
                    return Ok(block_chain);
                }
    
                let _ = logger.log_connection("Could not open file".to_string());
            }
    
            let genesis_header: BlockHeader = BlockHeader::generate_genesis_block_header();
            let genesis_block: Block = Block::new(genesis_header);
    
            let _ =
                logger.log_connection("Initializing blockchain from genesis block".to_string());
    
            Ok(BlockChain::new(genesis_block)?)
        })
    }
}



