use super::{
    error_block::ErrorBlock,
    hash::{hash256d, HashType},
    outpoint::Outpoint,
    transaction_input::TransactionInput,
    transaction_output::TransactionOutput,
};

use crate::{
    messages::compact_size::CompactSize,
    serialization::{
        deserializable_internal_order::DeserializableInternalOrder,
        deserializable_little_endian::DeserializableLittleEndian,
        error_serialization::ErrorSerialization,
        serializable_internal_order::SerializableInternalOrder,
        serializable_little_endian::SerializableLittleEndian,
    },
    wallet_structure::{account::Account, address::Address, error_wallet::ErrorWallet},
};

use chrono::offset::Utc;

use std::{
    cmp::PartialEq,
    collections::HashMap,
    fmt::Display,
    io::{Read, Write},
};

/// It's the representation of a transaction in the block chain
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub version: i32,
    pub tx_in: Vec<TransactionInput>,
    pub tx_out: Vec<TransactionOutput>,
    pub time: u32,
}

impl Transaction {
    /// It create the id for this transaction
    ///
    /// ### Error
    ///  * `ErrorBlock::CouldNotGetTxId`: It will appear when the transaction id could not be created
    pub fn get_tx_id(&self) -> Result<HashType, ErrorBlock> {
        let mut buffer = vec![];
        if self.io_serialize(&mut buffer).is_err() {
            return Err(ErrorBlock::CouldNotGetTxId);
        }
        match hash256d(&buffer) {
            Ok(txid) => Ok(txid),
            Err(_) => return Err(ErrorBlock::CouldNotGetTxId),
        }
    }

    /// It create the id for all the transaction
    ///
    /// ### Error
    ///  * `ErrorBlock::CouldNotGetTxId`: It will appear when the transaction id could not be created
    pub fn get_vec_txids(transactions: &[Transaction]) -> Result<Vec<HashType>, ErrorBlock> {
        let mut tx_ids: Vec<HashType> = Vec::new();
        for tx in transactions.iter() {
            tx_ids.push(tx.get_tx_id()?)
        }
        Ok(tx_ids)
    }

    /// Returns true if the address owns any of transaction output (works for P2PKH) and false otherwise
    pub fn verify_transaction_ownership(&self, address: &Address) -> bool {
        self.tx_out
            .iter()
            .any(|tx_out| address.verify_transaction_ownership(tx_out))
    }

    /// Returns a transaction given the amount and to whom it is sent
    ///
    /// ### Error
    ///  * `ErrorWallet::CannotCreateNewTransaction`: It will appear when a transaction cannot be created
    ///  * `ErrorWallet::NotEnoughFunds`: It will appear when an account does not have enough funds to create a transaction for the amount requested
    pub fn from_account_to_address(
        account_from: &Account,
        outputs_to_spend: &HashMap<Outpoint, TransactionOutput>,
        account_to: &Address,
        amount: i64,
        fee: i64,
    ) -> Result<Transaction, ErrorWallet> {
        let mut tx_in: Vec<TransactionInput> = Vec::new();
        for outpoint in outputs_to_spend.keys() {
            let new_transaction_input = TransactionInput::from_outpoint_unsigned(outpoint);
            tx_in.push(new_transaction_input);
        }

        let mut total_amount = 0;
        outputs_to_spend
            .iter()
            .for_each(|(_, output)| total_amount += output.value);

        let change = total_amount - amount - fee;

        let mut tx_out: Vec<TransactionOutput> = Vec::new();
        let transaction_output_to_address =
            TransactionOutput::new(amount, account_to.generate_script_pubkey_p2pkh());
        let transaction_output_change =
            TransactionOutput::new(change, account_from.address.generate_script_pubkey_p2pkh());

        tx_out.push(transaction_output_to_address);
        tx_out.push(transaction_output_change);

        let time: u32 = Utc::now().timestamp() as u32;

        let mut unsigned_transaction = Transaction {
            version: 1,
            tx_in,
            tx_out,
            time,
        };

        unsigned_transaction.get_signed_by_account(account_from)?;

        Ok(unsigned_transaction)
    }

