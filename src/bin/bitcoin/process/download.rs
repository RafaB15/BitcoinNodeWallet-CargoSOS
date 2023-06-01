use crate::error_execution::ErrorExecution;

use cargosos_bitcoin::{
    block_structure::{block::Block, block_chain::BlockChain, hash::HashType},
    configurations::download_config::DownloadConfig,
    logs::logger_sender::LoggerSender,
    node_structure::{
        block_broadcasting::BlockBroadcasting, block_download::BlockDownload,
        error_node::ErrorNode, initial_headers_download::InitialHeaderDownload,
    },
};

use std::{
    net::TcpStream,
    thread::{self, JoinHandle},
};

pub fn headers_first(
    peer_streams: Vec<TcpStream>,
    block_chain: &mut BlockChain,
    header_download: InitialHeaderDownload,
    block_download: BlockDownload,
    download_config: DownloadConfig,
    logger_sender: LoggerSender,
) -> Result<(), ErrorExecution> {
    logger_sender.log_connection("Getting initial download headers first".to_string())?;

    let mut peer_download_handles: Vec<JoinHandle<Vec<Block>>> = Vec::new();

    for peer_stream in peer_streams {
        let mut peer_stream = peer_stream;

        logger_sender.log_connection(format!("Connecting to peer: {:?}", peer_stream))?;

        get_peer_header(
            &mut peer_stream,
            &header_download,
            block_chain,
            &logger_sender,
        )?;

        let list_of_blocks = block_chain.get_blocks_after_timestamp(download_config.timestamp)?;

        let block_download_peer = block_download.clone();

        let logger_sender_clone = logger_sender.clone();
        let peer_download_handle = thread::spawn(move || {
            get_blocks(
                &mut peer_stream,
                block_download_peer,
                list_of_blocks,
                logger_sender_clone,
            )
        });

        peer_download_handles.push(peer_download_handle);
    }

    updating_block_chain(block_chain, peer_download_handles, logger_sender)
}

fn get_peer_header(
    peer_stream: &mut TcpStream,
    header_download: &InitialHeaderDownload,
    block_chain: &mut BlockChain,
    logger_sender: &LoggerSender,
) -> Result<(), ErrorExecution> {
    loop {
        let header_count: u32 = match header_download.get_headers(peer_stream, block_chain) {
            Err(ErrorNode::NodeNotResponding(message)) => {
                logger_sender.log_connection(format!("Node not responding, send: {}", message))?;
                break;
            }
            other_response => other_response?,
        };

        logger_sender.log_connection(format!("We get: {}", header_count))?;

        if header_count == 0 {
            break;
        }
    }

    Ok(())
}

pub(super) fn get_blocks(
    peer_stream: &mut TcpStream,
    block_download: BlockDownload,
    list_of_blocks: Vec<Block>,
    logger_sender: LoggerSender,
) -> Vec<Block> {
    let mut headers: Vec<HashType> = Vec::new();

    for block in list_of_blocks {
        let header_hash = match block.header.get_hash256d() {
            Ok(header_hash) => header_hash,
            Err(_) => continue,
        };

        headers.push(header_hash);
    }

    match block_download.get_data(peer_stream, headers) {
        Ok(blocks) => blocks,
        Err(error) => {
            let _ = logger_sender.log_connection(format!("Cannot get block, we get {:?}", error));
            vec![]
        }
    }
}

pub(super) fn updating_block_chain(
    block_chain: &mut BlockChain,
    peer_download_handles: Vec<JoinHandle<Vec<Block>>>,
    logger_sender: LoggerSender,
) -> Result<(), ErrorExecution> {
    for peer_download_handle in peer_download_handles {
        logger_sender.log_connection("Finish downloading, loading to blockchain".to_string())?;
        match peer_download_handle.join() {
            Ok(blocks) => {
                logger_sender
                    .log_connection(format!("Loading {} blocks to blockchain", blocks.len(),))?;

                for (i, block) in blocks.iter().enumerate() {
                    block_chain.update_block(block.clone())?;

                    if i % 50 == 0 {
                        logger_sender
                            .log_connection(format!("Loading [{i}] blocks to blockchain",))?;
                    }
                }
            }
            _ => return Err(ErrorExecution::FailThread),
        }
    }

    Ok(())
}

pub fn blocks_first() {
    todo!()
}

pub fn _block_broadcasting(
    peer_streams: Vec<TcpStream>,
    block_chain: &mut BlockChain,
    logger_sender: LoggerSender,
) -> Result<(), ErrorExecution> {
    logger_sender.log_connection("Broadcasting...".to_string())?;

    let block_broadcasting = BlockBroadcasting::new(logger_sender.clone());

    let blocks_download = BlockDownload::new(logger_sender.clone());

    let mut peer_download_handles: Vec<JoinHandle<Vec<Block>>> = Vec::new();

    for peer_stream in peer_streams {
        let mut peer_stream = peer_stream;

        let (header_count, headers) = match block_broadcasting
            .get_new_headers(&mut peer_stream, block_chain)
        {
            Err(ErrorNode::NodeNotResponding(message)) => {
                logger_sender.log_connection(format!("Node not responding, send: {}", message))?;
                break;
            }
            other_response => other_response?,
        };

        logger_sender.log_connection(format!("We get: {}", header_count))?;

        let blocks = headers.iter().map(|header| Block::new(*header)).collect();

        let logger_sender_clone = logger_sender.clone();
        let peer_block_download = blocks_download.clone();

        let peer_download_handle = thread::spawn(move || {
            get_blocks(
                &mut peer_stream,
                peer_block_download,
                blocks,
                logger_sender_clone,
            )
        });

        peer_download_handles.push(peer_download_handle);
    }

    updating_block_chain(block_chain, peer_download_handles, logger_sender)
}
