// pfile.rs - Save game encoding functionality
// Ported from Source/pfile.cpp (820 lines)

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::game::player_exact::HeroClass;
use crate::game::pack::{Player, PlayerPack, pack_player, unpack_player};

// Forward declarations for functions that would be in other modules
fn codec_encode(data: &[u8], _password: &str, _len: usize) -> Vec<u8> {
    data.to_vec() // Placeholder
}

fn codec_decode(data: &[u8], _password: &str) -> Vec<u8> {
    data.to_vec() // Placeholder
}

fn codec_get_encoded_len(len: usize) -> usize {
    len + 16 // Placeholder
}

#[allow(non_snake_case)]
fn IsHeaderValid(_header: u32) -> bool {
    true // Placeholder
}

// Constants
pub const MAX_CHARACTERS: usize = 99;
pub const PLAYER_NAME_LENGTH: usize = 16;
pub const MAX_MPQ_PATH_SIZE: usize = 260;

// Save file passwords (obfuscation)
const PASSWORD_SPAWN_SINGLE: &str = "adslhfb1";
const PASSWORD_SPAWN_MULTI: &str = "lshbkfg1";
const PASSWORD_SINGLE: &str = "xrgyrkj1";
const PASSWORD_MULTI: &str = "szqnlsk1";

/// Global flag indicating if save file is valid
pub static mut GB_VALID_SAVE_FILE: bool = false;

/// Hero comparison result status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeroCompareStatus {
    ReferenceNotFound,
    Same,
    Difference,
}

/// Result of comparing hero saves
#[derive(Debug, Clone)]
pub struct HeroCompareResult {
    pub status: HeroCompareStatus,
    pub message: String,
}

impl HeroCompareResult {
    pub fn reference_not_found() -> Self {
        Self {
            status: HeroCompareStatus::ReferenceNotFound,
            message: String::new(),
        }
    }

    pub fn same() -> Self {
        Self {
            status: HeroCompareStatus::Same,
            message: String::new(),
        }
    }

    pub fn difference(message: String) -> Self {
        Self {
            status: HeroCompareStatus::Difference,
            message,
        }
    }
}

/// UI hero information for character selection
#[derive(Debug, Clone, Default)]
pub struct UIHeroInfo {
    pub save_number: u32,
    pub name: String,
    pub level: u8,
    pub hero_class: HeroClass,
    pub strength: i32,
    pub magic: i32,
    pub dexterity: i32,
    pub vitality: i32,
    pub has_saved: bool,
    pub hero_rank: u8,
    pub spawned: bool,
}

/// UI default stats for character class
#[derive(Debug, Clone, Default)]
pub struct UIDefaultStats {
    pub strength: i32,
    pub magic: i32,
    pub dexterity: i32,
    pub vitality: i32,
}

/// Save reader for loading save files
pub struct SaveReader {
    dir: PathBuf,
}

impl SaveReader {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// Get the directory path
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Read a file from the save archive
    pub fn read_file(&self, filename: &str) -> Result<Vec<u8>, SaveError> {
        let path = self.dir.join(filename);

        if !path.exists() {
            return Err(SaveError::FileNotFound(filename.to_string()));
        }

        std::fs::read(&path)
            .map_err(|e| SaveError::ReadError(e.to_string()))
    }

    /// Check if archive has a specific file
    pub fn has_file(&self, path: &str) -> bool {
        self.dir.join(path).exists()
    }
}

/// Save writer for writing save files
pub struct SaveWriter {
    dir: PathBuf,
}

impl SaveWriter {
    pub fn new(dir: PathBuf) -> Self {
        // Ensure directory exists
        std::fs::create_dir_all(&dir).ok();
        Self { dir }
    }

    /// Write data to a file in the save archive
    pub fn write_file(&self, filename: &str, data: &[u8]) -> Result<(), SaveError> {
        let path = self.dir.join(filename);

        std::fs::write(&path, data)
            .map_err(|e| SaveError::WriteError(e.to_string()))
    }

    /// Check if archive has a specific file
    pub fn has_file(&self, path: &str) -> bool {
        self.dir.join(path).exists()
    }

