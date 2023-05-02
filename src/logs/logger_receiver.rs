use std::time::{SystemTime, UNIX_EPOCH};
use super::logger::MessageLog;

pub struct LoggerReceiver {
    receiver: Receiver<MessageLog>
    file: File
}

impl LoggerReceiver {
    ///Error posible: no se encontro el file que se paso
    pub(crate) fn new(logger_file: &Path, receiver: <MessageLog>) -> Result<Self, ErrorLog> {
        let resulting_file = OpenOptions::new().append(true).open(logger_file);

        let file = match resulting_file {
            Ok(file) => file,
            _ => { 
                return Err(ErrorLog::ErrorFileNotFound);
            }
        };

        Ok(LoggerReceiver {receiver, file })

    }

    pub fn receive_log(self) -> Result<(), ErrorLog> {
        let mut file = self.file;

        for (level, message) in self.receiver {
            let text = Self::format_message(level, message);

            if file.write(text.as_bytes()).is_err() {
                return Err(ErrorLog::ErrorCouldNotWriteInFile);
            }

            ///Simplemente para no abrir el logger constantemente
            print!("{text}");

        }

        Ok(())

    }

    /// Formato en el que se escribira el mensaje en el archivo.
    /// Incluye el tiempo en el que se recibe el mensaje
    fn format_message(level: Level, message: String) -> {
        let time = format_timestamp(SystemTime::now());

        format!("[{time}]: {level} {message}")
    }
}

