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

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_01_correct_transaction_input_from_outpoint() {
        let previous_output = Outpoint::new(
            [
                0x7b, 0x1e, 0xab, 0xe0, 0x20, 0x9b, 0x1f, 0xe7, 0x94, 0x12, 0x45, 0x75, 0xef, 0x80,
                0x70, 0x57, 0xc7, 0x7a, 0xda, 0x21, 0x38, 0xae, 0x4f, 0xa8, 0xd6, 0xc4, 0xde, 0x03,
                0x98, 0xa1, 0x4f, 0x3f,
            ],
            1,
        );
        let signature_script = vec![];
        let sequence = DEFAULT_SEQUENCE;
        let transaction_input =
            TransactionInput::new(previous_output.clone(), signature_script, sequence);

        let transaction_input_from_outpoint =
            TransactionInput::from_outpoint_unsigned(&previous_output);

        assert_eq!(transaction_input, transaction_input_from_outpoint);
    }

    #[test]
    fn test_02_correct_signature_script_creation() {
        let account = Account::new(
            "Rafa",
            &[
                0x0A, 0x52, 0x65, 0x08, 0x2E, 0x24, 0x11, 0x5F, 0x77, 0x54, 0x0A, 0xB3, 0xB8, 0xC2,
                0xB9, 0x20, 0x60, 0xAA, 0x30, 0xD6, 0xD2, 0xB8, 0x1A, 0x08, 0x5D, 0x71, 0xAB, 0x37,
                0xED, 0xA7, 0x68, 0x91,
            ],
            &[
                0x03, 0xBC, 0x6D, 0x45, 0xD2, 0x10, 0x1E, 0x91, 0x28, 0xDE, 0x14, 0xB5, 0xB6, 0x68,
                0x83, 0xD6, 0x9C, 0xF1, 0xC3, 0x1A, 0x50, 0xB9, 0x6F, 0xEA, 0x2D, 0xAD, 0x4E, 0xD2,
                0x35, 0x14, 0x92, 0x4A, 0x22,
            ],
        )
        .unwrap();

        let mut transaction_to_sign_bytes: &[u8] = &[
            0x01, 0x00, 0x00, 0x00, 0x01, 0x20, 0x25, 0xEF, 0x69, 0x2C, 0xA9, 0x87, 0xB3, 0x9A,
            0x81, 0x33, 0x6E, 0xFB, 0x59, 0xB0, 0x56, 0xFB, 0x90, 0xC0, 0x3A, 0x5E, 0xA4, 0xC4,
            0x54, 0x4C, 0xF9, 0x27, 0x57, 0x61, 0x3E, 0x2E, 0xA4, 0x01, 0x00, 0x00, 0x00, 0x00,
            0xFF, 0xFF, 0xFF, 0xFF, 0x02, 0xA0, 0x86, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19,
            0x76, 0xA9, 0x14, 0x7A, 0xA8, 0x18, 0x46, 0x85, 0xCA, 0x1F, 0x06, 0xF5, 0x43, 0xB6,
            0x4A, 0x50, 0x2E, 0xB3, 0xB6, 0x13, 0x5D, 0x67, 0x20, 0x88, 0xAC, 0xD0, 0xE6, 0x43,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x76, 0xA9, 0x14, 0x4B, 0x88, 0xC1, 0xD3, 0x87,
            0x49, 0x08, 0x36, 0x57, 0x73, 0xA7, 0x65, 0xCD, 0xB0, 0x52, 0xC9, 0xEF, 0x5F, 0x1A,
            0x80, 0x88, 0xAC, 0x98, 0xFB, 0x95, 0x64,
        ];

        let transaction_to_sign =
            Transaction::io_deserialize(&mut transaction_to_sign_bytes).unwrap();
        let sig_script =
            TransactionInput::create_signature_script(&account, transaction_to_sign, 0).unwrap();
        let actual_sig_script = vec![
            0x48, 0x30, 0x45, 0x02, 0x21, 0x00, 0xBB, 0xDB, 0xD2, 0x5E, 0x68, 0x06, 0xDC, 0x4F,
            0x82, 0x3F, 0xB9, 0x1B, 0x3C, 0xFB, 0xC2, 0xF6, 0xEC, 0xDE, 0x9D, 0x36, 0x67, 0x7C,
            0x59, 0x49, 0xC2, 0x1F, 0x01, 0xEE, 0xDF, 0x27, 0x09, 0x23, 0x02, 0x20, 0x19, 0x14,
            0x62, 0x04, 0x55, 0x94, 0xBD, 0xF6, 0x8C, 0x2A, 0x9E, 0x94, 0xA0, 0x60, 0xD3, 0xBD,
            0x80, 0xF8, 0x86, 0xCC, 0xC9, 0x43, 0xAD, 0x26, 0xBB, 0xDD, 0xC6, 0x05, 0x3A, 0xB5,
            0xF6, 0x35, 0x01, 0x21, 0x03, 0xBC, 0x6D, 0x45, 0xD2, 0x10, 0x1E, 0x91, 0x28, 0xDE,
            0x14, 0xB5, 0xB6, 0x68, 0x83, 0xD6, 0x9C, 0xF1, 0xC3, 0x1A, 0x50, 0xB9, 0x6F, 0xEA,
            0x2D, 0xAD, 0x4E, 0xD2, 0x35, 0x14, 0x92, 0x4A, 0x22,
        ];
        assert_eq!(sig_script, actual_sig_script)
    }

    #[test]
    fn test_03_correct_transaction_input_serialization() {
        let previous_output = Outpoint::new(
            [
                0x7b, 0x1e, 0xab, 0xe0, 0x20, 0x9b, 0x1f, 0xe7, 0x94, 0x12, 0x45, 0x75, 0xef, 0x80,
                0x70, 0x57, 0xc7, 0x7a, 0xda, 0x21, 0x38, 0xae, 0x4f, 0xa8, 0xd6, 0xc4, 0xde, 0x03,
                0x98, 0xa1, 0x4f, 0x3f,
            ],
            1,
        );
        let signature_script = vec![1, 2, 3];
        let sequence = DEFAULT_SEQUENCE;
        let input = TransactionInput {
            previous_output,
            signature_script,
            sequence,
        };
        let input_bytes_real = [
            0x7b, 0x1e, 0xab, 0xe0, 0x20, 0x9b, 0x1f, 0xe7, 0x94, 0x12, 0x45, 0x75, 0xef, 0x80,
            0x70, 0x57, 0xc7, 0x7a, 0xda, 0x21, 0x38, 0xae, 0x4f, 0xa8, 0xd6, 0xc4, 0xde, 0x03,
            0x98, 0xa1, 0x4f, 0x3f, 0x01, 0x00, 0x00, 0x00, 0x03, 0x01, 0x02, 0x03, 0xFF, 0xFF,
            0xFF, 0xFF,
        ];

        let mut stream: Vec<u8> = Vec::new();

        input.io_serialize(&mut stream).unwrap();
        assert_eq!(stream, input_bytes_real);
    }

    #[test]
    fn test_04_correct_transaction_input_deserialization() {
        let input_bytes_real = [
            0x7b, 0x1e, 0xab, 0xe0, 0x20, 0x9b, 0x1f, 0xe7, 0x94, 0x12, 0x45, 0x75, 0xef, 0x80,
            0x70, 0x57, 0xc7, 0x7a, 0xda, 0x21, 0x38, 0xae, 0x4f, 0xa8, 0xd6, 0xc4, 0xde, 0x03,
            0x98, 0xa1, 0x4f, 0x3f, 0x01, 0x00, 0x00, 0x00, 0x03, 0x01, 0x02, 0x03, 0xFF, 0xFF,
            0xFF, 0xFF,
        ];
        let input_deserialized =
            TransactionInput::io_deserialize(&mut input_bytes_real.as_ref()).unwrap();
        let previous_output = Outpoint::new(
            [
                0x7b, 0x1e, 0xab, 0xe0, 0x20, 0x9b, 0x1f, 0xe7, 0x94, 0x12, 0x45, 0x75, 0xef, 0x80,
                0x70, 0x57, 0xc7, 0x7a, 0xda, 0x21, 0x38, 0xae, 0x4f, 0xa8, 0xd6, 0xc4, 0xde, 0x03,
                0x98, 0xa1, 0x4f, 0x3f,
            ],
            1,
        );
        let signature_script = vec![1, 2, 3];
        let sequence = DEFAULT_SEQUENCE;
        let input = TransactionInput {
            previous_output,
            signature_script,
            sequence,
        };
        assert_eq!(input, input_deserialized);
    }
}
