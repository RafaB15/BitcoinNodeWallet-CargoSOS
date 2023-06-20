use super::logger::MessageLog;
use std::sync::mpsc::Sender;

use super::error_log::ErrorLog;
use super::level::Level;

/// Manages the log messages. This can be cloned to have multiple senders
///
/// ### Errores
///  * `Error::ReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
#[derive(Debug, Clone)]
pub struct LoggerSender {
    sender: Sender<MessageLog>,
}

impl LoggerSender {
    pub(crate) fn new(sender: Sender<MessageLog>) -> Self {
        LoggerSender { sender }
    }

    /// Sends the message with the desired level
    ///
    /// ### Errores
    ///  * `Error::ReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
    pub fn log(&self, level: Level, message: String) -> Result<(), ErrorLog> {
        if self.sender.send((level, message)).is_err() {
            return Err(ErrorLog::ReceiverNotFound);
        }
        Ok(())
    }

    /// Sends the desired message with level: `Level::NODE`
    ///
    /// ### Errores
    ///  * `Error::ReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
    pub fn log_node(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::NODE, mensaje)?;
        Ok(())
    }

    /// Sends the desired message with level: `Level::WALLET`
    ///
    /// ### Errores
    ///  * `Error::ReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
    pub fn log_wallet(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::WALLET, mensaje)?;
        Ok(())
    }

    /// Sends the desired message with level: `Level::TRANSACTION`
    ///
    /// ### Errores
    ///  * `Error::ReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
    pub fn log_transaction(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::TRANSACTION, mensaje)?;
        Ok(())
    }

    /// Sends the desired message with level: `Level::CONFIGURATION`
    ///
    /// ### Errores
    ///  * `Error::ReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
    pub fn log_configuration(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::CONFIGURATION, mensaje)?;
        Ok(())
    }

    /// Sends the desired message with level: `Level::CONNECTION`
    ///
    /// ### Errores
    ///  * `Error::ReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
    pub fn log_connection(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::CONNECTION, mensaje)?;
        Ok(())
    }

    /// Sends the desired message with level: `Level::FILE`
    ///
    /// ### Errores
    ///  * `Error::ReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
    pub fn log_file(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::FILE, mensaje)?;
        Ok(())
    }

    /// Sends the desired message with level: `Level::INTERFACE`
    ///
    /// ### Errores
    ///  * `Error::ReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
    pub fn log_interface(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::INTERFACE, mensaje)?;
        Ok(())
    }

    /// Sends the desired message with level: `Level::CONNECTION`
    ///
    /// ### Errores
    ///  * `Error::ReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
    pub fn log_error(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::ERROR, mensaje)?;
        Ok(())
    }
}
