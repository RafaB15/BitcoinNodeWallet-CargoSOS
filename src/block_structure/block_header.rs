use super::{
    block_version::BlockVersion, 
    transaction::Transaction,
    hash::HashType,
};

use crate::serialization::{
    serializable::Serializable,
    error_serialization::ErrorSerialization,
};

use std::io::{
    Write,
};


const GENESIS_BLOCK_VERSION: BlockVersion = BlockVersion::V1;
const GENESIS_PREVIOUS_BLOCK_HEADER_HASH: HashType = [0; 32];
const GENESIS_MERKLE_ROOT_HASH: HashType = [0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b, 0x12, 0xb2, 0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76, 0x8f, 0x61, 0x7f, 0xc8, 0x1b, 0xc3, 0x88, 0x8a, 0x51, 0x32, 0x3a, 0x9f, 0xb8, 0xaa, 0x4b, 0x1e, 0x5e, 0x4a];
const GENESIS_TIME: u32 = 1231013705;
const GENESIS_N_BITS: u32 = 0x1d00ffff;
const GENESIS_NONCE: u32 = 2083236893;

pub struct BlockHeader {
    pub version: BlockVersion,
    pub previous_block_header_hash: HashType,
    pub merkle_root_hash: HashType,
    pub time: u32,
    pub n_bits: u32,
    pub nonce: u32,
}

impl BlockHeader {
    pub fn new(
        version: BlockVersion,
        previous_block_header_hash: HashType,
        merkle_root_hash: HashType,
        time: u32,
        n_bits: u32,
        nonce: u32,
    ) -> Self {
        BlockHeader {
            version,
            previous_block_header_hash,
            merkle_root_hash,
            time,
            n_bits,
            nonce,
        }
    }

    pub fn generate_genesis_block_header() -> Self {
        BlockHeader::new(
            GENESIS_BLOCK_VERSION,
            GENESIS_PREVIOUS_BLOCK_HEADER_HASH,
            GENESIS_MERKLE_ROOT_HASH,
            GENESIS_TIME,
            GENESIS_N_BITS,
            GENESIS_NONCE,
        )
    }

    pub fn proof_of_work(&self) -> bool {
        todo!()
    }

    pub fn proof_of_inclusion(&self, transactions: &[Transaction]) -> bool {
        //iterar por transacciones, calcular hash de cada una y comparar con merkle_root_hash
        for transaction in transactions {
            hash(transaction.serialize());
            /*if transaction.hash() != self.merkle_root_hash {
                return false;
            }*/
        }
        //1. iterar por transacciones -> obtener el txid -> (serializacion y hash)
        //2. concatenar cada 2 txid
        //3. aplicar sha256d
        //4. repetir desde el paso 2 hasta que quede un solo hash
        todo!()
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

        Ok(())
    }
}