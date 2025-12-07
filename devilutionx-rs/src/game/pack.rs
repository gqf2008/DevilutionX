// pack.rs - Player data packing/unpacking for save files and network
// Ported from Source/pack.cpp (612 lines)

pub use crate::game::player_exact::HeroClass;

/// Inventory slot count
pub const NUM_INVLOC: usize = 7;
/// Belt item count
pub const MAX_BELT_ITEMS: usize = 8;
/// Inventory grid cell count
pub const INVENTORY_GRID_CELLS: usize = 40;
/// Player name length
pub const PLAYER_NAME_LENGTH: usize = 16;

/// Simplified Item for pack/unpack operations
/// (This is NOT the full game Item - see items.rs for that)
#[derive(Debug, Clone, Default)]
pub struct PackedItem {
    pub id: u16,
    pub seed: u32,
    pub create_info: u16,
    pub identified: bool,
    pub magical_level: u8,
    pub durability: i32,
    pub max_durability: i32,
    pub charges: i32,
    pub max_charges: i32,
    pub value: i32,
    pub buff_flags: u32,
}

/// Type alias for backwards compatibility
pub type Item = PackedItem;

impl PackedItem {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.id == 0
    }

    pub fn is_gold(&self) -> bool {
        self.id == 1 // IDI_GOLD placeholder
    }
}

/// Simplified Player for pack/unpack operations
/// (This is NOT the full game Player - see player.rs for that)
#[derive(Debug, Clone)]
pub struct PackedPlayer {
    pub name: String,
    pub class: HeroClass,
    pub level: u8,
    pub position: (i32, i32),
    pub dest_action: i8,
    pub dest_param1: i8,
    pub dest_param2: i8,
    pub base_strength: i32,
    pub base_magic: i32,
    pub base_dexterity: i32,
    pub base_vitality: i32,
    pub strength: i32,
    pub magic: i32,
    pub dexterity: i32,
    pub vitality: i32,
    pub stat_points: u8,
    pub experience: u32,
    pub gold: i32,
    pub hp_base: i32,
    pub max_hp_base: i32,
    pub hit_points: i32,
    pub max_hp: i32,
    pub mana_base: i32,
    pub max_mana_base: i32,
    pub mana: i32,
    pub max_mana: i32,
    pub mem_spells: u64,
    pub spell_levels: [u8; MAX_SPELLS],
    pub reflections: u16,
    pub diablo_kill_level: u8,
    pub inv_body: [Item; NUM_INVLOC],
    pub inv_list: [Item; INVENTORY_GRID_CELLS],
    pub inv_grid: [i8; INVENTORY_GRID_CELLS],
    pub num_inv: u8,
    pub spd_list: [Item; MAX_BELT_ITEMS],
    pub friendly_mode: bool,
    pub on_set_level: bool,
    pub damage_mod: i32,
    pub base_to_blk: i32,
    pub i_min_dam: i32,
    pub i_max_dam: i32,
    pub i_ac: i32,
    pub i_bonus_dam: i32,
    pub i_bonus_to_hit: i32,
    pub i_bonus_ac: i32,
    pub i_bonus_dam_mod: i32,
    pub i_get_hit: i32,
    pub i_en_ac: i32,
    pub i_f_min_dam: i32,
    pub i_f_max_dam: i32,
    pub i_l_min_dam: i32,
    pub i_l_max_dam: i32,
}

/// Type alias for backwards compatibility
pub type Player = PackedPlayer;

impl Default for PackedPlayer {
    fn default() -> Self {
        Self {
            name: String::new(),
            class: HeroClass::default(),
            level: 0,
            position: (0, 0),
            dest_action: 0,
            dest_param1: 0,
            dest_param2: 0,
            base_strength: 0,
            base_magic: 0,
            base_dexterity: 0,
            base_vitality: 0,
            strength: 0,
            magic: 0,
            dexterity: 0,
            vitality: 0,
            stat_points: 0,
            experience: 0,
            gold: 0,
            hp_base: 0,
            max_hp_base: 0,
            hit_points: 0,
            max_hp: 0,
            mana_base: 0,
            max_mana_base: 0,
            mana: 0,
            max_mana: 0,
            mem_spells: 0,
            spell_levels: [0; MAX_SPELLS],
            reflections: 0,
            diablo_kill_level: 0,
            inv_body: std::array::from_fn(|_| Item::default()),
            inv_list: std::array::from_fn(|_| Item::default()),
            inv_grid: [0; INVENTORY_GRID_CELLS],
            num_inv: 0,
            spd_list: std::array::from_fn(|_| Item::default()),
            friendly_mode: false,
            on_set_level: false,
            damage_mod: 0,
            base_to_blk: 0,
            i_min_dam: 0,
            i_max_dam: 0,
            i_ac: 0,
            i_bonus_dam: 0,
            i_bonus_to_hit: 0,
            i_bonus_ac: 0,
            i_bonus_dam_mod: 0,
            i_get_hit: 0,
            i_en_ac: 0,
            i_f_min_dam: 0,
            i_f_max_dam: 0,
            i_l_min_dam: 0,
            i_l_max_dam: 0,
        }
    }
}

