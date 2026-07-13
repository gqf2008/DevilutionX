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
use super::player::{Player, PlayerClass, PlayerStats};
use super::items::{Item, ItemType, ItemQuality, EquipSlot};

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
        Self {
            name: item.name.clone(),
            base_name: item.base_name.clone(),
            item_type: item.item_type,
            quality: item.quality,
            identified: item.identified,
            base_damage_min: item.base_damage_min as i32,
            base_damage_max: item.base_damage_max as i32,
            base_armor: item.base_armor as i32,
            quantity: item.quantity,
            durability: item.durability,
            max_durability: item.max_durability,
            buy_value: item.buy_value,
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
        helper.write_le_i32(48); // _iAnimWidth2
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

// ============================================================================
// SaveSlot - practical round-trippable snapshot of the live GameState
// ============================================================================
//
// The legacy `GameSave`/`SavedPlayer` types above were modelled on the *simple*
// `game::player::Player` struct and a JSON view of the inventory, which do not
// match the rich `player_exact::Player` that the live `GameState` actually
// holds (~150 fields, 64x fixed-point HP/Mana, etc.). Rather than mutate the
// read-only `player_exact` module, `SaveSlot` captures the player's *observable*
// gameplay state (identity, progression, 64x vitals, position, level) as plain
// serde-friendly primitives, so it can round-trip cleanly through bincode/JSON
// without depending on internal engine types.
//
// This is what F5 (save) / F9 (load) in the game loop use.

/// Snapshotted player vitals. All HP/Mana values are stored in their **native
/// 64x fixed-point** representation (as held by `player_exact::Player`), so the
/// save/restore is lossless — we never divide/multiply and lose precision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotPlayer {
    pub name: String,
    /// Hero class as u8 (matches `HeroClass`'s `From<HeroClass> for u8`).
    pub class: u8,
    /// Player level (1-50).
    pub level: u8,
    /// Current dungeon/town level id.
    pub plr_level: u8,

    // --- Attributes (current + base) ---
    pub strength: i32,
    pub base_str: i32,
    pub magic: i32,
    pub base_mag: i32,
    pub dexterity: i32,
    pub base_dex: i32,
    pub vitality: i32,
    pub base_vit: i32,
    pub stat_pts: i32,

    // --- HP (64x fixed-point, stored as-is) ---
    pub hp_base: i32,
    pub max_hp_base: i32,
    pub hit_points: i32,
    pub max_hp: i32,

    // --- Mana (64x fixed-point, stored as-is) ---
    pub mana_base: i32,
    pub max_mana_base: i32,
    pub mana: i32,
    pub max_mana: i32,

    // --- Progression ---
    pub experience: u32,
    pub gold: i32,

    // --- Position ---
    pub pos_x: i32,
    pub pos_y: i32,
}

/// World/level context captured alongside the player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotWorld {
    /// `DungeonType` as u8 (None=0, Town=1, Cathedral=2, ...).
    pub dungeon_type: u8,
    /// Whether the player is currently inside the dungeon (vs. town).
    pub in_dungeon: bool,
    pub is_town: bool,
    /// Current game tick.
    pub game_tick: u32,
    /// Camera position.
    pub cam_x: i32,
    pub cam_y: i32,
}

/// A practical save slot: a header + the player snapshot + the world snapshot.
///
/// Serialized to disk as JSON (human-readable, matches the existing
/// `SaveManager` format) via `SaveManager::save_slot`/`load_slot`.
///
/// Version 3 adds optional `level` / `game` blobs that carry the full per-level
/// and global binary state (monsters, objects, floor items, quests, portals,
/// missiles) produced by `loadsave::LevelSaveData` / `GameSaveDataEnvelope`.
/// Older saves (version 2) load with these fields absent (regenerated from the
/// level seed, the original behaviour).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveSlot {
    pub magic: u32,
    pub version: u32,
    /// Unix timestamp of the save.
    pub save_time: u64,
    pub player: SlotPlayer,
    pub world: SlotWorld,
    /// Optional full-level binary blob (version >= 3). Absent on legacy saves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<crate::game::loadsave::LevelSaveData>,
    /// Optional full-game binary blob (version >= 3). Absent on legacy saves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game: Option<crate::game::loadsave::GameSaveDataEnvelope>,
}

