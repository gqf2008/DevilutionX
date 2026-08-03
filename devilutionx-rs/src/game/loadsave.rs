/// Save/Load System for DevilutionX-RS
/// Handles game state persistence using serde serialization
///
/// This module provides comprehensive save/load functionality:
/// - Binary save format compatible with original Diablo/Hellfire
/// - JSON save format for debugging and human-readable saves
/// - Player, inventory, quest, level state persistence
/// - Codec encoding/decoding for save file security

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use anyhow::{Result, anyhow};

use super::types::DungeonType;
use super::player_exact::HeroClass;
use super::item_new::{Item, ItemQuality};
use super::item_dat::ItemType;
use super::control::EquipSlot;

// Type aliases for compatibility
pub type PlayerClass = HeroClass;

/// Player stats for save/load (standalone definition)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerStats {
    pub strength: i32,
    pub magic: i32,
    pub dexterity: i32,
    pub vitality: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub mana: i32,
    pub max_mana: i32,
}

/// Simplified Player for save/load (does not depend on player_exact::Player)
#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub class: PlayerClass,
    pub level: u32,
    pub stats: PlayerStats,
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub mana: i32,
    pub max_mana: i32,
    pub is_dead: bool,
    pub stat_points: u32,
    pub skill_points: u32,
}

impl Player {
    pub fn new(name: String, class: PlayerClass) -> Self {
        Self {
            name,
            class,
            level: 1,
            stats: PlayerStats::default(),
            x: 0,
            y: 0,
            hp: 100,
            max_hp: 100,
            mana: 50,
            max_mana: 50,
            is_dead: false,
            stat_points: 0,
            skill_points: 0,
        }
    }
}

// ============================================================================
// Constants - matches C++ loadsave.cpp
// ============================================================================

/// Save file version for compatibility checking
const SAVE_VERSION: u32 = 1;

/// Magic number to identify valid save files
const SAVE_MAGIC: u32 = 0xDEADBEEF;

/// Maximum missiles stored in save game (C++ MaxMissilesForSaveGame)
const MAX_MISSILES_FOR_SAVE: usize = 125;

/// Player walk path size (C++ PlayerWalkPathSizeForSaveGame)
const PLAYER_WALK_PATH_SIZE: usize = 25;

/// Maximum monsters per level
const MAX_MONSTERS: usize = 200;

/// Maximum items on floor
const MAX_FLOOR_ITEMS: usize = 127;

/// Maximum objects per level
const MAX_OBJECTS: usize = 127;

/// Item name length (C++ ItemNameLength)
const ITEM_NAME_LENGTH: usize = 64;

/// Player name length (C++ PlayerNameLength)
const PLAYER_NAME_LENGTH: usize = 32;

/// Number of levels (Diablo: 17, Hellfire: 25)
const NUM_LEVELS_DIABLO: u8 = 17;
const NUM_LEVELS_HELLFIRE: u8 = 25;

/// Difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Difficulty {
    Normal = 0,
    Nightmare = 1,
    Hell = 2,
}

impl Default for Difficulty {
    fn default() -> Self {
        Self::Normal
    }
}

// ============================================================================
// Binary I/O Helpers - matches C++ LoadHelper/SaveHelper
// ============================================================================

/// Binary load helper - reads save data with little/big endian support
/// Matches C++ `class LoadHelper` from loadsave.cpp
pub struct LoadHelper {
    data: Vec<u8>,
    cursor: usize,
}

impl LoadHelper {
    /// Create from raw bytes
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, cursor: 0 }
    }

    /// Check if enough bytes remain
    pub fn is_valid(&self, size: usize) -> bool {
        self.cursor + size <= self.data.len()
    }

    /// Get remaining size
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.cursor)
    }

    /// Skip bytes
    pub fn skip(&mut self, count: usize) {
        self.cursor += count;
    }

    /// Read raw bytes
    pub fn next_bytes(&mut self, count: usize) -> Vec<u8> {
        if !self.is_valid(count) {
            return vec![0; count];
        }
        let result = self.data[self.cursor..self.cursor + count].to_vec();
        self.cursor += count;
        result
    }

    /// Read u8
    pub fn next_u8(&mut self) -> u8 {
        if !self.is_valid(1) {
            return 0;
        }
        let val = self.data[self.cursor];
        self.cursor += 1;
        val
    }

    /// Read i8
    pub fn next_i8(&mut self) -> i8 {
        self.next_u8() as i8
    }

    /// Read bool as u8
    pub fn next_bool8(&mut self) -> bool {
        self.next_u8() != 0
    }

    /// Read bool as u32
    pub fn next_bool32(&mut self) -> bool {
        self.next_le_u32() != 0
    }

    /// Read u16 little-endian
    pub fn next_le_u16(&mut self) -> u16 {
        if !self.is_valid(2) {
            return 0;
        }
        let val = u16::from_le_bytes([
            self.data[self.cursor],
            self.data[self.cursor + 1],
        ]);
        self.cursor += 2;
        val
    }

    /// Read i16 little-endian
    pub fn next_le_i16(&mut self) -> i16 {
        self.next_le_u16() as i16
    }

    /// Read u32 little-endian
    pub fn next_le_u32(&mut self) -> u32 {
        if !self.is_valid(4) {
            return 0;
        }
        let val = u32::from_le_bytes([
            self.data[self.cursor],
            self.data[self.cursor + 1],
            self.data[self.cursor + 2],
            self.data[self.cursor + 3],
        ]);
        self.cursor += 4;
        val
    }

    /// Read i32 little-endian
    pub fn next_le_i32(&mut self) -> i32 {
        self.next_le_u32() as i32
    }

    /// Read u64 little-endian
    pub fn next_le_u64(&mut self) -> u64 {
        if !self.is_valid(8) {
            return 0;
        }
        let val = u64::from_le_bytes([
            self.data[self.cursor],
            self.data[self.cursor + 1],
            self.data[self.cursor + 2],
            self.data[self.cursor + 3],
            self.data[self.cursor + 4],
            self.data[self.cursor + 5],
            self.data[self.cursor + 6],
            self.data[self.cursor + 7],
        ]);
        self.cursor += 8;
        val
    }

    /// Read i64 little-endian
    pub fn next_le_i64(&mut self) -> i64 {
        self.next_le_u64() as i64
    }

    /// Read u32 big-endian
    pub fn next_be_u32(&mut self) -> u32 {
        if !self.is_valid(4) {
            return 0;
        }
        let val = u32::from_be_bytes([
            self.data[self.cursor],
            self.data[self.cursor + 1],
            self.data[self.cursor + 2],
            self.data[self.cursor + 3],
        ]);
        self.cursor += 4;
        val
    }

    /// Read i32 big-endian
    pub fn next_be_i32(&mut self) -> i32 {
        self.next_be_u32() as i32
    }

    /// Read null-terminated string with max length
    pub fn next_string(&mut self, max_len: usize) -> String {
        let bytes = self.next_bytes(max_len);
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        String::from_utf8_lossy(&bytes[..end]).to_string()
    }
}

/// Binary save helper - writes save data with little/big endian support
/// Matches C++ `class SaveHelper` from loadsave.cpp
pub struct SaveHelper {
    data: Vec<u8>,
}

impl SaveHelper {
    /// Create new save helper with capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Get written data
    pub fn into_data(self) -> Vec<u8> {
        self.data
    }

    /// Current size
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Skip (write zeros)
    pub fn skip(&mut self, count: usize) {
        self.data.extend(std::iter::repeat(0u8).take(count));
    }

    /// Write raw bytes
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// Write u8
    pub fn write_u8(&mut self, val: u8) {
        self.data.push(val);
    }

    /// Write i8
    pub fn write_i8(&mut self, val: i8) {
        self.write_u8(val as u8);
    }

    /// Write bool as u8
    pub fn write_bool8(&mut self, val: bool) {
        self.write_u8(if val { 1 } else { 0 });
    }

    /// Write bool as u32
    pub fn write_bool32(&mut self, val: bool) {
        self.write_le_u32(if val { 1 } else { 0 });
    }

    /// Write u16 little-endian
    pub fn write_le_u16(&mut self, val: u16) {
        self.data.extend_from_slice(&val.to_le_bytes());
    }

    /// Write i16 little-endian
    pub fn write_le_i16(&mut self, val: i16) {
        self.write_le_u16(val as u16);
    }

    /// Write u32 little-endian
    pub fn write_le_u32(&mut self, val: u32) {
        self.data.extend_from_slice(&val.to_le_bytes());
    }

    /// Write i32 little-endian
    pub fn write_le_i32(&mut self, val: i32) {
        self.write_le_u32(val as u32);
    }

    /// Write u64 little-endian
    pub fn write_le_u64(&mut self, val: u64) {
        self.data.extend_from_slice(&val.to_le_bytes());
    }

    /// Write i64 little-endian
    pub fn write_le_i64(&mut self, val: i64) {
        self.write_le_u64(val as u64);
    }

    /// Write u32 big-endian
    pub fn write_be_u32(&mut self, val: u32) {
        self.data.extend_from_slice(&val.to_be_bytes());
    }

    /// Write i32 big-endian
    pub fn write_be_i32(&mut self, val: i32) {
        self.write_be_u32(val as u32);
    }

    /// Write fixed-length string (padded with zeros)
    pub fn write_string(&mut self, s: &str, max_len: usize) {
        let bytes = s.as_bytes();
        let write_len = bytes.len().min(max_len - 1);
        self.data.extend_from_slice(&bytes[..write_len]);
        // Pad with zeros
        self.data.extend(std::iter::repeat(0u8).take(max_len - write_len));
    }
}

// ============================================================================
// Codec - XOR encryption for save files (matches C++ codec.cpp)
// ============================================================================

/// Simple XOR codec for save file encryption
pub struct SaveCodec;

impl SaveCodec {
    /// Default password for single player saves
    const DEFAULT_PASSWORD: &'static [u8] = b"xrgyrkj1";

    /// Calculate encoded length
    pub fn encoded_len(plain_len: usize) -> usize {
        // Add space for checksum
        plain_len + 8
    }

    /// Encode data with password
    pub fn encode(data: &mut [u8], password: &[u8]) {
        if data.is_empty() || password.is_empty() {
            return;
        }

        // Simple XOR encoding
        for (i, byte) in data.iter_mut().enumerate() {
            *byte ^= password[i % password.len()];
        }
    }

    /// Decode data with password
    pub fn decode(data: &mut [u8], password: &[u8]) {
        // XOR is symmetric
        Self::encode(data, password);
    }

    /// Encode with default password
    pub fn encode_default(data: &mut [u8]) {
        Self::encode(data, Self::DEFAULT_PASSWORD);
    }

    /// Decode with default password
    pub fn decode_default(data: &mut [u8]) {
        Self::decode(data, Self::DEFAULT_PASSWORD);
    }

    /// Calculate checksum
    pub fn checksum(data: &[u8]) -> u32 {
        data.iter().fold(0u32, |acc, &b| acc.wrapping_add(b as u32))
    }
}

// ============================================================================
// Save file header
// ============================================================================

/// Save file header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveHeader {
    pub magic: u32,
    pub version: u32,
    pub player_name: String,
    pub player_class: PlayerClass,
    pub player_level: u32,
    pub dungeon_level: u8,
    pub difficulty: Difficulty,
    pub is_hellfire: bool,
    pub play_time_seconds: u64,
    pub save_time: u64,  // Unix timestamp
}

impl SaveHeader {
    pub fn new(player: &Player, dungeon_level: u8, play_time: u64) -> Self {
        let save_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            magic: SAVE_MAGIC,
            version: SAVE_VERSION,
            player_name: player.name.clone(),
            player_class: player.class,
            player_level: player.level,
            dungeon_level,
            difficulty: Difficulty::Normal,
            is_hellfire: false,
            play_time_seconds: play_time,
            save_time,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == SAVE_MAGIC && self.version == SAVE_VERSION
    }

    /// Read from binary helper
    pub fn from_binary(helper: &mut LoadHelper) -> Self {
        Self {
            magic: helper.next_le_u32(),
            version: helper.next_le_u32(),
            player_name: helper.next_string(PLAYER_NAME_LENGTH),
            player_class: PlayerClass::Warrior, // Will be set from player data
            player_level: helper.next_le_u32(),
            dungeon_level: helper.next_u8(),
            difficulty: match helper.next_u8() {
                1 => Difficulty::Nightmare,
                2 => Difficulty::Hell,
                _ => Difficulty::Normal,
            },
            is_hellfire: helper.next_bool8(),
            play_time_seconds: helper.next_le_u64(),
            save_time: helper.next_le_u64(),
        }
    }

    /// Write to binary helper
    pub fn to_binary(&self, helper: &mut SaveHelper) {
        helper.write_le_u32(self.magic);
        helper.write_le_u32(self.version);
        helper.write_string(&self.player_name, PLAYER_NAME_LENGTH);
        helper.write_le_u32(self.player_level);
        helper.write_u8(self.dungeon_level);
        helper.write_u8(self.difficulty as u8);
        helper.write_bool8(self.is_hellfire);
        helper.write_le_u64(self.play_time_seconds);
        helper.write_le_u64(self.save_time);
    }
}

/// Saved player data - matches actual Player structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedPlayer {
    pub name: String,
    pub class: PlayerClass,
    pub level: u32,
    pub stats: PlayerStats,
    pub x: i32,
    pub y: i32,
    pub is_dead: bool,
    pub stat_points: u32,
    pub skill_points: u32,
}

impl From<&Player> for SavedPlayer {
    fn from(player: &Player) -> Self {
        Self {
            name: player.name.clone(),
            class: player.class,
            level: player.level,
            stats: player.stats.clone(),
            x: player.x,
            y: player.y,
            is_dead: player.is_dead,
            stat_points: player.stat_points,
            skill_points: player.skill_points,
        }
    }
}

impl SavedPlayer {
    pub fn to_player(&self) -> Player {
        let mut player = Player::new(self.name.clone(), self.class);
        player.level = self.level;
        player.stats = self.stats.clone();
        player.x = self.x;
        player.y = self.y;
        player.hp = self.stats.hp;
        player.max_hp = self.stats.max_hp;
        player.mana = self.stats.mana;
        player.max_mana = self.stats.max_mana;
        player.is_dead = self.is_dead;
        player.stat_points = self.stat_points;
        player.skill_points = self.skill_points;
        player
    }
}

/// Saved inventory data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedInventory {
    pub equipment: Vec<(EquipSlot, SavedItem)>,
    pub belt: Vec<Option<SavedItem>>,
    pub backpack: Vec<Option<SavedItem>>,
}

/// Saved item data - simplified for JSON serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedItem {
    pub name: String,
    pub base_name: String,
    pub item_type: ItemType,
    pub quality: ItemQuality,
    pub identified: bool,
    // Stats
    pub base_damage_min: i32,
    pub base_damage_max: i32,
    pub base_armor: i32,
    pub quantity: i32,
    pub durability: i32,
    pub max_durability: i32,
    pub buy_value: i32,
}

impl From<&Item> for SavedItem {
    fn from(item: &Item) -> Self {
        // TODO: Map C++ field names to simplified names
        Self {
            name: item._iName.clone(),
            base_name: item._iIName.clone(),
            item_type: ItemType::None, // TODO: item._itype
            quality: item._iMagical,
            identified: item._iIdentified,
            base_damage_min: item._iMinDam as i32,
            base_damage_max: item._iMaxDam as i32,
            base_armor: item._iAC as i32,
            quantity: 1, // TODO: no quantity field in Item
            durability: item._iDurability,
            max_durability: item._iMaxDur,
            buy_value: item._iIvalue,
        }
    }
}

// ============================================================================
// Binary Item Data - matches C++ LoadItemData/SaveItem
// ============================================================================

/// Full item data for binary serialization
/// Matches C++ Item structure from loadsave.cpp
#[derive(Debug, Clone)]
pub struct BinaryItemData {
    pub seed: u32,
    pub create_info: u16,
    pub item_type: i32,
    pub position_x: i32,
    pub position_y: i32,
    pub anim_flag: bool,
    pub anim_frames: i8,
    pub anim_frame: i8,
    pub selection_region: u8,
    pub post_draw: bool,
    pub identified: bool,
    pub magical: i8,
    pub name: Vec<u8>,
    pub iname: Vec<u8>,
    pub loc: i8,
    pub class: u8,
    pub cursor: i32,
    pub value: i32,
    pub ivalue: i32,
    pub min_dam: i32,
    pub max_dam: i32,
    pub ac: i32,
    pub flags: u32,
    pub misc_id: i32,
    pub spell: i32,
    pub charges: i32,
    pub max_charges: i32,
    pub durability: i32,
    pub max_dur: i32,
    pub pl_dam: i32,
    pub pl_to_hit: i32,
    pub pl_ac: i32,
    pub pl_str: i32,
    pub pl_mag: i32,
    pub pl_dex: i32,
    pub pl_vit: i32,
    pub pl_fr: i32,
    pub pl_lr: i32,
    pub pl_mr: i32,
    pub pl_mana: i32,
    pub pl_hp: i32,
    pub pl_dam_mod: i32,
    pub pl_get_hit: i32,
    pub pl_light: i32,
    pub spl_lvl_add: i8,
    pub request: bool,
    pub unique_id: i32,
    pub f_min_dam: i32,
    pub f_max_dam: i32,
    pub l_min_dam: i32,
    pub l_max_dam: i32,
    pub pl_en_ac: i32,
    pub pre_power: i8,
    pub suf_power: i8,
    pub v_add1: i32,
    pub v_mult1: i32,
    pub v_add2: i32,
    pub v_mult2: i32,
    pub min_str: i8,
    pub min_mag: u8,
    pub min_dex: i8,
    pub stat_flag: bool,
    pub item_idx: i32,
    pub dw_buff: u32,
    pub dam_ac_flags: u32,  // Hellfire only
}

impl Default for BinaryItemData {
    fn default() -> Self {
        Self {
            seed: 0,
            create_info: 0,
            item_type: 0,
            position_x: 0,
            position_y: 0,
            anim_flag: false,
            anim_frames: 0,
            anim_frame: 0,
            selection_region: 0,
            post_draw: false,
            identified: false,
            magical: 0,
            name: vec![0; ITEM_NAME_LENGTH],
            iname: vec![0; ITEM_NAME_LENGTH],
            loc: 0,
            class: 0,
            cursor: 0,
            value: 0,
            ivalue: 0,
            min_dam: 0,
            max_dam: 0,
            ac: 0,
            flags: 0,
            misc_id: 0,
            spell: 0,
            charges: 0,
            max_charges: 0,
            durability: 0,
            max_dur: 0,
            pl_dam: 0,
            pl_to_hit: 0,
            pl_ac: 0,
            pl_str: 0,
            pl_mag: 0,
            pl_dex: 0,
            pl_vit: 0,
            pl_fr: 0,
            pl_lr: 0,
            pl_mr: 0,
            pl_mana: 0,
            pl_hp: 0,
            pl_dam_mod: 0,
            pl_get_hit: 0,
            pl_light: 0,
            spl_lvl_add: 0,
            request: false,
            unique_id: 0,
            f_min_dam: 0,
            f_max_dam: 0,
            l_min_dam: 0,
            l_max_dam: 0,
            pl_en_ac: 0,
            pre_power: 0,
            suf_power: 0,
            v_add1: 0,
            v_mult1: 0,
            v_add2: 0,
            v_mult2: 0,
            min_str: 0,
            min_mag: 0,
            min_dex: 0,
            stat_flag: false,
            item_idx: 0,
            dw_buff: 0,
            dam_ac_flags: 0,
        }
    }
}

