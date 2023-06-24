use super::{error_node::ErrorNode, message_response::MessageResponse};

use crate::{
    block_structure::{hash::HashType, transaction::Transaction},
    configurations::connection_config::ConnectionConfig,
    connections::type_identifier::TypeIdentifier,
    logs::logger_sender::LoggerSender,
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
        inventory_vector::InventoryVector,
        message::{ignore_message, Message},
        message_header::MessageHeader,
        ping_message::PingMessage,
        pong_message::PongMessage,
        send_cmpct_message::SendCmpctMessage,
        send_headers_message::SendHeadersMessage,
        tx_message::TxMessage,
        verack_message::VerackMessage,
        version_message::VersionMessage,
    },
    node_structure::block_download::BlockDownload,
};

use std::{
    io::{Read, Write},
    sync::mpsc::{Receiver, Sender},
    sync::{Arc, Mutex},
};

/// It represents how to manage the the peer, listening to the there messages and sending them transactions
pub(super) struct PeerManager<RW>
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

    /// Listens and send messages to the peer
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    ///  * `ErrorNode::WhileDeserialization`: It will appear when there is an error in the deserialization
    ///  * `ErrorNode::NodeNotResponding`: It will appear when the node is not responding to the messages
    pub fn connecting_to_peer(mut self, logger: LoggerSender) -> Result<RW, ErrorNode> {
        while let Ok(header) = MessageHeader::deserialize_header(&mut self.peer) {
            self.manage_message(header)?;

            if let Ok(transaction) = self.receiver.try_recv() {
                self.send_transaction(transaction)?;
            }

            match self.stop.lock() {
                Ok(stop) => {
                    if *stop {
                        let _ = logger.log_configuration("Closing this peer".to_string());
                        break;
                    }
                }
                Err(_) => {
                    return Err(ErrorNode::NodeNotResponding(
                        "Could not determine if to stop".to_string(),
                    ))
                }
            }
        }

        Ok(self.peer)
    }

    /// Receives the message from the peer and manages it by sending to the peer or others threads via the sender
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    ///  * `ErrorNode::WhileDeserialization`: It will appear when there is an error in the deserialization
    ///  * `ErrorNode::WhileSendingMessage`: It will appear when there is an error while sending a message to others threads
    fn manage_message(&mut self, header: MessageHeader) -> Result<(), ErrorNode> {
        let magic_numbers = header.magic_numbers;

        let _ = self
            .logger
            .log_connection(format!("Receive message of type {:?}", header.command_name));

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
            CommandName::Headers => self.receive_headers(header)?,
            CommandName::GetData => ignore_message::<GetDataMessage>(&mut self.peer, header)?,
            CommandName::Block => self.receive_blocks(header)?,
            CommandName::Inventory => self.receive_inventory_message(header)?,
            CommandName::SendHeaders => {
                ignore_message::<SendHeadersMessage>(&mut self.peer, header)?
            }
            CommandName::SendCmpct => ignore_message::<SendCmpctMessage>(&mut self.peer, header)?,
            CommandName::Addr => ignore_message::<AddrMessage>(&mut self.peer, header)?,
            CommandName::FeeFilter => ignore_message::<FeeFilterMessage>(&mut self.peer, header)?,
            CommandName::Alert => ignore_message::<AlertMessage>(&mut self.peer, header)?,
            CommandName::Tx => self.receive_transaction(header)?,
        }

        Ok(())
    }

    /// Receives the message of a new header, and request its corresponding block
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    ///  * `ErrorNode::WhileDeserialization`: It will appear when there is an error in the deserialization
    ///  * `ErrorNode::WhileSendingMessage`: It will appear when there is an error while sending a message to others threads
    fn receive_headers(&mut self, header: MessageHeader) -> Result<(), ErrorNode> {
        let _ = self.logger.log_connection("Receiving headers".to_string());
        let headers_message = HeadersMessage::deserialize_message(&mut self.peer, header)?;

        let headers = headers_message.headers;
        let headers: Vec<HashType> = headers
            .iter()
            .filter_map(|header| match header.get_hash256d() {
                Ok(header_hash) => Some(header_hash),
                Err(_) => None,
            })
            .collect();

        let block_download =
            BlockDownload::new(self.connection_config.magic_numbers, self.logger.clone());

        let blocks = block_download.get_data(&mut self.peer, headers)?;

        for block in blocks {
            if self.sender.send(MessageResponse::Block(block)).is_err() {
                return Err(ErrorNode::WhileSendingMessage(
                    "Sending block back".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Receives the message of a new block, and send it to others threads via the sender
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    ///  * `ErrorNode::WhileDeserialization`: It will appear when there is an error in the deserialization
    ///  * `ErrorNode::WhileSendingMessage`: It will appear when there is an error while sending a message to others threads
    fn receive_blocks(&mut self, header: MessageHeader) -> Result<(), ErrorNode> {
        let _ = self.logger.log_connection("Receiving blocks".to_string());
        let block_message = BlockMessage::deserialize_message(&mut self.peer, header)?;

        if self
            .sender
            .send(MessageResponse::Block(block_message.block))
            .is_err()
        {
            return Err(ErrorNode::WhileSendingMessage(
                "Sending block back".to_string(),
            ));
        }

        Ok(())
    }

    /// Receives the message of a new transaction, and send it to others threads via the sender
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    ///  * `ErrorNode::WhileDeserialization`: It will appear when there is an error in the deserialization
    ///  * `ErrorNode::WhileSendingMessage`: It will appear when there is an error while sending a message to others threads
    fn receive_transaction(&mut self, header: MessageHeader) -> Result<(), ErrorNode> {
        let _ = self
            .logger
            .log_connection("Receiving a transaction".to_string());
        let tx_message = TxMessage::deserialize_message(&mut self.peer, header)?;

        if self
            .sender
            .send(MessageResponse::Transaction(tx_message.transaction))
            .is_err()
        {
            return Err(ErrorNode::WhileSendingMessage(
                "Sending transaction back".to_string(),
            ));
        }

        Ok(())
    }

    /// Receives the inventory message for requesting to know about a new transaction
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    ///  * `ErrorNode::WhileDeserialization`: It will appear when there is an error in the deserialization
    ///  * `ErrorNode::WhileSendingMessage`: It will appear when there is an error while sending a message to others threads
    fn receive_inventory_message(&mut self, header: MessageHeader) -> Result<(), ErrorNode> {
        let _ = self
            .logger
            .log_connection("Receiving a inventory message".to_string());
        let inventory_message = InventoryMessage::deserialize_message(&mut self.peer, header)?;

        let mut inventory_vectors: Vec<InventoryVector> = Vec::new();
        for inventory_vector in inventory_message.inventory_vectors {
            match inventory_vector.type_identifier.clone() {
                TypeIdentifier::TransactionId | TypeIdentifier::Block => {
                    inventory_vectors.push(inventory_vector);
                }
                _ => {}
            }
        }

        if inventory_vectors.is_empty() {
            return Ok(());
        }

        let get_data_message = GetDataMessage { inventory_vectors };

        let _ = self
            .logger
            .log_connection("Sending get data message to peer".to_string());

        if GetDataMessage::serialize_message(
            &mut self.peer,
            self.connection_config.magic_numbers,
            &get_data_message,
        )
        .is_err()
        {
            return Err(ErrorNode::WhileSendingMessage(
                "Sending get data message to peers".to_string(),
            ));
        }

        Ok(())
    }

    /// Sends a transaction to the peer
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    ///  * `ErrorNode::WhileDeserialization`: It will appear when there is an error in the deserialization
    ///  * `ErrorNode::WhileSendingMessage`: It will appear when there is an error while sending a message to others threads
    fn send_transaction(&mut self, transaction: Transaction) -> Result<(), ErrorNode> {
        let _ = self
            .logger
            .log_connection("Sending a transaction".to_string());
        let tx_message = TxMessage { transaction };

        if TxMessage::serialize_message(
            &mut self.peer,
            self.connection_config.magic_numbers,
            &tx_message,
        )
        .is_err()
        {
            return Err(ErrorNode::WhileSendingMessage(
                "Sending transaction to peers".to_string(),
            ));
        }

        Ok(())
    }
}
