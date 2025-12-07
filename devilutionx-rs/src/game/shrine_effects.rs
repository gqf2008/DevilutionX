//! Shrine Effects - Player Integration (Day 26-30)
//!
//! This module implements all shrine effects that modify player stats.
//! Ported from Source/objects.cpp OperateShrine* functions.
//!
//! ## Shrine Types (27 total)
//!
//! ### Stat Modification (6 shrines)
//! - **Mysterious**: -1 all stats, +6 random stat
//! - **Hidden**: +10 max durability all items, -20 random item
//! - **Gloomy**: Weapons -1 max damage, Armor +2 AC
//! - **Weird**: Swap 2 random stats
//! - **Magical**: Cast random spell
//! - **Stone**: Recharge all staves
//!
//! ### Restoration (4 shrines)
//! - **Religious**: Restore all item durability to max
//! - **Enchanted**: +1 all spell levels, -1 random spell
//! - **Thaumaturgic**: All chests become chests
//! - **Fascinating**: Restore mana, fill empty slots with gold
//!
//! ### Combat (6 shrines)
//! - **Cryptic**: Cast Nova
//! - **Eldritch**: All shrines become potions
//! - **Eerie**: -2 all light radius
//! - **Divine**: Full heal + mana, spawn potions
//! - **Holy**: Cast Phasing
//! - **Sacred**: Cast chain lightning
//!
//! ### Utility (5 shrines)
//! - **Spiritual**: Fill empty inventory with gold
//! - **Spooky**: Teleport to random player
//! - **Abandoned**: +2 dexterity
//! - **Creepy**: +2 strength
//! - **Quiet**: +2 vitality
//!
//! ### Special (6 shrines)
//! - **Secluded**: Reveal full map
//! - **Ornate**: Cast Mana Shield
//! - **Glimmering**: Identify all items
//! - **Tainted**: -1 all resistances
//! - **Oily**: +1 all resistances (Hellfire)
//! - **Glowing**: +5% magic damage (Hellfire)
//! - **Mendicant**: Full heal, lose all gold (Hellfire)
//! - **Sparkling**: +1 random stat
//! - **Town**: Teleport to town
//! - **Shimmering**: +2 magic
//! - **Solar**: +2 random stat
//! - **Murphy's**: +1 all stats

use crate::game::player_exact::{Player, CharacterAttribute};

// =============================================================================
// M11 Day 87: ShrineType Enumeration + Configuration Data
// =============================================================================

/// Shrine type enumeration (34 types total)
///
/// C++ Reference: `enum shrine_type` (Source/objects.cpp:64-98)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ShrineType {
    Mysterious = 0,
    Hidden = 1,
    Gloomy = 2,
    Weird = 3,
    Magical = 4,
    Stone = 5,
    Religious = 6,
    Enchanted = 7,
    Thaumaturgic = 8,
    Fascinating = 9,
    Cryptic = 10,
    MagicalL2 = 11,
    Eldritch = 12,
    Eerie = 13,
    Divine = 14,
    Holy = 15,
    Sacred = 16,
    Spiritual = 17,
    Spooky = 18,
    Abandoned = 19,
    Creepy = 20,
    Quiet = 21,
    Secluded = 22,
    Ornate = 23,
    Glimmering = 24,
    Tainted = 25,
    Oily = 26,
    Glowing = 27,
    Mendicant = 28,
    Sparkling = 29,
    Town = 30,
    Shimmering = 31,
    Solar = 32,
    Murphys = 33,
}

impl ShrineType {
    /// Total number of shrine types
    pub const COUNT: usize = 34;

    /// Convert from u8 to ShrineType
    ///
    /// Returns None if value is out of range (>= 34)
    pub fn from_u8(value: u8) -> Option<Self> {
        if value >= Self::COUNT as u8 {
            return None;
        }
        // SAFETY: We've validated that value is in range [0, 33]
        Some(unsafe { std::mem::transmute(value) })
    }

    /// Get shrine name
    ///
    /// C++ Reference: `ShrineNames[]` (Source/objects.cpp:120-145)
    pub fn name(self) -> &'static str {
        SHRINE_DATA[self as usize].name
    }

    /// Get shrine game type availability
    ///
    /// C++ Reference: `shrineavail[]` (Source/objects.cpp:163-197)
    pub fn game_type(self) -> ShrineGameType {
        SHRINE_DATA[self as usize].game_type
    }

    /// Check if shrine is available in the given game mode
    ///
    /// # Arguments
    /// * `is_multiplayer` - true for multiplayer, false for single player
    ///
    /// # Returns
    /// true if shrine can appear in this game mode
    pub fn is_available_in_mode(self, is_multiplayer: bool) -> bool {
        match self.game_type() {
            ShrineGameType::Any => true,
            ShrineGameType::Single => !is_multiplayer,
            ShrineGameType::Multi => is_multiplayer,
        }
    }

    /// Get all shrine types available in the given game mode
    pub fn available_shrines(is_multiplayer: bool) -> Vec<ShrineType> {
        (0..Self::COUNT as u8)
            .filter_map(|i| Self::from_u8(i))
            .filter(|&shrine| shrine.is_available_in_mode(is_multiplayer))
            .collect()
    }
}

/// Shrine game type (availability in single/multi-player)
///
/// C++ Reference: `enum shrine_gametype` (Source/objects.cpp:156-161)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShrineGameType {
    /// Available in both single-player and multi-player
    Any,
    /// Available only in single-player
    Single,
    /// Available only in multi-player
    Multi,
}

/// Shrine configuration data
pub struct ShrineData {
    pub name: &'static str,
    pub game_type: ShrineGameType,
}