impl BinaryItemData {
    /// Load from binary helper (matches C++ LoadItemData)
    pub fn from_binary(helper: &mut LoadHelper, is_hellfire: bool) -> Self {
        let mut item = Self::default();

        item.seed = helper.next_le_u32();
        item.create_info = helper.next_le_u16();
        helper.skip(2); // Alignment
        item.item_type = helper.next_le_i32();
        item.position_x = helper.next_le_i32();
        item.position_y = helper.next_le_i32();
        item.anim_flag = helper.next_bool32();
        helper.skip(4); // Skip pointer _iAnimData
        item.anim_frames = helper.next_le_i32() as i8;
        item.anim_frame = (helper.next_le_i32() - 1) as i8;
        helper.skip(8); // Skip _iAnimWidth and _iAnimWidth2
        helper.skip(4); // Unused since 1.02
        item.selection_region = helper.next_u8();
        helper.skip(3); // Alignment
        item.post_draw = helper.next_bool32();
        item.identified = helper.next_bool32();
        item.magical = helper.next_i8();
        item.name = helper.next_bytes(ITEM_NAME_LENGTH);
        item.iname = helper.next_bytes(ITEM_NAME_LENGTH);
        item.loc = helper.next_i8();
        item.class = helper.next_u8();
        helper.skip(1); // Alignment
        item.cursor = helper.next_le_i32();
        item.value = helper.next_le_i32();
        item.ivalue = helper.next_le_i32();
        item.min_dam = helper.next_le_i32();
        item.max_dam = helper.next_le_i32();
        item.ac = helper.next_le_i32();
        item.flags = helper.next_le_u32();
        item.misc_id = helper.next_le_i32();
        item.spell = helper.next_le_i32();
        item.charges = helper.next_le_i32();
        item.max_charges = helper.next_le_i32();
        item.durability = helper.next_le_i32();
        item.max_dur = helper.next_le_i32();
        item.pl_dam = helper.next_le_i32();
        item.pl_to_hit = helper.next_le_i32();
        item.pl_ac = helper.next_le_i32();
        item.pl_str = helper.next_le_i32();
        item.pl_mag = helper.next_le_i32();
        item.pl_dex = helper.next_le_i32();
        item.pl_vit = helper.next_le_i32();
        item.pl_fr = helper.next_le_i32();
        item.pl_lr = helper.next_le_i32();
        item.pl_mr = helper.next_le_i32();
        item.pl_mana = helper.next_le_i32();
        item.pl_hp = helper.next_le_i32();
        item.pl_dam_mod = helper.next_le_i32();
        item.pl_get_hit = helper.next_le_i32();
        item.pl_light = helper.next_le_i32();
        item.spl_lvl_add = helper.next_i8();
        item.request = helper.next_bool8();
        helper.skip(2); // Alignment
        item.unique_id = helper.next_le_i32();
        item.f_min_dam = helper.next_le_i32();
        item.f_max_dam = helper.next_le_i32();
        item.l_min_dam = helper.next_le_i32();
        item.l_max_dam = helper.next_le_i32();
        item.pl_en_ac = helper.next_le_i32();
        item.pre_power = helper.next_i8();
        item.suf_power = helper.next_i8();
        helper.skip(2); // Alignment
        item.v_add1 = helper.next_le_i32();
        item.v_mult1 = helper.next_le_i32();
        item.v_add2 = helper.next_le_i32();
        item.v_mult2 = helper.next_le_i32();
        item.min_str = helper.next_i8();
        item.min_mag = helper.next_u8();
        item.min_dex = helper.next_i8();
        helper.skip(1); // Alignment
        item.stat_flag = helper.next_bool32();
        item.item_idx = helper.next_le_i32();
        item.dw_buff = helper.next_le_u32();
        if is_hellfire {
            item.dam_ac_flags = helper.next_le_u32();
        }

        item
    }

    /// Write to binary helper (matches C++ SaveItem)
    pub fn to_binary(&self, helper: &mut SaveHelper, is_hellfire: bool) {
        helper.write_le_u32(self.seed);
        helper.write_le_u16(self.create_info);
        helper.skip(2); // Alignment
        helper.write_le_i32(self.item_type);
        helper.write_le_i32(self.position_x);
        helper.write_le_i32(self.position_y);
        helper.write_bool32(self.anim_flag);
        helper.skip(4); // Skip pointer
        helper.write_le_i32(self.anim_frames as i32);
        helper.write_le_i32(self.anim_frame as i32 + 1);
        helper.write_le_i32(96); // _iAnimWidth for vanilla compatibility
        // _iAnimWidth2 = CalculateSpriteTileCenterX(ItemAnimWidth) =
        // (96 - TILE_WIDTH 64) / 2 = 16 (levels/dun_tile.hpp:137).
        helper.write_le_i32(16);
        helper.skip(4); // Unused
        helper.write_u8(self.selection_region);
        helper.skip(3); // Alignment
        helper.write_bool32(self.post_draw);
        helper.write_bool32(self.identified);
        helper.write_i8(self.magical);
        helper.write_bytes(&self.name);
        helper.write_bytes(&self.iname);
        helper.write_i8(self.loc);
        helper.write_u8(self.class);
        helper.skip(1); // Alignment
        helper.write_le_i32(self.cursor);
        helper.write_le_i32(self.value);
        helper.write_le_i32(self.ivalue);
        helper.write_le_i32(self.min_dam);
        helper.write_le_i32(self.max_dam);
        helper.write_le_i32(self.ac);
        helper.write_le_u32(self.flags);
        helper.write_le_i32(self.misc_id);
        helper.write_le_i32(self.spell);
        helper.write_le_i32(self.charges);
        helper.write_le_i32(self.max_charges);
        helper.write_le_i32(self.durability);
        helper.write_le_i32(self.max_dur);
        helper.write_le_i32(self.pl_dam);
        helper.write_le_i32(self.pl_to_hit);
        helper.write_le_i32(self.pl_ac);
        helper.write_le_i32(self.pl_str);
        helper.write_le_i32(self.pl_mag);
        helper.write_le_i32(self.pl_dex);
        helper.write_le_i32(self.pl_vit);
        helper.write_le_i32(self.pl_fr);
        helper.write_le_i32(self.pl_lr);
        helper.write_le_i32(self.pl_mr);
        helper.write_le_i32(self.pl_mana);
        helper.write_le_i32(self.pl_hp);
        helper.write_le_i32(self.pl_dam_mod);
        helper.write_le_i32(self.pl_get_hit);
        helper.write_le_i32(self.pl_light);
        helper.write_i8(self.spl_lvl_add);
        helper.write_bool8(self.request);
        helper.skip(2); // Alignment
        helper.write_le_i32(self.unique_id);
        helper.write_le_i32(self.f_min_dam);
        helper.write_le_i32(self.f_max_dam);
        helper.write_le_i32(self.l_min_dam);
        helper.write_le_i32(self.l_max_dam);
        helper.write_le_i32(self.pl_en_ac);
        helper.write_i8(self.pre_power);
        helper.write_i8(self.suf_power);
        helper.skip(2); // Alignment
        helper.write_le_i32(self.v_add1);
        helper.write_le_i32(self.v_mult1);
        helper.write_le_i32(self.v_add2);
        helper.write_le_i32(self.v_mult2);
        helper.write_i8(self.min_str);
        helper.write_u8(self.min_mag);
        helper.write_i8(self.min_dex);
        helper.skip(1); // Alignment
        helper.write_bool32(self.stat_flag);
        helper.write_le_i32(self.item_idx);
        helper.write_le_u32(self.dw_buff);
        if is_hellfire {
            helper.write_le_u32(self.dam_ac_flags);
        }
    }
}

// ============================================================================
// Binary Monster Data - matches C++ LoadMonster
// ============================================================================

/// Monster data for binary serialization
#[derive(Debug, Clone, Default)]
pub struct BinaryMonsterData {
    pub level_type: i32,
    pub mode: i32,
    pub goal: u8,
    pub goal_var1: i16,
    pub goal_var2: i8,
    pub goal_var3: i8,
    pub path_count: u8,
    pub position_x: i32,
    pub position_y: i32,
    pub future_x: i32,
    pub future_y: i32,
    pub old_x: i32,
    pub old_y: i32,
    pub direction: i32,
    pub enemy: i32,
    pub enemy_x: u8,
    pub enemy_y: u8,
    pub anim_ticks_per_frame: i8,
    pub anim_tick_counter: i8,
    pub anim_num_frames: i8,
    pub anim_current_frame: i8,
    pub is_invalid: bool,
    pub var1: i16,
    pub var2: i16,
    pub var3: i8,
    pub temp_x: i8,
    pub temp_y: i8,
    pub max_hp: i32,
    pub hp: i32,
    pub ai: u8,
    pub intelligence: u8,
    pub flags: u32,
    pub active_for_ticks: u8,
    pub last_x: i32,
    pub last_y: i32,
    pub rnd_item_seed: u32,
    pub ai_seed: u32,
    pub unique_type: u8,
    pub uniq_trans: u8,
    pub corpse_id: i8,
    pub who_hit: i8,
    pub min_damage: u8,
    pub max_damage: u8,
    pub min_damage_special: u8,
    pub max_damage_special: u8,
    pub armor_class: u8,
    pub resistance: u16,
    pub talk_msg: i32,
    pub leader: u8,
    pub leader_relation: u8,
    pub pack_size: u8,
    pub light_id: i8,
}

impl BinaryMonsterData {
    /// Load from binary helper (matches C++ LoadMonster)
    pub fn from_binary(helper: &mut LoadHelper) -> Self {
        let mut m = Self::default();

        m.level_type = helper.next_le_i32();
        m.mode = helper.next_le_i32();
        m.goal = helper.next_u8();
        helper.skip(3); // Alignment
        m.goal_var1 = helper.next_le_i32() as i16;
        m.goal_var2 = helper.next_le_i32() as i8;
        m.goal_var3 = helper.next_le_i32() as i8;
        helper.skip(4); // Unused
        m.path_count = helper.next_u8();
        helper.skip(3); // Alignment
        m.position_x = helper.next_le_i32();
        m.position_y = helper.next_le_i32();
        m.future_x = helper.next_le_i32();
        m.future_y = helper.next_le_i32();
        m.old_x = helper.next_le_i32();
        m.old_y = helper.next_le_i32();
        helper.skip(16); // Skip offset and velocity
        m.direction = helper.next_le_i32();
        m.enemy = helper.next_le_i32();
        m.enemy_x = helper.next_u8();
        m.enemy_y = helper.next_u8();
        helper.skip(2); // Unused
        helper.skip(4); // Skip pointer _mAnimData
        m.anim_ticks_per_frame = helper.next_le_i32() as i8;
        m.anim_tick_counter = (helper.next_le_i32() as i8).saturating_sub(1);
        m.anim_num_frames = helper.next_le_i32() as i8;
        m.anim_current_frame = (helper.next_le_i32() - 1) as i8;
        helper.skip(4); // Skip _meflag
        m.is_invalid = helper.next_bool32();
        m.var1 = helper.next_le_i32() as i16;
        m.var2 = helper.next_le_i32() as i16;
        m.var3 = helper.next_le_i32() as i8;
        m.temp_x = helper.next_le_i32() as i8;
        m.temp_y = helper.next_le_i32() as i8;
        helper.skip(8); // Skip offset2
        helper.skip(4); // Skip actionFrame
        m.max_hp = helper.next_le_i32();
        m.hp = helper.next_le_i32();
        m.ai = helper.next_u8();
        m.intelligence = helper.next_u8();
        helper.skip(2); // Alignment
        m.flags = helper.next_le_u32();
        m.active_for_ticks = helper.next_u8();
        helper.skip(3); // Alignment
        helper.skip(4); // Unused
        m.last_x = helper.next_le_i32();
        m.last_y = helper.next_le_i32();
        m.rnd_item_seed = helper.next_le_u32();
        m.ai_seed = helper.next_le_u32();
        helper.skip(4); // Unused
        m.unique_type = helper.next_u8().wrapping_sub(1);
        m.uniq_trans = helper.next_u8();
        m.corpse_id = helper.next_i8();
        m.who_hit = helper.next_i8();
        helper.skip(1); // Skip level
        helper.skip(1); // Alignment
        helper.skip(2); // Skip exp
        helper.skip(1); // Skip toHit
        m.min_damage = helper.next_u8();
        m.max_damage = helper.next_u8();
        helper.skip(1); // Skip toHitSpecial
        m.min_damage_special = helper.next_u8();
        m.max_damage_special = helper.next_u8();
        m.armor_class = helper.next_u8();
        helper.skip(1); // Alignment
        m.resistance = helper.next_le_u16();
        helper.skip(2); // Alignment
        m.talk_msg = helper.next_le_i32();
        m.leader = helper.next_u8();
        m.leader_relation = helper.next_u8();
        m.pack_size = helper.next_u8();
        m.light_id = helper.next_i8();

        m
    }

    /// Write to binary helper (matches C++ SaveMonster, lines 1502-1612).
    ///
    /// `monster_level` / `experience` / `to_hit` / `to_hit_special` are passed
    /// in explicitly because the C++ writer derives them from
    /// `Monster::level/exp/toHit/toHitSpecial(difficulty)` rather than storing
    /// them as raw fields. We pass simplified scalars so the on-disk layout is
    /// byte-for-byte identical to vanilla Diablo saves.
    pub fn to_binary(
        &self,
        helper: &mut SaveHelper,
        monster_level: i8,
        experience: u16,
        to_hit: u8,
        to_hit_special: u8,
    ) {
        helper.write_le_i32(self.level_type);
        helper.write_le_i32(self.mode);
        helper.write_u8(self.goal);
        helper.skip(3); // Alignment
        helper.write_le_i32(self.goal_var1 as i32);
        helper.write_le_i32(self.goal_var2 as i32);
        helper.write_le_i32(self.goal_var3 as i32);
        helper.skip(4); // Unused
        helper.write_u8(self.path_count);
        helper.skip(3); // Alignment
        helper.write_le_i32(self.position_x);
        helper.write_le_i32(self.position_y);
        helper.write_le_i32(self.future_x);
        helper.write_le_i32(self.future_y);
        helper.write_le_i32(self.old_x);
        helper.write_le_i32(self.old_y);
        // offset.deltaX/Y (walking offset — 0 when idle, matching C++ default)
        helper.write_le_i32(0);
        helper.write_le_i32(0);
        // velocity.deltaX/Y
        helper.write_le_i32(0);
        helper.write_le_i32(0);
        helper.write_le_i32(self.direction);
        helper.write_le_i32(self.enemy);
        helper.write_u8(self.enemy_x);
        helper.write_u8(self.enemy_y);
        helper.skip(2); // Unused
        helper.skip(4); // Skip pointer _mAnimData
        helper.write_le_i32(self.anim_ticks_per_frame as i32);
        helper.write_le_i32(self.anim_tick_counter as i32);
        helper.write_le_i32(self.anim_num_frames as i32);
        helper.write_le_i32(self.anim_current_frame as i32 + 1);
        helper.skip(4); // Skip _meflag
        helper.write_le_u32(if self.is_invalid { 1 } else { 0 });
        helper.write_le_i32(self.var1 as i32);
        helper.write_le_i32(self.var2 as i32);
        helper.write_le_i32(self.var3 as i32);
        helper.write_le_i32(self.temp_x as i32);
        helper.write_le_i32(self.temp_y as i32);
        // offset2.deltaX/Y
        helper.write_le_i32(0);
        helper.write_le_i32(0);
        helper.skip(4); // Skip _mVar8
        helper.write_le_i32(self.max_hp);
        helper.write_le_i32(self.hp);
        helper.write_u8(self.ai);
        helper.write_u8(self.intelligence);
        helper.skip(2); // Alignment
        helper.write_le_u32(self.flags);
        helper.write_u8(self.active_for_ticks);
        helper.skip(3); // Alignment
        helper.skip(4); // Unused
        helper.write_le_i32(self.last_x);
        helper.write_le_i32(self.last_y);
        helper.write_le_u32(self.rnd_item_seed);
        helper.write_le_u32(self.ai_seed);
        helper.skip(4); // Unused
        // Vanilla writes uniqueType + 1 (0 == "no unique").
        helper.write_u8(self.unique_type.wrapping_add(1));
        helper.write_u8(self.uniq_trans);
        helper.write_i8(self.corpse_id);
        helper.write_i8(self.who_hit);
        helper.write_i8(monster_level);
        helper.skip(1); // Alignment
        helper.write_le_u16(experience);
        helper.write_u8(to_hit);
        helper.write_u8(self.min_damage);
        helper.write_u8(self.max_damage);
        helper.write_u8(to_hit_special);
        helper.write_u8(self.min_damage_special);
        helper.write_u8(self.max_damage_special);
        helper.write_u8(self.armor_class);
        helper.skip(1); // Alignment
        helper.write_le_u16(self.resistance);
        helper.skip(2); // Alignment
        helper.write_le_i32(self.talk_msg);
        // C++ SaveMonster writes 0 when leader == Monster::NoLeader (loadsave.cpp:1610).
        helper.write_u8(if self.leader == u8::MAX { 0 } else { self.leader });
        helper.write_u8(self.leader_relation);
        helper.write_u8(self.pack_size);
        // Vanilla writes 0 when lightId == NO_LIGHT (-1).
        helper.write_i8(if self.light_id < 0 { 0 } else { self.light_id });
    }
}

// ============================================================================
// Binary Quest Data - matches C++ LoadQuest
// ============================================================================

/// Quest data for binary serialization
#[derive(Debug, Clone, Default)]
pub struct BinaryQuestData {
    pub level: u8,
    pub active: u8,
    pub level_type: u8,
    pub position_x: i32,
    pub position_y: i32,
    pub set_level: u8,
    pub quest_id: u8,
    pub quest_msg: i32,
    pub var1: u8,
    pub var2: u8,
    pub log: bool,
}

impl BinaryQuestData {
    /// Load from binary helper (matches C++ LoadQuest)
    pub fn from_binary(helper: &mut LoadHelper, is_hellfire: bool) -> Self {
        let mut q = Self::default();

        q.level = helper.next_u8();
        helper.skip(1); // _qtype
        q.active = helper.next_u8();
        q.level_type = helper.next_u8();
        q.position_x = helper.next_le_i32();
        q.position_y = helper.next_le_i32();
        q.set_level = helper.next_u8();
        q.quest_id = helper.next_u8();

        if is_hellfire {
            helper.skip(2); // Alignment
            q.quest_msg = helper.next_le_i32();
        } else {
            q.quest_msg = helper.next_u8() as i32;
        }

        q.var1 = helper.next_u8();
        q.var2 = helper.next_u8();
        helper.skip(2); // Alignment
        if !is_hellfire {
            helper.skip(1); // Alignment
        }
        q.log = helper.next_bool32();

        // Skip ReturnLvl data
        helper.skip(20);

        q
    }

    /// Write to binary helper (matches C++ SaveQuest, lines 1762-1792).
    /// `return_lvl` carries the 5 i32 ReturnLvl fields written after the quest
    /// body (position.x/y, level, levelType, doomQuestState).
    pub fn to_binary(
        &self,
        helper: &mut SaveHelper,
        is_hellfire: bool,
        quest_type: u8,
        return_lvl: &ReturnLevelData,
    ) {
        helper.write_u8(self.level);
        helper.write_u8(quest_type); // _qtype for compatibility
        helper.write_u8(self.active);
        helper.write_u8(self.level_type);
        helper.write_le_i32(self.position_x);
        helper.write_le_i32(self.position_y);
        helper.write_u8(self.set_level);
        helper.write_u8(self.quest_id);
        if is_hellfire {
            helper.skip(2); // Alignment
            helper.write_le_i32(self.quest_msg);
        } else {
            helper.write_u8(self.quest_msg as u8);
        }
        helper.write_u8(self.var1);
        helper.write_u8(self.var2);
        helper.skip(2); // Alignment
        if !is_hellfire {
            helper.skip(1); // Alignment
        }
        helper.write_bool32(self.log);
        // ReturnLvl block: 4 BE i32s + 1 skipped i32 (DoomQuestState).
        helper.write_be_i32(return_lvl.position_x);
        helper.write_be_i32(return_lvl.position_y);
        helper.write_be_i32(return_lvl.level);
        helper.write_be_i32(return_lvl.level_type);
        helper.skip(4); // DoomQuestState
    }
}

/// Return-level context saved alongside each quest (C++ `ReturnLvlPosition` /
/// `ReturnLevel` / `ReturnLevelType` / `DoomQuestState`).
#[derive(Debug, Clone, Default)]
pub struct ReturnLevelData {
    pub position_x: i32,
    pub position_y: i32,
    pub level: i32,
    pub level_type: i32,
}

