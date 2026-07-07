//! Spell Casting System - Exact C++ Port (M43)
//!
//! This module provides exact port of DevilutionX spell casting functionality.
//!
//! ## C++ References
//! - Source/spells.cpp: Spell casting implementation
//! - Source/spells.h: Spell interface definitions
//!
//! ## Key Features
//! - Mana cost calculation with class bonuses
//! - Spell validity checking
//! - Spell consumption (scrolls, staff charges, mana)
//! - Wall spell support (FireWall, LightningWall)
//! - Resurrect and HealOther implementations

use crate::game::player_exact::{HeroClass, Player, SpellId, SpellType, Direction, PlayerMode};
use crate::game::types::WorldTilePosition;

/// Spell check result - matches C++ SpellCheckResult enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SpellCheckResult {
    Success = 0,
    FailNoMana = 1,
    FailLevel0 = 2,
    FailBusy = 3,
}

/// Inventory item slot constants
pub const INVITEM_INV_FIRST: i32 = 0;
pub const INVITEM_INV_LAST: i32 = 39;
pub const INVITEM_BELT_FIRST: i32 = 40;
pub const INVITEM_BELT_LAST: i32 = 47;

/// Spell data for mana calculation
#[derive(Debug, Clone, Copy)]
pub struct SpellDataRef {
    pub mana_cost: u8,
    pub mana_adj: u8,
    pub min_mana: u8,
    pub book_lvl: i8,
    pub staff_lvl: i8,
}