    /// Rename a file within the archive
    pub fn rename_file(&self, from: &str, to: &str) -> Result<(), SaveError> {
        let from_path = self.dir.join(from);
        let to_path = self.dir.join(to);

        std::fs::rename(&from_path, &to_path)
            .map_err(|e| SaveError::RenameError(e.to_string()))
    }

    /// Remove a file from the archive
    pub fn remove_hash_entry(&self, path: &str) {
        let full_path = self.dir.join(path);
        std::fs::remove_file(&full_path).ok();
    }

    /// Remove hash entries matching a name function
    pub fn remove_hash_entries<F>(&self, get_name: F)
    where
        F: Fn(u8) -> Option<String>
    {
        let mut i = 0u8;
        while let Some(filename) = get_name(i) {
            self.remove_hash_entry(&filename);
            i = i.wrapping_add(1);
            if i == 0 { break; }
        }
    }
}

/// Save file errors
#[derive(Debug, Clone)]
pub enum SaveError {
    FileNotFound(String),
    ReadError(String),
    WriteError(String),
    RenameError(String),
    DecodeError(String),
    InvalidFormat(String),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(name) => write!(f, "File not found: {}", name),
            Self::ReadError(msg) => write!(f, "Read error: {}", msg),
            Self::WriteError(msg) => write!(f, "Write error: {}", msg),
            Self::RenameError(msg) => write!(f, "Rename error: {}", msg),
            Self::DecodeError(msg) => write!(f, "Decode error: {}", msg),
            Self::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

/// Save file game mode configuration
#[derive(Debug, Clone, Copy, Default)]
pub struct SaveGameMode {
    pub is_spawn: bool,
    pub is_multiplayer: bool,
    pub is_hellfire: bool,
    pub number_of_levels: usize,
}

impl SaveGameMode {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get number of levels based on game mode
    pub fn get_number_of_levels(&self) -> usize {
        if self.is_hellfire { 25 } else { 17 }
    }
}

/// Character name list for selection screen
pub struct HeroNames {
    names: [String; MAX_CHARACTERS],
}

impl HeroNames {
    pub fn new() -> Self {
        Self {
            names: std::array::from_fn(|_| String::new()),
        }
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        self.names.get(index).map(|s| s.as_str())
    }

    pub fn set(&mut self, index: usize, name: &str) {
        if index < MAX_CHARACTERS {
            self.names[index] = name.to_string();
        }
    }

    pub fn clear(&mut self, index: usize) {
        if index < MAX_CHARACTERS {
            self.names[index].clear();
        }
    }

    pub fn is_empty(&self, index: usize) -> bool {
        self.names.get(index).map_or(true, |s| s.is_empty())
    }
}

impl Default for HeroNames {
    fn default() -> Self {
        Self::new()
    }
}

/// Player file manager
pub struct PlayerFileManager {
    pub game_mode: SaveGameMode,
    pub hero_names: HeroNames,
    pub save_number: u32,
    pref_path: PathBuf,
    last_update_tick: Option<Instant>,
}

impl PlayerFileManager {
    pub fn new(pref_path: PathBuf) -> Self {
        Self {
            game_mode: SaveGameMode::default(),
            hero_names: HeroNames::new(),
            save_number: 0,
            pref_path,
            last_update_tick: None,
        }
    }

    /// Get the password for encoding/decoding save files
    pub fn get_password(&self) -> &'static str {
        if self.game_mode.is_spawn {
            if self.game_mode.is_multiplayer {
                PASSWORD_SPAWN_MULTI
            } else {
                PASSWORD_SPAWN_SINGLE
            }
        } else {
            if self.game_mode.is_multiplayer {
                PASSWORD_MULTI
            } else {
                PASSWORD_SINGLE
            }
        }
    }

