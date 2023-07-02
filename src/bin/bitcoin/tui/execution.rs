use super::{notifier_tui::NotifierTUI, input_handler_tui::InputHandlerTUI};

use crate::{
    error_execution::ErrorExecution,
    process::{load_system::LoadSystem, save_system::SaveSystem, backend},
};

use cargosos_bitcoin::{
    block_structure::block_chain::BlockChain,
    configurations::{
        connection_config::ConnectionConfig, download_config::DownloadConfig,
        mode_config::ModeConfig,
    },
    logs::{logger_sender::LoggerSender},
};


fn _show_merkle_path(block_chain: &BlockChain, logger: LoggerSender) -> Result<(), ErrorExecution> {
    let latest = block_chain.latest();

    let last_block = match latest.last() {
        Some(last_block) => last_block,
        None => {
            return Err(ErrorExecution::_ErrorBlock(
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
            return Err(ErrorExecution::_ErrorBlock(
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

pub fn program_execution(
    mode_config: ModeConfig,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    load_system: &mut LoadSystem,
    logger: LoggerSender,
) -> Result<SaveSystem, ErrorExecution> {
    
    let notifier = NotifierTUI::new(logger.clone());
    let input_handler = InputHandlerTUI::new(notifier.clone(), logger.clone());

    backend::backend(
        mode_config,
        connection_config,
        download_config,
        load_system,
        input_handler,
        notifier,
        logger,
    )
}