// Copyright 2025 - DevilutionX Rust Port Contributors
//
// This file is part of the DevilutionX Rust Port.
//
// DevilutionX Rust Port is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! NPC/Towner System
//!
//! This module implements the complete NPC (Non-Player Character) system,
//! providing Rust equivalents for C++ Towner structures and behaviors.
//!
//! # C++ References
//! - Source/towners.h: Towner struct, _talker_id enum
//! - Source/towners.cpp: TownersData, IsTownerPresent, GetTowner
//! - Source/textdat.h: _speech_id enum
//! - Source/townerdat.hpp: TownerDataEntry
//!
//! # Architecture
//! ```text
//! TownerType enum (13 NPCs)
//!     ↓
//! TownerDataEntry (static config)
//!     ↓
//! Towner struct (runtime instance)
//!     ↓
//! TownerRegistry (global manager)
//! ```
//!
//! # Day 89: TownerType + SpeechId (300 lines, 8 tests)
//! # Day 90: Towner Struct + Availability (350 lines, 10 tests)
//! # Day 91: TownerData Config + Initialization (250 lines, 6 tests)

use crate::engine::types::Point;

// ============================================================================
// GameMode and Quest Stubs (Minimal implementation for M12)
// ============================================================================

/// Game mode (STUB for M12)
///
/// Full implementation in `game_mode.rs` module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    /// Original Diablo
    Diablo,
    /// Hellfire expansion
    Hellfire,
}

/// Quest identifier (STUB for M12)
///
/// Full implementation in `quest.rs` module
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestId {
    Butcher = 0,
    Farmer = 10,
    Girl = 11,
    // ... more quests (deferred)
}

/// Quest state (STUB for M12)
///
/// Full implementation in `quest.rs` module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestState {
    NotAvailable,
    Active,
    Done,
    HiveDone, // Special state for Farmer quest
}

impl QuestState {
    pub fn is_active(self) -> bool {
        self == QuestState::Active
    }

    pub fn is_done(self) -> bool {
        matches!(self, QuestState::Done | QuestState::HiveDone)
    }
}

// ============================================================================
// TownerType Enumeration (_talker_id)
// ============================================================================

/// NPC type identifier (C++ `_talker_id`)
///
/// Corresponds to Source/towners.h:26-38
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TownerType {
    /// Griswold - Blacksmith (shop owner)
    Smith = 0,
    /// Pepin - Healer (healing service)
    Healer = 1,
    /// Wounded Townsman - Butcher quest victim
    DeadGuy = 2,
    /// Ogden - Tavern owner (gossip)
    Tavern = 3,
    /// Deckard Cain - Storyteller (identification service)
    Story = 4,
    /// Farnham - Drunk (gossip)
    Drunk = 5,
    /// Adria - Witch (magic shop)
    Witch = 6,
    /// Gillian - Barmaid (gossip)
    Barmaid = 7,
    /// Wirt - Peg-legged boy (premium shop)
    PegBoy = 8,
    /// Cow (secret interaction)
    Cow = 9,
    /// Lester - Farmer (Hellfire quest)
    Farmer = 10,
    /// Celia - Girl (Hellfire quest)
    Girl = 11,
    /// Complete Cow Quest NPC (Hellfire)
    CowFarmer = 12,
}

impl TownerType {
    /// Total number of towner types
    pub const COUNT: usize = 13;

    /// Convert from u8 to TownerType
    ///
    /// # C++ Reference
    /// Corresponds to Source/towners.cpp:708 (GetNumTownerTypes)
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(TownerType::Smith),
            1 => Some(TownerType::Healer),
            2 => Some(TownerType::DeadGuy),
            3 => Some(TownerType::Tavern),
            4 => Some(TownerType::Story),
            5 => Some(TownerType::Drunk),
            6 => Some(TownerType::Witch),
            7 => Some(TownerType::Barmaid),
            8 => Some(TownerType::PegBoy),
            9 => Some(TownerType::Cow),
            10 => Some(TownerType::Farmer),
            11 => Some(TownerType::Girl),
            12 => Some(TownerType::CowFarmer),
            _ => None,
        }
    }

    /// Get internal name (enum name)
    pub fn name(self) -> &'static str {
        match self {
            TownerType::Smith => "Smith",
            TownerType::Healer => "Healer",
            TownerType::DeadGuy => "DeadGuy",
            TownerType::Tavern => "Tavern",
            TownerType::Story => "Story",
            TownerType::Drunk => "Drunk",
            TownerType::Witch => "Witch",
            TownerType::Barmaid => "Barmaid",
            TownerType::PegBoy => "PegBoy",
            TownerType::Cow => "Cow",
            TownerType::Farmer => "Farmer",
            TownerType::Girl => "Girl",
            TownerType::CowFarmer => "CowFarmer",
        }
    }

    /// Get display name (in-game name)
    ///
    /// # C++ Reference
    /// Corresponds to Source/towners.cpp:705 (TownerLongNames)
    pub fn display_name(self) -> &'static str {
        TOWNER_NAMES[self as usize]
    }
}