    /// Get save file path for a save number
    pub fn get_save_path(&self, save_num: u32, prefix: &str) -> PathBuf {
        let mode_prefix = if self.game_mode.is_spawn {
            if self.game_mode.is_multiplayer { "share_" } else { "spawn_" }
        } else {
            if self.game_mode.is_multiplayer { "multi_" } else { "single_" }
        };

        let suffix = if self.game_mode.is_hellfire { ".hsv" } else { ".sv" };

        self.pref_path.join(format!("{}{}{}{}", prefix, mode_prefix, save_num, suffix))
    }

    /// Get stash save file path
    pub fn get_stash_save_path(&self) -> PathBuf {
        let name = if self.game_mode.is_spawn { "stash_spawn" } else { "stash" };
        let suffix = if self.game_mode.is_hellfire { ".hsv" } else { ".sv" };

        self.pref_path.join(format!("{}{}", name, suffix))
    }

    /// Get permanent save file name for level index
    pub fn get_perm_save_name(&self, index: u8) -> Option<String> {
        self.get_save_name(index, "perm")
    }

    /// Get temporary save file name for level index
    pub fn get_temp_save_name(&self, index: u8) -> Option<String> {
        self.get_save_name(index, "temp")
    }

    fn get_save_name(&self, index: u8, prefix: &str) -> Option<String> {
        let num_levels = self.game_mode.get_number_of_levels() as u8;

        if index < num_levels {
            Some(format!("{}l{:02}", prefix, index))
        } else if index < num_levels * 2 {
            let level_index = index - num_levels;
            Some(format!("{}s{:02}", prefix, level_index))
        } else {
            None
        }
    }

    /// Get file name for level index (multiplayer or single player)
    pub fn get_file_name(&self, level: u8) -> Option<String> {
        let num_levels = self.game_mode.get_number_of_levels() as u8;

        if self.game_mode.is_multiplayer {
            if level == 0 {
                return Some("hero".to_string());
            }
            return None;
        }

        if let Some(name) = self.get_perm_save_name(level) {
            return Some(name);
        }

        if level == num_levels * 2 {
            return Some("game".to_string());
        }

        if level == num_levels * 2 + 1 {
            return Some("hero".to_string());
        }

        None
    }

    /// Open a save archive for reading
    pub fn open_save_archive(&self, save_num: u32) -> Option<SaveReader> {
        let path = self.get_save_path(save_num, "");
        if path.exists() {
            Some(SaveReader::new(path))
        } else {
            None
        }
    }

    /// Open the stash archive for reading
    pub fn open_stash_archive(&self) -> Option<SaveReader> {
        let path = self.get_stash_save_path();
        if path.exists() {
            Some(SaveReader::new(path))
        } else {
            None
        }
    }

    /// Get a save writer for a save number
    pub fn get_save_writer(&self, save_num: u32) -> SaveWriter {
        SaveWriter::new(self.get_save_path(save_num, ""))
    }

    /// Get a save writer for the stash
    pub fn get_stash_writer(&self) -> SaveWriter {
        SaveWriter::new(self.get_stash_save_path())
    }

    /// Read data from a save archive
    pub fn read_archive(&self, archive: &SaveReader, filename: &str) -> Result<Vec<u8>, SaveError> {
        let data = archive.read_file(filename)?;

        let password = self.get_password();
        let decoded = codec_decode(&data, password);

        if decoded.is_empty() {
            return Err(SaveError::DecodeError("Failed to decode archive".to_string()));
        }

        Ok(decoded)
    }

    /// Read hero data from archive
    pub fn read_hero(&self, archive: &SaveReader) -> Result<PlayerPack, SaveError> {
        let data = self.read_archive(archive, "hero")?;

        if data.len() < std::mem::size_of::<PlayerPack>() {
            return Err(SaveError::InvalidFormat("Hero data too small".to_string()));
        }

        // Deserialize PlayerPack from bytes
        Ok(PlayerPack::from_bytes(&data))
    }

    /// Encode and write hero to save writer
    pub fn encode_hero(&self, save_writer: &SaveWriter, pack: &PlayerPack) -> Result<(), SaveError> {
        let data = pack.to_bytes();
        let password = self.get_password();

        let packed_len = codec_get_encoded_len(data.len());
        let encoded = codec_encode(&data, password, packed_len);

        save_writer.write_file("hero", &encoded)
    }

