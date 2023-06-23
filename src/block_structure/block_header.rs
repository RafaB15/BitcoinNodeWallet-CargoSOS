use super::{
    block_version::BlockVersion,
    compact256::Compact256,
    hash::{hash256d, HashType},
    merkle_tree::MerkleTree,
    transaction::Transaction,
};

use crate::{
    messages::compact_size::CompactSize,
    serialization::{
        deserializable_big_endian::DeserializableBigEndian,
        deserializable_internal_order::DeserializableInternalOrder,
        deserializable_little_endian::DeserializableLittleEndian,
        error_serialization::ErrorSerialization, serializable_big_endian::SerializableBigEndian,
        serializable_internal_order::SerializableInternalOrder,
        serializable_little_endian::SerializableLittleEndian,
    },
};

use std::io::{Read, Write};

const GENESIS_BLOCK_VERSION: BlockVersion = BlockVersion::version(1);
const GENESIS_PREVIOUS_BLOCK_HEADER_HASH: HashType = [0; 32];
const GENESIS_MERKLE_ROOT_HASH: HashType = [
    0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b, 0x12, 0xb2, 0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76, 0x8f, 0x61,
    0x7f, 0xc8, 0x1b, 0xc3, 0x88, 0x8a, 0x51, 0x32, 0x3a, 0x9f, 0xb8, 0xaa, 0x4b, 0x1e, 0x5e, 0x4a,
];
const GENESIS_TIME: u32 = 0x4d49e5da;
const GENESIS_N_BITS: u32 = 0x1d00ffff;
const GENESIS_NONCE: u32 = 0x18aea41a;
const GENESIS_TRANSACTION_COUNT: u64 = 0;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BlockHeader {
    pub version: BlockVersion,
    pub previous_block_header_hash: HashType,
    pub merkle_root_hash: HashType,
    pub time: u32,
    pub n_bits: Compact256,
    pub nonce: u32,
    pub transaction_count: CompactSize,
}

impl BlockHeader {
    pub fn new(
        version: BlockVersion,
        previous_block_header_hash: HashType,
        merkle_root_hash: HashType,
        time: u32,
        n_bits: Compact256,
        nonce: u32,
        transaction_count: CompactSize,
    ) -> Self {
        BlockHeader {
            version,
            previous_block_header_hash,
            merkle_root_hash,
            time,
            n_bits,
            nonce,
            transaction_count,
        }
    }

    pub fn generate_genesis_block_header() -> Self {
        BlockHeader::new(
            GENESIS_BLOCK_VERSION,
            GENESIS_PREVIOUS_BLOCK_HEADER_HASH,
            GENESIS_MERKLE_ROOT_HASH,
            GENESIS_TIME,
            Compact256::from(GENESIS_N_BITS),
            GENESIS_NONCE,
            CompactSize::new(GENESIS_TRANSACTION_COUNT),
        )
    }

    pub fn proof_of_work(&self) -> bool {
        let hash = match self.get_hash256d() {
            Ok(hash) => hash,
            Err(_) => return false,
        };

        let compact_hash = match Compact256::try_from(hash) {
            Ok(compact_hash) => compact_hash,
            Err(_) => return false,
        };

        self.n_bits > compact_hash
    }

    pub fn proof_of_inclusion(&self, transactions: &[Transaction]) -> bool {
        let merkle_tree: MerkleTree = match MerkleTree::new(transactions) {
            Ok(merkle_tree) => merkle_tree,
            Err(error) => {
                println!("Error while creating merkle tree: {:?}", error);
                return false;
            }
        };

        match merkle_tree.get_root() {
            Ok(root) => {
                let resultado = root == self.merkle_root_hash;
                if !resultado {
                    println!("Roots are different");
                }
                resultado
            }
            Err(error) => {
                println!("Error while getting the root: {:?}", error);
                false
            }
        }
    }

    pub fn get_hash256d(&self) -> Result<HashType, ErrorSerialization> {
        let mut buffer = vec![];

        self.version.le_serialize(&mut buffer)?;
        self.previous_block_header_hash.le_serialize(&mut buffer)?;
        self.merkle_root_hash.be_serialize(&mut buffer)?;
        self.time.le_serialize(&mut buffer)?;
        self.n_bits.le_serialize(&mut buffer)?;
        self.nonce.le_serialize(&mut buffer)?;

        let buffer = {
            let mut temp: Vec<u8> = Vec::new();

            for byte in hash256d(&buffer)?.iter().rev() {
                temp.push(*byte);
            }

            temp
        };

        let buffer: HashType = match (*buffer.as_slice()).try_into() {
            Ok(buffer) => buffer,
            _ => {
                return Err(ErrorSerialization::ErrorInSerialization(
                    "Error while getting hash 256 d".to_string(),
                ))
            }
        };

        Ok(buffer)
    }
}

