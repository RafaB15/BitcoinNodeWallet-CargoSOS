use crate::{error_execution::ErrorExecution, error_initialization::ErrorInitialization};

use cargosos_bitcoin::{
    block_structure::block_chain::BlockChain,
    logs::logger_sender::LoggerSender,
    serialization::serializable_internal_order::SerializableInternalOrder,
    wallet_structure::wallet::Wallet,
    configurations::save_config::SaveConfig,
};

use std::fs::OpenOptions;

const BLOCKCHAIN_FILE: &str = "blockchain";
const WALLET_FILE: &str = "wallet";

pub struct SaveSystem {
    block_chain: BlockChain,
    wallet: Wallet,
    logger: LoggerSender,
}

impl SaveSystem {
    pub fn new(block_chain: BlockChain, wallet: Wallet, logger: LoggerSender) -> SaveSystem {
        SaveSystem { block_chain, wallet, logger }
    }

    pub fn save_to_files(
        self,
        save_config: SaveConfig,
    ) -> Result<(), ErrorExecution> {
        Self::save_value(
            self.block_chain,
            BLOCKCHAIN_FILE,
            save_config.write_block_chain, 
            self.logger.clone()
        )?;
        
        Self::save_value(
            self.wallet, 
            WALLET_FILE,
            save_config.write_wallet, 
            self.logger
        )?;

        Ok(())
    }

    /// Saves the blockchain to a file
    ///
    /// ### Error
    ///  * `ErrorInitialization::ValueFileDoesntExist`: It will appear when the file could not be created
    ///  * `ErrorSerialization::ErrorInSerialization`: It will appear when the serialization of the value fails
    fn save_value<V: SerializableInternalOrder>(
        value: V,
        name: &str,
        path: Option<String>,
        logger: LoggerSender,
    ) -> Result<(), ErrorExecution>
    {
        let path = match path {
            Some(path) => path,
            None => {
                let _ = logger.log_file(format!("No path to save the {name}"));
                return Ok(());
            }
        };

        let mut file = match OpenOptions::new().create(true).write(true).open(path) {
            Ok(file) => file,
            _ => return Err(ErrorInitialization::ValueFileDoesntExist.into()),
        };

        let _ = logger.log_file(format!("Writing the {name} to file"));

        value.io_serialize(&mut file)?;

        Ok(())
    }

    /*
    /// Saves the blockchain to a file
    ///
    /// ### Error
    ///  * `ErrorInitialization::BlockchainFileDoesntExist`: It will appear when the file could not be created
    ///  * `ErrorSerialization::ErrorInSerialization`: It will appear when the serialization of the blockchain fails
    fn save_block_chain(
        block_chain: BlockChain,
        path_block_chain: Option<String>,
        logger: LoggerSender,
    ) -> Result<(), ErrorExecution> {
        let path = match path_block_chain {
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

    fn save_wallet(
        wallet: Wallet,
        path_wallet: Option<String>,
        logger: LoggerSender,
    ) -> Result<(), ErrorExecution> {
        let path = match path_wallet {
            Some(path) => path,
            None => {
                let _ = logger.log_connection("No path to save the wallet".to_string());
                return Ok(());
            }
        };

        let mut file = match OpenOptions::new().create(true).write(true).open(path) {
            Ok(file) => file,
            _ => return Err(ErrorInitialization::BlockchainFileDoesntExist.into()),
        };

        let _ = logger.log_connection("Writing the wallet to file".to_string());

        wallet.io_serialize(&mut file)?;

        Ok(())
    }*/
}