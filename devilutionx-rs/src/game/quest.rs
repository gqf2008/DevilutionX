//! Quest System for DevilutionX-RS
//!
//! Implements the quest system from Diablo 1.
//! References: Source/quests.cpp, Source/quests.h

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::prelude::IndexedRandom;

/// Quest IDs (from quests.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuestId {
    // Cathedral Quests (Levels 1-4)
    Butcher,
    PoisonedWater,
    SkeletonKing,

    // Catacombs Quests (Levels 5-8)
    BoneChamber,
    Valor,

    // Caves Quests (Levels 9-12)
    Mushroom,
    Anvil,

    // Hell Quests (Levels 13-16)
    Warlord,
    Lachdanan,
    Archbishop,
    Diablo,

    // Hellfire Quests
    CowQuest,
    DefilerQuest,
    NaKrul,
    CornerStone,
    Farmer,
    GirlGrave,
    JerseyJersey,
    TheoPotion,
}

impl QuestId {
    /// Get all quests for a game (randomly selected subset)
    pub fn random_quest_set(hellfire: bool, seed: u64) -> Vec<QuestId> {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let mut rng = StdRng::seed_from_u64(seed);
        let mut quests = Vec::new();

        // Cathedral - always has Butcher and Skeleton King, one optional
        quests.push(QuestId::Butcher);
        quests.push(QuestId::SkeletonKing);
        if [true, false].choose(&mut rng).copied().unwrap_or(true) {
            quests.push(QuestId::PoisonedWater);
        }

        // Catacombs - always Valor, one optional
        quests.push(QuestId::Valor);
        if [true, false].choose(&mut rng).copied().unwrap_or(true) {
            quests.push(QuestId::BoneChamber);
        }

        // Caves - random selection
        let cave_quests = [QuestId::Mushroom, QuestId::Anvil];
        if let Some(&q) = cave_quests.choose(&mut rng) {
            quests.push(q);
        }

        // Hell - always Diablo, optional others
        quests.push(QuestId::Diablo);
        let hell_optional = [QuestId::Warlord, QuestId::Lachdanan, QuestId::Archbishop];
        for &q in &hell_optional {
            if [true, false].choose(&mut rng).copied().unwrap_or(false) {
                quests.push(q);
            }
        }

        // Hellfire quests
        if hellfire {
            quests.push(QuestId::CowQuest);
            quests.push(QuestId::DefilerQuest);
            quests.push(QuestId::NaKrul);
        }

        quests
    }

    /// Get the dungeon level where this quest takes place
    pub fn dungeon_level(&self) -> u8 {
        use QuestId::*;
        match self {
            Butcher => 2,
            PoisonedWater => 2,
            SkeletonKing => 3,
            BoneChamber => 6,
            Valor => 5,
            Mushroom => 9,
            Anvil => 10,
            Warlord => 13,
            Lachdanan => 14,
            Archbishop => 15,
            Diablo => 16,
            CowQuest => 0,  // Town
            DefilerQuest => 17,
            NaKrul => 20,
            CornerStone => 21,
            Farmer => 0,
            GirlGrave => 0,
            JerseyJersey => 0,
            TheoPotion => 0,
        }
    }

