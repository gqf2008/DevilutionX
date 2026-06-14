//! Diablo save-game packed-hero decoder.
//!
//! Port of the C++ `PlayerPack` struct and `UnPackPlayer` logic from
//! `Source/pack.h` / `Source/pack.cpp`. The packed player is the binary blob
//! stored inside a `.sv` MPQ archive (a hero save). This module decodes the
//! player's core fields; full inventory / item recreation is left as a TODO
//! (it requires the global item-data tables).
//!
//! ## Byte layout
//!
//! The C++ struct uses `#pragma pack(push, 1)` (no padding). We mirror it with
//! `#[repr(C, packed)]` and read every field through `read_unaligned` to avoid
//! creating unaligned references (which are UB / panic on packed structs).

use core::mem;

// ---------------------------------------------------------------------------
// Safe packed-struct field readers.
//
// For `#[repr(C, packed)]` structs, forming `&self.field` is itself an
// unaligned reference (UB), even when handed to `read_unaligned`. We instead
// derive a raw field pointer via `addr_of!` and read through it. These helpers
// take a base reference and a byte offset, so the call sites never name a
// field by reference.
// ---------------------------------------------------------------------------

#[inline]
fn read_u16<T>(base: &T, off: usize) -> u16 {
    assert!(off + 2 <= mem::size_of::<T>());
    unsafe {
        let p = (base as *const T as *const u8).add(off) as *const u16;
        ptr::read_unaligned(p)
    }
}

#[inline]
fn read_u32<T>(base: &T, off: usize) -> u32 {
    assert!(off + 4 <= mem::size_of::<T>());
    unsafe {
        let p = (base as *const T as *const u8).add(off) as *const u32;
        ptr::read_unaligned(p)
    }
}

#[inline]
fn read_i32<T>(base: &T, off: usize) -> i32 {
    assert!(off + 4 <= mem::size_of::<T>());
    unsafe {
        let p = (base as *const T as *const u8).add(off) as *const i32;
        ptr::read_unaligned(p)
    }
}

#[inline]
fn read_u64<T>(base: &T, off: usize) -> u64 {
    assert!(off + 8 <= mem::size_of::<T>());
    unsafe {
        let p = (base as *const T as *const u8).add(off) as *const u64;
        ptr::read_unaligned(p)
    }
}


// ---------------------------------------------------------------------------
// Layout constants — mirror the C++ `#define`s / `constexpr`s.
//   Source/player.h:
//     PlayerNameLength   = 32
//     NUM_INVLOC         = 7   (inv_body_loc enum)
//     InventoryGridCells = 40
//     MaxBeltItems       = 8
//   Source/pack.h:
//     ItemPack is 19 bytes  (4+2+2+1+1+1+1+1+2+4)
// ---------------------------------------------------------------------------

pub const PLAYER_NAME_LENGTH: usize = 32;
pub const NUM_INVLOC: usize = 7;
pub const INVENTORY_GRID_CELLS: usize = 40;
pub const MAX_BELT_ITEMS: usize = 8;
pub const DIABLO_SPELL_SLOTS: usize = 37; // pSplLvl[37]
pub const HELLFIRE_SPELL_SLOTS: usize = 10; // pSplLvl2[10]

/// `sizeof(ItemPack)` in bytes.
pub const ITEM_PACK_SIZE: usize = 19;
/// `sizeof(PlayerPack)` in bytes, computed from the field layout above.
pub const PLAYER_PACK_SIZE: usize = 1266;

// ---------------------------------------------------------------------------
// ItemPack — exact mirror of `Source/pack.h::ItemPack` (19 bytes, packed).
// ---------------------------------------------------------------------------

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct ItemPack {
    pub i_seed: u32,
    pub i_create_info: u16,
    pub idx: u16,
    pub b_id: u8,
    pub b_dur: u8,
    pub b_m_dur: u8,
    pub b_ch: u8,
    pub b_m_ch: u8,
    pub w_value: u16,
    pub dw_buff: u32,
}