impl Player {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_character_level(&self) -> u8 {
        self.level
    }

    pub fn set_character_level(&mut self, level: u8) {
        self.level = level;
    }

    pub fn create(_class: HeroClass) -> Self {
        Self {
            class: _class,
            ..Self::default()
        }
    }
}

/// Maximum number of spells
pub const MAX_SPELLS: usize = 52;

/// Maximum spell levels stored in base pack (vanilla compatibility)
pub const MAX_SPELL_LEVELS_BASE: usize = 37;

/// Additional spell levels for Hellfire
pub const MAX_SPELL_LEVELS_HELLFIRE: usize = 10;

/// Packed item structure for save files
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct ItemPack {
    /// Item seed for RNG
    pub seed: u32,
    /// Creation info flags
    pub create_info: u16,
    /// Item index
    pub idx: u16,
    /// ID byte (magical + identified flags)
    pub b_id: u8,
    /// Current durability
    pub b_dur: u8,
    /// Max durability
    pub b_max_dur: u8,
    /// Current charges
    pub b_ch: u8,
    /// Max charges
    pub b_max_ch: u8,
    /// Item value
    pub value: u16,
    /// Item buff flags
    pub dw_buff: u32,
}

impl ItemPack {
    /// Size of packed item in bytes
    pub const SIZE: usize = 18;

    /// Create empty item pack
    pub fn empty() -> Self {
        Self {
            idx: 0xFFFF,
            ..Default::default()
        }
    }

    /// Check if this is an empty item
    pub fn is_empty(&self) -> bool {
        self.idx == 0xFFFF
    }

    /// Check if this is an ear item
    pub fn is_ear(&self) -> bool {
        self.idx != 0xFFFF && (self.idx & 0xFF) == 25 // IDI_EAR
    }

    /// Get the item index
    pub fn item_index(&self) -> u16 {
        u16::from_le(self.idx)
    }

    /// Get the seed value
    pub fn get_seed(&self) -> u32 {
        u32::from_le(self.seed)
    }

    /// Get creation info
    pub fn get_create_info(&self) -> u16 {
        u16::from_le(self.create_info)
    }

    /// Is item identified
    pub fn is_identified(&self) -> bool {
        (self.b_id & 1) != 0
    }

    /// Get magical level (0-2)
    pub fn magical_level(&self) -> u8 {
        (self.b_id >> 1) & 0x03
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..4].copy_from_slice(&self.seed.to_le_bytes());
        bytes[4..6].copy_from_slice(&self.create_info.to_le_bytes());
        bytes[6..8].copy_from_slice(&self.idx.to_le_bytes());
        bytes[8] = self.b_id;
        bytes[9] = self.b_dur;
        bytes[10] = self.b_max_dur;
        bytes[11] = self.b_ch;
        bytes[12] = self.b_max_ch;
        bytes[13..15].copy_from_slice(&self.value.to_le_bytes());
        bytes[15..19].copy_from_slice(&self.dw_buff.to_le_bytes());
        bytes
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() < Self::SIZE {
            return Self::empty();
        }
        Self {
            seed: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            create_info: u16::from_le_bytes([bytes[4], bytes[5]]),
            idx: u16::from_le_bytes([bytes[6], bytes[7]]),
            b_id: bytes[8],
            b_dur: bytes[9],
            b_max_dur: bytes[10],
            b_ch: bytes[11],
            b_max_ch: bytes[12],
            value: u16::from_le_bytes([bytes[13], bytes[14]]),
            dw_buff: u32::from_le_bytes([bytes[15], bytes[16], bytes[17], bytes[18]]),
        }
    }
}

