use cargosos_bitcoin::{
    block_structure::{block::Block, block_header::BlockHeader, transaction::Transaction},
    connections::{p2p_protocol::ProtocolVersionP2P, supported_services::SupportedServices},
    messages::{
        bitfield_services::BitfieldServices, block_message::BlockMessage,
        headers_message::HeadersMessage, message::Message, tx_message::TxMessage,
        verack_message::VerackMessage, version_message::VersionMessage,
    },
    node_structure::{error_node::ErrorNode, handshake_data::HandshakeData},
    serialization::error_serialization::ErrorSerialization,
};

use std::{
    io::Write,
    net::Ipv4Addr,
};

use chrono::{offset::Utc, DateTime, NaiveDateTime};

pub fn serialize_verack_message<W: Write>(
    stream: &mut W,
    magic_number: [u8; 4],
) -> Result<(), ErrorNode> {
    VerackMessage::serialize_message(stream, magic_number, &VerackMessage)?;
    Ok(())
}

pub fn serialize_version_message<W: Write>(
    stream: &mut W,
    protocol_version: ProtocolVersionP2P,
    services: BitfieldServices,
    block_height: i32,
    handshake_data: HandshakeData,
    local_ip: (Ipv4Addr, u16),
    remote_ip: (Ipv4Addr, u16),
) -> Result<(), ErrorNode> {
    let naive = NaiveDateTime::from_timestamp_opt(1234 as i64, 0).unwrap();
    let timestamp: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    let version_message = VersionMessage {
        version: protocol_version,
        services: services,
        timestamp,
        recv_services: BitfieldServices::new(vec![SupportedServices::Unname]),
        recv_addr: Ipv4Addr::to_ipv6_mapped(&local_ip.0),
        recv_port: local_ip.1,
        trans_addr: Ipv4Addr::to_ipv6_mapped(&remote_ip.0),
        trans_port: remote_ip.1,
        nonce: handshake_data.nonce,
        user_agent: handshake_data.user_agent.clone(),
        start_height: block_height,
        relay: handshake_data.relay,
    };

    VersionMessage::serialize_message(stream, handshake_data.magic_number, &version_message)?;

    Ok(())
}

pub fn serialize_headers_message<W: Write>(
    stream: &mut W,
    magic_numbers: [u8; 4],
    headers: Vec<BlockHeader>,
) -> Result<(), ErrorSerialization> {
    let headers_message = HeadersMessage { headers };
    HeadersMessage::serialize_message(stream, magic_numbers, &headers_message)
}

pub fn serialize_block_message<W: Write>(
    stream: &mut W,
    magic_numbers: [u8; 4],
    block: Block,
) -> Result<(), ErrorSerialization> {
    let block_message = BlockMessage { block };
    BlockMessage::serialize_message(stream, magic_numbers, &block_message)
}

pub fn serialize_tx_message<W: Write>(
    stream: &mut W,
    magic_numbers: [u8; 4],
    transaction: Transaction,
) -> Result<(), ErrorSerialization> {
    let tx_message = TxMessage { transaction };
    TxMessage::serialize_message(stream, magic_numbers, &tx_message)
}
