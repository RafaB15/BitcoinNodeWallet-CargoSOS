use super::{frontend, menu, menu_option::MenuOption};

use crate::{
    process::reference::{get_reference, MutArc},
    ui::{account, error_ui::ErrorUI, input_handler::InputHandler},
};

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, utxo_set::UTXOSet},
    logs::logger_sender::LoggerSender,
    node_structure::broadcasting::Broadcasting,
    notifications::notifier::Notifier,
    wallet_structure::wallet::Wallet,
};

use std::io::{Read, Write};

pub struct InputHandlerTUI<N>
where
    N: Notifier,
{
    notifier: N,
    logger: LoggerSender,
}

impl<N: Notifier> InputHandlerTUI<N> {
    pub fn new(notifier: N, logger: LoggerSender) -> Self {
        Self { notifier, logger }
    }
}

impl<RW, N> InputHandler<RW> for InputHandlerTUI<N>
where
    RW: Read + Write + Send + 'static,
    N: Notifier,
{
    fn handle_input(
        &self,
        broadcasting: MutArc<Broadcasting<RW>>,
        wallet: MutArc<Wallet>,
        utxo_set: MutArc<UTXOSet>,
        block_chain: MutArc<BlockChain>,
    ) -> Result<(), ErrorUI> {
        loop {
            match menu::select_option(self.logger.clone())? {
                MenuOption::CreateAccount => {
                    let mut wallet_reference = get_reference(&wallet)?;
                    frontend::create_account(
                        &mut wallet_reference,
                        self.notifier.clone(),
                        self.logger.clone(),
                    )?
                }
                MenuOption::ChangeAccount => {
                    let mut wallet_reference = get_reference(&wallet)?;
                    frontend::change_account(
                        &mut wallet_reference,
                        self.notifier.clone(),
                        self.logger.clone(),
                    )?
                }
                MenuOption::RemoveAccount => {
                    let mut wallet_reference = get_reference(&wallet)?;
                    frontend::remove_account(&mut wallet_reference, self.logger.clone())?
                }
                MenuOption::SendTransaction => {
                    let wallet_reference = get_reference(&wallet)?;
                    let mut utxo_set_reference = get_reference(&utxo_set)?;
                    let mut broadcasting_reference = get_reference(&broadcasting)?;
                    frontend::sending_transaction(
                        &mut broadcasting_reference,
                        &wallet_reference,
                        &mut utxo_set_reference,
                        self.notifier.clone(),
                        self.logger.clone(),
                    )?
                }
                MenuOption::ShowAccounts => {
                    let wallet_reference = get_reference(&wallet)?;
                    frontend::show_accounts(&wallet_reference, self.logger.clone());
                }
                MenuOption::ShowBalance => {
                    let wallet_reference = get_reference(&wallet)?;
                    let utxo_set_reference = get_reference(&utxo_set)?;
                    account::give_account_balance(
                        &wallet_reference,
                        &utxo_set_reference,
                        self.notifier.clone(),
                    )
                }
                MenuOption::LastTransactions => {
                    let wallet_reference = get_reference(&wallet)?;
                    let blockchain_reference = get_reference(&block_chain)?;
                    account::give_account_transactions(
                        &wallet_reference,
                        &blockchain_reference,
                        self.notifier.clone(),
                        self.logger.clone(),
                    )?
                }
                MenuOption::MerkleProof => {
                    let blockchain_reference = get_reference(&block_chain)?;
                    frontend::create_merkle_proof_of_inclusion(
                        &blockchain_reference, 
                        self.notifier.clone(),
                        self.logger.clone()
                    )?
                },
                MenuOption::Exit => break,
            }
        }

        Ok(())
    }
}
