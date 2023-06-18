use super::{error_process::ErrorProcess, message_response::MessageResponse};

use cargosos_bitcoin::{
    block_structure::transaction::Transaction,
    messages::{
        addr_message::AddrMessage,
        alert_message::AlertMessage,
        block_message::BlockMessage,
        command_name::CommandName,
        fee_filter_message::FeeFilterMessage,
        get_data_message::GetDataMessage,
        get_headers_message::GetHeadersMessage,
        headers_message::HeadersMessage,
        inventory_message::InventoryMessage,
        message::{ignore_message, Message},
        message_header::MessageHeader,
        ping_message::PingMessage,
        pong_message::PongMessage,
        send_cmpct::SendCmpctMessage,
        send_headers::SendHeadersMessage,
        verack_message::VerackMessage,
        version_message::VersionMessage,
    },
};

use std::{
    io::{Read, Write},
    sync::mpsc::{Receiver, Sender},
};

pub struct PeerManager<RW>
where
    RW: Read + Write + Send + 'static,
{
    peer: RW,
    sender: Sender<MessageResponse>,
    receiver: Receiver<MessageResponse>,
}

impl<RW> PeerManager<RW>
where
    RW: Read + Write + Send + 'static,
{
    pub fn new(
        peer: RW,
        sender: Sender<MessageResponse>,
        receiver: Receiver<MessageResponse>,
    ) -> Self {
        PeerManager {
            peer,
            sender,
            receiver,
        }
    }

    pub fn listen_peers(mut self) -> Result<RW, ErrorProcess> {
        while let Ok(header) = MessageHeader::deserialize_header(&mut self.peer) {
            self.manage_message(header)?;

            if let Ok(message) = self.receiver.try_recv() {
                match message {
                    MessageResponse::Transaction(transaction) => self.send_transaction(transaction),
                    MessageResponse::Exit => break,
                    _ => (),
                }
            }
        }

        Ok(self.peer)
    }

    fn manage_message(&mut self, header: MessageHeader) -> Result<(), ErrorProcess> {
        let magic_numbers = header.magic_numbers;

        match header.command_name {
            CommandName::Version => ignore_message::<VersionMessage>(&mut self.peer, header)?,
            CommandName::Verack => ignore_message::<VerackMessage>(&mut self.peer, header)?,
            CommandName::Ping => {
                let ping = PingMessage::deserialize_message(&mut self.peer, header)?;

                let pong = PongMessage { nonce: ping.nonce };

                PongMessage::serialize_message(&mut self.peer, magic_numbers, &pong)?;
            }
            CommandName::Pong => ignore_message::<PongMessage>(&mut self.peer, header)?,
            CommandName::GetHeaders => ignore_message::<GetHeadersMessage>(&mut self.peer, header)?,
            CommandName::Headers => ignore_message::<HeadersMessage>(&mut self.peer, header)?,
            CommandName::GetData => ignore_message::<GetDataMessage>(&mut self.peer, header)?,
            CommandName::Block => ignore_message::<BlockMessage>(&mut self.peer, header)?,
            CommandName::Inventory => ignore_message::<InventoryMessage>(&mut self.peer, header)?,
            CommandName::SendHeaders => {
                ignore_message::<SendHeadersMessage>(&mut self.peer, header)?
            }
            CommandName::SendCmpct => ignore_message::<SendCmpctMessage>(&mut self.peer, header)?,
            CommandName::Addr => ignore_message::<AddrMessage>(&mut self.peer, header)?,
            CommandName::FeeFilter => ignore_message::<FeeFilterMessage>(&mut self.peer, header)?,
            CommandName::Alert => ignore_message::<AlertMessage>(&mut self.peer, header)?,
        }

        Ok(())
    }

    fn send_transaction(&self, _transaction: Transaction) {
        todo!()
    }
}