    /// Get the quest name
    pub fn name(&self) -> &'static str {
        use QuestId::*;
        match self {
            Butcher => "The Butcher",
            PoisonedWater => "Poisoned Water Supply",
            SkeletonKing => "The Curse of King Leoric",
            BoneChamber => "The Chamber of Bone",
            Valor => "Halls of the Blind",
            Mushroom => "Mushroom Quest",
            Anvil => "Anvil of Fury",
            Warlord => "Warlord of Blood",
            Lachdanan => "Lachdanan",
            Archbishop => "Archbishop Lazarus",
            Diablo => "Diablo",
            CowQuest => "The Jersey's Jersey",
            DefilerQuest => "The Defiler",
            NaKrul => "Na-Krul",
            CornerStone => "Cornerstone of the World",
            Farmer => "Farmer's Orchard",
            GirlGrave => "Girl's Grave",
            JerseyJersey => "Jersey's Jersey",
            TheoPotion => "Theodore's Potion",
        }
    }

    /// Get quest giver NPC
    pub fn quest_giver(&self) -> Option<&'static str> {
        use QuestId::*;
        match self {
            Butcher => Some("Wounded Townsman"),
            PoisonedWater => Some("Pepin"),
            SkeletonKing => Some("Ogden"),
            BoneChamber => None,
            Valor => Some("Book"),
            Mushroom => Some("Adria"),
            Anvil => Some("Griswold"),
            Warlord => None,
            Lachdanan => Some("Lachdanan"),
            Archbishop => Some("Cain"),
            Diablo => Some("Cain"),
            CowQuest => Some("Lester the Farmer"),
            _ => None,
        }
    }
}

/// Quest states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestState {
    /// Quest not yet available
    NotAvailable,
    /// Quest available but not started
    Available,
    /// Quest in progress (active)
    Active,
    /// Quest completed successfully
    Completed,
    /// Quest failed
    Failed,
}

/// Quest progress data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestProgress {
    pub id: QuestId,
    pub state: QuestState,
    /// Progress counter (quest-specific meaning)
    pub progress: i32,
    /// Max progress for completion
    pub max_progress: i32,
    /// Quest-specific flags
    pub flags: u32,
    /// Log entries shown to player
    pub log_entries: Vec<String>,
}

impl QuestProgress {
    pub fn new(id: QuestId) -> Self {
        let max_progress = match id {
            QuestId::Butcher => 1,         // Kill butcher
            QuestId::PoisonedWater => 1,   // Purify water
            QuestId::SkeletonKing => 1,    // Kill Leoric
            QuestId::BoneChamber => 1,     // Read book
            QuestId::Valor => 1,           // Find amulet
            QuestId::Mushroom => 3,        // Find mushroom, spectral elixir, brain
            QuestId::Anvil => 1,           // Find anvil
            QuestId::Warlord => 1,         // Kill warlord
            QuestId::Lachdanan => 1,       // Give golden elixir
            QuestId::Archbishop => 1,      // Kill Lazarus
            QuestId::Diablo => 1,          // Kill Diablo
            _ => 1,
        };

        Self {
            id,
            state: QuestState::NotAvailable,
            progress: 0,
            max_progress,
            flags: 0,
            log_entries: Vec::new(),
        }
    }

    /// Start the quest
    pub fn start(&mut self) {
        if self.state == QuestState::Available {
            self.state = QuestState::Active;
            self.add_log(format!("Quest started: {}", self.id.name()));
        }
    }

    /// Advance quest progress
    pub fn advance(&mut self, amount: i32) {
        if self.state == QuestState::Active {
            self.progress = (self.progress + amount).min(self.max_progress);
            if self.progress >= self.max_progress {
                self.complete();
            }
        }
    }

    /// Complete the quest
    pub fn complete(&mut self) {
        if self.state == QuestState::Active {
            self.state = QuestState::Completed;
            self.add_log(format!("Quest completed: {}", self.id.name()));
        }
    }

    /// Fail the quest
    pub fn fail(&mut self) {
        if self.state == QuestState::Active {
            self.state = QuestState::Failed;
            self.add_log(format!("Quest failed: {}", self.id.name()));
        }
    }

    /// Add log entry
    pub fn add_log(&mut self, entry: String) {
        self.log_entries.push(entry);
    }

    /// Check if quest is active
    pub fn is_active(&self) -> bool {
        self.state == QuestState::Active
    }

    /// Check if quest is complete
    pub fn is_complete(&self) -> bool {
        self.state == QuestState::Completed
    }

