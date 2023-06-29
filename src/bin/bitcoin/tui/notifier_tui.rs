use cargosos_bitcoin::{
    logs::logger_sender::LoggerSender,
    notifications::{notification::Notification, notifier::Notifier},
};

use std::cmp::max;

#[derive(Clone)]
pub struct NotifierTUI {
    logger: LoggerSender,
}

impl NotifierTUI {
    pub fn new(logger: LoggerSender) -> Self {
        Self { logger }
    }
}

impl Notifier for NotifierTUI {
    fn notify(&self, notification: Notification) {
        match notification {
            Notification::AttemptingHandshakeWithPeer(_) => todo!(),
            Notification::SuccessfulHandshakeWithPeer(_) => todo!(),
            Notification::FailedHandshakeWithPeer(_) => todo!(),
            Notification::TransactionOfAccountReceived(_, _) => todo!(),
            Notification::TransactionOfAccountInNewBlock(_) => todo!(),
            Notification::NewBlockAddedToTheBlockchain(_) => todo!(),
            Notification::UpdatedSelectedAccount(_) => todo!(),
            Notification::RegisterWalletAccount(_) => todo!(),
            Notification::NotifyBlockchainIsReady => todo!(),
            Notification::LoadAvailableBalance(_) => todo!(),
            Notification::AccountNotSelected => todo!(),
            Notification::AccountTransactions(_) => todo!(),
            Notification::InvalidAddressEnter => todo!(),
            Notification::InvalidPublicKeyEnter => todo!(),
            Notification::InvalidPrivateKeyEnter => todo!(),
            Notification::AccountCreationFail => todo!(),
            Notification::NotEnoughFunds => todo!(),
        }
    }
}

/// Notify the user in a clean way
fn notify(title: &str, body: &str, logger: LoggerSender) {
    let len_message = max(calculate_body_len(title), calculate_body_len(body));
    let border = "#".repeat(len_message + 4);

    let mut message = format!("{border}\n");
    for title_line in title.split('\n') {
        let spaces = len_message - title_line.len();
        message.push_str(&format!("# {}{} #\n", title_line, " ".repeat(spaces)));
    }

    for body_line in body.split('\n') {
        let spaces = len_message - body_line.len();
        message.push_str(&format!("# {}{} #\n", body_line, " ".repeat(spaces)));
    }

    message.push_str(border.as_str());

    println!("{message}");
    let _ = logger.log_notification(body.to_string());
}

/// Given a body of text, returns the length of the longest line
fn calculate_body_len(body: &str) -> usize {
    let mut len = 0;
    for line in body.split('\n') {
        len = max(len, line.len());
    }
    len
}
