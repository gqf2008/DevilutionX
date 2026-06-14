//! Storm Network Functions
//!
//! C++ Source: Source/storm/storm_net.cpp
//! C++ Header: Source/storm/storm_net.hpp
//!
//! Storm network API - the multiplayer networking interface.
//! Uses dvlnet (DevilutionX networking layer) internally.

// TODO: Port from Source/storm/storm_net.cpp
//
// Key functions to implement:
// - SNetReceiveMessage - Receive network message
// - SNetSendMessage - Send network message
// - SNetReceiveTurns - Receive turn data from all players
// - SNetSendTurn - Send turn data
// - SNetGetProviderCaps - Get network provider capabilities
// - SNetRegisterEventHandler - Register event callback
// - SNetUnregisterEventHandler - Unregister event callback
// - SNetDestroy - Cleanup network
// - SNetDropPlayer - Drop a player from game
// - SNetLeaveGame - Leave current game
// - SNetInitializeProvider - Initialize network provider
// - SNetCreateGame - Create multiplayer game
// - SNetJoinGame - Join multiplayer game
// - SNetGetOwnerTurnsWaiting - Get pending turns
// - SNetGetTurnsInTransit - Get turns in transit
// - SNetSetBasePlayer - Set base player
//
// DvlNet functions:
// - DvlNet_ProcessNetworkPackets
// - DvlNet_SendInfoRequest
// - DvlNet_ClearGamelist
// - DvlNet_GetGamelist
// - DvlNet_SetPassword
// - DvlNet_ClearPassword
// - DvlNet_IsPublicGame
// - DvlNet_GetLatencies

// TODO: Will use dvlnet when implementing
// use crate::dvlnet;

/// Network event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    PlayerMessage,
    PlayerLeave,
    PlayerJoin,
}

/// Leave info flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaveInfo(pub u32);

impl LeaveInfo {
    pub const NORMAL: Self = Self(0);
    pub const DROP: Self = Self(1);
}

/// Network capabilities
#[derive(Debug, Clone, Default)]
pub struct SNetCaps {
    pub size: u32,
    pub flags: u32,
    pub max_message_size: u32,
    pub max_queue_size: u32,
    pub max_players: u32,
    pub bytes_per_second: u32,
    pub latency_ms: u32,
    pub default_turn_delay: u32,
    pub default_turn_seconds: u32,
}

/// Game latency info
#[derive(Debug, Clone, Default)]
pub struct DvlNetLatencies {
    pub last_ms: u32,
    pub min_ms: u32,
    pub max_ms: u32,
    pub avg_ms: u32,
}

// TODO: Implement all SNet* functions
// These are stubs for now

pub fn snet_receive_message(_sender_id: &mut u8, _data: &mut Vec<u8>) -> bool {
    // TODO: Implement
    false
}

pub fn snet_send_message(_player_id: u8, _data: &[u8]) -> bool {
    // TODO: Implement
    false
}

pub fn snet_destroy() -> bool {
    // TODO: Implement
    true
}
