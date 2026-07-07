//! Multiplayer System - M24
//!
//! Precision port of DevilutionX multi.cpp
//! Handles multiplayer game synchronization and networking

// ============================================================================
// Constants
// ============================================================================

/// Maximum number of players
pub const MAX_PLRS: usize = 4;

/// Buffer size for network packets
pub const NET_BUFFER_SIZE: usize = 4096;

/// Network event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventType {
    /// Player left the game
    PlayerLeaveGame = 0,
    /// Player created a game
    PlayerCreateGame = 1,
    /// Player sent a message
    PlayerMessage = 2,
}

/// Leave reason codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum LeaveReason {
    #[default]
    Unknown = 0,
    /// Normal disconnect
    Normal = 1,
    /// Connection lost
    ConnectionLost = 2,
    /// Kicked by host
    Kicked = 3,
    /// Game ended
    GameEnded = 4,
}

impl LeaveReason {
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Normal,
            2 => Self::ConnectionLost,
            3 => Self::Kicked,
            4 => Self::GameEnded,
            _ => Self::Unknown,
        }
    }
}

// ============================================================================
// Game Data
// ============================================================================

/// Game initialization data
#[derive(Debug, Clone, Default)]
pub struct GameData {
    /// Size of the data structure
    pub size: u32,
    /// Program ID for version checking
    pub program_id: u32,
    /// Game seed (4 parts)
    pub game_seed: [u32; 4],
    /// Tick rate
    pub tick_rate: u8,
    /// Run in town enabled
    pub run_in_town: bool,
    /// Theo quest enabled
    pub theo_quest: bool,
    /// Cow quest enabled
    pub cow_quest: bool,
    /// Friendly fire mode
    pub friendly_fire: u8,
    /// Full quests enabled
    pub full_quests: bool,
}

impl GameData {
    pub fn new() -> Self {
        Self {
            size: std::mem::size_of::<Self>() as u32,
            program_id: 0x44494142, // "DIAB"
            ..Default::default()
        }
    }

    /// Swap to little endian
    pub fn to_le(&mut self) {
        self.size = self.size.to_le();
        self.program_id = self.program_id.to_le();
        for seed in &mut self.game_seed {
            *seed = seed.to_le();
        }
    }

    /// Swap from little endian
    pub fn from_le(&mut self) {
        self.size = u32::from_le(self.size);
        self.program_id = u32::from_le(self.program_id);
        for seed in &mut self.game_seed {
            *seed = u32::from_le(*seed);
        }
    }
}

// ============================================================================
// Network Buffer
// ============================================================================

/// Network transmit buffer
#[derive(Debug)]
pub struct NetBuffer {
    /// Current write offset
    write_offset: usize,
    /// Buffer data
    data: [u8; NET_BUFFER_SIZE],
}

impl Default for NetBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl NetBuffer {
    pub fn new() -> Self {
        Self {
            write_offset: 0,
            data: [0; NET_BUFFER_SIZE],
        }
    }

    /// Reset buffer
    pub fn clear(&mut self) {
        self.write_offset = 0;
        self.data[0] = 0;
    }

    /// Copy packet to buffer
    pub fn copy_packet(&mut self, packet: &[u8]) -> bool {
        if self.write_offset + packet.len() + 2 > NET_BUFFER_SIZE {
            return false;
        }

        self.data[self.write_offset] = packet.len() as u8;
        self.write_offset += 1;

        self.data[self.write_offset..self.write_offset + packet.len()].copy_from_slice(packet);
        self.write_offset += packet.len();
        self.data[self.write_offset] = 0;

        true
    }

    /// Get buffer contents
    pub fn get_data(&self) -> &[u8] {
        &self.data[..self.write_offset]
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.write_offset == 0
    }

    /// Get remaining capacity
    pub fn remaining(&self) -> usize {
        NET_BUFFER_SIZE - self.write_offset
    }
}

// ============================================================================
// Player State
// ============================================================================

/// Player connection state flags
pub mod player_state {
    pub const CONNECTED: u32 = 1 << 0;
    pub const ACTIVE: u32 = 1 << 1;
    pub const TURN_PENDING: u32 = 1 << 2;
    pub const DELTA_PENDING: u32 = 1 << 3;
    pub const LEFT_GAME: u32 = 1 << 4;
}

/// Multiplayer player info
#[derive(Debug, Clone, Default)]
pub struct MultiplayerPlayer {
    /// Player state flags
    pub state: u32,
    /// Has player's turn this frame
    pub has_turn: bool,
    /// Player left game
    pub left_game: bool,
    /// Send delta to this player
    pub send_delta: bool,
    /// Leave reason
    pub leave_reason: LeaveReason,
    /// Info request timer
    pub info_timer: u32,
}

