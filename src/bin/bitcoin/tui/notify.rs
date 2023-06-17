use crate::process::message_broadcasting::MessageBroadcasting;

use cargosos_bitcoin::logs::logger_sender::LoggerSender;

use std::{
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
};

// hacer que se pueda la informacion del usuario
// la idea es que de esta funcion se muestre el resultado al usuario 
// y que del main thread simplemente el usuario ingresa los valores
pub fn notification(
    receiver_broadcasting: Receiver<MessageBroadcasting>,
    logger: LoggerSender,
) -> JoinHandle<()> {
    thread::spawn(move || {
        for notification in receiver_broadcasting {
            match notification {
                _ => todo!(),
            }
        }
    })
}
