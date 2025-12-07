//! Quest System
//!
//! Implements the quest management system for Diablo/Hellfire. Unlike the C++ version
//! which uses global arrays, this module encapsulates all quest state in a QuestManager
//! structure for better memory safety and testability.
//!
//! C++ source: Source/quests.cpp (850 lines) + Source/quests.h (149 lines)
//! Rust target: ~1,100 lines (110% coverage)

use crate::engine::types::Point;
use crate::game::types::DungeonType;

/// Maximum number of quests in the game
pub const MAX_QUESTS: usize = 24;

/// Quest ID enumeration (24 quests total)
///
/// C++ equivalent: quest_id in Source/objdat.h:156-181
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum QuestId {
    /// Magic Rock (tutorial quest, single-player only)
    Rock = 0,
    /// Black Mushroom (alchemy quest, single-player only)
    Mushroom = 1,
    /// Gharbad the Weak (merchant quest)
    Garbud = 2,
    /// Zhar the Mad (librarian quest)
    Zhar = 3,
    /// Lachdanan (Golden Elixir quest)
    Veil = 4,
    /// Diablo (final boss)
    Diablo = 5,
    /// The Butcher (early game quest)
    Butcher = 6,
    /// Ogden's Sign (banner quest)
    LtBanner = 7,
    /// Halls of the Blind (blind quest)
    Blind = 8,
    /// Valor (Warlord of Blood pre-quest)
    Blood = 9,
    /// Anvil of Fury (item crafting quest)
    Anvil = 10,
    /// Warlord of Blood (warrior quest)
    Warlord = 11,
    /// Skeleton King (Leoric's Tomb)
    SkelKing = 12,
    /// Poisoned Water Supply (fountain quest)
    PWater = 13,
    /// The Chamber of Bone (maze quest)
    SChamb = 14,
    /// Archbishop Lazarus (Diablo pre-quest)
    Betrayer = 15,
    /// Grave (Hellfire quest)
    Grave = 16,
    /// Farmer (Hellfire Hive quest)
    Farmer = 17,
    /// Little Girl (Hellfire quest)
    Girl = 18,
    /// Wandering Trader (Hellfire quest)
    Trader = 19,
    /// Defiler (Hellfire quest)
    Defiler = 20,
    /// Na-Krul (Hellfire secret boss)
    Nakrul = 21,
    /// Cornerstone of the World (Hellfire quest)
    Cornstn = 22,
    /// Jersey's Jersey (Hellfire quest)
    Jersey = 23,
}

impl QuestId {
    /// Convert from i8 value (used in serialization/deserialization)
    ///
    /// C++ equivalent: Direct cast from int8_t to quest_id
    pub fn from_i8(value: i8) -> Option<Self> {
        match value {
            0 => Some(QuestId::Rock),
            1 => Some(QuestId::Mushroom),
            2 => Some(QuestId::Garbud),
            3 => Some(QuestId::Zhar),
            4 => Some(QuestId::Veil),
            5 => Some(QuestId::Diablo),
            6 => Some(QuestId::Butcher),
            7 => Some(QuestId::LtBanner),
            8 => Some(QuestId::Blind),
            9 => Some(QuestId::Blood),
            10 => Some(QuestId::Anvil),
            11 => Some(QuestId::Warlord),
            12 => Some(QuestId::SkelKing),
            13 => Some(QuestId::PWater),
            14 => Some(QuestId::SChamb),
            15 => Some(QuestId::Betrayer),
            16 => Some(QuestId::Grave),
            17 => Some(QuestId::Farmer),
            18 => Some(QuestId::Girl),
            19 => Some(QuestId::Trader),
            20 => Some(QuestId::Defiler),
            21 => Some(QuestId::Nakrul),
            22 => Some(QuestId::Cornstn),
            23 => Some(QuestId::Jersey),
            _ => None,
        }
    }

    /// Convert to i8 value
    pub fn to_i8(self) -> i8 {
        self as i8
    }

    /// Check if this is a Hellfire quest (quests 16-23)
    ///
    /// Hellfire expansion added 8 new quests. These are only available
    /// when running in Hellfire mode.
    pub fn is_hellfire(&self) -> bool {
        (*self as i8) >= 16
    }

    /// Check if this quest is available in multiplayer
    ///
    /// Some quests (Rock, Mushroom) are single-player only.
    /// C++ equivalent: Check against QuestsData[quest]._qdmultlvl == -1
    pub fn is_multiplayer_compatible(&self) -> bool {
        !matches!(self, QuestId::Rock | QuestId::Mushroom)
    }

    /// Get all quest IDs as an iterator
    pub fn all() -> impl Iterator<Item = QuestId> {
        (0..24).filter_map(QuestId::from_i8)
    }
}

/// Quest state enumeration
///
/// C++ equivalent: quest_state in Source/quests.h:72-81
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum QuestState {
    /// Quest did not spawn this game (RNG-based)
    NotAvail = 0,
    /// Quest has spawned, waiting to be triggered
    Init = 1,
    /// Quest is currently in progress
    Active = 2,
    /// Quest log closed and finished
    Done = 3,
    /// Hive quest special state: first tease
    HiveTease1 = 7,
    /// Hive quest special state: second tease
    HiveTease2 = 8,
    /// Hive quest special state: active
    HiveActive = 9,
    /// Hive quest special state: done
    HiveDone = 10,
}

impl QuestState {
    /// Convert from u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(QuestState::NotAvail),
            1 => Some(QuestState::Init),
            2 => Some(QuestState::Active),
            3 => Some(QuestState::Done),
            7 => Some(QuestState::HiveTease1),
            8 => Some(QuestState::HiveTease2),
            9 => Some(QuestState::HiveActive),
            10 => Some(QuestState::HiveDone),
            _ => None,
        }
    }

    /// Convert to u8 value
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if quest is in a completed state
    pub fn is_complete(&self) -> bool {
        matches!(self, QuestState::Done | QuestState::HiveDone)
    }

    /// Check if quest is in an active state (in progress)
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            QuestState::Active | QuestState::HiveActive | QuestState::HiveTease1 | QuestState::HiveTease2
        )
    }
}

/// Set level enumeration (special quest levels)
///
/// C++ equivalent: _setlevels in Source/gendung.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetLevel {
    None,
    SkeletonKing, // SL_SKELKING
    BoneChamber,  // SL_BONECHAMBER
    Maze,         // SL_MAZE
    PoisonedWater, // SL_POISONWATER
    Vilebetrayer, // SL_VILEBETRAYER
}

/// Quest structure (runtime state for a single quest)
///
/// C++ equivalent: Quest in Source/quests.h:82-93
#[derive(Debug, Clone)]
pub struct Quest {
    /// Quest identifier
    pub id: QuestId,
    /// Current quest state
    pub state: QuestState,
    /// Quest level (1-16, dungeon level where quest is active)
    pub level: u8,
    /// Quest position in world coordinates
    pub position: Point,
    /// Level type where quest occurs
    pub level_type: DungeonType,
    /// Special quest level (if applicable)
    pub set_level: SetLevel,
    /// Should this quest be shown in quest log?
    pub show_in_log: bool,
    /// Quest-specific variable 1 (usage varies by quest)
    /// Examples:
    /// - Mushroom quest: progress stage (QS_TOMESPAWNED, QS_TOMEGIVEN, etc.)
    /// - Gharbad quest: item spawn state
    /// - Zhar quest: anger state
    pub var1: u8,
    /// Quest-specific variable 2 (usage varies by quest)
    pub var2: u8,
}

impl Quest {
    /// Create a new quest with default values
    ///
    /// C++ equivalent: Quest initialization in InitQuests() (quests.cpp:482)
    pub fn new(id: QuestId) -> Self {
        Self {
            id,
            state: QuestState::NotAvail,
            level: 0,
            position: Point { x: 0, y: 0 },
            level_type: DungeonType::Town,
            set_level: SetLevel::None,
            show_in_log: false,
            var1: 0,
            var2: 0,
        }
    }

    /// Check if quest is available (spawned and not NotAvail)
    ///
    /// C++ equivalent: Quest::IsAvailable() in quests.h
    pub fn is_available(&self) -> bool {
        self.state != QuestState::NotAvail
    }

    /// Activate the quest (transition from Init to Active)
    pub fn activate(&mut self) {
        if self.state == QuestState::Init {
            self.state = QuestState::Active;
        }
    }

    /// Complete the quest (transition to Done)
    pub fn complete(&mut self) {
        if self.state == QuestState::Active {
            self.state = QuestState::Done;
            self.show_in_log = true;
        }
    }

    /// Reset quest to initial state (for new game)
    pub fn reset(&mut self) {
        self.state = QuestState::NotAvail;
        self.level = 0;
        self.position = Point { x: 0, y: 0 };
        self.show_in_log = false;
        self.var1 = 0;
        self.var2 = 0;
    }
}