/// Packed player structure for save files
#[derive(Debug, Clone)]
pub struct PlayerPack {
    /// Low date time
    pub dw_low_date_time: u32,
    /// High date time
    pub dw_high_date_time: u32,
    /// Destination action
    pub dest_action: i8,
    /// Destination param 1
    pub dest_param1: i8,
    /// Destination param 2
    pub dest_param2: i8,
    /// Player level (dungeon level)
    pub plr_level: u8,
    /// Position X
    pub px: u8,
    /// Position Y
    pub py: u8,
    /// Target X
    pub targ_x: u8,
    /// Target Y
    pub targ_y: u8,
    /// Player name
    pub name: String,
    /// Player class
    pub class: u8,
    /// Base strength
    pub base_str: u8,
    /// Base magic
    pub base_mag: u8,
    /// Base dexterity
    pub base_dex: u8,
    /// Base vitality
    pub base_vit: u8,
    /// Character level
    pub level: u8,
    /// Stat points available
    pub stat_pts: u8,
    /// Experience points
    pub experience: u32,
    /// Gold amount
    pub gold: i32,
    /// HP base
    pub hp_base: i32,
    /// Max HP base
    pub max_hp_base: i32,
    /// Mana base
    pub mana_base: i32,
    /// Max mana base
    pub max_mana_base: i32,
    /// Spell levels (base 37)
    pub spl_lvl: [u8; MAX_SPELL_LEVELS_BASE],
    /// Memorized spells bitmask
    pub mem_spells: u64,
    /// Inventory body slots
    pub inv_body: [ItemPack; NUM_INVLOC],
    /// Inventory list items
    pub inv_list: [ItemPack; INVENTORY_GRID_CELLS],
    /// Inventory grid layout
    pub inv_grid: [i8; INVENTORY_GRID_CELLS],
    /// Number of items in inventory
    pub num_inv: u8,
    /// Speed list (belt items)
    pub spd_list: [ItemPack; MAX_BELT_ITEMS],
    /// Town warps
    pub town_warps: i8,
    /// Dungeon messages
    pub dung_msgs: i8,
    /// Level load flags
    pub lvl_load: i8,
    /// Battle.net flag
    pub battle_net: u8,
    /// Mana shield active
    pub mana_shield: u8,
    /// Dungeon messages 2
    pub dung_msgs2: u8,
    /// Is Hellfire save format
    pub is_hellfire: i8,
    /// Reserved byte
    pub reserved: u8,
    /// Reflections count
    pub reflections: u16,
    /// Reserved bytes 2
    pub reserved2: [u8; 2],
    /// Hellfire spell levels
    pub spl_lvl2: [u8; MAX_SPELL_LEVELS_HELLFIRE],
    /// Reserved short
    pub reserved8: i16,
    /// Diablo kill level
    pub diablo_kill_level: u32,
    /// Difficulty level
    pub difficulty: u32,
    /// Damage AC flags
    pub dam_ac_flags: u32,
    /// Reserved bytes 3
    pub reserved3: [u8; 20],
}

impl Default for PlayerPack {
    fn default() -> Self {
        Self {
            dw_low_date_time: 0,
            dw_high_date_time: 0,
            dest_action: 0,
            dest_param1: 0,
            dest_param2: 0,
            plr_level: 0,
            px: 0,
            py: 0,
            targ_x: 0,
            targ_y: 0,
            name: String::new(),
            class: 0,
            base_str: 0,
            base_mag: 0,
            base_dex: 0,
            base_vit: 0,
            level: 0,
            stat_pts: 0,
            experience: 0,
            gold: 0,
            hp_base: 0,
            max_hp_base: 0,
            mana_base: 0,
            max_mana_base: 0,
            spl_lvl: [0; MAX_SPELL_LEVELS_BASE],
            mem_spells: 0,
            inv_body: [ItemPack::empty(); NUM_INVLOC],
            inv_list: [ItemPack::empty(); INVENTORY_GRID_CELLS],
            inv_grid: [0; INVENTORY_GRID_CELLS],
            num_inv: 0,
            spd_list: [ItemPack::empty(); MAX_BELT_ITEMS],
            town_warps: 0,
            dung_msgs: 0,
            lvl_load: 0,
            battle_net: 0,
            mana_shield: 0,
            dung_msgs2: 0,
            is_hellfire: 0,
            reserved: 0,
            reflections: 0,
            reserved2: [0; 2],
            spl_lvl2: [0; MAX_SPELL_LEVELS_HELLFIRE],
            reserved8: 0,
            diablo_kill_level: 0,
            difficulty: 0,
            dam_ac_flags: 0,
            reserved3: [0; 20],
        }
    }
}

impl PlayerPack {
    /// Create a new empty player pack
    pub fn new() -> Self {
        Self::default()
    }

    /// Get hero class from packed value
    pub fn hero_class(&self) -> HeroClass {
        HeroClass::try_from(self.class).unwrap_or(HeroClass::Warrior)
    }

    /// Check if this is a Hellfire save
    pub fn is_hellfire_save(&self) -> bool {
        self.is_hellfire != 0
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1024);

        bytes.extend_from_slice(&self.dw_low_date_time.to_le_bytes());
        bytes.extend_from_slice(&self.dw_high_date_time.to_le_bytes());
        bytes.push(self.dest_action as u8);
        bytes.push(self.dest_param1 as u8);
        bytes.push(self.dest_param2 as u8);
        bytes.push(self.plr_level);
        bytes.push(self.px);
        bytes.push(self.py);
        bytes.push(self.targ_x);
        bytes.push(self.targ_y);

        // Name (16 bytes, null-padded)
        let name_bytes = self.name.as_bytes();
        for i in 0..PLAYER_NAME_LENGTH {
            bytes.push(name_bytes.get(i).copied().unwrap_or(0));
        }