    /// Get progress percentage
    pub fn progress_percent(&self) -> f32 {
        if self.max_progress > 0 {
            self.progress as f32 / self.max_progress as f32
        } else {
            0.0
        }
    }
}

/// Quest reward types
#[derive(Debug, Clone)]
pub enum QuestReward {
    Experience(u32),
    Gold(i32),
    Item(String),  // Item template name
    Spell(u8),     // Spell ID
    StatPoint(u8), // Which stat
}

/// Quest data with all details
#[derive(Debug, Clone)]
pub struct QuestData {
    pub id: QuestId,
    pub name: &'static str,
    pub description: &'static str,
    pub dungeon_level: u8,
    pub rewards: Vec<QuestReward>,
    /// Dialogue when quest is given
    pub dialogue_start: &'static str,
    /// Dialogue when quest is complete
    pub dialogue_complete: &'static str,
}

impl QuestData {
    pub fn get(id: QuestId) -> Self {
        match id {
            QuestId::Butcher => Self {
                id,
                name: "The Butcher",
                description: "A wounded townsman has told of a hideous creature lurking in the labyrinth. Find and destroy this Butcher.",
                dungeon_level: 2,
                rewards: vec![
                    QuestReward::Experience(1000),
                    QuestReward::Item("Butcher's Cleaver".to_string()),
                ],
                dialogue_start: "Please, listen to me... The Archbishop Lazarus, he led us down here to find the lost prince. The Butcher killed them all... I saw him feast on their remains!",
                dialogue_complete: "You have done the impossible! The Butcher is dead and the souls of the innocent can rest in peace.",
            },
            QuestId::PoisonedWater => Self {
                id,
                name: "Poisoned Water Supply",
                description: "The town's water supply has been poisoned. Find the source and purify it.",
                dungeon_level: 2,
                rewards: vec![
                    QuestReward::Experience(500),
                    QuestReward::Gold(500),
                    QuestReward::Item("Ring of Truth".to_string()),
                ],
                dialogue_start: "Many of the townspeople have fallen ill from a mysterious plague. I believe the water supply has been poisoned. Can you find the source?",
                dialogue_complete: "The water is pure again. You have saved many lives this day!",
            },
            QuestId::SkeletonKing => Self {
                id,
                name: "The Curse of King Leoric",
                description: "The mad king still roams his tomb. Put his soul to rest.",
                dungeon_level: 3,
                rewards: vec![
                    QuestReward::Experience(2000),
                    QuestReward::Item("Undead Crown".to_string()),
                ],
                dialogue_start: "The story of King Leoric is a tragic one. Once a noble ruler, he fell to madness and now his cursed spirit haunts these halls.",
                dialogue_complete: "Leoric's spirit is finally at peace. The curse that bound him to this world has been broken.",
            },
            QuestId::Valor => Self {
                id,
                name: "Halls of the Blind",
                description: "Find the legendary Valor amulet in the Halls of the Blind.",
                dungeon_level: 5,
                rewards: vec![
                    QuestReward::Experience(1500),
                    QuestReward::Item("Arkaine's Valor".to_string()),
                ],
                dialogue_start: "I once read of a wondrous armor called Arkaine's Valor. It was hidden away in the Halls of the Blind.",
                dialogue_complete: "You have found Arkaine's Valor! This legendary armor will serve you well.",
            },
            QuestId::Diablo => Self {
                id,
                name: "Diablo",
                description: "Defeat the Lord of Terror himself.",
                dungeon_level: 16,
                rewards: vec![
                    QuestReward::Experience(10000),
                ],
                dialogue_start: "The Lord of Terror waits below. You must destroy Diablo to save us all.",
                dialogue_complete: "You have done it! Diablo is defeated and the world is safe... for now.",
            },
            _ => Self {
                id,
                name: id.name(),
                description: "A quest awaits...",
                dungeon_level: id.dungeon_level(),
                rewards: vec![QuestReward::Experience(500)],
                dialogue_start: "There is work to be done.",
                dialogue_complete: "Well done, hero!",
            },
        }
    }
}

