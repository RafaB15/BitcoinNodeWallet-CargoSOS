use super::error_node::ErrorNode;

use crate::{
    block_structure::{block_chain::BlockChain, block_header::BlockHeader, hash::HashType},
    connections::p2p_protocol::ProtocolVersionP2P,
    logs::logger_sender::LoggerSender,
};

use crate::messages::{
    command_name::CommandName,
    get_headers_message::GetHeadersMessage,
    headers_message::HeadersMessage,
    message::{self, Message},
    message_header::MagicType,
};

use std::io::{Read, Write};

const NO_STOP_HASH: HashType = [0; 32];

/// It represents the download of the headers from a peer
#[derive(Debug, Clone)]
pub struct InitialHeaderDownload {
    protocol_version: ProtocolVersionP2P,
    magic_number: MagicType,
    sender_log: LoggerSender,
}

impl InitialHeaderDownload {
    pub fn new(
        protocol_version: ProtocolVersionP2P,
        magic_number: MagicType,
        sender_log: LoggerSender,
    ) -> Self {
        InitialHeaderDownload {
            protocol_version,
            magic_number,
            sender_log,
        }
    }

    /// It sends a get headers message to the peer given the latest headers from the blockchain
    ///
    /// ### Error
    ///  * `ErrorNode::InSerialization`: It will appear when the serialization of the message fails or the SHA(SHA(header)) fails
    fn send_get_headers_message<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
        block_chain: &BlockChain,
    ) -> Result<(), ErrorNode> {
        let _ = self
            .sender_log
            .log_connection("Serializing last headers from blockchain".to_string());

        let mut header_locator_hashes: Vec<HashType> = Vec::new();

        for block in block_chain.latest().iter() {
            let last_header: &BlockHeader = &block.header;
            header_locator_hashes.push(last_header.get_hash256d()?);
        }

        let get_headers_message =
            GetHeadersMessage::new(self.protocol_version, header_locator_hashes, NO_STOP_HASH);

        GetHeadersMessage::serialize_message(peer_stream, self.magic_number, &get_headers_message)?;

        let _ = self
            .sender_log
            .log_connection("Sending the message".to_string());

        Ok(())
    }

    /// Updates the block chain with the headers received from the peer
    ///
    /// ### Error
    ///  * `ErrorNode::InSerialization`: It will appear when the serialization of the message fails or the SHA(SHA(header)) fails
    ///  * `ErrorNode::NodeNotResponding`: It will appear when no message is received from the node
    ///  * `ErrorNode::WhileValidating`: It will appear when a given header does not pass the proof of work to be added to the blockchain
    pub fn get_headers<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
        block_chain: &mut BlockChain,
    ) -> Result<u32, ErrorNode> {
        let _ = self
            .sender_log
            .log_connection("Sending get headers message".to_string());

        self.send_get_headers_message(peer_stream, block_chain)?;

        let header_headers_message =
            match message::deserialize_until_found(peer_stream, CommandName::Headers) {
                Ok(header) => header,
                Err(error) => {
                    return Err(ErrorNode::NodeNotResponding(format!(
                        "Error while receiving headers message: {:?}",
                        error
                    )))
                }
            };

        let _ = self
            .sender_log
            .log_connection("Receiving headers message".to_string());

        let received_headers_message =
            match HeadersMessage::deserialize_message(peer_stream, header_headers_message) {
                Ok(headers_message) => headers_message,
                Err(error) => {
                    return Err(ErrorNode::NodeNotResponding(format!(
                        "Error while receiving headers message: {:?}",
                        error
                    )))
                }
            };

        match block_chain.append_headers(received_headers_message.headers) {
            Ok(count) => Ok(count),
            Err(error) => Err(ErrorNode::WhileValidating(format!(
                "Error while validating headers: {:?}",
                error
            ))),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::{
        block_structure::{
            block::Block, 
            error_block::ErrorBlock,
            compact256::Compact256,
            block_version::BlockVersion,
        }, 
        node_structure::initial_headers_download,
        messages::compact_size::CompactSize,
        logs::{
            logger,
            logger_sender::LoggerSender,
        },
        serialization::error_serialization::ErrorSerialization,
    };

    struct Stream {
        stream: Vec<u8>,
        pointer: usize,
    }

    impl Stream {
        pub fn new() -> Stream {
            Stream { stream: Vec::new(), pointer: 0 }
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

    fn serialize_headers_message<RW: Read + Write>(stream: &mut RW, magic_numbers: [u8; 4], headers: Vec<BlockHeader>) -> Result<(), ErrorSerialization> {
        let headers_message = HeadersMessage { headers };
        HeadersMessage::serialize_message(stream, magic_numbers, &headers_message)
    }

    #[test]
    fn test01_initial_header_download_successfully() {
        let mut stream = Stream::new();
        let magic_numbers = [11, 17, 9, 7];

        let block = Block::new(BlockHeader::new(
            BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(0),
        ));

        let hash_of_first_block_header = block.header.get_hash256d().unwrap();

        let mut blockchain = BlockChain::new(block).unwrap();

        let header_to_append = BlockHeader::new(
            BlockVersion::version(1),
            hash_of_first_block_header.clone(),
            [3; 32],
            5,
            Compact256::from(u32::MAX),
            21,
            CompactSize::new(0),
        );

        serialize_headers_message(&mut stream, magic_numbers.clone(), vec![header_to_append.clone()]).unwrap();

        let mut expected_blockchain = blockchain.clone();
        expected_blockchain.append_header(header_to_append.clone()).unwrap();

        let mut logger_text: Vec<u8> = Vec::new();
        let (sender, _) = logger::initialize_logger(logger_text, false);

        let initial_headers_download = InitialHeaderDownload::new(
            ProtocolVersionP2P::V70016,
            magic_numbers,
            sender,
        );

        initial_headers_download.get_headers(&mut stream, &mut blockchain).unwrap();

        assert_eq!(expected_blockchain, blockchain);        
    }
}