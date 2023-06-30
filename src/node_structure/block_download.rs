use super::error_node::ErrorNode;

use crate::messages::{
    block_message::BlockMessage,
    command_name::CommandName,
    get_data_message::GetDataMessage,
    message::{self, Message},
    message_header::MagicType,
};

use crate::logs::logger_sender::LoggerSender;

use crate::block_structure::{block::Block, hash::HashType};

use std::io::{Read, Write};

const MAX_HEADERS_COUNT: usize = 50_000;

/// It represents the download of blocks given the headers to the block to download
#[derive(Debug, Clone)]
pub struct BlockDownload {
    magic_numbers: MagicType,
    sender_log: LoggerSender,
}

impl BlockDownload {
    pub fn new(magic_numbers: MagicType, sender_log: LoggerSender) -> Self {
        BlockDownload {
            magic_numbers,
            sender_log,
        }
    }

    /// It sends a get data message to the peer given the hashed headers
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    fn send_get_data_message<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
        hashed_headers: Vec<HashType>,
    ) -> Result<(), ErrorNode> {
        let _ = self.sender_log.log_connection("Getting data".to_string());

        let get_data_message = GetDataMessage::get_blocks(hashed_headers);

        GetDataMessage::serialize_message(peer_stream, self.magic_numbers, &get_data_message)?;

        Ok(())
    }

    /// It receives the blocks from the peer
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    ///  * `ErrorNode::WhileDeserialization`: It will appear when there is an error in the deserialization
    fn receive_blocks<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
        headers_count: usize,
    ) -> Result<Vec<Block>, ErrorNode> {
        let mut blocks: Vec<Block> = Vec::new();
        for i in 0..headers_count {
            if i % 100 == 0 {
                let _ = self
                    .sender_log
                    .log_connection(format!("Getting blocks [{i}]"));
            }

            let header = message::deserialize_until_found(peer_stream, CommandName::Block)?;
            let block_message = BlockMessage::deserialize_message(peer_stream, header)?;

            if !block_message.block.proof_of_inclusion() {
                return Err(ErrorNode::WhileValidating(
                    "Failed proof of inclusion".to_string()
                ));
            }

            blocks.push(block_message.block);
        }

        Ok(blocks)
    }

    /// Get the blocks from the peer given the hashed headers
    ///
    /// ### Error
    ///  * `ErrorNode::WhileSerializing`: It will appear when there is an error in the serialization
    ///  * `ErrorNode::WhileDeserialization`: It will appear when there is an error in the deserialization
    ///  * `ErrorNode::RequestedDataTooBig`: It will appear when the headers count is bigger than the maximum headers count of 50_000
    pub fn get_data<RW: Read + Write>(
        &self,
        peer_stream: &mut RW,
        hashed_headers: Vec<HashType>,
    ) -> Result<Vec<Block>, ErrorNode> {
        let headers_count = hashed_headers.len();

        if headers_count >= MAX_HEADERS_COUNT {
            let _ = self
                .sender_log
                .log_connection("More headers than possible".to_string());
            return Err(ErrorNode::RequestedDataTooBig);
        }

        self.send_get_data_message(peer_stream, hashed_headers)?;

        let _ = self
            .sender_log
            .log_connection(format!("Downloading {headers_count} blocks",));

        self.receive_blocks(peer_stream, headers_count)
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
            merkle_tree::MerkleTree,
        },
        connections::type_identifier::TypeIdentifier,
        logs::logger,
        messages::{compact_size::CompactSize, inventory_vector::InventoryVector},
        serialization::error_serialization::ErrorSerialization,
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

    fn serialize_block_message<RW: Read + Write>(
        stream: &mut RW,
        magic_numbers: [u8; 4],
        block: Block,
    ) -> Result<(), ErrorSerialization> {
        let block_message = BlockMessage { block };
        BlockMessage::serialize_message(stream, magic_numbers, &block_message)
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

    fn create_empty_block(transaction_count: u64) -> Block {
        Block::new(BlockHeader::new(
            BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(u32::MAX),
            0,
            CompactSize::new(transaction_count),
        ))
    }

    fn create_block(previous_header: HashType, transaction_count: u64) -> Block {
        Block::new(BlockHeader::new(
            BlockVersion::version(1),
            previous_header,
            [0; 32],
            0,
            Compact256::from(u32::MAX),
            0,
            CompactSize::new(transaction_count),
        ))
    }

    fn update_merkle_root_hash(block: &mut Block) {
        let merkle_tree = MerkleTree::new(&block.transactions).unwrap();
        block.header.merkle_root_hash = merkle_tree.root;
    }

    #[test]
    fn test01_block_download_successfully() {
        let mut stream = Stream::new();
        let magic_numbers = [11, 17, 9, 7];

        let mut first_block = create_empty_block(3);
        let first_block_header_hash = first_block.header.get_hash256d().unwrap();
        first_block
            .append_transaction(create_transaction(0))
            .unwrap();
        first_block
            .append_transaction(create_transaction(1))
            .unwrap();
        first_block
            .append_transaction(create_transaction(2))
            .unwrap();

        update_merkle_root_hash(&mut first_block);

        let mut second_block = create_block(first_block_header_hash, 3);
        let second_block_header_hash = second_block.header.get_hash256d().unwrap();
        second_block
            .append_transaction(create_transaction(1))
            .unwrap();
        second_block
            .append_transaction(create_transaction(2))
            .unwrap();
        second_block
            .append_transaction(create_transaction(3))
            .unwrap();

        update_merkle_root_hash(&mut second_block);

        serialize_block_message(&mut stream, magic_numbers.clone(), first_block.clone()).unwrap();
        serialize_block_message(&mut stream, magic_numbers.clone(), second_block.clone()).unwrap();

        let hashed_headers: Vec<HashType> = vec![first_block_header_hash, second_block_header_hash];

        let logger_text: Vec<u8> = Vec::new();
        let (sender, _) = logger::initialize_logger(logger_text, false);
        let block_download = BlockDownload::new(magic_numbers, sender);

        let blocks = block_download
            .get_data(&mut stream, hashed_headers)
            .unwrap();

        assert_eq!(blocks, vec![first_block, second_block]);

        let header = message::deserialize_until_found(&mut stream, CommandName::GetData).unwrap();

        assert_eq!(header.command_name, CommandName::GetData);

        let get_data_message = GetDataMessage::deserialize_message(&mut stream, header).unwrap();

        let expected_inventory_vectors: Vec<InventoryVector> = vec![
            InventoryVector::new(TypeIdentifier::Block, first_block_header_hash),
            InventoryVector::new(TypeIdentifier::Block, second_block_header_hash),
        ];

        assert_eq!(
            get_data_message.inventory_vectors,
            expected_inventory_vectors
        );
    }
}
