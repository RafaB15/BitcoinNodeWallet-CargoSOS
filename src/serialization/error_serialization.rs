/// It represents all posible errors that can occur in the process of serializing and deserializing
#[derive(Debug)]
pub enum ErrorSerialization {
    /// It will appear when there is an error in the serialization
    ErrorInSerialization(String),

    /// It will appear when there is an error in the deserialization
    ErrorInDeserialization(String),

    /// It will appear when the connection is lost
    ConnectionAborted,
    
    /// It will appear when the information is not yet send to the stream
    InformationNotReady,

    /// It will appear when there is an error in the writing to a stream
    ErrorWhileWriting,

    /// It will appear when there is an error in the reading from a stream
    ErrorWhileReading,
}
