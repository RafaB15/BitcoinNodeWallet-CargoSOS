use super::version_message::VersionMessage;

pub enum MessagePayload {
    Version(VersionMessage)
}