impl SaveSlot {
    /// Current on-disk save format version.
    pub const MAGIC: u32 = 0x53415645; // "SAVE"
    /// Version 2: player + world snapshots only.
    /// Version 3: + optional full level/game binary blobs.
    pub const VERSION: u32 = 3;
    /// Highest version that still uses the v2 (no level/game) layout.
    pub const VERSION_V2: u32 = 2;

    /// Validate magic + version.
    pub fn is_valid(&self) -> bool {
        self.magic == Self::MAGIC
            && (self.version == Self::VERSION || self.version == Self::VERSION_V2)
    }
}

impl SaveManager {
    /// Path for a `SaveSlot` file (kept separate from the legacy `save_{n}.json`
    /// files so they never collide).
    fn slot_path(&self, slot: u32) -> std::path::PathBuf {
        Path::new(&self.save_dir).join(format!("slot_{}.sv", slot))
    }

    /// Public accessor for the on-disk path of a slot (used for logging).
    pub fn slot_path_public(&self, slot: u32) -> String {
        self.slot_path(slot).to_string_lossy().to_string()
    }

    /// Save a `SaveSlot` to the given slot number as pretty JSON. Returns the
    /// path that was written (for logging).
    pub fn save_slot(&self, slot: u32, data: &SaveSlot) -> Result<String> {
        self.ensure_dir()?;
        let path = self.slot_path(slot);
        let file = File::create(&path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, data)?;
        Ok(path.to_string_lossy().to_string())
    }

    /// Load a `SaveSlot` from the given slot number.
    pub fn load_slot(&self, slot: u32) -> Result<SaveSlot> {
        let path = self.slot_path(slot);
        if !path.exists() {
            return Err(anyhow!("Save slot does not exist: {:?}", path));
        }
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let slot_data: SaveSlot = serde_json::from_reader(reader)?;
        if !slot_data.is_valid() {
            return Err(anyhow!(
                "Save slot {:?} is invalid (magic=0x{:X}, version={})",
                path, slot_data.magic, slot_data.version
            ));
        }
        Ok(slot_data)
    }

    /// Does a `SaveSlot` exist for this slot number?
    pub fn slot_exists_v2(&self, slot: u32) -> bool {
        self.slot_path(slot).exists()
    }
}

/// Convenience helpers: build a `SlotPlayer`/`SlotWorld`/`SaveSlot` directly
/// from the live engine types, with no engine-side serialization coupling.
impl SlotPlayer {
    /// Capture a `player_exact::Player` into a serialisable snapshot.
    ///
    /// We take the player as a type-erased `&dyn PlayerSnapshot` so `save.rs`
    /// does not depend on `player_exact` (which is a read-only module in this
    /// task). `GameState` implements `PlayerSnapshot` for its player in
    /// `game_state.rs`.
    pub fn from_snapshot(p: &dyn PlayerSnapshot) -> Self {
        Self {
            name: p.name(),
            class: p.class_u8(),
            level: p.level(),
            plr_level: p.plr_level(),
            strength: p.strength(),
            base_str: p.base_str(),
            magic: p.magic(),
            base_mag: p.base_mag(),
            dexterity: p.dexterity(),
            base_dex: p.base_dex(),
            vitality: p.vitality(),
            base_vit: p.base_vit(),
            stat_pts: p.stat_pts(),
            hp_base: p.hp_base(),
            max_hp_base: p.max_hp_base(),
            hit_points: p.hit_points(),
            max_hp: p.max_hp(),
            mana_base: p.mana_base(),
            max_mana_base: p.max_mana_base(),
            mana: p.mana(),
            max_mana: p.max_mana(),
            experience: p.experience(),
            gold: p.gold(),
            pos_x: p.pos_x(),
            pos_y: p.pos_y(),
        }
    }
}

