use super::error_log::ErrorLog;
use super::level::Level;
use super::logger::MessageLog;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
    sync::mpsc::Receiver,
};
/// LoggerReceiver manages the log messages that have to be sent to register the operations
///
/// ### Errores
///  * `Error::ErrorFileNotFound`: Este error va a aparecer cuando el archivo pasado no se exista
///  * `Error::ErrorCouldNotWriteInFile`: Este error va a aparece cuando no se puede agregar más lineas al archivo dado
#[derive(Debug)]
pub struct LoggerReceiver {
    receiver: Receiver<MessageLog>,
    file: File,
}

impl LoggerReceiver {
    /// Receiver creation from a file path and a channel receiver
    ///
    /// ### Errores
    ///  * `Error::ErrorFileNotFound`: Este error va a aparecer cuando el archivo pasado no se exista
    pub(crate) fn new(
        logger_file: &Path,
        receiver: Receiver<MessageLog>,
    ) -> Result<Self, ErrorLog> {
        let resulting_file = OpenOptions::new().append(true).open(logger_file);

        let file = match resulting_file {
            Ok(file) => file,
            _ => {
                return Err(ErrorLog::ErrorFileNotFound);
            }
        };

        Ok(LoggerReceiver { receiver, file })
    }

    /// Receive the messages sent by `LoggerSender`
    ///
    /// ### Errores
    ///  * `Error::ErrorCouldNotWriteInFile`: Este error va a aparece cuando no se puede agregar más lineas al archivo dado
    pub fn receive_log(self) -> Result<(), ErrorLog> {
        let mut file = self.file;

        for (level, message) in self.receiver {
            let text = Self::format_message(level, message);

            if file.write(text.as_bytes()).is_err() {
                return Err(ErrorLog::ErrorCouldNotWriteInFile);
            }

            //Simplemente para no abrir el logger constantemente
            print!("{text}");
        }

        Ok(())
    }

    /// Format in which the message will be written in the file
    /// Includes the time in which the message is received
    fn format_message(level: Level, message: String) -> String {
        //Esto deberia ir en el main
        let format_timestamp = |time: SystemTime| {
            let since_epoch = time.duration_since(UNIX_EPOCH).unwrap();
            format!("{:?}", since_epoch)
        };

        let time = format_timestamp(SystemTime::now());

        format!("Time: [{time}]: [{level}] {message}\n")
    }
}
