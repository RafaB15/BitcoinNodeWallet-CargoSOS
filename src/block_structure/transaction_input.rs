use super::outpoint;
use super::{
    outpoint::Outpoint,
    transaction_output::TransactionOutput,
};

use crate::messages::compact_size::CompactSize;

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};
use crate::wallet_structure::account::Account;

use std::io::{Read, Write};

use std::cmp::PartialEq;

const DEFAULT_SEQUENCE: u32 = 0xFFFFFFFF;
const SIGHASH_ALL: u8 = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct TransactionInput {
    pub previous_output: Outpoint,
    pub signature_script: Vec<u8>,
    pub sequence: u32,
}

impl TransactionInput {
    pub fn new(
        previous_output: Outpoint,
        signature_script: Vec<u8>,
        sequence: u32,
    ) -> TransactionInput {
        TransactionInput {
            previous_output,
            signature_script,
            sequence,
        }
    }
    
    pub fn create_signature_script(
        output_information: &(Outpoint, TransactionOutput),
        account: &Account,
    ) -> Result<Vec<u8>, ErrorSerialization> {
        let mut signature_script: Vec<u8> = Vec::new();

        let outpoint = output_information.0.clone();
        let output_to_spend = output_information.1.clone();
        let previous_pubkey_script = output_to_spend.pk_script.clone();

        let mut transaction_to_sign = TransactionInput::new(
            outpoint,
            previous_pubkey_script,
            DEFAULT_SEQUENCE,
        );

        let mut message: Vec<u8> = Vec::new();
        if let Err(e) = transaction_to_sign.io_serialize(&mut transaction_to_sign_serialized) {
            return Err(e);
        };

        let mut singed_message = account.sign(&message)?;

        transaction_to_sign_serialized.push(SIGHASH_ALL);
        


            
    }
    
    pub fn from_output(
        output_information: &(Outpoint, TransactionOutput),
    ) {
        let 
        let outpoint = output_information.0.clone();


    }
    
}

impl SerializableInternalOrder for TransactionInput {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.previous_output.io_serialize(stream)?;

        CompactSize::new(self.signature_script.len() as u64).le_serialize(stream)?;
        self.signature_script.io_serialize(stream)?;

        self.sequence.le_serialize(stream)?;

        Ok(())
    }
}

impl DeserializableInternalOrder for TransactionInput {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let previous_output = Outpoint::io_deserialize(stream)?;
        let length_sginature = CompactSize::le_deserialize(stream)?.value;

        let mut signature_script: Vec<u8> = Vec::new();
        for _ in 0..length_sginature {
            signature_script.push(u8::le_deserialize(stream)?);
        }
        let sequence = u32::le_deserialize(stream)?;

        Ok(TransactionInput {
            previous_output,
            signature_script,
            sequence,
        })
    }
}