/// Type-erased read-only view of the engine's Player, used so `save.rs` can
/// serialise player state without importing the (read-only) `player_exact`
/// module. `GameState` (which owns the real `Player`) implements this.
pub trait PlayerSnapshot {
    fn name(&self) -> String;
    fn class_u8(&self) -> u8;
    fn level(&self) -> u8;
    fn plr_level(&self) -> u8;
    fn strength(&self) -> i32;
    fn base_str(&self) -> i32;
    fn magic(&self) -> i32;
    fn base_mag(&self) -> i32;
    fn dexterity(&self) -> i32;
    fn base_dex(&self) -> i32;
    fn vitality(&self) -> i32;
    fn base_vit(&self) -> i32;
    fn stat_pts(&self) -> i32;
    fn hp_base(&self) -> i32;
    fn max_hp_base(&self) -> i32;
    fn hit_points(&self) -> i32;
    fn max_hp(&self) -> i32;
    fn mana_base(&self) -> i32;
    fn max_mana_base(&self) -> i32;
    fn mana(&self) -> i32;
    fn max_mana(&self) -> i32;
    fn experience(&self) -> u32;
    fn gold(&self) -> i32;
    fn pos_x(&self) -> i32;
    fn pos_y(&self) -> i32;
}

/// Apply a `SlotPlayer` snapshot back onto a `PlayerSnapshot`-mutable view.
/// `GameState` provides a mutable impl that writes into its real `Player`.
pub trait PlayerSnapshotMut {
    fn apply_slot(&mut self, s: &SlotPlayer);
}

/// Apply a loaded `LevelSaveData` (monsters/objects/floor items) back onto the
/// engine. `GameState` implements this in `game_state.rs` to restore the
/// per-level state captured by the F5/F9 save path.
pub trait LevelStateMut {
    fn apply_level(&mut self, data: &crate::game::loadsave::LevelSaveData);
}

/// Apply a loaded `GameSaveDataEnvelope` (quests/portals/missiles) back onto
/// the engine. `GameState` implements this in `game_state.rs`.
pub trait GameStateMut {
    fn apply_game(&mut self, data: &crate::game::loadsave::GameSaveDataEnvelope);
}

/// Build a fresh `SaveSlot` (header + player + world) from snapshots + tick.
pub fn build_save_slot(
    player: &dyn PlayerSnapshot,
    world: &dyn WorldSnapshot,
) -> SaveSlot {
    let save_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    SaveSlot {
        magic: SaveSlot::MAGIC,
        version: SaveSlot::VERSION,
        save_time,
        player: SlotPlayer::from_snapshot(player),
        world: SlotWorld {
            dungeon_type: world.dungeon_type_u8(),
            in_dungeon: world.in_dungeon(),
            is_town: world.is_town(),
            game_tick: world.game_tick(),
            cam_x: world.cam_x(),
            cam_y: world.cam_y(),
        },
        // `build_save_slot` produces a v2-compatible body; callers wanting the
        // full level/game blobs use `build_save_slot_full`.
        level: None,
        game: None,
    }
}

/// Type-erased read-only view of `GameState` world fields.
pub trait WorldSnapshot {
    fn dungeon_type_u8(&self) -> u8;
    fn in_dungeon(&self) -> bool;
    fn is_town(&self) -> bool;
    fn game_tick(&self) -> u32;
    fn cam_x(&self) -> i32;
    fn cam_y(&self) -> i32;
}