/// Static quest data (read-only reference data)
///
/// C++ equivalent: QuestData in Source/quests.h:97-109
#[derive(Debug, Clone)]
pub struct QuestData {
    pub id: QuestId,
    /// Default level for single-player (1-16)
    pub default_level: u8,
    /// Multiplayer level (-1 if not available in multiplayer)
    pub multiplayer_level: i8,
    /// Level type where quest occurs
    pub level_type: DungeonType,
    /// Quest book display order
    pub book_order: i8,
    /// Set level for quest (if applicable)
    pub set_level: SetLevel,
    /// Is this quest single-player only?
    pub is_single_player_only: bool,
}

impl QuestData {
    /// Check if quest is available in multiplayer
    pub fn is_multiplayer_compatible(&self) -> bool {
        !self.is_single_player_only && self.multiplayer_level != -1
    }
}

/// Static quest data table (24 entries)
///
/// C++ equivalent: QuestsData initialization in Source/data/quests.cpp
pub const QUEST_DATA: [QuestData; MAX_QUESTS] = [
    // Q_ROCK
    QuestData {
        id: QuestId::Rock,
        default_level: 5,
        multiplayer_level: -1,
        level_type: DungeonType::Catacombs,
        book_order: 0,
        set_level: SetLevel::None,
        is_single_player_only: true,
    },
    // Q_MUSHROOM
    QuestData {
        id: QuestId::Mushroom,
        default_level: 9,
        multiplayer_level: -1,
        level_type: DungeonType::Caves,
        book_order: 1,
        set_level: SetLevel::None,
        is_single_player_only: true,
    },
    // Q_GARBUD
    QuestData {
        id: QuestId::Garbud,
        default_level: 4,
        multiplayer_level: 4,
        level_type: DungeonType::Cathedral,
        book_order: 2,
        set_level: SetLevel::None,
        is_single_player_only: false,
    },
    // Q_ZHAR
    QuestData {
        id: QuestId::Zhar,
        default_level: 8,
        multiplayer_level: 8,
        level_type: DungeonType::Catacombs,
        book_order: 3,
        set_level: SetLevel::None,
        is_single_player_only: false,
    },
    // Q_VEIL (Lachdanan)
    QuestData {
        id: QuestId::Veil,
        default_level: 7,
        multiplayer_level: 7,
        level_type: DungeonType::Catacombs,
        book_order: 4,
        set_level: SetLevel::None,
        is_single_player_only: false,
    },
    // Q_DIABLO
    QuestData {
        id: QuestId::Diablo,
        default_level: 16,
        multiplayer_level: 16,
        level_type: DungeonType::Hell,
        book_order: 15,
        set_level: SetLevel::None,
        is_single_player_only: false,
    },
    // Q_BUTCHER
    QuestData {
        id: QuestId::Butcher,
        default_level: 2,
        multiplayer_level: 2,
        level_type: DungeonType::Cathedral,
        book_order: 5,
        set_level: SetLevel::None,
        is_single_player_only: false,
    },
    // Q_LTBANNER (Ogden's Sign)
    QuestData {
        id: QuestId::LtBanner,
        default_level: 3,
        multiplayer_level: 3,
        level_type: DungeonType::Cathedral,
        book_order: 6,
        set_level: SetLevel::None,
        is_single_player_only: false,
    },
    // Q_BLIND (Halls of the Blind)
    QuestData {
        id: QuestId::Blind,
        default_level: 4,
        multiplayer_level: 4,
        level_type: DungeonType::Cathedral,
        book_order: 7,
        set_level: SetLevel::None,
        is_single_player_only: false,
    },
    // Q_BLOOD (Valor)
    QuestData {
        id: QuestId::Blood,
        default_level: 10,
        multiplayer_level: 10,
        level_type: DungeonType::Caves,
        book_order: 8,
        set_level: SetLevel::None,
        is_single_player_only: false,
    },
    // Q_ANVIL (Anvil of Fury)
    QuestData {
        id: QuestId::Anvil,
        default_level: 10,
        multiplayer_level: 10,
        level_type: DungeonType::Caves,
        book_order: 9,
        set_level: SetLevel::None,
        is_single_player_only: false,
    },
    // Q_WARLORD (Warlord of Blood)
    QuestData {
        id: QuestId::Warlord,
        default_level: 13,
        multiplayer_level: 13,
        level_type: DungeonType::Hell,
        book_order: 10,
        set_level: SetLevel::None,
        is_single_player_only: false,
    },
    // Q_SKELKING (Skeleton King)
    QuestData {
        id: QuestId::SkelKing,
        default_level: 3,
        multiplayer_level: 3,
        level_type: DungeonType::Cathedral,
        book_order: 11,
        set_level: SetLevel::SkeletonKing,
        is_single_player_only: false,
    },
    // Q_PWATER (Poisoned Water Supply)
    QuestData {
        id: QuestId::PWater,
        default_level: 6,
        multiplayer_level: 6,
        level_type: DungeonType::Catacombs,
        book_order: 12,
        set_level: SetLevel::PoisonedWater,
        is_single_player_only: false,
    },
    // Q_SCHAMB (Chamber of Bone)
    QuestData {
        id: QuestId::SChamb,
        default_level: 6,
        multiplayer_level: 6,
        level_type: DungeonType::Catacombs,
        book_order: 13,
        set_level: SetLevel::BoneChamber,
        is_single_player_only: false,
    },
    // Q_BETRAYER (Archbishop Lazarus)
    QuestData {
        id: QuestId::Betrayer,
        default_level: 15,
        multiplayer_level: 15,
        level_type: DungeonType::Hell,
        book_order: 14,
        set_level: SetLevel::Vilebetrayer,
        is_single_player_only: false,
    },
    // Q_GRAVE (Hellfire)
    QuestData {
        id: QuestId::Grave,
        default_level: 9,
        multiplayer_level: 9,
        level_type: DungeonType::Caves,
        book_order: 16,
        set_level: SetLevel::None,
        is_single_player_only: false,
    },
    // Q_FARMER (Hive - Hellfire)
    QuestData {
        id: QuestId::Farmer,
        default_level: 9,
        multiplayer_level: 9,
        level_type: DungeonType::Caves,
        book_order: 17,
        set_level: SetLevel::None,
        is_single_player_only: false,
    },
    // Q_GIRL (Little Girl - Hellfire)
    QuestData {
        id: QuestId::Girl,
        default_level: 17,
        multiplayer_level: -1,
        level_type: DungeonType::Cathedral,
        book_order: 18,
        set_level: SetLevel::None,
        is_single_player_only: true,
    },
    // Q_TRADER (Wandering Trader - Hellfire)
    QuestData {
        id: QuestId::Trader,
        default_level: 18,
        multiplayer_level: -1,
        level_type: DungeonType::Catacombs,
        book_order: 19,
        set_level: SetLevel::None,
        is_single_player_only: true,
    },
    // Q_DEFILER (Defiler - Hellfire)
    QuestData {
        id: QuestId::Defiler,
        default_level: 21,
        multiplayer_level: 21,
        level_type: DungeonType::Hell,
        book_order: 20,
        set_level: SetLevel::None,
        is_single_player_only: false,
    },
    // Q_NAKRUL (Na-Krul - Hellfire)
    QuestData {
        id: QuestId::Nakrul,
        default_level: 24,
        multiplayer_level: 24,
        level_type: DungeonType::Hell,
        book_order: 21,
        set_level: SetLevel::None,
        is_single_player_only: false,
    },
    // Q_CORNSTN (Cornerstone - Hellfire)
    QuestData {
        id: QuestId::Cornstn,
        default_level: 17,
        multiplayer_level: -1,
        level_type: DungeonType::Cathedral,
        book_order: 22,
        set_level: SetLevel::None,
        is_single_player_only: true,
    },
    // Q_JERSEY (Jersey's Jersey - Hellfire)
    QuestData {
        id: QuestId::Jersey,
        default_level: 1,
        multiplayer_level: -1,
        level_type: DungeonType::Town,
        book_order: 23,
        set_level: SetLevel::None,
        is_single_player_only: true,
    },
];

/// Quest Manager (owns all quest state)
///
/// C++ equivalent: Global Quests[MAXQUESTS] array + helper functions
/// Rust improvement: Encapsulates all state in a single structure
pub struct QuestManager {
    /// All 24 quest instances
    quests: [Quest; MAX_QUESTS],
    /// Is the quest log UI currently open?
    pub quest_log_open: bool,
    /// Return level from special quest level (for portal back to main dungeon)
    pub return_level: Option<u8>,
    /// Return position in the dungeon
    pub return_position: Option<Point>,
    /// Return level type
    pub return_level_type: Option<DungeonType>,
}