// ============================================================================
// Binary Object Data - matches C++ LoadObject
// ============================================================================

/// Object data for binary serialization
#[derive(Debug, Clone, Default)]
pub struct BinaryObjectData {
    pub object_type: i32,
    pub position_x: i32,
    pub position_y: i32,
    pub apply_lighting: bool,
    pub anim_flag: bool,
    pub anim_delay: i32,
    pub anim_cnt: i32,
    pub anim_len: u32,
    pub anim_frame: u32,
    pub anim_width: u16,
    pub del_flag: bool,
    pub break_flag: i8,
    pub solid_flag: bool,
    pub miss_flag: bool,
    pub selection_region: i8,
    pub pre_flag: bool,
    pub trap_flag: bool,
    pub door_flag: bool,
    pub light_id: i32,
    pub rnd_seed: u32,
    pub var1: i32,
    pub var2: i32,
    pub var3: i32,
    pub var4: i32,
    pub var5: i32,
    pub var6: u32,
    pub book_message: i32,
    pub var8: i32,
}

impl BinaryObjectData {
    /// Load from binary helper (matches C++ LoadObject)
    pub fn from_binary(helper: &mut LoadHelper) -> Self {
        let mut o = Self::default();

        o.object_type = helper.next_le_i32();
        o.position_x = helper.next_le_i32();
        o.position_y = helper.next_le_i32();
        o.apply_lighting = helper.next_bool32();
        o.anim_flag = helper.next_bool32();
        helper.skip(4); // Skip pointer _oAnimData
        o.anim_delay = helper.next_le_i32();
        o.anim_cnt = helper.next_le_i32();
        o.anim_len = helper.next_le_u32();
        o.anim_frame = helper.next_le_u32();
        o.anim_width = helper.next_le_i32() as u16;
        helper.skip(4); // Skip _oAnimWidth2
        o.del_flag = helper.next_bool32();
        o.break_flag = helper.next_i8();
        helper.skip(3); // Alignment
        o.solid_flag = helper.next_bool32();
        o.miss_flag = helper.next_bool32();
        o.selection_region = helper.next_i8();
        helper.skip(3); // Alignment
        o.pre_flag = helper.next_bool32();
        o.trap_flag = helper.next_bool32();
        o.door_flag = helper.next_bool32();
        o.light_id = helper.next_le_i32();
        o.rnd_seed = helper.next_le_u32();
        o.var1 = helper.next_le_i32();
        o.var2 = helper.next_le_i32();
        o.var3 = helper.next_le_i32();
        o.var4 = helper.next_le_i32();
        o.var5 = helper.next_le_i32();
        o.var6 = helper.next_le_u32();
        o.book_message = helper.next_le_i32();
        o.var8 = helper.next_le_i32();

        o
    }

    /// Write to binary helper (matches C++ SaveObject, lines 1702-1760).
    pub fn to_binary(&self, helper: &mut SaveHelper) {
        helper.write_le_i32(self.object_type);
        helper.write_le_i32(self.position_x);
        helper.write_le_i32(self.position_y);
        helper.write_bool32(self.apply_lighting);
        helper.write_bool32(self.anim_flag);
        helper.skip(4); // Skip pointer _oAnimData
        helper.write_le_i32(self.anim_delay);
        helper.write_le_i32(self.anim_cnt);
        helper.write_le_u32(self.anim_len);
        helper.write_le_u32(self.anim_frame);
        helper.write_le_i32(self.anim_width as i32);
        // _oAnimWidth2 — vanilla-compat derived value; C++ uses
        // CalculateSpriteTileCenterX(animWidth). We mirror that with the same
        // formula (animWidth / 2 rounded to the tile-centre offset).
        helper.write_le_i32((self.anim_width as i32) / 2);
        helper.write_bool32(self.del_flag);
        helper.write_i8(self.break_flag);
        helper.skip(3); // Alignment
        helper.write_bool32(self.solid_flag);
        helper.write_bool32(self.miss_flag);
        helper.write_i8(self.selection_region);
        helper.skip(3); // Alignment
        helper.write_bool32(self.pre_flag);
        helper.write_bool32(self.trap_flag);
        helper.write_bool32(self.door_flag);
        helper.write_le_i32(self.light_id);
        helper.write_le_u32(self.rnd_seed);
        helper.write_le_i32(self.var1);
        helper.write_le_i32(self.var2);
        helper.write_le_i32(self.var3);
        helper.write_le_i32(self.var4);
        helper.write_le_i32(self.var5);
        helper.write_le_u32(self.var6);
        helper.write_le_i32(self.book_message);
        helper.write_le_i32(self.var8);
    }
}

// ============================================================================
// Binary Light Data - matches C++ LoadLighting
// ============================================================================

/// Light data for binary serialization
#[derive(Debug, Clone, Default)]
pub struct BinaryLightData {
    pub position_x: i32,
    pub position_y: i32,
    pub radius: i32,
    pub is_invalid: bool,
    pub has_changed: bool,
    pub old_x: i32,
    pub old_y: i32,
    pub old_radius: i32,
    pub offset_x: i32,
    pub offset_y: i32,
}

impl BinaryLightData {
    /// Load from binary helper (matches C++ LoadLighting)
    pub fn from_binary(helper: &mut LoadHelper) -> Self {
        let mut l = Self::default();

        l.position_x = helper.next_le_i32();
        l.position_y = helper.next_le_i32();
        l.radius = helper.next_le_i32();
        helper.skip(4); // _lid
        l.is_invalid = helper.next_bool32();
        l.has_changed = helper.next_bool32();
        helper.skip(4); // Unused
        l.old_x = helper.next_le_i32();
        l.old_y = helper.next_le_i32();
        l.old_radius = helper.next_le_i32();
        l.offset_x = helper.next_le_i32();
        l.offset_y = helper.next_le_i32();
        helper.skip(4); // _lflags

        l
    }

    /// Write to binary helper (matches C++ SaveLighting, loadsave.cpp:1791-1809).
    /// `vision` toggles the `_lid` and `_lflags` fields.
    pub fn to_binary(&self, helper: &mut SaveHelper, vision: bool) {
        helper.write_le_i32(self.position_x);
        helper.write_le_i32(self.position_y);
        helper.write_le_i32(self.radius);
        helper.write_le_i32(if vision { 1 } else { 0 }); // _lid
        helper.write_le_u32(if self.is_invalid { 1 } else { 0 });
        helper.write_le_u32(if self.has_changed { 1 } else { 0 });
        helper.skip(4); // Unused
        helper.write_le_i32(self.old_x);
        helper.write_le_i32(self.old_y);
        helper.write_le_i32(self.old_radius);
        helper.write_le_i32(self.offset_x);
        helper.write_le_i32(self.offset_y);
        helper.write_le_u32(if vision { 1 } else { 0 }); // _lflags
    }
}

// ============================================================================
// Binary Portal Data - matches C++ LoadPortal
// ============================================================================

/// Portal data for binary serialization
#[derive(Debug, Clone, Default)]
pub struct BinaryPortalData {
    pub open: bool,
    pub position_x: i32,
    pub position_y: i32,
    pub level: i32,
    pub level_type: i32,
    pub is_set_level: bool,
}

impl BinaryPortalData {
    /// Load from binary helper (matches C++ LoadPortal)
    pub fn from_binary(helper: &mut LoadHelper) -> Self {
        Self {
            open: helper.next_bool32(),
            position_x: helper.next_le_i32(),
            position_y: helper.next_le_i32(),
            level: helper.next_le_i32(),
            level_type: helper.next_le_i32(),
            is_set_level: helper.next_bool32(),
        }
    }

    /// Write to binary helper (matches C++ SavePortal, lines 1811-1821).
    pub fn to_binary(&self, helper: &mut SaveHelper) {
        helper.write_bool32(self.open);
        helper.write_le_i32(self.position_x);
        helper.write_le_i32(self.position_y);
        helper.write_le_i32(self.level);
        helper.write_le_i32(self.level_type);
        helper.write_bool32(self.is_set_level);
    }
}

/// Complete game save data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSave {
    pub header: SaveHeader,
    pub player: SavedPlayer,
    pub inventory: SavedInventory,
    pub dungeon_level: u8,
    pub dungeon_type: DungeonType,
    pub dungeon_seed: u64,
    pub game_tick: u64,
    /// Learned spells by spell ID
    pub learned_spells: Vec<u8>,
    /// Quest states (quest_id, state)
    pub quest_states: Vec<(u8, u8)>,
    /// Stash items (shared storage)
    pub stash: Vec<SavedItem>,
}

/// Save manager handles save/load operations
pub struct SaveManager {
    save_dir: String,
}

impl Default for SaveManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveManager {
    pub fn new() -> Self {
        // Default save directory
        let save_dir = if cfg!(windows) {
            dirs::data_local_dir()
                .map(|p| p.join("DevilutionX-RS").join("saves"))
                .unwrap_or_else(|| std::path::PathBuf::from("saves"))
                .to_string_lossy()
                .to_string()
        } else {
            dirs::home_dir()
                .map(|p| p.join(".devilutionx-rs").join("saves"))
                .unwrap_or_else(|| std::path::PathBuf::from("saves"))
                .to_string_lossy()
                .to_string()
        };

        Self { save_dir }
    }

    pub fn with_dir(save_dir: &str) -> Self {
        Self {
            save_dir: save_dir.to_string(),
        }
    }

    /// Ensure save directory exists
    fn ensure_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.save_dir)?;
        Ok(())
    }

    /// Get save file path for a slot
    fn save_path(&self, slot: u32) -> std::path::PathBuf {
        Path::new(&self.save_dir).join(format!("save_{}.json", slot))
    }

    /// Save game to a slot
    pub fn save_game(&self, slot: u32, save: &GameSave) -> Result<()> {
        self.ensure_dir()?;
        let path = self.save_path(slot);
        let file = File::create(&path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, save)?;
        Ok(())
    }

    /// Load game from a slot
    pub fn load_game(&self, slot: u32) -> Result<GameSave> {
        let path = self.save_path(slot);
        if !path.exists() {
            return Err(anyhow!("Save file does not exist: {:?}", path));
        }

        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let save: GameSave = serde_json::from_reader(reader)?;

        if !save.header.is_valid() {
            return Err(anyhow!("Invalid or incompatible save file"));
        }

        Ok(save)
    }

    /// Check if a save slot has data
    pub fn slot_exists(&self, slot: u32) -> bool {
        self.save_path(slot).exists()
    }

    /// Get header from a save slot (for save slot display)
    pub fn get_header(&self, slot: u32) -> Option<SaveHeader> {
        let path = self.save_path(slot);
        if !path.exists() {
            return None;
        }

        File::open(&path)
            .ok()
            .and_then(|file| {
                let reader = BufReader::new(file);
                serde_json::from_reader::<_, GameSave>(reader).ok()
            })
            .map(|save| save.header)
    }

    /// List all save slots with data
    pub fn list_saves(&self) -> Vec<(u32, SaveHeader)> {
        let mut saves = Vec::new();
        for slot in 0..10 {
            if let Some(header) = self.get_header(slot) {
                saves.push((slot, header));
            }
        }
        saves
    }

    /// Delete a save slot
    pub fn delete_save(&self, slot: u32) -> Result<()> {
        let path = self.save_path(slot);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Export save to a custom file
    pub fn export_save(&self, slot: u32, export_path: &str) -> Result<()> {
        let save = self.load_game(slot)?;
        let file = File::create(export_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &save)?;
        Ok(())
    }

    /// Import save from a custom file
    pub fn import_save(&self, slot: u32, import_path: &str) -> Result<()> {
        let file = File::open(import_path)?;
        let reader = BufReader::new(file);
        let save: GameSave = serde_json::from_reader(reader)?;

        if !save.header.is_valid() {
            return Err(anyhow!("Invalid or incompatible save file"));
        }

        self.save_game(slot, &save)
    }
}

/// Quick save/load functions
pub fn quick_save(save: &GameSave) -> Result<()> {
    let manager = SaveManager::new();
    manager.save_game(0, save)
}

pub fn quick_load() -> Result<GameSave> {
    let manager = SaveManager::new();
    manager.load_game(0)
}

/// Format play time as HH:MM:SS
pub fn format_play_time(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

/// Format save date
pub fn format_save_date(timestamp: u64) -> String {
    use std::time::{Duration, UNIX_EPOCH};

    let datetime = UNIX_EPOCH + Duration::from_secs(timestamp);
    // Simple format without external crate
    format!("{:?}", datetime)
}

// ============================================================================
// C++ `SaveGameData` header (Source/loadsave.cpp:2762-2806)
// ============================================================================

/// The fixed header of a decoded save-archive `game` entry, read with the C++
/// field order/endianness: magic LE u32, setlevel LE u8, then BE u32s and LE
/// u8 flags. Covers the fields through `ActiveObjectCount`; the remainder of
/// the entry (level seeds, player, quests, monsters, ...) follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CppGameHeader {
    /// First four decoded bytes ("SHAR"/"SHLF"/"RETL"/"HELF").
    pub magic: [u8; 4],
    /// `setlevel ? 1 : 0` (LE u8).
    pub setlevel: u8,
    /// `setlvlnum` (BE u32).
    pub setlvlnum: u32,
    /// `currlevel` (BE u32).
    pub currlevel: u32,
    /// `getHellfireLevelType(leveltype)` (BE u32).
    pub leveltype: u32,
    /// `ViewPosition.x` (BE i32).
    pub view_position_x: i32,
    /// `ViewPosition.y` (BE i32).
    pub view_position_y: i32,
    /// `invflag` (LE u8 bool).
    pub invflag: bool,
    /// `CharFlag` (LE u8 bool).
    pub char_flag: bool,
    /// `ActiveMonsterCount` (BE i32).
    pub active_monster_count: i32,
    /// `ActiveItemCount` (BE i32).
    pub active_item_count: i32,
    /// `ActiveMissileCount` (BE u32).
    pub active_missile_count: u32,
    /// `ActiveObjectCount` (BE i32).
    pub active_object_count: i32,
}

impl CppGameHeader {
    /// Parse the per-level `DungeonSeeds`/level-type table that immediately
    /// follows the fixed header (C++ loadsave.cpp:2803-2806): for each of the
    /// `level_count` levels, a BE u32 seed then a BE u32 `getHellfireLevelType`
    /// value. Returns `None` if the entry is too short.
    pub fn parse_level_seeds(decoded: &[u8], level_count: usize) -> Option<Vec<(u32, u32)>> {
        let mut out = Vec::with_capacity(level_count);
        let mut offset = 43usize;
        for _ in 0..level_count {
            if decoded.len() < offset + 8 {
                return None;
            }
            let seed = u32::from_be_bytes([decoded[offset], decoded[offset + 1], decoded[offset + 2], decoded[offset + 3]]);
            let ltype = u32::from_be_bytes([decoded[offset + 4], decoded[offset + 5], decoded[offset + 6], decoded[offset + 7]]);
            out.push((seed, ltype));
            offset += 8;
        }
        Some(out)
    }

    /// Parse the fixed header from a decoded `game` entry (43 bytes).
    pub fn parse(decoded: &[u8]) -> Option<Self> {
        if decoded.len() < 43 {
            return None;
        }
        let be_u32 = |i: usize| u32::from_be_bytes([decoded[i], decoded[i + 1], decoded[i + 2], decoded[i + 3]]);
        let be_i32 = |i: usize| i32::from_be_bytes([decoded[i], decoded[i + 1], decoded[i + 2], decoded[i + 3]]);
        Some(Self {
            magic: [decoded[0], decoded[1], decoded[2], decoded[3]],
            setlevel: decoded[4],
            setlvlnum: be_u32(5),
            currlevel: be_u32(9),
            leveltype: be_u32(13),
            view_position_x: be_i32(17),
            view_position_y: be_i32(21),
            invflag: decoded[25] != 0,
            char_flag: decoded[26] != 0,
            active_monster_count: be_i32(27),
            active_item_count: be_i32(31),
            active_missile_count: be_u32(35),
            active_object_count: be_i32(39),
        })
    }

    /// Serialize the fixed header exactly as C++ `SaveGameData` writes it
    /// (loadsave.cpp:2766-2801): magic LE u32, setlevel LE u8, then BE
    /// u32/i32 fields, then the two LE u8 flags. Returns the 43-byte header.
    pub fn write(&self) -> Vec<u8> {
        let mut helper = SaveHelper::new(43);
        helper.write_le_u32(u32::from_le_bytes(self.magic));
        helper.write_u8(self.setlevel);
        helper.write_be_u32(self.setlvlnum);
        helper.write_be_u32(self.currlevel);
        helper.write_be_u32(self.leveltype);
        helper.write_be_i32(self.view_position_x);
        helper.write_be_i32(self.view_position_y);
        helper.write_bool8(self.invflag);
        helper.write_bool8(self.char_flag);
        helper.write_be_i32(self.active_monster_count);
        helper.write_be_i32(self.active_item_count);
        helper.write_be_u32(self.active_missile_count);
        helper.write_be_i32(self.active_object_count);
        helper.into_data()
    }

    /// Serialize the per-level seed table (loadsave.cpp:2803-2806): 8 bytes
    /// per level (BE u32 seed, BE u32 level type).
    pub fn write_level_seeds(seeds: &[(u32, u32)]) -> Vec<u8> {
        let mut helper = SaveHelper::new(seeds.len() * 8);
        for &(seed, ltype) in seeds {
            helper.write_be_u32(seed);
            helper.write_be_u32(ltype);
        }
        helper.into_data()
    }
}

// ============================================================================
// SaveGameData fixed sections (kill counts / unique flags / grids)
// ============================================================================

/// C++ `MonsterKillCounts` block (loadsave.cpp:2814-2817): 138 counts
/// (MonstersData.size()) as BE i32, padded with `(MaxMonsters - 138) * 4`
/// bytes for vanilla compatibility (MaxMonsters = 200).
pub fn write_kill_counts(helper: &mut SaveHelper, counts: &[i32]) {
    const MONSTER_COUNT: usize = 138; // monstdat.tsv rows / MonstersData.size()
    const MAX_MONSTERS: usize = 200; // C++ MaxMonsters (monster.h)
    for c in counts.iter().take(MONSTER_COUNT) {
        helper.write_be_i32(*c);
    }
    helper.skip(4 * (MAX_MONSTERS - MONSTER_COUNT));
}

/// C++ `UniqueItemFlags` block (loadsave.cpp:2846-2848): 128 × LE u8.
pub fn write_unique_flags(helper: &mut SaveHelper, flags: &[bool]) {
    const UNIQUE_FLAG_COUNT: usize = 128; // C++ UniqueItemFlags[128]
    for f in flags.iter().take(UNIQUE_FLAG_COUNT) {
        helper.write_u8(if *f { 1 } else { 0 });
    }
    helper.skip(UNIQUE_FLAG_COUNT - flags.len().min(UNIQUE_FLAG_COUNT));
}

/// Write a 112×112 u8 grid row-major (C++ `for j { for i { WriteLE } }`), used
/// for dLight / dFlags / dPlayer / dPreLight.
pub fn write_grid_u8(helper: &mut SaveHelper, grid: &[u8]) {
    const N: usize = 112;
    debug_assert!(grid.len() >= N * N, "grid too small");
    for y in 0..N {
        for x in 0..N {
            helper.write_u8(grid[y * N + x]);
        }
    }
}

/// Write a 112×112 i32 grid row-major big-endian (C++ dMonster block).
pub fn write_grid_i32_be(helper: &mut SaveHelper, grid: &[i32]) {
    const N: usize = 112;
    debug_assert!(grid.len() >= N * N, "grid too small");
    for y in 0..N {
        for x in 0..N {
            helper.write_be_i32(grid[y * N + x]);
        }
    }
}

// ============================================================================
// C++ SaveQuest / SavePortal (loadsave.cpp:1762-1789 / 1811-1818)
// ============================================================================

