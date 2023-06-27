use std::net::SocketAddr;

pub type NotificationSender = std::sync::mpsc::Sender<Notification>;
pub type NotificationReceiver = std::sync::mpsc::Receiver<Notification>;

pub enum Notification {
    AttemptingHandshakeWithPeer(SocketAddr),
    SuccessfulHandshakeWithPeer(SocketAddr),
    FailedHandshakeWithPeer(SocketAddr),
}