        bytes.push(self.class);
        bytes.push(self.base_str);
        bytes.push(self.base_mag);
        bytes.push(self.base_dex);
        bytes.push(self.base_vit);
        bytes.push(self.level);
        bytes.push(self.stat_pts);
        bytes.extend_from_slice(&self.experience.to_le_bytes());
        bytes.extend_from_slice(&self.gold.to_le_bytes());
        bytes.extend_from_slice(&self.hp_base.to_le_bytes());
        bytes.extend_from_slice(&self.max_hp_base.to_le_bytes());
        bytes.extend_from_slice(&self.mana_base.to_le_bytes());
        bytes.extend_from_slice(&self.max_mana_base.to_le_bytes());
        bytes.extend_from_slice(&self.spl_lvl);
        bytes.extend_from_slice(&self.mem_spells.to_le_bytes());

        // Inventory body
        for item in &self.inv_body {
            bytes.extend_from_slice(&item.to_bytes());
        }

        // Inventory list
        for item in &self.inv_list {
            bytes.extend_from_slice(&item.to_bytes());
        }

        // Inventory grid
        for &grid in &self.inv_grid {
            bytes.push(grid as u8);
        }

        bytes.push(self.num_inv);

        // Speed list (belt)
        for item in &self.spd_list {
            bytes.extend_from_slice(&item.to_bytes());
        }

        bytes.push(self.town_warps as u8);
        bytes.push(self.dung_msgs as u8);
        bytes.push(self.lvl_load as u8);
        bytes.push(self.battle_net);
        bytes.push(self.mana_shield);
        bytes.push(self.dung_msgs2);
        bytes.push(self.is_hellfire as u8);
        bytes.push(self.reserved);
        bytes.extend_from_slice(&self.reflections.to_le_bytes());
        bytes.extend_from_slice(&self.reserved2);
        bytes.extend_from_slice(&self.spl_lvl2);
        bytes.extend_from_slice(&self.reserved8.to_le_bytes());
        bytes.extend_from_slice(&self.diablo_kill_level.to_le_bytes());
        bytes.extend_from_slice(&self.difficulty.to_le_bytes());
        bytes.extend_from_slice(&self.dam_ac_flags.to_le_bytes());
        bytes.extend_from_slice(&self.reserved3);

        bytes
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut pack = Self::default();

        if bytes.len() < 100 {
            return pack;
        }

        pack.dw_low_date_time = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        pack.dw_high_date_time = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        pack.dest_action = bytes[8] as i8;
        pack.dest_param1 = bytes[9] as i8;
        pack.dest_param2 = bytes[10] as i8;
        pack.plr_level = bytes[11];
        pack.px = bytes[12];
        pack.py = bytes[13];
        pack.targ_x = bytes[14];
        pack.targ_y = bytes[15];

        // Name (16 bytes)
        let mut offset = 16;
        let name_end = offset + PLAYER_NAME_LENGTH;
        let name_bytes = &bytes[offset..name_end];
        pack.name = String::from_utf8_lossy(
            &name_bytes[..name_bytes.iter().position(|&b| b == 0).unwrap_or(PLAYER_NAME_LENGTH)]
        ).to_string();
        offset = name_end;

        pack.class = bytes[offset]; offset += 1;
        pack.base_str = bytes[offset]; offset += 1;
        pack.base_mag = bytes[offset]; offset += 1;
        pack.base_dex = bytes[offset]; offset += 1;
        pack.base_vit = bytes[offset]; offset += 1;
        pack.level = bytes[offset]; offset += 1;
        pack.stat_pts = bytes[offset]; offset += 1;

