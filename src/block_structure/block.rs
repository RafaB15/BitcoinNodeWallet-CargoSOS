use bitcoin_hashes::sha256d;

use super::{
    block_header::BlockHeader, 
    transaction::Transaction,
    transaction_output::TransactionOutput,
    error_block::ErrorBlock,
    hash::{
        HashType,
        hash256d,
    },
};

use crate::serialization::{
    serializable::Serializable,
    deserializable::Deserializable,
    error_serialization::ErrorSerialization,
};

use crate::messages::{
    compact_size::CompactSize,
};

#[derive(Debug, Clone)]
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

    pub fn remove_spent_transactions(&self, utxo_from_address: &mut Vec<(TransactionOutput, HashType, u32)>) {
        for transaction in &self.transactions {
            for input in &transaction.tx_in {
                for (output, transaction_hash, index) in utxo_from_address.iter_mut() {
                    if input.previos_output.hash.eq(transaction_hash)  && input.previos_output.index == *index{
                        output.value = 0;
                    }
                }
            }
        }
        utxo_from_address.retain(|(output, _, _)| output.value != 0);
    }

    pub fn add_utxo_from_address(&self, address: &str, utxo_from_address: &mut Vec<(TransactionOutput, HashType, u32)>) {
        let index_utxo = 0;
        for transaction in &self.transactions {
            index_utxo = 0;
            for output in &transaction.tx_out {
                if output.get_public_key_hash() == address {
                    let serialized_transaction = Vec::new();
                    match transaction.serialize(&mut serialized_transaction) {
                        Ok(_) => (),
                        Err(_) => return,
                    }
                    let hashed_transaction = match hash256d(&serialized_transaction) {
                        Ok(hashed_transaction) => hashed_transaction,
                        Err(_) => return,
                    };
                    utxo_from_address.push((output.clone(), hashed_transaction, index_utxo));
                }
                index_utxo += 1;
            }
        }
    }

    pub fn update_utxo_from_address(&self, address: &str, utxo_from_address: &mut Vec<(TransactionOutput, HashType, u32)>) {
        self.remove_spent_transactions(utxo_from_address);
        self.add_utxo_from_address(address, utxo_from_address);
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
