use super::logger::MessageLog;
use std::fmt::Debug;
use std::sync::mpsc::Sender;

use super::error_log::ErrorLog;
use super::level::Level;

/// Manages the log messages. This can be cloned to have multiple senders
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
    ///  * `ErrorLog::ReceiverNotFound`: It will appear when the receiver it's drop and can't send the message
    pub fn log(&self, level: Level, message: String) -> Result<(), ErrorLog> {
        if self.sender.send((level, message)).is_err() {
            return Err(ErrorLog::ReceiverNotFound);
        }
        Ok(())
    }

    pub fn log_data<D: Debug>(&self, level: Level, data: D) -> Result<(), ErrorLog> {
        self.log(level, format!("{:?}", data))
    }

    /// Sends the desired message with level: `Level::NODE`
    ///
    /// ### Errores
    ///  * `ErrorLog::ReceiverNotFound`: It will appear when the receiver it's drop and can't send the message
    pub fn log_node(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::NODE, mensaje)?;
        Ok(())
    }

    /// Sends the desired message with level: `Level::WALLET`
    ///
    /// ### Errores
    ///  * `ErrorLog::ReceiverNotFound`: It will appear when the receiver it's drop and can't send the message
    pub fn log_wallet(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::WALLET, mensaje)?;
        Ok(())
    }

    /// Sends the desired message with level: `Level::TRANSACTION`
    ///
    /// ### Errores
    ///  * `ErrorLog::ReceiverNotFound`: It will appear when the receiver it's drop and can't send the message
    pub fn log_transaction(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::TRANSACTION, mensaje)?;
        Ok(())
    }

    /// Sends the desired message with level: `Level::CONFIGURATION`
    ///
    /// ### Errores
    ///  * `ErrorLog::ReceiverNotFound`: It will appear when the receiver it's drop and can't send the message
    pub fn log_configuration(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::CONFIGURATION, mensaje)?;
        Ok(())
    }

    /// Sends the desired message with level: `Level::CONNECTION`
    ///
    /// ### Errores
    ///  * `ErrorLog::ReceiverNotFound`: It will appear when the receiver it's drop and can't send the message
    pub fn log_connection(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::CONNECTION, mensaje)?;
        Ok(())
    }

    /// Sends the desired message with level: `Level::FILE`
    ///
    /// ### Errores
    ///  * `ErrorLog::ReceiverNotFound`: It will appear when the receiver it's drop and can't send the message
    pub fn log_file(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::FILE, mensaje)?;
        Ok(())
    }

    /// Sends the desired message with level: `Level::INTERFACE`
    ///
    /// ### Errores
    ///  * `ErrorLog::ReceiverNotFound`: It will appear when the receiver it's drop and can't send the message
    pub fn log_interface(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::INTERFACE, mensaje)?;
        Ok(())
    }

    /// Sends the desired message with level: `Level::NOTIFICATION`
    ///
    /// ### Errores
    ///  * `ErrorLog::ReceiverNotFound`: It will appear when the receiver it's drop and can't send the message
    pub fn log_notification(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::NOTIFICATION, mensaje)?;
        Ok(())
    }

    /// Sends the desired message with level: `Level::BROADCASTING`
    ///
    /// ### Errores
    ///  * `ErrorLog::ReceiverNotFound`: It will appear when the receiver it's drop and can't send the message
    pub fn log_broadcasting(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::BROADCASTING, mensaje)?;
        Ok(())
    }

    /// Sends the desired message with level: `Level::CONNECTION`
    ///
    /// ### Errores
    ///  * `ErrorLog::ReceiverNotFound`: It will appear when the receiver it's drop and can't send the message
    pub fn log_error(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.log(Level::ERROR, mensaje)?;
        Ok(())
    }
}