        pack.experience = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]);
        offset += 4;
        pack.gold = i32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]);
        offset += 4;
        pack.hp_base = i32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]);
        offset += 4;
        pack.max_hp_base = i32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]);
        offset += 4;
        pack.mana_base = i32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]);
        offset += 4;
        pack.max_mana_base = i32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]);
        offset += 4;

        pack.spl_lvl.copy_from_slice(&bytes[offset..offset + MAX_SPELL_LEVELS_BASE]);
        offset += MAX_SPELL_LEVELS_BASE;

        pack.mem_spells = u64::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
            bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
        ]);
        offset += 8;

        // Read inventory body
        for i in 0..NUM_INVLOC {
            pack.inv_body[i] = ItemPack::from_bytes(&bytes[offset..]);
            offset += ItemPack::SIZE;
        }

        // Read inventory list
        for i in 0..INVENTORY_GRID_CELLS {
            pack.inv_list[i] = ItemPack::from_bytes(&bytes[offset..]);
            offset += ItemPack::SIZE;
        }

        // Read inventory grid
        for i in 0..INVENTORY_GRID_CELLS {
            pack.inv_grid[i] = bytes[offset] as i8;
            offset += 1;
        }

        pack.num_inv = bytes[offset]; offset += 1;

        // Read speed list (belt)
        for i in 0..MAX_BELT_ITEMS {
            pack.spd_list[i] = ItemPack::from_bytes(&bytes[offset..]);
            offset += ItemPack::SIZE;
        }

        // Additional fields
        if offset + 30 <= bytes.len() {
            pack.town_warps = bytes[offset] as i8; offset += 1;
            pack.dung_msgs = bytes[offset] as i8; offset += 1;
            pack.lvl_load = bytes[offset] as i8; offset += 1;
            pack.battle_net = bytes[offset]; offset += 1;
            pack.mana_shield = bytes[offset]; offset += 1;
            pack.dung_msgs2 = bytes[offset]; offset += 1;
            pack.is_hellfire = bytes[offset] as i8; offset += 1;
            pack.reserved = bytes[offset]; offset += 1;
            pack.reflections = u16::from_le_bytes([bytes[offset], bytes[offset+1]]);
            offset += 2;
            pack.reserved2.copy_from_slice(&bytes[offset..offset+2]);
            offset += 2;
            pack.spl_lvl2.copy_from_slice(&bytes[offset..offset + MAX_SPELL_LEVELS_HELLFIRE]);
            offset += MAX_SPELL_LEVELS_HELLFIRE;
            pack.reserved8 = i16::from_le_bytes([bytes[offset], bytes[offset+1]]);
            offset += 2;
            pack.diablo_kill_level = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]);
            offset += 4;
            pack.difficulty = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]);
            offset += 4;
            pack.dam_ac_flags = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]);
        }

        pack
    }
}

/// Network packed item union (def/item/ear)
#[derive(Debug, Clone)]
pub enum ItemNetPack {
    Empty,
    Def(ItemNetDef),
    Item(ItemNetItem),
    Ear(ItemNetEar),
}

/// Network item definition
#[derive(Debug, Clone, Default)]
pub struct ItemNetDef {
    pub idx: u16,
    pub create_info: u16,
    pub seed: u32,
}

/// Network item data
#[derive(Debug, Clone, Default)]
pub struct ItemNetItem {
    pub def: ItemNetDef,
    pub durability: u8,
    pub max_durability: u8,
    pub charges: u8,
    pub max_charges: u8,
    pub buff: u32,
}

/// Network ear data
#[derive(Debug, Clone, Default)]
pub struct ItemNetEar {
    pub def: ItemNetDef,
    pub cursor_val: u8,
    pub hero_name: String,
}

/// Network packed player structure
#[derive(Debug, Clone)]
pub struct PlayerNetPack {
    pub plr_level: u8,
    pub px: u8,
    pub py: u8,
    pub name: String,
    pub class: u8,
    pub base_str: u8,
    pub base_mag: u8,
    pub base_dex: u8,
    pub base_vit: u8,
    pub level: i8,
    pub stat_pts: u8,
    pub experience: u32,
    pub hp_base: i32,
    pub max_hp_base: i32,
    pub mana_base: i32,
    pub max_mana_base: i32,
    pub spl_lvl: [u8; MAX_SPELLS],
    pub mem_spells: u64,
    pub inv_body: Vec<ItemNetPack>,
    pub inv_list: Vec<ItemNetPack>,
    pub inv_grid: [i8; INVENTORY_GRID_CELLS],
    pub num_inv: u8,
    pub spd_list: Vec<ItemNetPack>,
    pub mana_shield: u8,
    pub reflections: u16,
    pub diablo_kill_level: u8,
    pub friendly_mode: u8,
    pub is_on_set_level: u8,
    // Validation fields
    pub strength: i32,
    pub magic: i32,
    pub dexterity: i32,
    pub vitality: i32,
    pub hit_points: i32,
    pub max_hp: i32,
    pub mana: i32,
    pub max_mana: i32,
    pub damage_mod: i32,
    pub base_to_blk: i32,
    pub i_min_dam: i32,
    pub i_max_dam: i32,
    pub i_ac: i32,
    pub i_bonus_dam: i32,
    pub i_bonus_to_hit: i32,
    pub i_bonus_ac: i32,
    pub i_bonus_dam_mod: i32,
    pub i_get_hit: i32,
    pub i_en_ac: i32,
    pub i_f_min_dam: i32,
    pub i_f_max_dam: i32,
    pub i_l_min_dam: i32,
    pub i_l_max_dam: i32,
}