/// Quest manager
pub struct QuestManager {
    /// All quests in this game
    quests: HashMap<QuestId, QuestProgress>,
    /// Active quests for quick lookup
    active_quests: Vec<QuestId>,
    /// Completed quests
    completed_quests: Vec<QuestId>,
    /// Whether Hellfire content is enabled
    hellfire: bool,
}

impl Default for QuestManager {
    fn default() -> Self {
        Self::new(false, 0)
    }
}

impl QuestManager {
    pub fn new(hellfire: bool, seed: u64) -> Self {
        let quest_ids = QuestId::random_quest_set(hellfire, seed);
        let mut quests = HashMap::new();

        for id in quest_ids {
            quests.insert(id, QuestProgress::new(id));
        }

        Self {
            quests,
            active_quests: Vec::new(),
            completed_quests: Vec::new(),
            hellfire,
        }
    }

    /// Check if quest exists in this game
    pub fn has_quest(&self, id: QuestId) -> bool {
        self.quests.contains_key(&id)
    }

    /// Get quest progress
    pub fn get_quest(&self, id: QuestId) -> Option<&QuestProgress> {
        self.quests.get(&id)
    }

    /// Get mutable quest progress
    pub fn get_quest_mut(&mut self, id: QuestId) -> Option<&mut QuestProgress> {
        self.quests.get_mut(&id)
    }

    /// Make quest available
    pub fn make_available(&mut self, id: QuestId) {
        if let Some(quest) = self.quests.get_mut(&id) {
            if quest.state == QuestState::NotAvailable {
                quest.state = QuestState::Available;
            }
        }
    }

    /// Start a quest
    pub fn start_quest(&mut self, id: QuestId) -> bool {
        if let Some(quest) = self.quests.get_mut(&id) {
            if quest.state == QuestState::Available {
                quest.start();
                self.active_quests.push(id);
                return true;
            }
        }
        false
    }

    /// Complete a quest
    pub fn complete_quest(&mut self, id: QuestId) -> Vec<QuestReward> {
        if let Some(quest) = self.quests.get_mut(&id) {
            if quest.state == QuestState::Active {
                quest.complete();
                self.active_quests.retain(|&q| q != id);
                self.completed_quests.push(id);
                return QuestData::get(id).rewards;
            }
        }
        Vec::new()
    }

    /// Fail a quest
    pub fn fail_quest(&mut self, id: QuestId) {
        if let Some(quest) = self.quests.get_mut(&id) {
            quest.fail();
            self.active_quests.retain(|&q| q != id);
        }
    }

    /// Advance quest progress (e.g., kill count)
    pub fn advance_quest(&mut self, id: QuestId, amount: i32) -> bool {
        if let Some(quest) = self.quests.get_mut(&id) {
            quest.advance(amount);
            if quest.is_complete() {
                self.active_quests.retain(|&q| q != id);
                self.completed_quests.push(id);
                return true;  // Quest just completed
            }
        }
        false
    }

    /// Get all active quests
    pub fn active_quests(&self) -> &[QuestId] {
        &self.active_quests
    }

    /// Get all completed quests
    pub fn completed_quests(&self) -> &[QuestId] {
        &self.completed_quests
    }