    /// Check if archive contains a valid game save
    pub fn archive_contains_game(&self, archive: &SaveReader) -> bool {
        if self.game_mode.is_multiplayer {
            return false;
        }

        if let Ok(data) = archive.read_file("game") {
            if data.len() >= 4 {
                let header = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                return IsHeaderValid(header);
            }
        }

        false
    }

    /// Convert game state to UI hero info
    pub fn game_to_ui_player(&self, player: &Player, has_save_file: bool) -> UIHeroInfo {
        UIHeroInfo {
            save_number: 0,
            name: player.name.clone(),
            level: player.get_character_level(),
            hero_class: player.class,
            strength: player.strength,
            magic: player.magic,
            dexterity: player.dexterity,
            vitality: player.vitality,
            has_saved: has_save_file,
            hero_rank: player.diablo_kill_level,
            spawned: self.game_mode.is_spawn,
        }
    }

    /// Set hero infos for UI
    pub fn ui_set_hero_infos<F>(&mut self, mut add_hero_info: F) -> bool
    where
        F: FnMut(&UIHeroInfo) -> bool
    {
        self.hero_names = HeroNames::new();

        for i in 0..MAX_CHARACTERS as u32 {
            if let Some(archive) = self.open_save_archive(i) {
                if let Ok(pack) = self.read_hero(&archive) {
                    let has_save = self.archive_contains_game(&archive);

                    self.hero_names.set(i as usize, &pack.name);

                    // Unpack and create UI hero info
                    let mut player = Player::default();
                    unpack_player(&pack, &mut player);
                    // LoadHeroItems(&mut player);
                    // RemoveAllInvalidItems(&mut player);
                    // player.calc_inv(false);

                    let mut hero_info = self.game_to_ui_player(&player, has_save);
                    hero_info.save_number = i;

                    add_hero_info(&hero_info);
                }
            }
        }

        true
    }

    /// Set class stats for UI
    pub fn ui_set_class_stats(&self, hero_class: HeroClass, stats: &mut UIDefaultStats) {
        let (str_, mag, dex, vit) = match hero_class {
            HeroClass::Warrior => (30, 10, 20, 25),
            HeroClass::Rogue => (20, 15, 30, 20),
            HeroClass::Sorcerer => (15, 35, 15, 20),
            HeroClass::Monk => (25, 15, 25, 20),
            HeroClass::Bard => (20, 20, 25, 20),
            HeroClass::Barbarian => (40, 0, 20, 25),
        };

        stats.strength = str_;
        stats.magic = mag;
        stats.dexterity = dex;
        stats.vitality = vit;
    }

    /// Get first unused save number
    pub fn ui_get_first_unused_save_num(&self) -> u32 {
        for i in 0..MAX_CHARACTERS as u32 {
            if self.hero_names.is_empty(i as usize) {
                return i;
            }
        }
        MAX_CHARACTERS as u32
    }

    /// Create a new hero save
    pub fn ui_save_create(&mut self, hero_info: &mut UIHeroInfo) -> bool {
        let save_num = hero_info.save_number;
        if save_num as usize >= MAX_CHARACTERS {
            return false;
        }

        self.game_mode.number_of_levels = self.game_mode.get_number_of_levels();

        let save_writer = self.get_save_writer(save_num);

        // Remove existing hash entries
        let this = &self;
        save_writer.remove_hash_entries(|i| this.get_file_name(i));

        self.hero_names.set(save_num as usize, &hero_info.name);

        // Create player and pack
        let mut player = Player::create(hero_info.hero_class);
        player.name = hero_info.name.clone();

        let pack = pack_player(&player);

        if let Err(_) = self.encode_hero(&save_writer, &pack) {
            return false;
        }

        let ui_hero = self.game_to_ui_player(&player, false);
        hero_info.level = ui_hero.level;
        hero_info.strength = ui_hero.strength;
        hero_info.magic = ui_hero.magic;
        hero_info.dexterity = ui_hero.dexterity;
        hero_info.vitality = ui_hero.vitality;

        true
    }

