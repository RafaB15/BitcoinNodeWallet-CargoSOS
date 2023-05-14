use std::time::{SystemTime, UNIX_EPOCH};
use super::error_log::ErrorLog;
use super::logger::MessageLog;
use super::level::Level;
use std::{
    io::Write,
    sync::mpsc::Receiver,
};
    /// LoggerReceiver manages the log messages that have to be sent to register the operations
    /// 
    /// ### Errores
    ///  * `Error::ErrorCouldNotWriteInFile`: Este error va a aparece cuando no se puede agregar más lineas al archivo dado
#[derive(Debug)]
pub struct LoggerReceiver<W: Write> {
    receiver: Receiver<MessageLog>,
    output: W,
    display_in_terminal: bool,

}

impl<W: Write> LoggerReceiver<W> {
    /// Receiver creation from a file path and a channel receiver
    pub(crate) fn new(output: W, receiver: Receiver<MessageLog>, display_in_terminal: bool) -> Result<Self, ErrorLog> {

        Ok(LoggerReceiver {receiver, output, display_in_terminal })

    }

    /// Receive the messages sent by `LoggerSender`
    /// 
    /// ### Errores
    ///  * `Error::ErrorCouldNotWriteInFile`: Este error va a aparece cuando no se puede agregar más lineas al archivo dado
    pub fn receive_log(self) -> Result<(), ErrorLog> {
        let mut file = self.output;

        for (level, message) in self.receiver {
            let text = Self::format_message(level, message);

            if file.write(text.as_bytes()).is_err() {
                return Err(ErrorLog::ErrorCouldNotWriteInFile);
            }

            //Simplemente para no abrir el logger constantemente
            if self.display_in_terminal {
                print!("{}", text);
            }

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

