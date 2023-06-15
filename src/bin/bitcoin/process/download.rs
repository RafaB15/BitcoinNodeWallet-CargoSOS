use crate::error_execution::ErrorExecution;

use cargosos_bitcoin::{
    block_structure::{block::Block, block_chain::BlockChain, hash::HashType},
    configurations::{connection_config::ConnectionConfig, download_config::DownloadConfig},
    logs::logger_sender::LoggerSender,
    node_structure::{
        block_broadcasting::BlockBroadcasting, block_download::BlockDownload,
        error_node::ErrorNode, initial_headers_download::InitialHeaderDownload,
    },
};

use std::{
    fmt::Debug,
    io::{Read, Write},
    thread::{self, JoinHandle},
};

/// Given the peers connection, updates the blockchain with the new blocks of the respected peers.
/// The approch is to get the headers first and then the blocks.
///
/// ### Error
///  * `ErrorMessage::InSerialization`: It will appear when the serialization of the message fails or the SHA(SHA(header)) fails
///  * `ErrorNode::NodeNotResponding`: It will appear when
///  * `ErrorNode::WhileValidating`: It will appear when
///  * `ErrorBlock::CouldNotUpdate`: It will appear when the block is not in the blockchain.
///  * `ErrorExecution::FailThread`: It will appear when the thread fails
pub fn headers_first<RW: Read + Write + Send + Debug + 'static>(
    peer_streams: Vec<RW>,
    block_chain: &mut BlockChain,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    logger: LoggerSender,
) -> Result<Vec<RW>, ErrorExecution> {
    let header_download = InitialHeaderDownload::new(
        connection_config.p2p_protocol_version,
        connection_config.magic_numbers,
        logger.clone(),
    );

    let block_download = BlockDownload::new(connection_config.magic_numbers, logger.clone());

    let _ = logger.log_connection("Getting initial download headers first".to_string());

    let mut peer_download_handles: Vec<JoinHandle<(Vec<Block>, RW)>> = Vec::new();

    for peer_stream in peer_streams {
        let mut peer_stream = peer_stream;

        let _ = logger.log_connection(format!("Connecting to peer: {:?}", peer_stream));

        get_peer_header(&mut peer_stream, &header_download, block_chain, &logger)?;

        let mut list_of_blocks: Vec<Block> = Vec::new();
        for block in block_chain.get_blocks_after_timestamp(download_config.timestamp)? {
            if block.transactions.len() as u64 == 0 {
                list_of_blocks.push(block);
            }
        }

        peer_download_handles.push(get_blocks(
            peer_stream,
            block_download.clone(),
            list_of_blocks,
            logger.clone(),
        ));
    }

    let mut peer_stream: Vec<RW> = Vec::new();
    for peer_download_handle in peer_download_handles {
        peer_stream.push(updating_block_chain(
            block_chain,
            peer_download_handle,
            logger.clone(),
        )?);
    }
    Ok(peer_stream)
}

/// It updates the blockchain with a specific peer headers until it reach the last header
///
/// ### Error
///  * `ErrorMessage::InSerialization`: It will appear when the serialization of the message fails or the SHA(SHA(header)) fails
///  * `ErrorNode::NodeNotResponding`: It will appear when no message is received from the node
///  * `ErrorNode::WhileValidating`: It will appear when a given header does not pass the proof of work to be added to the blockchain
fn get_peer_header<RW: Read + Write>(
    peer_stream: &mut RW,
    header_download: &InitialHeaderDownload,
    block_chain: &mut BlockChain,
    logger: &LoggerSender,
) -> Result<(), ErrorExecution> {
    loop {
        let header_count: u32 = match header_download.get_headers(peer_stream, block_chain) {
            Err(ErrorNode::NodeNotResponding(message)) => {
                let _ = logger.log_connection(format!("Node not responding, send: {}", message));
                break;
            }
            other_response => other_response?,
        };

        let _ = logger.log_connection(format!("We get: {}", header_count));

        if header_count == 0 {
            break;
        }
    }

    Ok(())
}