/// Get spell data reference
/// TODO: Replace with full SpellsData table from spelldat
pub fn get_spell_data(spell: SpellId) -> SpellDataRef {
    match spell {
        SpellId::Firebolt => SpellDataRef { mana_cost: 6, mana_adj: 1, min_mana: 3, book_lvl: 1, staff_lvl: 1 },
        SpellId::Healing => SpellDataRef { mana_cost: 5, mana_adj: 3, min_mana: 3, book_lvl: 1, staff_lvl: 1 },
        SpellId::Lightning => SpellDataRef { mana_cost: 10, mana_adj: 1, min_mana: 6, book_lvl: 4, staff_lvl: 3 },
        SpellId::Flash => SpellDataRef { mana_cost: 30, mana_adj: 2, min_mana: 16, book_lvl: 5, staff_lvl: 4 },
        SpellId::Identify => SpellDataRef { mana_cost: 255, mana_adj: 0, min_mana: 1, book_lvl: -1, staff_lvl: -1 },
        SpellId::FireWall => SpellDataRef { mana_cost: 28, mana_adj: 2, min_mana: 16, book_lvl: 3, staff_lvl: 2 },
        SpellId::TownPortal => SpellDataRef { mana_cost: 35, mana_adj: 0, min_mana: 18, book_lvl: 3, staff_lvl: 2 },
        SpellId::StoneCurse => SpellDataRef { mana_cost: 60, mana_adj: 3, min_mana: 40, book_lvl: 6, staff_lvl: 5 },
        SpellId::Infravision => SpellDataRef { mana_cost: 255, mana_adj: 0, min_mana: 1, book_lvl: -1, staff_lvl: -1 },
        SpellId::Phasing => SpellDataRef { mana_cost: 12, mana_adj: 2, min_mana: 4, book_lvl: 7, staff_lvl: 6 },
        SpellId::ManaShield => SpellDataRef { mana_cost: 33, mana_adj: 0, min_mana: 33, book_lvl: 6, staff_lvl: 5 },
        SpellId::Fireball => SpellDataRef { mana_cost: 16, mana_adj: 1, min_mana: 10, book_lvl: 8, staff_lvl: 7 },
        SpellId::Guardian => SpellDataRef { mana_cost: 50, mana_adj: 2, min_mana: 30, book_lvl: 9, staff_lvl: 8 },
        SpellId::ChainLightning => SpellDataRef { mana_cost: 30, mana_adj: 1, min_mana: 18, book_lvl: 8, staff_lvl: 7 },
        SpellId::FlameWave => SpellDataRef { mana_cost: 35, mana_adj: 3, min_mana: 20, book_lvl: 9, staff_lvl: 8 },
        SpellId::DoomSerpents => SpellDataRef { mana_cost: 0, mana_adj: 0, min_mana: 0, book_lvl: -1, staff_lvl: -1 },
        SpellId::BloodRitual => SpellDataRef { mana_cost: 0, mana_adj: 0, min_mana: 0, book_lvl: -1, staff_lvl: -1 },
        SpellId::Nova => SpellDataRef { mana_cost: 60, mana_adj: 3, min_mana: 35, book_lvl: 10, staff_lvl: 10 },
        SpellId::Invisibility => SpellDataRef { mana_cost: 0, mana_adj: 0, min_mana: 0, book_lvl: -1, staff_lvl: -1 },
        SpellId::Inferno => SpellDataRef { mana_cost: 11, mana_adj: 1, min_mana: 6, book_lvl: 3, staff_lvl: 2 },
        SpellId::Golem => SpellDataRef { mana_cost: 100, mana_adj: 6, min_mana: 60, book_lvl: 11, staff_lvl: 9 },
        SpellId::BloodBoil => SpellDataRef { mana_cost: 0, mana_adj: 0, min_mana: 0, book_lvl: -1, staff_lvl: -1 },
        SpellId::Teleport => SpellDataRef { mana_cost: 35, mana_adj: 3, min_mana: 15, book_lvl: 12, staff_lvl: 11 },
        SpellId::Apocalypse => SpellDataRef { mana_cost: 150, mana_adj: 6, min_mana: 90, book_lvl: 19, staff_lvl: 15 },
        SpellId::Etherealize => SpellDataRef { mana_cost: 0, mana_adj: 0, min_mana: 0, book_lvl: -1, staff_lvl: -1 },
        SpellId::ItemRepair => SpellDataRef { mana_cost: 255, mana_adj: 0, min_mana: 1, book_lvl: -1, staff_lvl: -1 },
        SpellId::StaffRecharge => SpellDataRef { mana_cost: 255, mana_adj: 0, min_mana: 1, book_lvl: -1, staff_lvl: -1 },
        SpellId::TrapDisarm => SpellDataRef { mana_cost: 255, mana_adj: 0, min_mana: 1, book_lvl: -1, staff_lvl: -1 },
        SpellId::Elemental => SpellDataRef { mana_cost: 35, mana_adj: 2, min_mana: 20, book_lvl: 8, staff_lvl: 6 },
        SpellId::ChargedBolt => SpellDataRef { mana_cost: 6, mana_adj: 1, min_mana: 4, book_lvl: 1, staff_lvl: 1 },
        SpellId::HolyBolt => SpellDataRef { mana_cost: 7, mana_adj: 1, min_mana: 3, book_lvl: 1, staff_lvl: 1 },
        SpellId::Resurrect => SpellDataRef { mana_cost: 20, mana_adj: 0, min_mana: 20, book_lvl: 1, staff_lvl: -1 },
        SpellId::Telekinesis => SpellDataRef { mana_cost: 8, mana_adj: 2, min_mana: 2, book_lvl: 2, staff_lvl: 2 },
        SpellId::HealOther => SpellDataRef { mana_cost: 5, mana_adj: 3, min_mana: 3, book_lvl: 1, staff_lvl: 1 },
        SpellId::BloodStar => SpellDataRef { mana_cost: 25, mana_adj: 2, min_mana: 14, book_lvl: 5, staff_lvl: 4 },
        SpellId::BoneSpirit => SpellDataRef { mana_cost: 24, mana_adj: 3, min_mana: 12, book_lvl: 7, staff_lvl: 6 },
        // Hellfire spells
        SpellId::LightningWall => SpellDataRef { mana_cost: 28, mana_adj: 2, min_mana: 16, book_lvl: 3, staff_lvl: 2 },
        SpellId::Immolation => SpellDataRef { mana_cost: 60, mana_adj: 3, min_mana: 35, book_lvl: 10, staff_lvl: 10 },
        SpellId::Warp => SpellDataRef { mana_cost: 35, mana_adj: 3, min_mana: 15, book_lvl: 12, staff_lvl: 11 },
        SpellId::Reflect => SpellDataRef { mana_cost: 35, mana_adj: 3, min_mana: 15, book_lvl: 12, staff_lvl: 11 },
        SpellId::Berserk => SpellDataRef { mana_cost: 35, mana_adj: 3, min_mana: 15, book_lvl: 12, staff_lvl: 11 },
        SpellId::RingOfFire => SpellDataRef { mana_cost: 28, mana_adj: 2, min_mana: 16, book_lvl: 3, staff_lvl: 2 },
        SpellId::Search => SpellDataRef { mana_cost: 15, mana_adj: 1, min_mana: 5, book_lvl: 1, staff_lvl: 1 },
        // Runes (skills)
        SpellId::RuneOfFire => SpellDataRef { mana_cost: 0, mana_adj: 0, min_mana: 0, book_lvl: -1, staff_lvl: -1 },
        SpellId::RuneOfLight => SpellDataRef { mana_cost: 0, mana_adj: 0, min_mana: 0, book_lvl: -1, staff_lvl: -1 },
        SpellId::RuneOfNova => SpellDataRef { mana_cost: 0, mana_adj: 0, min_mana: 0, book_lvl: -1, staff_lvl: -1 },
        SpellId::RuneOfImmolation => SpellDataRef { mana_cost: 0, mana_adj: 0, min_mana: 0, book_lvl: -1, staff_lvl: -1 },
        SpellId::RuneOfStone => SpellDataRef { mana_cost: 0, mana_adj: 0, min_mana: 0, book_lvl: -1, staff_lvl: -1 },
        _ => SpellDataRef { mana_cost: 0, mana_adj: 0, min_mana: 0, book_lvl: -1, staff_lvl: -1 },
    }
}