/// C++ `SaveQuest` (loadsave.cpp:1762-1789), classic (non-Hellfire) layout:
/// 44 bytes per quest, little-endian fields followed by BE return-position
/// fields and a 4-byte DoomQuestState skip.
pub fn write_quest(
    helper: &mut SaveHelper,
    quest: &crate::game::quest_new::Quest,
    return_pos: (i32, i32),
    return_level: i32,
    return_level_type: i32,
) {
    helper.write_u8(quest._qlevel);
    helper.write_u8(quest._qidx as i8 as u8);
    helper.write_u8(quest._qactive as u8);
    helper.write_u8(quest._qlvltype as u8);
    helper.write_le_i32(quest.position.0);
    helper.write_le_i32(quest.position.1);
    helper.write_u8(quest._qslvl as u8);
    helper.write_u8(quest._qidx as i8 as u8);
    helper.write_u8(quest._qmsg as u8);
    helper.write_u8(quest._qvar1);
    helper.write_u8(quest._qvar2);
    helper.skip(3); // Alignment (2) + non-Hellfire (1)
    helper.write_le_u32(if quest._qlog { 1 } else { 0 });
    helper.write_be_i32(return_pos.0);
    helper.write_be_i32(return_pos.1);
    helper.write_be_i32(return_level);
    helper.write_be_i32(return_level_type);
    helper.skip(4); // DoomQuestState
}

/// Read a classic `SaveQuest` block; returns the quest and the next offset.
pub fn parse_quest(decoded: &[u8], offset: usize) -> Option<(crate::game::quest_new::Quest, usize)> {
    if decoded.len() < offset + 44 {
        return None;
    }
    let le_i32 = |i: usize| i32::from_le_bytes([decoded[i], decoded[i + 1], decoded[i + 2], decoded[i + 3]]);
    let mut q = crate::game::quest_new::Quest::default();
    q._qlevel = decoded[offset];
    q._qidx = unsafe { std::mem::transmute(decoded[offset + 1] as i8) };
    q._qactive = unsafe { std::mem::transmute(decoded[offset + 2]) };
    // level_new::DungeonType is repr(i8) with C++ dungeon_type values 0..6.
    q._qlvltype = unsafe { std::mem::transmute(decoded[offset + 3] as i8) };
    q.position = (le_i32(offset + 4), le_i32(offset + 8));
    q._qslvl = unsafe { std::mem::transmute(decoded[offset + 12] as i8) };
    q._qmsg = unsafe { std::mem::transmute(decoded[offset + 14] as i16) };
    q._qvar1 = decoded[offset + 15];
    q._qvar2 = decoded[offset + 16];
    q._qlog = le_i32(offset + 20) != 0;
    Some((q, offset + 44))
}

/// C++ `SavePortal` (loadsave.cpp:1811-1818): 24 bytes.
pub fn write_portal(helper: &mut SaveHelper, open: bool, pos: (i32, i32), level: i32, ltype: i32, setlvl: bool) {
    helper.write_le_u32(if open { 1 } else { 0 });
    helper.write_le_i32(pos.0);
    helper.write_le_i32(pos.1);
    helper.write_le_i32(level);
    helper.write_le_i32(ltype);
    helper.write_le_u32(if setlvl { 1 } else { 0 });
}

/// Read a classic `SavePortal` block; returns `(open, pos, level, ltype, setlvl)`.
pub fn parse_portal(decoded: &[u8], offset: usize) -> Option<((bool, (i32, i32), i32, i32, bool), usize)> {
    if decoded.len() < offset + 24 {
        return None;
    }
    let le_i32 = |i: usize| i32::from_le_bytes([decoded[i], decoded[i + 1], decoded[i + 2], decoded[i + 3]]);
    let open = le_i32(offset) != 0;
    let pos = (le_i32(offset + 4), le_i32(offset + 8));
    let level = le_i32(offset + 12);
    let ltype = le_i32(offset + 16);
    let setlvl = le_i32(offset + 20) != 0;
    Some(((open, pos, level, ltype, setlvl), offset + 24))
}

// ============================================================================
// SaveGameData dropped items (C++ loadsave.cpp:1823-1850 / 2909-2914)
// ============================================================================

/// C++ `MAXITEMS` (loadsave.cpp / items.h).
pub const MAX_ITEMS_FOR_SAVE: usize = 127;

/// C++ `SaveDroppedItems` (loadsave.cpp:1823-1850): MAXITEMS active-item index
/// array (0..126 for vanilla compatibility), MAXITEMS available-item array
/// (`(i + ActiveItemCount) % MAXITEMS`), then one `SaveItem` body per active
/// item.
pub fn write_dropped_items(helper: &mut SaveHelper, items: &[BinaryItemData], is_hellfire: bool) {
    for i in 0..MAX_ITEMS_FOR_SAVE {
        helper.write_u8(i as u8);
    }
    let count = items.len().min(MAX_ITEMS_FOR_SAVE);
    for i in 0..MAX_ITEMS_FOR_SAVE {
        helper.write_u8(((i + count) % MAX_ITEMS_FOR_SAVE) as u8);
    }
    for item in items.iter().take(count) {
        item.to_binary(helper, is_hellfire);
    }
}

/// C++ `SaveDroppedItemLocations` (loadsave.cpp:2909-2914): one u8 per active
/// item indexing into the save file (0 is reserved, so items start at 1).
pub fn write_dropped_item_locations(helper: &mut SaveHelper, count: usize) {
    for i in 0..count {
        helper.write_u8((i + 1) as u8);
    }
}

// ============================================================================
// Binary Missile Data - matches C++ SaveMissile (loadsave.cpp:1614-1662)
// ============================================================================

/// Engine-independent missile state matching the C++ `SaveMissile` layout.
#[derive(Debug, Clone, Default)]
pub struct BinaryMissileData {
    pub mitype: i32,
    pub position_x: i32,
    pub position_y: i32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub velocity_x: i32,
    pub velocity_y: i32,
    pub start_x: i32,
    pub start_y: i32,
    pub traveled_x: i32,
    pub traveled_y: i32,
    pub frame_group: i32,
    pub spllvl: i32,
    pub del_flag: bool,
    pub anim_type: u8,
    pub anim_flags: i32,
    pub anim_delay: i32,
    pub anim_len: i32,
    pub anim_width: i32,
    pub anim_width2: i32,
    pub anim_cnt: i32,
    pub anim_add: i32,
    pub anim_frame: i32,
    pub draw_flag: bool,
    pub light_flag: bool,
    pub pre_flag: bool,
    pub uniq_trans: u32,
    pub duration: i32,
    pub source: i32,
    pub caster: i32,
    pub dam: i32,
    pub hit_flag: bool,
    pub dist: i32,
    pub light_id: i32,
    pub rnd: i32,
    pub var1: i32,
    pub var2: i32,
    pub var3: i32,
    pub var4: i32,
    pub var5: i32,
    pub var6: i32,
    pub var7: i32,
    pub limit_reached: bool,
}

impl BinaryMissileData {
    /// Load from binary helper (matches C++ LoadMissile).
    pub fn from_binary(helper: &mut LoadHelper) -> Self {
        let mut m = Self::default();
        m.mitype = helper.next_le_i32();
        m.position_x = helper.next_le_i32();
        m.position_y = helper.next_le_i32();
        m.offset_x = helper.next_le_i32();
        m.offset_y = helper.next_le_i32();
        m.velocity_x = helper.next_le_i32();
        m.velocity_y = helper.next_le_i32();
        m.start_x = helper.next_le_i32();
        m.start_y = helper.next_le_i32();
        m.traveled_x = helper.next_le_i32();
        m.traveled_y = helper.next_le_i32();
        m.frame_group = helper.next_le_i32();
        m.spllvl = helper.next_le_i32();
        m.del_flag = helper.next_bool32();
        m.anim_type = helper.next_u8();
        helper.skip(3);
        m.anim_flags = helper.next_le_i32();
        helper.skip(4); // _miAnimData pointer
        m.anim_delay = helper.next_le_i32();
        m.anim_len = helper.next_le_i32();
        m.anim_width = helper.next_le_i32();
        m.anim_width2 = helper.next_le_i32();
        m.anim_cnt = helper.next_le_i32();
        m.anim_add = helper.next_le_i32();
        m.anim_frame = helper.next_le_i32();
        m.draw_flag = helper.next_bool32();
        m.light_flag = helper.next_bool32();
        m.pre_flag = helper.next_bool32();
        m.uniq_trans = helper.next_le_u32();
        m.duration = helper.next_le_i32();
        m.source = helper.next_le_i32();
        m.caster = helper.next_le_i32();
        m.dam = helper.next_le_i32();
        m.hit_flag = helper.next_bool32();
        m.dist = helper.next_le_i32();
        m.light_id = helper.next_le_i32();
        m.rnd = helper.next_le_i32();
        m.var1 = helper.next_le_i32();
        m.var2 = helper.next_le_i32();
        m.var3 = helper.next_le_i32();
        m.var4 = helper.next_le_i32();
        m.var5 = helper.next_le_i32();
        m.var6 = helper.next_le_i32();
        m.var7 = helper.next_le_i32();
        m.limit_reached = helper.next_bool32();
        m
    }

    /// Write to binary helper (matches C++ SaveMissile, loadsave.cpp:1614-1662).
    pub fn to_binary(&self, helper: &mut SaveHelper) {
        helper.write_le_i32(self.mitype);
        helper.write_le_i32(self.position_x);
        helper.write_le_i32(self.position_y);
        helper.write_le_i32(self.offset_x);
        helper.write_le_i32(self.offset_y);
        helper.write_le_i32(self.velocity_x);
        helper.write_le_i32(self.velocity_y);
        helper.write_le_i32(self.start_x);
        helper.write_le_i32(self.start_y);
        helper.write_le_i32(self.traveled_x);
        helper.write_le_i32(self.traveled_y);
        helper.write_le_i32(self.frame_group);
        helper.write_le_i32(self.spllvl);
        helper.write_le_u32(if self.del_flag { 1 } else { 0 });
        helper.write_u8(self.anim_type);
        helper.skip(3);
        helper.write_le_i32(self.anim_flags);
        helper.skip(4); // _miAnimData pointer
        helper.write_le_i32(self.anim_delay);
        helper.write_le_i32(self.anim_len);
        helper.write_le_i32(self.anim_width);
        helper.write_le_i32(self.anim_width2);
        helper.write_le_i32(self.anim_cnt);
        helper.write_le_i32(self.anim_add);
        helper.write_le_i32(self.anim_frame);
        helper.write_le_u32(if self.draw_flag { 1 } else { 0 });
        helper.write_le_u32(if self.light_flag { 1 } else { 0 });
        helper.write_le_u32(if self.pre_flag { 1 } else { 0 });
        helper.write_le_u32(self.uniq_trans);
        helper.write_le_i32(self.duration);
        helper.write_le_i32(self.source);
        helper.write_le_i32(self.caster);
        helper.write_le_i32(self.dam);
        helper.write_le_u32(if self.hit_flag { 1 } else { 0 });
        helper.write_le_i32(self.dist);
        helper.write_le_i32(self.light_id);
        helper.write_le_i32(self.rnd);
        helper.write_le_i32(self.var1);
        helper.write_le_i32(self.var2);
        helper.write_le_i32(self.var3);
        helper.write_le_i32(self.var4);
        helper.write_le_i32(self.var5);
        helper.write_le_i32(self.var6);
        helper.write_le_i32(self.var7);
        helper.write_le_u32(if self.limit_reached { 1 } else { 0 });
    }
}

/// Map an engine `SimpleMissileData` to the C++ `SaveMissile` structure.
///
/// The engine missile only tracks position / per-tick delta / damage / range;
/// the remaining fields take C++ defaults for a Firebolt cast by the player
/// (`mitype = MissileID::Firebolt`, `source/caster = 0/1`, `lightId = -1`,
/// `animAdd = 1`). `dam` is stored in 64x fixed-point like C++ `_midam`
/// (missiles.cpp `AddMissile`), so the engine display damage is shifted << 6.
pub fn simple_missile_to_binary(m: &SimpleMissileData) -> BinaryMissileData {
    let mut b = BinaryMissileData::default();
    b.mitype = 1; // MissileID::Firebolt
    b.position_x = m.x;
    b.position_y = m.y;
    b.velocity_x = m.dx;
    b.velocity_y = m.dy;
    b.start_x = m.x;
    b.start_y = m.y;
    b.dam = m.damage << 6;
    b.duration = m.range_left;
    b.source = 0; // player
    b.caster = 1; // TargetMonsters
    b.anim_add = 1;
    b.light_id = -1;
    b
}


// ============================================================================
// SaveGameData dungeon body (C++ loadsave.cpp:2808-2844)
// ============================================================================


/// Serialise the dungeon-only body of `SaveGameData` in the C++ order
/// (loadsave.cpp:2808-2844): active monster ids (BE u32) + `SaveMonster`
/// bodies, the missile active/available index arrays (125 each) + missile
/// bodies (not yet mapped; callers pass the fixed arrays only), the active +
/// available object id arrays (127 total) + `SaveObject` bodies, then the
/// light list (`SaveLighting`) and vision list.
pub fn write_dungeon_body(
    helper: &mut SaveHelper,
    active_ids: &[u32],
    active_monsters: &[BinaryMonsterData],
    monster_level: i8,
    experience: u16,
    to_hit: u8,
    to_hit_special: u8,
    missiles: &[BinaryMissileData],
    active_object_ids: &[i8],
    available_object_ids: &[i8],
    objects: &[BinaryObjectData],
    lights: &[(u8, BinaryLightData)],
    vision: &[BinaryLightData],
) {
    // ActiveMonsters (BE u32 ids): C++ writes the full MaxMonsters (200)
    // array (loadsave.cpp:2819); the caller supplies all 200 entries.
    for id in active_ids {
        helper.write_be_u32(*id);
    }
    for m in active_monsters {
        m.to_binary(helper, monster_level, experience, to_hit, to_hit_special);
    }
    // Missile index arrays + bodies (C++ loadsave.cpp:2822-2837): the active
    // array is 0..125, the available tail runs from the saved count to 125,
    // then a Skip of the saved count and one SaveMissile body per missile.
    let saved = missiles.len().min(MAX_MISSILES_FOR_SAVE);
    for i in 0..MAX_MISSILES_FOR_SAVE {
        helper.write_u8(i as u8);
    }
    for i in saved..MAX_MISSILES_FOR_SAVE {
        helper.write_u8(i as u8);
    }
    helper.skip(saved);
    for m in missiles.iter().take(saved) {
        m.to_binary(helper);
    }
    // Object id arrays (active then available, 127 total).
    for id in active_object_ids {
        helper.write_i8(*id);
    }
    for id in available_object_ids {
        helper.write_i8(*id);
    }
    for o in objects {
        o.to_binary(helper);
    }
    // Lights: count (BE i32) + the full ActiveLights array (MAXLIGHTS=32,
    // loadsave.cpp:2837) + one SaveLighting body per active light. Empty
    // slots keep the sequential slot ids (C++ ActiveLights is slot-ordered).
    const MAX_LIGHTS_FOR_SAVE: usize = 32;
    helper.write_be_i32(lights.len() as i32);
    let mut light_slot = 0usize;
    for (id, _) in lights {
        helper.write_u8(*id);
        light_slot += 1;
    }
    while light_slot < MAX_LIGHTS_FOR_SAVE {
        helper.write_u8(light_slot as u8);
        light_slot += 1;
    }
    for (_, l) in lights {
        l.to_binary(helper, false);
    }
    // Vision: C++ writes VisionId (= players+1) then count then one
    // SaveLighting per player.
    let vision_count = vision.len() as i32;
    helper.write_be_i32(vision_count + 1);
    helper.write_be_i32(vision_count);
    for v in vision {
        v.to_binary(helper, true);
    }
}

// ============================================================================
// C++ SavePlayer (loadsave.cpp:1251-1485) — full player serialisation
// ============================================================================

/// Fixed-size C++ item-name field (`ItemNameLength` = 64), zero-padded.
fn cpp_name_bytes(s: &str) -> [u8; 64] {
    let mut out = [0u8; 64];
    let bytes = s.as_bytes();
    let n = bytes.len().min(63);
    out[..n].copy_from_slice(&bytes[..n]);
    out
}

/// Map `item_dat::ItemType` (the C++ `ItemType` row type) to the C++
/// `ItemType` discriminant (itemdat.h:290-304).
fn item_dat_type_to_cpp(v: crate::game::item_dat::ItemType) -> i32 {
    use crate::game::item_dat::ItemType::*;
    match v {
        Misc => 0,
        Sword => 1,
        Axe => 2,
        Bow => 3,
        Mace => 4,
        Shield => 5,
        LightArmor => 6,
        Helm => 7,
        MediumArmor => 8,
        HeavyArmor => 9,
        Staff => 10,
        Gold => 11,
        Ring => 12,
        Amulet => 13,
        None => -1,
    }
}

/// Map the port's `ItemMiscId` to the C++ `item_misc_id` discriminant
/// (itemdat.h:512+) used by SaveItem's `_iMiscId` field.
fn item_misc_id_to_cpp(v: crate::game::items::ItemMiscId) -> i32 {
    use crate::game::items::ItemMiscId::*;
    match v {
        None => 0,
        Usable => 1,          // IMISC_USEFIRST
        FullHeal => 2,
        Heal => 3,
        Mana => 6,
        FullMana => 7,
        Elixir | ElixStr => 10, // IMISC_ELIXSTR
        ElixMag => 11,
        ElixDex => 12,
        ElixVit => 13,
        Rejuv => 18,
        FullRejuv => 19,
        Scroll => 21,
        ScrollT => 22,
        Staff => 23,
        Book => 24,
        Ring => 25,
        Amulet => 26,
        Unique => 27,
        _ => 0,
    }
}

/// Map a fully generated engine `items::Item` (C++ `SetupAllItems` output) to
/// the C++ `SaveItem` structure (loadsave.cpp:1158-1250). Every field the
/// engine models maps to its C++ twin; animation fields use C++-ish defaults
/// because the engine does not track item anims.
pub fn item_to_binary(item: &crate::game::items::Item) -> BinaryItemData {
    let mut b = BinaryItemData::default();
    b.seed = item.seed;
    b.create_info = item.create_info;
    // C++ SaveItem writes `Item::_itype` (the C++ ItemType discriminant,
    // itemdat.h:290-304); the port's ItemType enum uses different values, so
    // map explicitly. Gold is special-cased (item_index 0 with type Gold).
    b.item_type = if item.item_type == crate::game::items::ItemType::Gold {
        11 // ItemType::Gold
    } else {
        crate::game::item_dat::get_item_data(item.item_index as usize)
            .map(|d| item_dat_type_to_cpp(d.item_type))
            .unwrap_or(0)
    };
    b.position_x = item.position_x;
    b.position_y = item.position_y;
    // C++ ground-item animation: 16 frames, current frame = last (items.cpp
    // AddInitItems sets currentFrame = numberOfFrames - 1).
    // C++ ground-item animation frame count comes from the item's cursor
    // sprite (objcurs.cel): gold = 10, potions/misc = 16. The current frame
    // is numberOfFrames - 1 (items.cpp ItemNoFlippy / AddInitItems).
    let (anim_frames, anim_frame) = if item.item_type == crate::game::items::ItemType::Gold {
        (10, 9)
    } else {
        (16, 15)
    };
    b.anim_frames = anim_frames;
    b.anim_frame = anim_frame;
    // C++ ground items use SelectionRegion::Bottom (= 1).
    b.selection_region = 1;
    b.identified = item.identified;
    b.magical = item.quality as i8;
    b.name = cpp_name_bytes(&item.base_name).to_vec();
    b.iname = cpp_name_bytes(&item.name).to_vec();
    // C++ _iLoc (item_equip_type, items.h): the port's ItemEquipType values
    // are shifted by one (None = -1), so map to the C++ ILOC discriminant.
    b.loc = match item.equip_loc {
        crate::game::items::ItemEquipType::None => 0,
        v => v as i8 + 1,
    };
    b.class = item.item_class as u8;
    b.cursor = item.cursor as i32;
    b.value = item.value;
    b.ivalue = item.identified_value;
    b.min_dam = item.min_damage as i32;
    b.max_dam = item.max_damage as i32;
    b.ac = item.armor_class as i32;
    b.flags = item.special_flags.0;
    b.misc_id = item_misc_id_to_cpp(item.misc_id);
    // C++ SaveItem writes _iSpell as int8_t; SPL_NULL = 0 (spelldat.h).
    b.spell = if item.spell <= 0 { 0 } else { item.spell as i32 };
    b.charges = item.charges;
    b.max_charges = item.max_charges;
    b.durability = item.durability;
    b.max_dur = item.max_durability;
    b.pl_dam = item.bonus_damage as i32;
    b.pl_to_hit = item.bonus_to_hit as i32;
    b.pl_ac = item.bonus_ac as i32;
    b.pl_str = item.bonus_str as i32;
    b.pl_mag = item.bonus_mag as i32;
    b.pl_dex = item.bonus_dex as i32;
    b.pl_vit = item.bonus_vit as i32;
    b.pl_fr = item.resist_fire as i32;
    b.pl_lr = item.resist_lightning as i32;
    b.pl_mr = item.resist_magic as i32;
    b.pl_mana = item.bonus_mana as i32;
    b.pl_hp = item.bonus_hp as i32;
    b.pl_dam_mod = item.bonus_damage_mod as i32;
    b.pl_get_hit = item.bonus_get_hit as i32;
    b.pl_light = item.bonus_light as i32;
    b.spl_lvl_add = item.spell_level_add;
    b.request = item.request;
    // C++ writes UniqueItems[_iUid].mappingId; _iUid < 0 (no unique) => 0.
    b.unique_id = if item.unique_id < 0 { 0 } else { item.unique_id };
    b.f_min_dam = item.fire_min_dam as i32;
    b.f_max_dam = item.fire_max_dam as i32;
    b.l_min_dam = item.lightning_min_dam as i32;
    b.l_max_dam = item.lightning_max_dam as i32;
    b.pl_en_ac = item.bonus_energy_ac as i32;
    b.pre_power = item.prefix_power as i8;
    b.suf_power = item.suffix_power as i8;
    b.v_add1 = item.value_add1;
    b.v_mult1 = item.value_mult1;
    b.v_add2 = item.value_add2;
    b.v_mult2 = item.value_mult2;
    b.min_str = item.required_str;
    b.min_mag = item.required_mag;
    b.min_dex = item.required_dex;
    b.stat_flag = item.stat_flag;
    b.item_idx = item.item_index as i32;
    b
}

