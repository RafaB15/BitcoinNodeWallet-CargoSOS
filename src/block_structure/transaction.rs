use super::{
    error_block::ErrorBlock,
    hash::{hash256d, HashType},
    outpoint::Outpoint,
    transaction_input::TransactionInput,
    transaction_output::TransactionOutput,
};

use crate::{
    serialization::{
        deserializable_internal_order::DeserializableInternalOrder,
        deserializable_little_endian::DeserializableLittleEndian,
        error_serialization::ErrorSerialization,
        serializable_internal_order::SerializableInternalOrder,
        serializable_little_endian::SerializableLittleEndian,
    },
    wallet_structure::address::Address,
};

use crate::wallet_structure::{account::Account, address::Address, error_wallet::ErrorWallet};

use chrono::offset::Utc;

use crate::messages::compact_size::CompactSize;

use std::{
    cmp::PartialEq,
    collections::HashMap,
    fmt::Display,
    io::{Read, Write},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub version: i32,
    pub tx_in: Vec<TransactionInput>,
    pub tx_out: Vec<TransactionOutput>,
    pub time: u32,
}

impl Transaction {
    pub fn get_tx_id(&self) -> Result<HashType, ErrorBlock> {
        let mut buffer = vec![];
        if self.io_serialize(&mut buffer).is_err() {
            return Err(ErrorBlock::CouldNotGetTxId);
        }

        let buffer = {
            let mut temp: Vec<u8> = Vec::new();

            // Hash the buffer to get the transaction ID
            let txid = match hash256d(&buffer) {
                Ok(txid) => txid,
                Err(_) => return Err(ErrorBlock::CouldNotGetTxId),
            };

            for byte in txid.iter().rev() {
                temp.push(*byte);
            }

            temp
        };

        let buffer: HashType = match (*buffer.as_slice()).try_into() {
            Ok(buffer) => buffer,
            _ => return Err(ErrorBlock::CouldNotGetTxId),
        };

        Ok(buffer)
    }

    pub fn get_vec_txids(transactions: &[Transaction]) -> Result<Vec<HashType>, ErrorBlock> {
        let mut tx_ids: Vec<HashType> = Vec::new();
        for tx in transactions.iter() {
            match tx.get_tx_id() {
                Ok(txid) => tx_ids.push(txid),
                Err(_) => return Err(ErrorBlock::CouldNotGetTxId),
            };
        }
        Ok(tx_ids)
    }

    pub fn verify_transaction_ownership(&self, address: &Address) -> bool {
        self.tx_out
            .iter()
            .any(|tx_out| address.verify_transaction_ownership(tx_out))
    }

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

        if let Err(error) = unsigned_transaction.get_signed_by_account(account_from) {
            return Err(error);
        }
        Ok(unsigned_transaction)
    }

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
        write!(f, "Transaction: to do")
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
