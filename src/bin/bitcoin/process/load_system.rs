use crate::error_execution::ErrorExecution;

use cargosos_bitcoin::{
    block_structure::{block::Block, block_chain::BlockChain, block_header::BlockHeader},
    logs::logger_sender::LoggerSender,
    serialization::deserializable_internal_order::DeserializableInternalOrder,
    wallet_structure::wallet::Wallet,
    configurations::save_config::SaveConfig,
};

use std::{
    fs::OpenOptions,
    io::BufReader,
    thread::{self, JoinHandle},
    mem::replace,
};

type Handle<T> = Option<JoinHandle<T>>;

pub struct LoadSystem {
    block_chain: Handle<Result<BlockChain, ErrorExecution>>,
    wallet: Handle<Result<Wallet, ErrorExecution>>,
}

impl LoadSystem {
    
    pub fn new(
        save_config: SaveConfig,
        logger: LoggerSender,
    ) -> LoadSystem 
    {
        LoadSystem {
            block_chain: Some(Self::load_block_chain(save_config.read_block_chain, logger.clone())),
            wallet: Some(Self::load_wallet(save_config.read_wallet, logger)),
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

    pub fn get_wallet(&mut self) -> Result<Wallet, ErrorExecution> {
        let wallet_handle = replace(&mut self.wallet, None);

        if let Some(wallet_handle) = wallet_handle {
            return match wallet_handle.join() { 
                Ok(wallet) => wallet,
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

    fn load_wallet(
        path_wallet: Option<String>,
        logger: LoggerSender,
    ) -> JoinHandle<Result<Wallet, ErrorExecution>> {
        todo!()
    }
}