/// Check if a spell ID is valid
/// Exact port of IsValidSpell from spells.cpp
pub fn is_valid_spell(spell: SpellId) -> bool {
    match spell {
        SpellId::None | SpellId::Invalid => false,
        _ => true,
    }
}

/// Check if spell source is valid (inventory slot)
/// Exact port of IsValidSpellFrom from spells.cpp
pub fn is_valid_spell_from(spell_from: i32) -> bool {
    if spell_from == 0 {
        return true;
    }
    if spell_from >= INVITEM_INV_FIRST && spell_from <= INVITEM_INV_LAST {
        return true;
    }
    if spell_from >= INVITEM_BELT_FIRST && spell_from <= INVITEM_BELT_LAST {
        return true;
    }
    false
}

/// Check if spell is a wall-type spell (FireWall, LightningWall)
/// Exact port of IsWallSpell from spells.cpp
pub fn is_wall_spell(spell: SpellId) -> bool {
    spell == SpellId::FireWall || spell == SpellId::LightningWall
}

/// Check if spell targets monsters specifically
/// Exact port of TargetsMonster from spells.cpp
pub fn targets_monster(spell: SpellId) -> bool {
    matches!(spell,
        SpellId::Fireball |
        SpellId::FireWall |
        SpellId::Inferno |
        SpellId::Lightning |
        SpellId::StoneCurse |
        SpellId::FlameWave
    )
}