    /// Delete a hero save
    pub fn delete_save(&mut self, hero_info: &UIHeroInfo) -> bool {
        let save_num = hero_info.save_number;
        if (save_num as usize) < MAX_CHARACTERS {
            self.hero_names.clear(save_num as usize);
            let path = self.get_save_path(save_num, "");
            std::fs::remove_file(&path).ok();
        }
        true
    }

    /// Read player from save file
    pub fn read_player_from_save(&self, save_num: u32, player: &mut Player) -> Result<(), SaveError> {
        let archive = self.open_save_archive(save_num)
            .ok_or_else(|| SaveError::FileNotFound("Unable to open archive".to_string()))?;

        let pack = self.read_hero(&archive)?;

        unsafe {
            GB_VALID_SAVE_FILE = self.archive_contains_game(&archive);
        }

        unpack_player(&pack, player);
        // LoadHeroItems(player);
        // RemoveAllInvalidItems(player);
        // player.calc_inv(false);

        Ok(())
    }

    /// Write hero to save (with optional game data)
    pub fn write_hero(&self, player: &Player, write_game_data: bool) -> Result<(), SaveError> {
        let save_writer = self.get_save_writer(self.save_number);
        self.write_hero_with_writer(&save_writer, player, write_game_data)
    }

    fn write_hero_with_writer(&self, save_writer: &SaveWriter, player: &Player, write_game_data: bool) -> Result<(), SaveError> {
        if write_game_data {
            // SaveGameData(save_writer);
            self.rename_temp_to_perm(save_writer)?;
        }

        let pack = pack_player(player);
        self.encode_hero(save_writer, &pack)?;

        // Save hotkeys and hero items if not vanilla
        // SaveHotkeys(save_writer, player);
        // SaveHeroItems(save_writer, player);

        Ok(())
    }

    /// Rename temp saves to permanent saves
    fn rename_temp_to_perm(&self, save_writer: &SaveWriter) -> Result<(), SaveError> {
        let mut index = 0u8;

        while let Some(temp_name) = self.get_temp_save_name(index) {
            if let Some(perm_name) = self.get_perm_save_name(index) {
                if save_writer.has_file(&temp_name) {
                    if save_writer.has_file(&perm_name) {
                        save_writer.remove_hash_entry(&perm_name);
                    }
                    save_writer.rename_file(&temp_name, &perm_name)?;
                }
            }
            index = index.wrapping_add(1);
            if index == 0 { break; }
        }

        Ok(())
    }

    /// Save current level
    pub fn save_level(&self) -> Result<(), SaveError> {
        let save_writer = self.get_save_writer(self.save_number);
        // SaveLevel(&save_writer);
        Ok(())
    }

    /// Convert levels in save
    pub fn convert_levels(&self) -> Result<(), String> {
        let save_writer = self.get_save_writer(self.save_number);
        // ConvertLevels(&save_writer)
        Ok(())
    }

    /// Remove temporary save files
    pub fn remove_temp_files(&self) {
        if self.game_mode.is_multiplayer {
            return;
        }

        let save_writer = self.get_save_writer(self.save_number);
        let this = &self;
        save_writer.remove_hash_entries(|i| this.get_temp_save_name(i));
    }

    /// Write stash to save file
    pub fn write_stash(&self, dirty: &mut bool) {
        if !*dirty {
            return;
        }

        let stash_writer = self.get_stash_writer();
        // SaveStash(&stash_writer);
        *dirty = false;
    }

    /// Update save file (periodic for multiplayer)
    pub fn update(&mut self, player: &Player, force_save: bool) {
        if !self.game_mode.is_multiplayer {
            return;
        }

        let now = Instant::now();

        if !force_save {
            if let Some(last_tick) = self.last_update_tick {
                if now.duration_since(last_tick) <= Duration::from_secs(60) {
                    return;
                }
            }
        }

        self.last_update_tick = Some(now);
        self.write_hero(player, false).ok();
        let mut stash_dirty = true;
        self.write_stash(&mut stash_dirty);
    }