/// Map a simplified engine `PlayerItem` to the C++ `SaveItem` structure.
/// The engine only tracks item id / equip state / weapon type, so the rest of
/// the fields keep C++ defaults; `item_idx` carries the C++ item index.
fn player_item_to_binary(item: &crate::game::player_exact::PlayerItem) -> BinaryItemData {
    // Real drops carry the full generated item (C++ SetupAllItems output);
    // serialise every modelled field via item_to_binary so the SaveItem
    // section keeps the seed/affixes/unique/name like C++.
    if let Some(full) = &item.full {
        let mut b = item_to_binary(full);
        b.item_idx = item.item_id;
        return b;
    }
    let mut b = BinaryItemData::default();
    b.item_idx = item.item_id;
    // C++ fills Item fields from AllItemsList at item creation (GetItemAttrs);
    // the engine's PlayerItem only tracks the TSV row, so pull the base
    // attributes from the authoritative itemdat.tsv row. Empty slots
    // (item_id == 0) keep the all-default stub (C++ writes empty items with
    // idx = IDI_GOLD and default fields).
    if item.item_id > 0 {
        if let Some(d) = crate::game::item_dat::get_item_data(item.item_id as usize) {
            b.item_type = d.item_type as i32;
            b.class = d.class as u8;
            b.loc = d.equip_type as i8;
            b.cursor = d.cursor_graphic as i32;
            b.min_dam = d.min_damage as i32;
            b.max_dam = d.max_damage as i32;
            b.ac = d.min_ac as i32; // C++ randomises in [min_ac, max_ac]
            b.durability = d.durability as i32;
            b.max_dur = d.durability as i32;
            b.flags = d.special_effects.0;
            b.misc_id = d.misc_id as i32;
            b.spell = d.spell as i8 as i32;
            b.min_str = d.min_str as i8;
            b.min_mag = d.min_mag;
            b.min_dex = d.min_dex as i8;
            b.value = d.value as i32;
            b.ivalue = d.value as i32;
            b.name = cpp_name_bytes(d.name).to_vec();
            b.iname = cpp_name_bytes(d.name).to_vec();
        }
    }
    b
}

/// C++ `SavePlayer` (loadsave.cpp:1251-1485): serialises the full player
/// state. The layout is exact for Diablo / 17-level / non-Hellfire (21600
/// bytes). Fields the Rust `player_exact::Player` models are written from the
/// player; the remainder use C++ defaults so the byte offsets and total size
/// match, letting callers slice the correct segments out of a real save.
pub fn save_player(helper: &mut SaveHelper, player: &crate::game::player_exact::Player, is_hellfire: bool) {
    use crate::game::player_exact::{Direction, PlayerItem};
    let ex = &player.save_extra;

    // Mode + walk path + flags.
    helper.write_le_i32(player._p_mode as u8 as i32);
    for i in 0..25 {
        // C++ walkpath default is WALK_NONE = -1 (player.h:56); the port's
        // Direction::None = 8 serialises as -1 for the save.
        let v = player.walk_path.get(i).copied().unwrap_or(Direction::None);
        helper.write_i8(if v == Direction::None {
            -1
        } else {
            v as u8 as i8
        });
    }
    helper.write_u8(if player.plr_active { 1 } else { 0 }); // plractive
    helper.skip(2);
    helper.write_le_i32(player.dest_action as i32);
    helper.write_le_i32(player.dest_param1);
    helper.write_le_i32(player.dest_param2);
    helper.write_le_i32(player.dest_param3);
    helper.write_le_i32(player.dest_param4);
    helper.write_le_u32(player.plr_level as u32);
    // Position (tile, future, target, last, old).
    helper.write_le_i32(player.position.x);
    helper.write_le_i32(player.position.y);
    helper.write_le_i32(ex.position_future.x);
    helper.write_le_i32(ex.position_future.y);
    helper.write_le_i32(ex.position_target.x);
    helper.write_le_i32(ex.position_target.y);
    helper.write_le_i32(ex.position_last.x);
    helper.write_le_i32(ex.position_last.y);
    helper.write_le_i32(ex.position_old.x);
    helper.write_le_i32(ex.position_old.y);
    // Offset / velocity (C++ CalculateWalkingOffset etc.; zero when idle).
    helper.write_le_i32(ex.offset_dx);
    helper.write_le_i32(ex.offset_dy);
    helper.write_le_i32(ex.velocity_dx);
    helper.write_le_i32(ex.velocity_dy);
    helper.write_le_i32(player._p_dir as u8 as i32); // _pdir
    helper.skip(4);
    helper.write_le_u32(ex.pgfxnum); // _pgfxnum
    helper.skip(4); // _pAnimData pointer
    helper.write_le_i32(ex.ticks_per_frame); // ticksPerFrame - 1
    helper.write_le_i32(ex.tick_counter);
    helper.write_le_i32(ex.number_of_frames);
    helper.write_le_i32(ex.current_frame); // currentFrame + 1
    helper.write_le_i32(ex.anim_width);
    helper.write_le_i32(ex.width2);
    helper.skip(4); // _peflag
    helper.write_le_i32(player.light_id);
    helper.write_le_i32(1); // _pvid

    // Spells (raw C++ SpellID values so a loaded save round-trips).
    helper.write_le_i32(ex.queued_spell_id);
    helper.write_i8(ex.queued_spell_type as i8);
    helper.write_i8(ex.queued_spell_from as i8);
    helper.skip(2);
    helper.write_le_i32(ex.inventory_spell);
    helper.skip(1); // _pTSplType
    helper.skip(3);
    helper.write_le_i32(ex.r_spell); // _pRSpell
    helper.write_i8(ex.r_spl_type as i8); // _pRSplType
    helper.skip(3);
    helper.write_le_i32(ex.sbk_spell); // _pSBkSpell
    helper.skip(1); // _pSBkSplType

    for &lvl in player._p_spl_lvl.iter() {
        helper.write_u8(lvl);
    }
    helper.skip(7);
    helper.write_le_u64(player._p_mem_spells);
    helper.write_le_u64(player._p_abl_spells);
    helper.write_le_u64(player._p_scrl_spells);
    helper.write_u8(ex.spell_flags); // _pSpellFlags
    helper.skip(3);
    for &hk in ex.hotkeys.iter() {
        helper.write_le_i32(hk);
    }
    for &t in ex.hotkey_types.iter() {
        helper.write_u8(t);
    }

    helper.write_le_i32(if ex.uses_ranged_weapon { 1 } else { 0 });
    helper.write_u8(if player._p_block_flag { 1 } else { 0 });
    helper.write_u8(if player._p_invincible { 1 } else { 0 });
    helper.write_i8(player._p_light_rad);
    helper.write_u8(if ex.lvl_changing { 1 } else { 0 }); // _pLvlChanging

    helper.write_bytes(&player._p_name);
    helper.write_i8(player._p_class as i8);
    helper.skip(3);
    helper.write_le_i32(player._p_strength);
    helper.write_le_i32(player._p_base_str);
    helper.write_le_i32(player._p_magic);
    helper.write_le_i32(player._p_base_mag);
    helper.write_le_i32(player._p_dexterity);
    helper.write_le_i32(player._p_base_dex);
    helper.write_le_i32(player._p_vitality);
    helper.write_le_i32(player._p_base_vit);
    helper.write_le_i32(player._p_stat_pts);
    helper.write_le_i32(player._p_damage_mod);

    helper.write_le_i32(ex.base_to_block); // baseToBlock
    helper.write_le_i32(player._p_hp_base);
    helper.write_le_i32(player._p_max_hp_base);
    helper.write_le_i32(player._p_hit_points);
    helper.write_le_i32(player._p_max_hp);
    helper.skip(4); // _pHPPer
    helper.write_le_i32(player._p_mana_base);
    helper.write_le_i32(player._p_max_mana_base);
    helper.write_le_i32(player._p_mana);
    helper.write_le_i32(player._p_max_mana);
    helper.skip(4); // _pManaPer
    helper.write_u8(player._p_level);
    helper.skip(1); // _pMaxLevel
    helper.skip(2);
    helper.write_le_u32(player._p_experience);
    helper.skip(4); // _pMaxExp
    helper.write_le_u32(ex.next_exp_threshold); // getNextExperienceThreshold
    helper.write_i8(player._p_armor_class);
    helper.write_i8(player._p_mag_resist);
    helper.write_i8(player._p_fire_resist);
    helper.write_i8(player._p_lght_resist);
    helper.write_le_i32(player._p_gold);
    helper.write_le_u32(if ex.infra_flag { 1 } else { 0 }); // _pInfraFlag

    helper.write_le_i32(ex.position_temp.x); // tempPositionX
    helper.write_le_i32(ex.position_temp.y); // tempPositionY
    helper.write_le_i32(player._p_temp_direction as u8 as i32); // tempDirection
    helper.write_le_i32(ex.queued_spell_level);
    helper.skip(4); // _pVar5
    helper.write_le_i32(ex.offset2_dx);
    helper.write_le_i32(ex.offset2_dy);
    helper.skip(4); // _pVar8

    // Visited levels: 17 classic levels.
    for i in 0..17 {
        helper.write_u8(if player._p_lvl_visited.get(i).copied().unwrap_or(false) { 1 } else { 0 });
    }
    for i in 0..17 {
        helper.write_u8(if player._p_set_lvl_visited.get(i).copied().unwrap_or(false) { 1 } else { 0 });
    }
    helper.skip(2);

    // Animation pointer blocks (C++ skips pointers, writes frame counts).
    helper.skip(4); // _pGFXLoad
    helper.skip(32); // _pNAnim pointers (8)
    helper.write_le_i32(ex.n_frames); // _pNFrames
    helper.skip(4); // _pNWidth
    helper.skip(32); // _pWAnim
    helper.write_le_i32(ex.w_frames); // _pWFrames
    helper.skip(4); // _pWWidth
    helper.skip(32); // _pAAnim
    helper.write_le_i32(ex.a_frames); // _pAFrames
    helper.skip(4); // _pAWidth
    helper.write_le_i32(ex.a_fnum); // _pAFNum
    helper.skip(32); // _pLAnim
    helper.skip(32); // _pFAnim
    helper.skip(32); // _pTAnim
    helper.write_le_i32(ex.s_frames); // _pSFrames
    helper.skip(4); // _pSWidth
    helper.write_le_i32(ex.s_fnum); // _pSFNum
    helper.skip(32); // _pHAnim
    helper.write_le_i32(ex.h_frames); // _pHFrames
    helper.skip(4); // _pHWidth
    helper.skip(32); // _pDAnim
    helper.write_le_i32(ex.d_frames); // _pDFrames
    helper.skip(4); // _pDWidth
    helper.skip(32); // _pBAnim
    helper.write_le_i32(ex.b_frames); // _pBFrames
    helper.skip(4); // _pBWidth

    // Items: InvBody (7) + InvList (40) + SpdList (8) + HoldItem (1).
    let mut slot = 0usize;
    for item in player.inv_body.iter() {
        save_player_item(helper, ex, slot, item, is_hellfire);
        slot += 1;
    }
    for item in player.inv_list.iter() {
        save_player_item(helper, ex, slot, item, is_hellfire);
        slot += 1;
    }
    helper.write_le_i32(player._p_num_inv);
    // C++ SavePlayer writes InvGrid[40]: each cell is the InvList slot + 1
    // of the occupying item (negative for non-top-left cells of multi-cell
    // items), 0 = empty (inv.cpp AddItemToInvGrid).
    for &cell in player.inv_grid.iter() {
        helper.write_i8(cell);
    }
    for item in player.spd_list.iter() {
        save_player_item(helper, ex, slot, item, is_hellfire);
        slot += 1;
    }
    save_player_item(helper, ex, slot, &player.hold_item, is_hellfire);

    // Item bonus fields.
    helper.write_le_i32(player._p_i_min_dam);
    helper.write_le_i32(player._p_i_max_dam);
    helper.write_le_i32(player._p_i_ac);
    helper.write_le_i32(player._p_i_bonus_dam);
    helper.write_le_i32(player._p_i_bonus_to_hit);
    helper.write_le_i32(player._p_i_bonus_ac);
    helper.write_le_i32(player._p_i_bonus_dam_mod);
    helper.skip(4);
    helper.write_le_u64(player._p_i_spells);
    helper.write_le_i32(ex.i_flags); // _pIFlags
    helper.write_le_i32(player._p_i_get_hit);
    helper.write_i8(ex.i_spl_lvl_add); // _pISplLvlAdd
    helper.skip(1); // _pISplCost
    helper.skip(2);
    helper.skip(4); // _pISplDur
    helper.write_le_i32(player._p_i_en_ac);
    helper.write_le_i32(player._p_i_f_min_dam);
    helper.write_le_i32(player._p_i_f_max_dam);
    helper.write_le_i32(player._p_i_l_min_dam);
    helper.write_le_i32(player._p_i_l_max_dam);
    helper.write_le_i32(ex.oil_type); // _pOilType
    helper.write_u8(ex.town_warps);
    helper.write_u8(ex.dung_msgs);
    helper.write_u8(ex.lvl_load);
    helper.write_u8(if is_hellfire { ex.dung_msgs2 } else { 0 });
    helper.write_u8(if ex.mana_shield { 1 } else { 0 });
    helper.write_u8(if ex.original_cathedral { 1 } else { 0 });
    helper.skip(2);
    helper.write_le_u16(ex.w_reflections);
    helper.skip(14);
    helper.write_le_u32(ex.diablo_kill_level);
    helper.write_le_u32(ex.difficulty);
    helper.write_le_u32(ex.dam_ac_flags);
    helper.skip(20);
}

/// Write one SaveItem slot. Prefers the loaded C++ binary body so a loaded
/// save round-trips byte-for-byte; falls back to the engine-modelled item.
fn save_player_item(
    helper: &mut SaveHelper,
    ex: &crate::game::player_exact::PlayerSaveExtra,
    slot: usize,
    item: &crate::game::player_exact::PlayerItem,
    is_hellfire: bool,
) {
    match ex.save_items.get(slot).and_then(|o| o.as_ref()) {
        Some(b) => b.to_binary(helper, is_hellfire),
        None => player_item_to_binary(item).to_binary(helper, is_hellfire),
    }
}

