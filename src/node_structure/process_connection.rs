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

pub struct ProcessConnection {
    handshake: Handshake,

    sender_confirm_connection: Sender<(TcpStream, ConnectionId)>,
    receiver_potential_connections: Receiver<ConnectionEvent>,

    logger: LoggerSender,
}

impl ProcessConnection {
    pub fn new(
        connection_config: ConnectionConfig,
        sender_confirm_connection: Sender<(TcpStream, ConnectionId)>,
        receiver_potential_connections: Receiver<ConnectionEvent>,
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

        thread::spawn(move || {

            let mut stream = match create_stream(connection, logger.clone()) {
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

            if let Ok(true) = result {
                let _ = logger.log_connection(format!("Connection established with {:?}", connection));
                let _ = sender_confirm_connection.send((stream, connection));
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
}

fn create_stream(potential_connection: ConnectionId, logger: LoggerSender) -> Option<TcpStream>  {
    let mut stream = match TcpStream::connect(potential_connection.address) {
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

/// Creates a connection with the peers and if established then is return it's TCP stream
pub fn connect_to_peers<N: Notifier>(
    potential_connections: Vec<ConnectionId>,
    connection_config: ConnectionConfig,
    notifier: N,
    logger_sender: LoggerSender,
) -> Vec<(TcpStream, ConnectionId)> {
    let _ = logger_sender.log_connection("Connecting to potential peers".to_string());

    let node = Handshake::new(
        connection_config.p2p_protocol_version,
        connection_config.services,
        connection_config.block_height,
        HandshakeData {
            nonce: connection_config.nonce,
            user_agent: connection_config.user_agent,
            relay: connection_config.relay,
            magic_number: connection_config.magic_numbers,
        },
        logger_sender.clone(),
    );

    potential_connections
        .iter()
        .filter_map(|potential_peer| {
            filters_peer(
                *potential_peer,
                &node,
                logger_sender.clone(),
                notifier.clone(),
            )
        })
        .collect()
}

/// Creates a connection with a specific peer and if established then is return it's TCP stream
fn filters_peer<N: Notifier>(
    potential_connection: ConnectionId,
    node: &Handshake,
    logger_sender: LoggerSender,
    notifier: N,
) -> Option<(TcpStream, ConnectionId)> {
    let mut peer_stream = match TcpStream::connect(potential_connection.address) {
        Ok(stream) => stream,
        Err(error) => {
            let _ = logger_sender.log_connection(format!(
                "Cannot connect to address: {:?}, it appear {:?}",
                potential_connection, error
            ));
            return None;
        }
    };

    let local_socket = match peer_stream.local_addr() {
        Ok(addr) => addr,
        Err(error) => {
            let _ = logger_sender
                .log_connection(format!("Cannot get local address, it appear {:?}", error));
            return None;
        }
    };

    notifier.notify(Notification::AttemptingHandshakeWithPeer(
        potential_connection.address.clone(),
    ));

    let result = match potential_connection {
        ConnectionId { address, connection_type: ConnectionType::Peer } => {
            node.connect_to_peer(&mut peer_stream, &local_socket, &address)
        }, 
        ConnectionId { address, connection_type: ConnectionType::Client } => {
            node.connect_to_client(&mut peer_stream, &local_socket, &address)
        }, 
    };

    match result {
        Ok(_) => {
            notifier.notify(Notification::SuccessfulHandshakeWithPeer(potential_connection.address));
            Some((peer_stream, potential_connection))
        }
        Err(error) => {
            let _ = logger_sender.log_connection(format!(
                "Error while connecting to addres: {:?}, it appear {:?}",
                potential_connection, error
            ));
            notifier.notify(Notification::FailedHandshakeWithPeer(potential_connection.address));
            None
        }
    }
}