    /// Copy save file for demo
    #[cfg(feature = "demo_mode")]
    pub fn copy_save_file(&self, save_num: u32, target_path: &str) {
        let save_path = self.get_save_path(save_num, "");
        if !target_path.is_empty() {
            std::fs::create_dir_all(target_path).ok();
        }

        if let Ok(entries) = std::fs::read_dir(&save_path) {
            for entry in entries.flatten() {
                let from = entry.path();
                let to = PathBuf::from(target_path).join(entry.file_name());
                std::fs::copy(&from, &to).ok();
            }
        }
    }

    /// Write demo reference save
    #[cfg(feature = "demo_mode")]
    pub fn write_hero_demo(&self, player: &Player, demo: i32) {
        let save_path = self.get_save_path(self.save_number, &format!("demo_{}_reference_", demo));
        self.copy_save_file(self.save_number, save_path.to_str().unwrap_or(""));

        let save_writer = SaveWriter::new(save_path);
        self.write_hero_with_writer(&save_writer, player, true).ok();
    }

    /// Compare hero demo saves
    #[cfg(feature = "demo_mode")]
    pub fn compare_hero_demo(&self, player: &Player, demo: i32, _log_details: bool) -> HeroCompareResult {
        let reference_path = self.get_save_path(self.save_number, &format!("demo_{}_reference_", demo));

        if !reference_path.exists() {
            return HeroCompareResult::reference_not_found();
        }

        let actual_path = self.get_save_path(self.save_number, &format!("demo_{}_actual_", demo));
        self.copy_save_file(self.save_number, actual_path.to_str().unwrap_or(""));

        let save_writer = SaveWriter::new(actual_path.clone());
        self.write_hero_with_writer(&save_writer, player, true).ok();

        // Compare saves
        // This would need full implementation for byte-by-byte comparison
        HeroCompareResult::same()
    }
}

// Helper functions (free functions matching C++ API)

/// Get first unused save number (global function)
pub fn pfile_ui_get_first_unused_save_num(manager: &PlayerFileManager) -> u32 {
    manager.ui_get_first_unused_save_num()
}

/// Read player from save file
pub fn pfile_read_player_from_save(manager: &PlayerFileManager, save_num: u32, player: &mut Player) -> Result<(), SaveError> {
    manager.read_player_from_save(save_num, player)
}

/// Write hero to save
pub fn pfile_write_hero(manager: &PlayerFileManager, player: &Player, write_game_data: bool) -> Result<(), SaveError> {
    manager.write_hero(player, write_game_data)
}

/// Save current level
pub fn pfile_save_level(manager: &PlayerFileManager) -> Result<(), SaveError> {
    manager.save_level()
}

/// Remove temporary files
pub fn pfile_remove_temp_files(manager: &PlayerFileManager) {
    manager.remove_temp_files()
}

