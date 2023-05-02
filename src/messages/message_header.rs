pub struct MessageHeader {
    pub magic_bytes: [u8; 4],
    pub message_type: [u8; 12],
    pub payload_size: u32,
    pub checksum: [u8; 4]
}