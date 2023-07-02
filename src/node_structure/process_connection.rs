use super::{
    connection_event::ConnectionEvent, connection_id::ConnectionId,
    connection_type::ConnectionType, error_node::ErrorNode, handshake::Handshake,
    handshake_data::HandshakeData,
};

use crate::{
    concurrency::{stop::Stop, work::Work},
    configurations::connection_config::ConnectionConfig,
    logs::logger_sender::LoggerSender,
    notifications::{notification::Notification, notifier::Notifier},
    serialization::error_serialization::ErrorSerialization,
};

use std::{
    net::SocketAddr,
    net::TcpStream,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

pub type SenderConfirm = Sender<(TcpStream, ConnectionId)>;
pub type ReceiverConfirm = Receiver<(TcpStream, ConnectionId)>;

pub type SenderPotential = Sender<ConnectionEvent>;
pub type ReceiverPotential = Receiver<ConnectionEvent>;

pub struct ProcessConnection<N: Notifier + Send + 'static> {
    handshake: Handshake,

    sender_confirm_connection: SenderConfirm,
    receiver_potential_connections: ReceiverPotential,

    notifier: N,
    logger: LoggerSender,
}

impl<N: Notifier + Send + 'static> ProcessConnection<N> {
    pub fn new(
        connection_config: ConnectionConfig,
        sender_confirm_connection: SenderConfirm,
        receiver_potential_connections: ReceiverPotential,
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

    /// Handle the incoming potentail connections
    ///
    /// ###
    ///  * `ErrorNode::WhileSendingMessage`: It will appear when there is an error while sending a message to a peer or others threads
    ///  * `ErrorNode::FailThread`: It will appear when thread is poisoned
    pub fn execution(self) -> Result<(), ErrorNode> {
        let mut pending_connection_handlers: Vec<(JoinHandle<()>, Sender<Stop>)> = Vec::new();

        for connection_event in &self.receiver_potential_connections {
            match connection_event {
                ConnectionEvent::PotentialConnection(connection) => {
                    let (sender, receiver) = channel::<Stop>();

                    let handler = self.handle_connection_event(connection, receiver);

                    pending_connection_handlers.push((handler, sender));
                }
                ConnectionEvent::Stop => {
                    break;
                }
            }
        }

        let mut result = Ok(());
        for (handler, sender) in pending_connection_handlers {
            let _ = sender.send(Stop::Stop);
            if handler.join().is_err() {
                result = Err(ErrorNode::FailThread)
            }
        }

        result
    }

    /// Create a thread to handle the new potential connection to establish the handshake
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
                None => {
                    return;
                }
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
                ConnectionId {
                    address,
                    connection_type: ConnectionType::Peer,
                } => Self::connect_to_peer(
                    &mut stream,
                    &local_socket,
                    &address,
                    &handshake,
                    &receiver,
                ),
                ConnectionId {
                    address,
                    connection_type: ConnectionType::Client,
                } => Self::connect_to_client(
                    &mut stream,
                    &local_socket,
                    &address,
                    &handshake,
                    &receiver,
                ),
            };

            match result {
                Ok(true) => {
                    let _ = logger
                        .log_connection(format!("Connection established with {:?}", connection));
                    if sender_confirm_connection.send((stream, connection)).is_ok() {
                        notifier.notify(Notification::SuccessfulHandshakeWithPeer(
                            connection.address,
                        ));
                        notifier.notify(Notification::ConnectionUpdated(connection));
                    } else {
                        notifier.notify(Notification::FailedHandshakeWithPeer(connection.address));
                    }
                }
                Ok(false) => {}
                Err(_) => {
                    notifier.notify(Notification::FailedHandshakeWithPeer(connection.address));
                }
            }
        })
    }

    /// Establish the handshake with a peer
    ///
    /// ### Error
    ///  * `ErrorSerialization::ErrorSerialization`: It will appear when there is an error in the serialization
    ///  * `ErrorSerialization::ErrorInDeserialization`: It will appear when there is an error in the deserialization
    ///  * `ErrorSerialization::ErrorWhileReading`: It will appear when there is an error in the reading from a stream
    fn connect_to_peer(
        stream: &mut TcpStream,
        local_socket: &SocketAddr,
        potential_socket: &SocketAddr,
        handshake: &Handshake,
        receiver: &Receiver<Stop>,
    ) -> Result<bool, ErrorSerialization> {
        handshake.send_version_message(stream, local_socket, potential_socket)?;

        loop {
            match Work::listen(stream, &receiver) {
                Work::Message(header) => {
                    handshake.receive_version_message(stream, header, potential_socket)?;
                    break;
                }
                Work::Information(()) => continue,
                Work::Stop => {
                    return Ok(false);
                }
            }
        }

        handshake.send_verack_message(stream, potential_socket)?;

        loop {
            match Work::listen(stream, &receiver) {
                Work::Message(header) => {
                    handshake.receive_verack_message(stream, header, potential_socket)?;
                    break;
                }
                Work::Information(()) => continue,
                Work::Stop => {
                    return Ok(false);
                }
            }
        }

        handshake.send_sendheaders_message(stream)?;

        Ok(true)
    }

    /// Establish the handshake with a client
    ///
    /// ### Error
    ///  * `ErrorSerialization::ErrorSerialization`: It will appear when there is an error in the serialization
    ///  * `ErrorSerialization::ErrorInDeserialization`: It will appear when there is an error in the deserialization
    ///  * `ErrorSerialization::ErrorWhileReading`: It will appear when there is an error in the reading from a stream
    fn connect_to_client(
        stream: &mut TcpStream,
        local_socket: &SocketAddr,
        potential_socket: &SocketAddr,
        handshake: &Handshake,
        receiver: &Receiver<Stop>,
    ) -> Result<bool, ErrorSerialization> {
        loop {
            match Work::listen(stream, &receiver) {
                Work::Message(header) => {
                    handshake.receive_version_message(stream, header, potential_socket)?;
                    break;
                }
                Work::Information(()) => continue,
                Work::Stop => {
                    return Ok(false);
                }
            }
        }

        handshake.send_version_message(stream, local_socket, potential_socket)?;

        loop {
            match Work::listen(stream, &receiver) {
                Work::Message(header) => {
                    handshake.receive_verack_message(stream, header, potential_socket)?;
                    break;
                }
                Work::Information(()) => continue,
                Work::Stop => {
                    return Ok(false);
                }
            }
        }

        handshake.send_verack_message(stream, potential_socket)?;

        Ok(true)
    }

    /// Create a stream to connect to a potential connection
    fn create_stream(
        potential_connection: ConnectionId,
        logger: LoggerSender,
    ) -> Option<TcpStream> {
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
