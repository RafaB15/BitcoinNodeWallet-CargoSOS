use super::message_header::MessageHeader;
use super::message_payload::MessagePayload;

pub struct NetworkMessage {
    pub header: MessageHeader,
    pub payload: MessagePayload
}