/// Shrine configuration table
///
/// C++ Reference:
/// - Names: `ShrineNames[]` (Source/objects.cpp:120-145)
/// - Availability: `shrineavail[]` (Source/objects.cpp:163-197)
pub const SHRINE_DATA: [ShrineData; 34] = [
    ShrineData { name: "Mysterious", game_type: ShrineGameType::Any },
    ShrineData { name: "Hidden", game_type: ShrineGameType::Any },
    ShrineData { name: "Gloomy", game_type: ShrineGameType::Single },
    ShrineData { name: "Weird", game_type: ShrineGameType::Single },
    ShrineData { name: "Magical", game_type: ShrineGameType::Any },
    ShrineData { name: "Stone", game_type: ShrineGameType::Any },
    ShrineData { name: "Religious", game_type: ShrineGameType::Any },
    ShrineData { name: "Enchanted", game_type: ShrineGameType::Any },
    ShrineData { name: "Thaumaturgic", game_type: ShrineGameType::Single },
    ShrineData { name: "Fascinating", game_type: ShrineGameType::Any },
    ShrineData { name: "Cryptic", game_type: ShrineGameType::Any },
    ShrineData { name: "Magical", game_type: ShrineGameType::Any }, // MagicalL2 (same name)
    ShrineData { name: "Eldritch", game_type: ShrineGameType::Any },
    ShrineData { name: "Eerie", game_type: ShrineGameType::Any },
    ShrineData { name: "Divine", game_type: ShrineGameType::Any },
    ShrineData { name: "Holy", game_type: ShrineGameType::Any },
    ShrineData { name: "Sacred", game_type: ShrineGameType::Any },
    ShrineData { name: "Spiritual", game_type: ShrineGameType::Any },
    ShrineData { name: "Spooky", game_type: ShrineGameType::Multi },
    ShrineData { name: "Abandoned", game_type: ShrineGameType::Any },
    ShrineData { name: "Creepy", game_type: ShrineGameType::Any },
    ShrineData { name: "Quiet", game_type: ShrineGameType::Any },
    ShrineData { name: "Secluded", game_type: ShrineGameType::Any },
    ShrineData { name: "Ornate", game_type: ShrineGameType::Any },
    ShrineData { name: "Glimmering", game_type: ShrineGameType::Any },
    ShrineData { name: "Tainted", game_type: ShrineGameType::Multi },
    ShrineData { name: "Oily", game_type: ShrineGameType::Any },
    ShrineData { name: "Glowing", game_type: ShrineGameType::Any },
    ShrineData { name: "Mendicant", game_type: ShrineGameType::Any },
    ShrineData { name: "Sparkling", game_type: ShrineGameType::Any },
    ShrineData { name: "Town", game_type: ShrineGameType::Any },
    ShrineData { name: "Shimmering", game_type: ShrineGameType::Any },
    ShrineData { name: "Solar", game_type: ShrineGameType::Single },
    ShrineData { name: "Murphy's", game_type: ShrineGameType::Any },
];

// =============================================================================
// M11 Day 26-30: Shrine Effect Functions (Pre-existing)
// =============================================================================

/// Apply Mysterious Shrine effect
///
/// C++ Reference: `OperateShrineMysterious()` (Source/objects.cpp:2252)
///
/// Effect:
/// - -1 to all stats (Str, Mag, Dex, Vit)
/// - +6 to one random stat
///
/// Net result: +2 to one stat, -1 to the other three
pub fn apply_mysterious(player: &mut Player, random_stat: CharacterAttribute) {
    // Decrease all stats by 1
    player.set_base_attribute(CharacterAttribute::Strength,
        player.get_base_attribute(CharacterAttribute::Strength) - 1);
    player.set_base_attribute(CharacterAttribute::Magic,
        player.get_base_attribute(CharacterAttribute::Magic) - 1);
    player.set_base_attribute(CharacterAttribute::Dexterity,
        player.get_base_attribute(CharacterAttribute::Dexterity) - 1);
    player.set_base_attribute(CharacterAttribute::Vitality,
        player.get_base_attribute(CharacterAttribute::Vitality) - 1);

    // Increase random stat by 6
    let current = player.get_base_attribute(random_stat);
    player.set_base_attribute(random_stat, current + 6);

    // TODO: CheckStats(player) - validate stat ranges
    // TODO: CalcPlrInv(player, true) - recalculate inventory bonuses
}

/// Apply Hidden Shrine effect
///
/// C++ Reference: `OperateShrineHidden()` (Source/objects.cpp:2284)
///
/// Effect:
/// - +10 max durability to all equipped items
/// - -20 max durability to one random equipped item
///
/// For now, this is a stub (requires full Item system)
pub fn apply_hidden(_player: &mut Player, _random_item_index: usize) {
    // TODO: Requires Item system with durability
    // 1. Iterate player.inv_body
    // 2. Add 10 to _iDurability and _iMaxDur (skip indestructible)
    // 3. Subtract 20 from random item
}

/// Apply Gloomy Shrine effect
///
/// C++ Reference: `OperateShrineGloomy()` (Source/objects.cpp:2331)
///
/// Effect:
/// - All weapons: -1 max damage
/// - All armor: +2 AC
///
/// For now, this is a stub (requires full Item system)
pub fn apply_gloomy(_player: &mut Player) {
    // TODO: Requires Item system with damage/AC modifiers
}

/// Apply Weird Shrine effect
///
/// C++ Reference: `OperateShrineWeird()` (Source/objects.cpp:2365)
///
/// Effect:
/// - Swap values of two random stats
pub fn apply_weird(player: &mut Player, stat1: CharacterAttribute, stat2: CharacterAttribute) {
    let val1 = player.get_base_attribute(stat1);
    let val2 = player.get_base_attribute(stat2);

    player.set_base_attribute(stat1, val2);
    player.set_base_attribute(stat2, val1);

    // TODO: CheckStats(player) - validate stat ranges
    // TODO: CalcPlrInv(player, true) - recalculate bonuses
}

/// Apply Magical Shrine effect
///
/// C++ Reference: `OperateShrinemagical()` (Source/objects.cpp:2394)
///
/// Effect:
/// - Cast a random spell
///
/// For now, this is a stub (requires spell system)
pub fn apply_magical(_player: &Player, _random_spell_id: i32) {
    // TODO: Requires spell casting system
    // AddMissile(player.position.tile, target, direction, spell, ...)
}

/// Apply Stone Shrine effect
///
/// C++ Reference: `OperateShrineStone()` (Source/objects.cpp:2412)
///
/// Effect:
/// - Recharge all staves to full
///
/// For now, this is a stub (requires Item system)
pub fn apply_stone(_player: &mut Player) {
    // TODO: Requires Item system with charges
    // Set _iCharges = _iMaxCharges for all staves
}

