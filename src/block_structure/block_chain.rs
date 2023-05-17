use super::{
    block::Block, 
    block_header::BlockHeader, 
    hash::{
        HashType,
        hash256d,
    },
    transaction_output::TransactionOutput,
    error_block::ErrorBlock,
};

use crate::serialization::{
    serializable::Serializable,
};

#[derive(Debug, Clone)]
pub struct BlockChain {
    pub next_blocks: Vec<BlockChain>,
    pub block: Block,   
}

impl BlockChain {
    pub fn new(block: Block) -> Self {
        BlockChain {
            next_blocks: vec![],
            block,
        }
    }

    pub fn append_header(&mut self, header: BlockHeader) -> Result<(), ErrorBlock> {
        self.append_block(Block::new(header))
    }

    pub fn append_block(&mut self, block: Block) -> Result<(), ErrorBlock> {
    
        let previous_hashed_header: HashType = block.header.previous_block_header_hash;

        let mut serialized_header: Vec<u8> = Vec::new();
        if self.block.header.serialize(&mut serialized_header).is_err() {
            return Err(ErrorBlock::CouldNotSerialize);
        }

        let hashed_header: HashType = match hash256d(&serialized_header) {
            Ok(hashed_header) => hashed_header,
            _ => return Err(ErrorBlock::CouldNotHash),
        };

        if previous_hashed_header == hashed_header {
            self.next_blocks.push(BlockChain::new(block));    
            return Ok(())
        }
        
        for next_block in self.next_blocks.iter_mut() {

            let block_clone = block.clone();
            match next_block.append_block(block_clone) {
                Err(ErrorBlock::CouldNotAppendBlock) | Ok(_) => continue,
                err => return err,
            }
        }
        
        Err(ErrorBlock::CouldNotAppendBlock)
    }

    pub fn update_block(&mut self, block: Block) -> Result<(), ErrorBlock> {
        
        if self.block.header == block.header {
            self.block = block;
            return Ok(())
        }

        for next_block in self.next_blocks.iter_mut() {

            let block_clone = block.clone();
            match next_block.update_block(block_clone) {
                Err(ErrorBlock::CouldNotUpdate) | Ok(_) => continue,
                err => return err,
            }
        }

        Err(ErrorBlock::CouldNotUpdate)
    }

    pub fn get_block_after_timestamp(&self, timestamp: u32) -> Result<BlockChain, ErrorBlock> {
        
        if timestamp < self.block.header.time {
            return Ok(self.clone());
        }

        for next_block in self.next_blocks.iter() {
            match next_block.get_block_after_timestamp(timestamp) {
                Ok(block_after_timestamp) => return Ok(block_after_timestamp),
                Err(ErrorBlock::CouldNotFindBlockFarEnough) => continue,
                err => return err,
            }
        }

        Err(ErrorBlock::CouldNotFindBlockFarEnough)
    }

    pub fn last(&self) -> Vec<Block> {
        
        if self.next_blocks.is_empty() {
            return vec![self.block.clone()];
        }

        let mut last_blocks: Vec<Block> = Vec::new();
        for next_block in self.next_blocks.iter() {

            last_blocks.extend(next_block.last());
        }

        last_blocks
    }

    pub fn get_utxo(&self) -> Vec<TransactionOutput> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::block_structure::{
        compact256::Compact256,
        hash::hash256d,
        block_version,
        transaction_input::TransactionInput,
        transaction_output::TransactionOutput,
        transaction::Transaction,
        outpoint::Outpoint,
    };
    use crate::serialization::serializable::Serializable;
    use super::*;

    #[test]
    fn test_01_correct_append_header() {
        let mut blockchain = BlockChain::new(
            Block::new(
                BlockHeader::new(
                    block_version::BlockVersion::V1,
                    [0; 32],
                    [0; 32],
                    0,
                    Compact256::from_u32(10),
                    0,
                )
            )
        );
        let mut serialized_blockchain_header = Vec::new();
        blockchain.block.header.serialize(&mut serialized_blockchain_header).unwrap();
        let hash_of_first_block_header = hash256d(&serialized_blockchain_header).unwrap();

        let header_to_append = BlockHeader::new(
                block_version::BlockVersion::V1,
                hash_of_first_block_header.clone(),
                [0; 32],
                0,
                Compact256::from_u32(10),
                0,
        );

        blockchain.append_header(header_to_append.clone()).unwrap();
        assert_eq!(blockchain.next_blocks[0].block.header, header_to_append);

    }

    #[test]
    fn test_02_correct_block_update() {

        let transaction_input = TransactionInput::new(
            Outpoint { hash: [1;32], index: 23 },
            String::from("Prueba in"),
            24
        );

        let transaction_output = TransactionOutput{
            value: 10, 
            pk_script: String::from("Prueba out")
        };

        let transaction = Transaction {
            version : 1,
            tx_in: vec![transaction_input.clone()],
            tx_out: vec![transaction_output.clone()],
            time: 0,
        };

        let empty_block = Block::new(
            BlockHeader::new(
                block_version::BlockVersion::V1,
                [0; 32],
                [0; 32],
                0,
                Compact256::from_u32(10),
                0,
            )
        );

        let mut block_with_transactions = empty_block.clone();
        block_with_transactions.append_transaction(transaction.clone()).unwrap();

        let mut blockchain = BlockChain::new(empty_block);

        blockchain.update_block(block_with_transactions).unwrap();

        assert_eq!(blockchain.block.transactions[0], transaction);

    } 

    #[test]
    fn test_03_correct_get_block_after_timestamp() {

        let mut blockchain = BlockChain::new(
            Block::new(
                BlockHeader::new(
                    block_version::BlockVersion::V1,
                    [0; 32],
                    [0; 32],
                    0,
                    Compact256::from_u32(10),
                    0,
                )
            )
        );

        let mut serialized_blockchain_header = Vec::new();
        blockchain.block.header.serialize(&mut serialized_blockchain_header).unwrap();
        let hash_of_first_block_header = hash256d(&serialized_blockchain_header).unwrap();

        let header_to_append = BlockHeader::new(
                block_version::BlockVersion::V1,
                hash_of_first_block_header.clone(),
                [3; 32],
                5,
                Compact256::from_u32(10),
                21,
        );

        blockchain.append_header(header_to_append.clone()).unwrap();

        let block_after_timestamp = blockchain.get_block_after_timestamp(3).unwrap();
        assert_eq!(block_after_timestamp.block.header, header_to_append);

    }

    #[test]
    fn test_04_correct_get_last() {
    
            let mut blockchain = BlockChain::new(
                Block::new(
                    BlockHeader::new(
                        block_version::BlockVersion::V1,
                        [0; 32],
                        [0; 32],
                        0,
                        Compact256::from_u32(10),
                        0,
                    )
                )
            );
    
            let mut serialized_blockchain_header = Vec::new();
            blockchain.block.header.serialize(&mut serialized_blockchain_header).unwrap();
            let hash_of_first_block_header = hash256d(&serialized_blockchain_header).unwrap();
    
            let header_to_append = BlockHeader::new(
                    block_version::BlockVersion::V1,
                    hash_of_first_block_header.clone(),
                    [3; 32],
                    5,
                    Compact256::from_u32(10),
                    21,
            );
    
            blockchain.append_header(header_to_append.clone()).unwrap();
    
            let last_blocks = blockchain.last();
            assert_eq!(last_blocks[0].header, header_to_append);
    }

}