
use std::thread;
use std::time;
use log::info;

use laminar::{Packet as LaminarPacket, Socket as LaminarSocket, SocketEvent as LaminarEvent, Config as LaminarConfig};
use crossbeam_channel::{self, Receiver, Sender};

use super::socket_event::SocketEvent;
use super::message_sender::MessageSender;
use gaia_socket_shared::{SERVER_HANDSHAKE_MESSAGE, CLIENT_HANDSHAKE_MESSAGE};
use crate::error::GaiaServerSocketError;

/////

pub struct UdpServerSocket {
    sender: Sender<LaminarPacket>,
    receiver: Receiver<LaminarEvent>
}

impl UdpServerSocket {
    pub async fn bind(address: &str) -> UdpServerSocket {
        info!("UDP Server listening on: {}", address);

        let mut config = LaminarConfig::default();
        config.heartbeat_interval = Option::Some(time::Duration::from_millis(500));
        let mut socket = LaminarSocket::bind_with_config(address, config).unwrap();
        let (sender, receiver) = (socket.get_packet_sender(), socket.get_event_receiver());

        let _thread = thread::spawn(move || socket.start_polling());

        UdpServerSocket {
            sender,
            receiver,
        }
    }

    pub async fn receive(&mut self) -> Result<SocketEvent, GaiaServerSocketError> {
        let mut output: Option<Result<SocketEvent, GaiaServerSocketError>> = None;
        while output.is_none() {
            match self.receiver.recv() {
                Ok(event) => {
                    match event {
                        LaminarEvent::Connect(packet_addr) => {
                            self.sender.send(LaminarPacket::reliable_unordered(packet_addr, SERVER_HANDSHAKE_MESSAGE.to_string().into_bytes()))
                                .expect("send error");

                            output = Some(Ok(SocketEvent::Connection(packet_addr)));
                        }
                        LaminarEvent::Packet(packet) => {
                            let msg = String::from_utf8_lossy(packet.payload());

                            if !msg.eq(CLIENT_HANDSHAKE_MESSAGE) {
                                output = Some(Ok(SocketEvent::Message(packet.addr(), msg.to_string())));
                            }
                        }
                        LaminarEvent::Timeout(address) => {
                            output = Some(Ok(SocketEvent::Disconnection(address)));
                        }
                    }
                }
                Err(err) => {
                    output = Some(Err(GaiaServerSocketError::Wrapped(Box::new(err))));
                }
            }
        }
        return output.unwrap();
    }

    pub fn get_sender(&mut self) -> MessageSender {
        return MessageSender::new(self.sender.clone());
    }
}