/// Apply Religious Shrine effect
///
/// C++ Reference: `OperateShrineReligious()` (Source/objects.cpp:2429)
///
/// Effect:
/// - Restore all item durability to maximum
///
/// For now, this is a stub (requires Item system)
pub fn apply_religious(_player: &mut Player) {
    // TODO: Requires Item system with durability
    // Set _iDurability = _iMaxDur for all items
}

/// Apply Enchanted Shrine effect
///
/// C++ Reference: `OperateShrineEnchanted()` (Source/objects.cpp:2441)
///
/// Effect:
/// - +1 level to all known spells
/// - -1 level to one random spell
///
/// For now, this is a stub (requires spell system)
pub fn apply_enchanted(_player: &mut Player, _random_spell_id: i32) {
    // TODO: Requires spell system
    // Iterate _pSplLvl[], increment all (except random), clamp to MaxSpellLevel
}

/// Apply Thaumaturgic Shrine effect
///
/// C++ Reference: `OperateShrineThaumaturgic()` (Source/objects.cpp:2489)
///
/// Effect:
/// - Convert all non-unique monsters to chests
///
/// This requires monster system integration
pub fn apply_thaumaturgic() {
    // TODO: Requires monster system
    // Iterate monsters[], convert non-unique to chests
}

/// Apply Fascinating Shrine effect (Hellfire)
///
/// C++ Reference: `OperateShrineFascinating()` (Source/objects.cpp:~2500)
///
/// Effect:
/// - Restore mana to max
/// - Fill empty inventory slots with gold
///
/// Partial implementation (mana restore only)
pub fn apply_fascinating(player: &mut Player) {
    player.restore_mana_to_full();
    // TODO: Fill empty inventory slots with gold
}

/// Apply Cryptic Shrine effect
///
/// C++ Reference: `OperateShrineCryptic()` (Source/objects.cpp:2550)
///
/// Effect:
/// - Cast Nova spell at player position
///
/// For now, this is a stub (requires spell system)
pub fn apply_cryptic(_player: &Player) {
    // TODO: Requires spell system
    // AddMissile(player.position.tile, {0,0}, Direction::South, MissileID::Nova, ...)
}

/// Apply Eldritch Shrine effect
///
/// C++ Reference: `OperateShrineEldritch()` (Source/objects.cpp:2573)
///
/// Effect:
/// - Convert all shrines on current level to potions
///
/// This requires object system integration
pub fn apply_eldritch() {
    // TODO: Requires object iteration
    // Convert all shrine objects to healing potions
}

/// Apply Eerie Shrine effect
///
/// C++ Reference: `OperateShrineEerie()` (Source/objects.cpp:2606)
///
/// Effect:
/// - -2 to light radius
///
/// For now, this is a stub (requires light system)
pub fn apply_eerie(player: &mut Player) {
    player._p_light_rad -= 2;
    // TODO: Update light rendering
    // ChangeLightRadius(myPlayer.lightId, myPlayer._pLightRad)
}

/// Apply Divine Shrine effect
///
/// C++ Reference: `OperateShrineDivine()` (Source/objects.cpp:2625)
///
/// Effect:
/// - Full heal HP and Mana
/// - Spawn healing potions at shrine location
pub fn apply_divine(player: &mut Player) {
    player.heal_to_full();
    player.restore_mana_to_full();

    // TODO: Spawn potions
    // CreateTypeItem(spawnPosition, ..., ItemType::Misc, IMISC_FULLMANA, ...)
}

/// Apply Holy Shrine effect
///
/// C++ Reference: `OperateShrineHoly()` (Source/objects.cpp:2648)
///
/// Effect:
/// - Cast Phasing spell (damages nearby monsters)
///
/// For now, this is a stub (requires spell system)
pub fn apply_holy(_player: &Player) {
    // TODO: Requires spell system
    // AddMissile(player.position.tile, {0,0}, Direction::South, MissileID::Phasing, ...)
}

/// Apply Sacred Shrine effect (Hellfire)
///
/// C++ Reference: `OperateShrineSacred()` (Source/objects.cpp:~2660)
///
/// Effect:
/// - Cast chain lightning
///
/// For now, this is a stub (requires spell system)
pub fn apply_sacred(_player: &Player) {
    // TODO: Requires spell system
    // AddMissile(..., MissileID::ChainLightning, ...)
}

/// Apply Spiritual Shrine effect
///
/// C++ Reference: `OperateShrineSpiritual()` (Source/objects.cpp:2658)
///
/// Effect:
/// - Fill all empty inventory slots with gold
///
/// For now, this is a stub (requires inventory system)
pub fn apply_spiritual(_player: &mut Player, _level: u8) {
    // TODO: Requires inventory system
    // For each empty InvGrid cell:
    //   Create gold item (5 * level + random(10 * level))
}

/// Apply Spooky Shrine effect
///
/// C++ Reference: `OperateShrineSpooky()` (Source/objects.cpp:2677)
///
/// Effect:
/// - Teleport to a random player
///
/// For now, this is a stub (requires multiplayer and teleport)
pub fn apply_spooky(_player: &mut Player) {
    // TODO: Requires multiplayer system
    // WalkInDir(...) to random player position
}

/// Apply Abandoned Shrine effect
///
/// C++ Reference: `OperateShrineAbandoned()` (Source/objects.cpp:2696)
///
/// Effect:
/// - +2 Dexterity
pub fn apply_abandoned(player: &mut Player) {
    let current = player.get_base_attribute(CharacterAttribute::Dexterity);
    player.set_base_attribute(CharacterAttribute::Dexterity, current + 2);
    // TODO: CheckStats(player)
}

/// Apply Creepy Shrine effect
///
/// C++ Reference: `OperateShrineCreepy()` (Source/objects.cpp:2709)
///
/// Effect:
/// - +2 Strength
pub fn apply_creepy(player: &mut Player) {
    let current = player.get_base_attribute(CharacterAttribute::Strength);
    player.set_base_attribute(CharacterAttribute::Strength, current + 2);
    // TODO: CheckStats(player)
}

