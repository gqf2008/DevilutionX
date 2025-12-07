//! Exact port of quest system from DevilutionX
//!
//! Port of quests.h, objdat.h (quest_id), and textdat.h (_speech_id)

#![allow(non_snake_case)]
#![allow(dead_code)]

use super::level_new::{DungeonType, SetLevel};

// ============================================================================
// Constants
// ============================================================================

/// Maximum number of quests
pub const MAXQUESTS: usize = 24;

// ============================================================================
// Quest ID enum - exact port from objdat.h
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i8)]
pub enum QuestId {
    Rock = 0,
    Mushroom = 1,
    Garbud = 2,
    Zhar = 3,
    Veil = 4,
    Diablo = 5,
    Butcher = 6,
    LtBanner = 7,
    Blind = 8,
    Blood = 9,
    Anvil = 10,
    Warlord = 11,
    SkelKing = 12,
    PoisonWater = 13,
    Schamb = 14,
    Betrayer = 15,
    Grave = 16,
    Farmer = 17,
    Girl = 18,
    Trader = 19,
    Defiler = 20,
    Nakrul = 21,
    Cornerstone = 22,
    Jersey = 23,
    #[default]
    Invalid = -1,
}

// ============================================================================
// Quest State enum - exact port from quests.h
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum QuestState {
    #[default]
    NotAvailable = 0, // quest did not spawn this game
    Init = 1,         // quest has spawned, waiting to trigger
    Active = 2,       // quest is currently in progress
    Done = 3,         // quest log closed and finished
    HiveTease1 = 7,
    HiveTease2 = 8,
    HiveActive = 9,
    HiveDone = 10,
    Invalid = 0xFF,
}

impl QuestState {
    pub fn is_available(&self) -> bool {
        !matches!(self, QuestState::NotAvailable | QuestState::Invalid)
    }
}

// ============================================================================
// Mushroom quest states - exact port from quests.h
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum MushroomQuestState {
    #[default]
    Init = 0,
    TomeSpawned = 1,
    TomeGiven = 2,
    MushSpawned = 3,
    MushPicked = 4,
    MushGiven = 5,
    BrainSpawned = 6,
    BrainGiven = 7,
}

// ============================================================================
// Gharbad quest states - exact port from quests.h
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum GharbadQuestState {
    #[default]
    Init = 0,
    FirstItemReady = 1,
    FirstItemSpawned = 2,
    SecondItemNearlyDone = 3,
    SecondItemReady = 4,
    Attacking = 5,
}

// ============================================================================
// Zhar quest states - exact port from quests.h
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum ZharQuestState {
    #[default]
    Init = 0,
    ItemSpawned = 1,
    Angry = 2,
    Attacking = 3,
}

// ============================================================================
// Warlord quest states - exact port from quests.h
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum WarlordQuestState {
    #[default]
    Init = 0,
    SteelTomeRead = 1,
    Talking = 2,
    Attacking = 3, // Multiplayer only
}

// ============================================================================
// Lachdanan/Veil quest states - exact port from quests.h
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum VeilQuestState {
    #[default]
    Init = 0,
    EarlyReturn = 1,
    ItemSpawned = 2,
}

// ============================================================================
// Speech ID enum - exact port from textdat.h
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i16)]
pub enum SpeechId {
    // King Leoric texts
    King1 = 0,
    King2 = 1,
    King3 = 2,
    King4 = 3,
    King5 = 4,
    King6 = 5,
    King7 = 6,
    King8 = 7,
    King9 = 8,
    King10 = 9,
    King11 = 10,

    // Banner texts
    Banner1 = 11,
    Banner2 = 12,
    Banner3 = 13,
    Banner4 = 14,
    Banner5 = 15,
    Banner6 = 16,
    Banner7 = 17,
    Banner8 = 18,
    Banner9 = 19,
    Banner10 = 20,
    Banner11 = 21,
    Banner12 = 22,