impl Default for PlayerNetPack {
    fn default() -> Self {
        Self {
            plr_level: 0,
            px: 0,
            py: 0,
            name: String::new(),
            class: 0,
            base_str: 0,
            base_mag: 0,
            base_dex: 0,
            base_vit: 0,
            level: 0,
            stat_pts: 0,
            experience: 0,
            hp_base: 0,
            max_hp_base: 0,
            mana_base: 0,
            max_mana_base: 0,
            spl_lvl: [0; MAX_SPELLS],
            mem_spells: 0,
            inv_body: Vec::new(),
            inv_list: Vec::new(),
            inv_grid: [0; INVENTORY_GRID_CELLS],
            num_inv: 0,
            spd_list: Vec::new(),
            mana_shield: 0,
            reflections: 0,
            diablo_kill_level: 0,
            friendly_mode: 0,
            is_on_set_level: 0,
            strength: 0,
            magic: 0,
            dexterity: 0,
            vitality: 0,
            hit_points: 0,
            max_hp: 0,
            mana: 0,
            max_mana: 0,
            damage_mod: 0,
            base_to_blk: 0,
            i_min_dam: 0,
            i_max_dam: 0,
            i_ac: 0,
            i_bonus_dam: 0,
            i_bonus_to_hit: 0,
            i_bonus_ac: 0,
            i_bonus_dam_mod: 0,
            i_get_hit: 0,
            i_en_ac: 0,
            i_f_min_dam: 0,
            i_f_max_dam: 0,
            i_l_min_dam: 0,
            i_l_max_dam: 0,
        }
    }
}

/// Pack an item into ItemPack format
#[allow(unused_variables)]
pub fn pack_item(item: &Item, is_hellfire: bool) -> ItemPack {
    if item.is_empty() {
        return ItemPack::empty();
    }

    let mut packed = ItemPack::default();
    packed.idx = item.id.to_le();
    packed.seed = item.seed.to_le();
    packed.create_info = item.create_info.to_le();

    let id_byte = (item.magical_level << 1) | (if item.identified { 1 } else { 0 });
    packed.b_id = id_byte;

    packed.b_max_dur = if item.max_durability > 255 { 254 } else { item.max_durability as u8 };
    packed.b_dur = std::cmp::min(item.durability as u8, packed.b_max_dur);
    packed.b_ch = item.charges as u8;
    packed.b_max_ch = item.max_charges as u8;

    if item.is_gold() {
        packed.value = (item.value as u16).to_le();
    }

    packed.dw_buff = item.buff_flags;

    packed
}

/// Unpack an ItemPack into an Item
#[allow(unused_variables)]
pub fn unpack_item(packed: &ItemPack, _player: &Player, is_hellfire: bool) -> Item {
    if packed.is_empty() {
        return Item::empty();
    }

    let mut item = Item::default();
    item.id = u16::from_le(packed.idx);
    item.seed = u32::from_le(packed.seed);
    item.create_info = u16::from_le(packed.create_info);
    item.identified = packed.is_identified();
    item.magical_level = packed.magical_level();
    item.max_durability = packed.b_max_dur as i32;
    item.durability = std::cmp::min(packed.b_dur as i32, item.max_durability);
    item.max_charges = packed.b_max_ch as i32;
    item.charges = std::cmp::min(packed.b_ch as i32, item.max_charges);
    item.value = u16::from_le(packed.value) as i32;
    item.buff_flags = u32::from_le(packed.dw_buff);

    item
}

/// Pack a player into PlayerPack format
pub fn pack_player(player: &Player) -> PlayerPack {
    let mut packed = PlayerPack::default();

    packed.dest_action = player.dest_action;
    packed.dest_param1 = player.dest_param1;
    packed.dest_param2 = player.dest_param2;
    packed.plr_level = player.level;
    packed.px = player.position.0 as u8;
    packed.py = player.position.1 as u8;
    packed.targ_x = player.position.0 as u8;
    packed.targ_y = player.position.1 as u8;
    packed.name = player.name.clone();
    packed.class = player.class as u8;
    packed.base_str = player.base_strength as u8;
    packed.base_mag = player.base_magic as u8;
    packed.base_dex = player.base_dexterity as u8;
    packed.base_vit = player.base_vitality as u8;
    packed.level = player.get_character_level();
    packed.stat_pts = player.stat_points;
    packed.experience = player.experience.to_le();
    packed.gold = player.gold.to_le();
    packed.hp_base = player.hp_base.to_le();
    packed.max_hp_base = player.max_hp_base.to_le();
    packed.mana_base = player.mana_base.to_le();
    packed.max_mana_base = player.max_mana_base.to_le();
    packed.mem_spells = player.mem_spells.to_le();

    // Copy spell levels
    for i in 0..MAX_SPELL_LEVELS_BASE {
        packed.spl_lvl[i] = player.spell_levels.get(i).copied().unwrap_or(0);
    }
    for i in 0..MAX_SPELL_LEVELS_HELLFIRE {
        packed.spl_lvl2[i] = player.spell_levels.get(MAX_SPELL_LEVELS_BASE + i).copied().unwrap_or(0);
    }

    packed.reflections = player.reflections.to_le();
    packed.diablo_kill_level = player.diablo_kill_level as u32;

    packed
}