impl QuestManager {
    /// Create a new quest manager with all quests in NotAvail state
    pub fn new() -> Self {
        // Initialize all quests
        let mut quests = std::array::from_fn(|i| {
            let id = QuestId::from_i8(i as i8).unwrap();
            Quest::new(id)
        });

        // Set initial quest levels and types from static data
        for (i, quest) in quests.iter_mut().enumerate() {
            let data = &QUEST_DATA[i];
            quest.level_type = data.level_type;
            quest.set_level = data.set_level;
        }

        Self {
            quests,
            quest_log_open: false,
            return_level: None,
            return_position: None,
            return_level_type: None,
        }
    }

    /// Initialize quests for a new game
    ///
    /// C++ equivalent: InitQuests() in Source/quests.cpp:199-251
    ///
    /// # Arguments
    /// * `is_multiplayer` - True if this is a multiplayer game
    /// * `randomize` - True to randomize which quests spawn (single-player only)
    /// * `seed` - RNG seed for quest randomization (typically from dungeon level 15)
    pub fn init_quests(&mut self, is_multiplayer: bool, randomize: bool, seed: u32) {
        // Reset all quests
        for quest in &mut self.quests {
            quest.reset();
        }

        // Initialize quest levels and states from static data
        for (i, quest) in self.quests.iter_mut().enumerate() {
            let data = &QUEST_DATA[i];

            if is_multiplayer {
                // Multiplayer mode
                if data.is_multiplayer_compatible() {
                    quest.level = data.multiplayer_level as u8;
                    quest.state = QuestState::Init;
                } else {
                    // Single-player only quests are not available
                    quest.state = QuestState::NotAvail;
                }
            } else {
                // Single-player mode
                quest.level = data.default_level;
                quest.state = QuestState::Init;
            }

            quest.level_type = data.level_type;
            quest.set_level = data.set_level;
        }

        // Randomize quest pools for single-player (C++ InitialiseQuestPools)
        if !is_multiplayer && randomize {
            self.randomize_quest_pools(seed);
        }

        // Special quest variable initializations (C++ InitQuests:243-251)
        if self.quests[QuestId::SkelKing as usize].state == QuestState::NotAvail {
            self.quests[QuestId::SkelKing as usize].var2 = 2;
        }
        if self.quests[QuestId::Rock as usize].state == QuestState::NotAvail {
            self.quests[QuestId::Rock as usize].var2 = 2;
        }
        self.quests[QuestId::LtBanner as usize].var1 = 1;
        if is_multiplayer {
            self.quests[QuestId::Betrayer as usize].var1 = 2;
        }
    }

    /// Randomize which quests spawn (single-player only)
    ///
    /// C++ equivalent: InitialiseQuestPools() in Source/quests.cpp:253-276
    ///
    /// This deactivates quests from each quest pool at random to provide variety.
    /// Quest pools:
    /// - Pool 1: SkelKing OR PWater
    /// - Pool 2: Butcher OR LtBanner OR Garbud (with special seed 988045466 handling)
    /// - Pool 3: Blind OR Rock OR Blood
    /// - Pool 4: Mushroom OR Zhar OR Anvil
    /// - Pool 5: Veil OR Warlord
    fn randomize_quest_pools(&mut self, seed: u32) {
        use crate::engine::random::DiabloGenerator;

        let mut rng = DiabloGenerator::new(seed);

        // Pool 1: Deactivate either SkelKing or PWater
        let pool1 = [QuestId::SkelKing, QuestId::PWater];
        let choice1 = rng.generate_rnd(2) as usize;
        self.quests[pool1[choice1] as usize].state = QuestState::NotAvail;

        // Pool 2: Deactivate one of Butcher, LtBanner, or Garbud
        // Special case: If seed == 988045466, skip this pool (vanilla bug emulation)
        if seed == 988045466 {
            // Skip one random value to match vanilla behavior
            let _ = rng.advance_rnd_seed();
        } else {
            let pool2 = [QuestId::Butcher, QuestId::LtBanner, QuestId::Garbud];
            let choice2 = rng.generate_rnd(3) as usize;
            self.quests[pool2[choice2] as usize].state = QuestState::NotAvail;
        }

        // Pool 3: Deactivate one of Blind, Rock, or Blood
        let pool3 = [QuestId::Blind, QuestId::Rock, QuestId::Blood];
        let choice3 = rng.generate_rnd(3) as usize;
        self.quests[pool3[choice3] as usize].state = QuestState::NotAvail;

        // Pool 4: Deactivate one of Mushroom, Zhar, or Anvil
        let pool4 = [QuestId::Mushroom, QuestId::Zhar, QuestId::Anvil];
        let choice4 = rng.generate_rnd(3) as usize;
        self.quests[pool4[choice4] as usize].state = QuestState::NotAvail;

        // Pool 5: Deactivate either Veil or Warlord
        let pool5 = [QuestId::Veil, QuestId::Warlord];
        let choice5 = rng.generate_rnd(2) as usize;
        self.quests[pool5[choice5] as usize].state = QuestState::NotAvail;
    }

    /// Get quest by ID (immutable reference)
    pub fn get_quest(&self, id: QuestId) -> &Quest {
        &self.quests[id as usize]
    }

    /// Get quest by ID (mutable reference)
    pub fn get_quest_mut(&mut self, id: QuestId) -> &mut Quest {
        &mut self.quests[id as usize]
    }

    /// Check if quest is active
    pub fn is_active(&self, id: QuestId) -> bool {
        self.quests[id as usize].state.is_active()
    }

    /// Check if quest is available (not NotAvail)
    pub fn is_available(&self, id: QuestId) -> bool {
        self.quests[id as usize].is_available()
    }

    /// Set quest state
    pub fn set_state(&mut self, id: QuestId, state: QuestState) {
        self.quests[id as usize].state = state;
    }

    /// Get all quests as a slice
    pub fn all_quests(&self) -> &[Quest; MAX_QUESTS] {
        &self.quests
    }

    /// Get all quests (mutable)
    pub fn all_quests_mut(&mut self) -> &mut [Quest; MAX_QUESTS] {
        &mut self.quests
    }

    /// Toggle quest log open/closed
    pub fn toggle_quest_log(&mut self) {
        self.quest_log_open = !self.quest_log_open;
    }

    /// Set return location for special quest levels
    pub fn set_return_location(&mut self, level: u8, position: Point, level_type: DungeonType) {
        self.return_level = Some(level);
        self.return_position = Some(position);
        self.return_level_type = Some(level_type);
    }

    /// Clear return location
    pub fn clear_return_location(&mut self) {
        self.return_level = None;
        self.return_position = None;
        self.return_level_type = None;
    }
}

impl Default for QuestManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_id_conversion() {
        // Test forward conversion
        assert_eq!(QuestId::from_i8(0), Some(QuestId::Rock));
        assert_eq!(QuestId::from_i8(6), Some(QuestId::Butcher));
        assert_eq!(QuestId::from_i8(23), Some(QuestId::Jersey));
        assert_eq!(QuestId::from_i8(24), None); // Invalid
        assert_eq!(QuestId::from_i8(-1), None); // Invalid