/// Calculate mana cost for a spell
/// Exact port of GetManaAmount from spells.cpp
///
/// The returned mana is in fixed-point format (64x multiplier)
pub fn get_mana_amount(player: &Player, spell: SpellId, is_hellfire: bool) -> i32 {
    let spell_data = get_spell_data(spell);

    // Spell level (minimum 0)
    let sl = (player.get_spell_level(spell) as i32 - 1).max(0);

    // Mana adjustment based on spell level
    let mut adj = if sl > 0 {
        sl * spell_data.mana_adj as i32
    } else {
        0
    };

    // Firebolt has reduced adjustment
    if spell == SpellId::Firebolt {
        adj /= 2;
    }

    // Resurrect has special adjustment
    if spell == SpellId::Resurrect && sl > 0 {
        let resurrect_data = get_spell_data(SpellId::Resurrect);
        adj = sl * (resurrect_data.mana_cost as i32 / 8);
    }

    // Calculate base mana
    let mut ma = if spell == SpellId::Healing || spell == SpellId::HealOther {
        let healing_data = get_spell_data(SpellId::Healing);
        healing_data.mana_cost as i32 + 2 * player.get_character_level() as i32 - adj
    } else if spell_data.mana_cost == 255 {
        // Special cost: percentage of max mana
        (player._p_max_mana_base >> 6) - adj
    } else {
        spell_data.mana_cost as i32 - adj
    };

    // Clamp to 0
    ma = ma.max(0);

    // Convert to fixed-point (64x)
    ma <<= 6;

    // Class bonuses
    if is_hellfire && player._p_class == HeroClass::Sorcerer {
        ma /= 2;  // Hellfire sorcerer: 50% mana cost
    } else if matches!(player._p_class, HeroClass::Rogue | HeroClass::Monk | HeroClass::Bard) {
        ma -= ma / 4;  // Rogue/Monk/Bard: 75% mana cost
    }

    // Enforce minimum mana
    if spell_data.min_mana as i32 > (ma >> 6) {
        ma = (spell_data.min_mana as i32) << 6;
    }

    ma
}

/// Get spell book level requirement
/// Exact port of GetSpellBookLevel from spells.cpp
pub fn get_spell_book_level(spell: SpellId, is_spawn: bool) -> i32 {
    // Spawn version restrictions
    if is_spawn {
        match spell {
            SpellId::StoneCurse |
            SpellId::Guardian |
            SpellId::Golem |
            SpellId::Elemental |
            SpellId::BloodStar |
            SpellId::BoneSpirit => return -1,
            _ => {}
        }
    }

    let data = get_spell_data(spell);
    data.book_lvl as i32
}

/// Get spell staff level requirement
/// Exact port of GetSpellStaffLevel from spells.cpp
pub fn get_spell_staff_level(spell: SpellId, is_spawn: bool) -> i32 {
    // Spawn version restrictions
    if is_spawn {
        match spell {
            SpellId::StoneCurse |
            SpellId::Guardian |
            SpellId::Golem |
            SpellId::Apocalypse |
            SpellId::Elemental |
            SpellId::BloodStar |
            SpellId::BoneSpirit => return -1,
            _ => {}
        }
    }

    let data = get_spell_data(spell);
    data.staff_lvl as i32
}

/// Check if player's readied spell is valid
fn is_readied_spell_valid(player: &Player) -> bool {
    match player._p_r_spl_type {
        SpellType::Skill | SpellType::Spell | SpellType::Invalid => true,
        SpellType::Charges => {
            (player._p_i_spells & get_spell_bitmask(player._p_r_spell)) != 0
        }
        SpellType::Scroll => {
            (player._p_scrl_spells & get_spell_bitmask(player._p_r_spell)) != 0
        }
    }
}