// ============================================================================
// Towner Display Names (TownerLongNames)
// ============================================================================

/// NPC display names shown in-game
///
/// Corresponds to Source/towners.cpp:705 (TownerLongNames)
const TOWNER_NAMES: [&str; TownerType::COUNT] = [
    "Griswold",             // Smith (0)
    "Pepin",                // Healer (1)
    "Wounded Townsman",     // DeadGuy (2)
    "Ogden",                // Tavern (3)
    "Deckard Cain",         // Story (4)
    "Farnham",              // Drunk (5)
    "Adria",                // Witch (6)
    "Gillian",              // Barmaid (7)
    "Wirt",                 // PegBoy (8)
    "Cow",                  // Cow (9)
    "Lester the Farmer",    // Farmer (10)
    "Celia",                // Girl (11)
    "Complete Cow Quest",   // CowFarmer (12)
];

// ============================================================================
// SpeechId Enumeration (_speech_id) - STUB
// ============================================================================

/// Dialogue/text identifier (C++ `_speech_id`)
///
/// **NOTE**: This is a STUB implementation containing only essential values.
/// Full implementation (449 enum values) deferred to `textdat.rs` module.
///
/// # C++ Reference
/// Corresponds to Source/textdat.h:17-449
#[repr(i16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeechId {
    // Quest dialogs (partial)
    King1 = 0,
    King2 = 1,
    Banner1 = 11,
    Vile1 = 22,
    Butch1 = 67,

    // Common NPC gossip (placeholder values)
    Gossip1 = 300,
    Gossip2 = 301,
    Gossip3 = 302,

    // Special states
    /// No dialogue (C++ TEXT_NONE = -1)
    None = -1,
}

impl SpeechId {
    /// Convert from i16 to SpeechId
    ///
    /// Returns None for unknown values (not yet implemented)
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            -1 => Some(SpeechId::None),
            0 => Some(SpeechId::King1),
            1 => Some(SpeechId::King2),
            11 => Some(SpeechId::Banner1),
            22 => Some(SpeechId::Vile1),
            67 => Some(SpeechId::Butch1),
            300 => Some(SpeechId::Gossip1),
            301 => Some(SpeechId::Gossip2),
            302 => Some(SpeechId::Gossip3),
            _ => None, // Unknown speech ID (not implemented)
        }
    }
}

// ============================================================================
// Towner Struct (runtime instance)
// ============================================================================

/// NPC instance (C++ `Towner` struct)
///
/// Represents a spawned NPC in the game world.
///
/// # C++ Reference
/// Corresponds to Source/towners.h:48-80
#[derive(Debug, Clone)]
pub struct Towner {
    /// NPC type identifier
    pub towner_type: TownerType,
    /// Display name (in-game)
    pub name: String,
    /// Tile position in town
    pub position: Point,
    /// Current gossip/dialogue text ID
    pub gossip: SpeechId,

    // Animation state
    /// Sprite width (for rendering offset)
    pub anim_width: u16,
    /// Delay between animation frames (in game ticks)
    pub anim_delay: i16,
    /// Current animation tick counter
    pub anim_cnt: i16,
    /// Total frames in current animation
    pub anim_len: u8,
    /// Current frame index
    pub anim_frame: u8,
    /// Frame counter (for multi-frame sprites)
    pub anim_frame_cnt: u8,

    // Deferred fields (require other modules)
    // pub anim: Option<ClxSpriteList>,        // Requires animation module
    // pub anim_order: Vec<u8>,                // Requires animation module
    // pub talk_fn: fn(&mut Player, &mut Towner), // Requires dialogue module
}

impl Towner {
    /// Create new NPC instance
    ///
    /// # Arguments
    /// * `towner_type` - NPC type
    /// * `position` - Tile position in town
    /// * `gossip` - Initial gossip text ID
    pub fn new(
        towner_type: TownerType,
        position: Point,
        gossip: SpeechId,
    ) -> Self {
        let name = towner_type.display_name().to_string();

        Self {
            towner_type,
            name,
            position,
            gossip,
            anim_width: 96,  // Default sprite width
            anim_delay: 3,   // Default animation delay
            anim_cnt: 0,
            anim_len: 16,    // Default frame count
            anim_frame: 0,
            anim_frame_cnt: 0,
        }
    }