/// Apply Quiet Shrine effect
///
/// C++ Reference: `OperateShrineQuiet()` (Source/objects.cpp:2722)
///
/// Effect:
/// - +2 Vitality
pub fn apply_quiet(player: &mut Player) {
    let current = player.get_base_attribute(CharacterAttribute::Vitality);
    player.set_base_attribute(CharacterAttribute::Vitality, current + 2);
    // TODO: CheckStats(player)
}

/// Apply Secluded Shrine effect
///
/// C++ Reference: `OperateShrineSecluded()` (Source/objects.cpp:2735)
///
/// Effect:
/// - Reveal entire dungeon map
///
/// For now, this is a stub (requires map system)
pub fn apply_secluded(_player: &Player) {
    // TODO: Requires map system
    // Set automapflag for all tiles
}

/// Apply Ornate Shrine effect (Hellfire)
///
/// C++ Reference: `OperateShrineOrnate()` (Source/objects.cpp:~2750)
///
/// Effect:
/// - Cast Mana Shield spell
///
/// For now, this is a stub (requires spell system)
pub fn apply_ornate(_player: &mut Player) {
    // TODO: Requires spell system
    // player.pManaShield = true
}

/// Apply Glimmering Shrine effect (Hellfire)
///
/// C++ Reference: `OperateShrineGlimmering()` (Source/objects.cpp:~2760)
///
/// Effect:
/// - Identify all items in inventory
///
/// For now, this is a stub (requires item system)
pub fn apply_glimmering(_player: &mut Player) {
    // TODO: Requires item system
    // Set _iIdentified = true for all items
}

/// Apply Tainted Shrine effect (Hellfire)
///
/// C++ Reference: `OperateShrineTainted()` (Source/objects.cpp:~2770)
///
/// Effect:
/// - -1 to all resistances (Fire, Lightning, Magic)
pub fn apply_tainted(player: &mut Player) {
    player.modify_resistance("fire", -1);
    player.modify_resistance("lightning", -1);
    player.modify_resistance("magic", -1);
}

/// Apply Oily Shrine effect (Hellfire)
///
/// C++ Reference: `OperateShrineOily()` (Source/objects.cpp:~2785)
///
/// Effect:
/// - +1 to all resistances (Fire, Lightning, Magic)
pub fn apply_oily(player: &mut Player) {
    player.modify_resistance("fire", 1);
    player.modify_resistance("lightning", 1);
    player.modify_resistance("magic", 1);
}

/// Apply Glowing Shrine effect (Hellfire)
///
/// C++ Reference: `OperateShrineGlowing()` (Source/objects.cpp:~2800)
///
/// Effect:
/// - +5% magic damage
///
/// For now, this is a stub (requires item system)
pub fn apply_glowing(_player: &mut Player) {
    // TODO: Requires item bonus system
    // player._pIBonusDamMod += 5
}

/// Apply Mendicant Shrine effect (Hellfire)
///
/// C++ Reference: `OperateShrineMendicant()` (Source/objects.cpp:~2815)
///
/// Effect:
/// - Full heal HP
/// - Lose all gold
pub fn apply_mendicant(player: &mut Player) {
    player.heal_to_full();
    player._p_gold = 0;
    // TODO: Remove gold from inventory
}

/// Apply Sparkling Shrine effect (Hellfire)
///
/// C++ Reference: `OperateShrineSparkling()` (Source/objects.cpp:~2830)
///
/// Effect:
/// - +1 to one random stat
pub fn apply_sparkling(player: &mut Player, random_stat: CharacterAttribute) {
    let current = player.get_base_attribute(random_stat);
    player.set_base_attribute(random_stat, current + 1);
    // TODO: CheckStats(player)
}

/// Apply Town Shrine effect (Hellfire)
///
/// C++ Reference: `OperateShrineTown()` (Source/objects.cpp:~2845)
///
/// Effect:
/// - Teleport player to town
///
/// For now, this is a stub (requires level system)
pub fn apply_town(_player: &mut Player) {
    // TODO: Requires level transition system
    // StartNewLvl(..., WM_DIABSETLVL, 0)
}

/// Apply Shimmering Shrine effect (Hellfire)
///
/// C++ Reference: `OperateShrineShimmering()` (Source/objects.cpp:~2860)
///
/// Effect:
/// - +2 Magic
pub fn apply_shimmering(player: &mut Player) {
    let current = player.get_base_attribute(CharacterAttribute::Magic);
    player.set_base_attribute(CharacterAttribute::Magic, current + 2);
    // TODO: CheckStats(player)
}

/// Apply Solar Shrine effect (Hellfire)
///
/// C++ Reference: `OperateShrineSolar()` (Source/objects.cpp:~2875)
///
/// Effect:
/// - +2 to one random stat
pub fn apply_solar(player: &mut Player, random_stat: CharacterAttribute) {
    let current = player.get_base_attribute(random_stat);
    player.set_base_attribute(random_stat, current + 2);
    // TODO: CheckStats(player)
}

/// Apply Murphy's Shrine effect (Hellfire)
///
/// C++ Reference: `OperateShrineMurphys()` (Source/objects.cpp:~2890)
///
/// Effect:
/// - +1 to all stats (Str, Mag, Dex, Vit)
pub fn apply_murphys(player: &mut Player) {
    player.set_base_attribute(CharacterAttribute::Strength,
        player.get_base_attribute(CharacterAttribute::Strength) + 1);
    player.set_base_attribute(CharacterAttribute::Magic,
        player.get_base_attribute(CharacterAttribute::Magic) + 1);
    player.set_base_attribute(CharacterAttribute::Dexterity,
        player.get_base_attribute(CharacterAttribute::Dexterity) + 1);
    player.set_base_attribute(CharacterAttribute::Vitality,
        player.get_base_attribute(CharacterAttribute::Vitality) + 1);
    // TODO: CheckStats(player)
}

// =============================================================================
// M11 Day 88: operate_shrine() Main Dispatcher + Helper Functions
// =============================================================================

/// Shrine operation result type
pub type ShrineResult = Result<ShrineEffect, ShrineError>;