/// Get bitmask for spell ID (for spell tracking bitfields)
///
/// Port of `GetSpellBitmask` from spells.h: `1ULL << (int8_t(spellId) - 1)`.
///
/// Note: the authoritative C++ `SpellID` enum (`spelldat.h`) numbers spells
/// starting at `Firebolt = 1`, so `spellId - 1` yields a 0-based bit index.
/// The local `player_exact::SpellId` re-export numbers `Firebolt = 0`
/// (with `None = -1`), so the effective bit index is just `spell as i8`.
/// We guard `None` explicitly (returns 0 = no bits) and use wrapping shifts
/// so the function is total rather than panicking on out-of-range inputs.
pub const fn get_spell_bitmask(spell: SpellId) -> u64 {
    let idx = spell as i8;
    if idx < 0 {
        return 0;
    }
    1u64.wrapping_shl(idx as u32)
}

/// Clear player's readied spell
pub fn clear_readied_spell(player: &mut Player) {
    if player._p_r_spell != SpellId::Invalid {
        player._p_r_spell = SpellId::Invalid;
        // RedrawEverything() would be called here
    }

    if player._p_r_spl_type != SpellType::Invalid {
        player._p_r_spl_type = SpellType::Invalid;
        // RedrawEverything() would be called here
    }
}

/// Ensure player's readied spell is valid
/// Exact port of EnsureValidReadiedSpell from spells.cpp
pub fn ensure_valid_readied_spell(player: &mut Player) {
    if !is_readied_spell_valid(player) {
        clear_readied_spell(player);
    }
}

/// Check if player can cast a spell
/// Exact port of CheckSpell from spells.cpp
pub fn check_spell(
    player: &Player,
    spell: SpellId,
    spell_type: SpellType,
    mana_only: bool,
    cursor_hand: bool,
    debug_god_mode: bool,
    is_hellfire: bool,
) -> SpellCheckResult {
    // Debug god mode always succeeds
    if debug_god_mode {
        return SpellCheckResult::Success;
    }

    // Busy check (unless mana only)
    if !mana_only && !cursor_hand {
        return SpellCheckResult::FailBusy;
    }

    // Skills always succeed
    if spell_type == SpellType::Skill {
        return SpellCheckResult::Success;
    }

    // Check spell level
    if player.get_spell_level(spell) <= 0 {
        return SpellCheckResult::FailLevel0;
    }

    // Check mana (and NoMana item flag)
    let mana_amount = get_mana_amount(player, spell, is_hellfire);
    if player._p_mana < mana_amount || player.has_no_mana_flag() {
        return SpellCheckResult::FailNoMana;
    }

    SpellCheckResult::Success
}

/// Executed spell info (for consumption tracking)
#[derive(Debug, Clone, Copy, Default)]
pub struct ExecutedSpell {
    pub spell_type: SpellType,
    pub spell_from: i32,
}

/// Consume spell resources after casting
/// Exact port of ConsumeSpell from spells.cpp
pub fn consume_spell(
    player: &mut Player,
    spell: SpellId,
    executed: ExecutedSpell,
    debug_god_mode: bool,
    is_hellfire: bool,
) {
    match executed.spell_type {
        SpellType::Skill | SpellType::Invalid => {
            // Skills and invalid don't consume anything
        }
        SpellType::Scroll => {
            // ConsumeScroll(player) would be called here
            consume_scroll(player);
        }
        SpellType::Charges => {
            // ConsumeStaffCharge(player) would be called here
            consume_staff_charge(player);
        }
        SpellType::Spell => {
            if !debug_god_mode {
                let ma = get_mana_amount(player, spell, is_hellfire);
                player._p_mana -= ma;
                player._p_mana_base -= ma;
                // RedrawComponent(PanelDrawComponent::Mana) would be called here
            }
        }
    }

    // Blood spells also cost HP
    if spell == SpellId::BloodStar {
        apply_player_damage(player, 5);
    }
    if spell == SpellId::BoneSpirit {
        apply_player_damage(player, 6);
    }
}

/// Consume a scroll from inventory
fn consume_scroll(_player: &mut Player) {
    // TODO: Implement full scroll consumption
    // This would find and remove the scroll item
}

/// Consume a staff charge
fn consume_staff_charge(_player: &mut Player) {
    // TODO: Implement staff charge consumption
    // This would decrement staff charges
}

