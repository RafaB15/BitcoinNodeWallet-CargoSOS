use super::error_process::ErrorProcess;

use cargosos_bitcoin::{
    block_structure::{block::Block, block_chain::BlockChain, hash::HashType, utxo_set::UTXOSet},
    configurations::{connection_config::ConnectionConfig, download_config::DownloadConfig},
    connections::ibd_methods::IBDMethod,
    logs::logger_sender::LoggerSender,
    node_structure::{
        block_download::BlockDownload, connection_id::ConnectionId, error_node::ErrorNode,
        initial_headers_download::InitialHeaderDownload,
    },
    notifications::{notification::Notification, notifier::Notifier},
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
///  * `ErrorProcess::FailThread`: It will appear when the thread fails
fn headers_first<N: Notifier, RW: Read + Write + Send + Debug + 'static>(
    connections: Vec<(RW, ConnectionId)>,
    block_chain: &mut BlockChain,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    notifier: N,
    logger: LoggerSender,
) -> Result<Vec<(RW, ConnectionId)>, ErrorProcess> {
    let header_download = InitialHeaderDownload::new(
        connection_config.p2p_protocol_version,
        connection_config.magic_numbers,
        logger.clone(),
    );

    let block_download = BlockDownload::new(connection_config.magic_numbers, logger.clone());

    let _ = logger.log_connection("Getting initial download headers first".to_string());

    let mut peer_download_handles: Vec<(JoinHandle<(Vec<Block>, RW)>, ConnectionId)> = Vec::new();

    for (peer_stream, id) in connections {
        let mut peer_stream = peer_stream;

        let _ = logger.log_connection(format!("Connecting to peer: {}", id));

        get_peer_header(
            &mut peer_stream,
            &header_download,
            block_chain,
            notifier.clone(),
            &logger,
        )?;

        let mut list_of_blocks: Vec<Block> = Vec::new();
        for block in block_chain.get_blocks_after_timestamp(download_config.timestamp) {
            if block.transactions.len() as u64 == 0 {
                list_of_blocks.push(block);
            }
        }

        peer_download_handles.push((
            get_blocks(
                peer_stream,
                block_download.clone(),
                list_of_blocks,
                logger.clone(),
            ),
            id,
        ));
    }

    let mut connections: Vec<(RW, ConnectionId)> = Vec::new();
    for (peer_download_handle, id) in peer_download_handles {
        let stream = updating_block_chain(
            block_chain,
            peer_download_handle,
            notifier.clone(),
            logger.clone(),
        )?;
        connections.push((stream, id));
    }
    Ok(connections)
}

/// It updates the blockchain with a specific peer headers until it reach the last header
///
/// ### Error
///  * `ErrorMessage::InSerialization`: It will appear when the serialization of the message fails or the SHA(SHA(header)) fails
///  * `ErrorNode::NodeNotResponding`: It will appear when no message is received from the node
///  * `ErrorNode::WhileValidating`: It will appear when a given header does not pass the proof of work to be added to the blockchain
fn get_peer_header<N: Notifier, RW: Read + Write>(
    peer_stream: &mut RW,
    header_download: &InitialHeaderDownload,
    block_chain: &mut BlockChain,
    notifier: N,
    logger: &LoggerSender,
) -> Result<(), ErrorProcess> {
    loop {
        let header_count: u32 = match header_download.get_headers(peer_stream, block_chain) {
            Err(ErrorNode::NodeNotResponding(message)) => {
                let _ = logger.log_connection(format!("Node not responding, send: {}", message));
                break;
            }
            Ok(count) => count,
            Err(ErrorNode::WhileSerializing(_)) => return Err(ErrorProcess::ErrorWriting),
            Err(_) => {
                return Err(ErrorProcess::ErrorFromPeer(
                    "Proof of work failed".to_string(),
                ))
            }
        };

        let _ = logger.log_connection(format!("We get: {}", header_count));
        notifier.notify(Notification::HeadersReceived(header_count));

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

/// Updates the blockchain with the new blocks and returns the TcpStreams that are still connected
pub fn update_block_chain<N: Notifier, RW: Read + Write + Send + Debug + 'static>(
    connections: Vec<(RW, ConnectionId)>,
    block_chain: &mut BlockChain,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    notifier: N,
    logger: LoggerSender,
) -> Result<Vec<(RW, ConnectionId)>, ErrorProcess> {
    let _ = logger.log_connection("Getting block chain".to_string());

    Ok(match connection_config.ibd_method {
        IBDMethod::HeaderFirst => headers_first(
            connections,
            block_chain,
            connection_config,
            download_config,
            notifier,
            logger,
        )?,
        IBDMethod::BlocksFirst => blocks_first::<RW>(),
    })
}

/// Creates the UTXO set from the given block chain
pub fn get_utxo_set(block_chain: &BlockChain, logger: LoggerSender) -> UTXOSet {
    let _ = logger.log_wallet("Creating the UTXO set".to_string());

    let utxo_set = UTXOSet::from_blockchain(block_chain);

    let _ = logger.log_wallet("UTXO set finished successfully".to_string());
    utxo_set
}

/// Updates the blockchain of the thread of a peer
///
/// ### Error
///  * `ErrorBlock::CouldNotUpdate`: It will appear when the block is not in the blockchain.
///  * `ErrorExecution::FailThread`: It will appear when the thread fails
fn updating_block_chain<N: Notifier, RW: Read + Write + Send>(
    block_chain: &mut BlockChain,
    peer_download_handle: JoinHandle<(Vec<Block>, RW)>,
    notifier: N,
    logger: LoggerSender,
) -> Result<RW, ErrorProcess> {
    let _ = logger.log_connection("Finish downloading, loading to blockchain".to_string());
    match peer_download_handle.join() {
        Ok((blocks, peer_stream)) => {
            let total_blocks = blocks.len();
            let _ = logger.log_connection(format!("Loading {total_blocks} blocks to blockchain"));

            for (i, block) in blocks.iter().enumerate() {
                if block_chain.update_block(block.clone()).is_err() {
                    continue;
                }

                if i % 50 == 0 {
                    let _ = logger.log_connection(format!("Loading [{i}] blocks to blockchain",));
                    notifier.notify(Notification::ProgressDownloadingBlocks(
                        i as u32,
                        total_blocks as u32,
                    ));
                }
            }

            let _ =
                logger.log_connection(format!("Loading [{}] blocks to blockchain", blocks.len()));

            Ok(peer_stream)
        }
        _ => Err(ErrorProcess::FailThread),
    }
}

/// Given the peers connection, updates the blockchain with the new blocks of the respected peers.
/// The approch is to get the entire block.
fn blocks_first<RW: Read + Write + Send>() -> Vec<(RW, ConnectionId)> {
    todo!()
}