    /// Get current sprite index for rendering
    ///
    /// # C++ Reference
    /// Corresponds to Towner::currentSprite() (Source/towners.h:76)
    pub fn current_sprite_index(&self) -> usize {
        self.anim_frame as usize
    }

    /// Update animation state (advance frame)
    pub fn update_animation(&mut self) {
        self.anim_cnt += 1;

        if self.anim_cnt >= self.anim_delay {
            self.anim_cnt = 0;
            self.anim_frame = (self.anim_frame + 1) % self.anim_len;
        }
    }
}

// ============================================================================
// Towner Availability Logic (IsTownerPresent)
// ============================================================================

/// Check if NPC is available in current game state
///
/// # C++ Reference
/// Corresponds to Source/towners.cpp:714-729 (IsTownerPresent)
pub fn is_towner_available(
    towner_type: TownerType,
    game_mode: GameMode,
    quests: &[QuestState],
    player_visited_levels: &[bool],
) -> bool {
    match towner_type {
        TownerType::DeadGuy => {
            // Butcher quest: active but not done
            if quests.len() <= QuestId::Butcher as usize {
                return false;
            }
            let quest_state = quests[QuestId::Butcher as usize];
            quest_state.is_active() && !quest_state.is_done()
        }
        TownerType::Farmer => {
            // Hellfire + Hive quest not done + no cow quest
            if game_mode != GameMode::Hellfire {
                return false;
            }
            if quests.len() <= QuestId::Farmer as usize {
                return false;
            }
            let farmer_quest = quests[QuestId::Farmer as usize];
            farmer_quest != QuestState::HiveDone
            // Note: bCowQuest check deferred (requires GameInitInfo)
        }
        TownerType::CowFarmer => {
            // Hellfire + cow quest active
            game_mode == GameMode::Hellfire
            // Note: bCowQuest check deferred
        }
        TownerType::Girl => {
            // Hellfire + Theo quest + visited Hell (level 17) + quest not done
            if game_mode != GameMode::Hellfire {
                return false;
            }
            if player_visited_levels.len() <= 17 {
                return false;
            }
            if quests.len() <= QuestId::Girl as usize {
                return false;
            }
            let girl_quest = quests[QuestId::Girl as usize];
            player_visited_levels[17] && !girl_quest.is_done()
            // Note: bTheoQuest check deferred
        }
        _ => true, // All other NPCs always available
    }
}

// ============================================================================
// TownerRegistry (global NPC manager)
// ============================================================================

/// Global NPC registry
///
/// Manages all spawned NPCs in the current game.
///
/// # C++ Reference
/// Corresponds to Source/towners.h:86 (std::vector<Towner> Towners)
#[derive(Debug, Clone, Default)]
pub struct TownerRegistry {
    towners: Vec<Towner>,
}

impl TownerRegistry {
    /// Create empty registry
    pub fn new() -> Self {
        Self {
            towners: Vec::new(),
        }
    }

    /// Add NPC to registry
    pub fn add_towner(&mut self, towner: Towner) {
        self.towners.push(towner);
    }

    /// Get NPC by type (immutable)
    ///
    /// # C++ Reference
    /// Corresponds to Source/towners.cpp:731-737 (GetTowner)
    pub fn get_towner(&self, towner_type: TownerType) -> Option<&Towner> {
        self.towners.iter()
            .find(|t| t.towner_type == towner_type)
    }

    /// Get NPC by type (mutable)
    pub fn get_towner_mut(&mut self, towner_type: TownerType) -> Option<&mut Towner> {
        self.towners.iter_mut()
            .find(|t| t.towner_type == towner_type)
    }

    /// Iterate over all NPCs (immutable)
    pub fn iter(&self) -> impl Iterator<Item = &Towner> {
        self.towners.iter()
    }

    /// Iterate over all NPCs (mutable)
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Towner> {
        self.towners.iter_mut()
    }

    /// Get total NPC count
    ///
    /// # C++ Reference
    /// Corresponds to Source/towners.cpp:713 (GetNumTowners)
    pub fn count(&self) -> usize {
        self.towners.len()
    }

    /// Clear all NPCs
    pub fn clear(&mut self) {
        self.towners.clear();
    }
}

// ============================================================================
// TownerDataEntry (static configuration)
// ============================================================================