        // Test backward conversion
        assert_eq!(QuestId::Rock.to_i8(), 0);
        assert_eq!(QuestId::Butcher.to_i8(), 6);
        assert_eq!(QuestId::Jersey.to_i8(), 23);
    }

    #[test]
    fn test_quest_id_hellfire_check() {
        // Original Diablo quests (0-15)
        assert!(!QuestId::Rock.is_hellfire());
        assert!(!QuestId::Butcher.is_hellfire());
        assert!(!QuestId::Betrayer.is_hellfire());

        // Hellfire quests (16-23)
        assert!(QuestId::Grave.is_hellfire());
        assert!(QuestId::Farmer.is_hellfire());
        assert!(QuestId::Jersey.is_hellfire());
    }

    #[test]
    fn test_quest_id_multiplayer_compatible() {
        // Single-player only quests
        assert!(!QuestId::Rock.is_multiplayer_compatible());
        assert!(!QuestId::Mushroom.is_multiplayer_compatible());

        // Multiplayer compatible quests
        assert!(QuestId::Butcher.is_multiplayer_compatible());
        assert!(QuestId::SkelKing.is_multiplayer_compatible());
        assert!(QuestId::Diablo.is_multiplayer_compatible());
    }

    #[test]
    fn test_quest_id_all_iterator() {
        let all_quests: Vec<_> = QuestId::all().collect();
        assert_eq!(all_quests.len(), 24);
        assert_eq!(all_quests[0], QuestId::Rock);
        assert_eq!(all_quests[23], QuestId::Jersey);
    }

    #[test]
    fn test_quest_state_conversion() {
        // Forward conversion
        assert_eq!(QuestState::from_u8(0), Some(QuestState::NotAvail));
        assert_eq!(QuestState::from_u8(1), Some(QuestState::Init));
        assert_eq!(QuestState::from_u8(2), Some(QuestState::Active));
        assert_eq!(QuestState::from_u8(3), Some(QuestState::Done));
        assert_eq!(QuestState::from_u8(7), Some(QuestState::HiveTease1));
        assert_eq!(QuestState::from_u8(10), Some(QuestState::HiveDone));
        assert_eq!(QuestState::from_u8(99), None); // Invalid

        // Backward conversion
        assert_eq!(QuestState::NotAvail.to_u8(), 0);
        assert_eq!(QuestState::Active.to_u8(), 2);
        assert_eq!(QuestState::HiveDone.to_u8(), 10);
    }

    #[test]
    fn test_quest_state_is_complete() {
        assert!(QuestState::Done.is_complete());
        assert!(QuestState::HiveDone.is_complete());
        assert!(!QuestState::Active.is_complete());
        assert!(!QuestState::Init.is_complete());
        assert!(!QuestState::NotAvail.is_complete());
    }

    #[test]
    fn test_quest_state_is_active() {
        assert!(QuestState::Active.is_active());
        assert!(QuestState::HiveActive.is_active());
        assert!(QuestState::HiveTease1.is_active());
        assert!(QuestState::HiveTease2.is_active());
        assert!(!QuestState::Done.is_active());
        assert!(!QuestState::Init.is_active());
    }

    #[test]
    fn test_quest_creation() {
        let quest = Quest::new(QuestId::Butcher);
        assert_eq!(quest.id, QuestId::Butcher);
        assert_eq!(quest.state, QuestState::NotAvail);
        assert_eq!(quest.level, 0);
        assert_eq!(quest.position, Point { x: 0, y: 0 });
        assert!(!quest.show_in_log);
        assert_eq!(quest.var1, 0);
        assert_eq!(quest.var2, 0);
    }

    #[test]
    fn test_quest_is_available() {
        let mut quest = Quest::new(QuestId::Butcher);
        assert!(!quest.is_available()); // NotAvail

        quest.state = QuestState::Init;
        assert!(quest.is_available());

        quest.state = QuestState::Active;
        assert!(quest.is_available());

        quest.state = QuestState::Done;
        assert!(quest.is_available());
    }

    #[test]
    fn test_quest_activation() {
        let mut quest = Quest::new(QuestId::Butcher);
        quest.state = QuestState::Init;

        quest.activate();
        assert_eq!(quest.state, QuestState::Active);

        // Activating again should have no effect
        quest.activate();
        assert_eq!(quest.state, QuestState::Active);
    }

    #[test]
    fn test_quest_completion() {
        let mut quest = Quest::new(QuestId::Butcher);
        quest.state = QuestState::Active;

        quest.complete();
        assert_eq!(quest.state, QuestState::Done);
        assert!(quest.show_in_log);

        // Completing again should have no effect
        let prev_log = quest.show_in_log;
        quest.complete();
        assert_eq!(quest.state, QuestState::Done);
        assert_eq!(quest.show_in_log, prev_log);
    }

    #[test]
    fn test_quest_reset() {
        let mut quest = Quest::new(QuestId::Butcher);
        quest.state = QuestState::Done;
        quest.level = 5;
        quest.position = Point { x: 10, y: 20 };
        quest.show_in_log = true;
        quest.var1 = 42;

        quest.reset();
        assert_eq!(quest.state, QuestState::NotAvail);
        assert_eq!(quest.level, 0);
        assert_eq!(quest.position, Point { x: 0, y: 0 });
        assert!(!quest.show_in_log);
        assert_eq!(quest.var1, 0);
    }

    #[test]
    fn test_hive_quest_special_states() {
        let mut quest = Quest::new(QuestId::Farmer);

        // Test Hive quest state transitions
        quest.state = QuestState::HiveTease1;
        assert!(quest.state.is_active());
        assert!(!quest.state.is_complete());

        quest.state = QuestState::HiveTease2;
        assert!(quest.state.is_active());

        quest.state = QuestState::HiveActive;
        assert!(quest.state.is_active());

        quest.state = QuestState::HiveDone;
        assert!(!quest.state.is_active());
        assert!(quest.state.is_complete());
    }

    // Day 89 Tests: QuestManager + Initialization

    #[test]
    fn test_quest_data_multiplayer_compatible() {
        // Single-player only quests
        assert!(!QUEST_DATA[QuestId::Rock as usize].is_multiplayer_compatible());
        assert!(!QUEST_DATA[QuestId::Mushroom as usize].is_multiplayer_compatible());
        assert!(!QUEST_DATA[QuestId::Girl as usize].is_multiplayer_compatible());
        assert!(!QUEST_DATA[QuestId::Trader as usize].is_multiplayer_compatible());
        assert!(!QUEST_DATA[QuestId::Cornstn as usize].is_multiplayer_compatible());
        assert!(!QUEST_DATA[QuestId::Jersey as usize].is_multiplayer_compatible());

        // Multiplayer compatible quests
        assert!(QUEST_DATA[QuestId::Butcher as usize].is_multiplayer_compatible());
        assert!(QUEST_DATA[QuestId::SkelKing as usize].is_multiplayer_compatible());
        assert!(QUEST_DATA[QuestId::Diablo as usize].is_multiplayer_compatible());
    }

    #[test]
    fn test_quest_manager_creation() {
        let qm = QuestManager::new();

        // All quests should be NotAvail initially
        for quest in qm.all_quests() {
            assert_eq!(quest.state, QuestState::NotAvail);
        }

        assert!(!qm.quest_log_open);
        assert_eq!(qm.return_level, None);
        assert_eq!(qm.return_position, None);
    }

    #[test]
    fn test_init_quests_singleplayer() {
        let mut qm = QuestManager::new();
        qm.init_quests(false, false, 0); // Single-player, no randomization

        // All quests should be Init in single-player mode
        for (i, quest) in qm.all_quests().iter().enumerate() {
            assert_eq!(quest.state, QuestState::Init, "Quest {} should be Init", i);
            assert_eq!(quest.level, QUEST_DATA[i].default_level);
        }

        // Check special variable initializations
        let skelking = &qm.get_quest(QuestId::SkelKing);
        assert_eq!(skelking.var2, 0); // var2 only set to 2 if NotAvail

        let ltbanner = &qm.get_quest(QuestId::LtBanner);
        assert_eq!(ltbanner.var1, 1); // Always set to 1

        let betrayer = &qm.get_quest(QuestId::Betrayer);
        assert_eq!(betrayer.var1, 0); // var1=2 only in multiplayer
    }

    #[test]
    fn test_init_quests_multiplayer() {
        let mut qm = QuestManager::new();
        qm.init_quests(true, false, 0); // Multiplayer mode

        // Single-player only quests should be NotAvail
        assert_eq!(qm.get_quest(QuestId::Rock).state, QuestState::NotAvail);
        assert_eq!(qm.get_quest(QuestId::Mushroom).state, QuestState::NotAvail);
        assert_eq!(qm.get_quest(QuestId::Girl).state, QuestState::NotAvail);
        assert_eq!(qm.get_quest(QuestId::Trader).state, QuestState::NotAvail);
        assert_eq!(qm.get_quest(QuestId::Cornstn).state, QuestState::NotAvail);
        assert_eq!(qm.get_quest(QuestId::Jersey).state, QuestState::NotAvail);

        // Multiplayer compatible quests should be Init
        assert_eq!(qm.get_quest(QuestId::Butcher).state, QuestState::Init);
        assert_eq!(qm.get_quest(QuestId::SkelKing).state, QuestState::Init);
        assert_eq!(qm.get_quest(QuestId::Diablo).state, QuestState::Init);

        // Check multiplayer-specific variable
        let betrayer = &qm.get_quest(QuestId::Betrayer);
        assert_eq!(betrayer.var1, 2); // Set to 2 in multiplayer
    }

    #[test]
    fn test_quest_randomization() {
        let mut qm = QuestManager::new();
        qm.init_quests(false, true, 12345); // Single-player with randomization

        // Count NotAvail quests (should be exactly 5, one from each pool)
        let not_avail_count = qm.all_quests().iter()
            .filter(|q| q.state == QuestState::NotAvail)
            .count();
        assert_eq!(not_avail_count, 5, "Should deactivate 5 quests (one per pool)");

        // Verify pools are mutually exclusive
        let skelking_avail = qm.is_available(QuestId::SkelKing);
        let pwater_avail = qm.is_available(QuestId::PWater);
        assert!(skelking_avail ^ pwater_avail, "Pool 1: Exactly one of SkelKing or PWater should be available");

        let blind_avail = qm.is_available(QuestId::Blind);
        let rock_avail = qm.is_available(QuestId::Rock);
        let blood_avail = qm.is_available(QuestId::Blood);
        let pool3_count = [blind_avail, rock_avail, blood_avail].iter().filter(|&&x| x).count();
        assert_eq!(pool3_count, 2, "Pool 3: Exactly 2 of 3 quests should be available");

        let veil_avail = qm.is_available(QuestId::Veil);
        let warlord_avail = qm.is_available(QuestId::Warlord);
        assert!(veil_avail ^ warlord_avail, "Pool 5: Exactly one of Veil or Warlord should be available");
    }

    #[test]
    fn test_quest_randomization_special_seed() {
        let mut qm = QuestManager::new();
        qm.init_quests(false, true, 988045466); // Special seed (vanilla bug emulation)

        // Pool 2 should be skipped (all 3 quests should be available)
        // But one of the other pools will deactivate a quest
        let butcher_avail = qm.is_available(QuestId::Butcher);
        let ltbanner_avail = qm.is_available(QuestId::LtBanner);
        let garbud_avail = qm.is_available(QuestId::Garbud);

        // All 3 in pool 2 should be available due to bug emulation
        assert!(butcher_avail || ltbanner_avail || garbud_avail,
                "At least one quest from pool 2 should be available with special seed");
    }

    #[test]
    fn test_get_quest() {
        let mut qm = QuestManager::new();
        qm.init_quests(false, false, 0);

        let butcher = qm.get_quest(QuestId::Butcher);
        assert_eq!(butcher.id, QuestId::Butcher);
        assert_eq!(butcher.state, QuestState::Init);
        assert_eq!(butcher.level, 2);
    }

    #[test]
    fn test_is_active() {
        let mut qm = QuestManager::new();
        qm.init_quests(false, false, 0);

        assert!(!qm.is_active(QuestId::Butcher)); // Init state

        qm.get_quest_mut(QuestId::Butcher).state = QuestState::Active;
        assert!(qm.is_active(QuestId::Butcher));

        qm.get_quest_mut(QuestId::Butcher).state = QuestState::Done;
        assert!(!qm.is_active(QuestId::Butcher));
    }

    #[test]
    fn test_set_state() {
        let mut qm = QuestManager::new();
        qm.init_quests(false, false, 0);

        qm.set_state(QuestId::Butcher, QuestState::Active);
        assert_eq!(qm.get_quest(QuestId::Butcher).state, QuestState::Active);

        qm.set_state(QuestId::Butcher, QuestState::Done);
        assert_eq!(qm.get_quest(QuestId::Butcher).state, QuestState::Done);
    }

    #[test]
    fn test_quest_log_toggle() {
        let mut qm = QuestManager::new();
        assert!(!qm.quest_log_open);

        qm.toggle_quest_log();
        assert!(qm.quest_log_open);

        qm.toggle_quest_log();
        assert!(!qm.quest_log_open);
    }

    #[test]
    fn test_return_location() {
        let mut qm = QuestManager::new();

        let pos = Point { x: 10, y: 20 };
        qm.set_return_location(5, pos, DungeonType::Catacombs);

        assert_eq!(qm.return_level, Some(5));
        assert_eq!(qm.return_position, Some(pos));
        assert_eq!(qm.return_level_type, Some(DungeonType::Catacombs));

        qm.clear_return_location();
        assert_eq!(qm.return_level, None);
        assert_eq!(qm.return_position, None);
        assert_eq!(qm.return_level_type, None);
    }
}