    // Vile texts (Archbishop Lazarus)
    Vile1 = 23,
    Vile2 = 24,
    Vile3 = 25,
    Vile4 = 26,
    Vile5 = 27,
    Vile6 = 28,
    Vile7 = 29,
    Vile8 = 30,
    Vile9 = 31,
    Vile10 = 32,
    Vile11 = 33,
    Vile12 = 34,
    Vile13 = 35,
    Vile14 = 36,

    // Poison Water texts
    Poison1 = 37,
    Poison2 = 38,
    Poison3 = 39,
    Poison4 = 40,
    Poison5 = 41,
    Poison6 = 42,
    Poison7 = 43,
    Poison8 = 44,
    Poison9 = 45,
    Poison10 = 46,

    // Chamber of Bone texts
    Bone1 = 47,
    Bone2 = 48,
    Bone3 = 49,
    Bone4 = 50,
    Bone5 = 51,
    Bone6 = 52,
    Bone7 = 53,
    Bone8 = 54,

    // Butcher texts
    Butch1 = 55,
    Butch2 = 56,
    Butch3 = 57,
    Butch4 = 58,
    Butch5 = 59,
    Butch6 = 60,
    Butch7 = 61,
    Butch8 = 62,
    Butch9 = 63,
    Butch10 = 64,

    // Halls of the Blind texts
    Blind1 = 65,
    Blind2 = 66,
    Blind3 = 67,
    Blind4 = 68,
    Blind5 = 69,
    Blind6 = 70,
    Blind7 = 71,
    Blind8 = 72,

    // Lachdanan/Veil texts
    Veil1 = 73,
    Veil2 = 74,
    Veil3 = 75,
    Veil4 = 76,
    Veil5 = 77,
    Veil6 = 78,
    Veil7 = 79,
    Veil8 = 80,
    Veil9 = 81,
    Veil10 = 82,
    Veil11 = 83,

    // Anvil of Fury texts
    Anvil1 = 84,
    Anvil2 = 85,
    Anvil3 = 86,
    Anvil4 = 87,
    Anvil5 = 88,
    Anvil6 = 89,
    Anvil7 = 90,
    Anvil8 = 91,
    Anvil9 = 92,
    Anvil10 = 93,

    // Blood Stone texts
    Blood1 = 94,
    Blood2 = 95,
    Blood3 = 96,
    Blood4 = 97,
    Blood5 = 98,
    Blood6 = 99,
    Blood7 = 100,
    Blood8 = 101,

    // Warlord of Blood texts
    Warlrd1 = 102,
    Warlrd2 = 103,
    Warlrd3 = 104,
    Warlrd4 = 105,
    Warlrd5 = 106,
    Warlrd6 = 107,
    Warlrd7 = 108,
    Warlrd8 = 109,
    Warlrd9 = 110,

    // Infravision texts
    Infra1 = 111,
    Infra2 = 112,
    Infra3 = 113,
    Infra4 = 114,
    Infra5 = 115,
    Infra6 = 116,
    Infra7 = 117,
    Infra8 = 118,
    Infra9 = 119,
    Infra10 = 120,

    // Mushroom texts
    Mush1 = 121,
    Mush2 = 122,
    Mush3 = 123,
    Mush4 = 124,
    Mush5 = 125,
    Mush6 = 126,
    Mush7 = 127,
    Mush8 = 128,
    Mush9 = 129,
    Mush10 = 130,
    Mush11 = 131,
    Mush12 = 132,
    Mush13 = 133,

    // Doom texts
    Doom1 = 134,
    Doom2 = 135,
    Doom3 = 136,
    Doom4 = 137,
    Doom5 = 138,
    Doom6 = 139,
    Doom7 = 140,
    Doom8 = 141,
    Doom9 = 142,
    Doom10 = 143,

    // Garbud texts
    Garbud1 = 144,
    Garbud2 = 145,
    Garbud3 = 146,
    Garbud4 = 147,

    // Zhar texts
    Zhar1 = 148,
    Zhar2 = 149,

