use super::logger::MessageLog;
use std::sync::mpsc::Sender;

use super::error_log::{ErrorLog};
use super::level::Level;

/// Se encarga de mandar los mensajes de logs, este se puede clonar para tener varios senders
/// 
/// ### Errores
///  * `Error::ErrorReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
#[derive(Debug, Clone)]
pub struct LoggerSender {
    sender: MessageLog,
}

impl LoggerSender {

    pub(crate) fn new(sender: Sender<MessageLog>) -> Self {
        LoggerSender { sender }
    }

    /// Envia el mensaje deseado con su nivel de prioridad
    /// ### Errores
    ///  * `Error::ErrorReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
    pub(crate) fn log(&self, level: Level, message: String) -> Result<(), ErrorLog>{
        if self.sender.send(level, message).is_err() {
            return Err(ErrorLog::ErrorReceiverNotFound);
        }
        Ok(())
    }

    /// Envia el mensaje deseado con el nivel `Nivel::NODE`
    /// 
    /// ### Errores
    ///  * `Error::ErrorReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
    pub fn log_node(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.loggear(Nivel::NODE, mensaje)?;
        Ok(())
    }

    /// Envia el mensaje deseado con el nivel `Nivel::WALLET`
    /// 
    /// ### Errores
    ///  * `Error::ErrorReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
    pub fn log_wallet(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.loggear(Nivel::WALLET, mensaje)?;
        Ok(())
    }

    /// Envia el mensaje deseado con el nivel `Nivel::TRANSACTION`
    /// 
    /// ### Errores
    ///  * `Error::ErrorReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
    pub fn log_transaction(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.loggear(Nivel::TRANSACTION, mensaje)?;
        Ok(())
    }

    /// Envia el mensaje deseado con el nivel `Nivel::CONFIGURATION`
    /// 
    /// ### Errores
    ///  * `Error::ErrorReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
    pub fn log_configuration(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.loggear(Nivel::CONFIGURATION, mensaje)?;
        Ok(())
    }

    /// Envia el mensaje deseado con el nivel `Nivel::CONNECTION`
    /// 
    /// ### Errores
    ///  * `Error::ErrorReceiverNotFound`: Este error puede aparecer cuando no existe un receiver
    pub fn log_connection(&self, mensaje: String) -> Result<(), ErrorLog> {
        self.loggear(Nivel::CONNECTION, mensaje)?;
        Ok(())
    }
}