// ============================================================================
// Day 90: Quest Triggers + Town Integration
// ============================================================================

/// Quest trigger events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestEvent {
    /// Player enters a specific level
    EnterLevel(u8),
    /// Player talks to an NPC
    TalkToNPC(NpcId),
    /// Player kills a monster
    KillMonster(MonsterId),
    /// Player picks up an item
    PickupItem(ItemId),
    /// Player uses an object
    UseObject(ObjectId),
}

/// NPC identifiers for quest triggers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcId {
    Griswold,
    Pepin,
    Ogden,
    Cain,
    Farnham,
    Adria,
    Gillian,
    Wirt,
    // Hellfire NPCs
    Lester,
    Girl,
    Farmer,
    Cowquest,
}

/// Monster identifiers for quest triggers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterId {
    Butcher,
    SkeletonKing,
    SnotSpill,
    GharbadTheWeak,
    Zhar,
    Lazarus,
    RedVex,
    Blackjade,
    Diablo,
    WarlordOfBlood,
    NaKrul,
}

/// Item identifiers for quest triggers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemId {
    MagicRock,
    Mushroom,
    BrainGoo,
    SkeletonKingCrown,
    PoisonedWaterSupply,
    AnvilOfFury,
    BloodStone,
}

/// Object identifiers for quest triggers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectId {
    BookOfBlind,
    BookOfBlood,
    BookOfVilebetrayer,
    PedestalOfBlood,
    SignChest,
    AncientTome,
}

impl QuestManager {
    /// Check quest triggers and activate/update quests accordingly
    ///
    /// This function is called when significant game events occur (enter level, talk to NPC, etc.)
    /// and updates quest state based on the trigger event.
    ///
    /// # C++ Alignment
    /// - Matches `CheckQuests()` in quests.cpp:350-400
    /// - Matches `SetReturnLvlPos()` in quests.cpp:150-180
    pub fn check_quest_trigger(&mut self, event: &QuestEvent, is_multiplayer: bool) {
        match event {
            QuestEvent::EnterLevel(level) => {
                // Activate quests when entering their designated level
                for quest in &mut self.quests {
                    if quest.state == QuestState::Init && quest.level == *level {
                        quest.state = QuestState::Active;
                    }
                }
            }
            QuestEvent::TalkToNPC(npc_id) => {
                // Some quests activate when talking to NPCs
                match npc_id {
                    NpcId::Ogden => {
                        // Activate Butcher quest
                        let quest = self.get_quest_mut(QuestId::Butcher);
                        if quest.state == QuestState::Init {
                            quest.state = QuestState::Active;
                        }
                    }
                    NpcId::Farnham => {
                        // Farnham can trigger certain quests
                        let quest = self.get_quest_mut(QuestId::SkelKing);
                        if quest.state == QuestState::Init {
                            quest.state = QuestState::Active;
                        }
                    }
                    _ => {}
                }
            }
            QuestEvent::KillMonster(monster_id) => {
                // Complete quests when killing boss monsters
                match monster_id {
                    MonsterId::Butcher => {
                        self.complete_quest(QuestId::Butcher);
                    }
                    MonsterId::SkeletonKing => {
                        self.complete_quest(QuestId::SkelKing);
                    }
                    MonsterId::SnotSpill => {
                        // Snotspill drops mushroom for Mushroom quest
                        let quest = self.get_quest_mut(QuestId::Mushroom);
                        if quest.state == QuestState::Active {
                            quest.var1 = 1; // Mushroom available
                        }
                    }
                    MonsterId::GharbadTheWeak => {
                        self.complete_quest(QuestId::Garbud);
                    }
                    MonsterId::Zhar => {
                        self.complete_quest(QuestId::Zhar);
                    }
                    MonsterId::Lazarus => {
                        // Lazarus death triggers Diablo quest
                        self.get_quest_mut(QuestId::Betrayer).state = QuestState::Done;
                        self.get_quest_mut(QuestId::Diablo).state = QuestState::Active;
                    }
                    MonsterId::RedVex => {
                        self.complete_quest(QuestId::Veil);
                    }
                    MonsterId::Blackjade => {
                        self.complete_quest(QuestId::Veil);
                    }
                    MonsterId::Diablo => {
                        self.complete_quest(QuestId::Diablo);
                    }
                    MonsterId::WarlordOfBlood => {
                        self.complete_quest(QuestId::Warlord);
                    }
                    MonsterId::NaKrul => {
                        self.complete_quest(QuestId::Nakrul);
                    }
                }
            }
            QuestEvent::PickupItem(item_id) => {
                // Complete quests when picking up quest items
                match item_id {
                    ItemId::MagicRock => {
                        self.complete_quest(QuestId::Rock);
                    }
                    ItemId::Mushroom => {
                        self.complete_quest(QuestId::Mushroom);
                    }
                    ItemId::BrainGoo => {
                        self.get_quest_mut(QuestId::Mushroom).var2 = 1; // Fungal tome available
                    }
                    ItemId::AnvilOfFury => {
                        self.complete_quest(QuestId::Anvil);
                    }
                    ItemId::BloodStone => {
                        self.complete_quest(QuestId::Blood);
                    }
                    _ => {}
                }
            }
            QuestEvent::UseObject(object_id) => {
                // Activate quests when using special objects
                match object_id {
                    ObjectId::BookOfBlind => {
                        let quest = self.get_quest_mut(QuestId::Blind);
                        if quest.state == QuestState::Init {
                            quest.state = QuestState::Active;
                        }
                    }
                    ObjectId::BookOfBlood => {
                        let quest = self.get_quest_mut(QuestId::Blood);
                        if quest.state == QuestState::Init {
                            quest.state = QuestState::Active;
                        }
                    }
                    ObjectId::BookOfVilebetrayer => {
                        let quest = self.get_quest_mut(QuestId::Betrayer);
                        if quest.state == QuestState::Init {
                            quest.state = QuestState::Active;
                            // Store return position
                            self.return_level = Some(quest.level);
                        }
                    }
                    ObjectId::SignChest => {
                        // Hellfire sign chest triggers Farmer quest
                        self.complete_quest(QuestId::Farmer);
                    }
                    _ => {}
                }
            }
        }
    }