/// Build a full `SaveSlot` including the level + game binary blobs (version 3).
/// Used by the F5/F9 path to capture the complete engine state. The `level`
/// snapshot is built from `level_src`, the `game` snapshot from `game_src`
/// (which itself embeds the level snapshot, so `level_src` is read twice —
/// once for the top-level `level` field, once inside `game.level`).
pub fn build_save_slot_full(
    player: &dyn PlayerSnapshot,
    world: &dyn WorldSnapshot,
    level_src: &dyn crate::game::loadsave::LevelSnapshot,
    game_src: &dyn crate::game::loadsave::GameSnapshot,
) -> SaveSlot {
    let mut slot = build_save_slot(player, world);
    let level_snap = crate::game::loadsave::build_level_snapshot(level_src);
    let game_snap = crate::game::loadsave::build_game_snapshot(game_src, level_src);
    slot.level = Some(crate::game::loadsave::LevelSaveData::from_snapshot(&level_snap));
    slot.game = Some(crate::game::loadsave::GameSaveDataEnvelope::from_snapshot(&game_snap));
    slot
}

#[cfg(test)]
mod slot_tests {
    use super::*;

    /// Round-trip a `SaveSlot` through JSON to prove the format is stable and
    /// lossless (including the 64x fixed-point HP/Mana values).
    #[test]
    fn test_save_slot_json_roundtrip() {
        let slot = SaveSlot {
            magic: SaveSlot::MAGIC,
            version: SaveSlot::VERSION,
            save_time: 12345,
            player: SlotPlayer {
                name: "Aidan".to_string(),
                class: 0,
                level: 7,
                plr_level: 3,
                strength: 30,
                base_str: 25,
                magic: 10,
                base_mag: 10,
                dexterity: 20,
                base_dex: 20,
                vitality: 25,
                base_vit: 25,
                stat_pts: 5,
                hp_base: 64 * 50,
                max_hp_base: 64 * 60,
                hit_points: 64 * 45,
                max_hp: 64 * 60,
                mana_base: 64 * 10,
                max_mana_base: 64 * 12,
                mana: 64 * 8,
                max_mana: 64 * 12,
                experience: 5000,
                gold: 750,
                pos_x: 56,
                pos_y: 56,
            },
            world: SlotWorld {
                dungeon_type: 2,
                in_dungeon: true,
                is_town: false,
                game_tick: 999,
                cam_x: 56,
                cam_y: 56,
            },
            level: None,
            game: None,
        };

        let json = serde_json::to_string(&slot).unwrap();
        let back: SaveSlot = serde_json::from_str(&json).unwrap();

        assert_eq!(slot, back);
        assert!(back.is_valid());
        // Spot-check a 64x value survived intact.
        assert_eq!(back.player.hit_points, 64 * 45);
        assert_eq!(back.player.max_mana, 64 * 12);
    }

    /// Round-trip a `SaveSlot` through the `SaveManager` on disk (slot 0 in a
    /// temp dir).
    #[test]
    fn test_save_slot_disk_roundtrip() {
        let tmp = std::env::temp_dir().join(format!(
            "devilutionx_rs_save_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mgr = SaveManager::with_dir(tmp.to_str().unwrap());

        let slot = SaveSlot {
            magic: SaveSlot::MAGIC,
            version: SaveSlot::VERSION,
            save_time: 1,
            player: SlotPlayer {
                name: "Test".to_string(),
                class: 1,
                level: 2,
                plr_level: 1,
                strength: 1,
                base_str: 1,
                magic: 1,
                base_mag: 1,
                dexterity: 1,
                base_dex: 1,
                vitality: 1,
                base_vit: 1,
                stat_pts: 0,
                hp_base: 64,
                max_hp_base: 64,
                hit_points: 64,
                max_hp: 64,
                mana_base: 0,
                max_mana_base: 0,
                mana: 0,
                max_mana: 0,
                experience: 0,
                gold: 0,
                pos_x: 1,
                pos_y: 2,
            },
            world: SlotWorld {
                dungeon_type: 1,
                in_dungeon: false,
                is_town: true,
                game_tick: 5,
                cam_x: 75,
                cam_y: 68,
            },
            level: None,
            game: None,
        };

        mgr.save_slot(0, &slot).unwrap();
        assert!(mgr.slot_exists_v2(0));
        let back = mgr.load_slot(0).unwrap();
        assert_eq!(slot, back);

        // Cleanup.
        let _ = std::fs::remove_dir_all(&tmp);
    }
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
}