impl ItemPack {
    /// `true` when this slot is empty (mirrors `idx == 0xFFFF` in `UnPackItem`).
    pub fn is_empty(&self) -> bool {
        self.idx() == 0xFFFF
    }
    pub fn i_seed(&self) -> u32 {
        u32::from_le(read_u32(self, 0))
    }
    pub fn i_create_info(&self) -> u16 {
        u16::from_le(read_u16(self, 4))
    }
    pub fn idx(&self) -> u16 {
        u16::from_le(read_u16(self, 6))
    }
    pub fn w_value(&self) -> u16 {
        u16::from_le(read_u16(self, 13))
    }
    pub fn dw_buff(&self) -> u32 {
        u32::from_le(read_u32(self, 15))
    }
}

// ---------------------------------------------------------------------------
// PlayerPack — exact mirror of `Source/pack.h::PlayerPack` (1266 bytes, packed).
// ---------------------------------------------------------------------------

#[repr(C, packed)]
pub struct PlayerPack {
    pub dw_low_date_time: u32,
    pub dw_high_date_time: u32,
    pub dest_action: i8,
    pub dest_param1: i8,
    pub dest_param2: i8,
    pub plrlevel: u8,
    pub px: u8,
    pub py: u8,
    pub targx: u8,
    pub targy: u8,
    pub p_name: [u8; PLAYER_NAME_LENGTH],
    pub p_class: u8,
    pub p_base_str: u8,
    pub p_base_mag: u8,
    pub p_base_dex: u8,
    pub p_base_vit: u8,
    pub p_level: u8,
    pub p_stat_pts: u8,
    pub p_experience: u32,
    pub p_gold: i32,
    pub p_hp_base: i32,
    pub p_max_hp_base: i32,
    pub p_mana_base: i32,
    pub p_max_mana_base: i32,
    pub p_spl_lvl: [u8; DIABLO_SPELL_SLOTS],
    pub p_mem_spells: u64,
    pub inv_body: [ItemPack; NUM_INVLOC],
    pub inv_list: [ItemPack; INVENTORY_GRID_CELLS],
    pub inv_grid: [i8; INVENTORY_GRID_CELLS],
    pub _p_num_inv: u8,
    pub spd_list: [ItemPack; MAX_BELT_ITEMS],
    pub p_town_warps: i8,
    pub p_dung_msgs: i8,
    pub p_lvl_load: i8,
    pub p_battle_net: u8,
    pub p_mana_shield: u8,
    pub p_dung_msgs2: u8,
    /// `int8_t bIsHellfire`: 0 = Diablo format, non-zero = Hellfire.
    pub b_is_hellfire: i8,
    pub reserved: u8,
    pub w_reflections: u16,
    pub reserved2: [u8; 2],
    /// Hellfire-only spell levels (`pSplLvl2[10]`).
    pub p_spl_lvl2: [u8; HELLFIRE_SPELL_SLOTS],
    pub w_reserved8: i16,
    pub p_diablo_kill_level: u32,
    pub p_difficulty: u32,
    pub p_dam_ac_flags: u32,
    pub reserved3: [u8; 20],
}

// ---------------------------------------------------------------------------
// HeroClass — mirror of `Source/playerdat.hpp::enum class HeroClass`.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum HeroClass {
    Warrior = 0,
    Rogue = 1,
    Sorcerer = 2,
    Monk = 3,
    Bard = 4,
    Barbarian = 5,
}

impl HeroClass {
    /// Number of canonical player classes (mirrors `GetNumPlayerClasses()`).
    pub const COUNT: u8 = 6;

    /// Parse from the raw packed byte, clamping out-of-range values the same way
    /// `UnPackPlayer` does (`std::clamp(pClass, 0, COUNT-1)`).
    pub fn from_packed(raw: u8) -> HeroClass {
        let clamped = raw.min(HeroClass::COUNT - 1);
        match clamped {
            0 => HeroClass::Warrior,
            1 => HeroClass::Rogue,
            2 => HeroClass::Sorcerer,
            3 => HeroClass::Monk,
            4 => HeroClass::Bard,
            _ => HeroClass::Barbarian,
        }
    }
}

// ---------------------------------------------------------------------------
// DecodedPlayer — friendly Rust view of the fields decoded by `UnPackPlayer`.
// ---------------------------------------------------------------------------

/// A packed inventory item slot, decoded into a plain struct.
#[derive(Debug, Clone)]
pub struct DecodedItem {
    pub i_seed: u32,
    pub i_create_info: u16,
    pub idx: u16,
    pub b_id: u8,
    pub b_dur: u8,
    pub b_m_dur: u8,
    pub b_ch: u8,
    pub b_m_ch: u8,
    pub w_value: u16,
    pub dw_buff: u32,
    /// `true` for an empty slot (`idx == 0xFFFF`).
    pub empty: bool,
}