    /// Get quests for a dungeon level
    pub fn quests_for_level(&self, level: u8) -> Vec<QuestId> {
        self.quests.iter()
            .filter(|(_, q)| q.id.dungeon_level() == level)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Update quests when entering a dungeon level
    pub fn on_enter_level(&mut self, level: u8) {
        // Make quests available on their respective levels
        for id in self.quests_for_level(level) {
            self.make_available(id);
        }
    }

    /// Check if killing a monster progresses any quest
    pub fn on_monster_killed(&mut self, monster_type: &str) -> Option<QuestId> {
        // Check if this monster is a quest target
        match monster_type {
            "Butcher" => {
                if self.advance_quest(QuestId::Butcher, 1) {
                    return Some(QuestId::Butcher);
                }
            }
            "Skeleton King" | "King Leoric" => {
                if self.advance_quest(QuestId::SkeletonKing, 1) {
                    return Some(QuestId::SkeletonKing);
                }
            }
            "Archbishop Lazarus" => {
                if self.advance_quest(QuestId::Archbishop, 1) {
                    return Some(QuestId::Archbishop);
                }
            }
            "Diablo" => {
                if self.advance_quest(QuestId::Diablo, 1) {
                    return Some(QuestId::Diablo);
                }
            }
            _ => {}
        }
        None
    }

    /// Serialize quest state for saving
    pub fn to_save_data(&self) -> Vec<(u8, u8, i32)> {
        self.quests.iter()
            .map(|(&id, progress)| {
                let id_byte = match id {
                    QuestId::Butcher => 0,
                    QuestId::PoisonedWater => 1,
                    QuestId::SkeletonKing => 2,
                    QuestId::BoneChamber => 3,
                    QuestId::Valor => 4,
                    QuestId::Mushroom => 5,
                    QuestId::Anvil => 6,
                    QuestId::Warlord => 7,
                    QuestId::Lachdanan => 8,
                    QuestId::Archbishop => 9,
                    QuestId::Diablo => 10,
                    _ => 255,
                };
                let state_byte = match progress.state {
                    QuestState::NotAvailable => 0,
                    QuestState::Available => 1,
                    QuestState::Active => 2,
                    QuestState::Completed => 3,
                    QuestState::Failed => 4,
                };
                (id_byte, state_byte, progress.progress)
            })
            .collect()
    }
}

/// Quest log for displaying to player
pub struct QuestLog {
    pub entries: Vec<QuestLogEntry>,
}

#[derive(Debug, Clone)]
pub struct QuestLogEntry {
    pub quest_id: QuestId,
    pub name: String,
    pub description: String,
    pub state: QuestState,
    pub progress_text: String,
}

impl QuestLog {
    pub fn from_manager(manager: &QuestManager) -> Self {
        let entries = manager.quests.values()
            .filter(|q| q.state != QuestState::NotAvailable)
            .map(|q| {
                let data = QuestData::get(q.id);
                let progress_text = match q.state {
                    QuestState::Completed => "Completed".to_string(),
                    QuestState::Failed => "Failed".to_string(),
                    QuestState::Active => format!("{}/{}", q.progress, q.max_progress),
                    _ => "Not started".to_string(),
                };
                QuestLogEntry {
                    quest_id: q.id,
                    name: data.name.to_string(),
                    description: data.description.to_string(),
                    state: q.state,
                    progress_text,
                }
            })
            .collect();

        Self { entries }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_creation() {
        let manager = QuestManager::new(false, 12345);
        assert!(manager.has_quest(QuestId::Butcher));
        assert!(manager.has_quest(QuestId::SkeletonKing));
        assert!(manager.has_quest(QuestId::Diablo));
    }

    #[test]
    fn test_quest_progression() {
        let mut manager = QuestManager::new(false, 12345);

        // Make butcher quest available
        manager.make_available(QuestId::Butcher);

        // Start quest
        assert!(manager.start_quest(QuestId::Butcher));
        assert!(manager.get_quest(QuestId::Butcher).unwrap().is_active());

        // Complete by killing butcher
        let completed = manager.on_monster_killed("Butcher");
        assert_eq!(completed, Some(QuestId::Butcher));
        assert!(manager.get_quest(QuestId::Butcher).unwrap().is_complete());
    }

    #[test]
    fn test_quest_data() {
        let butcher = QuestData::get(QuestId::Butcher);
        assert_eq!(butcher.name, "The Butcher");
        assert_eq!(butcher.dungeon_level, 2);
        assert!(!butcher.rewards.is_empty());
    }
}