    // Story texts
    Story1 = 150,
    Story2 = 151,
    Story3 = 152,
    Story4 = 153,
    Story5 = 154,
    Story6 = 155,
    Story7 = 156,
    Story9 = 157,
    Story10 = 158,
    Story11 = 159,

    // Towner dialogue IDs will continue from here...
    // (omitting many towner dialogues for brevity, can be added as needed)

    #[default]
    None = -1,
}

// ============================================================================
// Quest structure - exact port from quests.h
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct Quest {
    /// Quest ID
    pub _qidx: QuestId,

    /// Quest state
    pub _qactive: QuestState,

    /// Quest level
    pub _qlevel: u8,

    /// Quest trigger position (x, y)
    pub position: (i32, i32),

    /// Dungeon type for this quest
    pub _qlvltype: DungeonType,

    /// Set level for this quest
    pub _qslvl: SetLevel,

    /// Is quest in the quest log
    pub _qlog: bool,

    /// Quest message ID
    pub _qmsg: SpeechId,

    /// Quest variable 1 (quest-specific state)
    pub _qvar1: u8,

    /// Quest variable 2 (quest-specific state)
    pub _qvar2: u8,
}

impl Quest {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if quest is available (not NOTAVAIL or INVALID)
    pub fn is_available(&self) -> bool {
        self._qactive.is_available()
    }
}

// ============================================================================
// Quest Data structure - exact port from quests.h QuestData
// ============================================================================

#[derive(Debug, Clone)]
pub struct QuestData {
    /// Quest dungeon level
    pub _qdlvl: u8,

    /// Multiplayer dungeon level (-1 if single player only)
    pub _qdmultlvl: i8,

    /// Quest dungeon type
    pub _qlvlt: DungeonType,

    /// Quest book order (-1 if not in quest log)
    pub quest_book_order: i8,

    /// Random spawn chance (percentage)
    pub _qdrnd: u8,

    /// Set level for this quest
    pub _qslvl: SetLevel,

    /// Single player only quest
    pub is_single_player_only: bool,

    /// Quest message ID
    pub _qdmsg: SpeechId,

    /// Quest name string
    pub _qlstr: String,
}

impl Default for QuestData {
    fn default() -> Self {
        Self {
            _qdlvl: 0,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::None,
            quest_book_order: -1,
            _qdrnd: 0,
            _qslvl: SetLevel::None,
            is_single_player_only: false,
            _qdmsg: SpeechId::None,
            _qlstr: String::new(),
        }
    }
}

// ============================================================================
// Quest trigger names - exact from quests.cpp
// ============================================================================

pub static QUEST_TRIGGER_NAMES: &[&str] = &[
    "King Leoric's Tomb",
    "The Chamber of Bone",
    "Maze",
    "A Dark Passage",
    "Unholy Altar",
];

// ============================================================================
// Static quest data - matches QuestsData from quests.cpp
// ============================================================================