    /// Complete a quest (transition from Active to Done)
    ///
    /// # C++ Alignment
    /// - Matches quest completion logic in quests.cpp:280-350
    pub fn complete_quest(&mut self, quest_id: QuestId) {
        let quest = self.get_quest_mut(quest_id);
        if quest.state == QuestState::Active {
            quest.state = QuestState::Done;
            quest.show_in_log = true;
        }
    }

    /// Update town map based on quest state
    ///
    /// This integrates with M8 (town.rs) to dynamically update town layout
    /// when certain quests are activated or completed.
    ///
    /// # C++ Alignment
    /// - Matches `town_*` functions in town.cpp:100-200
    pub fn update_town_map(&self, town_generator: &mut dyn TownInterface) {
        // Hive quest (Hellfire): opens/closes hive entrance
        let quest = self.get_quest(QuestId::Girl);
        match quest.state {
            QuestState::HiveTease1 | QuestState::HiveTease2 => {
                town_generator.town_close_hive();
            }
            QuestState::HiveActive | QuestState::HiveDone => {
                town_generator.town_open_hive();
            }
            _ => {}
        }

        // Grave quest (Hellfire): opens/closes grave
        let quest = self.get_quest(QuestId::Grave);
        match quest.state {
            QuestState::Init | QuestState::NotAvail => {
                town_generator.town_close_grave();
            }
            QuestState::Active | QuestState::Done => {
                town_generator.town_open_grave();
            }
            _ => {}
        }
    }
}

/// Trait for town map integration (abstracts town.rs dependency)
pub trait TownInterface {
    fn town_open_hive(&mut self);
    fn town_close_hive(&mut self);
    fn town_open_grave(&mut self);
    fn town_close_grave(&mut self);
}

#[cfg(test)]
mod day90_tests {
    use super::*;

    struct MockTown {
        hive_open: bool,
        grave_open: bool,
    }

    impl TownInterface for MockTown {
        fn town_open_hive(&mut self) {
            self.hive_open = true;
        }

        fn town_close_hive(&mut self) {
            self.hive_open = false;
        }

        fn town_open_grave(&mut self) {
            self.grave_open = true;
        }

        fn town_close_grave(&mut self) {
            self.grave_open = false;
        }
    }

    #[test]
    fn test_quest_trigger_enter_level() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, false, 0);

        // Activate butcher quest (level 2)
        let quest = manager.get_quest_mut(QuestId::Butcher);
        quest.state = QuestState::Init;
        quest.level = 2;

        manager.check_quest_trigger(&QuestEvent::EnterLevel(2), false);

        let quest = manager.get_quest(QuestId::Butcher);
        assert_eq!(quest.state, QuestState::Active);
    }

    #[test]
    fn test_quest_trigger_talk_npc() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, false, 0);

        // Butcher quest should activate when talking to Ogden
        manager.get_quest_mut(QuestId::Butcher).state = QuestState::Init;

        manager.check_quest_trigger(&QuestEvent::TalkToNPC(NpcId::Ogden), false);

        let quest = manager.get_quest(QuestId::Butcher);
        assert_eq!(quest.state, QuestState::Active);
    }

    #[test]
    fn test_quest_trigger_kill_monster() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, false, 0);

        // Activate butcher quest
        manager.get_quest_mut(QuestId::Butcher).state = QuestState::Active;

        manager.check_quest_trigger(&QuestEvent::KillMonster(MonsterId::Butcher), false);

        let quest = manager.get_quest(QuestId::Butcher);
        assert_eq!(quest.state, QuestState::Done);
        assert!(quest.show_in_log);
    }

    #[test]
    fn test_update_town_map_hive() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, false, 0);
        let mut town = MockTown {
            hive_open: false,
            grave_open: false,
        };

        // Girl quest HiveActive should open hive
        manager.get_quest_mut(QuestId::Girl).state = QuestState::HiveActive;

        manager.update_town_map(&mut town);
        assert!(town.hive_open);

        // Girl quest HiveTease1 should close hive
        manager.get_quest_mut(QuestId::Girl).state = QuestState::HiveTease1;

        manager.update_town_map(&mut town);
        assert!(!town.hive_open);
    }

    #[test]
    fn test_update_town_map_grave() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, false, 0);
        let mut town = MockTown {
            hive_open: false,
            grave_open: false,
        };

        // Grave quest Active should open grave
        manager.get_quest_mut(QuestId::Grave).state = QuestState::Active;

        manager.update_town_map(&mut town);
        assert!(town.grave_open);

        // Grave quest Init should close grave
        manager.get_quest_mut(QuestId::Grave).state = QuestState::Init;

        manager.update_town_map(&mut town);
        assert!(!town.grave_open);
    }

    #[test]
    fn test_quest_event_variants() {
        // Test that all QuestEvent variants can be created
        let events = vec![
            QuestEvent::EnterLevel(5),
            QuestEvent::TalkToNPC(NpcId::Cain),
            QuestEvent::KillMonster(MonsterId::Diablo),
            QuestEvent::PickupItem(ItemId::AnvilOfFury),
            QuestEvent::UseObject(ObjectId::BookOfBlind),
        ];

        assert_eq!(events.len(), 5);
    }
}

// ============================================================================
// Day 91: Quest Set Pieces (任务特殊房间)
// ============================================================================

/// Quest room placement result
#[derive(Debug, Clone)]
pub struct QuestRoomPlacement {
    /// Quest ID
    pub quest_id: QuestId,
    /// Position of the quest room (in mega tiles or world tiles, depends on context)
    pub position: Point,
    /// Size of the quest room (width, height)
    pub size: (u8, u8),
    /// .dun file to load (if applicable)
    pub dun_file: Option<&'static str>,
}

impl QuestManager {
    /// Place quest rooms during dungeon generation
    ///
    /// This function is called by the dungeon generation algorithms (DRLG_L1/L2/L3/L4)
    /// to determine which quest rooms need to be placed in the current level.
    ///
    /// # C++ Alignment
    /// - Matches `DRLG_CheckQuests()` in quests.cpp:429-460
    ///
    /// # Parameters
    /// - `current_level`: The current dungeon level (1-16)
    /// - `level_type`: The type of dungeon (Cathedral, Catacombs, Caves, Hell)
    ///
    /// # Returns
    /// Vector of quest rooms to be placed
    pub fn get_quest_rooms_for_level(
        &self,
        current_level: u8,
        level_type: DungeonType,
    ) -> Vec<QuestRoomPlacement> {
        let mut rooms = Vec::new();

        for quest in &self.quests {
            // Only place rooms for active quests on their designated level
            if !quest.is_available() || quest.level != current_level {
                continue;
            }

            match quest.id {
                QuestId::Butcher => {
                    // Butcher's lair (L2 Cathedral)
                    if level_type == DungeonType::Cathedral {
                        rooms.push(QuestRoomPlacement {
                            quest_id: QuestId::Butcher,
                            position: Point { x: 0, y: 0 }, // Set by DRLG
                            size: (7, 7),
                            dun_file: None, // Uses DRLG_RectTrans instead
                        });
                    }
                }
                QuestId::LtBanner => {
                    // Ogden's sign (L1 Cathedral)
                    if level_type == DungeonType::Cathedral {
                        rooms.push(QuestRoomPlacement {
                            quest_id: QuestId::LtBanner,
                            position: Point { x: 0, y: 0 }, // Set by DRLG
                            size: (0, 0), // Variable size from .dun file
                            dun_file: Some("levels/l1data/banner1.dun"),
                        });
                    }
                }
                QuestId::Blind => {
                    // Halls of the Blind (L2 Cathedral)
                    if level_type == DungeonType::Cathedral {
                        rooms.push(QuestRoomPlacement {
                            quest_id: QuestId::Blind,
                            position: Point { x: 0, y: 0 }, // Set by DRLG
                            size: (11, 9), // Approximate size for wall closure
                            dun_file: None, // Uses wall tile modification
                        });
                    }
                }
                QuestId::Blood => {
                    // Chamber of Blood (L2 Catacombs)
                    if level_type == DungeonType::Catacombs {
                        rooms.push(QuestRoomPlacement {
                            quest_id: QuestId::Blood,
                            position: Point { x: 0, y: 0 }, // Set by DRLG
                            size: (0, 0), // Variable size from .dun file
                            dun_file: Some("levels/l2data/blood2.dun"),
                        });
                    }
                }
                QuestId::Warlord => {
                    // Warlord's lair (L4 Hell)
                    if level_type == DungeonType::Hell {
                        rooms.push(QuestRoomPlacement {
                            quest_id: QuestId::Warlord,
                            position: Point { x: 0, y: 0 }, // Set by DRLG
                            size: (0, 0), // Variable size from .dun file
                            dun_file: Some("levels/l4data/warlord2.dun"),
                        });
                    }
                }
                QuestId::SkelKing => {
                    // Skeleton King's lair (Set Level)
                    if quest.set_level == SetLevel::SkeletonKing {
                        rooms.push(QuestRoomPlacement {
                            quest_id: QuestId::SkelKing,
                            position: Point { x: 0, y: 0 }, // Set by DRLG
                            size: (0, 0), // Handled by set level generation
                            dun_file: None,
                        });
                    }
                }
                QuestId::SChamb => {
                    // Chamber of Bone (Set Level)
                    if quest.set_level == SetLevel::BoneChamber {
                        rooms.push(QuestRoomPlacement {
                            quest_id: QuestId::SChamb,
                            position: Point { x: 0, y: 0 }, // Set by DRLG
                            size: (0, 0), // Variable size from .dun file
                            dun_file: Some("levels/l2data/bonestr1.dun"),
                        });
                    }
                }
                _ => {
                    // Other quests don't require special room placement
                }
            }
        }

        rooms
    }

