use super::connection_id::ConnectionId;

#[derive(Debug)]
pub enum ConnectionEvent {
    PotentialConnection(ConnectionId),
    Stop,
}