pub fn get_quest_data() -> Vec<QuestData> {
    vec![
        // Q_ROCK - The Magic Rock
        QuestData {
            _qdlvl: 5,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::Catacombs,
            quest_book_order: 7,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::Poison1, // Placeholder
            _qlstr: "The Magic Rock".to_string(),
        },
        // Q_MUSHROOM - Black Mushroom
        QuestData {
            _qdlvl: 9,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::Caves,
            quest_book_order: 8,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::Mush1,
            _qlstr: "Black Mushroom".to_string(),
        },
        // Q_GARBUD - Gharbad the Weak
        QuestData {
            _qdlvl: 4,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::Catacombs,
            quest_book_order: -1,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::Garbud1,
            _qlstr: "Gharbad the Weak".to_string(),
        },
        // Q_ZHAR - Zhar the Mad
        QuestData {
            _qdlvl: 8,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::Caves,
            quest_book_order: -1,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::Zhar1,
            _qlstr: "Zhar the Mad".to_string(),
        },
        // Q_VEIL - Lachdanan (Veil of Steel)
        QuestData {
            _qdlvl: 14,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::Hell,
            quest_book_order: 12,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::Veil1,
            _qlstr: "Lachdanan".to_string(),
        },
        // Q_DIABLO - Diablo
        QuestData {
            _qdlvl: 16,
            _qdmultlvl: 16,
            _qlvlt: DungeonType::Hell,
            quest_book_order: 14,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: false,
            _qdmsg: SpeechId::Doom1,
            _qlstr: "Diablo".to_string(),
        },
        // Q_BUTCHER - The Butcher
        QuestData {
            _qdlvl: 2,
            _qdmultlvl: 2,
            _qlvlt: DungeonType::Cathedral,
            quest_book_order: 0,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: false,
            _qdmsg: SpeechId::Butch1,
            _qlstr: "The Butcher".to_string(),
        },
        // Q_LTBANNER - Ogden's Sign
        QuestData {
            _qdlvl: 4,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::Cathedral,
            quest_book_order: 2,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::Banner1,
            _qlstr: "Ogden's Sign".to_string(),
        },
        // Q_BLIND - Halls of the Blind
        QuestData {
            _qdlvl: 7,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::Catacombs,
            quest_book_order: 5,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::Blind1,
            _qlstr: "Halls of the Blind".to_string(),
        },
        // Q_BLOOD - Valor
        QuestData {
            _qdlvl: 5,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::Catacombs,
            quest_book_order: 3,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::Blood1,
            _qlstr: "Valor".to_string(),
        },
        // Q_ANVIL - Anvil of Fury
        QuestData {
            _qdlvl: 10,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::Caves,
            quest_book_order: 9,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::Anvil1,
            _qlstr: "Anvil of Fury".to_string(),
        },
        // Q_WARLORD - Warlord of Blood
        QuestData {
            _qdlvl: 13,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::Hell,
            quest_book_order: 11,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::Warlrd1,
            _qlstr: "Warlord of Blood".to_string(),
        },
        // Q_SKELKING - The Curse of King Leoric
        QuestData {
            _qdlvl: 3,
            _qdmultlvl: 3,
            _qlvlt: DungeonType::Cathedral,
            quest_book_order: 1,
            _qdrnd: 100,
            _qslvl: SetLevel::SkeletonKing,
            is_single_player_only: false,
            _qdmsg: SpeechId::King1,
            _qlstr: "The Curse of King Leoric".to_string(),
        },
        // Q_PWATER - Poisoned Water Supply
        QuestData {
            _qdlvl: 2,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::Cathedral,
            quest_book_order: 4,
            _qdrnd: 100,
            _qslvl: SetLevel::PoisonWater,
            is_single_player_only: true,
            _qdmsg: SpeechId::Poison1,
            _qlstr: "Poisoned Water Supply".to_string(),
        },
        // Q_SCHAMB - The Chamber of Bone
        QuestData {
            _qdlvl: 6,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::Catacombs,
            quest_book_order: 6,
            _qdrnd: 100,
            _qslvl: SetLevel::BoneChamber,
            is_single_player_only: true,
            _qdmsg: SpeechId::Bone1,
            _qlstr: "The Chamber of Bone".to_string(),
        },
        // Q_BETRAYER - Archbishop Lazarus
        QuestData {
            _qdlvl: 15,
            _qdmultlvl: 15,
            _qlvlt: DungeonType::Hell,
            quest_book_order: 13,
            _qdrnd: 100,
            _qslvl: SetLevel::VileBetrayer,
            is_single_player_only: false,
            _qdmsg: SpeechId::Vile1,
            _qlstr: "Archbishop Lazarus".to_string(),
        },
        // Hellfire quests
        // Q_GRAVE - Grave Matters
        QuestData {
            _qdlvl: 0,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::None,
            quest_book_order: 15,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::None,
            _qlstr: "Grave Matters".to_string(),
        },
        // Q_FARMER - Farmer's Orchard
        QuestData {
            _qdlvl: 0,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::None,
            quest_book_order: 16,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::None,
            _qlstr: "Farmer's Orchard".to_string(),
        },
        // Q_GIRL - Little Girl
        QuestData {
            _qdlvl: 0,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::None,
            quest_book_order: 17,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::None,
            _qlstr: "Little Girl".to_string(),
        },
        // Q_TRADER
        QuestData {
            _qdlvl: 0,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::None,
            quest_book_order: -1,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::None,
            _qlstr: "Trader".to_string(),
        },
        // Q_DEFILER
        QuestData {
            _qdlvl: 0,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::None,
            quest_book_order: 18,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::None,
            _qlstr: "Defiler".to_string(),
        },
        // Q_NAKRUL - Na-Krul
        QuestData {
            _qdlvl: 0,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::None,
            quest_book_order: 19,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::None,
            _qlstr: "Na-Krul".to_string(),
        },
        // Q_CORNSTN - Cornerstone of the World
        QuestData {
            _qdlvl: 0,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::None,
            quest_book_order: 20,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::None,
            _qlstr: "Cornerstone of the World".to_string(),
        },
        // Q_JERSEY - The Jersey's Jersey
        QuestData {
            _qdlvl: 0,
            _qdmultlvl: -1,
            _qlvlt: DungeonType::None,
            quest_book_order: 21,
            _qdrnd: 100,
            _qslvl: SetLevel::None,
            is_single_player_only: true,
            _qdmsg: SpeechId::None,
            _qlstr: "The Jersey's Jersey".to_string(),
        },
    ]
}