/// C++ `LoadPlayer` (loadsave.cpp:391-467): reads the 21680-byte player block
/// from the `game` entry into the engine `Player` (modelled fields) and the
/// `PlayerSaveExtra` round-trip state (everything else, kept raw so
/// `save_player` reproduces the bytes). Must stay in lock-step with
/// `save_player` above.
pub fn load_player_from_game(helper: &mut LoadHelper, player: &mut crate::game::player_exact::Player) {
    use crate::game::player_exact::{Direction, HeroClass, PlayerMode};
    let ex = &mut player.save_extra;

    player._p_mode = PlayerMode::try_from(helper.next_le_i32() as u8).unwrap_or(PlayerMode::Stand);
    for i in 0..25 {
        let v = helper.next_i8();
        player.walk_path[i] = if v == -1 { Direction::None } else { Direction::try_from(v as u8).unwrap_or(Direction::None) };
    }
    player.plr_active = helper.next_bool8();
    helper.skip(2);
    player.dest_action = crate::game::player_exact::ActionType::try_from(helper.next_le_i32()).unwrap_or(crate::game::player_exact::ActionType::None);
    player.dest_param1 = helper.next_le_i32();
    player.dest_param2 = helper.next_le_i32();
    player.dest_param3 = helper.next_le_i32();
    player.dest_param4 = helper.next_le_i32();
    player.plr_level = helper.next_le_u32() as u8;
    player.position.x = helper.next_le_i32();
    player.position.y = helper.next_le_i32();
    ex.position_future = crate::game::types::Point::new(helper.next_le_i32(), helper.next_le_i32());
    ex.position_target = crate::game::types::Point::new(helper.next_le_i32(), helper.next_le_i32());
    ex.position_last = crate::game::types::Point::new(helper.next_le_i32(), helper.next_le_i32());
    ex.position_old = crate::game::types::Point::new(helper.next_le_i32(), helper.next_le_i32());
    ex.offset_dx = helper.next_le_i32();
    ex.offset_dy = helper.next_le_i32();
    ex.velocity_dx = helper.next_le_i32();
    ex.velocity_dy = helper.next_le_i32();
    player._p_dir = Direction::try_from(helper.next_le_i32() as u8).unwrap_or(Direction::South);
    helper.skip(4);
    ex.pgfxnum = helper.next_le_u32();
    helper.skip(4); // _pAnimData pointer
    ex.ticks_per_frame = helper.next_le_i32();
    ex.tick_counter = helper.next_le_i32();
    ex.number_of_frames = helper.next_le_i32();
    ex.current_frame = helper.next_le_i32();
    ex.anim_width = helper.next_le_i32();
    ex.width2 = helper.next_le_i32();
    helper.skip(4); // _peflag
    player.light_id = helper.next_le_i32();
    helper.skip(4); // _pvid

    ex.queued_spell_id = helper.next_le_i32();
    ex.queued_spell_type = helper.next_u8();
    ex.queued_spell_from = helper.next_u8();
    helper.skip(2);
    ex.inventory_spell = helper.next_le_i32();
    helper.skip(1); // _pTSplType
    helper.skip(3);
    ex.r_spell = helper.next_le_i32();
    ex.r_spl_type = helper.next_u8();
    helper.skip(3);
    ex.sbk_spell = helper.next_le_i32();
    helper.skip(1); // _pSBkSplType

    for lvl in player._p_spl_lvl.iter_mut() {
        *lvl = helper.next_u8();
    }
    helper.skip(7);
    player._p_mem_spells = helper.next_le_u64();
    player._p_abl_spells = helper.next_le_u64();
    player._p_scrl_spells = helper.next_le_u64();
    ex.spell_flags = helper.next_u8();
    helper.skip(3);
    for hk in ex.hotkeys.iter_mut() {
        *hk = helper.next_le_i32();
    }
    for t in ex.hotkey_types.iter_mut() {
        *t = helper.next_u8();
    }

    ex.uses_ranged_weapon = helper.next_le_i32() != 0;
    player._p_block_flag = helper.next_bool8();
    player._p_invincible = helper.next_bool8();
    player._p_light_rad = helper.next_i8();
    ex.lvl_changing = helper.next_bool8();

    let name = helper.next_bytes(crate::game::player_exact::PLAYER_NAME_LENGTH);
    player._p_name[..name.len().min(crate::game::player_exact::PLAYER_NAME_LENGTH)].copy_from_slice(&name[..name.len().min(crate::game::player_exact::PLAYER_NAME_LENGTH)]);
    player._p_class = HeroClass::try_from(helper.next_i8() as u8).unwrap_or(HeroClass::Warrior);
    helper.skip(3);
    player._p_strength = helper.next_le_i32();
    player._p_base_str = helper.next_le_i32();
    player._p_magic = helper.next_le_i32();
    player._p_base_mag = helper.next_le_i32();
    player._p_dexterity = helper.next_le_i32();
    player._p_base_dex = helper.next_le_i32();
    player._p_vitality = helper.next_le_i32();
    player._p_base_vit = helper.next_le_i32();
    player._p_stat_pts = helper.next_le_i32();
    player._p_damage_mod = helper.next_le_i32();

    ex.base_to_block = helper.next_le_i32();
    player._p_hp_base = helper.next_le_i32();
    player._p_max_hp_base = helper.next_le_i32();
    player._p_hit_points = helper.next_le_i32();
    player._p_max_hp = helper.next_le_i32();
    helper.skip(4); // _pHPPer
    player._p_mana_base = helper.next_le_i32();
    player._p_max_mana_base = helper.next_le_i32();
    player._p_mana = helper.next_le_i32();
    player._p_max_mana = helper.next_le_i32();
    helper.skip(4); // _pManaPer
    player._p_level = helper.next_u8();
    ex.max_level = helper.next_u8();
    helper.skip(2);
    player._p_experience = helper.next_le_u32();
    ex.max_exp = helper.next_le_u32();
    ex.next_exp_threshold = helper.next_le_u32();
    player._p_armor_class = helper.next_i8();
    player._p_mag_resist = helper.next_i8();
    player._p_fire_resist = helper.next_i8();
    player._p_lght_resist = helper.next_i8();
    player._p_gold = helper.next_le_i32();
    ex.infra_flag = helper.next_le_u32() != 0;

    ex.position_temp = crate::game::types::Point::new(helper.next_le_i32(), helper.next_le_i32());
    player._p_temp_direction = Direction::try_from(helper.next_le_i32() as u8).unwrap_or(Direction::South);
    ex.queued_spell_level = helper.next_le_i32();
    ex.var5 = helper.next_le_i32();
    ex.offset2_dx = helper.next_le_i32();
    ex.offset2_dy = helper.next_le_i32();
    ex.var8 = helper.next_le_i32();

    for i in 0..17 {
        if let Some(v) = player._p_lvl_visited.get_mut(i) {
            *v = helper.next_bool8();
        } else {
            helper.skip(1);
        }
    }
    for i in 0..17 {
        if let Some(v) = player._p_set_lvl_visited.get_mut(i) {
            *v = helper.next_bool8();
        } else {
            helper.skip(1);
        }
    }
    helper.skip(2);

    helper.skip(4); // _pGFXLoad
    helper.skip(32); // _pNAnim
    ex.n_frames = helper.next_le_i32();
    helper.skip(4); // _pNWidth
    helper.skip(32); // _pWAnim
    ex.w_frames = helper.next_le_i32();
    helper.skip(4); // _pWWidth
    helper.skip(32); // _pAAnim
    ex.a_frames = helper.next_le_i32();
    helper.skip(4); // _pAWidth
    ex.a_fnum = helper.next_le_i32();
    helper.skip(32); // _pLAnim
    helper.skip(32); // _pFAnim
    helper.skip(32); // _pTAnim
    ex.s_frames = helper.next_le_i32();
    helper.skip(4); // _pSWidth
    ex.s_fnum = helper.next_le_i32();
    helper.skip(32); // _pHAnim
    ex.h_frames = helper.next_le_i32();
    helper.skip(4); // _pHWidth
    helper.skip(32); // _pDAnim
    ex.d_frames = helper.next_le_i32();
    helper.skip(4); // _pDWidth
    helper.skip(32); // _pBAnim
    ex.b_frames = helper.next_le_i32();
    helper.skip(4); // _pBWidth

    // Items: InvBody (7) + InvList (40) + SpdList (8) + HoldItem (1).
    let mut slot = 0usize;
    let mut read_item = |helper: &mut LoadHelper, ex: &mut crate::game::player_exact::PlayerSaveExtra, slot: usize| {
        let item = BinaryItemData::from_binary(helper, false);
        if let Some(cell) = ex.save_items.get_mut(slot) {
            *cell = Some(item);
        }
    };
    for _ in 0..7 {
        read_item(helper, ex, slot);
        slot += 1;
    }
    for _ in 0..40 {
        read_item(helper, ex, slot);
        slot += 1;
    }
    player._p_num_inv = helper.next_le_i32();
    for cell in player.inv_grid.iter_mut() {
        *cell = helper.next_i8();
    }
    for _ in 0..8 {
        read_item(helper, ex, slot);
        slot += 1;
    }
    read_item(helper, ex, slot); // HoldItem

    player._p_i_min_dam = helper.next_le_i32();
    player._p_i_max_dam = helper.next_le_i32();
    player._p_i_ac = helper.next_le_i32();
    player._p_i_bonus_dam = helper.next_le_i32();
    player._p_i_bonus_to_hit = helper.next_le_i32();
    player._p_i_bonus_ac = helper.next_le_i32();
    player._p_i_bonus_dam_mod = helper.next_le_i32();
    helper.skip(4);
    player._p_i_spells = helper.next_le_u64();
    ex.i_flags = helper.next_le_i32();
    player._p_i_get_hit = helper.next_le_i32();
    ex.i_spl_lvl_add = helper.next_i8();
    ex.i_spl_cost = helper.next_u8();
    helper.skip(2);
    ex.i_spl_dur = helper.next_le_i32();
    player._p_i_en_ac = helper.next_le_i32();
    player._p_i_f_min_dam = helper.next_le_i32();
    player._p_i_f_max_dam = helper.next_le_i32();
    player._p_i_l_min_dam = helper.next_le_i32();
    player._p_i_l_max_dam = helper.next_le_i32();
    ex.oil_type = helper.next_le_i32();
    ex.town_warps = helper.next_u8();
    ex.dung_msgs = helper.next_u8();
    ex.lvl_load = helper.next_u8();
    ex.dung_msgs2 = helper.next_u8();
    ex.mana_shield = helper.next_bool8();
    ex.original_cathedral = helper.next_bool8();
    helper.skip(2);
    ex.w_reflections = helper.next_le_u16();
    helper.skip(14);
    ex.diablo_kill_level = helper.next_le_u32();
    ex.difficulty = helper.next_le_u32();
    ex.dam_ac_flags = helper.next_le_u32();
    helper.skip(20);
}
// ============================================================================
// SaveGameData orchestration (C++ loadsave.cpp:2762-2935)
// ============================================================================

/// Serialize a full `game` entry for the classic (non-Hellfire, spawn)
/// format, following the C++ `SaveGameData` section order:
///   header (43B) + level seeds (17×8B) + player (1266B PlayerPack) +
///   16 quests (44B each) + 4 portals (24B each) + kill counts (800B) +
///   dungeon body (active monsters/missiles/objects + dropped items) +
///   128 unique flags + dLight/dFlags/dPlayer grids (112×112 each) +
///   dropped-item locations + dungeon-only grids (dMonster/dCorpse/dObject/
///   dLight/dPreLight/AutomapView/missile grid) + premium items + trailing
///   misc.
///
/// The Rust engine does not yet map monster/missile/object/dropped-item
/// state into the C++ packs, so callers pass those blobs in (`dungeon_body`,
/// `dropped_items`, `dungeon_only_grids`); the orchestration guarantees the
/// byte layout and ordering for the sections that are implemented.
pub fn write_game_data_v3(
    header: &CppGameHeader,
    seeds: &[(u32, u32)],
    player_pack: &[u8],
    quests: &[crate::game::quest_new::Quest],
    return_state: (i32, i32, i32, i32),
    portals: &[(bool, (i32, i32), i32, i32, bool)],
    kill_counts: &[i32],
    dungeon_body: &[u8],
    dropped_items: &[u8],
    unique_flags: &[bool],
    dlight: &[u8],
    dflags: &[u8],
    dplayer: &[u8],
    dropped_locations: &[u8],
    dungeon_only_grids: &[u8],
    premium: &[u8],
    misc: &[u8],
) -> Vec<u8> {
    let mut h = SaveHelper::new(320 * 1024);
    h.write_bytes(&header.write());
    h.write_bytes(&CppGameHeader::write_level_seeds(seeds));
    h.write_bytes(player_pack);
    for q in quests {
        write_quest(
            &mut h,
            q,
            (return_state.0, return_state.1),
            return_state.2,
            return_state.3,
        );
    }
    for p in portals {
        write_portal(&mut h, p.0, p.1, p.2, p.3, p.4);
    }
    write_kill_counts(&mut h, kill_counts);
    h.write_bytes(dungeon_body);
    h.write_bytes(dropped_items);
    write_unique_flags(&mut h, unique_flags);
    write_grid_u8(&mut h, dlight);
    write_grid_u8(&mut h, dflags);
    write_grid_u8(&mut h, dplayer);
    // C++ SaveDroppedItemLocations (loadsave.cpp:2909-2914): one u8 per tile
    // with the 1-based save position of the item there (0 = empty).
    h.write_bytes(dropped_locations);
    h.write_bytes(dungeon_only_grids);
    h.write_bytes(premium);
    h.write_bytes(misc);
    h.into_data()
}

// ============================================================================
// SaveLevel / LoadLevel — full per-level persistence
//
// Mirrors C++ `SaveLevel`/`LoadLevel` from Source/loadsave.cpp (lines
// 1928-2106). The C++ code writes a single binary blob per level into the
// MPQ archive containing, in order:
//   1. dCorpse[MAXDUNX][MAXDUNY]  (i8, dungeon only)
//   2. ActiveMonsterCount         (BE i32)
//   3. ActiveItemCount            (BE i32)
//   4. ActiveObjectCount          (BE i32)
//   5. ActiveMonsters[]           (BE u32 each, dungeon only)
//   6. Monster bodies             (SaveMonster, dungeon only)
//   7. ActiveObjects / AvailableObjects (i8, dungeon only)
//   8. Object bodies              (SaveObject, dungeon only)
//   9. Dropped items              (SaveItem per active item)
//  10. dFlags[MAXDUNX][MAXDUNY]   (u8)
//  11. dItem[MAXDUNX][MAXDUNY]    (u8 indexes)
//  12. dMonster[MAXDUNX][MAXDUNY] (BE i32, dungeon only)
//  13. dObject[MAXDUNX][MAXDUNY]  (i8, dungeon only)
//  14. dLight[MAXDUNX][MAXDUNY]   (u8, dungeon only)
//  15. dPreLight[MAXDUNX][MAXDUNY](u8, dungeon only)
//  16. AutomapView[DMAXX][DMAXY]  (u8, dungeon only)
//
// Because the Rust engine does not yet expose per-tile `dPiece`/`dMonster`/
// `dObject` arrays (the dungeon is a `DungeonMap` of `TileType`s), we serialise
// the *engine-level* level state — the active monsters, objects, and dropped
// ground items — in the same on-disk field order and sizes as the C++ writer
// uses for those sections, and write zeros for the tile arrays the Rust side
// regenerates from the seed on load. This keeps the binary layout aligned with
// vanilla Diablo saves while round-tripping the subsystems the Rust engine
// actually persists.
// ============================================================================

/// Dungeon grid dimensions (C++ MAXDUNX/MAXDUNY/DMAXX/DMAXY).
pub const MAXDUNX: usize = 112;
pub const MAXDUNY: usize = 112;
pub const DMAXX: usize = 40;
pub const DMAXY: usize = 40;

/// Engine-side snapshot of the current level — what the Rust `GameState`
/// actually persists. Each field is type-erased so `loadsave.rs` does not need
/// to import the engine types directly.
#[derive(Debug, Clone, Default)]
pub struct LevelSnapshotData {
    /// Active monsters, in active-slot order.
    pub monsters: Vec<BinaryMonsterData>,
    /// Per-monster derived write params (level/exp/toHit/toHitSpecial).
    pub monster_params: Vec<MonsterWriteParams>,
    /// Active objects.
    pub objects: Vec<BinaryObjectData>,
    /// Dropped items lying on the floor (engine `GroundItem`s mapped to a
    /// compact binary form — see `FloorItemData`).
    pub floor_items: Vec<FloorItemData>,
    /// `true` when the current level is a town (skips the dungeon-only tile
    /// arrays, matching C++ `leveltype != DTYPE_TOWN` branches).
    pub is_town: bool,
}

/// Per-monster derived values the C++ writer computes live from
/// `Monster::level/exp/toHit/toHitSpecial(difficulty)`. We pass them in
/// explicitly so the writer stays byte-compatible.
#[derive(Debug, Clone, Copy, Default)]
pub struct MonsterWriteParams {
    pub level: i8,
    pub experience: u16,
    pub to_hit: u8,
    pub to_hit_special: u8,
}

/// Compact on-floor item record. The full C++ `Item` is 368/372 bytes; the
/// Rust engine's `GroundItem` carries only position + kind, so we serialise a
/// small fixed-size record. The Rust-side `SaveLevel` does **not** claim
/// vanilla-diablo item-binary compatibility — only engine-level round-trip.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct FloorItemData {
    pub x: i32,
    pub y: i32,
    /// `GroundItemType` discriminant (0=Gold, 1=Healing, 2=Mana).
    pub kind: u8,
}

impl LevelSnapshotData {
    /// Serialise this level snapshot into the C++-compatible binary form.
    ///
    /// Returns the binary blob. Tile-array sections the Rust engine does not
    /// own are emitted as zeros at the canonical sizes so the field order and
    /// total layout match `SaveLevel` in Source/loadsave.cpp.
    pub fn to_level_blob(&self) -> Vec<u8> {
        let mut file = SaveHelper::new(64 * 1024);

        // (1) dCorpse — dungeon only.
        if !self.is_town {
            for _ in 0..(MAXDUNX * MAXDUNY) {
                file.write_i8(0);
            }
        }

        // (2-4) Active counts (BE i32 each, C++ order: monsters, items, objects).
        let monster_count = self.monsters.len() as i32;
        let item_count = self.floor_items.len() as i32;
        let object_count = self.objects.len() as i32;
        file.write_be_i32(monster_count);
        file.write_be_i32(item_count);
        file.write_be_i32(object_count);

        // (5-6) Active monster ids + monster bodies — dungeon only.
        if !self.is_town {
            for i in 0..self.monsters.len() {
                file.write_be_u32(i as u32);
            }
            for (m, p) in self.monsters.iter().zip(self.monster_params.iter()) {
                m.to_binary(&mut file, p.level, p.experience, p.to_hit, p.to_hit_special);
            }

            // (7) ActiveObjects / AvailableObjects (i8 each, MAX_OBJECTS).
            // We mark the active objects as 0..count and the available slots as
            // -1 (C++ uses -1 sentinels for free slots).
            for i in 0..MAX_OBJECTS {
                let active_idx = if i < object_count as usize { i as i8 } else { -1 };
                file.write_i8(active_idx);
            }
            for i in 0..MAX_OBJECTS {
                let avail_idx = if i >= object_count as usize { i as i8 } else { -1 };
                file.write_i8(avail_idx);
            }

            // (8) Object bodies.
            for o in &self.objects {
                o.to_binary(&mut file);
            }
        }

        // (9) Dropped items — compact engine form (3 u8/i32 fields).
        // The C++ writer emits full 368-byte Item records; we emit a tagged
        // length-prefixed block so load can read exactly what we wrote.
        file.write_be_u32(item_count as u32);
        for it in &self.floor_items {
            file.write_le_i32(it.x);
            file.write_le_i32(it.y);
            file.write_u8(it.kind);
        }

        // (10) dFlags[MAXDUNX][MAXDUNY] (u8) — zeros (engine regenerates).
        for _ in 0..(MAXDUNX * MAXDUNY) {
            file.write_u8(0);
        }
        // (11) dItem indexes (u8) — zeros (populated via floor_items above).
        for _ in 0..(MAXDUNX * MAXDUNY) {
            file.write_u8(0);
        }

        if !self.is_town {
            // (12) dMonster[MAXDUNX][MAXDUNY] (BE i32).
            for _ in 0..(MAXDUNX * MAXDUNY) {
                file.write_be_i32(0);
            }
            // (13) dObject[MAXDUNX][MAXDUNY] (i8).
            for _ in 0..(MAXDUNX * MAXDUNY) {
                file.write_i8(0);
            }
            // (14) dLight[MAXDUNX][MAXDUNY] (u8).
            for _ in 0..(MAXDUNX * MAXDUNY) {
                file.write_u8(0);
            }
            // (15) dPreLight[MAXDUNX][MAXDUNY] (u8).
            for _ in 0..(MAXDUNX * MAXDUNY) {
                file.write_u8(0);
            }
            // (16) AutomapView[DMAXX][DMAXY] (u8).
            for _ in 0..(DMAXX * DMAXY) {
                file.write_u8(0);
            }
        }

        file.into_data()
    }

    /// Parse a level blob produced by `to_level_blob` back into a snapshot.
    pub fn from_level_blob(blob: &[u8]) -> Self {
        let mut file = LoadHelper::new(blob.to_vec());
        let mut out = Self::default();

        // (1) dCorpse — dungeon flag inferred from blob size is unreliable, so
        // we read deterministically: the writer always emits dCorpse unless the
        // snapshot was a town. We detect town by the leading counts section:
        // town blobs skip dCorpse. To stay robust, we read counts relative to
        // cursor and treat the absence of dCorpse as town. Simplest faithful
        // approach: try dungeon layout; if counts are absurd, retry as town.
        // For round-trip correctness we store `is_town` in the enveloping
        // `LevelSaveData`, so here we honour whatever the caller already set.
        if !out.is_town {
            // Skip dCorpse (default false here); caller sets is_town before
            // calling. We read it to advance the cursor.
            file.skip(MAXDUNX * MAXDUNY);
        }

        let monster_count = file.next_be_i32().max(0) as usize;
        let item_count = file.next_be_i32().max(0) as usize;
        let object_count = file.next_be_i32().max(0) as usize;

        if !out.is_town {
            // Active monster ids.
            for _ in 0..monster_count {
                let _id = file.next_be_u32();
            }
            for _ in 0..monster_count {
                out.monsters.push(BinaryMonsterData::from_binary(&mut file));
            }
            // ActiveObjects / AvailableObjects.
            file.skip(MAX_OBJECTS);
            file.skip(MAX_OBJECTS);
            for _ in 0..object_count {
                out.objects.push(BinaryObjectData::from_binary(&mut file));
            }
        }

        // (9) Floor items — length-prefixed compact form.
        let stored_item_count = file.next_be_u32() as usize;
        for _ in 0..stored_item_count {
            let x = file.next_le_i32();
            let y = file.next_le_i32();
            let kind = file.next_u8();
            out.floor_items.push(FloorItemData { x, y, kind });
        }
        let _ = item_count; // authoritative count is the stored one

        // (10-16) tile arrays — skipped (zeros).
        file.skip(MAXDUNX * MAXDUNY); // dFlags
        file.skip(MAXDUNX * MAXDUNY); // dItem
        if !out.is_town {
            file.skip(MAXDUNX * MAXDUNY * 4); // dMonster (i32)
            file.skip(MAXDUNX * MAXDUNY);     // dObject
            file.skip(MAXDUNX * MAXDUNY);     // dLight
            file.skip(MAXDUNX * MAXDUNY);     // dPreLight
            file.skip(DMAXX * DMAXY);         // AutomapView
        }

        out
    }
}

