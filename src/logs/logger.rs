use super::error_log::{ErrorLog};
use super::logger_receiver::LoggerReceiver;
use super::logger_sender::LoggerSender;
use super::level::Level;
use std::io::Write;
use std::sync::mpsc;

pub(crate) type MessageLog = (Level, String);
    /// We create the sender and the receiver for the logger, receiving the path of the file where we want to write the logs
    /// 
    /// Errores:
    /// # ErrorFileNotFound No se encontro el file
    /// # ErrorCouldNotWriteInFile No se pudo escribir en el file
    /// # ErrorCouldNotFindReceiver No se encontro el receiver
pub fn initialize_logger<W: Write>(output: W) -> Result<(LoggerSender, LoggerReceiver<W>), ErrorLog> {
    let (sender, receiver) = mpsc::channel::<MessageLog>();

    let logger_sender = LoggerSender::new(sender);
    let logger_receiver = LoggerReceiver::new(output, receiver)?;

    Ok((logger_sender, logger_receiver))
}

#[cfg(test)]
mod tests {

    use std::fs::File;
    use std::io::Read;
    use std::path::Path;
    use super::*;
    
    #[test]
    fn correct_log_creation() {
    
        let log_file_path = Path::new("tests/common/logs/test_log.txt");
        let log_file = File::create(log_file_path).expect("failed to create log file");
        let (logger_sender, _logger_receiver) = initialize_logger(log_file).unwrap();

        logger_sender.log(Level::NODE,"A block".to_string()).unwrap();
        logger_sender.log(Level::NODE, "Another block".to_string()).unwrap();

        //Wait for the logs to be written in the file
        std::thread::sleep(std::time::Duration::from_secs(5));

        let mut file = File::open(&log_file_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("[NODE] A block"));
        assert!(contents.contains("[NODE] Another block"));

    }

    #[test]
    fn error_receiver_not_found() {
        let log_file_path = Path::new("tests/common/logs/test_log.txt");
        let log_file = File::create(log_file_path).expect("failed to create log file");
        let (logger_sender, logger_receiver) = initialize_logger(log_file).unwrap();

        std::mem::drop(logger_receiver);

        let error_message: ErrorLog = logger_sender.log(Level::NODE,"A block".to_string()).unwrap_err();

        assert_eq!(error_message, ErrorLog::ErrorReceiverNotFound);

    }
}