impl DecodedItem {
    /// Decode an `ItemPack` from its raw 19-byte representation starting at
    /// `bytes[0..]`. Caller must guarantee at least `ITEM_PACK_SIZE` bytes.
    fn from_bytes(bytes: &[u8]) -> DecodedItem {
        let i_seed = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let i_create_info = u16::from_le_bytes([bytes[4], bytes[5]]);
        let idx = u16::from_le_bytes([bytes[6], bytes[7]]);
        let w_value = u16::from_le_bytes([bytes[13], bytes[14]]);
        let dw_buff = u32::from_le_bytes([bytes[15], bytes[16], bytes[17], bytes[18]]);
        DecodedItem {
            i_seed,
            i_create_info,
            idx,
            b_id: bytes[8],
            b_dur: bytes[9],
            b_m_dur: bytes[10],
            b_ch: bytes[11],
            b_m_ch: bytes[12],
            w_value,
            dw_buff,
            empty: idx == 0xFFFF,
        }
    }

    #[allow(dead_code)]
    fn from_packed(p: &ItemPack) -> DecodedItem {
        DecodedItem {
            i_seed: p.i_seed(),
            i_create_info: p.i_create_info(),
            idx: p.idx(),
            b_id: p.b_id,
            b_dur: p.b_dur,
            b_m_dur: p.b_m_dur,
            b_ch: p.b_ch,
            b_m_ch: p.b_m_ch,
            w_value: p.w_value(),
            dw_buff: p.dw_buff(),
            empty: p.is_empty(),
        }
    }
}

/// Decode `count` packed items from `bytes`, starting at byte offset `start`,
/// each `ITEM_PACK_SIZE` bytes apart.
fn decode_item_array(bytes: &[u8], start: usize, count: usize) -> Vec<DecodedItem> {
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let off = start + i * ITEM_PACK_SIZE;
        out.push(DecodedItem::from_bytes(&bytes[off..off + ITEM_PACK_SIZE]));
    }
    out
}

#[derive(Debug, Clone)]
pub struct DecodedPlayer {
    /// Player class (clamped to a valid `HeroClass`).
    pub class: HeroClass,
    /// Character level (`pLevel`).
    pub level: u8,
    /// Hero name as a UTF-8 string (NUL-padded in the save; trailing NULs trimmed).
    pub name: String,
    /// Dungeon level index (0..NUMLEVELS).
    pub plrlevel: u8,
    /// Tile position.
    pub px: u8,
    pub py: u8,

    // Base attributes.
    pub base_str: u8,
    pub base_mag: u8,
    pub base_dex: u8,
    pub base_vit: u8,
    pub stat_pts: u8,

    // Progression / economy.
    pub experience: u32,
    pub gold: i32,

    // HP / mana (raw packed values; `UnPackPlayer` clamps HP to [0, max] and to
    // a minimum of 64; we expose the raw values plus a `hp_clamped` helper).
    pub hp_base: i32,
    pub max_hp_base: i32,
    pub mana_base: i32,
    pub max_mana_base: i32,

    // Spells.
    /// Diablo spell levels (`pSplLvl[37]`).
    pub spl_lvl: Vec<u8>,
    /// Hellfire spell levels (`pSplLvl2[10]`).
    pub spl_lvl2: Vec<u8>,
    /// Bitmask of memorised spells (`pMemSpells`, little-endian).
    pub mem_spells: u64,

    // Inventory.
    pub inv_body: Vec<DecodedItem>,
    pub inv_list: Vec<DecodedItem>,
    /// Inventory grid (raw `InvGrid[40]`).
    pub inv_grid: Vec<i8>,
    pub num_inv: u8,
    pub spd_list: Vec<DecodedItem>,

    // Misc flags.
    pub is_hellfire: bool,
    pub town_warps: i8,
    pub dung_msgs: i8,
    pub lvl_load: i8,
    pub battle_net: bool,
    pub mana_shield: bool,
    pub dung_msgs2: u8,
    pub reflections: u16,
    pub diablo_kill_level: u32,
    pub difficulty: u32,
    pub dam_ac_flags: u32,

    /// Full raw bytes of the source blob, kept for diagnostics / future fields.
    pub raw: Vec<u8>,
}

