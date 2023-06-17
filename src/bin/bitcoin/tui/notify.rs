use crate::process::message_notify::MessageNotify;

use cargosos_bitcoin::logs::logger_sender::LoggerSender;

use std::{
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
};

pub fn notification(
    receiver_notify: Receiver<MessageNotify>,
    logger: LoggerSender,
) -> JoinHandle<()> {
    thread::spawn(move || {
        for notification in receiver_notify {
            match notification {
                MessageNotify::Balance(balance) => {
                    let _ = logger.log_node(format!("New balance: {:?}", balance));
                }
                MessageNotify::Transaction(transaction) => {
                    let _ = logger.log_node(format!("New transaction: {:?}", transaction));
                }
                MessageNotify::TransactionInBlock((transaction, block)) => {
                    let _ = logger.log_node(format!(
                        "The transaction: {:?} is in the block with header: {:?}",
                        transaction, block.header
                    ));
                },
            }
        }
    })
}
