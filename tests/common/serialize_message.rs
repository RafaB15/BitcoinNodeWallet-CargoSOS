use super::stream::Stream;

use cargosos_bitcoin::{
    serialization::{
        error_serialization::ErrorSerialization,
    },
    connections::{
        p2p_protocol::ProtocolVersionP2P,
        supported_services::SupportedServices,
        ibd_methods::IBDMethod,
        type_identifier::TypeIdentifier,
    },
    messages::{
        bitfield_services::BitfieldServices,
        addr_message::AddrMessage,
        alert_message::AlertMessage,
        block_message::BlockMessage,
        command_name::CommandName,
        fee_filter_message::FeeFilterMessage,
        get_data_message::GetDataMessage,
        get_headers_message::GetHeadersMessage,
        headers_message::HeadersMessage,
        inventory_message::InventoryMessage,
        inventory_vector::InventoryVector,
        message::{ignore_message, Message},
        message_header::MessageHeader,
        ping_message::PingMessage,
        pong_message::PongMessage,
        send_cmpct_message::SendCmpctMessage,
        send_headers_message::SendHeadersMessage,
        tx_message::TxMessage,
        verack_message::VerackMessage,
        version_message::VersionMessage,
    },
    block_structure::{
        block::Block,
        block_header::BlockHeader,
        block_chain::BlockChain,
        transaction::Transaction,
        hash::HashType,
    },
    node_structure::{
        handshake::Handshake,
        handshake_data::HandshakeData,
        initial_headers_download::InitialHeaderDownload,
        block_download::BlockDownload,
        peer_manager::PeerManager,
        message_response::MessageResponse,
        error_node::ErrorNode,
    },
};

use std::{
    io::{Read, Write},
    net::{Ipv4Addr, IpAddr, SocketAddr},
};

use chrono::{DateTime, NaiveDateTime, offset::Utc};

pub fn serialize_verack_message<RW: Read + Write>(
    stream: &mut RW,
    magic_number: [u8; 4],
) -> Result<(), ErrorNode> {
    VerackMessage::serialize_message(stream, magic_number, &VerackMessage)?;
    Ok(())
}

pub fn serialize_version_message<RW: Read + Write>(
    stream: &mut RW,
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

pub fn serialize_headers_message<RW: Read + Write>(
    stream: &mut RW,
    magic_numbers: [u8; 4],
    headers: Vec<BlockHeader>,
) -> Result<(), ErrorSerialization> {
    let headers_message = HeadersMessage { headers };
    HeadersMessage::serialize_message(stream, magic_numbers, &headers_message)
}

pub fn serialize_block_message<RW: Read + Write>(
    stream: &mut RW,
    magic_numbers: [u8; 4],
    block: Block,
) -> Result<(), ErrorSerialization> {
    let block_message = BlockMessage { block };
    BlockMessage::serialize_message(stream, magic_numbers, &block_message)
}

pub fn serialize_tx_message<RW: Read + Write>(
    stream: &mut RW,
    magic_numbers: [u8; 4],
    transaction: Transaction,
) -> Result<(), ErrorSerialization> {
    let tx_message = TxMessage { transaction };
    TxMessage::serialize_message(stream, magic_numbers, &tx_message)
}

pub fn serialize_inv_message<RW: Read + Write>(
    stream: &mut RW,
    magic_numbers: [u8; 4],
    inventory_vectors: Vec<InventoryVector>,
) -> Result<(), ErrorSerialization> {
    let inventory_message = InventoryMessage::new(inventory_vectors);
    InventoryMessage::serialize_message(stream, magic_numbers, &inventory_message)
}

pub fn serialize_ping_message<RW: Read + Write>(
    stream: &mut RW,
    magic_numbers: [u8; 4],
) -> Result<(), ErrorSerialization> {
    let ping_message = PingMessage { nonce: 1234 };
    PingMessage::serialize_message(stream, magic_numbers, &ping_message)
}