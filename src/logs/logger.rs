use super::error_log::ErrorLog;
use super::level::Level;
use super::logger_receiver::LoggerReceiver;
use super::logger_sender::LoggerSender;
use std::io::Write;
use std::sync::mpsc;

pub(crate) type MessageLog = (Level, String);
    /// We create the sender and the receiver for the logger, receiving the path of the file where we want to write the logs
    /// 
    /// Errores:
    /// # CouldNotWriteInFile No se pudo escribir en el file
    /// # ErrorCouldNotFindReceiver No se encontro el receiver
pub fn initialize_logger<W: Write>(output: W, display_in_terminal: bool) -> Result<(LoggerSender, LoggerReceiver<W>), ErrorLog> {
    let (sender, receiver) = mpsc::channel::<MessageLog>();

    let logger_sender = LoggerSender::new(sender);
    let logger_receiver = LoggerReceiver::new(output, receiver, display_in_terminal)?;

    Ok((logger_sender, logger_receiver))
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fs::File;
    use std::path::Path;
    
    #[test]
    fn correct_log_creation() {
    
        let mut vec: Vec<u8> = Vec::new();
        let (logger_sender, logger_receiver) = initialize_logger(&mut vec, false).unwrap(); // Pass vec as a mutable reference

        logger_sender.log(Level::NODE, "A block".to_string()).unwrap();
        logger_sender.log(Level::NODE, "Another block".to_string()).unwrap();
        std::mem::drop(logger_sender);

        logger_receiver.receive_log().unwrap();

        let contents = String::from_utf8(vec).unwrap();

        assert!(contents.contains("[NODE] A block"));
        assert!(contents.contains("[NODE] Another block"));
    }

    #[test]
    fn error_receiver_not_found() {
        let log_file_path = Path::new("tests/common/logs/test_log.txt");
        let log_file = File::create(log_file_path).expect("failed to create log file");
        let (logger_sender, logger_receiver) = initialize_logger(log_file, false).unwrap();

        std::mem::drop(logger_receiver);

        let error_message: ErrorLog = logger_sender
            .log(Level::NODE, "A block".to_string())
            .unwrap_err();

        assert_eq!(error_message, ErrorLog::ReceiverNotFound);
    }
}