/// Shrine effect variants
///
/// Represents the different types of effects a shrine can produce.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShrineEffect {
    /// Player stats were modified
    StatModified,
    /// Player items were modified
    ItemModified,
    /// Player was healed
    Healed,
    /// Player mana was restored
    ManaRestored,
    /// A spell was cast
    SpellCast,
    /// Player was teleported
    Teleported,
    /// Dungeon map was revealed
    MapRevealed,
    /// Items were identified
    ItemsIdentified,
    /// No visible effect (for future shrines)
    NoEffect,
}

/// Shrine operation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShrineError {
    /// Shrine not available in current game mode
    NotAvailable,
    /// Requirements not met (e.g., no items to modify)
    RequirementsNotMet,
    /// Other error with description
    Other(String),
}

impl std::fmt::Display for ShrineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShrineError::NotAvailable => write!(f, "Shrine not available in current game mode"),
            ShrineError::RequirementsNotMet => write!(f, "Requirements not met"),
            ShrineError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ShrineError {}

/// Random character attribute selector
///
/// Selects one of the four base attributes at random.
fn random_character_attribute(seed: u32) -> CharacterAttribute {
    let attributes = [
        CharacterAttribute::Strength,
        CharacterAttribute::Magic,
        CharacterAttribute::Dexterity,
        CharacterAttribute::Vitality,
    ];
    attributes[(seed % 4) as usize]
}

/// Random two different stats selector
///
/// Selects two different attributes for swapping (Weird Shrine).
fn random_two_stats(seed: u32) -> (CharacterAttribute, CharacterAttribute) {
    let attributes = [
        CharacterAttribute::Strength,
        CharacterAttribute::Magic,
        CharacterAttribute::Dexterity,
        CharacterAttribute::Vitality,
    ];
    let idx1 = (seed % 4) as usize;
    let idx2 = ((seed / 4) % 3) as usize;
    let idx2 = if idx2 >= idx1 { idx2 + 1 } else { idx2 };
    (attributes[idx1], attributes[idx2 % 4])
}

/// Random stat by time of day (Solar Shrine)
///
/// C++ Reference: `OperateShrineSolar()` (Source/objects.cpp:2936-2957)
///
/// Returns attribute based on current hour:
/// - 4:00-11:59 → Dexterity
/// - 12:00-17:59 → Strength
/// - 18:00-19:59 → Magic
/// - 20:00-3:59 → Vitality
fn random_stat_by_time() -> CharacterAttribute {
    use std::time::SystemTime;

    // Get current hour (0-23)
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let hour = ((now / 3600) % 24) as u8;

    if hour >= 20 || hour < 4 {
        CharacterAttribute::Vitality  // 20:00-3:59
    } else if hour >= 18 {
        CharacterAttribute::Magic      // 18:00-19:59
    } else if hour >= 12 {
        CharacterAttribute::Strength   // 12:00-17:59
    } else {
        CharacterAttribute::Dexterity  // 4:00-11:59
    }
}

