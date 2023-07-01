use super::{
    connection_id::ConnectionId, connection_type::ConnectionType, handshake::Handshake,
    handshake_data::HandshakeData, connection_event::ConnectionEvent,
};

use crate::{
    configurations::connection_config::ConnectionConfig,
    logs::logger_sender::LoggerSender,
    notifications::{notification::Notification, notifier::Notifier},
    concurrency::{work::Work, stop::Stop}, serialization::error_serialization::ErrorSerialization,
};

use std::{
    net::TcpStream,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Duration,
    net::SocketAddr,
};

pub struct ProcessConnection<N: Notifier + Send + 'static> {
    handshake: Handshake,

    sender_confirm_connection: Sender<(TcpStream, ConnectionId)>,
    receiver_potential_connections: Receiver<ConnectionEvent>,

    notifier: N,
    logger: LoggerSender,
}

impl<N: Notifier + Send + 'static> ProcessConnection<N> {
    pub fn new(
        connection_config: ConnectionConfig,
        sender_confirm_connection: Sender<(TcpStream, ConnectionId)>,
        receiver_potential_connections: Receiver<ConnectionEvent>,
        notifier: N,
        logger: LoggerSender,
    ) -> Self {

        let handshake = Handshake::new(
            connection_config.p2p_protocol_version,
            connection_config.services,
            connection_config.block_height,
            HandshakeData {
                nonce: connection_config.nonce,
                user_agent: connection_config.user_agent,
                relay: connection_config.relay,
                magic_number: connection_config.magic_numbers,
            },
            logger.clone(),
        );

        Self {
            handshake,
            sender_confirm_connection,
            receiver_potential_connections,
            notifier,
            logger,
        }
    }

    pub fn execution(self) {
        let mut pending_connection_handlers: Vec<(JoinHandle<()>, Sender<Stop>)> = Vec::new();

        for connection_event in &self.receiver_potential_connections {

            match connection_event {
                ConnectionEvent::PotentialConnection(connection) => {
                    let (sender, receiver) = channel::<Stop>();

                    let handler = self.handle_connection_event(
                        connection, 
                        receiver,
                    );

                    pending_connection_handlers.push((handler, sender));
                }
                ConnectionEvent::Stop => {
                    break;
                }
            }
        }

        for (handler, sender) in pending_connection_handlers {
            let _ = sender.send(Stop::Stop);
            let _ = handler.join();
        }
    }

    fn handle_connection_event(
        &self,
        connection: ConnectionId, 
        receiver: Receiver<Stop>,
    ) -> JoinHandle<()> {

        let handshake = self.handshake.clone();
        let logger = self.logger.clone();
        let sender_confirm_connection = self.sender_confirm_connection.clone();
        let notifier = self.notifier.clone();

        thread::spawn(move || {

            let mut stream = match Self::create_stream(connection, logger.clone()) {
                Some(stream) => stream,
                None => { return; }
            };

            let local_socket = match stream.local_addr() {
                Ok(addr) => addr,
                Err(error) => {
                    let _ = logger
                        .log_connection(format!("Cannot get local address, it appear {:?}", error));
                    return;
                }
            };

            notifier.notify(Notification::AttemptingHandshakeWithPeer(
                connection.address.clone(),
            ));

            let result = match connection {
                ConnectionId { address, connection_type: ConnectionType::Peer } => {
                    Self::connect_to_peer(
                        &mut stream,
                        &local_socket,
                        &address,
                        &handshake,
                        &receiver,
                    )
                },
                ConnectionId { address, connection_type: ConnectionType::Client } => {
                    Self::connect_to_client(
                        &mut stream,
                        &local_socket,
                        &address,
                        &handshake,
                        &receiver,
                    )
                }
            };

            match result {
                Ok(true) => {
                    let _ = logger.log_connection(format!("Connection established with {:?}", connection));
                    if sender_confirm_connection.send((stream, connection)).is_ok() {
                        notifier.notify(Notification::SuccessfulHandshakeWithPeer(connection.address));
                    } else {
                        notifier.notify(Notification::FailedHandshakeWithPeer(connection.address));    
                    }
                },
                Ok(false) => {},
                Err(_) => {
                    notifier.notify(Notification::FailedHandshakeWithPeer(connection.address));
                },
            }
        })
    }

    fn connect_to_peer(
        stream: &mut TcpStream,
        local_socket: &SocketAddr,
        potential_socket: &SocketAddr,
        handshake: &Handshake,
        receiver: &Receiver<Stop>,
    ) -> Result<bool, ErrorSerialization>{
        handshake.send_version_message(
            stream,
            local_socket,
            potential_socket,
        )?;

        loop {
            match Work::listen(stream, &receiver) {
                Work::Message(header) => {
                    handshake.receive_version_message(stream, header, potential_socket)?;
                    break;
                }
                Work::Information(()) => continue,
                Work::Stop => { return Ok(false); }
            }
        }

        handshake.send_verack_message(
            stream,
            potential_socket,
        )?;


        loop {
            match Work::listen(stream, &receiver) {
                Work::Message(header) => {
                    handshake.receive_verack_message(stream, header, potential_socket)?;
                    break;
                }
                Work::Information(()) => continue,
                Work::Stop => { return Ok(false); }
            }
        }

        handshake.send_sendheaders_message(stream)?;

        Ok(true)
    }

    fn connect_to_client(
        stream: &mut TcpStream,
        local_socket: &SocketAddr,
        potential_socket: &SocketAddr,
        handshake: &Handshake,
        receiver: &Receiver<Stop>,
    ) -> Result<bool, ErrorSerialization>{
        loop {
            match Work::listen(stream, &receiver) {
                Work::Message(header) => {
                    handshake.receive_version_message(stream, header, potential_socket)?;
                    break;
                }
                Work::Information(()) => continue,
                Work::Stop => { return Ok(false); }
            }
        }

        handshake.send_version_message(
            stream,
            local_socket,
            potential_socket,
        )?;

        loop {
            match Work::listen(stream, &receiver) {
                Work::Message(header) => {
                    handshake.receive_verack_message(stream, header, potential_socket)?;
                    break;
                }
                Work::Information(()) => continue,
                Work::Stop => { return Ok(false); }
            }
        }

        handshake.send_verack_message(
            stream,
            potential_socket,
        )?;

        Ok(true)
    }

    fn create_stream(potential_connection: ConnectionId, logger: LoggerSender) -> Option<TcpStream>  {
        let stream = match TcpStream::connect(potential_connection.address) {
            Ok(stream) => stream,
            Err(error) => {
                let _ = logger.log_connection(format!(
                    "Cannot connect to address: {:?}, it appear {:?}",
                    potential_connection.address, error
                ));
                return None;
            }
        };

        if let Err(error) = stream.set_read_timeout(Some(Duration::from_secs(1))) {
            let _ = logger.log_connection(format!(
                "Cannot connect to address: {:?}, it appear {:?}",
                potential_connection.address, error
            ));
            return None;
        };

        Some(stream)
    }
}