/// Static configuration data for NPC types
///
/// # C++ Reference
/// Corresponds to Source/townerdat.hpp:21-34 (TownerDataEntry)
#[derive(Debug, Clone)]
pub struct TownerDataEntry {
    pub towner_type: TownerType,
    pub name: &'static str,
    pub default_position: Point,
    pub anim_width: u16,
    pub anim_frames: u8,
    pub anim_delay: i16,
    /// CL2 sprite path from towners.tsv `animPath` (no extension; empty =
    /// no sprite).
    pub anim_path: &'static str,
    pub gossip_texts: &'static [SpeechId],
}

/// Static NPC configuration data (TOWNER_DATA)
///
/// # C++ Reference
/// Corresponds to Source/towners.cpp:682-697 (TownersData)
/// Combined with Source/townerdat.hpp (TownersDataEntries from TSV)
const TOWNER_DATA: [TownerDataEntry; TownerType::COUNT] = [
    // Smith (Griswold) - Blacksmith
    TownerDataEntry {
        towner_type: TownerType::Smith,
        name: "Griswold",
        default_position: Point { x: 62, y: 63 },
        anim_width: 96,
        anim_frames: 16,
        anim_delay: 3,
        anim_path: "towners\\smith\\smithn",
        gossip_texts: &[SpeechId::Gossip1, SpeechId::Gossip2],
    },
    // Healer (Pepin)
    TownerDataEntry {
        towner_type: TownerType::Healer,
        name: "Pepin",
        default_position: Point { x: 55, y: 79 },
        anim_width: 96,
        anim_frames: 20,
        anim_delay: 3,
        anim_path: "towners\\healer\\healer",
        gossip_texts: &[SpeechId::Gossip1, SpeechId::Gossip2],
    },
    // DeadGuy (Wounded Townsman)
    TownerDataEntry {
        towner_type: TownerType::DeadGuy,
        name: "Wounded Townsman",
        default_position: Point { x: 24, y: 32 },
        anim_width: 96,
        anim_frames: 8,
        anim_delay: 6,
        anim_path: "towners\\butch\\deadguy",
        gossip_texts: &[SpeechId::Butch1],
    },
    // Tavern (Ogden)
    TownerDataEntry {
        towner_type: TownerType::Tavern,
        name: "Ogden",
        default_position: Point { x: 55, y: 62 },
        anim_width: 96,
        anim_frames: 16,
        anim_delay: 3,
        anim_path: "towners\\twnf\\twnfn",
        gossip_texts: &[SpeechId::Gossip1, SpeechId::Gossip2],
    },
    // Story (Deckard Cain)
    TownerDataEntry {
        towner_type: TownerType::Story,
        name: "Deckard Cain",
        default_position: Point { x: 62, y: 71 },
        anim_width: 96,
        anim_frames: 18,
        anim_delay: 3,
        anim_path: "towners\\strytell\\strytell",
        gossip_texts: &[SpeechId::King1, SpeechId::King2],
    },
    // Drunk (Farnham)
    TownerDataEntry {
        towner_type: TownerType::Drunk,
        name: "Farnham",
        default_position: Point { x: 71, y: 84 },
        anim_width: 96,
        anim_frames: 16,
        anim_delay: 3,
        anim_path: "towners\\drunk\\twndrunk",
        gossip_texts: &[SpeechId::Gossip1, SpeechId::Gossip2],
    },
    // Witch (Adria)
    TownerDataEntry {
        towner_type: TownerType::Witch,
        name: "Adria",
        default_position: Point { x: 80, y: 20 },
        anim_width: 96,
        anim_frames: 18,
        anim_delay: 6,
        anim_path: "towners\\townwmn1\\witch",
        gossip_texts: &[SpeechId::Gossip1, SpeechId::Gossip2],
    },
    // Barmaid (Gillian)
    TownerDataEntry {
        towner_type: TownerType::Barmaid,
        name: "Gillian",
        default_position: Point { x: 43, y: 66 },
        anim_width: 96,
        anim_frames: 18,
        anim_delay: 3,
        anim_path: "towners\\townwmn1\\wmnn",
        gossip_texts: &[SpeechId::Gossip1, SpeechId::Gossip2],
    },
    // PegBoy (Wirt)
    TownerDataEntry {
        towner_type: TownerType::PegBoy,
        name: "Wirt",
        default_position: Point { x: 11, y: 53 },
        anim_width: 96,
        anim_frames: 16,
        anim_delay: 5,
        anim_path: "towners\\townboy\\pegkid1",
        gossip_texts: &[SpeechId::Gossip1],
    },
    // Cow
    TownerDataEntry {
        towner_type: TownerType::Cow,
        name: "Cow",
        default_position: Point { x: 58, y: 16 },
        anim_width: 128,
        anim_frames: 12,
        anim_delay: 3,
        anim_path: "",
        gossip_texts: &[SpeechId::None],
    },
    // Farmer (Lester)
    TownerDataEntry {
        towner_type: TownerType::Farmer,
        name: "Lester the Farmer",
        default_position: Point { x: 62, y: 16 },
        anim_width: 96,
        anim_frames: 15,
        anim_delay: 3,
        anim_path: "",
        gossip_texts: &[SpeechId::Gossip1],
    },
    // Girl (Celia)
    TownerDataEntry {
        towner_type: TownerType::Girl,
        name: "Celia",
        default_position: Point { x: 77, y: 43 },
        anim_width: 96,
        anim_frames: 20,
        anim_delay: 6,
        anim_path: "",
        gossip_texts: &[SpeechId::Gossip1],
    },
    // CowFarmer
    TownerDataEntry {
        towner_type: TownerType::CowFarmer,
        name: "Complete Cow Quest",
        default_position: Point { x: 61, y: 22 },
        anim_width: 96,
        anim_frames: 15,
        anim_delay: 3,
        anim_path: "",
        gossip_texts: &[SpeechId::None],
    },
];

