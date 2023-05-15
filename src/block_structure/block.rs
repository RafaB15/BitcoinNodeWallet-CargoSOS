use super::{
    block_header::BlockHeader, 
    transaction::Transaction,
    error_block::ErrorBlock, 
};

use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_serialization::ErrorSerialization,
};

use crate::messages::{
    compact_size::CompactSize,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(header: BlockHeader) -> Self {
        Block {
            header,
            transactions: vec![],
        }
    }

    pub fn proof_of_inclusion(&self) -> bool {
        self.header.proof_of_inclusion(&self.transactions)
    }

    pub fn append_transaction(&mut self, transaction: Transaction) -> Result<(), ErrorBlock> {

        match self.transactions.iter().any(|this_transaction| *this_transaction == transaction) {
            true => return Err(ErrorBlock::TransactionAlreadyInBlock),
            false => self.transactions.push(transaction),
        }

        Ok(())
    }
}

impl Serializable for Block {

    fn serialize(&self, stream: &mut dyn std::io::Write) -> Result<(), ErrorSerialization> {
        self.header.serialize(stream)?;
        CompactSize::new(self.transactions.len() as u64).serialize(stream)?;
        for transaction in self.transactions.iter() {
            transaction.serialize(stream)?;
        }

        Ok(())
    }
}

impl Deserializable for Block {
    fn deserialize(stream: &mut dyn std::io::Read) -> Result<Self, ErrorSerialization> {
        let header = BlockHeader::deserialize(stream)?;
        let compact_size = CompactSize::deserialize(stream)?;
        
        let mut block = Block::new(header);

        for _ in 0..compact_size.value {
            let transaction = Transaction::deserialize(stream)?;
            match block.append_transaction(transaction) {
                Ok(_) | Err(ErrorBlock::TransactionAlreadyInBlock) => continue,
                _ => return Err(ErrorSerialization::ErrorInDeserialization(
                    "Appending transactions to the block".to_string()
                )),
            }
        }

        Ok(block)
    }
}
