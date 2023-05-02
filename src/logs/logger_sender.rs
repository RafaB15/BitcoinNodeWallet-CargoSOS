use super::logger::MessageLog;
use std::sync::mpsc::Sender;

use super::error_log;ErrorLog;
use super::level::Level;


#[derive(Debug, Clone)]
pub struct LoggerSender {
    sender: <MessageLog>,
}

impl LoggerSender {
    pub(crate) fn new(sender: Sender<MessageLog>) -> Self {
        if self.sender.send(level, message).is_err() {
            return Err(ErrorLog::ErrorReceiverNotFound);
        }
        Ok(())
    }

}