/// Unpack a PlayerPack into a Player
pub fn unpack_player(packed: &PlayerPack, player: &mut Player) {
    player.name = packed.name.clone();
    player.class = packed.hero_class();
    player.level = packed.plr_level;
    player.position = (packed.px as i32, packed.py as i32);

    player.base_strength = packed.base_str as i32;
    player.base_magic = packed.base_mag as i32;
    player.base_dexterity = packed.base_dex as i32;
    player.base_vitality = packed.base_vit as i32;

    player.strength = player.base_strength;
    player.magic = player.base_magic;
    player.dexterity = player.base_dexterity;
    player.vitality = player.base_vitality;

    player.set_character_level(packed.level);
    player.stat_points = packed.stat_pts;
    player.experience = u32::from_le(packed.experience);
    player.gold = i32::from_le(packed.gold);

    player.hp_base = i32::from_le(packed.hp_base);
    player.max_hp_base = i32::from_le(packed.max_hp_base);
    player.hp_base = player.hp_base.clamp(0, player.max_hp_base);

    player.mana_base = i32::from_le(packed.mana_base);
    player.max_mana_base = i32::from_le(packed.max_mana_base);
    player.mana_base = player.mana_base.clamp(0, player.max_mana_base);

    player.mem_spells = u64::from_le(packed.mem_spells);

    // Copy spell levels
    for i in 0..MAX_SPELL_LEVELS_BASE {
        if i < player.spell_levels.len() {
            player.spell_levels[i] = packed.spl_lvl[i];
        }
    }
    for i in 0..MAX_SPELL_LEVELS_HELLFIRE {
        let idx = MAX_SPELL_LEVELS_BASE + i;
        if idx < player.spell_levels.len() {
            player.spell_levels[idx] = packed.spl_lvl2[i];
        }
    }

    player.reflections = u16::from_le(packed.reflections);
    player.diablo_kill_level = packed.diablo_kill_level as u8;
}

