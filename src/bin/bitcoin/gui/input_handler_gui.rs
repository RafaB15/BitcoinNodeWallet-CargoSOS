use super::{frontend, signal_to_back::SignalToBack};

use crate::{
    process::{
        reference::{get_reference, MutArc},
        transaction,
    },
    ui::{account, error_ui::ErrorUI, input_handler::InputHandler},
};

use cargosos_bitcoin::{
    block_structure::{block_chain::BlockChain, utxo_set::UTXOSet},
    logs::logger_sender::LoggerSender,
    node_structure::broadcasting::Broadcasting,
    notifications::{notification::Notification, notifier::Notifier},
    wallet_structure::{address::Address, wallet::Wallet},
};

use std::{
    io::{Read, Write},
    sync::mpsc::Receiver,
};

pub struct InputHandlerGUI<N>
where
    N: Notifier,
{
    rx_from_front: Receiver<SignalToBack>,
    notifier: N,
    logger: LoggerSender,
}

impl<N: Notifier> InputHandlerGUI<N> {
    pub fn new(rx_from_front: Receiver<SignalToBack>, notifier: N, logger: LoggerSender) -> Self {
        Self {
            rx_from_front,
            notifier,
            logger,
        }
    }
}

impl<RW, N> InputHandler<RW> for InputHandlerGUI<N>
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
        for rx in &self.rx_from_front {
            let mut wallet_reference = get_reference(&wallet)?;
            let mut utxo_set_reference = get_reference(&utxo_set)?;
            let mut broadcasting_reference = get_reference(&broadcasting)?;
            let block_chain_reference = get_reference(&block_chain)?;

            match rx {
                SignalToBack::GetAccountBalance => {
                    account::give_account_balance(
                        &wallet_reference,
                        &utxo_set_reference,
                        self.notifier.clone(),
                    );
                }
                SignalToBack::ChangeSelectedAccount(account_name) => {
                    account::change_selected_account(
                        account_name,
                        &mut wallet_reference,
                        self.notifier.clone(),
                    )?;
                }
                SignalToBack::CreateTransaction(address_string, amount, fee) => {
                    let address = match Address::new(&address_string) {
                        Ok(address) => address,
                        Err(_) => {
                            self.notifier.notify(Notification::InvalidAddressEnter);
                            return Ok(());
                        }
                    };

                    transaction::sending_transaction(
                        &mut broadcasting_reference,
                        &wallet_reference,
                        &mut utxo_set_reference,
                        address,
                        (amount, fee),
                        self.notifier.clone(),
                        self.logger.clone(),
                    )?;
                }
                SignalToBack::CreateAccount(name, private_key, public_key) => {
                    frontend::create_account(
                        wallet.clone(),
                        &name,
                        &private_key,
                        &public_key,
                        self.notifier.clone(),
                    )?;
                }
                SignalToBack::GetAccountTransactions => {
                    account::give_account_transactions(
                        &wallet_reference,
                        &block_chain_reference,
                        self.notifier.clone(),
                        self.logger.clone(),
                    )?;
                }
                SignalToBack::ExitProgram => {
                    break;
                }
            }
        }
        Ok(())
    }
}
