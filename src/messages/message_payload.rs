use crate::messages::version_message::VersionMessage;

pub enum MessagePayload {
    Version(VersionMessage)
}