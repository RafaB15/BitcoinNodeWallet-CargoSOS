use crate::{error_execution::ErrorExecution, error_initialization::ErrorInitialization};

use cargosos_bitcoin::{
    block_structure::block_chain::BlockChain, configurations::save_config::SaveConfig,
    logs::logger_sender::LoggerSender,
    serialization::serializable_internal_order::SerializableInternalOrder,
    wallet_structure::wallet::Wallet,
};

use std::fs::OpenOptions;

const BLOCKCHAIN_FILE: &str = "Blockchain";
const WALLET_FILE: &str = "Wallet";

/// Represents the elements to save to files
pub struct SaveSystem {
    block_chain: BlockChain,
    wallet: Wallet,
    logger: LoggerSender,
}

impl SaveSystem {
    pub fn new(block_chain: BlockChain, wallet: Wallet, logger: LoggerSender) -> SaveSystem {
        SaveSystem {
            block_chain,
            wallet,
            logger,
        }
    }

    /// Saves the block chain and a wallet to there respective files if given
    ///
    /// ### Error
    ///  * `ErrorInitialization::ValueFileDoesntExist`: It will appear when the file could not be created
    ///  * `ErrorSerialization::ErrorInSerialization`: It will appear when the serialization of the value fails
    pub fn save_to_files(self, save_config: SaveConfig) -> Result<(), ErrorExecution> {
        Self::save_value(
            self.block_chain,
            BLOCKCHAIN_FILE,
            save_config.write_block_chain,
            self.logger.clone(),
        )?;

        Self::save_value(
            self.wallet,
            WALLET_FILE,
            save_config.write_wallet,
            self.logger,
        )?;

        Ok(())
    }

    /// Saves a serializable to a file
    ///
    /// ### Error
    ///  * `ErrorInitialization::ValueFileDoesntExist`: It will appear when the file could not be created
    ///  * `ErrorSerialization::ErrorInSerialization`: It will appear when the serialization of the value fails
    fn save_value<V: SerializableInternalOrder>(
        value: V,
        name: &str,
        path: Option<String>,
        logger: LoggerSender,
    ) -> Result<(), ErrorExecution> {
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
}