/// Envelope combining a level snapshot with the metadata needed to load it
/// back faithfully (`is_town`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelSaveData {
    pub is_town: bool,
    /// Raw binary blob (C++-compatible field order, see `to_level_blob`).
    #[serde(default)]
    pub blob: Vec<u8>,
}

impl LevelSaveData {
    /// Capture a snapshot into a serialisable envelope.
    pub fn from_snapshot(snap: &LevelSnapshotData) -> Self {
        Self {
            is_town: snap.is_town,
            blob: snap.to_level_blob(),
        }
    }

    /// Decode the envelope back into a snapshot.
    pub fn to_snapshot(&self) -> LevelSnapshotData {
        LevelSnapshotData::from_level_blob_typed(&self.blob, self.is_town)
    }
}

impl LevelSnapshotData {
    /// Variant of `from_level_blob` that takes the `is_town` flag explicitly,
    /// so the dCorpse skip is always correct regardless of envelope ordering.
    pub fn from_level_blob_typed(blob: &[u8], is_town: bool) -> Self {
        let mut snap = Self::default();
        snap.is_town = is_town;
        let mut file = LoadHelper::new(blob.to_vec());

        if !is_town {
            file.skip(MAXDUNX * MAXDUNY); // dCorpse
        }

        let monster_count = file.next_be_i32().max(0) as usize;
        let _item_count = file.next_be_i32().max(0) as usize;
        let object_count = file.next_be_i32().max(0) as usize;

        if !is_town {
            for _ in 0..monster_count {
                let _id = file.next_be_u32();
            }
            for _ in 0..monster_count {
                snap.monsters.push(BinaryMonsterData::from_binary(&mut file));
            }
            file.skip(MAX_OBJECTS);
            file.skip(MAX_OBJECTS);
            for _ in 0..object_count {
                snap.objects.push(BinaryObjectData::from_binary(&mut file));
            }
        }

        let stored_item_count = file.next_be_u32() as usize;
        for _ in 0..stored_item_count {
            let x = file.next_le_i32();
            let y = file.next_le_i32();
            let kind = file.next_u8();
            snap.floor_items.push(FloorItemData { x, y, kind });
        }

        // Remaining tile arrays — skipped.
        file.skip(MAXDUNX * MAXDUNY); // dFlags
        file.skip(MAXDUNX * MAXDUNY); // dItem
        if !is_town {
            file.skip(MAXDUNX * MAXDUNY * 4);
            file.skip(MAXDUNX * MAXDUNY);
            file.skip(MAXDUNX * MAXDUNY);
            file.skip(MAXDUNX * MAXDUNY);
            file.skip(DMAXX * DMAXY);
        }

        snap
    }
}

// ============================================================================
// SaveGame / LoadGame — global game-state persistence
//
// Mirrors C++ `SaveGameData`/`LoadGameData` from Source/loadsave.cpp
// (lines ~2453-2660 and ~2762-2830). The C++ writer emits, in order:
//   - global header (number of missiles, setlevel flag, level/town seeds...)
//   - player body (SavePlayer — 1584 bytes)
//   - quest bodies (SaveQuest × giNumberOfQests)
//   - portal bodies (SavePortal × MAXPORTALS)
//   - additional missiles block
//
// The Rust engine does not yet expose all of those subsystems, so we serialise
// what we have (a compact global header + the level snapshot + simple missiles)
// and pad to keep the format self-describing. Round-trip is the goal.
// ============================================================================

/// Maximum number of town portals (C++ MAXPORTALS).
pub const MAX_PORTALS: usize = 4;

/// Engine-side snapshot of global game state.
#[derive(Debug, Clone, Default)]
pub struct GameSnapshotData {
    /// Current level id (1-25).
    pub curr_level: u8,
    /// Is this a set (quest) level?
    pub is_set_level: bool,
    /// Is the game a Hellfire game?
    pub is_hellfire: bool,
    /// Difficulty (0=Normal, 1=Nightmare, 2=Hell).
    pub difficulty: u8,
    /// Town seed.
    pub dungeon_seed: u32,
    /// Per-level seeds (C++ DungeonSeeds[MAXLEVELS]).
    pub level_seeds: Vec<u32>,
    /// Quest states.
    pub quests: Vec<BinaryQuestData>,
    /// Portal states.
    pub portals: Vec<BinaryPortalData>,
    /// Simple missiles (engine `SimpleMissile`s).
    pub simple_missiles: Vec<SimpleMissileData>,
    /// The embedded level snapshot (current level).
    pub level: LevelSnapshotData,
}

/// Serialised form of an engine `SimpleMissile`.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SimpleMissileData {
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
    pub damage: i32,
    pub range_left: i32,
}

impl GameSnapshotData {
    /// Serialise into the C++-compatible global blob.
    pub fn to_game_blob(&self) -> Vec<u8> {
        let mut file = SaveHelper::new(64 * 1024);

        // --- Header (matches SaveGameData ordering) ---
        // C++ writes: setlevel (u8), setlvlnum (u8), currlevel (u8), leveltype
        // (u8), currlevel again forHellfire, then the per-level seed table, the
        //Difficulty, then int32 dungeon/seeds.
        file.write_bool8(self.is_set_level);
        file.write_u8(0); // setlvlnum
        file.write_u8(self.curr_level);
        file.write_u8(0); // leveltype (regenerated on load)
        file.write_bool8(self.is_hellfire);
        file.write_u8(self.difficulty);
        file.write_le_u32(self.dungeon_seed);

        // Per-level seeds (length-prefixed for forward-compat).
        file.write_be_u32(self.level_seeds.len() as u32);
        for &seed in &self.level_seeds {
            file.write_le_u32(seed);
        }

        // --- Quests (length-prefixed) ---
        file.write_be_u32(self.quests.len() as u32);
        for q in &self.quests {
            q.to_binary(
                &mut file,
                self.is_hellfire,
                q.quest_id, // quest_type == quest_id for our purposes
                &ReturnLevelData::default(),
            );
        }

        // --- Portals (length-prefixed; engine usually has ≤ MAX_PORTALS) ---
        file.write_be_u32(self.portals.len() as u32);
        for p in &self.portals {
            p.to_binary(&mut file);
        }

        // --- Simple missiles (length-prefixed) ---
        file.write_be_u32(self.simple_missiles.len() as u32);
        for m in &self.simple_missiles {
            file.write_le_i32(m.x);
            file.write_le_i32(m.y);
            file.write_le_i32(m.dx);
            file.write_le_i32(m.dy);
            file.write_le_i32(m.damage);
            file.write_le_i32(m.range_left);
        }

        // --- Embedded level snapshot (length-prefixed; prefixed by is_town) ---
        file.write_bool8(self.level.is_town);
        let level_blob = self.level.to_level_blob();
        file.write_be_u32(level_blob.len() as u32);
        file.write_bytes(&level_blob);

        file.into_data()
    }

    /// Parse a global blob back into a snapshot.
    pub fn from_game_blob(blob: &[u8]) -> Self {
        let mut file = LoadHelper::new(blob.to_vec());
        let mut out = Self::default();

        out.is_set_level = file.next_bool8();
        let _setlvlnum = file.next_u8();
        out.curr_level = file.next_u8();
        let _leveltype = file.next_u8();
        out.is_hellfire = file.next_bool8();
        out.difficulty = file.next_u8();
        out.dungeon_seed = file.next_le_u32();

        let seed_count = file.next_be_u32() as usize;
        for _ in 0..seed_count {
            out.level_seeds.push(file.next_le_u32());
        }

        let quest_count = file.next_be_u32() as usize;
        for _ in 0..quest_count {
            out.quests.push(BinaryQuestData::from_binary(&mut file, out.is_hellfire));
        }

        let portal_count = file.next_be_u32() as usize;
        for _ in 0..portal_count {
            out.portals.push(BinaryPortalData::from_binary(&mut file));
        }

        let missile_count = file.next_be_u32() as usize;
        for _ in 0..missile_count {
            out.simple_missiles.push(SimpleMissileData {
                x: file.next_le_i32(),
                y: file.next_le_i32(),
                dx: file.next_le_i32(),
                dy: file.next_le_i32(),
                damage: file.next_le_i32(),
                range_left: file.next_le_i32(),
            });
        }

        let level_is_town = file.next_bool8();
        let level_blob_len = file.next_be_u32() as usize;
        let level_blob = file.next_bytes(level_blob_len);
        out.level = LevelSnapshotData::from_level_blob_typed(&level_blob, level_is_town);

        out
    }
}

/// JSON-serialisable envelope for the global game blob.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSaveDataEnvelope {
    pub is_hellfire: bool,
    #[serde(default)]
    pub blob: Vec<u8>,
}

impl GameSaveDataEnvelope {
    pub fn from_snapshot(snap: &GameSnapshotData) -> Self {
        Self {
            is_hellfire: snap.is_hellfire,
            blob: snap.to_game_blob(),
        }
    }

    pub fn to_snapshot(&self) -> GameSnapshotData {
        let mut snap = GameSnapshotData::from_game_blob(&self.blob);
        snap.is_hellfire = self.is_hellfire;
        snap
    }
}

// ============================================================================
// LevelSnapshot / GameSnapshot traits — type-erased engine access
//
// These mirror the existing `PlayerSnapshot`/`WorldSnapshot` traits in
// `save.rs`. `GameState` implements them in `game_state.rs`, so the
// F5/F9 path can capture/restore the full level + game state without
// `loadsave.rs` depending on the engine types.
// ============================================================================

/// Read-only view of the current level's persistent state.
pub trait LevelSnapshot {
    /// `true` if the current level is the town.
    fn is_town(&self) -> bool;
    /// Active monsters (in active-slot order) + their derived write params.
    fn capture_monsters(&self) -> (Vec<BinaryMonsterData>, Vec<MonsterWriteParams>);
    /// Active objects.
    fn capture_objects(&self) -> Vec<BinaryObjectData>;
    /// Items lying on the floor.
    fn capture_floor_items(&self) -> Vec<FloorItemData>;
}

/// Read-only view of the global game state (quests/portals/missiles/level).
pub trait GameSnapshot {
    fn curr_level(&self) -> u8;
    fn is_set_level(&self) -> bool;
    fn is_hellfire(&self) -> bool;
    fn difficulty_u8(&self) -> u8;
    fn dungeon_seed(&self) -> u32;
    fn level_seeds(&self) -> Vec<u32>;
    fn capture_quests(&self) -> Vec<BinaryQuestData>;
    fn capture_portals(&self) -> Vec<BinaryPortalData>;
    fn capture_simple_missiles(&self) -> Vec<SimpleMissileData>;
}

/// Build a `LevelSnapshotData` from any `LevelSnapshot`.
pub fn build_level_snapshot(s: &dyn LevelSnapshot) -> LevelSnapshotData {
    let (monsters, monster_params) = s.capture_monsters();
    LevelSnapshotData {
        is_town: s.is_town(),
        monsters,
        monster_params,
        objects: s.capture_objects(),
        floor_items: s.capture_floor_items(),
    }
}

/// Build a `GameSnapshotData` from `GameSnapshot` + `LevelSnapshot`.
pub fn build_game_snapshot(
    g: &dyn GameSnapshot,
    level: &dyn LevelSnapshot,
) -> GameSnapshotData {
    GameSnapshotData {
        curr_level: g.curr_level(),
        is_set_level: g.is_set_level(),
        is_hellfire: g.is_hellfire(),
        difficulty: g.difficulty_u8(),
        dungeon_seed: g.dungeon_seed(),
        level_seeds: g.level_seeds(),
        quests: g.capture_quests(),
        portals: g.capture_portals(),
        simple_missiles: g.capture_simple_missiles(),
        level: build_level_snapshot(level),
    }
}

// ============================================================================
// Sectioned save/load helpers — Delta / Misc / NObjects
//
// These mirror the *sections* of the C++ SaveGameData/SaveLevel pipeline
// (Source/loadsave.cpp) rather than single C++ functions, so the Rust engine
// can stream each section independently without owning the full global state.
//
// Each pair is byte-compatible with the corresponding slice of the vanilla
// blob on the wire, and round-trips through the `*DeltaData`/`*MiscData`/
// `*NObjectsData` structs defined below.
// ============================================================================

/// "Delta" section — per-level seed deltas written by `SaveLevelSeeds()`.
///
/// **C++ Reference**: `SaveLevelSeeds()` / `LoadLevelSeeds()` in
/// Source/loadsave.cpp:1901-1926.
///
/// Format: for each level `i`, a `u8` "present" flag followed by a `le u32`
/// seed when present. `None` levels serialise as a single `0` byte.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeltaData {
    /// Per-level seed deltas; `None` means "not yet generated".
    pub level_seeds: Vec<Option<u32>>,
}

impl DeltaData {
    /// Number of bytes this delta section occupies on the wire.
    pub fn byte_len(&self) -> usize {
        // 1 byte flag per level + 4 bytes seed when present.
        let present = self.level_seeds.iter().filter(|s| s.is_some()).count();
        self.level_seeds.len() + present * 4
    }

    /// Serialise the delta section into `file`.
    ///
    /// **C++ Reference**: `SaveLevelSeeds()` Source/loadsave.cpp:1901-1911.
    pub fn save_delta(&self, file: &mut SaveHelper) {
        for seed in &self.level_seeds {
            match seed {
                Some(s) => {
                    file.write_u8(1);
                    file.write_le_u32(*s);
                }
                None => file.write_u8(0),
            }
        }
    }

    /// Parse a delta section of `count` levels from `file`.
    ///
    /// **C++ Reference**: `LoadLevelSeeds()` Source/loadsave.cpp:1913-1926.
    pub fn load_delta(file: &mut LoadHelper, count: usize) -> Self {
        let mut out = Self::default();
        for _ in 0..count {
            if file.next_u8() != 0 {
                out.level_seeds.push(Some(file.next_le_u32()));
            } else {
                out.level_seeds.push(None);
            }
        }
        out
    }
}

/// Save the delta (level-seed) section into a standalone blob.
pub fn save_delta_blob(data: &DeltaData) -> Vec<u8> {
    let mut file = SaveHelper::new(data.byte_len().max(16));
    data.save_delta(&mut file);
    file.into_data()
}

/// Load a delta section of `count` levels from a standalone blob.
pub fn load_delta_blob(blob: &[u8], count: usize) -> DeltaData {
    let mut file = LoadHelper::new(blob.to_vec());
    DeltaData::load_delta(&mut file, count)
}

/// "Misc" section — the trailing miscellaneous game data written by the tail
/// of `SaveGameData()`: premium-item count/level, the premium items themselves,
/// the automap-active flag, and the automap scale.
///
/// **C++ Reference**: tail of `SaveGameData()` in Source/loadsave.cpp:2916-2923.
#[derive(Debug, Clone, Default)]
pub struct MiscData {
    /// Number of premium (Wirt) items currently for sale.
    pub premium_item_count: i32,
    /// Current premium item level.
    pub premium_item_level: i32,
    /// Premium item bodies (compact engine form).
    pub premium_items: Vec<BinaryItemData>,
    /// Whether the automap overlay is active.
    pub automap_active: bool,
    /// Automap zoom scale.
    pub automap_scale: i32,
}

impl MiscData {
    /// Serialise the misc section into `file`.
    ///
    /// **C++ Reference**: `SaveGameData()` tail in Source/loadsave.cpp:2916-2923.
    pub fn save_misc(&self, file: &mut SaveHelper, hellfire_premium_slots: usize, is_hellfire: bool) {
        file.write_be_i32(self.premium_item_count);
        file.write_be_i32(self.premium_item_level);

        // Premium items — C++ writes exactly giNumberOfSmithPremiumItems slots
        // (6 for Diablo, 15 for Hellfire). We pad with default items.
        let slots = hellfire_premium_slots.max(self.premium_items.len());
        let dummy = BinaryItemData::default();
        for i in 0..slots {
            let it = self.premium_items.get(i).unwrap_or(&dummy);
            it.to_binary(file, is_hellfire);
        }

        file.write_bool8(self.automap_active);
        file.write_be_i32(self.automap_scale);
    }

    /// Parse the misc section from `file`.
    pub fn load_misc(file: &mut LoadHelper, hellfire_premium_slots: usize, is_hellfire: bool) -> Self {
        let mut out = Self::default();
        out.premium_item_count = file.next_be_i32();
        out.premium_item_level = file.next_be_i32();
        for _ in 0..hellfire_premium_slots {
            out.premium_items
                .push(BinaryItemData::from_binary(file, is_hellfire));
        }
        out.automap_active = file.next_bool8();
        out.automap_scale = file.next_be_i32();
        out
    }
}

/// Save the misc section into a standalone blob.
pub fn save_misc_blob(
    data: &MiscData,
    hellfire_premium_slots: usize,
    is_hellfire: bool,
) -> Vec<u8> {
    // Worst-case size: counts (8) + premium items + automap (5).
    let item_bytes = if is_hellfire { 372 } else { 368 };
    let cap = 16 + hellfire_premium_slots * item_bytes + 8;
    let mut file = SaveHelper::new(cap);
    data.save_misc(&mut file, hellfire_premium_slots, is_hellfire);
    file.into_data()
}

/// Load the misc section from a standalone blob.
pub fn load_misc_blob(blob: &[u8], hellfire_premium_slots: usize, is_hellfire: bool) -> MiscData {
    let mut file = LoadHelper::new(blob.to_vec());
    MiscData::load_misc(&mut file, hellfire_premium_slots, is_hellfire)
}

/// "NObjects" section — the network-sync object slots written by
/// `SaveLevel()`/`SaveGameData()`: the `ActiveObjects[]` and
/// `AvailableObjects[]` i8 index arrays (each `MAX_OBJECTS` long) followed by
/// the per-object bodies for every active object.
///
/// **C++ Reference**: object-slot writes inside `SaveLevel()` /
/// `SaveGameData()` in Source/loadsave.cpp:1961-1967 / 2843-2848.
#[derive(Debug, Clone, Default)]
pub struct NObjectsData {
    /// Active object slot indices (C++ `ActiveObjects[]`, length MAX_OBJECTS).
    pub active: Vec<i8>,
    /// Free object slot indices (C++ `AvailableObjects[]`, length MAX_OBJECTS).
    pub available: Vec<i8>,
    /// Bodies of the active objects, in active order.
    pub objects: Vec<BinaryObjectData>,
}

impl NObjectsData {
    /// Serialise the object-slot section into `file`.
    ///
    /// **C++ Reference**: object-slot block in `SaveLevel()`
    /// Source/loadsave.cpp:1961-1967.
    pub fn save_nobjects(&self, file: &mut SaveHelper) {
        debug_assert!(
            self.active.len() <= MAX_OBJECTS,
            "active object list overflow"
        );
        debug_assert!(
            self.available.len() <= MAX_OBJECTS,
            "available object list overflow"
        );
        // Pad both arrays to MAX_OBJECTS with the -1 sentinel C++ uses for
        // empty slots.
        let neg = -1i8;
        for i in 0..MAX_OBJECTS {
            file.write_i8(self.active.get(i).copied().unwrap_or(neg));
        }
        for i in 0..MAX_OBJECTS {
            file.write_i8(self.available.get(i).copied().unwrap_or(neg));
        }
        for o in &self.objects {
            o.to_binary(file);
        }
    }

    /// Parse the object-slot section for `object_count` active objects.
    ///
    /// **C++ Reference**: object-slot block in `LoadLevel()`
    /// Source/loadsave.cpp:2039-2044.
    pub fn load_nobjects(file: &mut LoadHelper, object_count: usize) -> Self {
        let mut out = Self::default();
        for _ in 0..MAX_OBJECTS {
            out.active.push(file.next_i8());
        }
        for _ in 0..MAX_OBJECTS {
            out.available.push(file.next_i8());
        }
        for _ in 0..object_count {
            out.objects.push(BinaryObjectData::from_binary(file));
        }
        out
    }
}

