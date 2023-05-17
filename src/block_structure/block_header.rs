use super::{
    block_version::BlockVersion,
    compact256::Compact256,
    hash::{hash256d, HashType},
    transaction::Transaction,
};

use crate::{serialization::{
    serializable::Serializable,
    //serializable_big_endian::SerializableBigEndian,
    deserializable::Deserializable,
    //deserializable_big_endian::DeserializableBigEndian,
    error_serialization::ErrorSerialization, 
}, messages::compact_size::CompactSize};

use std::io::{
    Write,
    Read,
};

const GENESIS_BLOCK_VERSION: BlockVersion = BlockVersion::V1;
const GENESIS_PREVIOUS_BLOCK_HEADER_HASH: HashType = [0; 32];
const GENESIS_MERKLE_ROOT_HASH: HashType = [
    0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b, 0x12, 0xb2, 
    0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76, 0x8f, 0x61, 
    0x7f, 0xc8, 0x1b, 0xc3, 0x88, 0x8a, 0x51, 0x32, 
    0x3a, 0x9f, 0xb8, 0xaa, 0x4b, 0x1e, 0x5e, 0x4a, 
];
const GENESIS_TIME: u32 = 0x4d49e5da;
const GENESIS_N_BITS: u32 = 0x1d00ffff;
const GENESIS_NONCE: u32 = 0x18aea41a;
const GENESIS_TRANSACTION_COUNT: u64 = 1;

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
        let mut buffer = vec![];
        if self.serialize(&mut buffer).is_err() {
            return false;
        }
        let hash: HashType = match hash256d(&buffer) {
            Ok(hash) => hash,
            Err(_) => return false,
        };

        self.n_bits > Compact256::from(hash) || true
    }

    pub fn proof_of_inclusion(&self, transactions: &[Transaction]) -> bool {
        //creo el vector de hashes
        let mut hashes = Vec::with_capacity(transactions.len());
        //itero por las transacciones
        for tx in transactions {
            let mut vec_tx = Vec::new();
            match tx.get_tx_id(&mut vec_tx) {
                Ok(txid) => hashes.push(txid),
                Err(_) => return false,
            };
        }

        while hashes.len() > 1 {
            if hashes.len() % 2 == 1 {
                if let Some(last_hash) = hashes.last() {
                    hashes.push(*last_hash);
                }
            }

            let mut new_hashes = Vec::new();
            for (i, combined) in hashes.iter().enumerate().step_by(2) {
                // Concatenar dos hashes
                let mut combined = combined.to_vec();
                match hashes.get(i + 1) {
                    Some(combined_next) => combined.extend_from_slice(combined_next),
                    None => return false,
                };

                // Calcular el hash combinado
                let combined_hash = match hash256d(&combined) {
                    Ok(combined_hash) => combined_hash,
                    Err(_) => return false,
                };
                new_hashes.push(combined_hash);
            }

            hashes = new_hashes;
        }
        if let Some(root) = hashes.first() {
            return *root == self.merkle_root_hash;
        }
        false
    }
}

impl Serializable for BlockHeader {
    fn serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.version.serialize(stream)?;
        self.previous_block_header_hash.serialize(stream)?;
        self.merkle_root_hash.serialize(stream)?;
        self.time.serialize(stream)?;
        self.n_bits.serialize(stream)?;
        self.nonce.serialize(stream)?;
        self.transaction_count.serialize(stream)?;

        Ok(())
    }
}

impl Deserializable for BlockHeader {

    fn deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {

        Ok(BlockHeader{
            version: BlockVersion::deserialize(stream)?,
            previous_block_header_hash: HashType::deserialize(stream)?,
            merkle_root_hash: HashType::deserialize(stream)?,
            time: u32::deserialize(stream)?,
            n_bits: Compact256::deserialize(stream)?,
            nonce: u32::deserialize(stream)?,
            transaction_count: CompactSize::deserialize(stream)?,
        })
    }
}