impl DecodedPlayer {
    /// Effective current HP after `UnPackPlayer`'s clamp: `clamp(hp_base, 0, max_hp_base)`
    /// then floor to 64 if `hp_base & 0xFFFFFFC0 < 64` (matches C++).
    pub fn effective_hp(&self) -> i32 {
        let clamped = self.hp_base.clamp(0, self.max_hp_base);
        if (clamped & 0xFFFF_FFC0u32 as i32) < 64 {
            64
        } else {
            clamped
        }
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// The input buffer is shorter than `PLAYER_PACK_SIZE`.
    BufferTooShort { have: usize, need: usize },
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DecodeError::BufferTooShort { have, need } => write!(
                f,
                "save_decode: buffer too short: have {} bytes, need {}",
                have, need
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Decoder
// ---------------------------------------------------------------------------

/// Decode a packed hero (`PlayerPack`) blob into engine-facing fields.
///
/// `bytes` is the raw packed player blob (e.g. the `hero` file inside an `.sv`
/// MPQ archive). Multi-byte integers are little-endian, mirroring the C++
/// `Swap*LE` helpers in `UnPackPlayer`.
pub fn unpack_player(bytes: &[u8]) -> Result<DecodedPlayer, DecodeError> {
    if bytes.len() < PLAYER_PACK_SIZE {
        return Err(DecodeError::BufferTooShort {
            have: bytes.len(),
            need: PLAYER_PACK_SIZE,
        });
    }

    // SAFETY: PlayerPack is #[repr(C, packed)] and exactly PLAYER_PACK_SIZE
    // bytes. `bytes` has at least that many bytes and the cast reads them as
    // the packed struct. We never form a reference to the packed struct's
    // interior — every multi-byte field access goes through the byte-offset
    // helpers above, and u8/i8 fields are copied directly (1-byte aligned, so
    // a reference is fine).
    let packed: &PlayerPack =
        unsafe { &*(bytes.as_ptr() as *const PlayerPack) };

    // --- scalar fields (multi-byte ints via byte-offset readers) ---
    let p_class_raw = packed.p_class; // u8: 1-byte aligned, copy is fine
    let p_level = packed.p_level;
    let plrlevel = packed.plrlevel;
    let px = packed.px;
    let py = packed.py;

    let base_str = packed.p_base_str;
    let base_mag = packed.p_base_mag;
    let base_dex = packed.p_base_dex;
    let base_vit = packed.p_base_vit;
    let stat_pts = packed.p_stat_pts;

    let experience = u32::from_le(read_u32(packed, 55));
    let gold = i32::from_le(read_i32(packed, 59));
    let hp_base = i32::from_le(read_i32(packed, 63));
    let max_hp_base = i32::from_le(read_i32(packed, 67));
    let mana_base = i32::from_le(read_i32(packed, 71));
    let max_mana_base = i32::from_le(read_i32(packed, 75));
    let mem_spells = u64::from_le(read_u64(packed, 116));

    let reflections = u16::from_le(read_u16(packed, 1218));
    let diablo_kill_level = u32::from_le(read_u32(packed, 1234));
    let difficulty = u32::from_le(read_u32(packed, 1238));
    let dam_ac_flags = u32::from_le(read_u32(packed, 1242));

    // --- name: C++ does CopyUtf8(dest, src, sizeof); trim trailing NULs ---
    // p_name is [u8; 32] at offset 16; u8 is 1-byte aligned so a slice read
    // is fine, but we copy via raw pointer to stay consistent & avoid any
    // packed-field reference lint.
    let mut name_bytes = [0u8; PLAYER_NAME_LENGTH];
    unsafe {
        ptr::copy_nonoverlapping(
            (packed as *const PlayerPack as *const u8).add(16),
            name_bytes.as_mut_ptr(),
            PLAYER_NAME_LENGTH,
        );
    }
    let mut name_end = PLAYER_NAME_LENGTH;
    for (i, &b) in name_bytes.iter().enumerate() {
        if b == 0 {
            name_end = i;
            break;
        }
    }
    let name = String::from_utf8_lossy(&name_bytes[..name_end]).into_owned();

    // --- spell level arrays ([u8; N], 1-byte aligned) ---
    let spl_lvl = {
        let mut v = vec![0u8; DIABLO_SPELL_SLOTS];
        unsafe {
            ptr::copy_nonoverlapping(
                (packed as *const PlayerPack as *const u8).add(79),
                v.as_mut_ptr(),
                DIABLO_SPELL_SLOTS,
            );
        }
        v
    };
    let spl_lvl2 = {
        let mut v = vec![0u8; HELLFIRE_SPELL_SLOTS];
        unsafe {
            ptr::copy_nonoverlapping(
                (packed as *const PlayerPack as *const u8).add(1222),
                v.as_mut_ptr(),
                HELLFIRE_SPELL_SLOTS,
            );
        }
        v
    };

    // --- inventory items (decoded but not yet recreated into full Items) ---
    // Iterating `packed.inv_body.iter()` would form `&ItemPack` references into
    // the packed struct (unaligned). Instead we decode each ItemPack directly
    // from the raw byte slice at its known offset.
    let inv_body = decode_item_array(bytes, 124, NUM_INVLOC);
    let inv_list = decode_item_array(bytes, 124 + NUM_INVLOC * ITEM_PACK_SIZE, INVENTORY_GRID_CELLS);
    let spd_list = decode_item_array(bytes, 1058, MAX_BELT_ITEMS);
    let inv_grid = {
        // [i8; 40] at offset 1017 — 1-byte aligned, copy via raw ptr.
        let mut v = vec![0i8; INVENTORY_GRID_CELLS];
        unsafe {
            ptr::copy_nonoverlapping(
                (packed as *const PlayerPack as *const u8).add(1017),
                v.as_mut_ptr() as *mut u8,
                INVENTORY_GRID_CELLS,
            );
        }
        v
    };
    let is_hellfire = packed.b_is_hellfire != 0;

    Ok(DecodedPlayer {
        class: HeroClass::from_packed(p_class_raw),
        level: p_level,
        name,
        plrlevel,
        px,
        py,
        base_str,
        base_mag,
        base_dex,
        base_vit,
        stat_pts,
        experience,
        gold,
        hp_base,
        max_hp_base,
        mana_base,
        max_mana_base,
        spl_lvl,
        spl_lvl2,
        mem_spells,
        inv_body,
        inv_list,
        inv_grid,
        num_inv: packed._p_num_inv,
        spd_list,
        is_hellfire,
        town_warps: packed.p_town_warps,
        dung_msgs: packed.p_dung_msgs,
        lvl_load: packed.p_lvl_load,
        battle_net: packed.p_battle_net != 0,
        mana_shield: packed.p_mana_shield != 0,
        dung_msgs2: packed.p_dung_msgs2,
        reflections,
        diablo_kill_level,
        difficulty,
        dam_ac_flags,
        raw: bytes[..PLAYER_PACK_SIZE].to_vec(),
    })
}

// Re-export std::ptr under a short alias for the `read_unaligned` calls above.
// (We use a module-level `use` to keep call sites concise.)
use std::ptr;

// ---------------------------------------------------------------------------
// Sanity: struct sizes match the C++ layout at compile time.
// ---------------------------------------------------------------------------

const _: () = {
    assert!(mem::size_of::<ItemPack>() == ITEM_PACK_SIZE);
    assert!(mem::size_of::<PlayerPack>() == PLAYER_PACK_SIZE);
};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a zeroed buffer of `PLAYER_PACK_SIZE` bytes and patch selected
    /// fields at their exact byte offsets. Offsets mirror the C++ layout.
    fn make_packed() -> Vec<u8> {
        vec![0u8; PLAYER_PACK_SIZE]
    }

    // Field offsets (from the C++ layout, #pragma pack(1)).
    const OFF_DW_LOW: usize = 0; // u32
    const OFF_DW_HIGH: usize = 4; // u32
    const OFF_PLRLEVEL: usize = 11; // u8
    const OFF_PX: usize = 12;
    const OFF_PY: usize = 13;
    const OFF_PNAME: usize = 16; // [u8;32]
    const OFF_PCLASS: usize = 48; // u8
    const OFF_BASE_STR: usize = 49;
    const OFF_BASE_MAG: usize = 50;
    const OFF_BASE_DEX: usize = 51;
    const OFF_BASE_VIT: usize = 52;
    const OFF_PLEVEL: usize = 53;
    const OFF_PSTATPTS: usize = 54;
    const OFF_PEXPERIENCE: usize = 55; // u32
    const OFF_PGOLD: usize = 59; // i32
    const OFF_PHPBASE: usize = 63; // i32
    const OFF_PMAXHPBASE: usize = 67; // i32
    const OFF_PMANABASE: usize = 71; // i32
    const OFF_PMAXMANABASE: usize = 75; // i32
    const OFF_PSPLLVL: usize = 79; // [u8;37]
    const OFF_PMEMSPELLS: usize = 116; // u64
    // inv_body starts at 124, inv_list at 124+7*19=257, inv_grid at 257+40*19=1017,
    // _pNumInv at 1057, spd_list at 1058..1210.
    const OFF_INVBODY: usize = 124;
    const OFF_INVLIST: usize = 124 + NUM_INVLOC * ITEM_PACK_SIZE; // 257
    const OFF_INVGRID: usize = OFF_INVLIST + INVENTORY_GRID_CELLS * ITEM_PACK_SIZE; // 1017
    const OFF_PNUMINV: usize = OFF_INVGRID + INVENTORY_GRID_CELLS; // 1057
    const OFF_SPDLIST: usize = OFF_PNUMINV + 1; // 1058
    const OFF_PTOWNWARPS: usize = OFF_SPDLIST + MAX_BELT_ITEMS * ITEM_PACK_SIZE; // 1210
    const OFF_PDUNGMSGS: usize = 1211;
    const OFF_PLVLLOAD: usize = 1212;
    const OFF_PBATTLENET: usize = 1213;
    const OFF_PMANASHIELD: usize = 1214;
    const OFF_PDUNGMSGS2: usize = 1215;
    const OFF_BISHELLFIRE: usize = 1216;
    const OFF_RESERVED: usize = 1217;
    const OFF_WREFLECTIONS: usize = 1218; // u16
    const OFF_PSPLLVL2: usize = 1222; // [u8;10]
    const OFF_WRESERVED8: usize = 1232; // i16
    const OFF_PDIABLOKILL: usize = 1234; // u32
    const OFF_PDIFFICULTY: usize = 1238; // u32
    const OFF_PDAMACFLAGS: usize = 1242; // u32

    fn put_u16(buf: &mut [u8], off: usize, v: u16) {
        buf[off..off + 2].copy_from_slice(&v.to_le_bytes());
    }
    fn put_u32(buf: &mut [u8], off: usize, v: u32) {
        buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
    }
    fn put_i32(buf: &mut [u8], off: usize, v: i32) {
        buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
    }

    #[test]
    fn struct_sizes_match_cpp_layout() {
        assert_eq!(mem::size_of::<ItemPack>(), 19);
        assert_eq!(mem::size_of::<PlayerPack>(), 1266);
    }

    #[test]
    fn unpack_decodes_core_fields() {
        let mut buf = make_packed();

        put_u32(&mut buf, OFF_DW_LOW, 0x11223344);
        put_u32(&mut buf, OFF_DW_HIGH, 0x55667788);
        buf[OFF_PLRLEVEL] = 5;
        buf[OFF_PX] = 75;
        buf[OFF_PY] = 110;

        // Name "Merlin" at offset 16.
        let name = b"Merlin";
        buf[OFF_PNAME..OFF_PNAME + name.len()].copy_from_slice(name);

        buf[OFF_PCLASS] = 2; // Sorcerer
        buf[OFF_BASE_STR] = 20;
        buf[OFF_BASE_MAG] = 60;
        buf[OFF_BASE_DEX] = 30;
        buf[OFF_BASE_VIT] = 25;
        buf[OFF_PLEVEL] = 18;
        buf[OFF_PSTATPTS] = 12;

        put_u32(&mut buf, OFF_PEXPERIENCE, 123_456);
        put_i32(&mut buf, OFF_PGOLD, 9_500);
        put_i32(&mut buf, OFF_PHPBASE, 1_920);
        put_i32(&mut buf, OFF_PMAXHPBASE, 2_000);
        put_i32(&mut buf, OFF_PMANABASE, 4_800);
        put_i32(&mut buf, OFF_PMAXMANABASE, 5_000);

        // Spell level slot 3 (Firebolt index 1 region) set to 7.
        buf[OFF_PSPLLVL + 3] = 7;
        buf[OFF_PSPLLVL + 1] = 9;

        // Memorised spells bitmask: low 3 bits set.
        put_u32(&mut buf, OFF_PMEMSPELLS, 0x0000_0007);
        put_u32(&mut buf, OFF_PMEMSPELLS + 4, 0x0000_0000);

        buf[OFF_PTOWNWARPS] = 2;
        buf[OFF_PBATTLENET] = 1;
        buf[OFF_PMANASHIELD] = 1;
        buf[OFF_BISHELLFIRE] = 0; // Diablo format
        put_u16(&mut buf, OFF_WREFLECTIONS, 3);
        buf[OFF_PSPLLVL2 + 0] = 4;
        put_u32(&mut buf, OFF_PDIABLOKILL, 16);
        put_u32(&mut buf, OFF_PDIFFICULTY, 1);
        put_u32(&mut buf, OFF_PDAMACFLAGS, 0xDEAD_BEEF);

        let player = unpack_player(&buf).expect("decode should succeed");

        assert_eq!(player.class, HeroClass::Sorcerer);
        assert_eq!(player.level, 18);
        assert_eq!(player.name, "Merlin");
        assert_eq!(player.plrlevel, 5);
        assert_eq!(player.px, 75);
        assert_eq!(player.py, 110);
        assert_eq!(player.base_str, 20);
        assert_eq!(player.base_mag, 60);
        assert_eq!(player.base_dex, 30);
        assert_eq!(player.base_vit, 25);
        assert_eq!(player.stat_pts, 12);
        assert_eq!(player.experience, 123_456);
        assert_eq!(player.gold, 9_500);
        assert_eq!(player.hp_base, 1_920);
        assert_eq!(player.max_hp_base, 2_000);
        assert_eq!(player.mana_base, 4_800);
        assert_eq!(player.max_mana_base, 5_000);
        assert_eq!(player.spl_lvl.len(), DIABLO_SPELL_SLOTS);
        assert_eq!(player.spl_lvl[3], 7);
        assert_eq!(player.spl_lvl[1], 9);
        assert_eq!(player.mem_spells, 0x0000_0000_0000_0007);
        assert_eq!(player.town_warps, 2);
        assert!(player.battle_net);
        assert!(player.mana_shield);
        assert!(!player.is_hellfire);
        assert_eq!(player.reflections, 3);
        assert_eq!(player.spl_lvl2[0], 4);
        assert_eq!(player.diablo_kill_level, 16);
        assert_eq!(player.difficulty, 1);
        assert_eq!(player.dam_ac_flags, 0xDEAD_BEEF);
    }

    #[test]
    fn effective_hp_clamps_and_floors() {
        // hp_base below 64 (after mask) -> floored to 64.
        let mut buf = make_packed();
        buf[OFF_PCLASS] = 0;
        put_i32(&mut buf, OFF_PHPBASE, 0);
        put_i32(&mut buf, OFF_PMAXHPBASE, 1000);
        let p = unpack_player(&buf).unwrap();
        assert_eq!(p.effective_hp(), 64);

        // hp_base above max -> clamped to max.
        let mut buf = make_packed();
        buf[OFF_PCLASS] = 0;
        put_i32(&mut buf, OFF_PHPBASE, 5000);
        put_i32(&mut buf, OFF_PMAXHPBASE, 1000);
        let p = unpack_player(&buf).unwrap();
        assert_eq!(p.effective_hp(), 1000);

        // Normal range -> unchanged.
        let mut buf = make_packed();
        buf[OFF_PCLASS] = 0;
        put_i32(&mut buf, OFF_PHPBASE, 750);
        put_i32(&mut buf, OFF_PMAXHPBASE, 1000);
        let p = unpack_player(&buf).unwrap();
        assert_eq!(p.effective_hp(), 750);
    }

    #[test]
    fn hero_class_clamps_out_of_range() {
        let mut buf = make_packed();
        buf[OFF_PCLASS] = 200; // out of range -> clamped to Barbarian (5)
        let p = unpack_player(&buf).unwrap();
        assert_eq!(p.class, HeroClass::Barbarian);

        // Each valid index maps exactly.
        for (raw, expected) in [
            (0u8, HeroClass::Warrior),
            (1, HeroClass::Rogue),
            (2, HeroClass::Sorcerer),
            (3, HeroClass::Monk),
            (4, HeroClass::Bard),
            (5, HeroClass::Barbarian),
        ] {
            let mut buf = make_packed();
            buf[OFF_PCLASS] = raw;
            let p = unpack_player(&buf).unwrap();
            assert_eq!(p.class, expected, "raw class byte {}", raw);
        }
    }

    #[test]
    fn name_handles_trailing_nuls_and_non_ascii() {
        let mut buf = make_packed();
        // "Ab" then NUL then garbage later — only "Ab" should decode.
        buf[OFF_PNAME] = b'A';
        buf[OFF_PNAME + 1] = b'b';
        // (rest already 0)
        let p = unpack_player(&buf).unwrap();
        assert_eq!(p.name, "Ab");

        // Full 32-byte name with no NUL (lossy UTF-8).
        let mut buf = make_packed();
        for b in buf[OFF_PNAME..OFF_PNAME + PLAYER_NAME_LENGTH].iter_mut() {
            *b = b'X';
        }
        let p = unpack_player(&buf).unwrap();
        assert_eq!(p.name.len(), PLAYER_NAME_LENGTH);
    }

    #[test]
    fn inv_body_and_spd_list_decode_empty_slots() {
        // Zeroed buffers => every item has idx 0x0000 != 0xFFFF, but idx 0 is the
        // "no item" sentinel in practice. Mark them empty explicitly by setting idx.
        let mut buf = make_packed();
        // First InvBody slot: set idx = 0xFFFF (empty), non-trivial seed.
        let body0 = OFF_INVBODY;
        put_u32(&mut buf, body0, 0xCAFEBABE); // iSeed
        put_u16(&mut buf, body0 + 4, 0x0102); // iCreateInfo
        put_u16(&mut buf, body0 + 6, 0xFFFF); // idx -> empty
        // SpdList[3] (belt slot 3): a real-ish item idx.
        let spd3 = OFF_SPDLIST + 3 * ITEM_PACK_SIZE;
        put_u32(&mut buf, spd3, 0x12345678);
        put_u16(&mut buf, spd3 + 6, 42); // idx 42 (non-empty)

        let p = unpack_player(&buf).unwrap();
        assert_eq!(p.inv_body.len(), NUM_INVLOC);
        assert!(p.inv_body[0].empty, "idx 0xFFFF slot should be empty");
        assert_eq!(p.inv_body[0].i_seed, 0xCAFEBABE);
        assert_eq!(p.inv_body[0].i_create_info, 0x0102);

        assert_eq!(p.spd_list.len(), MAX_BELT_ITEMS);
        assert!(!p.spd_list[3].empty);
        assert_eq!(p.spd_list[3].idx, 42);
        assert_eq!(p.spd_list[3].i_seed, 0x12345678);
    }

    #[test]
    fn inv_grid_and_num_inv_decode() {
        let mut buf = make_packed();
        buf[OFF_PNUMINV] = 7;
        // Set a few InvGrid cells.
        buf[OFF_INVGRID + 0] = 3u8.wrapping_neg() as u8; // -3
        buf[OFF_INVGRID + 5] = 5u8 as u8; // 5
        let p = unpack_player(&buf).unwrap();
        assert_eq!(p.num_inv, 7);
        assert_eq!(p.inv_grid.len(), INVENTORY_GRID_CELLS);
        assert_eq!(p.inv_grid[0], -3);
        assert_eq!(p.inv_grid[5], 5);
    }

    #[test]
    fn buffer_too_short_errors() {
        let short = vec![0u8; PLAYER_PACK_SIZE - 1];
        let err = unpack_player(&short).unwrap_err();
        assert_eq!(
            err,
            DecodeError::BufferTooShort {
                have: PLAYER_PACK_SIZE - 1,
                need: PLAYER_PACK_SIZE,
            }
        );
    }

    #[test]
    fn exact_sized_buffer_decodes() {
        // Exactly PLAYER_PACK_SIZE bytes should be accepted.
        let buf = vec![0u8; PLAYER_PACK_SIZE];
        let p = unpack_player(&buf).expect("exact-size buffer must decode");
        assert_eq!(p.class, HeroClass::Warrior); // class byte 0
        assert_eq!(p.raw.len(), PLAYER_PACK_SIZE);
    }

    #[test]
    fn extra_trailing_bytes_are_ignored() {
        let mut buf = vec![0u8; PLAYER_PACK_SIZE + 50];
        buf[OFF_PLEVEL] = 9;
        let p = unpack_player(&buf).unwrap();
        assert_eq!(p.level, 9);
        assert_eq!(p.raw.len(), PLAYER_PACK_SIZE);
    }
}