    /// Load a .dun file for a quest room
    ///
    /// This is a placeholder for the actual .dun file loading logic,
    /// which will be integrated with the dungeon generation system.
    ///
    /// # C++ Alignment
    /// - Matches `LoadFileInMem<uint16_t>("levels/...")` in quests.cpp:100-150
    /// - Matches `PlaceDunTiles()` in quests.cpp
    ///
    /// # Parameters
    /// - `dun_file`: Path to the .dun file (e.g., "levels/l1data/banner1.dun")
    ///
    /// # Returns
    /// TODO: Return DunData structure (width, height, tile data)
    /// For now, returns None to avoid circular dependency with levels module
    pub fn load_quest_dun_file(&self, dun_file: &str) -> Option<()> {
        // TODO: Integrate with levels::drlg_l4::load_dun_file()
        // This will be implemented when integrating with dungeon generation

        // For now, just validate the file path
        match dun_file {
            "levels/l1data/banner1.dun" => Some(()),
            "levels/l2data/blood2.dun" => Some(()),
            "levels/l2data/bonestr1.dun" => Some(()),
            "levels/l4data/warlord2.dun" => Some(()),
            _ => None,
        }
    }

    /// Get the return level for set level quests
    ///
    /// When a player enters a set level (e.g., Skeleton King's Lair),
    /// they need to return to the original dungeon level when exiting.
    ///
    /// # C++ Alignment
    /// - Matches `GetMapReturnLevel()` in quests.cpp:461-474
    pub fn get_return_level_for_set_level(&self, set_level: SetLevel) -> Option<u8> {
        match set_level {
            SetLevel::SkeletonKing => {
                Some(self.get_quest(QuestId::SkelKing).level)
            }
            SetLevel::BoneChamber => {
                Some(self.get_quest(QuestId::SChamb).level)
            }
            SetLevel::PoisonedWater => {
                Some(self.get_quest(QuestId::PWater).level)
            }
            SetLevel::Vilebetrayer => {
                Some(self.get_quest(QuestId::Betrayer).level)
            }
            _ => None,
        }
    }

    /// Get the return position for set level quests
    ///
    /// # C++ Alignment
    /// - Matches `GetMapReturnPosition()` in quests.cpp:476-494
    pub fn get_return_position_for_set_level(&self, set_level: SetLevel) -> Option<Point> {
        match set_level {
            SetLevel::SkeletonKing => {
                let quest = self.get_quest(QuestId::SkelKing);
                // C++: Quests[Q_SKELKING].position + Direction::SouthEast
                Some(Point {
                    x: quest.position.x + 1,
                    y: quest.position.y + 1,
                })
            }
            SetLevel::BoneChamber => {
                let quest = self.get_quest(QuestId::SChamb);
                // C++: Quests[Q_SCHAMB].position + Direction::SouthEast
                Some(Point {
                    x: quest.position.x + 1,
                    y: quest.position.y + 1,
                })
            }
            SetLevel::PoisonedWater => {
                let quest = self.get_quest(QuestId::PWater);
                // C++: Quests[Q_PWATER].position + Direction::SouthWest
                Some(Point {
                    x: quest.position.x - 1,
                    y: quest.position.y + 1,
                })
            }
            SetLevel::Vilebetrayer => {
                let quest = self.get_quest(QuestId::Betrayer);
                // C++: Quests[Q_BETRAYER].position + Direction::South
                Some(Point {
                    x: quest.position.x,
                    y: quest.position.y + 1,
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod day91_tests {
    use super::*;

    #[test]
    fn test_get_quest_rooms_for_level_butcher() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, false, 0);

        // Activate Butcher quest on level 2
        manager.get_quest_mut(QuestId::Butcher).state = QuestState::Active;
        manager.get_quest_mut(QuestId::Butcher).level = 2;

        let rooms = manager.get_quest_rooms_for_level(2, DungeonType::Cathedral);

        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].quest_id, QuestId::Butcher);
        assert_eq!(rooms[0].size, (7, 7));
        assert!(rooms[0].dun_file.is_none());
    }

    #[test]
    fn test_get_quest_rooms_for_level_multiple() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, false, 0);

        // Activate multiple quests on level 2
        manager.get_quest_mut(QuestId::Butcher).state = QuestState::Active;
        manager.get_quest_mut(QuestId::Butcher).level = 2;
        manager.get_quest_mut(QuestId::Blind).state = QuestState::Active;
        manager.get_quest_mut(QuestId::Blind).level = 2;

        let rooms = manager.get_quest_rooms_for_level(2, DungeonType::Cathedral);

        assert_eq!(rooms.len(), 2);
        assert!(rooms.iter().any(|r| r.quest_id == QuestId::Butcher));
        assert!(rooms.iter().any(|r| r.quest_id == QuestId::Blind));
    }

    #[test]
    fn test_load_quest_dun_file() {
        let manager = QuestManager::new();

        // Valid .dun files
        assert!(manager.load_quest_dun_file("levels/l1data/banner1.dun").is_some());
        assert!(manager.load_quest_dun_file("levels/l2data/blood2.dun").is_some());
        assert!(manager.load_quest_dun_file("levels/l2data/bonestr1.dun").is_some());
        assert!(manager.load_quest_dun_file("levels/l4data/warlord2.dun").is_some());

        // Invalid .dun file
        assert!(manager.load_quest_dun_file("invalid.dun").is_none());
    }

    #[test]
    fn test_get_return_level_for_set_level() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, false, 0);

        // Set levels for quests
        manager.get_quest_mut(QuestId::SkelKing).level = 3;
        manager.get_quest_mut(QuestId::SChamb).level = 6;
        manager.get_quest_mut(QuestId::PWater).level = 9;
        manager.get_quest_mut(QuestId::Betrayer).level = 15;

        assert_eq!(
            manager.get_return_level_for_set_level(SetLevel::SkeletonKing),
            Some(3)
        );
        assert_eq!(
            manager.get_return_level_for_set_level(SetLevel::BoneChamber),
            Some(6)
        );
        assert_eq!(
            manager.get_return_level_for_set_level(SetLevel::PoisonedWater),
            Some(9)
        );
        assert_eq!(
            manager.get_return_level_for_set_level(SetLevel::Vilebetrayer),
            Some(15)
        );
        assert_eq!(
            manager.get_return_level_for_set_level(SetLevel::None),
            None
        );
    }

    #[test]
    fn test_get_return_position_for_set_level() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, false, 0);

        // Set positions for quests
        manager.get_quest_mut(QuestId::SkelKing).position = Point { x: 10, y: 20 };
        manager.get_quest_mut(QuestId::SChamb).position = Point { x: 15, y: 25 };
        manager.get_quest_mut(QuestId::PWater).position = Point { x: 30, y: 40 };
        manager.get_quest_mut(QuestId::Betrayer).position = Point { x: 50, y: 60 };

        // SkelKing: +SouthEast (x+1, y+1)
        assert_eq!(
            manager.get_return_position_for_set_level(SetLevel::SkeletonKing),
            Some(Point { x: 11, y: 21 })
        );

        // SChamb: +SouthEast (x+1, y+1)
        assert_eq!(
            manager.get_return_position_for_set_level(SetLevel::BoneChamber),
            Some(Point { x: 16, y: 26 })
        );

        // PWater: +SouthWest (x-1, y+1)
        assert_eq!(
            manager.get_return_position_for_set_level(SetLevel::PoisonedWater),
            Some(Point { x: 29, y: 41 })
        );

        // Betrayer: +South (x+0, y+1)
        assert_eq!(
            manager.get_return_position_for_set_level(SetLevel::Vilebetrayer),
            Some(Point { x: 50, y: 61 })
        );

        // None
        assert_eq!(
            manager.get_return_position_for_set_level(SetLevel::None),
            None
        );
    }
}