/// Pack a player for network transfer
pub fn pack_net_player(player: &Player) -> PlayerNetPack {
    let mut packed = PlayerNetPack::default();

    packed.plr_level = player.level;
    packed.px = player.position.0 as u8;
    packed.py = player.position.1 as u8;
    packed.name = player.name.clone();
    packed.class = player.class as u8;
    packed.base_str = player.base_strength as u8;
    packed.base_mag = player.base_magic as u8;
    packed.base_dex = player.base_dexterity as u8;
    packed.base_vit = player.base_vitality as u8;
    packed.level = player.get_character_level() as i8;
    packed.stat_pts = player.stat_points;
    packed.experience = player.experience.to_le();
    packed.hp_base = player.hp_base.to_le();
    packed.max_hp_base = player.max_hp_base.to_le();
    packed.mana_base = player.mana_base.to_le();
    packed.max_mana_base = player.max_mana_base.to_le();
    packed.mem_spells = player.mem_spells.to_le();

    // Validation fields
    packed.strength = player.strength.to_le();
    packed.magic = player.magic.to_le();
    packed.dexterity = player.dexterity.to_le();
    packed.vitality = player.vitality.to_le();
    packed.hit_points = player.hit_points.to_le();
    packed.max_hp = player.max_hp.to_le();
    packed.mana = player.mana.to_le();
    packed.max_mana = player.max_mana.to_le();

    packed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_pack_empty() {
        let pack = ItemPack::empty();
        assert!(pack.is_empty());
        let idx = { pack.idx }; // Copy to avoid unaligned reference
        assert_eq!(idx, 0xFFFF);
    }

    #[test]
    fn test_item_pack_size() {
        assert_eq!(ItemPack::SIZE, 18);
    }

    #[test]
    fn test_item_pack_identified() {
        let mut pack = ItemPack::default();
        pack.b_id = 0;
        assert!(!pack.is_identified());

        pack.b_id = 1;
        assert!(pack.is_identified());

        pack.b_id = 3;
        assert!(pack.is_identified());
    }

    #[test]
    fn test_item_pack_magical_level() {
        let mut pack = ItemPack::default();
        pack.b_id = 0b00000000;
        assert_eq!(pack.magical_level(), 0);

        pack.b_id = 0b00000010;
        assert_eq!(pack.magical_level(), 1);

        pack.b_id = 0b00000100;
        assert_eq!(pack.magical_level(), 2);

        pack.b_id = 0b00000110;
        assert_eq!(pack.magical_level(), 3);
    }

    #[test]
    fn test_item_pack_serialize() {
        let pack = ItemPack {
            seed: 12345,
            create_info: 100,
            idx: 50,
            b_id: 1,
            b_dur: 25,
            b_max_dur: 30,
            b_ch: 5,
            b_max_ch: 10,
            value: 500,
            dw_buff: 0xFF,
        };

        let bytes = pack.to_bytes();
        let restored = ItemPack::from_bytes(&bytes);

        // Copy to local vars to avoid unaligned reference
        let (r_seed, p_seed) = ({ restored.seed }, { pack.seed });
        let (r_create, p_create) = ({ restored.create_info }, { pack.create_info });
        let (r_idx, p_idx) = ({ restored.idx }, { pack.idx });
        let (r_bid, p_bid) = (restored.b_id, pack.b_id);
        let (r_dur, p_dur) = (restored.b_dur, pack.b_dur);

        assert_eq!(r_seed, p_seed);
        assert_eq!(r_create, p_create);
        assert_eq!(r_idx, p_idx);
        assert_eq!(r_bid, p_bid);
        assert_eq!(r_dur, p_dur);
    }

    #[test]
    fn test_player_pack_default() {
        let pack = PlayerPack::default();
        assert!(pack.name.is_empty());
        assert_eq!(pack.class, 0);
        assert_eq!(pack.level, 0);
        assert_eq!(pack.is_hellfire, 0);
    }

    #[test]
    fn test_player_pack_hero_class() {
        let mut pack = PlayerPack::default();

        pack.class = 0;
        assert_eq!(pack.hero_class(), HeroClass::Warrior);

        pack.class = 1;
        assert_eq!(pack.hero_class(), HeroClass::Rogue);

        pack.class = 2;
        assert_eq!(pack.hero_class(), HeroClass::Sorcerer);
    }

    #[test]
    fn test_player_pack_is_hellfire() {
        let mut pack = PlayerPack::default();

        pack.is_hellfire = 0;
        assert!(!pack.is_hellfire_save());

        pack.is_hellfire = 1;
        assert!(pack.is_hellfire_save());
    }

    #[test]
    fn test_max_spells() {
        assert_eq!(MAX_SPELLS, 52);
        assert_eq!(MAX_SPELL_LEVELS_BASE, 37);
        assert_eq!(MAX_SPELL_LEVELS_HELLFIRE, 10);
    }

    #[test]
    fn test_pack_unpack_item() {
        let mut item = Item::default();
        item.id = 10;
        item.seed = 54321;
        item.create_info = 200;
        item.identified = true;
        item.magical_level = 2;
        item.durability = 20;
        item.max_durability = 30;
        item.charges = 3;
        item.max_charges = 5;

        let packed = pack_item(&item, false);
        let player = Player::default();
        let restored = unpack_item(&packed, &player, false);

        assert_eq!(restored.id, item.id);
        assert_eq!(restored.seed, item.seed);
        assert_eq!(restored.identified, item.identified);
    }

    #[test]
    fn test_pack_empty_item() {
        let item = Item::empty();
        let packed = pack_item(&item, false);
        assert!(packed.is_empty());
    }

    #[test]
    fn test_pack_unpack_player() {
        let mut player = Player::default();
        player.name = "TestHero".to_string();
        player.class = HeroClass::Warrior;
        player.level = 5;
        player.position = (10, 20);
        player.base_strength = 30;
        player.base_magic = 10;
        player.base_dexterity = 20;
        player.base_vitality = 25;
        player.experience = 1000;
        player.gold = 500;

        let packed = pack_player(&player);

        let mut restored = Player::default();
        unpack_player(&packed, &mut restored);

        assert_eq!(restored.name, player.name);
        assert_eq!(restored.class, player.class);
        assert_eq!(restored.position, player.position);
        assert_eq!(restored.base_strength, player.base_strength);
    }

    #[test]
    fn test_item_net_pack_empty() {
        let pack = ItemNetPack::Empty;
        assert!(matches!(pack, ItemNetPack::Empty));
    }

    #[test]
    fn test_item_net_def() {
        let def = ItemNetDef {
            idx: 10,
            create_info: 100,
            seed: 12345,
        };
        assert_eq!(def.idx, 10);
        assert_eq!(def.create_info, 100);
    }

    #[test]
    fn test_player_net_pack_default() {
        let pack = PlayerNetPack::default();
        assert!(pack.name.is_empty());
        assert_eq!(pack.plr_level, 0);
        assert_eq!(pack.strength, 0);
    }

    #[test]
    fn test_pack_net_player() {
        let mut player = Player::default();
        player.name = "NetPlayer".to_string();
        player.level = 3;
        player.strength = 25;
        player.magic = 15;

        let packed = pack_net_player(&player);

        assert_eq!(packed.name, "NetPlayer");
        assert_eq!(packed.plr_level, 3);
    }

    #[test]
    fn test_player_pack_serialize_deserialize() {
        let mut pack = PlayerPack::default();
        pack.name = "SerializeTest".to_string();
        pack.class = 1;
        pack.level = 10;
        pack.experience = 5000;
        pack.gold = 1000;
        pack.hp_base = 100;
        pack.max_hp_base = 150;

        let bytes = pack.to_bytes();
        let restored = PlayerPack::from_bytes(&bytes);

        assert_eq!(restored.name, pack.name);
        assert_eq!(restored.class, pack.class);
        assert_eq!(restored.level, pack.level);
    }
}
