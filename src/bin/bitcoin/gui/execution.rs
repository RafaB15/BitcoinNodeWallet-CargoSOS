use super::{
    notifier_gui::NotifierGUI, signal_to_back::SignalToBack,
    signal_to_front::SignalToFront, input_handler_gui::InputHandlerGUI,
    frontend
};

use crate::{
    error_execution::ErrorExecution, 
    process::{save_system::SaveSystem, backend, load_system::LoadSystem}, 
};

use cargosos_bitcoin::{
    logs::logger_sender::LoggerSender,
    notifications::notifier::Notifier,
    configurations::{
        connection_config::ConnectionConfig, download_config::DownloadConfig, mode_config::ModeConfig,
        save_config::SaveConfig,
    }
};

use std::{
    cell::Cell, 
    sync::mpsc::{Receiver, channel},
    thread::{self, JoinHandle},
};

use gtk::{prelude::*, Application, glib};

/// Function that spawns the backend handler thread
fn spawn_backend_handler<N: Notifier + 'static>(
    mode_config: ModeConfig,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    save_config: SaveConfig,
    rx_from_front: Receiver<SignalToBack>,
    notifier: N,
    logger: LoggerSender,
) -> JoinHandle<Result<SaveSystem, ErrorExecution>> {
    thread::spawn(move || {
        let mut load_system = LoadSystem::new(save_config.clone(), logger.clone());
        
        let input_handler = InputHandlerGUI::new(
            rx_from_front,
            notifier.clone(),
            logger.clone(),
        );

        backend::backend(
            mode_config,
            connection_config,
            download_config,
            &mut load_system,
            input_handler,
            notifier,
            logger,
        )
    })
}

/// The main function of the program for the graphical interface.
pub fn program_execution(
    mode_config: ModeConfig,
    connection_config: ConnectionConfig,
    download_config: DownloadConfig,
    save_config: SaveConfig,
    logger: LoggerSender,
) -> Result<SaveSystem, ErrorExecution> {
    let (tx_to_back, rx_from_front) = channel::<SignalToBack>();
    let (tx_to_front, rx_from_back) =
        glib::MainContext::channel::<SignalToFront>(glib::PRIORITY_DEFAULT);

    let notifier = NotifierGUI::new(tx_to_front, logger.clone());

    let backend_handler = spawn_backend_handler(
        mode_config,
        connection_config,
        download_config,
        save_config,
        rx_from_front,
        notifier,
        logger,
    );

    let glade_src = include_str!("WindowNotebook.glade");

    let application = Application::builder().build();

    let wrapped_rx_to_back: Cell<Option<gtk::glib::Receiver<SignalToFront>>> =
        Cell::new(Some(rx_from_back));

    application.connect_activate(move |app| {
        if let Err(error) = frontend::build_ui(
            tx_to_back.clone(),
            wrapped_rx_to_back.take(),
            app,
            glade_src,
        ) {
            println!("Error: {:?}", error);
        }
    });
    let vector: Vec<String> = Vec::new();
    application.run_with_args(&vector);

    match backend_handler.join() {
        Ok(save_system) => save_system,
        Err(_) => Err(ErrorExecution::FailThread),
    }
}