    /// Sign the transaction with the given account
    ///
    /// ### Error
    ///  * `ErrorWallet::CannotCreateNewTransaction`: It will appear when a transaction cannot be created
    pub fn get_signed_by_account(&mut self, account: &Account) -> Result<(), ErrorWallet> {
        let unsigned_transaction = self.clone();

        for (index, tx_in) in self.tx_in.iter_mut().enumerate() {
            let script_sig = TransactionInput::create_signature_script(
                account,
                unsigned_transaction.clone(),
                index,
            )?;
            tx_in.signature_script = script_sig;
        }
        Ok(())
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let transaction_id = match self.get_tx_id() {
            Ok(transaction_id) => transaction_id,
            Err(_) => return write!(f, "Transaction fail at get tx id"),
        };

        let mut transaction_id_string = "".to_string();
        for byte in transaction_id.iter().rev() {
            transaction_id_string.push_str(&format!("{:02x}", byte));
        }

        write!(f, "{transaction_id_string}")
    }
}

impl SerializableInternalOrder for Transaction {
    fn io_serialize(&self, stream: &mut dyn Write) -> Result<(), ErrorSerialization> {
        self.version.le_serialize(stream)?;

        CompactSize::new(self.tx_in.len() as u64).le_serialize(stream)?;
        for tx_in in self.tx_in.iter() {
            tx_in.io_serialize(stream)?;
        }

        CompactSize::new(self.tx_out.len() as u64).le_serialize(stream)?;
        for tx_out in &self.tx_out {
            tx_out.io_serialize(stream)?;
        }

        self.time.le_serialize(stream)?;
        Ok(())
    }
}

impl DeserializableInternalOrder for Transaction {
    fn io_deserialize(stream: &mut dyn Read) -> Result<Self, ErrorSerialization> {
        let version = i32::le_deserialize(stream)?;

        let length_tx_in = CompactSize::le_deserialize(stream)?.value;
        let mut tx_in: Vec<TransactionInput> = Vec::new();
        for _ in 0..length_tx_in {
            tx_in.push(TransactionInput::io_deserialize(stream)?);
        }

        let length_tx_out = CompactSize::le_deserialize(stream)?.value;
        let mut tx_out: Vec<TransactionOutput> = Vec::new();
        for _ in 0..length_tx_out {
            tx_out.push(TransactionOutput::io_deserialize(stream)?);
        }

        let time = u32::le_deserialize(stream)?;

        Ok(Transaction {
            version,
            tx_in,
            tx_out,
            time,
        })
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_01_correct_transaction_serialization() {
        let transaction_input =
            TransactionInput::new(Outpoint::new([1; 32], 23), vec![1, 2, 3], 24);

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![4, 5, 6],
        };

        let transaction = Transaction {
            version: 1,
            tx_in: vec![transaction_input.clone()],
            tx_out: vec![transaction_output.clone()],
            time: 0,
        };

        let mut buffer: Vec<u8> = Vec::new();
        transaction.io_serialize(&mut buffer).unwrap();

        let mut expected_buffer: Vec<u8> = Vec::new();
        let version: i32 = 1;
        version.le_serialize(&mut expected_buffer).unwrap();
        CompactSize::new(1 as u64)
            .le_serialize(&mut expected_buffer)
            .unwrap();
        transaction_input
            .io_serialize(&mut expected_buffer)
            .unwrap();
        CompactSize::new(1 as u64)
            .le_serialize(&mut expected_buffer)
            .unwrap();
        transaction_output
            .io_serialize(&mut expected_buffer)
            .unwrap();
        let time: u32 = 0;
        time.le_serialize(&mut expected_buffer).unwrap();

        assert_eq!(buffer, expected_buffer);
    }

    #[test]
    fn test_02_correct_transaction_deserialization() {
        let transaction_input =
            TransactionInput::new(Outpoint::new([1; 32], 23), vec![1, 2, 3], 24);

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![4, 5, 6],
        };

        let transaction = Transaction {
            version: 1,
            tx_in: vec![transaction_input.clone()],
            tx_out: vec![transaction_output.clone()],
            time: 0,
        };

        let mut buffer: Vec<u8> = Vec::new();
        transaction.io_serialize(&mut buffer).unwrap();

        let deserialized_transaction = Transaction::io_deserialize(&mut buffer.as_slice()).unwrap();