impl MultiplayerPlayer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if player is connected
    pub fn is_connected(&self) -> bool {
        self.state & player_state::CONNECTED != 0
    }

    /// Check if player is active
    pub fn is_active(&self) -> bool {
        self.state & player_state::ACTIVE != 0
    }

    /// Set connected state
    pub fn set_connected(&mut self, connected: bool) {
        if connected {
            self.state |= player_state::CONNECTED;
        } else {
            self.state &= !player_state::CONNECTED;
        }
    }

    /// Set active state
    pub fn set_active(&mut self, active: bool) {
        if active {
            self.state |= player_state::ACTIVE;
        } else {
            self.state &= !player_state::ACTIVE;
        }
    }
}

// ============================================================================
// Multiplayer Manager
// ============================================================================

/// Multiplayer game manager
pub struct MultiplayerManager {
    /// Is multiplayer game
    pub is_multiplayer: bool,
    /// Is network initialized
    pub net_initialized: bool,
    /// Is loopback mode (fake multiplayer for testing)
    pub is_loopback: bool,
    /// Game is destroyed
    pub game_destroyed: bool,
    /// Select provider screen pending
    pub select_provider: bool,
    /// Someone won the game
    pub somebody_won: bool,
    /// Timeout active
    pub timeout_active: bool,
    /// Timeout start time
    pub timeout_start: i32,

    /// Number of active players
    pub active_players: u8,
    /// Delta sender player ID
    pub delta_sender: u8,

    /// Game loops counter
    pub game_loops: u32,
    /// Packets sent this cycle
    pub sent_this_cycle: u32,

    /// Game name
    pub game_name: String,
    /// Game password
    pub game_password: String,
    /// Is public game
    pub public_game: bool,

    /// Game initialization data
    pub game_init_info: GameData,

    /// High priority send buffer
    high_priority_buffer: NetBuffer,
    /// Low priority send buffer
    low_priority_buffer: NetBuffer,

    /// Share next high priority message
    pub share_next_high_priority: bool,

    /// Player multiplayer state
    pub players: [MultiplayerPlayer; MAX_PLRS],

    /// Pack player offset table
    pub pack_player_offsets: [u16; MAX_PLRS],
}

impl Default for MultiplayerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiplayerManager {
    pub fn new() -> Self {
        Self {
            is_multiplayer: false,
            net_initialized: false,
            is_loopback: false,
            game_destroyed: false,
            select_provider: false,
            somebody_won: false,
            timeout_active: false,
            timeout_start: 0,
            active_players: 1,
            delta_sender: 0,
            game_loops: 0,
            sent_this_cycle: 0,
            game_name: String::new(),
            game_password: String::new(),
            public_game: false,
            game_init_info: GameData::new(),
            high_priority_buffer: NetBuffer::new(),
            low_priority_buffer: NetBuffer::new(),
            share_next_high_priority: false,
            players: Default::default(),
            pack_player_offsets: [0; MAX_PLRS],
        }
    }

    /// Initialize for single player
    pub fn init_single_player(&mut self) {
        self.is_multiplayer = false;
        self.active_players = 1;
        self.players[0].set_connected(true);
        self.players[0].set_active(true);
    }

    /// Initialize for multiplayer
    pub fn init_multiplayer(&mut self, game_name: &str, password: &str, public_game: bool) {
        self.is_multiplayer = true;
        self.game_name = game_name.to_string();
        self.game_password = password.to_string();
        self.public_game = public_game;
        // A multiplayer game starts empty: players join explicitly via
        // `player_joined`. Unlike single player (where the host is always
        // active), the host hasn't been registered yet at this point.
        self.active_players = 0;
        self.clear_buffers();
    }

    /// Clear send buffers
    pub fn clear_buffers(&mut self) {
        self.high_priority_buffer.clear();
        self.low_priority_buffer.clear();
        self.sent_this_cycle = 0;
    }

    /// Queue high priority message
    pub fn queue_high_priority(&mut self, packet: &[u8]) -> bool {
        self.high_priority_buffer.copy_packet(packet)
    }

    /// Queue low priority message
    pub fn queue_low_priority(&mut self, packet: &[u8]) -> bool {
        self.low_priority_buffer.copy_packet(packet)
    }

    /// Get player count
    pub fn player_count(&self) -> u8 {
        self.active_players
    }

    /// Check if player is in game
    pub fn is_player_active(&self, player_id: usize) -> bool {
        if player_id < MAX_PLRS {
            self.players[player_id].is_active()
        } else {
            false
        }
    }

    /// Handle player leaving
    pub fn player_left(&mut self, player_id: usize, reason: LeaveReason) {
        if player_id < MAX_PLRS {
            self.players[player_id].left_game = true;
            self.players[player_id].leave_reason = reason;
            self.players[player_id].set_active(false);
            if self.active_players > 0 {
                self.active_players -= 1;
            }
        }
    }

    /// Handle player joining
    pub fn player_joined(&mut self, player_id: usize) {
        if player_id < MAX_PLRS {
            self.players[player_id].set_connected(true);
            self.players[player_id].set_active(true);
            self.players[player_id].left_game = false;
            self.active_players += 1;
        }
    }

    /// Check if uses multiplayer quests
    pub fn use_multiplayer_quests(&self) -> bool {
        self.is_multiplayer && self.active_players > 1
    }

