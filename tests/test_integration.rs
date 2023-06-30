mod common;

#[cfg(test)]
mod test_integration {

    use super::common::{creation, serialize_message, stream::Stream};

    use cargosos_bitcoin::{
        block_structure::{block_chain::BlockChain, hash::HashType, transaction::Transaction},
        connections::{p2p_protocol::ProtocolVersionP2P, supported_services::SupportedServices},
        logs::logger,
        messages::{
            bitfield_services::BitfieldServices,
            command_name::CommandName,
            get_headers_message::GetHeadersMessage,
            message::{self, Message},
            send_headers_message::SendHeadersMessage,
            tx_message::TxMessage,
            verack_message::VerackMessage,
            version_message::VersionMessage,
        },
        node_structure::{
            block_download::BlockDownload, handshake::Handshake, handshake_data::HandshakeData,
            initial_headers_download::InitialHeaderDownload, message_response::MessageResponse,
            peer_manager::PeerManager,
        },
        notifications::{notification::Notification, notifier::Notifier},
    };

    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::mpsc::channel,
        sync::{Arc, Mutex},
    };

    fn read_message<M: Message>(stream: &mut Stream, message_type: CommandName) -> M {
        let header = message::deserialize_until_found(stream, message_type).unwrap();
        assert_eq!(header.command_name, message_type);
        M::deserialize_message(stream, header).unwrap()
    }

    #[derive(Clone)]
    struct NotificationMock {}

    impl Notifier for NotificationMock {
        fn notify(&self, _notification: Notification) {}
    }

    #[test]
    fn test01_program_run_correctly() {
        let mut stream: Stream = Stream::new();
        let magic_numbers = [11, 17, 9, 7];

        let handshake_data = HandshakeData {
            nonce: 0,
            user_agent: "".to_string(),
            relay: false,
            magic_number: magic_numbers.clone(),
        };

        let local_ip: (Ipv4Addr, u16) = (Ipv4Addr::new(127, 0, 0, 1), 8333);
        let remote_ip: (Ipv4Addr, u16) = (Ipv4Addr::new(127, 0, 0, 2), 8333);

        let p2p_protocol = ProtocolVersionP2P::V70016;
        let services = BitfieldServices::new(vec![SupportedServices::Unname]);
        let block_height = 0;

        let local_socket = SocketAddr::new(IpAddr::V4(local_ip.0), local_ip.1);
        let potential_peer = SocketAddr::new(IpAddr::V4(remote_ip.0), remote_ip.1);

        serialize_message::serialize_version_message(
            &mut stream,
            p2p_protocol.clone(),
            services.clone(),
            block_height,
            handshake_data.clone(),
            local_ip.clone(),
            remote_ip.clone(),
        )
        .unwrap();

        serialize_message::serialize_verack_message(&mut stream, handshake_data.magic_number)
            .unwrap();

        let first_block = creation::create_genesis_block();

        let first_block_header_hash = first_block.header.get_hash256d().unwrap();

        let mut blockchain = BlockChain::new(first_block.clone()).unwrap();

        let mut block_to_append = creation::create_block(first_block_header_hash.clone(), 2);
        block_to_append
            .append_transaction(creation::create_transaction(1))
            .unwrap();
        block_to_append
            .append_transaction(creation::create_transaction(2))
            .unwrap();
        let second_block_header_hash = block_to_append.header.get_hash256d().unwrap();

        serialize_message::serialize_headers_message(
            &mut stream,
            magic_numbers.clone(),
            vec![block_to_append.header.clone()],
        )
        .unwrap();

        serialize_message::serialize_block_message(
            &mut stream,
            magic_numbers.clone(),
            first_block.clone(),
        )
        .unwrap();
        serialize_message::serialize_block_message(
            &mut stream,
            magic_numbers.clone(),
            block_to_append.clone(),
        )
        .unwrap();

        let mut expected_blockchain = BlockChain::new(creation::create_genesis_block()).unwrap();
        expected_blockchain
            .append_header(creation::create_header(first_block_header_hash.clone(), 2))
            .unwrap();

        let new_transaction = creation::create_transaction(3);

        serialize_message::serialize_tx_message(
            &mut stream,
            magic_numbers.clone(),
            new_transaction.clone(),
        )
        .unwrap();

        let send_transaction = creation::create_transaction(4);

        // program

        let logger_text: Vec<u8> = Vec::new();
        let (sender, _) = logger::initialize_logger(logger_text, false);

        let handshake = Handshake::new(
            p2p_protocol.clone(),
            services,
            block_height,
            handshake_data,
            sender.clone(),
        );

        handshake
            .connect_to_peer(&mut stream, &local_socket, &potential_peer)
            .unwrap();

        let initial_headers_download =
            InitialHeaderDownload::new(p2p_protocol, magic_numbers.clone(), sender.clone());

        initial_headers_download
            .get_headers(&mut stream, &mut blockchain)
            .unwrap();

        assert_eq!(expected_blockchain, blockchain);

        let hashed_headers: Vec<HashType> = vec![first_block_header_hash, second_block_header_hash];

        let block_download = BlockDownload::new(magic_numbers.clone(), sender.clone());

        let blocks = block_download
            .get_data(&mut stream, hashed_headers)
            .unwrap();

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks, vec![first_block, block_to_append]);

        let (sender_message, receiver_message) = channel::<MessageResponse>();
        let (sender_transaction, receiver_transaction) = channel::<Transaction>();
        let notifier = NotificationMock {};

        let peer_manager = PeerManager::new(
            stream,
            sender_message,
            receiver_transaction,
            Arc::new(Mutex::new(true)),
            magic_numbers,
            notifier,
            sender,
        );

        sender_transaction.send(send_transaction.clone()).unwrap();

        let mut stream = peer_manager.connecting_to_peer().unwrap();

        assert_eq!(
            MessageResponse::Transaction(new_transaction),
            receiver_message.try_recv().unwrap()
        );

        let _ = read_message::<VersionMessage>(&mut stream, CommandName::Version);
        let _ = read_message::<VerackMessage>(&mut stream, CommandName::Verack);
        let _ = read_message::<SendHeadersMessage>(&mut stream, CommandName::SendHeaders);

        let get_headers_message =
            read_message::<GetHeadersMessage>(&mut stream, CommandName::GetHeaders);

        assert_eq!(
            get_headers_message.header_locator_hashes,
            vec![first_block_header_hash]
        );

        let transaction_message = read_message::<TxMessage>(&mut stream, CommandName::Tx);

        assert_eq!(transaction_message.transaction, send_transaction);
    }
}
