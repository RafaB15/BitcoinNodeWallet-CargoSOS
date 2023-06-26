use super::{command_name::CommandName, message::Message};

use crate::block_structure::block::Block;

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
};

use std::io::{Read, Write};

/// It's the block message
#[derive(Debug, std::cmp::PartialEq)]
pub struct BlockMessage {
    pub block: Block,
}

impl Message for BlockMessage {
    fn get_command_name() -> CommandName {
        CommandName::Block
    }
}

impl SerializableInternalOrder for BlockMessage {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.block.io_serialize(stream)?;
        Ok(())
    }
}

impl DeserializableInternalOrder for BlockMessage {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(BlockMessage {
            block: Block::io_deserialize(stream)?,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::block_structure::{
        block_version,
        block_header::BlockHeader,
        compact256::Compact256, 
        outpoint::Outpoint, 
        transaction::Transaction,
        transaction_input::TransactionInput,
        transaction_output::TransactionOutput,
    };

    use crate::messages::compact_size::CompactSize;

    #[test]
    fn test_01_correct_block_message_serialization() {
        let block_header = BlockHeader::new(
            block_version::BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(2),
        );

        let transaction_input = TransactionInput::new(
            Outpoint::new(
                [1; 32],
                23,
        ),
            vec![1, 2, 3],
            24,
        );

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![4, 5, 6],
        };

        let transaction_1 = Transaction {
            version: 1,
            tx_in: vec![transaction_input],
            tx_out: vec![transaction_output],
            time: 0,
        };

        let transaction_2 = transaction_1.clone();

        let transactions = vec![transaction_1, transaction_2];

        let block = Block {
            header: block_header.clone(),
            transactions: transactions.clone(),
        };

        let block_message = BlockMessage {
            block: block.clone(),
        };

        let mut serialized_fields = Vec::new();
        block_header.io_serialize(&mut serialized_fields).unwrap();
        for transaction in transactions.iter() {
            transaction.io_serialize(&mut serialized_fields).unwrap();
        }

        let mut serialized_block_message = Vec::new();
        block_message.io_serialize(&mut serialized_block_message).unwrap();

        assert_eq!(serialized_fields, serialized_block_message);
    }

    #[test]
    fn test_02_correct_block_message_deserialization() {
        let block_header = BlockHeader::new(
            block_version::BlockVersion::version(1),
            [0; 32],
            [0; 32],
            0,
            Compact256::from(10),
            0,
            CompactSize::new(2),
        );

        let transaction_input = TransactionInput::new(
            Outpoint::new(
                [1; 32],
                23,
            ),
            vec![1, 2, 3],
            24,
        );

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![4, 5, 6],
        };

        let transaction_1 = Transaction {
            version: 1,
            tx_in: vec![transaction_input],
            tx_out: vec![transaction_output],
            time: 0,
        };

        let transaction_2 = transaction_1.clone();

        let transactions = vec![transaction_1, transaction_2];

        let block = Block {
            header: block_header.clone(),
            transactions: transactions.clone(),
        };

        let block_message = BlockMessage {
            block: block.clone(),
        };

        let mut serialized_block_message = Vec::new();
        block_message.io_serialize(&mut serialized_block_message).unwrap();

        let deserialized_block = BlockMessage::io_deserialize(&mut serialized_block_message.as_slice()).unwrap();

        assert_eq!(block_message, deserialized_block);

    }
}