impl SerializableInternalOrder for BlockHeader {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.version.le_serialize(stream)?;
        self.previous_block_header_hash.le_serialize(stream)?;
        self.merkle_root_hash.be_serialize(stream)?;
        self.time.le_serialize(stream)?;
        self.n_bits.le_serialize(stream)?;
        self.nonce.le_serialize(stream)?;
        self.transaction_count.le_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableInternalOrder for BlockHeader {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        Ok(BlockHeader {
            version: BlockVersion::le_deserialize(stream)?,
            previous_block_header_hash: HashType::le_deserialize(stream)?,
            merkle_root_hash: HashType::be_deserialize(stream)?,
            time: u32::le_deserialize(stream)?,
            n_bits: Compact256::le_deserialize(stream)?,
            nonce: u32::le_deserialize(stream)?,
            transaction_count: CompactSize::le_deserialize(stream)?,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_correct_genesis_block_header() {
        let genesis_block_header = BlockHeader::generate_genesis_block_header();

        assert_eq!(genesis_block_header.version, GENESIS_BLOCK_VERSION);
        assert_eq!(
            genesis_block_header.previous_block_header_hash,
            GENESIS_PREVIOUS_BLOCK_HEADER_HASH
        );
        assert_eq!(
            genesis_block_header.merkle_root_hash,
            GENESIS_MERKLE_ROOT_HASH
        );
        assert_eq!(genesis_block_header.time, GENESIS_TIME);
        assert_eq!(u32::from(genesis_block_header.n_bits), GENESIS_N_BITS);
        assert_eq!(genesis_block_header.nonce, GENESIS_NONCE);
        assert_eq!(
            genesis_block_header.transaction_count,
            CompactSize::new(GENESIS_TRANSACTION_COUNT)
        );
    }

    #[test]
    fn test_02_correct_header_serialization() {
        let genesis_block_header = BlockHeader::generate_genesis_block_header();

        let mut buffer = vec![];

        genesis_block_header
            .io_serialize(&mut buffer)
            .expect("Error while serializing");

        assert_eq!(buffer.len(), 81);
    }

    #[test]

    fn test_3_correct_header_deserialization() {
        let block_header_bytes = vec![0x00, 0xe0, 0xf8, 0x2c, 0x3a, 0x41, 0xed, 0xdf, 0xb7, 0x7e, 0x88, 0x5d, 0x5a, 0x15, 0x47, 0x6e, 0xbe, 0x14, 0xe4, 0x11, 0x58, 0x81, 0xed, 0xf8, 0xfc, 0x64, 0x30, 0x3e, 0x1e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x81, 0xc4, 0xf8, 0x11, 0x4e, 0x74, 0xc8, 0x6f, 0xec, 0xe0, 0xe8, 0xba, 0xfe, 0xff, 0x77, 0x3f, 0xc7, 0x3e, 0xa1, 0x8c, 0x62, 0xad, 0x08, 0x54, 0xe5, 0xf8, 0xb0, 0xc5, 0x2f, 0x68, 0x3a, 0xb5, 0x41, 0xa9, 0x95, 0x64, 0x7f, 0x5d, 0x21, 0x19, 0x4d, 0x58, 0xb1, 0x0f, 0x00, 0x00, 0x00, 0x00];
        let header = BlockHeader::io_deserialize(&mut block_header_bytes.as_slice());

        assert!(header.is_ok());
    }

    #[test]
    fn test_04_correct_proof_of_work() {
        let block_header_bytes = vec![0x00, 0xe0, 0xf8, 0x2c, 0x3a, 0x41, 0xed, 0xdf, 0xb7, 0x7e, 0x88, 0x5d, 0x5a, 0x15, 0x47, 0x6e, 0xbe, 0x14, 0xe4, 0x11, 0x58, 0x81, 0xed, 0xf8, 0xfc, 0x64, 0x30, 0x3e, 0x1e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x81, 0xc4, 0xf8, 0x11, 0x4e, 0x74, 0xc8, 0x6f, 0xec, 0xe0, 0xe8, 0xba, 0xfe, 0xff, 0x77, 0x3f, 0xc7, 0x3e, 0xa1, 0x8c, 0x62, 0xad, 0x08, 0x54, 0xe5, 0xf8, 0xb0, 0xc5, 0x2f, 0x68, 0x3a, 0xb5, 0x41, 0xa9, 0x95, 0x64, 0x7f, 0x5d, 0x21, 0x19, 0x4d, 0x58, 0xb1, 0x0f, 0x00, 0x00, 0x00, 0x00];
        let header = BlockHeader::io_deserialize(&mut block_header_bytes.as_slice()).unwrap();

        assert!(header.proof_of_work());
    }

    #[test]
    fn test_05_correct_hash_of_header() {
        let genesis_block_header = BlockHeader::generate_genesis_block_header();
        let mut genesis_hash = [0x43, 0x49, 0x7f, 0xd7, 0xf8, 0x26, 0x95, 0x71, 0x08, 0xf4, 0xa3, 0x0f, 0xd9, 0xce, 0xc3, 0xae, 0xba, 0x79, 0x97, 0x20, 0x84, 0xe9, 0x0e, 0xad, 0x01, 0xea, 0x33, 0x09, 0x00, 0x00, 0x00, 0x00];
        genesis_hash.reverse();
        assert_eq!(genesis_block_header.get_hash256d().unwrap(), genesis_hash);
    }


}
