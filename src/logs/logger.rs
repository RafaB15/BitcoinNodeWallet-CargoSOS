use super::level::Level;
use super::logger_receiver::LoggerReceiver;
use super::logger_sender::LoggerSender;
use std::io::Write;
use std::sync::mpsc;

pub(crate) type MessageLog = (Level, String);
/// We create the sender and the receiver for the logger, receiving the path of the file where we want to write the logs
pub fn initialize_logger<W: Write>(
    output: W,
    display_in_terminal: bool,
) -> (LoggerSender, LoggerReceiver<W>) {
    let (sender, receiver) = mpsc::channel::<MessageLog>();

    let logger_sender = LoggerSender::new(sender);
    let logger_receiver = LoggerReceiver::new(output, receiver, display_in_terminal);

    (logger_sender, logger_receiver)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::logs::error_log::ErrorLog;

    #[test]
    fn test01_correct_log_creation() {
        let mut vec: Vec<u8> = Vec::new();
        let (logger_sender, logger_receiver) = initialize_logger(&mut vec, false); // Pass vec as a mutable reference

        logger_sender
            .log(Level::NODE, "A block".to_string())
            .unwrap();
        logger_sender
            .log(Level::NODE, "Another block".to_string())
            .unwrap();
        std::mem::drop(logger_sender);

        logger_receiver.receive_log().unwrap();

        let contents = String::from_utf8(vec).unwrap();

        assert!(contents.contains("[NODE] A block"));
        assert!(contents.contains("[NODE] Another block"));
    }

    #[test]
    fn test02_error_receiver_not_found() {
        let log_file: Vec<u8> = Vec::new();
        let (logger_sender, logger_receiver) = initialize_logger(log_file, false);

        std::mem::drop(logger_receiver);

        let error_message: ErrorLog = logger_sender
            .log(Level::NODE, "A block".to_string())
            .unwrap_err();

        assert_eq!(error_message, ErrorLog::ReceiverNotFound);
    }
}
