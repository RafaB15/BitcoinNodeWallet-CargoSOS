use super::connection_id::ConnectionId;

use crate::concurrency::work::Work;

use std::convert::From;

#[derive(Debug)]
pub enum ConnectionEvent {
    PotentialConnection(ConnectionId),
    Stop,
}

impl From<ConnectionEvent> for Work<ConnectionId> {
    fn from(connection_event: ConnectionEvent) -> Self {
        match connection_event {
            ConnectionEvent::PotentialConnection(connection_id) => Work::Information(connection_id),
            ConnectionEvent::Stop => Work::Stop,
        }
    }
}