/// Networking module - multiplayer support
#[cfg(feature = "network")]
pub mod packet;
#[cfg(feature = "network")]
pub mod base_protocol;
#[cfg(feature = "network")]
pub mod storm;
#[cfg(feature = "network")]
pub mod transport;
#[cfg(feature = "network")]
pub mod protocol;
#[cfg(feature = "network")]
pub mod client;

#[cfg(feature = "network")]
pub use protocol::GameProtocol;
#[cfg(feature = "network")]
pub use client::GameClient;
