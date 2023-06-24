use super::{hash::hash256d, outpoint::Outpoint, transaction::Transaction};

use crate::serialization::{
    deserializable_internal_order::DeserializableInternalOrder,
    deserializable_little_endian::DeserializableLittleEndian,
    error_serialization::ErrorSerialization,
    serializable_internal_order::SerializableInternalOrder,
    serializable_little_endian::SerializableLittleEndian,
};

use crate::wallet_structure::{account::Account, error_wallet::ErrorWallet};

use crate::messages::compact_size::CompactSize;

use std::{
    cmp::PartialEq,
    io::{Read, Write},
};

const DEFAULT_SEQUENCE: u32 = 0xFFFFFFFF;
const SIGHASH_ALL_MESSAGE: [u8; 4] = [0x01, 0x00, 0x00, 0x00];
const SIGHASH_ALL_SIG_SCRIPT: u8 = 1;

/// It's the representation of a transaction input
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

    /// It create the signature script from the given transaction
    ///
    /// ### Error
    ///  * `ErrorWallet::CannotCreateNewTransaction`: It will appear when a transaction cannot be created
    pub fn create_signature_script(
        account: &Account,
        unsigned_transaction: Transaction,
        input_index: usize,
    ) -> Result<Vec<u8>, ErrorWallet> {
        let mut unsigned_transaction = unsigned_transaction;
        unsigned_transaction.tx_in[input_index].signature_script =
            account.address.generate_script_pubkey_p2pkh();

        let mut message: Vec<u8> = Vec::new();
        if let Err(e) = unsigned_transaction.io_serialize(&mut message) {
            return Err(ErrorWallet::CannotCreateNewTransaction(format!(
                "Error serializing the transaction to sign: {:?}",
                e
            )));
        };

        message.extend(SIGHASH_ALL_MESSAGE.clone());

        let hashed_message = match hash256d(&message) {
            Ok(hashed_message) => hashed_message,
            Err(e) => {
                return Err(ErrorWallet::CannotCreateNewTransaction(format!(
                    "Error hashing the transaction to sign: {:?}",
                    e
                )))
            }
        };

        let mut signed_message = account.sign(&hashed_message)?;

        signed_message.push(SIGHASH_ALL_SIG_SCRIPT);

        let mut final_script_signature = vec![];
        final_script_signature.push(signed_message.len() as u8);
        final_script_signature.extend(signed_message);

        final_script_signature.push(account.public_key.as_bytes().len() as u8);
        final_script_signature.extend(account.public_key.as_bytes());

        Ok(final_script_signature)
    }

    /// It create a new transaction input from the given outpoint
    pub fn from_outpoint_unsigned(outpoint: &Outpoint) -> TransactionInput {
        let signature_script = vec![];
        let sequence = DEFAULT_SEQUENCE;
        TransactionInput::new(outpoint.clone(), signature_script, sequence)
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
