use std::time::{SystemTime, UNIX_EPOCH};
use super::error_log::ErrorLog;
use super::logger::MessageLog;
use super::level::Level;
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
    sync::mpsc::Receiver,
};

    /// Se encarga de manejar los mensajes de logs que se tenga que enviar para registrar las operaciones
    /// 
    /// ### Errores
    ///  * `Error::ErrorFileNotFound`: Este error va a aparecer cuando el archivo pasado no se exista
    ///  * `Error::ErrorCouldNotWriteInFile`: Este error va a aparece cuando no se puede agregar más lineas al archivo dado
pub struct LoggerReceiver {
    receiver: Receiver<MessageLog>,
    file: File
}

impl LoggerReceiver {
    /// Crea el receiver a partir del path de un archivo y un receiver de channel
    /// 
    /// ### Errores
    ///  * `Error::ErrorFileNotFound`: Este error va a aparecer cuando el archivo pasado no se exista
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

    /// La acción de recibir los mensajes mandados por `LoggerSender`
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

            ///Simplemente para no abrir el logger constantemente
            print!("{text}");

        }

        Ok(())

    }

    /// Formato en el que se escribira el mensaje en el archivo.
    /// Incluye el tiempo en el que se recibe el mensaje
    fn format_message(level: Level, message: String) -> String {
        let time = format_timestamp(SystemTime::now());

        format!("[{time}]: {level} {message}")
    }
}