        assert_eq!(transaction, deserialized_transaction);
    }

    #[test]
    fn test_03_correct_tx_id() {
        let transaction_input =
            TransactionInput::new(Outpoint::new([1; 32], 23), vec![1, 2, 3], 24);

        let transaction_output = TransactionOutput {
            value: 10,
            pk_script: vec![4, 5, 6],
        };

        let transaction = Transaction {
            version: 1,
            tx_in: vec![transaction_input.clone()],
            tx_out: vec![transaction_output.clone()],
            time: 0,
        };

        let tx_id = transaction.get_tx_id().unwrap();
        let actual_tx_id: [u8; 32] = [
            56, 175, 42, 225, 30, 123, 58, 87, 39, 77, 48, 171, 101, 224, 173, 30, 97, 13, 161,
            104, 231, 123, 182, 62, 124, 226, 161, 25, 164, 106, 65, 45,
        ];

        assert_eq!(tx_id, actual_tx_id);
    }

    #[test]
    fn test_04_correct_verification_of_transaction_ownership() {
        let account_old = Address::new("mnQLoVaZ3w1NLVmUhfG8hh6WoG3iu7cnNw").unwrap();
        let account_new = Address::new("mrhW6tcF2LDetj3kJvaDTvatrVxNK64NXk").unwrap();
        let transaction_bytes: Vec<u8> = vec![
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
        let transaction = Transaction::io_deserialize(&mut transaction_bytes.as_slice()).unwrap();

        assert!(transaction.verify_transaction_ownership(&account_old));
        assert!(transaction.verify_transaction_ownership(&account_new));
    }

    #[test]
    fn test_05_correct_transaction_signing() {
        let account_old = Account::new(
            "Old",
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

        let transaction_to_be_signed_bytes: Vec<u8> = vec![
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
        let mut transaction_to_be_signed =
            Transaction::io_deserialize(&mut transaction_to_be_signed_bytes.as_slice()).unwrap();

        transaction_to_be_signed
            .get_signed_by_account(&account_old)
            .unwrap();

        let signed_transaction_bytes: Vec<u8> = vec![
            0x01, 0x00, 0x00, 0x00, 0x01, 0x20, 0x25, 0xEF, 0x69, 0x2C, 0xA9, 0x87, 0xB3, 0x9A,
            0x81, 0x33, 0x6E, 0xFB, 0x59, 0xB0, 0x56, 0xFB, 0x90, 0xC0, 0x3A, 0x5E, 0xA4, 0xC4,
            0x54, 0x4C, 0xF9, 0x27, 0x57, 0x61, 0x3E, 0x2E, 0xA4, 0x01, 0x00, 0x00, 0x00, 0x6B,
            0x48, 0x30, 0x45, 0x02, 0x21, 0x00, 0xBB, 0xDB, 0xD2, 0x5E, 0x68, 0x06, 0xDC, 0x4F,
            0x82, 0x3F, 0xB9, 0x1B, 0x3C, 0xFB, 0xC2, 0xF6, 0xEC, 0xDE, 0x9D, 0x36, 0x67, 0x7C,
            0x59, 0x49, 0xC2, 0x1F, 0x01, 0xEE, 0xDF, 0x27, 0x09, 0x23, 0x02, 0x20, 0x19, 0x14,
            0x62, 0x04, 0x55, 0x94, 0xBD, 0xF6, 0x8C, 0x2A, 0x9E, 0x94, 0xA0, 0x60, 0xD3, 0xBD,
            0x80, 0xF8, 0x86, 0xCC, 0xC9, 0x43, 0xAD, 0x26, 0xBB, 0xDD, 0xC6, 0x05, 0x3A, 0xB5,
            0xF6, 0x35, 0x01, 0x21, 0x03, 0xBC, 0x6D, 0x45, 0xD2, 0x10, 0x1E, 0x91, 0x28, 0xDE,
            0x14, 0xB5, 0xB6, 0x68, 0x83, 0xD6, 0x9C, 0xF1, 0xC3, 0x1A, 0x50, 0xB9, 0x6F, 0xEA,
            0x2D, 0xAD, 0x4E, 0xD2, 0x35, 0x14, 0x92, 0x4A, 0x22, 0xFF, 0xFF, 0xFF, 0xFF, 0x02,
            0xA0, 0x86, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x76, 0xA9, 0x14, 0x7A, 0xA8,
            0x18, 0x46, 0x85, 0xCA, 0x1F, 0x06, 0xF5, 0x43, 0xB6, 0x4A, 0x50, 0x2E, 0xB3, 0xB6,
            0x13, 0x5D, 0x67, 0x20, 0x88, 0xAC, 0xD0, 0xE6, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x19, 0x76, 0xA9, 0x14, 0x4B, 0x88, 0xC1, 0xD3, 0x87, 0x49, 0x08, 0x36, 0x57, 0x73,
            0xA7, 0x65, 0xCD, 0xB0, 0x52, 0xC9, 0xEF, 0x5F, 0x1A, 0x80, 0x88, 0xAC, 0x98, 0xFB,
            0x95, 0x64,
        ];
        let signed_transaction =
            Transaction::io_deserialize(&mut signed_transaction_bytes.as_slice()).unwrap();

        assert_eq!(transaction_to_be_signed, signed_transaction);
    }
}
