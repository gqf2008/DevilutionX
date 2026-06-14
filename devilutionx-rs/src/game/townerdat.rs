//! Towner Data - NPC data loading from TSV files
//!
//! C++ Reference: Source/townerdat.cpp, Source/townerdat.hpp
//!
//! Implementation of towner data loading.

use std::collections::HashMap;
use crate::engine::{Point, Direction};
use super::textdat::SpeechId;
use super::quests::QuestId;

/// Towner type ID (simplified from TalkerId)
///
/// C++ Reference: `_talker_id` enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i8)]
pub enum TalkerId {
    #[default]
    None = -1,
    Smith = 0,
    Healer = 1,
    Tavern = 2,
    Story = 3,
    Drunk = 4,
    Witch = 5,
    PegBoy = 6,
    Cow = 7,
    Farmer = 8,
    Girl = 9,
    CowFarmer = 10,
}

/// Maximum number of quests
pub const MAXQUESTS: usize = 24;

/// Data for a single towner entry loaded from TSV
///
/// C++ Reference: `TownerDataEntry` struct in townerdat.hpp
#[derive(Debug, Clone)]
pub struct TownerDataEntry {
    /// Towner type
    pub towner_type: TalkerId,
    /// Display name
    pub name: String,
    /// Position in town
    pub position: Point,
    /// Facing direction
    pub direction: Direction,
    /// Animation width
    pub anim_width: u16,
    /// Animation file path
    pub anim_path: String,
    /// Number of animation frames
    pub anim_frames: u8,
    /// Animation delay
    pub anim_delay: i16,
    /// Gossip text IDs
    pub gossip_texts: Vec<SpeechId>,
    /// Animation frame order
    pub anim_order: Vec<u8>,
}

impl Default for TownerDataEntry {
    fn default() -> Self {
        Self {
            towner_type: TalkerId::None,
            name: String::new(),
            position: Point::new(0, 0),
            direction: Direction::South,
            anim_width: 0,
            anim_path: String::new(),
            anim_frames: 0,
            anim_delay: 0,
            gossip_texts: Vec::new(),
            anim_order: Vec::new(),
        }
    }
}

/// Contains the data for all towners loaded from TSV
pub static mut TOWNERS_DATA_ENTRIES: Vec<TownerDataEntry> = Vec::new();

/// Contains the quest dialog table loaded from TSV
/// Indexed by [towner_type][quest_id]
pub static mut TOWNER_QUEST_DIALOG_TABLE: Option<HashMap<TalkerId, [SpeechId; MAXQUESTS]>> = None;

/// Load towner data from TSV files
///
/// C++ Reference: `LoadTownerData()` in townerdat.cpp
pub fn load_towner_data() {
    load_towners_from_file();
    load_quest_dialog_from_file();
}

/// Load towners from TSV file
fn load_towners_from_file() {
    unsafe {
        TOWNERS_DATA_ENTRIES.clear();
    }
    
    // TODO: Load from txtdata/towners/towners.tsv
}

/// Load quest dialog from TSV file
fn load_quest_dialog_from_file() {
    unsafe {
        if TOWNER_QUEST_DIALOG_TABLE.is_none() {
            TOWNER_QUEST_DIALOG_TABLE = Some(HashMap::new());
        } else {
            TOWNER_QUEST_DIALOG_TABLE.as_mut().unwrap().clear();
        }
    }
    
    // TODO: Load from txtdata/towners/quest_dialog.tsv
}

/// Get the quest dialog speech ID for a towner and quest combination
///
/// C++ Reference: `GetTownerQuestDialog()` in townerdat.cpp
pub fn get_towner_quest_dialog(towner_type: TalkerId, quest: QuestId) -> SpeechId {
    let quest_idx = quest as usize;
    if quest_idx >= MAXQUESTS {
        return SpeechId::None;
    }
    
    unsafe {
        if let Some(ref table) = TOWNER_QUEST_DIALOG_TABLE {
            if let Some(dialogs) = table.get(&towner_type) {
                return dialogs[quest_idx];
            }
        }
    }
    
    SpeechId::None
}

/// Set the quest dialog speech ID for a towner and quest combination
///
/// C++ Reference: `SetTownerQuestDialog()` in townerdat.cpp
pub fn set_towner_quest_dialog(towner_type: TalkerId, quest: QuestId, speech: SpeechId) {
    let quest_idx = quest as usize;
    if quest_idx >= MAXQUESTS {
        return;
    }
    
    unsafe {
        if TOWNER_QUEST_DIALOG_TABLE.is_none() {
            TOWNER_QUEST_DIALOG_TABLE = Some(HashMap::new());
        }
        
        let table = TOWNER_QUEST_DIALOG_TABLE.as_mut().unwrap();
        let dialogs = table.entry(towner_type).or_insert([SpeechId::None; MAXQUESTS]);
        dialogs[quest_idx] = speech;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_towner_data_entry_default() {
        let entry = TownerDataEntry::default();
        assert_eq!(entry.towner_type, TalkerId::None);
        assert!(entry.name.is_empty());
        assert_eq!(entry.anim_frames, 0);
    }
}