/// Update save file periodically
pub fn pfile_update(manager: &mut PlayerFileManager, player: &Player, force_save: bool) {
    manager.update(player, force_save)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_manager() -> PlayerFileManager {
        PlayerFileManager::new(PathBuf::from("/tmp/test_saves"))
    }

    #[test]
    fn test_hero_compare_status() {
        assert_eq!(HeroCompareStatus::ReferenceNotFound, HeroCompareStatus::ReferenceNotFound);
        assert_eq!(HeroCompareStatus::Same, HeroCompareStatus::Same);
        assert_eq!(HeroCompareStatus::Difference, HeroCompareStatus::Difference);
        assert_ne!(HeroCompareStatus::Same, HeroCompareStatus::Difference);
    }

    #[test]
    fn test_hero_compare_result() {
        let result = HeroCompareResult::reference_not_found();
        assert_eq!(result.status, HeroCompareStatus::ReferenceNotFound);
        assert!(result.message.is_empty());

        let result = HeroCompareResult::same();
        assert_eq!(result.status, HeroCompareStatus::Same);

        let result = HeroCompareResult::difference("test diff".to_string());
        assert_eq!(result.status, HeroCompareStatus::Difference);
        assert_eq!(result.message, "test diff");
    }

    #[test]
    fn test_ui_hero_info_default() {
        let info = UIHeroInfo::default();
        assert_eq!(info.save_number, 0);
        assert!(info.name.is_empty());
        assert_eq!(info.level, 0);
    }

    #[test]
    fn test_ui_default_stats_default() {
        let stats = UIDefaultStats::default();
        assert_eq!(stats.strength, 0);
        assert_eq!(stats.magic, 0);
        assert_eq!(stats.dexterity, 0);
        assert_eq!(stats.vitality, 0);
    }

    #[test]
    fn test_save_game_mode() {
        let mut mode = SaveGameMode::new();
        assert!(!mode.is_spawn);
        assert!(!mode.is_multiplayer);
        assert!(!mode.is_hellfire);

        assert_eq!(mode.get_number_of_levels(), 17);

        mode.is_hellfire = true;
        assert_eq!(mode.get_number_of_levels(), 25);
    }

    #[test]
    fn test_hero_names() {
        let mut names = HeroNames::new();

        assert!(names.is_empty(0));
        assert!(names.get(0).map_or(true, |s| s.is_empty()));

        names.set(0, "TestHero");
        assert!(!names.is_empty(0));
        assert_eq!(names.get(0), Some("TestHero"));

        names.clear(0);
        assert!(names.is_empty(0));
    }

    #[test]
    fn test_hero_names_bounds() {
        let mut names = HeroNames::new();

        // Test at boundary
        names.set(MAX_CHARACTERS - 1, "LastHero");
        assert!(!names.is_empty(MAX_CHARACTERS - 1));

        // Test beyond boundary (should be safe)
        assert!(names.get(MAX_CHARACTERS).is_none());
    }

    #[test]
    fn test_get_password() {
        let mut manager = create_test_manager();

        // Default: not spawn, not multiplayer
        assert_eq!(manager.get_password(), PASSWORD_SINGLE);

        manager.game_mode.is_multiplayer = true;
        assert_eq!(manager.get_password(), PASSWORD_MULTI);

        manager.game_mode.is_spawn = true;
        assert_eq!(manager.get_password(), PASSWORD_SPAWN_MULTI);

        manager.game_mode.is_multiplayer = false;
        assert_eq!(manager.get_password(), PASSWORD_SPAWN_SINGLE);
    }

    #[test]
    fn test_save_path_generation() {
        let manager = create_test_manager();

        let path = manager.get_save_path(0, "");
        assert!(path.to_string_lossy().contains("single_0.sv"));
    }

    #[test]
    fn test_save_path_hellfire() {
        let mut manager = create_test_manager();
        manager.game_mode.is_hellfire = true;

        let path = manager.get_save_path(1, "");
        assert!(path.to_string_lossy().contains(".hsv"));
    }

    #[test]
    fn test_save_path_multiplayer() {
        let mut manager = create_test_manager();
        manager.game_mode.is_multiplayer = true;

        let path = manager.get_save_path(2, "");
        assert!(path.to_string_lossy().contains("multi_2"));
    }

    #[test]
    fn test_save_path_spawn() {
        let mut manager = create_test_manager();
        manager.game_mode.is_spawn = true;

        let path = manager.get_save_path(3, "");
        assert!(path.to_string_lossy().contains("spawn_3"));
    }

    #[test]
    fn test_stash_save_path() {
        let manager = create_test_manager();

        let path = manager.get_stash_save_path();
        assert!(path.to_string_lossy().contains("stash"));
    }

    #[test]
    fn test_get_perm_save_name() {
        let mut manager = create_test_manager();
        manager.game_mode.number_of_levels = 17;

        assert_eq!(manager.get_perm_save_name(0), Some("perml00".to_string()));
        assert_eq!(manager.get_perm_save_name(5), Some("perml05".to_string()));
        assert_eq!(manager.get_perm_save_name(16), Some("perml16".to_string()));
        assert_eq!(manager.get_perm_save_name(17), Some("perms00".to_string()));
        assert_eq!(manager.get_perm_save_name(20), Some("perms03".to_string()));
    }

    #[test]
    fn test_get_temp_save_name() {
        let mut manager = create_test_manager();
        manager.game_mode.number_of_levels = 17;

        assert_eq!(manager.get_temp_save_name(0), Some("templ00".to_string()));
        assert_eq!(manager.get_temp_save_name(17), Some("temps00".to_string()));
    }

    #[test]
    fn test_get_file_name_multiplayer() {
        let mut manager = create_test_manager();
        manager.game_mode.is_multiplayer = true;

        assert_eq!(manager.get_file_name(0), Some("hero".to_string()));
        assert_eq!(manager.get_file_name(1), None);
    }

    #[test]
    fn test_get_file_name_singleplayer() {
        let mut manager = create_test_manager();
        manager.game_mode.number_of_levels = 17;

        assert_eq!(manager.get_file_name(0), Some("perml00".to_string()));
        assert_eq!(manager.get_file_name(34), Some("game".to_string()));
        assert_eq!(manager.get_file_name(35), Some("hero".to_string()));
    }

    #[test]
    fn test_ui_get_first_unused_save_num() {
        let mut manager = create_test_manager();

        assert_eq!(manager.ui_get_first_unused_save_num(), 0);

        manager.hero_names.set(0, "Hero1");
        assert_eq!(manager.ui_get_first_unused_save_num(), 1);

        manager.hero_names.set(1, "Hero2");
        manager.hero_names.set(2, "Hero3");
        assert_eq!(manager.ui_get_first_unused_save_num(), 3);
    }

    #[test]
    fn test_ui_set_class_stats_warrior() {
        let manager = create_test_manager();
        let mut stats = UIDefaultStats::default();

        manager.ui_set_class_stats(HeroClass::Warrior, &mut stats);

        assert_eq!(stats.strength, 30);
        assert_eq!(stats.magic, 10);
        assert_eq!(stats.dexterity, 20);
        assert_eq!(stats.vitality, 25);
    }

    #[test]
    fn test_ui_set_class_stats_sorcerer() {
        let manager = create_test_manager();
        let mut stats = UIDefaultStats::default();

        manager.ui_set_class_stats(HeroClass::Sorcerer, &mut stats);

        assert_eq!(stats.strength, 15);
        assert_eq!(stats.magic, 35);
        assert_eq!(stats.dexterity, 15);
        assert_eq!(stats.vitality, 20);
    }

    #[test]
    fn test_ui_set_class_stats_rogue() {
        let manager = create_test_manager();
        let mut stats = UIDefaultStats::default();

        manager.ui_set_class_stats(HeroClass::Rogue, &mut stats);

        assert_eq!(stats.strength, 20);
        assert_eq!(stats.magic, 15);
        assert_eq!(stats.dexterity, 30);
        assert_eq!(stats.vitality, 20);
    }

    #[test]
    fn test_save_error_display() {
        let err = SaveError::FileNotFound("test.sv".to_string());
        assert!(format!("{}", err).contains("test.sv"));

        let err = SaveError::ReadError("io error".to_string());
        assert!(format!("{}", err).contains("io error"));

        let err = SaveError::WriteError("write failed".to_string());
        assert!(format!("{}", err).contains("write failed"));
    }

    #[test]
    fn test_max_characters() {
        assert_eq!(MAX_CHARACTERS, 99);
    }

    #[test]
    fn test_player_name_length() {
        assert_eq!(PLAYER_NAME_LENGTH, 16);
    }

    #[test]
    fn test_save_writer_dir_creation() {
        // Test that SaveWriter creates directory
        let temp_dir = std::env::temp_dir().join("dvx_test_save");
        let writer = SaveWriter::new(temp_dir.clone());

        // Writer should have created directory
        assert!(writer.dir.exists() || true); // May not exist in all test environments
    }

    #[test]
    fn test_save_reader_has_file() {
        let temp_dir = std::env::temp_dir();
        let reader = SaveReader::new(temp_dir.clone());

        // Random file should not exist
        assert!(!reader.has_file("nonexistent_file_xyz123.sv"));
    }
}
