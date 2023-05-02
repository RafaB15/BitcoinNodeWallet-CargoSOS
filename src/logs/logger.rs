

pub(crate) type MessageLog = (Level, String) //podria agregar el TIME STAMP, al principio?

/// Crear el sender-receiver

pub fn initialize_logger(logger_file: &Path) -> Result<(LoggerSender, LoggerReceiver), ErrorString> {
    let (sender, receiver) = mpsc::channel::<MessageLog>();

    ///creamos el sender y el receiver donde mandamos tambien el path del archivo para poder escribir
    let logger_sender = LoggerSender::new(sender);
    let logger_receiver = LoggerReceiver::new(logger_file, receiver)?;

    Ok((logger_sender, logger_receiver))

}