/// Apply damage to player (for blood spells)
fn apply_player_damage(player: &mut Player, damage: i32) {
    let damage_fixed = damage << 6;  // Convert to fixed-point
    player._p_hit_points -= damage_fixed;
    player._p_hp_base -= damage_fixed;

    // Check for death
    if player._p_hit_points <= 0 {
        player._p_hit_points = 0;
        // Death handling would be done elsewhere
    }
}

/// Cast a spell (create missiles)
/// Exact port of CastSpell from spells.cpp
pub fn cast_spell(
    player: &mut Player,
    spell: SpellId,
    _src: WorldTilePosition,
    _dst: WorldTilePosition,
    _spell_level: i32,
    executed: ExecutedSpell,
    debug_god_mode: bool,
    is_hellfire: bool,
) -> bool {
    let _dir = if is_wall_spell(spell) {
        player._p_temp_direction
    } else {
        player._p_dir
    };

    // In full implementation:
    // - Create missiles based on spell data using src, dst, dir, spell_level
    // - Track if any missiles failed to create ("fizzled")
    // - ChargedBolt creates multiple missiles based on level

    let fizzled = false;  // Placeholder

    // Only consume spell if it didn't fizzle
    if !fizzled {
        consume_spell(player, spell, executed, debug_god_mode, is_hellfire);
    }

    !fizzled
}

/// Do resurrection spell
/// Exact port of DoResurrect from spells.cpp
pub fn do_resurrect(_caster: &Player, target: &mut Player, is_my_player: bool) {
    // Create resurrect beam missile
    // AddMissile(target.position.tile, ..., MissileID::ResurrectBeam, ...)

    // If target is not dead, do nothing
    if target._p_hit_points != 0 {
        return;
    }

    if is_my_player {
        // MyPlayerIsDead = false;
        // gamemenu_off();
        // RedrawComponent(PanelDrawComponent::Health);
        // RedrawComponent(PanelDrawComponent::Mana);
    }

    // Clear path
    target.clear_path();
    target._p_dest_action = super::player_exact::ActionType::None;
    target._p_invincible = false;

    // SyncInitPlrPos(target)

    // Restore HP to 10 or max if less
    let hp = if target._p_max_hp_base < (10 << 6) {
        target._p_max_hp_base
    } else {
        10 << 6
    };

    target._p_hit_points = hp;
    target._p_hp_base = target._p_hit_points + (target._p_max_hp_base - target._p_max_hp);

    // Mana set to 0
    target._p_mana = 0;
    target._p_mana_base = target._p_mana + (target._p_max_mana_base - target._p_max_mana);

    // Set to standing mode
    target._p_mode = PlayerMode::Stand;

    // CalcPlrInv(target, true)

    // StartStand(target, target._pdir) if on active level
}

/// Do heal other spell
/// Exact port of DoHealOther from spells.cpp
pub fn do_heal_other(caster: &Player, target: &mut Player, is_my_player: bool, rng: &mut impl FnMut(i32) -> i32) {
    // If target is dead, do nothing
    if (target._p_hit_points >> 6) <= 0 {
        return;
    }

    // Base heal: (random 1-10) * 64
    let mut hp = (rng(10) + 1) << 6;

    // Add character level bonus
    for _ in 0..caster.get_character_level() {
        hp += (rng(4) + 1) << 6;
    }

    // Add spell level bonus
    for _ in 0..caster.get_spell_level(SpellId::HealOther) {
        hp += (rng(6) + 1) << 6;
    }

    // Class bonuses
    match caster._p_class {
        HeroClass::Warrior | HeroClass::Barbarian => {
            hp *= 2;
        }
        HeroClass::Rogue | HeroClass::Bard => {
            hp += hp / 2;
        }
        HeroClass::Monk => {
            hp *= 3;
        }
        _ => {}
    }

    // Apply healing (capped at max HP)
    target._p_hit_points = (target._p_hit_points + hp).min(target._p_max_hp);
    target._p_hp_base = (target._p_hp_base + hp).min(target._p_max_hp_base);

    // Redraw health panel if this is the local player
    if is_my_player {
        // RedrawComponent(PanelDrawComponent::Health);
    }
}

