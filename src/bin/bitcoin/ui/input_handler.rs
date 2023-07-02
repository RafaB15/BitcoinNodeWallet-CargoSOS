use super::error_ui::ErrorUI;

use crate::process::reference::MutArc;

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, utxo_set::UTXOSet},
    node_structure::broadcasting::Broadcasting,
    wallet_structure::wallet::Wallet,
};

use std::io::{Read, Write};

pub trait InputHandler<RW>
where
    RW: Read + Write + Send + 'static,
{
    fn handle_input(
        &self,
        broadcasting: MutArc<Broadcasting<RW>>,
        wallet: MutArc<Wallet>,
        utxo_set: MutArc<UTXOSet>,
        block_chain: MutArc<BlockChain>,
    ) -> Result<(), ErrorUI>;
}
