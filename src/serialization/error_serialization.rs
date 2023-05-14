
#[derive(Debug)]
pub enum ErrorSerialization {
    ErrorInSerialization(String),

    ErrorInDeserialization(String),

    ErrorWhileWriting,

    ErrorWhileReading,
}