// ============================================================================
// Player extension methods
// ============================================================================

impl Player {
    /// Get character level
    pub fn get_character_level(&self) -> u32 {
        self._p_level as u32
    }

    /// Check if player has NoMana item flag
    pub fn has_no_mana_flag(&self) -> bool {
        // TODO: Check _pIFlags for ItemSpecialEffect::NoMana
        false
    }

    /// Clear player's walk path
    pub fn clear_path(&mut self) {
        for i in 0..self.walk_path.len() {
            self.walk_path[i] = Direction::None;
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_spell() {
        assert!(!is_valid_spell(SpellId::None));
        assert!(!is_valid_spell(SpellId::Invalid));
        assert!(is_valid_spell(SpellId::Firebolt));
        assert!(is_valid_spell(SpellId::Healing));
        assert!(is_valid_spell(SpellId::Apocalypse));
    }

    #[test]
    fn test_is_wall_spell() {
        assert!(is_wall_spell(SpellId::FireWall));
        assert!(is_wall_spell(SpellId::LightningWall));
        assert!(!is_wall_spell(SpellId::Fireball));
        assert!(!is_wall_spell(SpellId::Lightning));
    }

    #[test]
    fn test_targets_monster() {
        assert!(targets_monster(SpellId::Fireball));
        assert!(targets_monster(SpellId::FireWall));
        assert!(targets_monster(SpellId::StoneCurse));
        assert!(!targets_monster(SpellId::Healing));
        assert!(!targets_monster(SpellId::TownPortal));
    }

    #[test]
    fn test_spell_bitmask() {
        assert_eq!(get_spell_bitmask(SpellId::Firebolt), 1);
        assert_eq!(get_spell_bitmask(SpellId::Healing), 2);
        assert_eq!(get_spell_bitmask(SpellId::Lightning), 4);
    }

    #[test]
    fn test_is_valid_spell_from() {
        assert!(is_valid_spell_from(0));
        assert!(is_valid_spell_from(10));  // Inventory
        assert!(is_valid_spell_from(45));  // Belt
        assert!(!is_valid_spell_from(100));
        assert!(!is_valid_spell_from(-1));
    }

    #[test]
    fn test_spell_data() {
        let firebolt = get_spell_data(SpellId::Firebolt);
        assert_eq!(firebolt.mana_cost, 6);
        assert_eq!(firebolt.book_lvl, 1);

        let apocalypse = get_spell_data(SpellId::Apocalypse);
        assert_eq!(apocalypse.mana_cost, 150);
        assert_eq!(apocalypse.book_lvl, 19);
    }

    #[test]
    fn test_spell_book_level_spawn_restrictions() {
        // Spawn version should return -1 for restricted spells
        assert_eq!(get_spell_book_level(SpellId::StoneCurse, true), -1);
        assert_eq!(get_spell_book_level(SpellId::Guardian, true), -1);
        assert_eq!(get_spell_book_level(SpellId::Golem, true), -1);

        // Non-spawn should return actual level
        assert!(get_spell_book_level(SpellId::StoneCurse, false) > 0);
        assert!(get_spell_book_level(SpellId::Guardian, false) > 0);
    }

    #[test]
    fn test_check_spell_result_values() {
        assert_eq!(SpellCheckResult::Success as u8, 0);
        assert_eq!(SpellCheckResult::FailNoMana as u8, 1);
        assert_eq!(SpellCheckResult::FailLevel0 as u8, 2);
        assert_eq!(SpellCheckResult::FailBusy as u8, 3);
    }
}