/// CL2 sprite path for a towner type (towners.tsv `animPath`, no extension;
/// empty = no sprite in the classic set). **C++ Reference**: townerdat.hpp
/// `TownersDataEntries` + towners.cpp `LoadTownerAnimations`.
pub fn towner_anim_path(towner_type: TownerType) -> &'static str {
    TOWNER_DATA[towner_type as usize].anim_path
}

/// Sprite width for a towner type (towners.tsv `animWidth`).
pub fn towner_anim_width(towner_type: TownerType) -> u16 {
    TOWNER_DATA[towner_type as usize].anim_width
}

// ============================================================================
// TownerFactory (initialization)
// ============================================================================

/// NPC creation factory
///
/// # C++ Reference
/// Corresponds to Source/towners.cpp:739-780 (InitTowners)
pub struct TownerFactory;

impl TownerFactory {
    /// Get static data for NPC type
    pub fn get_data(towner_type: TownerType) -> &'static TownerDataEntry {
        &TOWNER_DATA[towner_type as usize]
    }

    /// Create NPC instance from type
    ///
    /// # Arguments
    /// * `towner_type` - NPC type
    /// * `position` - Optional custom position (uses default if None)
    pub fn create_towner(
        towner_type: TownerType,
        position: Option<Point>,
    ) -> Towner {
        let data = Self::get_data(towner_type);
        let pos = position.unwrap_or(data.default_position);
        let gossip = Self::random_gossip(data.gossip_texts);

        Towner {
            towner_type,
            name: data.name.to_string(),
            position: pos,
            gossip,
            anim_width: data.anim_width,
            anim_delay: data.anim_delay,
            anim_cnt: 0,
            anim_len: data.anim_frames,
            anim_frame: 0,
            anim_frame_cnt: 0,
        }
    }

    /// Select random gossip text
    ///
    /// # C++ Reference
    /// Corresponds to Source/towners.cpp:116-145 (InitTownerFromData)
    fn random_gossip(texts: &[SpeechId]) -> SpeechId {
        if texts.is_empty() {
            SpeechId::None
        } else {
            // Simplified: always pick first (full RNG deferred)
            texts[0]
        }
    }
}

// ============================================================================
// Global Initialization (InitTowners)
// ============================================================================

