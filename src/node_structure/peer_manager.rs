use super::{error_node::ErrorNode, message_response::MessageResponse};

use crate::{
    block_structure::{
        transaction::Transaction, 
        hash::HashType,
    },
    logs::logger_sender::LoggerSender,
    configurations::connection_config::ConnectionConfig,
    node_structure::block_download::BlockDownload,
    messages::{
        addr_message::AddrMessage,
        alert_message::AlertMessage,
        block_message::BlockMessage,
        command_name::CommandName,
        fee_filter_message::FeeFilterMessage,
        get_data_message::GetDataMessage,
        get_headers_message::GetHeadersMessage,
        headers_message::HeadersMessage,
        inventory_message::InventoryMessage,
        message::{ignore_message, Message},
        message_header::MessageHeader,
        ping_message::PingMessage,
        pong_message::PongMessage,
        send_cmpct_message::SendCmpctMessage,
        send_headers_message::SendHeadersMessage,
        verack_message::VerackMessage,
        version_message::VersionMessage,
        tx_message::TxMessage,
    }, 
};

use std::{
    io::{Read, Write},
    sync::mpsc::{Receiver, Sender},
    sync::{Arc, Mutex},
};

pub struct PeerManager<RW>
where
    RW: Read + Write + Send + 'static,
{
    peer: RW,
    sender: Sender<MessageResponse>,
    receiver: Receiver<Transaction>,
    stop: Arc<Mutex<bool>>,
    connection_config: ConnectionConfig,
    logger: LoggerSender,
}

impl<RW> PeerManager<RW>
where
    RW: Read + Write + Send + 'static,
{
    pub fn new(
        peer: RW,
        sender: Sender<MessageResponse>,
        receiver: Receiver<Transaction>,
        stop: Arc<Mutex<bool>>,
        connection_config: ConnectionConfig,
        logger: LoggerSender,        
    ) -> Self {
        PeerManager {
            peer,
            sender,
            receiver,
            stop,
            connection_config,
            logger,
        }
    }

    pub fn listen_peers(mut self, logger: LoggerSender) -> Result<RW, ErrorNode> {

        while let Ok(header) = MessageHeader::deserialize_header(&mut self.peer) {
            self.manage_message(header)?;

            if let Ok(transaction) = self.receiver.try_recv() {
                self.send_transaction(transaction);
            }

            match self.stop.lock() {
                Ok(stop) => {
                    let _ = logger.log_configuration("Closing this peer".to_string());
                    if *stop {
                        break;
                    }
                }
                Err(_) => todo!(),
            }
        }

        Ok(self.peer)
    }

    fn manage_message(&mut self, header: MessageHeader) -> Result<(), ErrorNode> {
        let magic_numbers = header.magic_numbers;

        match header.command_name {
            CommandName::Version => ignore_message::<VersionMessage>(&mut self.peer, header)?,
            CommandName::Verack => ignore_message::<VerackMessage>(&mut self.peer, header)?,
            CommandName::Ping => {
                let ping = PingMessage::deserialize_message(&mut self.peer, header)?;

                let pong = PongMessage { nonce: ping.nonce };

                PongMessage::serialize_message(&mut self.peer, magic_numbers, &pong)?;
            }
            CommandName::Pong => ignore_message::<PongMessage>(&mut self.peer, header)?,
            CommandName::GetHeaders => ignore_message::<GetHeadersMessage>(&mut self.peer, header)?,
            CommandName::Headers => self.receive_headers(header),
            CommandName::GetData => ignore_message::<GetDataMessage>(&mut self.peer, header)?,
            CommandName::Block => self.receive_blocks(header),
            CommandName::Inventory => ignore_message::<InventoryMessage>(&mut self.peer, header)?,
            CommandName::SendHeaders => {
                ignore_message::<SendHeadersMessage>(&mut self.peer, header)?
            }
            CommandName::SendCmpct => ignore_message::<SendCmpctMessage>(&mut self.peer, header)?,
            CommandName::Addr => ignore_message::<AddrMessage>(&mut self.peer, header)?,
            CommandName::FeeFilter => ignore_message::<FeeFilterMessage>(&mut self.peer, header)?,
            CommandName::Alert => ignore_message::<AlertMessage>(&mut self.peer, header)?,
            CommandName::Tx => self.receive_transaction(header),
        }

        Ok(())
    }

    fn receive_headers(&mut self, header: MessageHeader) {
        let headers_message = match HeadersMessage::deserialize_message(&mut self.peer, header) {
            Ok(headers_message) => headers_message,
            _ => todo!()
        };

        let headers = headers_message.headers;
        let headers: Vec<HashType> = headers.iter().filter_map(|header| 
            match header.get_hash256d() {
                Ok(header_hash) => Some(header_hash),
                Err(_) => None,
            }
        ).collect();

        let block_download = BlockDownload::new(
            self.connection_config.magic_numbers, 
            self.logger.clone(),
        );

        let blocks = match block_download.get_data(&mut self.peer, headers) {
            Ok(blocks) => blocks,
            Err(_) => todo!(),
        };

        for block in blocks {
            if self.sender.send(MessageResponse::Block(block)).is_err() {
                todo!()
            }
        }
    }

    fn receive_blocks(&mut self, header: MessageHeader) {
        let block_message = match BlockMessage::deserialize_message(&mut self.peer, header) {
            Ok(block_message) => block_message,
            _ => todo!()
        };

        if self.sender.send(MessageResponse::Block(block_message.block)).is_err() {
            todo!()
        }   
    }

    fn receive_transaction(&mut self, header: MessageHeader) {
        let tx_message = match TxMessage::deserialize_message(&mut self.peer, header) {
            Ok(tx_message) => tx_message,
            _ => todo!()
        };

        if self.sender.send(MessageResponse::Transaction(tx_message.transaction)).is_err() {
            todo!()
        }   
    }

    fn send_transaction(&mut self, transaction: Transaction) {
        let tx_message = TxMessage { transaction };

        if TxMessage::serialize_message(
            &mut self.peer, 
            self.connection_config.magic_numbers, 
            &tx_message
        ).is_err() {
            todo!()
        }
    }
}