/// It gets the blocks from a specific peer in a thread
fn get_blocks<RW: Read + Write + Send + 'static>(
    mut peer_stream: RW,
    block_download: BlockDownload,
    list_of_blocks: Vec<Block>,
    logger: LoggerSender,
) -> JoinHandle<(Vec<Block>, RW)> {
    thread::spawn(move || {
        let mut headers: Vec<HashType> = Vec::new();

        for block in list_of_blocks {
            let header_hash = match block.header.get_hash256d() {
                Ok(header_hash) => header_hash,
                Err(_) => continue,
            };

            headers.push(header_hash);
        }

        match block_download.get_data(&mut peer_stream, headers) {
            Ok(blocks) => (blocks, peer_stream),
            Err(error) => {
                let _ = logger.log_connection(format!("Cannot get block, we get {:?}", error));
                (vec![], peer_stream)
            }
        }
    })
}

/// Updates the blockchain of the thread of a peer
///
/// ### Error
///  * `ErrorBlock::CouldNotUpdate`: It will appear when the block is not in the blockchain.
///  * `ErrorExecution::FailThread`: It will appear when the thread fails
fn updating_block_chain<RW: Read + Write + Send>(
    block_chain: &mut BlockChain,
    peer_download_handle: JoinHandle<(Vec<Block>, RW)>,
    logger: LoggerSender,
) -> Result<RW, ErrorExecution> {
    let _ = logger.log_connection("Finish downloading, loading to blockchain".to_string());
    match peer_download_handle.join() {
        Ok((blocks, peer_stream)) => {
            let _ =
                logger.log_connection(format!("Loading {} blocks to blockchain", blocks.len(),));

            for (i, block) in blocks.iter().enumerate() {
                block_chain.update_block(block.clone())?;

                if i % 50 == 0 {
                    let _ = logger.log_connection(format!("Loading [{i}] blocks to blockchain",));
                }
            }

            let _ =
                logger.log_connection(format!("Loading [{}] blocks to blockchain", blocks.len()));

            Ok(peer_stream)
        }
        _ => Err(ErrorExecution::FailThread),
    }
}

/// Given the peers connection, updates the blockchain with the new blocks of the respected peers.
/// The approch is to get the entire block.
pub fn blocks_first<RW: Read + Write + Send>() -> Vec<RW> {
    todo!()
}

/// It updates the blockchain listening to the new headers of the peers
///
/// ### Error
///  * `ErrorNode::NodeNotResponding`: It will appear when no message is received from the node
///  * `ErrorNode::WhileValidating`: It will appear when a given header does not pass the proof of work to be added to the blockchain
///  * `ErrorBlock::CouldNotUpdate`: It will appear when the block is not in the blockchain.
///  * `ErrorExecution::FailThread`: It will appear when the thread fails
pub fn block_broadcasting<RW: Read + Write + Send + 'static>(
    peer_streams: Vec<RW>,
    block_chain: &mut BlockChain,
    connection_config: ConnectionConfig,
    logger: LoggerSender,
) -> Result<Vec<RW>, ErrorExecution> {
    let block_download = BlockDownload::new(connection_config.magic_numbers, logger.clone());

    let block_broadcasting = BlockBroadcasting::new(logger.clone());

    let mut peer_download_handles: Vec<JoinHandle<(Vec<Block>, RW)>> = Vec::new();

    for peer_stream in peer_streams {
        let mut peer_stream = peer_stream;

        let (header_count, headers) = match block_broadcasting
            .get_new_headers(&mut peer_stream, block_chain)
        {
            Err(ErrorNode::NodeNotResponding(message)) => {
                let _ = logger.log_connection(format!("Node not responding, send: {}", message));
                break;
            }
            other_response => other_response?,
        };

        let _ = logger.log_connection(format!("We get: {}", header_count));

        peer_download_handles.push(get_blocks(
            peer_stream,
            block_download.clone(),
            headers.iter().map(|header| Block::new(*header)).collect(),
            logger.clone(),
        ));
    }

    let mut peer_streams: Vec<RW> = Vec::new();
    for peer_download_handle in peer_download_handles {
        peer_streams.push(updating_block_chain(
            block_chain,
            peer_download_handle,
            logger.clone(),
        )?);
    }

    Ok(peer_streams)
}
