use super::error_node::ErrorNode;

use crate::{
    block_structure::{block_chain::BlockChain, block_header::BlockHeader, hash::HashType},
    connections::p2p_protocol::ProtocolVersionP2P,
    logs::logger_sender::LoggerSender,
};

use crate::messages::{
    command_name::CommandName,
    error_message::ErrorMessage,
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
    ///  * `ErrorMessage::InSerialization`: It will appear when the serialization of the message fails or the SHA(SHA(header)) fails
    fn send_get_headers_message<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
        block_chain: &BlockChain,
    ) -> Result<(), ErrorMessage> {
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
    ///  * `ErrorMessage::InSerialization`: It will appear when the serialization of the message fails or the SHA(SHA(header)) fails
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