    /// Increment game loop counter
    pub fn tick(&mut self) {
        self.game_loops += 1;
    }

    /// Start timeout
    pub fn start_timeout(&mut self, start_time: i32) {
        self.timeout_active = true;
        self.timeout_start = start_time;
    }

    /// Clear timeout
    pub fn clear_timeout(&mut self) {
        self.timeout_active = false;
        self.timeout_start = 0;
    }
}

// ============================================================================
// Network Message Types
// ============================================================================

/// Network command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NetCmd {
    /// Walk command
    Walk = 0,
    /// Stand command
    Stand = 1,
    /// Attack command
    Attack = 2,
    /// Cast spell command
    Spell = 3,
    /// Talk to NPC
    TalkNpc = 4,
    /// Operate object
    OperateObj = 5,
    /// Pickup item
    PickupItem = 6,
    /// Drop item
    DropItem = 7,
    /// Use item
    UseItem = 8,
    /// Change equipment
    ChangeEquip = 9,
    /// Delete item
    DelItem = 10,
    /// Player damage
    Damage = 11,
    /// Player death
    Death = 12,
    /// Resurrect
    Resurrect = 13,
    /// Heal other
    HealOther = 14,
    /// Chat message
    String = 15,
    /// Sync data
    Sync = 16,
    /// Monster action
    Monster = 17,
    /// Quest update
    Quest = 18,
    /// End game
    EndGame = 19,
}

impl NetCmd {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Walk),
            1 => Some(Self::Stand),
            2 => Some(Self::Attack),
            3 => Some(Self::Spell),
            4 => Some(Self::TalkNpc),
            5 => Some(Self::OperateObj),
            6 => Some(Self::PickupItem),
            7 => Some(Self::DropItem),
            8 => Some(Self::UseItem),
            9 => Some(Self::ChangeEquip),
            10 => Some(Self::DelItem),
            11 => Some(Self::Damage),
            12 => Some(Self::Death),
            13 => Some(Self::Resurrect),
            14 => Some(Self::HealOther),
            15 => Some(Self::String),
            16 => Some(Self::Sync),
            17 => Some(Self::Monster),
            18 => Some(Self::Quest),
            19 => Some(Self::EndGame),
            _ => None,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_data() {
        let mut data = GameData::new();
        assert_eq!(data.program_id, 0x44494142);

        data.game_seed = [1, 2, 3, 4];
        data.to_le();
        data.from_le();
        assert_eq!(data.game_seed, [1, 2, 3, 4]);
    }

    #[test]
    fn test_net_buffer() {
        let mut buffer = NetBuffer::new();
        assert!(buffer.is_empty());

        let packet = [1, 2, 3, 4, 5];
        assert!(buffer.copy_packet(&packet));
        assert!(!buffer.is_empty());

        buffer.clear();
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_net_buffer_overflow() {
        let mut buffer = NetBuffer::new();
        let large_packet = [0u8; NET_BUFFER_SIZE];
        assert!(!buffer.copy_packet(&large_packet));
    }

    #[test]
    fn test_multiplayer_player() {
        let mut player = MultiplayerPlayer::new();
        assert!(!player.is_connected());
        assert!(!player.is_active());

        player.set_connected(true);
        assert!(player.is_connected());

        player.set_active(true);
        assert!(player.is_active());

        player.set_connected(false);
        assert!(!player.is_connected());
    }

    #[test]
    fn test_multiplayer_manager_single() {
        let mut manager = MultiplayerManager::new();
        manager.init_single_player();

        assert!(!manager.is_multiplayer);
        assert_eq!(manager.active_players, 1);
        assert!(manager.is_player_active(0));
    }

    #[test]
    fn test_multiplayer_manager_multi() {
        let mut manager = MultiplayerManager::new();
        manager.init_multiplayer("TestGame", "password", true);

        assert!(manager.is_multiplayer);
        assert_eq!(manager.game_name, "TestGame");
        assert!(manager.public_game);
    }

    #[test]
    fn test_player_join_leave() {
        let mut manager = MultiplayerManager::new();
        manager.init_multiplayer("Test", "", false);

        manager.player_joined(0);
        assert_eq!(manager.active_players, 1);
        assert!(manager.is_player_active(0));

        manager.player_joined(1);
        assert_eq!(manager.active_players, 2);

        manager.player_left(1, LeaveReason::Normal);
        assert_eq!(manager.active_players, 1);
        assert!(!manager.is_player_active(1));
    }

    #[test]
    fn test_net_cmd() {
        assert_eq!(NetCmd::from_u8(0), Some(NetCmd::Walk));
        assert_eq!(NetCmd::from_u8(15), Some(NetCmd::String));
        assert_eq!(NetCmd::from_u8(255), None);
    }

    #[test]
    fn test_leave_reason() {
        assert_eq!(LeaveReason::from_u8(1), LeaveReason::Normal);
        assert_eq!(LeaveReason::from_u8(2), LeaveReason::ConnectionLost);
        assert_eq!(LeaveReason::from_u8(255), LeaveReason::Unknown);
    }
}
