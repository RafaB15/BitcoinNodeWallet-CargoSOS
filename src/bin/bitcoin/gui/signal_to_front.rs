use cargosos_bitcoin::{
    block_structure::block_chain::BlockChain,
};

pub enum SignalToFront {
    RegisterWallet(String),
    LoadAvailableBalance(u64),
    //LoadRecentTransactions(Vec<String>),
    LoadBlockChain(BlockChain),
}