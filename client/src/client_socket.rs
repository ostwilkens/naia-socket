
use super::{message_sender::MessageSender, socket_event::SocketEvent};
use crate::error::NaiaClientSocketError;

/// Defines the functionality of a Naia Client Socket
pub trait ClientSocketTrait {
    /// Receive a new packet from the socket, or a tick event
    fn receive(&mut self) -> Result<SocketEvent, NaiaClientSocketError>;
    /// Gets a MessageSender you can use to send messages through the Server
    /// Socket
    fn get_sender(&mut self) -> MessageSender;
}

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        // WebRTC Client //
        pub use crate::webrtc_client_socket::WebrtcClientSocket;
        /// ClientSocket is an alias for a socket abstraction using either UDP or WebRTC for communications
        pub type ClientSocket = WebrtcClientSocket;
    }
    else {
        // UDP Client //
        pub use crate::udp_client_socket::UdpClientSocket;
        /// ClientSocket is an alias for a socket abstraction using either UDP or WebRTC for communications
        pub type ClientSocket = UdpClientSocket;
    }
}