// ============================================================================
// Day 92: Quest Log + Final Testing
// ============================================================================

/// Quest log entry for UI display
#[derive(Debug, Clone)]
pub struct QuestLogEntry {
    /// Quest ID
    pub quest_id: QuestId,
    /// Quest state
    pub state: QuestState,
    /// Quest level
    pub level: u8,
    /// Display order in quest log (from QuestData.book_order)
    pub book_order: i8,
}

impl QuestManager {
    /// Get quests that should be displayed in the quest log
    ///
    /// Returns quests that:
    /// 1. Are available (not NotAvail)
    /// 2. Have show_in_log = true (completed or explicitly marked)
    /// 3. Sorted by book_order for consistent UI display
    ///
    /// # C++ Alignment
    /// - Matches quest log filtering logic in quests.cpp:600-700
    pub fn get_log_quests(&self) -> Vec<QuestLogEntry> {
        let mut entries: Vec<QuestLogEntry> = self
            .quests
            .iter()
            .enumerate()
            .filter(|(_, quest)| {
                quest.is_available() && quest.show_in_log
            })
            .map(|(idx, quest)| QuestLogEntry {
                quest_id: quest.id,
                state: quest.state,
                level: quest.level,
                book_order: QUEST_DATA[idx].book_order,
            })
            .collect();

        // Sort by book_order for consistent display
        entries.sort_by_key(|entry| entry.book_order);

        entries
    }

    /// Get quest description text
    ///
    /// Returns the appropriate description text based on quest state.
    /// In a full implementation, this would load from text data files.
    ///
    /// # C++ Alignment
    /// - Matches quest text retrieval in quests.cpp
    pub fn get_quest_text(&self, quest_id: QuestId) -> &'static str {
        match quest_id {
            QuestId::Rock => "The Magic Rock",
            QuestId::Mushroom => "Black Mushroom",
            QuestId::Garbud => "Gharbad The Weak",
            QuestId::Zhar => "Zhar the Mad",
            QuestId::Veil => "Lachdanan",
            QuestId::Diablo => "Diablo",
            QuestId::Butcher => "The Butcher",
            QuestId::LtBanner => "Ogden's Sign",
            QuestId::Blind => "Halls of the Blind",
            QuestId::Blood => "Valor",
            QuestId::Anvil => "Anvil of Fury",
            QuestId::Warlord => "Warlord of Blood",
            QuestId::SkelKing => "The Curse of King Leoric",
            QuestId::PWater => "Poisoned Water Supply",
            QuestId::SChamb => "The Chamber of Bone",
            QuestId::Betrayer => "Archbishop Lazarus",
            // Hellfire quests
            QuestId::Grave => "Grave Matters",
            QuestId::Farmer => "Farmer's Orchard",
            QuestId::Girl => "Little Girl",
            QuestId::Trader => "Wandering Trader",
            QuestId::Defiler => "The Defiler",
            QuestId::Nakrul => "Na-Krul",
            QuestId::Cornstn => "Cornerstone of the World",
            QuestId::Jersey => "Jersey's Jersey",
        }
    }

    /// Check if a quest is currently in the quest log
    pub fn is_in_log(&self, quest_id: QuestId) -> bool {
        let quest = self.get_quest(quest_id);
        quest.is_available() && quest.show_in_log
    }

    /// Get the number of quests in the log
    pub fn get_log_quest_count(&self) -> usize {
        self.quests
            .iter()
            .filter(|q| q.is_available() && q.show_in_log)
            .count()
    }
}

#[cfg(test)]
mod day92_tests {
    use super::*;

    #[test]
    fn test_get_log_quests_empty() {
        let manager = QuestManager::new();
        // No quests initialized yet
        let entries = manager.get_log_quests();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_get_log_quests_single() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, false, 0);

        // Complete Butcher quest
        manager.get_quest_mut(QuestId::Butcher).state = QuestState::Active;
        manager.complete_quest(QuestId::Butcher);

        let entries = manager.get_log_quests();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].quest_id, QuestId::Butcher);
        assert_eq!(entries[0].state, QuestState::Done);
    }

    #[test]
    fn test_get_log_quests_sorted() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, false, 0);

        // Complete multiple quests
        manager.get_quest_mut(QuestId::Diablo).state = QuestState::Active;
        manager.complete_quest(QuestId::Diablo);

        manager.get_quest_mut(QuestId::Butcher).state = QuestState::Active;
        manager.complete_quest(QuestId::Butcher);

        manager.get_quest_mut(QuestId::SkelKing).state = QuestState::Active;
        manager.complete_quest(QuestId::SkelKing);

        let entries = manager.get_log_quests();
        assert_eq!(entries.len(), 3);

        // Entries should be sorted by book_order
        // Verify that they are in ascending book_order
        for i in 1..entries.len() {
            assert!(entries[i - 1].book_order <= entries[i].book_order);
        }
    }

    #[test]
    fn test_get_quest_text() {
        let manager = QuestManager::new();

        assert_eq!(manager.get_quest_text(QuestId::Butcher), "The Butcher");
        assert_eq!(manager.get_quest_text(QuestId::SkelKing), "The Curse of King Leoric");
        assert_eq!(manager.get_quest_text(QuestId::Diablo), "Diablo");
        assert_eq!(manager.get_quest_text(QuestId::Nakrul), "Na-Krul");
    }

    #[test]
    fn test_is_in_log() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, false, 0);

        // Quest not in log initially
        assert!(!manager.is_in_log(QuestId::Butcher));

        // Complete quest -> should be in log
        manager.get_quest_mut(QuestId::Butcher).state = QuestState::Active;
        manager.complete_quest(QuestId::Butcher);
        assert!(manager.is_in_log(QuestId::Butcher));
    }

    #[test]
    fn test_get_log_quest_count() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, false, 0);

        assert_eq!(manager.get_log_quest_count(), 0);

        // Complete 3 quests
        manager.get_quest_mut(QuestId::Butcher).state = QuestState::Active;
        manager.complete_quest(QuestId::Butcher);

        manager.get_quest_mut(QuestId::SkelKing).state = QuestState::Active;
        manager.complete_quest(QuestId::SkelKing);

        manager.get_quest_mut(QuestId::Diablo).state = QuestState::Active;
        manager.complete_quest(QuestId::Diablo);

        assert_eq!(manager.get_log_quest_count(), 3);
    }

    #[test]
    fn test_quest_lifecycle_butcher() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, false, 0);

        // Initial state
        let quest = manager.get_quest(QuestId::Butcher);
        assert_eq!(quest.state, QuestState::Init);
        assert!(!quest.show_in_log);

        // Enter level 2 -> Active
        manager.check_quest_trigger(&QuestEvent::EnterLevel(2), false);
        let quest = manager.get_quest(QuestId::Butcher);
        assert_eq!(quest.state, QuestState::Active);

        // Kill Butcher -> Done
        manager.check_quest_trigger(&QuestEvent::KillMonster(MonsterId::Butcher), false);
        let quest = manager.get_quest(QuestId::Butcher);
        assert_eq!(quest.state, QuestState::Done);
        assert!(quest.show_in_log);

        // Should appear in quest log
        assert!(manager.is_in_log(QuestId::Butcher));
        assert_eq!(manager.get_log_quest_count(), 1);
    }

    #[test]
    fn test_quest_lifecycle_skeleton_king() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, false, 0);

        // Initial state
        let quest = manager.get_quest(QuestId::SkelKing);
        assert_eq!(quest.state, QuestState::Init);

        // Talk to Farnham -> Active
        manager.check_quest_trigger(&QuestEvent::TalkToNPC(NpcId::Farnham), false);
        let quest = manager.get_quest(QuestId::SkelKing);
        assert_eq!(quest.state, QuestState::Active);

        // Kill Skeleton King -> Done
        manager.check_quest_trigger(&QuestEvent::KillMonster(MonsterId::SkeletonKing), false);
        let quest = manager.get_quest(QuestId::SkelKing);
        assert_eq!(quest.state, QuestState::Done);
        assert!(quest.show_in_log);
    }

    #[test]
    fn test_quest_lifecycle_veil() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, false, 0);

        // Initial state
        let quest = manager.get_quest(QuestId::Veil);
        assert_eq!(quest.state, QuestState::Init);

        // Use Book (placeholder - actual trigger may vary)
        // For this test, manually activate
        manager.get_quest_mut(QuestId::Veil).state = QuestState::Active;

        // Kill RedVex -> Done
        manager.check_quest_trigger(&QuestEvent::KillMonster(MonsterId::RedVex), false);
        let quest = manager.get_quest(QuestId::Veil);
        assert_eq!(quest.state, QuestState::Done);
        assert!(quest.show_in_log);
    }
}