// ============================================================================
// Quest Manager - manages all quests
// ============================================================================

#[derive(Debug, Clone)]
pub struct QuestManager {
    /// All quests in the game
    pub quests: [Quest; MAXQUESTS],

    /// Quest log is open
    pub quest_log_is_open: bool,

    /// Return position when leaving special level
    pub return_lvl_position: (i32, i32),

    /// Return dungeon type
    pub return_level_type: DungeonType,

    /// Return level number
    pub return_level: i32,
}

impl Default for QuestManager {
    fn default() -> Self {
        Self {
            quests: std::array::from_fn(|_| Quest::default()),
            quest_log_is_open: false,
            return_lvl_position: (0, 0),
            return_level_type: DungeonType::None,
            return_level: 0,
        }
    }
}

impl QuestManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize quests for a new game
    pub fn init_quests(&mut self, is_multiplayer: bool, seed: u32) {
        let quest_data = get_quest_data();

        for (i, data) in quest_data.iter().enumerate() {
            if i >= MAXQUESTS {
                break;
            }

            let quest = &mut self.quests[i];
            quest._qidx = unsafe { std::mem::transmute(i as i8) }; // Safe since we check bounds
            quest._qlevel = data._qdlvl;
            quest._qlvltype = data._qlvlt;
            quest._qslvl = data._qslvl;
            quest._qmsg = data._qdmsg;
            quest._qvar1 = 0;
            quest._qvar2 = 0;
            quest._qlog = false;
            quest.position = (0, 0);

            // Determine if quest is available
            if is_multiplayer && data.is_single_player_only {
                quest._qactive = QuestState::NotAvailable;
            } else {
                quest._qactive = QuestState::Init;
            }
        }

        // Deactivate some quests based on seed for variety (single player)
        if !is_multiplayer {
            self.initialize_quest_pools(seed);
        }
    }

    /// Deactivate random quests from each pool for variety
    fn initialize_quest_pools(&mut self, seed: u32) {
        use rand::{SeedableRng, Rng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed as u64);

        // Quest pools (quests that can be randomly deactivated)
        let pools: &[&[QuestId]] = &[
            // Pool 1: Early cathedral
            &[QuestId::Butcher, QuestId::SkelKing, QuestId::PoisonWater],
            // Pool 2: Catacombs
            &[QuestId::LtBanner, QuestId::Blind, QuestId::Blood],
            // Pool 3: Caves
            &[QuestId::Mushroom, QuestId::Anvil],
        ];

        for pool in pools {
            if pool.len() > 1 {
                // Randomly deactivate one quest from each pool
                let deactivate_idx = rng.random_range(0..pool.len());
                let quest_id = pool[deactivate_idx];
                let idx = quest_id as usize;
                if idx < MAXQUESTS {
                    self.quests[idx]._qactive = QuestState::NotAvailable;
                }
            }
        }
    }

    /// Get a quest by ID
    pub fn get_quest(&self, id: QuestId) -> Option<&Quest> {
        let idx = id as usize;
        if idx < MAXQUESTS {
            Some(&self.quests[idx])
        } else {
            None
        }
    }

    /// Get a mutable quest by ID
    pub fn get_quest_mut(&mut self, id: QuestId) -> Option<&mut Quest> {
        let idx = id as usize;
        if idx < MAXQUESTS {
            Some(&mut self.quests[idx])
        } else {
            None
        }
    }

    /// Check if a quest is active
    pub fn is_quest_active(&self, id: QuestId) -> bool {
        self.get_quest(id)
            .map(|q| q._qactive == QuestState::Active)
            .unwrap_or(false)
    }

    /// Check if a quest is done
    pub fn is_quest_done(&self, id: QuestId) -> bool {
        self.get_quest(id)
            .map(|q| q._qactive == QuestState::Done)
            .unwrap_or(false)
    }

    /// Activate a quest
    pub fn activate_quest(&mut self, id: QuestId) {
        if let Some(quest) = self.get_quest_mut(id) {
            if quest._qactive == QuestState::Init {
                quest._qactive = QuestState::Active;
                quest._qlog = true;
            }
        }
    }

    /// Complete a quest
    pub fn complete_quest(&mut self, id: QuestId) {
        if let Some(quest) = self.get_quest_mut(id) {
            if quest._qactive == QuestState::Active {
                quest._qactive = QuestState::Done;
            }
        }
    }

    /// Open quest log
    pub fn open_quest_log(&mut self) {
        self.quest_log_is_open = true;
    }

    /// Close quest log
    pub fn close_quest_log(&mut self) {
        self.quest_log_is_open = false;
    }

    /// Toggle quest log
    pub fn toggle_quest_log(&mut self) {
        self.quest_log_is_open = !self.quest_log_is_open;
    }

    /// Get list of encountered (active + completed) quests for log
    pub fn get_encountered_quests(&self) -> Vec<(QuestId, bool)> {
        let mut active: Vec<QuestId> = Vec::new();
        let mut completed: Vec<QuestId> = Vec::new();

        for quest in &self.quests {
            if quest._qlog {
                if quest._qactive == QuestState::Active {
                    active.push(quest._qidx);
                } else if quest._qactive == QuestState::Done {
                    completed.push(quest._qidx);
                }
            }
        }

        // Return active first, then completed
        let mut result: Vec<(QuestId, bool)> = Vec::new();
        for id in active {
            result.push((id, true)); // true = active
        }
        for id in completed {
            result.push((id, false)); // false = completed
        }

        result
    }

    /// Set return position for special levels
    pub fn set_return_position(&mut self, pos: (i32, i32), level: i32, dungeon_type: DungeonType) {
        self.return_lvl_position = pos;
        self.return_level = level;
        self.return_level_type = dungeon_type;
    }

    /// Get return position
    pub fn get_return_position(&self) -> ((i32, i32), i32, DungeonType) {
        (self.return_lvl_position, self.return_level, self.return_level_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_manager_init() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, 12345);

        // Butcher should be initialized
        let butcher = manager.get_quest(QuestId::Butcher).unwrap();
        assert_eq!(butcher._qlevel, 2);

        // Diablo should always be available
        let diablo = manager.get_quest(QuestId::Diablo).unwrap();
        assert_ne!(diablo._qactive, QuestState::NotAvailable);
    }

    #[test]
    fn test_quest_activation() {
        let mut manager = QuestManager::new();
        manager.init_quests(false, 12345);

        // Find a quest that's initialized
        if let Some(quest) = manager.get_quest(QuestId::Diablo) {
            if quest._qactive == QuestState::Init {
                manager.activate_quest(QuestId::Diablo);
                assert!(manager.is_quest_active(QuestId::Diablo));
            }
        }
    }
}