/// Main shrine activation dispatcher
///
/// C++ Reference: `OperateShrine()` (Source/objects.cpp:3007-3092)
///
/// # Arguments
/// * `player` - Mutable reference to the player
/// * `shrine_type` - Type of shrine to activate
/// * `shrine_seed` - Random seed for shrine effects (from Object._oRndSeed)
///
/// # Returns
/// Result containing the effect type or an error
///
/// # Notes
/// Some shrine effects are deferred (require spell/item/map systems):
/// - Magical, Cryptic, Holy, Sacred, Ornate: Require spell casting
/// - Hidden, Gloomy, Stone, Religious, Glimmering, Glowing: Require item system
/// - Thaumaturgic, Eldritch: Require monster/object systems
/// - Secluded: Requires map system
/// - Spooky, Town: Require teleport/level systems
pub fn operate_shrine(
    player: &mut Player,
    shrine_type: ShrineType,
    shrine_seed: u32,
) -> ShrineResult {
    match shrine_type {
        ShrineType::Mysterious => {
            let random_stat = random_character_attribute(shrine_seed);
            apply_mysterious(player, random_stat);
            Ok(ShrineEffect::StatModified)
        }
        ShrineType::Hidden => {
            let random_item_index = (shrine_seed % 7) as usize;
            apply_hidden(player, random_item_index);
            Ok(ShrineEffect::ItemModified)
        }
        ShrineType::Gloomy => {
            apply_gloomy(player);
            Ok(ShrineEffect::ItemModified)
        }
        ShrineType::Weird => {
            let (stat1, stat2) = random_two_stats(shrine_seed);
            apply_weird(player, stat1, stat2);
            Ok(ShrineEffect::StatModified)
        }
        ShrineType::Magical | ShrineType::MagicalL2 => {
            let random_spell_id = (shrine_seed % 37) as i32;
            apply_magical(player, random_spell_id);
            Ok(ShrineEffect::SpellCast)
        }
        ShrineType::Stone => {
            apply_stone(player);
            Ok(ShrineEffect::ItemModified)
        }
        ShrineType::Religious => {
            apply_religious(player);
            Ok(ShrineEffect::ItemModified)
        }
        ShrineType::Enchanted => {
            let random_spell_id = (shrine_seed % 37) as i32;
            apply_enchanted(player, random_spell_id);
            Ok(ShrineEffect::StatModified)
        }
        ShrineType::Thaumaturgic => {
            apply_thaumaturgic();
            Ok(ShrineEffect::NoEffect)
        }
        ShrineType::Fascinating => {
            apply_fascinating(player);
            Ok(ShrineEffect::ManaRestored)
        }
        ShrineType::Cryptic => {
            apply_cryptic(player);
            Ok(ShrineEffect::SpellCast)
        }
        ShrineType::Eldritch => {
            apply_eldritch();
            Ok(ShrineEffect::NoEffect)
        }
        ShrineType::Eerie => {
            apply_eerie(player);
            Ok(ShrineEffect::StatModified)
        }
        ShrineType::Divine => {
            apply_divine(player);
            Ok(ShrineEffect::Healed)
        }
        ShrineType::Holy => {
            apply_holy(player);
            Ok(ShrineEffect::SpellCast)
        }
        ShrineType::Sacred => {
            apply_sacred(player);
            Ok(ShrineEffect::SpellCast)
        }
        ShrineType::Spiritual => {
            let level = player._p_level;
            apply_spiritual(player, level);
            Ok(ShrineEffect::ItemModified)
        }
        ShrineType::Spooky => {
            apply_spooky(player);
            Ok(ShrineEffect::Teleported)
        }
        ShrineType::Abandoned => {
            apply_abandoned(player);
            Ok(ShrineEffect::StatModified)
        }
        ShrineType::Creepy => {
            apply_creepy(player);
            Ok(ShrineEffect::StatModified)
        }
        ShrineType::Quiet => {
            apply_quiet(player);
            Ok(ShrineEffect::StatModified)
        }
        ShrineType::Secluded => {
            apply_secluded(player);
            Ok(ShrineEffect::MapRevealed)
        }
        ShrineType::Ornate => {
            apply_ornate(player);
            Ok(ShrineEffect::SpellCast)
        }
        ShrineType::Glimmering => {
            apply_glimmering(player);
            Ok(ShrineEffect::ItemsIdentified)
        }
        ShrineType::Tainted => {
            apply_tainted(player);
            Ok(ShrineEffect::StatModified)
        }
        ShrineType::Oily => {
            apply_oily(player);
            Ok(ShrineEffect::StatModified)
        }
        ShrineType::Glowing => {
            apply_glowing(player);
            Ok(ShrineEffect::ItemModified)
        }
        ShrineType::Mendicant => {
            apply_mendicant(player);
            Ok(ShrineEffect::Healed)
        }
        ShrineType::Sparkling => {
            let random_stat = random_character_attribute(shrine_seed);
            apply_sparkling(player, random_stat);
            Ok(ShrineEffect::StatModified)
        }
        ShrineType::Town => {
            apply_town(player);
            Ok(ShrineEffect::Teleported)
        }
        ShrineType::Shimmering => {
            apply_shimmering(player);
            Ok(ShrineEffect::StatModified)
        }
        ShrineType::Solar => {
            let stat = random_stat_by_time();
            apply_solar(player, stat);
            Ok(ShrineEffect::StatModified)
        }
        ShrineType::Murphys => {
            apply_murphys(player);
            Ok(ShrineEffect::StatModified)
        }
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::player_exact::HeroClass;

    fn create_test_player() -> Player {
        let mut player = Player::new();
        player._p_class = HeroClass::Warrior;
        player._p_level = 10;

        // Set base stats
        player.set_base_attribute(CharacterAttribute::Strength, 30);
        player.set_base_attribute(CharacterAttribute::Magic, 20);
        player.set_base_attribute(CharacterAttribute::Dexterity, 25);
        player.set_base_attribute(CharacterAttribute::Vitality, 35);

        // Set HP/Mana
        player._p_max_hp = 200 * 64;
        player._p_max_hp_base = 200 * 64;
        player._p_hit_points = 150 * 64;
        player._p_hp_base = 150 * 64;

        player._p_max_mana = 100 * 64;
        player._p_max_mana_base = 100 * 64;
        player._p_mana = 50 * 64;
        player._p_mana_base = 50 * 64;

        player
    }

    #[test]
    fn test_mysterious_shrine() {
        let mut player = create_test_player();

        // Apply Mysterious with +6 to Strength
        apply_mysterious(&mut player, CharacterAttribute::Strength);

        // Strength: 30 - 1 + 6 = 35
        assert_eq!(player.get_base_attribute(CharacterAttribute::Strength), 35);
        // Others: -1 each
        assert_eq!(player.get_base_attribute(CharacterAttribute::Magic), 19);
        assert_eq!(player.get_base_attribute(CharacterAttribute::Dexterity), 24);
        assert_eq!(player.get_base_attribute(CharacterAttribute::Vitality), 34);
    }

    #[test]
    fn test_weird_shrine() {
        let mut player = create_test_player();

        // Swap Strength (30) and Magic (20)
        apply_weird(&mut player, CharacterAttribute::Strength, CharacterAttribute::Magic);

        assert_eq!(player.get_base_attribute(CharacterAttribute::Strength), 20);
        assert_eq!(player.get_base_attribute(CharacterAttribute::Magic), 30);
    }

    #[test]
    fn test_abandoned_shrine() {
        let mut player = create_test_player();

        apply_abandoned(&mut player);

        // Dexterity: 25 + 2 = 27
        assert_eq!(player.get_base_attribute(CharacterAttribute::Dexterity), 27);
    }

    #[test]
    fn test_creepy_shrine() {
        let mut player = create_test_player();

        apply_creepy(&mut player);

        // Strength: 30 + 2 = 32
        assert_eq!(player.get_base_attribute(CharacterAttribute::Strength), 32);
    }

    #[test]
    fn test_quiet_shrine() {
        let mut player = create_test_player();

        apply_quiet(&mut player);

        // Vitality: 35 + 2 = 37
        assert_eq!(player.get_base_attribute(CharacterAttribute::Vitality), 37);
    }

    #[test]
    fn test_shimmering_shrine() {
        let mut player = create_test_player();

        apply_shimmering(&mut player);

        // Magic: 20 + 2 = 22
        assert_eq!(player.get_base_attribute(CharacterAttribute::Magic), 22);
    }

    #[test]
    fn test_divine_shrine() {
        let mut player = create_test_player();

        // HP at 150/200, Mana at 50/100
        apply_divine(&mut player);

        // Should be full
        assert_eq!(player.get_hp(), 200);
        assert_eq!(player.get_mana(), 100);
    }

    #[test]
    fn test_fascinating_shrine() {
        let mut player = create_test_player();

        // Mana at 50/100
        apply_fascinating(&mut player);

        // Should restore mana to full
        assert_eq!(player.get_mana(), 100);
    }

    #[test]
    fn test_tainted_shrine() {
        let mut player = create_test_player();

        // Start with some resistances
        player.modify_resistance("fire", 30);
        player.modify_resistance("lightning", 25);
        player.modify_resistance("magic", 20);

        apply_tainted(&mut player);

        // Each reduced by 1
        assert_eq!(player.get_resistance("fire"), 29);
        assert_eq!(player.get_resistance("lightning"), 24);
        assert_eq!(player.get_resistance("magic"), 19);
    }

    #[test]
    fn test_oily_shrine() {
        let mut player = create_test_player();

        apply_oily(&mut player);

        // Each increased by 1
        assert_eq!(player.get_resistance("fire"), 1);
        assert_eq!(player.get_resistance("lightning"), 1);
        assert_eq!(player.get_resistance("magic"), 1);
    }

    #[test]
    fn test_mendicant_shrine() {
        let mut player = create_test_player();
        player._p_gold = 1000;

        // HP at 150/200
        apply_mendicant(&mut player);

        // Should heal to full and lose all gold
        assert_eq!(player.get_hp(), 200);
        assert_eq!(player._p_gold, 0);
    }

    #[test]
    fn test_sparkling_shrine() {
        let mut player = create_test_player();

        apply_sparkling(&mut player, CharacterAttribute::Dexterity);

        // Dexterity: 25 + 1 = 26
        assert_eq!(player.get_base_attribute(CharacterAttribute::Dexterity), 26);
    }

    #[test]
    fn test_solar_shrine() {
        let mut player = create_test_player();

        apply_solar(&mut player, CharacterAttribute::Vitality);

        // Vitality: 35 + 2 = 37
        assert_eq!(player.get_base_attribute(CharacterAttribute::Vitality), 37);
    }

    #[test]
    fn test_murphys_shrine() {
        let mut player = create_test_player();

        apply_murphys(&mut player);

        // All stats: +1
        assert_eq!(player.get_base_attribute(CharacterAttribute::Strength), 31);
        assert_eq!(player.get_base_attribute(CharacterAttribute::Magic), 21);
        assert_eq!(player.get_base_attribute(CharacterAttribute::Dexterity), 26);
        assert_eq!(player.get_base_attribute(CharacterAttribute::Vitality), 36);
    }

    #[test]
    fn test_eerie_shrine() {
        let mut player = create_test_player();
        player._p_light_rad = 10;

        apply_eerie(&mut player);

        // Light radius: 10 - 2 = 8
        assert_eq!(player._p_light_rad, 8);
    }

    // =============================================================================
    // Test Group 2: M11 Day 87 - ShrineType Enumeration Tests
    // =============================================================================

    #[test]
    fn test_shrine_type_from_u8_valid() {
        // Test all valid shrine types
        assert_eq!(ShrineType::from_u8(0), Some(ShrineType::Mysterious));
        assert_eq!(ShrineType::from_u8(1), Some(ShrineType::Hidden));
        assert_eq!(ShrineType::from_u8(10), Some(ShrineType::Cryptic));
        assert_eq!(ShrineType::from_u8(33), Some(ShrineType::Murphys));
    }

    #[test]
    fn test_shrine_type_from_u8_invalid() {
        // Test out of range values
        assert_eq!(ShrineType::from_u8(34), None);
        assert_eq!(ShrineType::from_u8(100), None);
        assert_eq!(ShrineType::from_u8(255), None);
    }

    #[test]
    fn test_shrine_type_count() {
        // Verify COUNT constant
        assert_eq!(ShrineType::COUNT, 34);
    }

    #[test]
    fn test_shrine_data_lookup() {
        // Test name lookup
        assert_eq!(ShrineType::Mysterious.name(), "Mysterious");
        assert_eq!(ShrineType::Divine.name(), "Divine");
        assert_eq!(ShrineType::Murphys.name(), "Murphy's");

        // Verify all names are non-empty
        for i in 0..ShrineType::COUNT {
            let shrine = ShrineType::from_u8(i as u8).unwrap();
            assert!(!shrine.name().is_empty(), "Shrine {} has empty name", i);
        }
    }

    #[test]
    fn test_shrine_game_type_single() {
        // Single-player only shrines
        assert_eq!(ShrineType::Gloomy.game_type(), ShrineGameType::Single);
        assert_eq!(ShrineType::Weird.game_type(), ShrineGameType::Single);
        assert_eq!(ShrineType::Thaumaturgic.game_type(), ShrineGameType::Single);
        assert_eq!(ShrineType::Solar.game_type(), ShrineGameType::Single);
    }

    #[test]
    fn test_shrine_game_type_multi() {
        // Multi-player only shrines
        assert_eq!(ShrineType::Spooky.game_type(), ShrineGameType::Multi);
        assert_eq!(ShrineType::Tainted.game_type(), ShrineGameType::Multi);
    }

    #[test]
    fn test_shrine_game_type_any() {
        // Any game mode shrines
        assert_eq!(ShrineType::Mysterious.game_type(), ShrineGameType::Any);
        assert_eq!(ShrineType::Divine.game_type(), ShrineGameType::Any);
        assert_eq!(ShrineType::Murphys.game_type(), ShrineGameType::Any);
    }

    #[test]
    fn test_shrine_availability_sp() {
        // Single-player mode
        assert!(ShrineType::Mysterious.is_available_in_mode(false));
        assert!(ShrineType::Gloomy.is_available_in_mode(false)); // SP only
        assert!(!ShrineType::Spooky.is_available_in_mode(false)); // MP only
        assert!(!ShrineType::Tainted.is_available_in_mode(false)); // MP only

        // Count SP-available shrines (should be 32: 30 Any + 4 Single - 2 Multi)
        let sp_shrines = ShrineType::available_shrines(false);
        assert_eq!(sp_shrines.len(), 32);
    }

    #[test]
    fn test_shrine_availability_mp() {
        // Multi-player mode
        assert!(ShrineType::Mysterious.is_available_in_mode(true));
        assert!(!ShrineType::Gloomy.is_available_in_mode(true)); // SP only
        assert!(ShrineType::Spooky.is_available_in_mode(true)); // MP only
        assert!(ShrineType::Tainted.is_available_in_mode(true)); // MP only

        // Count MP-available shrines (should be 32: 30 Any + 2 Multi - 4 Single)
        let mp_shrines = ShrineType::available_shrines(true);
        assert_eq!(mp_shrines.len(), 30);
    }

    // =============================================================================
    // Test Group 3: M11 Day 88 - operate_shrine() Dispatcher Tests
    // =============================================================================

    #[test]
    fn test_operate_shrine_mysterious() {
        let mut player = create_test_player();
        let result = operate_shrine(&mut player, ShrineType::Mysterious, 0);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ShrineEffect::StatModified);

        // Verify stat changes: -1 all, +6 to one (seed 0 → Strength)
        assert_eq!(player.get_base_attribute(CharacterAttribute::Strength), 35);
        assert_eq!(player.get_base_attribute(CharacterAttribute::Magic), 19);
        assert_eq!(player.get_base_attribute(CharacterAttribute::Dexterity), 24);
        assert_eq!(player.get_base_attribute(CharacterAttribute::Vitality), 34);
    }

    #[test]
    fn test_operate_shrine_weird() {
        let mut player = create_test_player();
        let result = operate_shrine(&mut player, ShrineType::Weird, 1);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ShrineEffect::StatModified);

        // Verify stats were swapped (seed 1 → swap Magic and Strength)
        assert_eq!(player.get_base_attribute(CharacterAttribute::Strength), 20); // was Magic
        assert_eq!(player.get_base_attribute(CharacterAttribute::Magic), 30); // was Strength
    }

    #[test]
    fn test_operate_shrine_abandoned() {
        let mut player = create_test_player();
        let result = operate_shrine(&mut player, ShrineType::Abandoned, 0);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ShrineEffect::StatModified);
        assert_eq!(player.get_base_attribute(CharacterAttribute::Dexterity), 27); // 25 + 2
    }

    #[test]
    fn test_operate_shrine_creepy() {
        let mut player = create_test_player();
        let result = operate_shrine(&mut player, ShrineType::Creepy, 0);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ShrineEffect::StatModified);
        assert_eq!(player.get_base_attribute(CharacterAttribute::Strength), 32); // 30 + 2
    }

    #[test]
    fn test_operate_shrine_quiet() {
        let mut player = create_test_player();
        let result = operate_shrine(&mut player, ShrineType::Quiet, 0);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ShrineEffect::StatModified);
        assert_eq!(player.get_base_attribute(CharacterAttribute::Vitality), 37); // 35 + 2
    }

    #[test]
    fn test_operate_shrine_shimmering() {
        let mut player = create_test_player();
        let result = operate_shrine(&mut player, ShrineType::Shimmering, 0);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ShrineEffect::StatModified);
        assert_eq!(player.get_base_attribute(CharacterAttribute::Magic), 22); // 20 + 2
    }

    #[test]
    fn test_operate_shrine_divine() {
        let mut player = create_test_player();
        let result = operate_shrine(&mut player, ShrineType::Divine, 0);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ShrineEffect::Healed);

        // Should restore to full HP and Mana
        assert_eq!(player.get_hp(), 200);
        assert_eq!(player.get_mana(), 100);
    }

    #[test]
    fn test_operate_shrine_fascinating() {
        let mut player = create_test_player();
        let result = operate_shrine(&mut player, ShrineType::Fascinating, 0);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ShrineEffect::ManaRestored);

        // Should restore mana to full
        assert_eq!(player.get_mana(), 100);
    }

    #[test]
    fn test_operate_shrine_tainted() {
        let mut player = create_test_player();
        player.modify_resistance("fire", 30);
        player.modify_resistance("lightning", 25);
        player.modify_resistance("magic", 20);

        let result = operate_shrine(&mut player, ShrineType::Tainted, 0);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ShrineEffect::StatModified);

        // Each resistance -1
        assert_eq!(player.get_resistance("fire"), 29);
        assert_eq!(player.get_resistance("lightning"), 24);
        assert_eq!(player.get_resistance("magic"), 19);
    }

    #[test]
    fn test_operate_shrine_oily() {
        let mut player = create_test_player();
        let result = operate_shrine(&mut player, ShrineType::Oily, 0);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ShrineEffect::StatModified);

        // Each resistance +1
        assert_eq!(player.get_resistance("fire"), 1);
        assert_eq!(player.get_resistance("lightning"), 1);
        assert_eq!(player.get_resistance("magic"), 1);
    }

    #[test]
    fn test_operate_shrine_murphys() {
        let mut player = create_test_player();
        let result = operate_shrine(&mut player, ShrineType::Murphys, 0);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ShrineEffect::StatModified);

        // All stats +1
        assert_eq!(player.get_base_attribute(CharacterAttribute::Strength), 31);
        assert_eq!(player.get_base_attribute(CharacterAttribute::Magic), 21);
        assert_eq!(player.get_base_attribute(CharacterAttribute::Dexterity), 26);
        assert_eq!(player.get_base_attribute(CharacterAttribute::Vitality), 36);
    }

    #[test]
    fn test_operate_shrine_sparkling() {
        let mut player = create_test_player();
        let result = operate_shrine(&mut player, ShrineType::Sparkling, 0);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ShrineEffect::StatModified);

        // +1 to one random stat (seed 0 → Strength)
        assert_eq!(player.get_base_attribute(CharacterAttribute::Strength), 31);
    }

    #[test]
    fn test_random_character_attribute() {
        // Test deterministic randomness
        assert_eq!(random_character_attribute(0), CharacterAttribute::Strength);
        assert_eq!(random_character_attribute(1), CharacterAttribute::Magic);
        assert_eq!(random_character_attribute(2), CharacterAttribute::Dexterity);
        assert_eq!(random_character_attribute(3), CharacterAttribute::Vitality);
        assert_eq!(random_character_attribute(4), CharacterAttribute::Strength); // wraps
    }

    #[test]
    fn test_random_two_stats() {
        let (stat1, stat2) = random_two_stats(0);
        assert_ne!(stat1, stat2);

        let (stat3, stat4) = random_two_stats(5);
        assert_ne!(stat3, stat4);
    }

    #[test]
    fn test_random_stat_by_time() {
        // Just verify it returns a valid attribute
        let stat = random_stat_by_time();
        assert!(matches!(
            stat,
            CharacterAttribute::Strength
                | CharacterAttribute::Magic
                | CharacterAttribute::Dexterity
                | CharacterAttribute::Vitality
        ));
    }
}