/// Initialize all available NPCs for current game mode
///
/// # C++ Reference
/// Corresponds to Source/towners.cpp:739-780 (InitTowners)
pub fn init_towners(
    game_mode: GameMode,
    quests: &[QuestState],
    player_visited_levels: &[bool],
) -> TownerRegistry {
    let mut registry = TownerRegistry::new();

    // Iterate all NPC types
    for towner_type_val in 0..TownerType::COUNT as u8 {
        if let Some(towner_type) = TownerType::from_u8(towner_type_val) {
            // Check availability
            if is_towner_available(
                towner_type,
                game_mode,
                quests,
                player_visited_levels,
            ) {
                // Create and add NPC
                let towner = TownerFactory::create_towner(towner_type, None);
                registry.add_towner(towner);
            }
        }
    }

    registry
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_towner_type_from_u8_valid() {
        assert_eq!(TownerType::from_u8(0), Some(TownerType::Smith));
        assert_eq!(TownerType::from_u8(1), Some(TownerType::Healer));
        assert_eq!(TownerType::from_u8(4), Some(TownerType::Story));
        assert_eq!(TownerType::from_u8(12), Some(TownerType::CowFarmer));
    }

    #[test]
    fn test_towner_type_from_u8_invalid() {
        assert_eq!(TownerType::from_u8(13), None);
        assert_eq!(TownerType::from_u8(255), None);
    }

    #[test]
    fn test_towner_type_count() {
        assert_eq!(TownerType::COUNT, 13);
    }

    #[test]
    fn test_towner_name_lookup() {
        assert_eq!(TownerType::Smith.name(), "Smith");
        assert_eq!(TownerType::Healer.name(), "Healer");
        assert_eq!(TownerType::Story.name(), "Story");
        assert_eq!(TownerType::CowFarmer.name(), "CowFarmer");
    }

    #[test]
    fn test_towner_display_name() {
        assert_eq!(TownerType::Smith.display_name(), "Griswold");
        assert_eq!(TownerType::Healer.display_name(), "Pepin");
        assert_eq!(TownerType::Story.display_name(), "Deckard Cain");
        assert_eq!(TownerType::DeadGuy.display_name(), "Wounded Townsman");
        assert_eq!(TownerType::CowFarmer.display_name(), "Complete Cow Quest");
    }

    #[test]
    fn test_speech_id_from_i16_valid() {
        assert_eq!(SpeechId::from_i16(-1), Some(SpeechId::None));
        assert_eq!(SpeechId::from_i16(0), Some(SpeechId::King1));
        assert_eq!(SpeechId::from_i16(67), Some(SpeechId::Butch1));
        assert_eq!(SpeechId::from_i16(300), Some(SpeechId::Gossip1));
    }

    #[test]
    fn test_speech_id_from_i16_invalid() {
        assert_eq!(SpeechId::from_i16(999), None); // Unknown
        assert_eq!(SpeechId::from_i16(100), None); // Not implemented
    }

    #[test]
    fn test_speech_id_none() {
        assert_eq!(SpeechId::None as i16, -1);
    }

    // ========================================================================
    // Day 90 Tests: Towner Struct + Availability
    // ========================================================================

    #[test]
    fn test_towner_new() {
        let towner = Towner::new(
            TownerType::Smith,
            Point { x: 62, y: 63 },
            SpeechId::Gossip1,
        );

        assert_eq!(towner.towner_type, TownerType::Smith);
        assert_eq!(towner.name, "Griswold");
        assert_eq!(towner.position.x, 62);
        assert_eq!(towner.position.y, 63);
        assert_eq!(towner.gossip, SpeechId::Gossip1);
        assert_eq!(towner.anim_frame, 0);
    }

    #[test]
    fn test_towner_sprite_index() {
        let mut towner = Towner::new(
            TownerType::Healer,
            Point { x: 55, y: 79 },
            SpeechId::None,
        );

        assert_eq!(towner.current_sprite_index(), 0);

        towner.anim_frame = 5;
        assert_eq!(towner.current_sprite_index(), 5);
    }

    #[test]
    fn test_towner_update_animation() {
        let mut towner = Towner::new(
            TownerType::Story,
            Point { x: 25, y: 29 },
            SpeechId::None,
        );
        towner.anim_delay = 2;
        towner.anim_len = 4;

        // First tick: no frame advance
        towner.update_animation();
        assert_eq!(towner.anim_frame, 0);
        assert_eq!(towner.anim_cnt, 1);

        // Second tick: frame advance
        towner.update_animation();
        assert_eq!(towner.anim_frame, 1);
        assert_eq!(towner.anim_cnt, 0);

        // Loop test
        towner.anim_frame = 3;
        towner.update_animation();
        towner.update_animation();
        assert_eq!(towner.anim_frame, 0); // Wrap around
    }

    #[test]
    fn test_towner_availability_always() {
        let quests = vec![QuestState::NotAvailable; 20];
        let visited = vec![false; 20];

        // Smith, Healer, Tavern, Story, Drunk, Witch, Barmaid, PegBoy, Cow
        assert!(is_towner_available(
            TownerType::Smith,
            GameMode::Diablo,
            &quests,
            &visited
        ));
        assert!(is_towner_available(
            TownerType::Healer,
            GameMode::Hellfire,
            &quests,
            &visited
        ));
        assert!(is_towner_available(
            TownerType::Story,
            GameMode::Diablo,
            &quests,
            &visited
        ));
    }

    #[test]
    fn test_towner_availability_deadguy() {
        let mut quests = vec![QuestState::NotAvailable; 20];
        let visited = vec![false; 20];

        // Quest not available → not present
        assert!(!is_towner_available(
            TownerType::DeadGuy,
            GameMode::Diablo,
            &quests,
            &visited
        ));

        // Quest active → present
        quests[QuestId::Butcher as usize] = QuestState::Active;
        assert!(is_towner_available(
            TownerType::DeadGuy,
            GameMode::Diablo,
            &quests,
            &visited
        ));

        // Quest done → not present
        quests[QuestId::Butcher as usize] = QuestState::Done;
        assert!(!is_towner_available(
            TownerType::DeadGuy,
            GameMode::Diablo,
            &quests,
            &visited
        ));
    }

    #[test]
    fn test_towner_availability_farmer() {
        let mut quests = vec![QuestState::NotAvailable; 20];
        let visited = vec![false; 20];

        // Diablo mode → not present
        assert!(!is_towner_available(
            TownerType::Farmer,
            GameMode::Diablo,
            &quests,
            &visited
        ));

        // Hellfire + quest not done → present
        quests[QuestId::Farmer as usize] = QuestState::Active;
        assert!(is_towner_available(
            TownerType::Farmer,
            GameMode::Hellfire,
            &quests,
            &visited
        ));

        // Hellfire + hive done → not present
        quests[QuestId::Farmer as usize] = QuestState::HiveDone;
        assert!(!is_towner_available(
            TownerType::Farmer,
            GameMode::Hellfire,
            &quests,
            &visited
        ));
    }

    #[test]
    fn test_towner_availability_cowfarmer() {
        let quests = vec![QuestState::NotAvailable; 20];
        let visited = vec![false; 20];

        // Diablo → not present
        assert!(!is_towner_available(
            TownerType::CowFarmer,
            GameMode::Diablo,
            &quests,
            &visited
        ));

        // Hellfire → present (cow quest check deferred)
        assert!(is_towner_available(
            TownerType::CowFarmer,
            GameMode::Hellfire,
            &quests,
            &visited
        ));
    }

    #[test]
    fn test_towner_availability_girl() {
        let mut quests = vec![QuestState::NotAvailable; 20];
        let mut visited = vec![false; 20];

        // Diablo → not present
        assert!(!is_towner_available(
            TownerType::Girl,
            GameMode::Diablo,
            &quests,
            &visited
        ));

        // Hellfire + not visited Hell → not present
        assert!(!is_towner_available(
            TownerType::Girl,
            GameMode::Hellfire,
            &quests,
            &visited
        ));

        // Hellfire + visited Hell + quest active → present
        visited[17] = true;
        quests[QuestId::Girl as usize] = QuestState::Active;
        assert!(is_towner_available(
            TownerType::Girl,
            GameMode::Hellfire,
            &quests,
            &visited
        ));

        // Quest done → not present
        quests[QuestId::Girl as usize] = QuestState::Done;
        assert!(!is_towner_available(
            TownerType::Girl,
            GameMode::Hellfire,
            &quests,
            &visited
        ));
    }

    #[test]
    fn test_towner_registry_add() {
        let mut registry = TownerRegistry::new();
        assert_eq!(registry.count(), 0);

        let towner = Towner::new(
            TownerType::Smith,
            Point { x: 62, y: 63 },
            SpeechId::None,
        );
        registry.add_towner(towner);

        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_towner_registry_get() {
        let mut registry = TownerRegistry::new();

        let smith = Towner::new(
            TownerType::Smith,
            Point { x: 62, y: 63 },
            SpeechId::Gossip1,
        );
        let healer = Towner::new(
            TownerType::Healer,
            Point { x: 55, y: 79 },
            SpeechId::Gossip2,
        );

        registry.add_towner(smith);
        registry.add_towner(healer);

        // Get by type
        let found_smith = registry.get_towner(TownerType::Smith);
        assert!(found_smith.is_some());
        assert_eq!(found_smith.unwrap().name, "Griswold");

        let found_healer = registry.get_towner(TownerType::Healer);
        assert!(found_healer.is_some());
        assert_eq!(found_healer.unwrap().name, "Pepin");

        // Not found
        let not_found = registry.get_towner(TownerType::Witch);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_towner_registry_iter() {
        let mut registry = TownerRegistry::new();

        registry.add_towner(Towner::new(
            TownerType::Smith,
            Point { x: 62, y: 63 },
            SpeechId::None,
        ));
        registry.add_towner(Towner::new(
            TownerType::Healer,
            Point { x: 55, y: 79 },
            SpeechId::None,
        ));
        registry.add_towner(Towner::new(
            TownerType::Story,
            Point { x: 25, y: 29 },
            SpeechId::None,
        ));

        let count = registry.iter().count();
        assert_eq!(count, 3);

        let types: Vec<_> = registry.iter()
            .map(|t| t.towner_type)
            .collect();
        assert_eq!(types, vec![
            TownerType::Smith,
            TownerType::Healer,
            TownerType::Story,
        ]);
    }

    // ========================================================================
    // Day 91 Tests: TownerData Config + Initialization
    // ========================================================================

    /// towners.tsv rows (townerdat.cpp:113-114 loads position_x/y): the
    /// factory defaults must match the authoritative upstream data.
    #[test]
    fn test_towner_positions_match_tsv() {
        let expected: &[(TownerType, i32, i32)] = &[
            (TownerType::Smith, 62, 63),
            (TownerType::Healer, 55, 79),
            (TownerType::DeadGuy, 24, 32),
            (TownerType::Tavern, 55, 62),
            (TownerType::Story, 62, 71),
            (TownerType::Drunk, 71, 84),
            (TownerType::Witch, 80, 20),
            (TownerType::Barmaid, 43, 66),
            (TownerType::PegBoy, 11, 53),
            (TownerType::Cow, 58, 16),
        ];
        for (ty, x, y) in expected.iter().copied() {
            let d = TownerFactory::get_data(ty);
            assert_eq!((d.default_position.x, d.default_position.y), (x, y), "towner {:?}", ty);
        }
    }

    #[test]
    fn test_towner_data_lookup() {
        let smith_data = TownerFactory::get_data(TownerType::Smith);
        assert_eq!(smith_data.name, "Griswold");
        assert_eq!(smith_data.default_position.x, 62);
        assert_eq!(smith_data.default_position.y, 63);
        assert_eq!(smith_data.anim_width, 96);
        assert_eq!(smith_data.anim_frames, 16);
        assert_eq!(smith_data.anim_delay, 3);

        let story_data = TownerFactory::get_data(TownerType::Story);
        assert_eq!(story_data.name, "Deckard Cain");
        // towners.tsv row TOWN_STORY: position (62, 71).
        assert_eq!(story_data.default_position.x, 62);
        assert_eq!(story_data.default_position.y, 71);
    }

    #[test]
    fn test_towner_factory_create_default() {
        let towner = TownerFactory::create_towner(TownerType::Healer, None);

        assert_eq!(towner.towner_type, TownerType::Healer);
        assert_eq!(towner.name, "Pepin");
        assert_eq!(towner.position.x, 55); // Default position
        assert_eq!(towner.position.y, 79);
        assert_eq!(towner.anim_width, 96);
        assert_eq!(towner.anim_len, 20);
    }

    #[test]
    fn test_towner_factory_create_custom_position() {
        let custom_pos = Point { x: 100, y: 200 };
        let towner = TownerFactory::create_towner(
            TownerType::Witch,
            Some(custom_pos),
        );

        assert_eq!(towner.position.x, 100); // Custom position
        assert_eq!(towner.position.y, 200);
        assert_eq!(towner.name, "Adria");
    }

    #[test]
    fn test_towner_factory_gossip_selection() {
        let towner = TownerFactory::create_towner(TownerType::Story, None);

        // Should select first gossip text
        assert_eq!(towner.gossip, SpeechId::King1);
    }

    #[test]
    fn test_init_towners_diablo() {
        let quests = vec![QuestState::NotAvailable; 20];
        let visited = vec![false; 20];

        let registry = init_towners(GameMode::Diablo, &quests, &visited);

        // Diablo mode: 9 NPCs (no DeadGuy, Farmer, Girl, CowFarmer)
        // Smith, Healer, Tavern, Story, Drunk, Witch, Barmaid, PegBoy, Cow
        assert_eq!(registry.count(), 9);

        assert!(registry.get_towner(TownerType::Smith).is_some());
        assert!(registry.get_towner(TownerType::Healer).is_some());
        assert!(registry.get_towner(TownerType::Cow).is_some());
        assert!(registry.get_towner(TownerType::DeadGuy).is_none());
        assert!(registry.get_towner(TownerType::Farmer).is_none());
    }

    #[test]
    fn test_init_towners_hellfire() {
        let mut quests = vec![QuestState::NotAvailable; 20];
        let mut visited = vec![false; 20];

        // Setup for Girl to appear
        visited[17] = true;
        quests[QuestId::Girl as usize] = QuestState::Active;
        quests[QuestId::Farmer as usize] = QuestState::Active;

        let registry = init_towners(GameMode::Hellfire, &quests, &visited);

        // Hellfire can have up to 13 NPCs
        // DeadGuy requires Butcher quest, so not present
        // CowFarmer always present in Hellfire (cow quest check deferred)
        assert!(registry.count() >= 10); // At least 10 NPCs

        assert!(registry.get_towner(TownerType::Farmer).is_some());
        assert!(registry.get_towner(TownerType::Girl).is_some());
        assert!(registry.get_towner(TownerType::CowFarmer).is_some());
    }
}
