use super::{
    connection_id::ConnectionId, error_node::ErrorNode, message_response::MessageResponse,
    message_to_peer::MessageToPeer,
};

use crate::{
    block_structure::{block_chain::BlockChain, hash::HashType, transaction::Transaction},
    concurrency::work::Work,
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
    notifications::{notification::Notification, notifier::Notifier},
};

use std::{
    io::{Read, Write},
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
};

/// It represents how to manage the the peer, listening to the there messages and sending them transactions
pub struct PeerManager<RW, N>
where
    RW: Read + Write + Send + 'static,
    N: Notifier + 'static,
{
    id: ConnectionId,
    peer: RW,
    sender: Sender<MessageResponse>,
    blockchain: Arc<Mutex<BlockChain>>,
    magic_numbers: [u8; 4],
    notifier: N,
    logger: LoggerSender,
}

impl<RW, N> PeerManager<RW, N>
where
    RW: Read + Write + Send + 'static,
    N: Notifier,
{
    pub fn new(
        id: ConnectionId,
        peer: RW,
        sender: Sender<MessageResponse>,
        blockchain: Arc<Mutex<BlockChain>>,
        magic_numbers: [u8; 4],
        notifier: N,
        logger: LoggerSender,
    ) -> Self {
        PeerManager {
            id,
            peer,
            sender,
            blockchain,
            magic_numbers,
            notifier,
            logger,
        }
    }

    /// Listens and send messages to the peer
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    ///  * `ErrorNode::WhileDeserialization`: It will appear when there is an error in the deserialization
    ///  * `ErrorNode::NodeNotResponding`: It will appear when the node is not responding to the messages
    pub fn connecting_to_peer(
        mut self,
        receiver: Receiver<MessageToPeer>,
    ) -> Result<(RW, ConnectionId), ErrorNode> {
        loop {
            match Work::listen(&mut self.peer, &receiver) {
                Work::Message(header) => self.manage_message(header)?,
                Work::Information(transaction) => self.send_transaction(transaction)?,
                Work::Stop => {
                    let _ = self
                        .logger
                        .log_configuration("Closing this peer".to_string());
                    self.notifier.notify(Notification::ClosingPeer);
                    break;
                }
            }
        }

        Ok((self.peer, self.id))
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

        self.notifier
            .notify(Notification::ReceivedMessage(header.command_name));

        match header.command_name {
            CommandName::Version => ignore_message::<RW, VersionMessage>(&mut self.peer, header)?,
            CommandName::Verack => ignore_message::<RW, VerackMessage>(&mut self.peer, header)?,
            CommandName::Ping => {
                let ping = PingMessage::deserialize_message(&mut self.peer, header)?;

                let pong = PongMessage { nonce: ping.nonce };

                PongMessage::serialize_message(&mut self.peer, magic_numbers, &pong)?;
            }
            CommandName::Pong => ignore_message::<RW, PongMessage>(&mut self.peer, header)?,
            CommandName::GetHeaders => self.replay_to_get_headers_message(header)?,
            CommandName::Headers => self.receive_headers(header)?,
            CommandName::GetData => self.reply_to_get_data_message(header)?,
            CommandName::Block => self.receive_blocks(header)?,
            CommandName::Inventory => self.receive_inventory_message(header)?,
            CommandName::SendHeaders => {
                ignore_message::<RW, SendHeadersMessage>(&mut self.peer, header)?
            }
            CommandName::SendCmpct => {
                ignore_message::<RW, SendCmpctMessage>(&mut self.peer, header)?
            }
            CommandName::Addr => ignore_message::<RW, AddrMessage>(&mut self.peer, header)?,
            CommandName::FeeFilter => {
                ignore_message::<RW, FeeFilterMessage>(&mut self.peer, header)?
            }
            CommandName::Alert => ignore_message::<RW, AlertMessage>(&mut self.peer, header)?,
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

    /// Creates a response to a get headers message
    fn replay_to_get_headers_message(&mut self, header: MessageHeader) -> Result<(), ErrorNode> {
        let magic_numbers = header.magic_numbers.clone();
        let get_headers = GetHeadersMessage::deserialize_message(&mut self.peer, header)?;
        let headers = self.generate_headers_message(get_headers)?;
        HeadersMessage::serialize_message(&mut self.peer, magic_numbers, &headers)?;
        Ok(())
    }

    /// Creates a response to a get headers message
    ///
    /// ### Error
    /// * `ErrorNode::WhileCreatingMessage`: It will appear when there is an error while creating the message
    fn generate_headers_message(
        &self,
        get_headers_message: GetHeadersMessage,
    ) -> Result<HeadersMessage, ErrorNode> {
        let mut blockchain = match self.blockchain.lock() {
            Ok(blockchain) => blockchain,
            Err(_) => {
                return Err(ErrorNode::WhileCreatingMessage(
                    "While locking the blockchain to create the headers message".to_string(),
                ))
            }
        };
        let most_recent_hash = match blockchain
            .get_most_recent_hash(get_headers_message.header_locator_hashes)
        {
            Ok(most_recent_hash) => most_recent_hash,
            Err(_) => {
                return Err(ErrorNode::WhileCreatingMessage(
                    "While getting the most recent hash to create the headers message".to_string(),
                ))
            }
        };
        let headers_to_send = match blockchain
            .get_headers_from_header_hash(&most_recent_hash, &get_headers_message.stop_hash)
        {
            Ok(headers_to_send) => headers_to_send,
            Err(_) => {
                return Err(ErrorNode::WhileCreatingMessage(
                    "While getting the headers to send to create the headers message".to_string(),
                ))
            }
        };
        Ok(HeadersMessage {
            headers: headers_to_send,
        })
    }

    /// Creates a response to a get data message
    fn reply_to_get_data_message(&mut self, header: MessageHeader) -> Result<(), ErrorNode> {
        let magic_numbers = header.magic_numbers.clone();
        let get_data_message = GetDataMessage::deserialize_message(&mut self.peer, header)?;

        for inventory_vector in get_data_message.inventory_vectors.iter() {
            if let TypeIdentifier::Block = inventory_vector.type_identifier {
                let blockchain = match self.blockchain.lock() {
                    Ok(blockchain) => blockchain,
                    Err(_) => {
                        return Err(ErrorNode::WhileCreatingMessage(
                            "While locking the blockchain to create the get data message"
                                .to_string(),
                        ))
                    }
                };
                if let Some(block) = blockchain.get_block_with_hash(&inventory_vector.hash_value) {
                    BlockMessage::serialize_message(&mut self.peer, magic_numbers, &block)?;
                }
            }
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
            compact256::Compact256, outpoint::Outpoint, transaction::Transaction,
            transaction_input::TransactionInput, transaction_output::TransactionOutput,
        },
        connections::type_identifier::TypeIdentifier,
        logs::logger,
        messages::{compact_size::CompactSize, inventory_vector::InventoryVector, message},
        node_structure::connection_type::ConnectionType,
        notifications::{notification::Notification, notifier::Notifier},
        serialization::error_serialization::ErrorSerialization,
    };

    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::mpsc::channel,
    };

    #[derive(Clone)]
    struct NotificationMock {}

    impl Notifier for NotificationMock {
        fn notify(&self, _notification: Notification) {}
    }

    struct Stream {
        write_stream: Vec<u8>,
        read_stream: Vec<u8>,
        pointer: usize,
    }

    impl Stream {
        pub fn new(read_stream: Vec<u8>) -> Stream {
            Stream {
                read_stream,
                write_stream: Vec::new(),
                pointer: 0,
            }
        }

        pub fn get_write_stream(&self) -> Stream {
            Stream {
                read_stream: self.write_stream.clone(),
                write_stream: Vec::new(),
                pointer: 0,
            }
        }
    }

    impl Read for Stream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let mut i = 0;
            while i < buf.len() && self.pointer < self.read_stream.len() {
                buf[i] = self.read_stream[self.pointer];
                self.pointer += 1;
                i += 1;
            }
            if i == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::WouldBlock,
                    "Error reading the stream",
                ));
            }
            Ok(i)
        }
    }

    impl Write for Stream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let mut i = 0;
            while i < buf.len() {
                self.write_stream.push(buf[i]);
                i += 1;
            }
            Ok(i)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn serialize_headers_message<W: Write>(
        stream: &mut W,
        magic_numbers: [u8; 4],
        headers: Vec<BlockHeader>,
    ) -> Result<(), ErrorSerialization> {
        let headers_message = HeadersMessage { headers };
        HeadersMessage::serialize_message(stream, magic_numbers, &headers_message)
    }

    fn serialize_block_message<W: Write>(
        stream: &mut W,
        magic_numbers: [u8; 4],
        block: Block,
    ) -> Result<(), ErrorSerialization> {
        let block_message = BlockMessage { block };
        BlockMessage::serialize_message(stream, magic_numbers, &block_message)
    }

    fn serialize_tx_message<W: Write>(
        stream: &mut W,
        magic_numbers: [u8; 4],
        transaction: Transaction,
    ) -> Result<(), ErrorSerialization> {
        let tx_message = TxMessage { transaction };
        TxMessage::serialize_message(stream, magic_numbers, &tx_message)
    }

    fn serialize_inv_message<W: Write>(
        stream: &mut W,
        magic_numbers: [u8; 4],
        inventory_vectors: Vec<InventoryVector>,
    ) -> Result<(), ErrorSerialization> {
        let inventory_message = InventoryMessage::new(inventory_vectors);
        InventoryMessage::serialize_message(stream, magic_numbers, &inventory_message)
    }

    fn serialize_ping_message<W: Write>(
        stream: &mut W,
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

    fn create_mock_blockchain() -> BlockChain {
        let transaction_input = TransactionInput::new(
            Outpoint::new([1; 32], 23),
            "Prueba in".as_bytes().to_vec(),
            24,
        );

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: "Prueba out".as_bytes().to_vec(),
        };

        let transaction = Transaction {
            version: 1,
            tx_in: vec![transaction_input.clone()],
            tx_out: vec![transaction_output.clone()],
            time: 0,
        };

        let empty_block = Block::new(BlockHeader::new(
            BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(u32::MAX),
            0,
            CompactSize::new(0),
        ));

        let mut block_with_transactions = empty_block.clone();
        block_with_transactions
            .append_transaction(transaction.clone())
            .unwrap();

        let mut blockchain = BlockChain::new(empty_block).unwrap();

        blockchain.update_block(block_with_transactions).unwrap();
        blockchain
    }

    #[test]
    fn test01_peer_manager_receives_transaction_successfully() {
        let mut stream: Vec<u8> = Vec::new();
        let magic_numbers = [11, 17, 9, 7];

        let transaction = create_transaction(0);

        serialize_tx_message(&mut stream, magic_numbers.clone(), transaction.clone()).unwrap();

        let stream = Stream::new(stream);

        let (sender_message, receiver_message) = channel::<MessageResponse>();
        let (sender_transaction, receiver_transaction) = channel::<MessageToPeer>();
        let notifier = NotificationMock {};
        let blockchain = create_mock_blockchain();
        let blockchain: Arc<Mutex<BlockChain>> = Arc::new(Mutex::new(blockchain));

        let id_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8333);

        let logger_text: Vec<u8> = Vec::new();
        let (sender, _) = logger::initialize_logger(logger_text, false);
        let peer_manager = PeerManager::new(
            ConnectionId::new(id_address, ConnectionType::Client),
            stream,
            sender_message,
            blockchain,
            magic_numbers,
            notifier,
            sender,
        );

        sender_transaction.send(MessageToPeer::Stop).unwrap();

        let _ = peer_manager
            .connecting_to_peer(receiver_transaction)
            .unwrap();

        assert_eq!(
            MessageResponse::Transaction(transaction),
            receiver_message.try_recv().unwrap()
        );
    }

    #[test]
    fn test02_peer_manager_receives_block_successfully() {
        let mut stream = Vec::new();
        let magic_numbers = [11, 17, 9, 7];

        let mut block = create_empty_block(3);
        block.append_transaction(create_transaction(0)).unwrap();
        block.append_transaction(create_transaction(1)).unwrap();
        block.append_transaction(create_transaction(2)).unwrap();

        serialize_block_message(&mut stream, magic_numbers.clone(), block.clone()).unwrap();

        let stream = Stream::new(stream);

        let (sender_message, receiver_message) = channel::<MessageResponse>();
        let (sender_transaction, receiver_transaction) = channel::<MessageToPeer>();
        let notifier = NotificationMock {};
        let blockchain = create_mock_blockchain();
        let blockchain: Arc<Mutex<BlockChain>> = Arc::new(Mutex::new(blockchain));

        let id_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8333);

        let logger_text: Vec<u8> = Vec::new();
        let (sender, _) = logger::initialize_logger(logger_text, false);
        let peer_manager = PeerManager::new(
            ConnectionId::new(id_address, ConnectionType::Peer),
            stream,
            sender_message,
            blockchain,
            magic_numbers,
            notifier,
            sender,
        );

        sender_transaction.send(MessageToPeer::Stop).unwrap();

        let _ = peer_manager
            .connecting_to_peer(receiver_transaction)
            .unwrap();

        assert_eq!(
            MessageResponse::Block(block),
            receiver_message.try_recv().unwrap()
        );
    }

    #[test]
    fn test03_peer_manager_receives_headers_successfully() {
        let mut stream = Vec::new();
        let magic_numbers = [11, 17, 9, 7];

        let first_header = create_header(4);
        let first_header_hash = first_header.get_hash256d().unwrap();

        let second_header = create_header(2);
        let second_header_hash = second_header.get_hash256d().unwrap();

        serialize_headers_message(
            &mut stream,
            magic_numbers.clone(),
            vec![first_header.clone(), second_header.clone()],
        )
        .unwrap();

        let stream = Stream::new(stream);

        let (sender_message, _) = channel::<MessageResponse>();
        let (sender_transaction, receiver_transaction) = channel::<MessageToPeer>();
        let notifier = NotificationMock {};
        let blockchain = create_mock_blockchain();
        let blockchain: Arc<Mutex<BlockChain>> = Arc::new(Mutex::new(blockchain));

        let id_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8333);

        let logger_text: Vec<u8> = Vec::new();
        let (sender, _) = logger::initialize_logger(logger_text, false);
        let peer_manager = PeerManager::new(
            ConnectionId::new(id_address, ConnectionType::Peer),
            stream,
            sender_message,
            blockchain,
            magic_numbers,
            notifier,
            sender,
        );

        sender_transaction.send(MessageToPeer::Stop).unwrap();

        let (stream, _) = peer_manager
            .connecting_to_peer(receiver_transaction)
            .unwrap();
        let mut stream = stream.get_write_stream();

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
        let mut stream = Vec::new();
        let magic_numbers = [11, 17, 9, 7];

        let block = create_empty_block(4);
        let block_hash = block.header.get_hash256d().unwrap();

        let transaction = create_transaction(0);
        let transaction_id = transaction.get_tx_id().unwrap();

        serialize_inv_message(
            &mut stream,
            magic_numbers.clone(),
            vec![
                InventoryVector::new(TypeIdentifier::Block, block_hash),
                InventoryVector::new(TypeIdentifier::TransactionId, transaction_id),
            ],
        )
        .unwrap();

        let stream = Stream::new(stream);

        let (sender_message, _) = channel::<MessageResponse>();
        let (sender_transaction, receiver_transaction) = channel::<MessageToPeer>();
        let notifier = NotificationMock {};
        let blockchain = create_mock_blockchain();
        let blockchain: Arc<Mutex<BlockChain>> = Arc::new(Mutex::new(blockchain));

        let id_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8333);

        let logger_text: Vec<u8> = Vec::new();
        let (sender, _) = logger::initialize_logger(logger_text, false);
        let peer_manager = PeerManager::new(
            ConnectionId::new(id_address, ConnectionType::Peer),
            stream,
            sender_message,
            blockchain,
            magic_numbers,
            notifier,
            sender,
        );

        sender_transaction.send(MessageToPeer::Stop).unwrap();
        let (stream, _) = peer_manager
            .connecting_to_peer(receiver_transaction)
            .unwrap();
        let mut stream = stream.get_write_stream();

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
        let mut stream = Vec::new();
        let magic_numbers = [11, 17, 9, 7];

        let transaction = create_transaction(0);

        serialize_ping_message(&mut stream, magic_numbers.clone()).unwrap();

        let stream = Stream::new(stream);

        let (sender_message, _) = channel::<MessageResponse>();
        let (sender_transaction, receiver_transaction) = channel::<MessageToPeer>();
        let notifier = NotificationMock {};
        let blockchain = create_mock_blockchain();
        let blockchain: Arc<Mutex<BlockChain>> = Arc::new(Mutex::new(blockchain));

        let id_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8333);

        let logger_text: Vec<u8> = Vec::new();
        let (sender, _) = logger::initialize_logger(logger_text, false);
        let peer_manager = PeerManager::new(
            ConnectionId::new(id_address, ConnectionType::Peer),
            stream,
            sender_message,
            blockchain,
            magic_numbers,
            notifier,
            sender,
        );

        sender_transaction
            .send(MessageToPeer::SendTransaction(transaction.clone()))
            .unwrap();
        sender_transaction.send(MessageToPeer::Stop).unwrap();

        let (stream, _) = peer_manager
            .connecting_to_peer(receiver_transaction)
            .unwrap();
        let mut stream = stream.get_write_stream();

        let header = message::deserialize_until_found(&mut stream, CommandName::Pong).unwrap();

        assert_eq!(header.command_name, CommandName::Pong);

        let _ = PongMessage::deserialize_message(&mut stream, header).unwrap();

        let header = message::deserialize_until_found(&mut stream, CommandName::Tx).unwrap();

        assert_eq!(header.command_name, CommandName::Tx);

        let transaction_message = TxMessage::deserialize_message(&mut stream, header).unwrap();

        assert_eq!(transaction, transaction_message.transaction);
    }
}
