use cargosos_bitcoin::{
    block_structure::block_chain::BlockChain,
};

pub enum SignalToFront {
    RegisterWallet(String),
    LoadAvailableBalance(f64),
    //LoadRecentTransactions(Vec<String>),
    LoadBlockChain,
}