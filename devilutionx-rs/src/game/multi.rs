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

    /// Loopback log of packets "sent" this session (drained by recv_packet).
    pub sent_log: Vec<Packet>,

    /// C++-compatible storm network session (wire-framed loopback). Populated
    /// in `net_init` for the loopback path so game commands travel through the
    /// dvlnet PT_MESSAGE wire format.
    #[cfg(feature = "network")]
    pub storm: crate::net::storm::StormNet,

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
            sent_log: Vec::new(),
            #[cfg(feature = "network")]
            storm: crate::net::storm::StormNet::new(),
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

    // ------------------------------------------------------------------------
    // Network lifecycle (stubs for single-player / loopback mode)
    // ------------------------------------------------------------------------

    /// Initialise the network layer.
    ///
    /// Port of `NetInit(bool bSinglePlayer)` from `multi.cpp`. The headless
    /// Rust port only supports loopback/single-player, so this is a simplified
    /// stub: it clears player state, activates the local player and seeds the
    /// game-init info. Returns `true` on success (always, for single player).
    pub fn net_init(&mut self, single_player: bool) -> bool {
        // Reset all per-game state (mirrors the memset block in NetInit).
        self.game_destroyed = false;
        self.somebody_won = false;
        self.clear_timeout();
        self.clear_buffers();
        self.pack_player_offsets = [0; MAX_PLRS];
        self.share_next_high_priority = true;
        self.game_loops = 0;
        self.sent_this_cycle = 0;
        self.delta_sender = 0;
        self.game_init_info = GameData::new();

        for p in &mut self.players {
            *p = MultiplayerPlayer::default();
        }

        if single_player {
            self.init_single_player();
            self.is_loopback = true;
            self.net_initialized = true;
        } else {
            // Loopback "multiplayer" used by tests: host joins as player 0.
            self.init_multiplayer("loopback", "", false);
            self.player_joined(0);
            self.is_loopback = true;
            self.net_initialized = true;
        }
        #[cfg(feature = "network")]
        {
            // Host a wire-framed loopback session (storm SNetCreateGame).
            self.storm.create_game(&self.game_name, &self.game_password, &[]);
        }
        true
    }

    /// Tear down the network layer.
    ///
    /// Port of `NetClose()` from `multi.cpp`. Flushes buffers and marks the
    /// session inactive. In single-player there is no real socket to close.
    pub fn net_close(&mut self) {
        if !self.net_initialized {
            return;
        }
        self.net_initialized = false;
        self.clear_buffers();
        for p in &mut self.players {
            p.set_connected(false);
            p.set_active(false);
        }
        self.active_players = 0;
    }

    // ------------------------------------------------------------------------
    // Sending
    // ------------------------------------------------------------------------

    /// Send a single packet to a player.
    ///
    /// Port of `SendPacket(playerId, packet, size)` from `multi.cpp`. In
    /// single-player the packet is simply appended to the receive log so
    /// tests can inspect what would have been transmitted.
    pub fn send_packet(&mut self, player_id: u8, data: &[u8]) -> bool {
        if !self.net_initialized {
            return false;
        }
        if player_id as usize >= MAX_PLRS {
            return false;
        }
        #[cfg(feature = "network")]
        if self.is_loopback {
            // Frame the command as a dvlnet PT_MESSAGE wire packet and send it
            // through the loopback transport (storm SNetSendMessage).
            self.sent_this_cycle += 1;
            return self.storm.send_message(player_id, data);
        }
        let pkt = Packet::from_body(data);
        self.sent_log.push(pkt);
        self.sent_this_cycle += 1;
        true
    }

    /// Send a low-priority command packet.
    ///
    /// Port of `NetSendLoPri(playerId, data, size)`: copies into the low-pri
    /// buffer (for later batched re-send) and fires off an immediate copy.
    pub fn net_send_lo_pri(&mut self, player_id: u8, data: &[u8]) -> bool {
        if data.is_empty() {
            return false;
        }
        let _ = self.low_priority_buffer.copy_packet(data);
        self.send_packet(player_id, data)
    }

    /// Send a high-priority command packet.
    ///
    /// Port of `NetSendHiPri(playerId, data, size)`.
    pub fn net_send_hi_pri(&mut self, player_id: u8, data: &[u8]) -> bool {
        if data.is_empty() {
            return false;
        }
        let _ = self.high_priority_buffer.copy_packet(data);
        self.send_packet(player_id, data)
    }

    /// Build and send a typed network command.
    ///
    /// Conceptual port of the various `NetSendCmd*` helpers: encodes the
    /// command byte plus an opaque payload.
    pub fn multi_send_cmd(&mut self, player_id: u8, cmd: NetCmd, payload: &[u8]) -> bool {
        let mut body = Vec::with_capacity(1 + payload.len());
        body.push(cmd as u8);
        body.extend_from_slice(payload);
        self.send_packet(player_id, &body)
    }

    // ------------------------------------------------------------------------
    // Receiving / parsing
    // ------------------------------------------------------------------------

    /// Drain the sent log, returning the packets that were "received".
    ///
    /// Single-player loopback model: every sent packet is immediately
    /// receivable on the next call. Mirrors the `tmsg_get` polling loop in
    /// `ProcessTmsgs`.
    pub fn recv_packet(&mut self) -> Option<Packet> {
        #[cfg(feature = "network")]
        if self.is_loopback {
            // Decode the next wire packet from the loopback transport and
            // present it as a game packet (storm SNetReceiveMessage).
            return self
                .storm
                .receive_message()
                .map(|(_, payload)| Packet::from_body(&payload));
        }
        if self.sent_log.is_empty() {
            return None;
        }
        Some(self.sent_log.remove(0))
    }

    /// Parse a single command out of a packet body.
    ///
    /// Port of the per-message slice of `HandleAllPackets` / `ParseCmd`.
    /// Returns the decoded command and the consumed byte length, or `None`
    /// if the body is too short.
    pub fn parse_packet(body: &[u8]) -> Option<(NetCmd, usize)> {
        if body.is_empty() {
            return None;
        }
        let cmd = NetCmd::from_u8(body[0])?;
        // Command byte alone counts as 1 consumed byte; callers append
        // payload-specific sizes on top.
        Some((cmd, 1))
    }

    /// Simulate receiving a full player-info update.
    ///
    /// Port of `recv_plrinfo` / `NetReceivePlayerData`: in single-player this
    /// just marks the named player slot as connected and active.
    pub fn net_receive_player(&mut self, player_id: usize) -> bool {
        if player_id >= MAX_PLRS {
            return false;
        }
        if !self.players[player_id].is_connected() {
            self.player_joined(player_id);
        }
        true
    }

    /// Synchronise a player's state for the current turn.
    ///
    /// Simplified port of the per-player slice of `multi_handle_delta` /
    /// `SyncPlayer`. Returns `true` if the player was active for this sync.
    pub fn sync_player(&mut self, player_id: usize) -> bool {
        if player_id >= MAX_PLRS {
            return false;
        }
        if !self.players[player_id].is_active() {
            return false;
        }
        self.players[player_id].has_turn = true;
        true
    }

    /// Drain all buffered high/low-priority packets into the sent log.
    ///
    /// Conceptual port of `multi_handle_delta`'s flush step. Returns the
    /// number of packets emitted.
    pub fn flush_buffers(&mut self) -> usize {
        let mut count = 0;
        while !self.high_priority_buffer.is_empty() {
            let data = self.high_priority_buffer.get_data().to_vec();
            self.sent_log.push(Packet::from_body(&data));
            self.high_priority_buffer.clear();
            count += 1;
        }
        while !self.low_priority_buffer.is_empty() {
            let data = self.low_priority_buffer.get_data().to_vec();
            self.sent_log.push(Packet::from_body(&data));
            self.low_priority_buffer.clear();
            count += 1;
        }
        count
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
// Network Packet (TPkt equivalent)
// ============================================================================

/// Network packet header (mirrors C++ `TPktHdr`).
///
/// `wLen` is the total packet length (header + body) stored little-endian.
#[derive(Debug, Clone, Copy, Default)]
pub struct PktHeader {
    /// Total packet length in bytes (header + body).
    pub w_len: u16,
}

/// Packet body capacity (matches the C++ `TPkt` body region).
pub const PKT_BODY_SIZE: usize = NET_BUFFER_SIZE;

/// A single network packet: header + body slice.
#[derive(Debug, Clone)]
pub struct Packet {
    pub header: PktHeader,
    pub body: Vec<u8>,
}

impl Packet {
    /// Build a packet from a body payload, filling in the length header.
    pub fn from_body(body: &[u8]) -> Self {
        let total_len = std::mem::size_of::<PktHeader>() + body.len();
        Self {
            header: PktHeader {
                w_len: total_len as u16,
            },
            body: body.to_vec(),
        }
    }

    /// Total wire length (header + body).
    pub fn len(&self) -> usize {
        std::mem::size_of::<PktHeader>() + self.body.len()
    }

    /// Whether the packet body is empty.
    pub fn is_empty(&self) -> bool {
        self.body.is_empty()
    }

    /// Serialise to a byte vector (LE header followed by body).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.len());
        out.extend_from_slice(&self.header.w_len.to_le_bytes());
        out.extend_from_slice(&self.body);
        out
    }

    /// Deserialise from a byte slice, returning `None` on truncation.
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < std::mem::size_of::<PktHeader>() {
            return None;
        }
        let w_len = u16::from_le_bytes([data[0], data[1]]) as usize;
        if w_len < std::mem::size_of::<PktHeader>() || w_len > data.len() {
            return None;
        }
        let body_len = w_len - std::mem::size_of::<PktHeader>();
        Some(Self {
            header: PktHeader {
                w_len: w_len as u16,
            },
            body: data[std::mem::size_of::<PktHeader>()..std::mem::size_of::<PktHeader>() + body_len]
                .to_vec(),
        })
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

    #[test]
    fn test_packet_roundtrip() {
        let pkt = Packet::from_body(&[1, 2, 3, 4]);
        assert_eq!(pkt.len(), std::mem::size_of::<PktHeader>() + 4);

        let bytes = pkt.to_bytes();
        let restored = Packet::from_bytes(&bytes).unwrap();
        assert_eq!(restored.body, vec![1, 2, 3, 4]);
        assert_eq!(restored.header.w_len, pkt.header.w_len);
    }

    #[test]
    fn test_packet_from_bytes_truncated() {
        assert!(Packet::from_bytes(&[0]).is_none());
        // Declared length shorter than header.
        assert!(Packet::from_bytes(&[0, 0]).is_none());
    }

    #[test]
    fn test_net_init_single_player() {
        let mut mgr = MultiplayerManager::new();
        assert!(mgr.net_init(true));
        assert!(mgr.net_initialized);
        assert!(mgr.is_loopback);
        assert_eq!(mgr.active_players, 1);
        assert!(mgr.is_player_active(0));
    }

    #[test]
    fn test_net_init_multi_loopback() {
        let mut mgr = MultiplayerManager::new();
        assert!(mgr.net_init(false));
        assert!(mgr.net_initialized);
        assert!(mgr.is_multiplayer);
        // Host joins as player 0.
        assert!(mgr.is_player_active(0));
    }

    #[test]
    fn test_net_close_clears_state() {
        let mut mgr = MultiplayerManager::new();
        mgr.net_init(true);
        assert!(mgr.net_initialized);
        mgr.net_close();
        assert!(!mgr.net_initialized);
        assert_eq!(mgr.active_players, 0);
    }

    #[test]
    fn test_net_close_idempotent() {
        let mut mgr = MultiplayerManager::new();
        // Closing before init is a no-op.
        mgr.net_close();
        assert!(!mgr.net_initialized);
    }

    #[test]
    fn test_send_and_recv_packet() {
        let mut mgr = MultiplayerManager::new();
        mgr.net_init(true);

        assert!(mgr.send_packet(0, &[0xAB, 0xCD]));
        let pkt = mgr.recv_packet().unwrap();
        assert_eq!(pkt.body, vec![0xAB, 0xCD]);

        // Log drained.
        assert!(mgr.recv_packet().is_none());
    }

    #[test]
    fn test_send_packet_rejects_uninit() {
        let mut mgr = MultiplayerManager::new();
        assert!(!mgr.send_packet(0, &[1]));
    }

    #[test]
    fn test_send_packet_rejects_bad_player() {
        let mut mgr = MultiplayerManager::new();
        mgr.net_init(true);
        assert!(!mgr.send_packet(99, &[1]));
    }

    #[test]
    fn test_multi_send_cmd() {
        let mut mgr = MultiplayerManager::new();
        mgr.net_init(true);

        assert!(mgr.multi_send_cmd(0, NetCmd::Walk, &[10, 20]));
        let pkt = mgr.recv_packet().unwrap();
        assert_eq!(pkt.body[0], NetCmd::Walk as u8);
        assert_eq!(&pkt.body[1..], &[10, 20]);
    }

    #[test]
    fn test_net_send_lo_hi_pri() {
        let mut mgr = MultiplayerManager::new();
        mgr.net_init(true);

        assert!(mgr.net_send_lo_pri(0, &[1, 2]));
        assert!(mgr.net_send_hi_pri(0, &[3, 4]));
        // Empty payloads are rejected.
        assert!(!mgr.net_send_lo_pri(0, &[]));
        assert!(!mgr.net_send_hi_pri(0, &[]));
    }

    #[test]
    fn test_parse_packet() {
        let (cmd, n) = MultiplayerManager::parse_packet(&[5, 0xAA]).unwrap();
        assert_eq!(cmd, NetCmd::OperateObj);
        assert_eq!(n, 1);
        assert!(MultiplayerManager::parse_packet(&[]).is_none());
        assert!(MultiplayerManager::parse_packet(&[255]).is_none());
    }

    #[test]
    fn test_net_receive_player() {
        let mut mgr = MultiplayerManager::new();
        mgr.net_init(false);
        // Player 1 not yet connected.
        assert!(!mgr.is_player_active(1));
        assert!(mgr.net_receive_player(1));
        assert!(mgr.is_player_active(1));
        // Out of range rejected.
        assert!(!mgr.net_receive_player(99));
    }

    #[test]
    fn test_sync_player() {
        let mut mgr = MultiplayerManager::new();
        mgr.net_init(true);
        assert!(mgr.sync_player(0));
        assert!(mgr.players[0].has_turn);
        // Inactive player not synced.
        assert!(!mgr.sync_player(2));
        // Out of range rejected.
        assert!(!mgr.sync_player(99));
    }

    #[test]
    fn test_flush_buffers() {
        let mut mgr = MultiplayerManager::new();
        mgr.net_init(true);
        mgr.net_send_hi_pri(0, &[1]);
        mgr.net_send_lo_pri(0, &[2]);
        let n = mgr.flush_buffers();
        assert_eq!(n, 2);
        assert!(mgr.high_priority_buffer.is_empty());
        assert!(mgr.low_priority_buffer.is_empty());
    }
    #[test]
    fn test_loopback_send_receive_through_wire_format() {
        let mut mgr = MultiplayerManager::new();
        assert!(mgr.net_init(false)); // loopback multiplayer

        let cmd = [0x01u8, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        assert!(mgr.send_packet(0, &cmd));
        assert_eq!(mgr.sent_this_cycle, 1);

        let pkt = mgr.recv_packet().expect("command loops back");
        assert_eq!(pkt.body, cmd.to_vec(), "game command survives the wire frame");

        assert!(mgr.recv_packet().is_none(), "queue drained");
    }


}
