use super::{error_node::ErrorNode, message_response::MessageResponse};

use crate::{
    block_structure::{hash::HashType, transaction::Transaction},
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
};

use std::{
    io::{Read, Write},
    sync::mpsc::{Receiver, Sender},
    sync::{Arc, Mutex},
};

/// It represents how to manage the the peer, listening to the there messages and sending them transactions
pub struct PeerManager<RW>
where
    RW: Read + Write + Send + 'static,
{
    peer: RW,
    sender: Sender<MessageResponse>,
    receiver: Receiver<Transaction>,
    stop: Arc<Mutex<bool>>,
    magic_numbers: [u8; 4],
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
        magic_numbers: [u8; 4],
        logger: LoggerSender,
    ) -> Self {
        PeerManager {
            peer,
            sender,
            receiver,
            stop,
            magic_numbers,
            logger,
        }
    }

    /// Listens and send messages to the peer
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    ///  * `ErrorNode::WhileDeserialization`: It will appear when there is an error in the deserialization
    ///  * `ErrorNode::NodeNotResponding`: It will appear when the node is not responding to the messages
    pub fn connecting_to_peer(mut self) -> Result<RW, ErrorNode> {
        while let Ok(header) = MessageHeader::deserialize_header(&mut self.peer) {
            self.manage_message(header)?;

            if let Ok(transaction) = self.receiver.try_recv() {
                self.send_transaction(transaction)?;
            }

            match self.stop.lock() {
                Ok(stop) => {
                    if *stop {
                        let _ = self
                            .logger
                            .log_configuration("Closing this peer".to_string());
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

        let get_data_message = GetDataMessage::get_blocks(headers);

        let _ = self
            .logger
            .log_connection("Sending get data message of blocks to peer".to_string());

        if GetDataMessage::serialize_message(&mut self.peer, self.magic_numbers, &get_data_message)
            .is_err()
        {
            return Err(ErrorNode::WhileSendingMessage(
                "Sending get data message to peers".to_string(),
            ));
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

        let get_data_message = GetDataMessage::new(inventory_vectors);

        let _ = self.logger.log_connection(
            "Sending get data message of transactions and blocks to peer".to_string(),
        );

        if GetDataMessage::serialize_message(&mut self.peer, self.magic_numbers, &get_data_message)
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

        if TxMessage::serialize_message(&mut self.peer, self.magic_numbers, &tx_message).is_err() {
            return Err(ErrorNode::WhileSendingMessage(
                "Sending transaction to peers".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::{
        block_structure::{
            block::Block, block_header::BlockHeader, block_version::BlockVersion,
            compact256::Compact256, error_block::ErrorBlock, outpoint::Outpoint,
            transaction::{Transaction, self}, transaction_input::TransactionInput,
            transaction_output::TransactionOutput,
        },
        connections::type_identifier::TypeIdentifier,
        logs::{logger, logger_sender::LoggerSender},
        messages::{compact_size::CompactSize, inventory_vector::{InventoryVector, self}, tx_message, message, pong_message, inventory_message},
        node_structure::initial_headers_download,
        serialization::error_serialization::ErrorSerialization,
    };

    use std::{
        thread::{self, JoinHandle},
        sync::mpsc::channel,
    };

    struct Stream {
        stream: Vec<u8>,
        pointer: usize,
    }

    impl Stream {
        pub fn new() -> Stream {
            Stream {
                stream: Vec::new(),
                pointer: 0,
            }
        }
    }

    impl Read for Stream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let mut i = 0;
            while i < buf.len() && self.pointer < self.stream.len() {
                buf[i] = self.stream[self.pointer];
                self.pointer += 1;
                i += 1;
            }
            Ok(i)
        }
    }

    impl Write for Stream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let mut i = 0;
            while i < buf.len() {
                self.stream.push(buf[i]);
                i += 1;
            }
            Ok(i)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn serialize_headers_message<RW: Read + Write>(
        stream: &mut RW,
        magic_numbers: [u8; 4],
        headers: Vec<BlockHeader>,
    ) -> Result<(), ErrorSerialization> {
        let headers_message = HeadersMessage { headers };
        HeadersMessage::serialize_message(stream, magic_numbers, &headers_message)
    }

    fn serialize_block_message<RW: Read + Write>(
        stream: &mut RW,
        magic_numbers: [u8; 4],
        block: Block,
    ) -> Result<(), ErrorSerialization> {
        let block_message = BlockMessage { block };
        BlockMessage::serialize_message(stream, magic_numbers, &block_message)
    }

    fn serialize_tx_message<RW: Read + Write>(
        stream: &mut RW,
        magic_numbers: [u8; 4],
        transaction: Transaction,
    ) -> Result<(), ErrorSerialization> {
        let tx_message = TxMessage { transaction };
        TxMessage::serialize_message(stream, magic_numbers, &tx_message)
    }

    fn serialize_inv_message<RW: Read + Write>(
        stream: &mut RW,
        magic_numbers: [u8; 4],
        inventory_vectors: Vec<InventoryVector>,
    ) -> Result<(), ErrorSerialization> {
        let inventory_message = InventoryMessage::new(inventory_vectors);
        InventoryMessage::serialize_message(stream, magic_numbers, &inventory_message)
    }

    fn serialize_ping_message<RW: Read + Write>(
        stream: &mut RW,
        magic_numbers: [u8; 4],
    ) -> Result<(), ErrorSerialization> {
        let ping_message = PingMessage { nonce: 1234 };
        PingMessage::serialize_message(stream, magic_numbers, &ping_message)
    }

    fn create_transaction(time: u32) -> Transaction {
        let transaction_input =
            TransactionInput::new(Outpoint::new([1; 32], 23), vec![1, 2, 3], 24);

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![4, 5, 6],
        };

        Transaction {
            version: 1,
            tx_in: vec![transaction_input.clone()],
            tx_out: vec![transaction_output.clone()],
            time,
        }
    }

    fn create_header(transaction_count: u64) -> BlockHeader {
        BlockHeader::new(
            BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(u32::MAX),
            0,
            CompactSize::new(transaction_count),
        )
    }

    fn create_empty_block(transaction_count: u64) -> Block {
        Block::new(create_header(transaction_count))
    }

    #[test]
    fn test01_peer_manager_receives_transaction_successfully() {
        let mut stream = Stream::new();
        let magic_numbers = [11, 17, 9, 7];

        let transaction = create_transaction(0);

        serialize_tx_message(&mut stream, magic_numbers.clone(), transaction.clone());

        let (sender_message, receiver_message) = channel::<MessageResponse>();
        let (sender_transaction, receiver_transaction) = channel::<Transaction>();

        let mut logger_text: Vec<u8> = Vec::new();
        let (sender, _) = logger::initialize_logger(logger_text, false);
        let peer_manager = PeerManager::new(
            stream, 
            sender_message,
            receiver_transaction,
            Arc::new(Mutex::new(true)),
            magic_numbers,
            sender,
        );

        let mut stream = peer_manager.connecting_to_peer();

        assert_eq!(MessageResponse::Transaction(transaction), receiver_message.try_recv().unwrap());
    }

    #[test]
    fn test02_peer_manager_receives_block_successfully() {
        let mut stream = Stream::new();
        let magic_numbers = [11, 17, 9, 7];

        let mut block = create_empty_block(3);
        block.append_transaction(create_transaction(0)).unwrap();
        block.append_transaction(create_transaction(1)).unwrap();
        block.append_transaction(create_transaction(2)).unwrap();

        serialize_block_message(&mut stream, magic_numbers.clone(), block.clone());

        let (sender_message, receiver_message) = channel::<MessageResponse>();
        let (sender_transaction, receiver_transaction) = channel::<Transaction>();

        let mut logger_text: Vec<u8> = Vec::new();
        let (sender, _) = logger::initialize_logger(logger_text, false);
        let peer_manager = PeerManager::new(
            stream, 
            sender_message,
            receiver_transaction,
            Arc::new(Mutex::new(true)),
            magic_numbers,
            sender,
        );

        let mut stream = peer_manager.connecting_to_peer();

        assert_eq!(MessageResponse::Block(block), receiver_message.try_recv().unwrap());
    }

    #[test]
    fn test03_peer_manager_receives_headers_successfully() {
        let mut stream = Stream::new();
        let magic_numbers = [11, 17, 9, 7];

        let first_header = create_header(4);
        let first_header_hash = first_header.get_hash256d().unwrap();

        let second_header = create_header(2);
        let second_header_hash = second_header.get_hash256d().unwrap();

        serialize_headers_message(&mut stream, magic_numbers.clone(), vec![first_header.clone(), second_header.clone()]);

        let (sender_message, receiver_message) = channel::<MessageResponse>();
        let (sender_transaction, receiver_transaction) = channel::<Transaction>();

        let mut logger_text: Vec<u8> = Vec::new();
        let (sender, _) = logger::initialize_logger(logger_text, false);
        let peer_manager = PeerManager::new(
            stream, 
            sender_message,
            receiver_transaction,
            Arc::new(Mutex::new(true)),
            magic_numbers,
            sender,
        );

        let mut stream = peer_manager.connecting_to_peer().unwrap();

        let header = message::deserialize_until_found(&mut stream, CommandName::GetData).unwrap();

        assert_eq!(header.command_name, CommandName::GetData);

        let get_data_message = GetDataMessage::deserialize_message(&mut stream, header).unwrap();

        let inventory_vectors = vec![
            InventoryVector::new(TypeIdentifier::Block, first_header_hash),
            InventoryVector::new(TypeIdentifier::Block, second_header_hash),
        ];

        assert_eq!(inventory_vectors, get_data_message.inventory_vectors);        
    }

    #[test]
    fn test04_peer_manager_receives_inventory_message_successfully() {
        let mut stream = Stream::new();
        let magic_numbers = [11, 17, 9, 7];

        let block = create_empty_block(4);
        let block_hash = block.header.get_hash256d().unwrap();

        let transaction = create_transaction(0);
        let transaction_id = transaction.get_tx_id().unwrap();

        serialize_inv_message(&mut stream, magic_numbers.clone(), vec![
            InventoryVector::new(TypeIdentifier::Block, block_hash),
            InventoryVector::new(TypeIdentifier::TransactionId, transaction_id),
        ]);

        let (sender_message, receiver_message) = channel::<MessageResponse>();
        let (sender_transaction, receiver_transaction) = channel::<Transaction>();

        let mut logger_text: Vec<u8> = Vec::new();
        let (sender, _) = logger::initialize_logger(logger_text, false);
        let peer_manager = PeerManager::new(
            stream, 
            sender_message,
            receiver_transaction,
            Arc::new(Mutex::new(true)),
            magic_numbers,
            sender,
        );

        let mut stream = peer_manager.connecting_to_peer().unwrap();

        let header = message::deserialize_until_found(&mut stream, CommandName::GetData).unwrap();

        assert_eq!(header.command_name, CommandName::GetData);

        let get_data_message = GetDataMessage::deserialize_message(&mut stream, header).unwrap();

        let inventory_vectors = vec![
            InventoryVector::new(TypeIdentifier::Block, block_hash),
            InventoryVector::new(TypeIdentifier::TransactionId, transaction_id),
        ];

        assert_eq!(inventory_vectors, get_data_message.inventory_vectors);        
    }

    #[test]
    fn test05_peer_manager_sends_transaction_successfully() {
        let mut stream = Stream::new();
        let magic_numbers = [11, 17, 9, 7];

        let transaction = create_transaction(0);

        serialize_ping_message(&mut stream, magic_numbers.clone());

        let (sender_message, receiver_message) = channel::<MessageResponse>();
        let (sender_transaction, receiver_transaction) = channel::<Transaction>();

        let mut logger_text: Vec<u8> = Vec::new();
        let (sender, _) = logger::initialize_logger(logger_text, false);
        let peer_manager = PeerManager::new(
            stream, 
            sender_message,
            receiver_transaction,
            Arc::new(Mutex::new(true)),
            magic_numbers,
            sender,
        );

        sender_transaction.send(transaction.clone()).unwrap();

        let mut stream = peer_manager.connecting_to_peer().unwrap();

        let header = message::deserialize_until_found(&mut stream, CommandName::Pong).unwrap();

        assert_eq!(header.command_name, CommandName::Pong);

        let _ = PongMessage::deserialize_message(&mut stream, header).unwrap();

        let header = message::deserialize_until_found(&mut stream, CommandName::Tx).unwrap();

        assert_eq!(header.command_name, CommandName::Tx);

        let transaction_message = TxMessage::deserialize_message(&mut stream, header).unwrap();

        assert_eq!(transaction, transaction_message.transaction);
    }
}