/// Save the object-slot section into a standalone blob.
pub fn save_nobjects_blob(data: &NObjectsData) -> Vec<u8> {
    // Two MAX_OBJECTS i8 arrays + one body per active object (~100 bytes).
    let cap = MAX_OBJECTS * 2 + data.objects.len() * 128;
    let mut file = SaveHelper::new(cap.max(16));
    data.save_nobjects(&mut file);
    file.into_data()
}

/// Load the object-slot section for `object_count` active objects from a blob.
pub fn load_nobjects_blob(blob: &[u8], object_count: usize) -> NObjectsData {
    let mut file = LoadHelper::new(blob.to_vec());
    NObjectsData::load_nobjects(&mut file, object_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_header() {
        let header = SaveHeader {
            magic: SAVE_MAGIC,
            version: SAVE_VERSION,
            player_name: "Test".to_string(),
            player_class: PlayerClass::Warrior,
            player_level: 10,
            dungeon_level: 5,
            difficulty: Difficulty::Normal,
            is_hellfire: false,
            play_time_seconds: 3600,
            save_time: 0,
        };

        assert!(header.is_valid());
    }

    #[test]
    fn test_format_play_time() {
        assert_eq!(format_play_time(0), "00:00:00");
        assert_eq!(format_play_time(61), "00:01:01");
        assert_eq!(format_play_time(3661), "01:01:01");
    }

    #[test]
    fn test_load_helper() {
        let data = vec![
            0x01, 0x02, 0x03, 0x04,  // u32 LE = 0x04030201
            0x10, 0x20,              // u16 LE = 0x2010
            0x42,                    // u8 = 0x42
            0x01,                    // bool = true
        ];
        let mut helper = LoadHelper::new(data);

        assert_eq!(helper.next_le_u32(), 0x04030201);
        assert_eq!(helper.next_le_u16(), 0x2010);
        assert_eq!(helper.next_u8(), 0x42);
        assert!(helper.next_bool8());
    }

    #[test]
    fn test_save_helper() {
        let mut helper = SaveHelper::new(64);

        helper.write_le_u32(0x04030201);
        helper.write_le_u16(0x2010);
        helper.write_u8(0x42);
        helper.write_bool8(true);

        let data = helper.into_data();
        assert_eq!(data[0..4], [0x01, 0x02, 0x03, 0x04]);
        assert_eq!(data[4..6], [0x10, 0x20]);
        assert_eq!(data[6], 0x42);
        assert_eq!(data[7], 0x01);
    }

    #[test]
    fn test_codec() {
        let original = vec![0x12, 0x34, 0x56, 0x78];
        let mut encoded = original.clone();

        SaveCodec::encode_default(&mut encoded);
        assert_ne!(encoded, original); // Should be different after encoding

        SaveCodec::decode_default(&mut encoded);
        assert_eq!(encoded, original); // Should match after decoding
    }

    #[test]
    fn test_string_io() {
        let mut helper = SaveHelper::new(64);
        helper.write_string("Test", 32);

        let data = helper.into_data();
        let mut load = LoadHelper::new(data);
        let s = load.next_string(32);

        assert_eq!(s, "Test");
    }

    #[test]
    fn test_big_endian() {
        let mut helper = SaveHelper::new(16);
        helper.write_be_u32(0x01020304);

        let data = helper.into_data();
        assert_eq!(data, vec![0x01, 0x02, 0x03, 0x04]);

        let mut load = LoadHelper::new(data);
        assert_eq!(load.next_be_u32(), 0x01020304);
    }

    // ========================================================================
    // SaveLevel / LoadLevel + SaveGame / LoadGame round-trip coverage.
    // These exercise the binary blobs the F5/F9 path now persists.
    // ========================================================================

    /// Helper: build a populated `LevelSnapshotData` for testing.
    fn sample_level_snapshot(is_town: bool) -> LevelSnapshotData {
        let mut snap = LevelSnapshotData {
            is_town,
            ..Default::default()
        };
        if !is_town {
            snap.monsters.push(BinaryMonsterData {
                level_type: 1,
                mode: 0,
                goal: 1,
                position_x: 42,
                position_y: 17,
                future_x: 42,
                future_y: 17,
                old_x: 40,
                old_y: 15,
                direction: 0,
                enemy: -1,
                enemy_x: 0,
                enemy_y: 0,
                hp: 64,
                max_hp: 64,
                ai: 8,
                intelligence: 2,
                flags: 0x1010,
                active_for_ticks: 5,
                last_x: 41,
                last_y: 16,
                rnd_item_seed: 0xCAFEBABE,
                ai_seed: 0xDEADBEEF,
                unique_type: 0,
                uniq_trans: 0,
                resistance: 0b110,
                min_damage: 1,
                max_damage: 4,
                armor_class: 10,
                ..Default::default()
            });
            snap.monster_params.push(MonsterWriteParams {
                level: 2,
                experience: 250,
                to_hit: 55,
                to_hit_special: 0,
            });
            snap.objects.push(BinaryObjectData {
                object_type: 1,
                position_x: 50,
                position_y: 50,
                apply_lighting: true,
                anim_flag: false,
                anim_len: 10,
                anim_frame: 1,
                anim_width: 96,
                del_flag: false,
                break_flag: 0,
                solid_flag: true,
                trap_flag: false,
                door_flag: false,
                selection_region: 3,
                pre_flag: false,
                light_id: -1,
                rnd_seed: 0x1234,
                var1: 7,
                ..Default::default()
            });
        }
        snap.floor_items.push(FloorItemData { x: 30, y: 30, kind: 0 });
        snap.floor_items.push(FloorItemData { x: 31, y: 31, kind: 1 });
        snap
    }

    #[test]
    fn test_save_level_roundtrip_dungeon() {
        let snap = sample_level_snapshot(false);
        let env = LevelSaveData::from_snapshot(&snap);
        let back = env.to_snapshot();

        assert_eq!(back.is_town, false);
        assert_eq!(back.monsters.len(), 1);
        assert_eq!(back.objects.len(), 1);
        assert_eq!(back.floor_items.len(), 2);

        let m = &back.monsters[0];
        assert_eq!(m.position_x, 42);
        assert_eq!(m.position_y, 17);
        assert_eq!(m.hp, 64);
        assert_eq!(m.max_hp, 64);
        assert_eq!(m.rnd_item_seed, 0xCAFEBABE);
        assert_eq!(m.ai_seed, 0xDEADBEEF);
        assert_eq!(m.resistance, 0b110);
        assert_eq!(m.min_damage, 1);
        assert_eq!(m.max_damage, 4);
        assert_eq!(m.armor_class, 10);

        let o = &back.objects[0];
        assert_eq!(o.position_x, 50);
        assert_eq!(o.position_y, 50);
        assert_eq!(o.rnd_seed, 0x1234);
        assert_eq!(o.var1, 7);

        assert_eq!(back.floor_items[0].x, 30);
        assert_eq!(back.floor_items[0].kind, 0);
        assert_eq!(back.floor_items[1].kind, 1);
    }

    #[test]
    fn test_save_level_roundtrip_town() {
        // Town skips dCorpse + the dungeon-only tile arrays + monster/object
        // bodies, so the blob is smaller; floor items still round-trip.
        let snap = sample_level_snapshot(true);
        let env = LevelSaveData::from_snapshot(&snap);
        let back = env.to_snapshot();

        assert_eq!(back.is_town, true);
        assert!(back.monsters.is_empty());
        assert!(back.objects.is_empty());
        assert_eq!(back.floor_items.len(), 2);
        assert_eq!(back.floor_items[0].x, 30);
    }

    #[test]
    fn test_save_game_roundtrip() {
        let level = sample_level_snapshot(false);
        let mut snap = GameSnapshotData {
            curr_level: 5,
            is_set_level: false,
            is_hellfire: false,
            difficulty: 1,
            dungeon_seed: 0xABCDEF01,
            level_seeds: vec![1, 2, 3, 4, 5],
            level,
            ..Default::default()
        };
        snap.simple_missiles.push(SimpleMissileData {
            x: 10,
            y: 11,
            dx: 1,
            dy: 0,
            damage: 25,
            range_left: 3,
        });

        let env = GameSaveDataEnvelope::from_snapshot(&snap);
        let back = env.to_snapshot();

        assert_eq!(back.curr_level, 5);
        assert_eq!(back.difficulty, 1);
        assert_eq!(back.dungeon_seed, 0xABCDEF01);
        assert_eq!(back.level_seeds, vec![1, 2, 3, 4, 5]);
        assert_eq!(back.simple_missiles.len(), 1);
        assert_eq!(back.simple_missiles[0].x, 10);
        assert_eq!(back.simple_missiles[0].damage, 25);
        // Embedded level snapshot round-trips too.
        assert_eq!(back.level.monsters.len(), 1);
        assert_eq!(back.level.monsters[0].position_x, 42);
        assert_eq!(back.level.floor_items.len(), 2);
    }

    #[test]
    fn test_monster_writer_matches_reader_field_order() {
        // Write a monster, read it back, and verify the writer/reader agree on
        // every field (catches alignment drift between SaveMonster/LoadMonster).
        let m = BinaryMonsterData {
            level_type: 3,
            mode: 4,
            goal: 2,
            goal_var1: 100,
            goal_var2: 5,
            goal_var3: 6,
            path_count: 7,
            position_x: 11,
            position_y: 22,
            future_x: 33,
            future_y: 44,
            old_x: 55,
            old_y: 66,
            direction: 1,
            enemy: 2,
            enemy_x: 70,
            enemy_y: 71,
            anim_ticks_per_frame: 2,
            anim_tick_counter: 3,
            anim_num_frames: 8,
            anim_current_frame: 4,
            is_invalid: false,
            var1: 200,
            var2: 201,
            var3: 9,
            temp_x: 10,
            temp_y: 11,
            max_hp: 999,
            hp: 500,
            ai: 3,
            intelligence: 1,
            flags: 0xCAFE,
            active_for_ticks: 4,
            last_x: 80,
            last_y: 81,
            rnd_item_seed: 0x1111,
            ai_seed: 0x2222,
            unique_type: 2,
            uniq_trans: 3,
            corpse_id: -1,
            who_hit: 0,
            min_damage: 2,
            max_damage: 6,
            min_damage_special: 3,
            max_damage_special: 8,
            armor_class: 12,
            resistance: 0b1010,
            talk_msg: 42,
            leader: 1,
            leader_relation: 1,
            pack_size: 4,
            light_id: 5,
        };
        let p = MonsterWriteParams {
            level: 6,
            experience: 500,
            to_hit: 60,
            to_hit_special: 40,
        };

        let mut helper = SaveHelper::new(1024);
        m.to_binary(&mut helper, p.level, p.experience, p.to_hit, p.to_hit_special);
        let blob = helper.into_data();
        let mut reader = LoadHelper::new(blob);
        let back = BinaryMonsterData::from_binary(&mut reader);

        // Verify the reader undoes the writer's +1/-1 adjustments.
        assert_eq!(back.level_type, 3);
        assert_eq!(back.mode, 4);
        assert_eq!(back.goal, 2);
        assert_eq!(back.position_x, 11);
        assert_eq!(back.position_y, 22);
        assert_eq!(back.hp, 500);
        assert_eq!(back.max_hp, 999);
        assert_eq!(back.flags, 0xCAFE);
        assert_eq!(back.rnd_item_seed, 0x1111);
        assert_eq!(back.ai_seed, 0x2222);
        assert_eq!(back.resistance, 0b1010);
        assert_eq!(back.armor_class, 12);
        assert_eq!(back.talk_msg, 42);
        assert_eq!(back.leader, 1);
        assert_eq!(back.pack_size, 4);
        // uniqueType is stored as unique_type+1 and read back as -1, so it
        // round-trips to the same value.
        assert_eq!(back.unique_type, 2);
        // anim_current_frame is stored as +1 and read back as -1.
        assert_eq!(back.anim_current_frame, 4);
    }

    #[test]
    fn test_save_slot_v3_carries_level_and_game() {
        // A v3 SaveSlot built via build_save_slot_full should JSON-round-trip
        // with the level + game blobs intact.
        let level = sample_level_snapshot(false);
        let game = GameSnapshotData {
            curr_level: 7,
            is_hellfire: false,
            level,
            ..Default::default()
        };
        let env_level = LevelSaveData::from_snapshot(&sample_level_snapshot(false));
        let env_game = GameSaveDataEnvelope::from_snapshot(&game);

        let json_level = serde_json::to_string(&env_level).unwrap();
        let json_game = serde_json::to_string(&env_game).unwrap();
        let back_level: LevelSaveData = serde_json::from_str(&json_level).unwrap();
        let back_game: GameSaveDataEnvelope = serde_json::from_str(&json_game).unwrap();

        assert_eq!(env_level, back_level);
        assert_eq!(env_game, back_game);
        assert_eq!(back_game.to_snapshot().curr_level, 7);
        assert_eq!(back_level.to_snapshot().monsters.len(), 1);
    }

    #[test]
    fn test_delta_section_roundtrip() {
        let data = DeltaData {
            level_seeds: vec![Some(0x1111_2222), None, Some(0x3333_4444), None, Some(0)],
        };
        let expected_bytes = data.byte_len();
        let blob = save_delta_blob(&data);
        assert_eq!(blob.len(), expected_bytes);
        let back = load_delta_blob(&blob, data.level_seeds.len());
        assert_eq!(back, data);
    }

    #[test]
    fn test_delta_section_empty_levels() {
        let data = DeltaData {
            level_seeds: vec![None, None, None],
        };
        let blob = save_delta_blob(&data);
        // 3 absent levels = 3 flag bytes, no seed bytes.
        assert_eq!(blob.len(), 3);
        let back = load_delta_blob(&blob, 3);
        assert_eq!(back, data);
    }

    #[test]
    fn test_delta_section_all_present() {
        let data = DeltaData {
            level_seeds: vec![Some(1), Some(2), Some(3), Some(4)],
        };
        let blob = save_delta_blob(&data);
        // 4 flag bytes + 4 * 4 seed bytes.
        assert_eq!(blob.len(), 4 + 16);
        let back = load_delta_blob(&blob, 4);
        assert_eq!(back, data);
    }

    #[test]
    fn test_misc_section_roundtrip() {
        let data = MiscData {
            premium_item_count: 3,
            premium_item_level: 7,
            premium_items: vec![BinaryItemData::default(); 2],
            automap_active: true,
            automap_scale: 2,
        };
        // Hellfire uses 15 premium slots; we pad to that.
        let blob = save_misc_blob(&data, 15, true);
        let back = load_misc_blob(&blob, 15, true);
        assert_eq!(back.premium_item_count, 3);
        assert_eq!(back.premium_item_level, 7);
        assert_eq!(back.premium_items.len(), 15);
        assert!(back.automap_active);
        assert_eq!(back.automap_scale, 2);
    }

    #[test]
    fn test_misc_section_diablo_slots() {
        let data = MiscData {
            premium_item_count: 0,
            premium_item_level: 0,
            premium_items: vec![],
            automap_active: false,
            automap_scale: 1,
        };
        let blob = save_misc_blob(&data, 6, false);
        let back = load_misc_blob(&blob, 6, false);
        assert_eq!(back.premium_items.len(), 6);
        assert!(!back.automap_active);
        assert_eq!(back.automap_scale, 1);
    }

    #[test]
    fn test_nobjects_section_roundtrip() {
        let data = NObjectsData {
            active: vec![0, 1, 2],
            available: vec![3, 4],
            objects: vec![
                BinaryObjectData {
                    object_type: 5,
                    position_x: 10,
                    position_y: 20,
                    rnd_seed: 0xABCDEF01,
                    ..Default::default()
                },
                BinaryObjectData {
                    object_type: 7,
                    position_x: 30,
                    position_y: 40,
                    ..Default::default()
                },
            ],
        };
        let blob = save_nobjects_blob(&data);
        let back = load_nobjects_blob(&blob, data.objects.len());
        // Both slot arrays are padded to MAX_OBJECTS.
        assert_eq!(back.active.len(), MAX_OBJECTS);
        assert_eq!(back.available.len(), MAX_OBJECTS);
        // First slots carry the active/available ids we wrote.
        assert_eq!(back.active[0..3], [0i8, 1, 2]);
        assert_eq!(back.available[0..2], [3i8, 4]);
        // Tail slots are the -1 sentinel.
        assert_eq!(back.active[3], -1);
        assert_eq!(back.available[2], -1);
        // Bodies round-trip.
        assert_eq!(back.objects.len(), 2);
        assert_eq!(back.objects[0].object_type, 5);
        assert_eq!(back.objects[0].position_x, 10);
        assert_eq!(back.objects[0].rnd_seed, 0xABCDEF01);
        assert_eq!(back.objects[1].object_type, 7);
    }

    #[test]
    fn test_nobjects_section_empty() {
        let data = NObjectsData::default();
        let blob = save_nobjects_blob(&data);
        // Two MAX_OBJECTS arrays of i8, no bodies.
        assert_eq!(blob.len(), MAX_OBJECTS * 2);
        let back = load_nobjects_blob(&blob, 0);
        assert_eq!(back.active.len(), MAX_OBJECTS);
        assert_eq!(back.available.len(), MAX_OBJECTS);
        assert!(back.objects.is_empty());
        // Every slot is the -1 sentinel.
        assert!(back.active.iter().all(|&v| v == -1));
        assert!(back.available.iter().all(|&v| v == -1));
    }

    #[test]
    /// A real drop with the full generated item serialises its seed and
    /// affix data into the player's SaveItem (C++ SetupAllItems output).
    #[test]
    fn test_player_item_full_serialises_seed_and_affix() {
        use crate::game::player_exact::PlayerItem;
        let mut full = crate::game::items::Item::empty();
        full.item_index = 119; // Short Sword
        full.seed = 0xA1B2C3D4;
        full.bonus_damage = 7;
        full.name = "Short Sword of the Bear".to_string();
        full.base_name = "Short Sword".to_string();
        let it = PlayerItem {
            item_id: 119,
            equipped: false,
            _itype: crate::game::item_dat::ItemType::Sword,
            full: Some(full),
        };
        let b = player_item_to_binary(&it);
        let mut h = SaveHelper::new(512);
        b.to_binary(&mut h, false);
        let d = h.into_data();
        assert_eq!(d.len(), 368, "SaveItem is 368 bytes");
        assert_eq!(&d[0..4], &0xA1B2C3D4u32.to_le_bytes(), "seed from the full item");
        assert_eq!(&d[244..248], &7i32.to_le_bytes(), "plDam (affix bonus)");
        assert_eq!(&d[125..148], b"Short Sword of the Bear", "identified name");
        assert_eq!(&d[360..364], &119i32.to_le_bytes(), "IDidx");
    }

    fn test_player_item_maps_base_attrs_from_tsv_row() {
        use crate::game::player_exact::PlayerItem;
        let mut it = PlayerItem::empty();
        it.item_id = 119; // Short Sword (itemdat.tsv row)
        let b = player_item_to_binary(&it);
        assert_eq!(b.item_idx, 119, "IDidx is the TSV row");
        assert_eq!(b.item_type, crate::game::item_dat::ItemType::Sword as i32);
        assert_eq!(b.class, crate::game::item_dat::ItemClass::Weapon as u8);
        assert_eq!(b.loc, crate::game::item_dat::ItemEquipType::OneHand as i8);
        assert_eq!(b.min_dam, 2, "base min damage from itemdat.tsv");
        assert_eq!(b.max_dam, 6, "base max damage from itemdat.tsv");
        assert_eq!(b.durability, 24, "base durability from itemdat.tsv");
        assert_eq!(b.max_dur, 24);
        assert_eq!(b.value, 120, "base value from itemdat.tsv");
        assert_eq!(b.ivalue, 120);
        assert_eq!(b.cursor, 64, "cursor graphic from itemdat.tsv");
        assert_eq!(b.min_str, 18, "strength requirement from itemdat.tsv");
        assert_eq!(&b.name[..11], b"Short Sword");
        // Empty slots stay the all-default stub.
        let empty = player_item_to_binary(&PlayerItem::empty());
        assert_eq!(empty.item_idx, 0);
        assert!(empty.name.iter().all(|&b| b == 0), "empty slot keeps empty name");
    }
}
