/// Networking module - multiplayer support
#[cfg(feature = "network")]
pub mod protocol;
#[cfg(feature = "network")]
pub mod client;

#[cfg(feature = "network")]
pub use protocol::GameProtocol;
#[cfg(feature = "network")]
pub use client::GameClient;
