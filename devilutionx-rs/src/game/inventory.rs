//! Inventory System
//!
//! Implements the player inventory and equipment management system for Diablo/Hellfire.
//! This includes equipment slots, backpack grid, belt, and item usage.
//!
//! C++ source: Source/inv.cpp (1,946 lines) + Source/inv.h (395 lines)
//! Rust target: ~850 lines (core logic only, excluding UI rendering)

use crate::game::items::{Item, ItemType};
use crate::game::player_dat::HeroClass;

/// Maximum inventory grid size (10 columns × 4 rows)
pub const INVENTORY_WIDTH: usize = 10;
pub const INVENTORY_HEIGHT: usize = 4;
pub const INVENTORY_SIZE: usize = INVENTORY_WIDTH * INVENTORY_HEIGHT; // 40 slots

/// Number of equipment slots
pub const NUM_EQUIPMENT_SLOTS: usize = 7;

/// Number of belt slots
pub const NUM_BELT_SLOTS: usize = 8;

/// Total number of inventory screen slots (equipment + inventory + belt)
pub const NUM_XY_SLOTS: usize = 55;

// Day 93: Inventory Foundation
// =============================
// - InventorySlot enum (55 screen slots)
// - Equipment struct (7 equipment slots)
// - Inventory struct (40 grid slots)
// - Belt struct (8 belt slots)
// - Basic methods: new(), is_empty(), clear()
//
// C++ Reference: Source/inv.h:26-76, Source/player.h:216-218

/// Inventory slot enumeration (55 total slots on inventory screen)
///
/// C++ equivalent: inv_item + inv_xy_slot in Source/inv.h:26-76
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum InventorySlot {
    /// Head slot (helmet) - slot 0
    Head = 0,

    /// Left ring slot - slot 1
    RingLeft = 1,

    /// Right ring slot - slot 2
    RingRight = 2,

    /// Amulet slot - slot 3
    Amulet = 3,

    /// Left hand slot (weapon/shield) - slot 4
    HandLeft = 4,

    /// Right hand slot (weapon/shield) - slot 5
    HandRight = 5,

    /// Chest slot (armor) - slot 6
    Chest = 6,

    /// Inventory grid slots (7-46, 10×4 = 40 slots)
    Inventory(u8), // 0-39 index

    /// Belt slots (47-54, 8 slots for potions/scrolls)
    Belt(u8), // 0-7 index
}

impl InventorySlot {
    /// Convert slot to index (0-54)
    pub fn to_index(self) -> u8 {
        match self {
            Self::Head => 0,
            Self::RingLeft => 1,
            Self::RingRight => 2,
            Self::Amulet => 3,
            Self::HandLeft => 4,
            Self::HandRight => 5,
            Self::Chest => 6,
            Self::Inventory(idx) => {
                debug_assert!(idx < INVENTORY_SIZE as u8, "Invalid inventory index: {}", idx);
                7 + idx
            }
            Self::Belt(idx) => {
                debug_assert!(idx < NUM_BELT_SLOTS as u8, "Invalid belt index: {}", idx);
                47 + idx
            }
        }
    }

    /// Create from index (0-54)
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::Head),
            1 => Some(Self::RingLeft),
            2 => Some(Self::RingRight),
            3 => Some(Self::Amulet),
            4 => Some(Self::HandLeft),
            5 => Some(Self::HandRight),
            6 => Some(Self::Chest),
            7..=46 => Some(Self::Inventory(index - 7)),
            47..=54 => Some(Self::Belt(index - 47)),
            _ => None,
        }
    }

    /// Check if this is an equipment slot
    pub fn is_equipment(&self) -> bool {
        matches!(self,
            Self::Head | Self::RingLeft | Self::RingRight |
            Self::Amulet | Self::HandLeft | Self::HandRight | Self::Chest
        )
    }

    /// Check if this is an inventory slot
    pub fn is_inventory(&self) -> bool {
        matches!(self, Self::Inventory(_))
    }

    /// Check if this is a belt slot
    pub fn is_belt(&self) -> bool {
        matches!(self, Self::Belt(_))
    }
}

/// Equipment slot location
///
/// C++ equivalent: inv_body_loc in Source/inv.h (implicit from INVITEM_* enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EquipSlot {
    Head = 0,
    RingLeft = 1,
    RingRight = 2,
    Amulet = 3,
    HandLeft = 4,
    HandRight = 5,
    Chest = 6,
}

impl EquipSlot {
    pub fn to_inventory_slot(self) -> InventorySlot {
        match self {
            Self::Head => InventorySlot::Head,
            Self::RingLeft => InventorySlot::RingLeft,
            Self::RingRight => InventorySlot::RingRight,
            Self::Amulet => InventorySlot::Amulet,
            Self::HandLeft => InventorySlot::HandLeft,
            Self::HandRight => InventorySlot::HandRight,
            Self::Chest => InventorySlot::Chest,
        }
    }
}

/// Player equipment (7 slots)
///
/// C++ equivalent: Player::InvBody[NUM_INVLOC] in Source/player.h:216
#[derive(Debug, Clone)]
pub struct Equipment {
    /// Helmet
    pub head: Option<Item>,

    /// Left ring
    pub ring_left: Option<Item>,

    /// Right ring
    pub ring_right: Option<Item>,

    /// Amulet
    pub amulet: Option<Item>,

    /// Left hand (weapon or shield)
    pub hand_left: Option<Item>,

    /// Right hand (weapon or shield)
    pub hand_right: Option<Item>,

    /// Chest armor
    pub chest: Option<Item>,
}

impl Equipment {
    /// Create new empty equipment
    pub fn new() -> Self {
        Self {
            head: None,
            ring_left: None,
            ring_right: None,
            amulet: None,
            hand_left: None,
            hand_right: None,
            chest: None,
        }
    }

    /// Get item at equipment slot
    pub fn get(&self, slot: EquipSlot) -> Option<&Item> {
        match slot {
            EquipSlot::Head => self.head.as_ref(),
            EquipSlot::RingLeft => self.ring_left.as_ref(),
            EquipSlot::RingRight => self.ring_right.as_ref(),
            EquipSlot::Amulet => self.amulet.as_ref(),
            EquipSlot::HandLeft => self.hand_left.as_ref(),
            EquipSlot::HandRight => self.hand_right.as_ref(),
            EquipSlot::Chest => self.chest.as_ref(),
        }
    }

    /// Get mutable item at equipment slot
    pub fn get_mut(&mut self, slot: EquipSlot) -> &mut Option<Item> {
        match slot {
            EquipSlot::Head => &mut self.head,
            EquipSlot::RingLeft => &mut self.ring_left,
            EquipSlot::RingRight => &mut self.ring_right,
            EquipSlot::Amulet => &mut self.amulet,
            EquipSlot::HandLeft => &mut self.hand_left,
            EquipSlot::HandRight => &mut self.hand_right,
            EquipSlot::Chest => &mut self.chest,
        }
    }

    /// Check if equipment slot is empty
    pub fn is_empty(&self, slot: EquipSlot) -> bool {
        self.get(slot).is_none()
    }

    /// Check if all equipment slots are empty
    pub fn is_all_empty(&self) -> bool {
        self.head.is_none()
            && self.ring_left.is_none()
            && self.ring_right.is_none()
            && self.amulet.is_none()
            && self.hand_left.is_none()
            && self.hand_right.is_none()
            && self.chest.is_none()
    }

    /// Clear all equipment
    pub fn clear(&mut self) {
        self.head = None;
        self.ring_left = None;
        self.ring_right = None;
        self.amulet = None;
        self.hand_left = None;
        self.hand_right = None;
        self.chest = None;
    }
}

impl Default for Equipment {
    fn default() -> Self {
        Self::new()
    }
}

/// Player inventory (10×4 grid = 40 slots)
///
/// C++ equivalent: Player::InvList[InventoryGridCells] + Player::InvGrid[InventoryGridCells]
/// in Source/player.h:217-218
#[derive(Debug, Clone)]
pub struct Inventory {
    /// Item storage (indexed by item index, not grid position)
    /// C++ equivalent: Player::InvList[InventoryGridCells]
    pub items: Vec<Option<Item>>,

    /// Grid occupation map (10×4 = 40 cells)
    /// -1 = empty, >=0 = index into items array
    /// C++ equivalent: Player::InvGrid[InventoryGridCells]
    pub grid: [i8; INVENTORY_SIZE],

    /// Number of items in inventory
    /// C++ equivalent: Player::_pNumInv
    pub count: usize,
}

impl Inventory {
    /// Create new empty inventory
    pub fn new() -> Self {
        Self {
            items: Vec::with_capacity(INVENTORY_SIZE),
            grid: [-1; INVENTORY_SIZE],
            count: 0,
        }
    }

    /// Check if inventory is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Check if inventory is full (no empty slots)
    pub fn is_full(&self) -> bool {
        self.grid.iter().all(|&cell| cell >= 0)
    }

    /// Get item at grid position (x, y)
    pub fn get_at(&self, x: usize, y: usize) -> Option<&Item> {
        if x >= INVENTORY_WIDTH || y >= INVENTORY_HEIGHT {
            return None;
        }

        let grid_index = y * INVENTORY_WIDTH + x;
        let item_index = self.grid[grid_index];

        if item_index < 0 {
            None
        } else {
            self.items.get(item_index as usize).and_then(|opt| opt.as_ref())
        }
    }

    /// Clear all inventory items
    pub fn clear(&mut self) {
        self.items.clear();
        self.grid = [-1; INVENTORY_SIZE];
        self.count = 0;
    }

    /// Get number of items
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if inventory contains an item of the specified type
    ///
    /// **C++ Reference**: Similar to `RemoveInventoryItemById()` check in Source/inv.cpp
    ///
    /// # Arguments
    /// * `item_type` - ItemType to search for
    ///
    /// # Returns
    /// `true` if an item of the specified type is found
    pub fn has_item_by_type(&self, item_type: ItemType) -> bool {
        self.items.iter()
            .filter_map(|opt| opt.as_ref())
            .any(|item| item.item_type == item_type)
    }

    /// Find and return first item of the specified type
    ///
    /// # Arguments
    /// * `item_type` - ItemType to search for
    ///
    /// # Returns
    /// Reference to the first matching item, or None
    pub fn find_item_by_type(&self, item_type: ItemType) -> Option<&Item> {
        self.items.iter()
            .filter_map(|opt| opt.as_ref())
            .find(|item| item.item_type == item_type)
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

/// Player belt (8 slots for potions/scrolls)
///
/// C++ equivalent: Player::SpdList[MaxBeltItems] in Source/player.h:218
#[derive(Debug, Clone)]
pub struct Belt {
    /// Belt item slots (8 slots)
    pub items: [Option<Item>; NUM_BELT_SLOTS],
}

impl Belt {
    /// Create new empty belt
    pub fn new() -> Self {
        Self {
            items: [None, None, None, None, None, None, None, None],
        }
    }

    /// Get item at belt slot
    pub fn get(&self, slot: u8) -> Option<&Item> {
        if slot >= NUM_BELT_SLOTS as u8 {
            return None;
        }
        self.items[slot as usize].as_ref()
    }

    /// Get mutable item at belt slot
    pub fn get_mut(&mut self, slot: u8) -> Option<&mut Item> {
        if slot >= NUM_BELT_SLOTS as u8 {
            return None;
        }
        self.items[slot as usize].as_mut()
    }

    /// Check if belt slot is empty
    pub fn is_empty(&self, slot: u8) -> bool {
        if slot >= NUM_BELT_SLOTS as u8 {
            return true;
        }
        self.items[slot as usize].is_none()
    }

    /// Check if all belt slots are empty
    pub fn is_all_empty(&self) -> bool {
        self.items.iter().all(|item| item.is_none())
    }

    /// Clear all belt items
    pub fn clear(&mut self) {
        for item in &mut self.items {
            *item = None;
        }
    }

    /// Find first empty belt slot
    pub fn find_empty_slot(&self) -> Option<u8> {
        self.items.iter()
            .position(|item| item.is_none())
            .map(|idx| idx as u8)
    }

    /// Count number of items in belt
    pub fn count(&self) -> usize {
        self.items.iter().filter(|item| item.is_some()).count()
    }
}

impl Default for Belt {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Day 94: Equip/Unequip Logic
// ============================================================================
// - EquipError enum (装备错误类型)
// - ItemLocation enum (物品装备位置 ILOC_*)
// - can_equip() - 装备条件检查
// - equip() / unequip() - 装备/卸下物品
// - auto_equip() - 自动装备最佳槽位
//
// C++ Reference: Source/inv.cpp:164-290, Source/itemdat.h:99-108

/// Item equipment location type
///
/// C++ equivalent: item_equip_type in Source/itemdat.h:98-109
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum ItemLocation {
    None = 0,
    OneHand = 1,
    TwoHand = 2,
    Armor = 3,
    Helm = 4,
    Ring = 5,
    Amulet = 6,
    Unequipable = 7,
    Belt = 8,
    Invalid = -1,
}

impl ItemLocation {
    /// Check if item can be equipped
    pub fn is_equipable(self) -> bool {
        matches!(self,
            Self::OneHand | Self::TwoHand | Self::Armor |
            Self::Helm | Self::Ring | Self::Amulet
        )
    }

    /// Check if item is a weapon
    pub fn is_weapon(self) -> bool {
        matches!(self, Self::OneHand | Self::TwoHand)
    }
}

/// Equipment error types
///
/// Represents reasons why an item cannot be equipped
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquipError {
    /// Equipment slot is already occupied
    SlotOccupied,

    /// Item cannot be equipped in this slot
    InvalidSlot,

    /// Player class cannot use this item type
    ClassRestriction,

    /// Insufficient strength
    StrengthRequired(u8),

    /// Insufficient magic
    MagicRequired(u8),

    /// Insufficient dexterity
    DexterityRequired(u8),

    /// Insufficient level
    LevelRequired(u8),

    /// Two-handed weapon conflicts with occupied hand
    TwoHandedConflict,

    /// Both hands occupied, cannot dual wield
    BothHandsOccupied,

    /// Item requirements not met (generic)
    RequirementsNotMet,

    /// Inventory full, cannot move displaced item
    InventoryFull,
}

impl std::fmt::Display for EquipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SlotOccupied => write!(f, "Equipment slot is already occupied"),
            Self::InvalidSlot => write!(f, "Item cannot be equipped in this slot"),
            Self::ClassRestriction => write!(f, "Your class cannot use this item"),
            Self::StrengthRequired(str) => write!(f, "Requires {} Strength", str),
            Self::MagicRequired(mag) => write!(f, "Requires {} Magic", mag),
            Self::DexterityRequired(dex) => write!(f, "Requires {} Dexterity", dex),
            Self::LevelRequired(lvl) => write!(f, "Requires Level {}", lvl),
            Self::TwoHandedConflict => write!(f, "Cannot equip two-handed weapon with occupied hand"),
            Self::BothHandsOccupied => write!(f, "Both hands are occupied"),
            Self::RequirementsNotMet => write!(f, "Item requirements not met"),
            Self::InventoryFull => write!(f, "Inventory full, cannot move displaced item"),
        }
    }
}

impl std::error::Error for EquipError {}

/// Player stats for equipment checks
///
/// Simplified version of Player struct for equipment validation
#[derive(Debug, Clone)]
pub struct PlayerStats {
    pub class: HeroClass,
    pub strength: u8,
    pub magic: u8,
    pub dexterity: u8,
    pub level: u8,
}

impl PlayerStats {
    pub fn new(class: HeroClass, strength: u8, magic: u8, dexterity: u8, level: u8) -> Self {
        Self {
            class,
            strength,
            magic,
            dexterity,
            level,
        }
    }
}

impl Equipment {
    /// Check if an item can be equipped in a specific slot
    ///
    /// C++ equivalent: CanEquip(Player &player, const Item &item, inv_body_loc bodyLocation)
    /// in Source/inv.cpp:244-280
    ///
    /// # Arguments
    /// * `player_stats` - Player stats for requirement checks
    /// * `item` - Item to check
    /// * `slot` - Target equipment slot
    pub fn can_equip(&self, player_stats: &PlayerStats, item: &Item, slot: EquipSlot) -> Result<(), EquipError> {
        // Check if slot is occupied
        if !self.is_empty(slot) {
            return Err(EquipError::SlotOccupied);
        }

        // Check basic requirements
        Self::check_item_requirements(player_stats, item)?;

        // Get item location type
        let item_loc = Self::get_item_location(item);

        // Check slot compatibility
        match slot {
            EquipSlot::Head => {
                if item_loc != ItemLocation::Helm {
                    return Err(EquipError::InvalidSlot);
                }
            }
            EquipSlot::Chest => {
                if item_loc != ItemLocation::Armor {
                    return Err(EquipError::InvalidSlot);
                }
            }
            EquipSlot::Amulet => {
                if item_loc != ItemLocation::Amulet {
                    return Err(EquipError::InvalidSlot);
                }
            }
            EquipSlot::RingLeft | EquipSlot::RingRight => {
                if item_loc != ItemLocation::Ring {
                    return Err(EquipError::InvalidSlot);
                }
            }
            EquipSlot::HandLeft | EquipSlot::HandRight => {
                self.can_wield(player_stats, item, slot)?;
            }
        }

        Ok(())
    }

    /// Check if item can be wielded in hand
    ///
    /// C++ equivalent: CanWield(Player &player, const Item &item)
    /// in Source/inv.cpp:195-242
    fn can_wield(&self, player_stats: &PlayerStats, item: &Item, target_slot: EquipSlot) -> Result<(), EquipError> {
        let item_loc = Self::get_item_location(item);

        // Only weapons can be wielded
        if !item_loc.is_weapon() {
            return Err(EquipError::InvalidSlot);
        }

        let left_hand = &self.hand_left;
        let right_hand = &self.hand_right;

        // Both hands empty - always OK
        if left_hand.is_none() && right_hand.is_none() {
            return Ok(());
        }

        // Both hands occupied - cannot equip
        if left_hand.is_some() && right_hand.is_some() {
            return Err(EquipError::BothHandsOccupied);
        }

        // One hand occupied
        let occupied_item = left_hand.as_ref().or(right_hand.as_ref()).unwrap();
        let occupied_loc = Self::get_item_location(occupied_item);

        // Two-handed weapon requires both hands
        if item_loc == ItemLocation::TwoHand {
            return Err(EquipError::TwoHandedConflict);
        }

        // Occupied hand has two-handed weapon
        if occupied_loc == ItemLocation::TwoHand {
            return Err(EquipError::TwoHandedConflict);
        }

        // Bard can dual wield swords and maces
        if player_stats.class == HeroClass::Bard {
            let occupied_is_sword_or_mace = matches!(
                occupied_item.item_type,
                ItemType::Sword | ItemType::Mace
            ) && occupied_loc == ItemLocation::OneHand;

            let equip_is_sword_or_mace = matches!(
                item.item_type,
                ItemType::Sword | ItemType::Mace
            ) && item_loc == ItemLocation::OneHand;

            if occupied_is_sword_or_mace && equip_is_sword_or_mace {
                return Ok(());
            }
        }

        // Both one-handed, different classes - can dual wield
        if item_loc == ItemLocation::OneHand && occupied_loc == ItemLocation::OneHand {
            // Check if different weapon/shield classes
            let item_is_weapon = matches!(
                item.item_type,
                ItemType::Sword | ItemType::Axe | ItemType::Mace | ItemType::Bow | ItemType::Staff
            );
            let occupied_is_weapon = matches!(
                occupied_item.item_type,
                ItemType::Sword | ItemType::Axe | ItemType::Mace | ItemType::Bow | ItemType::Staff
            );

            // Can equip if one is weapon and other is shield, or both are weapons (for specific classes)
            if item_is_weapon != occupied_is_weapon {
                return Ok(());
            }
        }

        Err(EquipError::TwoHandedConflict)
    }

    /// Check item stat requirements
    ///
    /// C++ equivalent: CanEquip(const Item &item) in Source/inv.cpp:186-193
    fn check_item_requirements(player_stats: &PlayerStats, item: &Item) -> Result<(), EquipError> {
        // Check strength requirement
        if player_stats.strength < item.required_str as u8 {
            return Err(EquipError::StrengthRequired(item.required_str as u8));
        }

        // Check magic requirement
        if player_stats.magic < item.required_mag as u8 {
            return Err(EquipError::MagicRequired(item.required_mag as u8));
        }

        // Check dexterity requirement
        if player_stats.dexterity < item.required_dex as u8 {
            return Err(EquipError::DexterityRequired(item.required_dex as u8));
        }

        // Check level requirement
        if player_stats.level < item.required_level as u8 {
            return Err(EquipError::LevelRequired(item.required_level as u8));
        }

        Ok(())
    }

    /// Get item equipment location type
    ///
    /// Maps ItemType to ItemLocation (simplified version)
    fn get_item_location(item: &Item) -> ItemLocation {
        match item.item_type {
            ItemType::Sword | ItemType::Axe | ItemType::Mace | ItemType::Dagger => ItemLocation::OneHand,
            ItemType::Bow | ItemType::Staff => ItemLocation::TwoHand,
            ItemType::Shield => ItemLocation::OneHand,
            ItemType::Armor => ItemLocation::Armor,
            ItemType::Helm => ItemLocation::Helm,
            ItemType::Ring => ItemLocation::Ring,
            ItemType::Amulet => ItemLocation::Amulet,
            ItemType::HealthPotion | ItemType::ManaPotion => ItemLocation::Belt,
            _ => ItemLocation::Unequipable,
        }
    }

    /// Equip an item to a specific slot
    ///
    /// C++ equivalent: ChangeEquipment() + AutoEquip() in Source/inv.cpp:282-311
    ///
    /// # Arguments
    /// * `player_stats` - Player stats for requirement checks
    /// * `item` - Item to equip
    /// * `slot` - Target equipment slot
    ///
    /// # Returns
    /// * `Ok(Some(Item))` - Previously equipped item (if any)
    /// * `Ok(None)` - Slot was empty, item equipped successfully
    /// * `Err(EquipError)` - Cannot equip item
    pub fn equip(&mut self, player_stats: &PlayerStats, item: Item, slot: EquipSlot) -> Result<Option<Item>, EquipError> {
        // Validate can equip
        self.can_equip(player_stats, &item, slot)?;

        // Get mutable reference to target slot
        let slot_ref = self.get_mut(slot);

        // Replace item in slot
        Ok(slot_ref.replace(item))
    }

    /// Unequip an item from a specific slot
    ///
    /// C++ equivalent: RemoveEquipment() in Source/inv.h:112-118
    ///
    /// # Arguments
    /// * `slot` - Equipment slot to unequip
    ///
    /// # Returns
    /// * `Some(Item)` - The unequipped item
    /// * `None` - Slot was already empty
    pub fn unequip(&mut self, slot: EquipSlot) -> Option<Item> {
        self.get_mut(slot).take()
    }

    /// Auto-equip item to best available slot
    ///
    /// C++ equivalent: AutoEquip(Player &player, const Item &item, bool persistItem, bool sendNetworkMessage)
    /// in Source/inv.cpp:289-311
    ///
    /// Tries to find the best equipment slot for the item and equips it.
    /// For rings, prefers empty slot or replaces weaker ring.
    /// For weapons, tries both hands.
    ///
    /// # Arguments
    /// * `player_stats` - Player stats for requirement checks
    /// * `item` - Item to auto-equip
    ///
    /// # Returns
    /// * `Ok(EquipSlot)` - Slot where item was equipped
    /// * `Err(EquipError)` - Cannot equip item anywhere
    pub fn auto_equip(&mut self, player_stats: &PlayerStats, item: Item) -> Result<EquipSlot, EquipError> {
        let item_loc = Self::get_item_location(&item);

        // Determine primary slot(s) based on item type
        let candidate_slots: Vec<EquipSlot> = match item_loc {
            ItemLocation::Helm => vec![EquipSlot::Head],
            ItemLocation::Armor => vec![EquipSlot::Chest],
            ItemLocation::Amulet => vec![EquipSlot::Amulet],
            ItemLocation::Ring => vec![EquipSlot::RingLeft, EquipSlot::RingRight],
            ItemLocation::OneHand | ItemLocation::TwoHand => vec![EquipSlot::HandRight, EquipSlot::HandLeft],
            _ => return Err(EquipError::InvalidSlot),
        };

        // Try each candidate slot
        for slot in candidate_slots {
            if self.can_equip(player_stats, &item, slot).is_ok() {
                // Equip to this slot
                self.equip(player_stats, item, slot)?;
                return Ok(slot);
            }
        }

        // No suitable slot found
        Err(EquipError::SlotOccupied)
    }

    /// Change to two-hand weapon, handling existing equipment
    ///
    /// C++ equivalent: ChangeTwoHandItem() in Source/inv.cpp:388-418
    ///
    /// When equipping a two-hand weapon:
    /// 1. If both hands occupied, try to move one item to inventory
    /// 2. Prefer moving shield to inventory over weapon
    /// 3. Equip two-hand weapon to left hand
    /// 4. Return previously equipped item (if any)
    ///
    /// # Arguments
    /// * `inventory` - Player's inventory for auto-placing displaced items
    /// * `held_item` - Two-hand weapon to equip
    ///
    /// # Returns
    /// * `Ok(Option<Item>)` - Previously equipped item (to hold in cursor)
    /// * `Err(EquipError)` - Cannot equip (inventory full, etc.)
    pub fn change_two_hand_item(
        &mut self,
        inventory: &mut Inventory,
        held_item: Item,
    ) -> Result<Option<Item>, EquipError> {
        let left_occupied = self.hand_left.is_some();
        let right_occupied = self.hand_right.is_some();

        // If both hands occupied, need to move one item to inventory
        if left_occupied && right_occupied {
            // Prefer moving shield to inventory (right hand usually has shield)
            let location_to_unequip = if self.hand_right.as_ref()
                .map(|i| i.item_type == ItemType::Shield)
                .unwrap_or(false)
            {
                EquipSlot::HandRight
            } else {
                EquipSlot::HandLeft
            };

            // Get the item to move to inventory
            let item_to_move = self.get(location_to_unequip).cloned();
            if let Some(item) = item_to_move {
                // Try to auto-place in inventory
                // Assume 1x2 size for weapons, 2x2 for shields
                let size = if item.item_type == ItemType::Shield {
                    ItemSize::new(2, 2)
                } else {
                    ItemSize::new(1, 2)
                };

                if inventory.auto_place(item.clone(), size).is_err() {
                    return Err(EquipError::InventoryFull);
                }

                // Remove from equipment
                self.get_mut(location_to_unequip).take();
            }
        }

        // Now equip the two-hand weapon
        if self.hand_right.is_none() {
            // Right hand empty, equip to left hand and return previous left item
            let previous = self.hand_left.replace(held_item);
            Ok(previous)
        } else {
            // Right hand has item, need to handle it
            let previous = self.hand_right.take();
            self.hand_left = Some(held_item);
            Ok(previous)
        }
    }
}

// ============================================================================
// Day 95: Auto-Placement System
// ============================================================================
// - Item size types (1x1, 1x2, 2x2, 2x3)
// - can_fit() - 检查物品是否能放入背包
// - find_empty_slot() - 查找物品的空位
// - add_item() / remove_item() - 添加/移除物品
// - auto_place_in_belt() - 腰带自动放置
// - auto_place_gold() - 金币智能堆叠
//
// C++ Reference: Source/inv.cpp:660-737, 1323-1537

/// Item size in inventory grid cells (width × height)
///
/// C++ equivalent: Size in Source/engine/size.hpp
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ItemSize {
    pub width: u8,
    pub height: u8,
}

impl ItemSize {
    pub const fn new(width: u8, height: u8) -> Self {
        Self { width, height }
    }
}

impl Inventory {
    /// Check if item can fit in inventory
    ///
    /// C++ equivalent: CheckItemFitsInInventorySlot in Source/inv.cpp:659-679
    ///
    /// # Arguments
    /// * `slot_index` - Starting grid position (0-39)
    /// * `item_size` - Item dimensions (width × height)
    ///
    /// # Returns
    /// True if item fits, false otherwise
    fn can_fit_at_slot(&self, slot_index: usize, item_size: ItemSize) -> bool {
        if slot_index >= INVENTORY_SIZE {
            return false;
        }

        let start_x = slot_index % INVENTORY_WIDTH;
        let start_y = slot_index / INVENTORY_WIDTH;

        // Check if item extends beyond grid boundaries
        if start_x + item_size.width as usize > INVENTORY_WIDTH {
            return false;
        }
        if start_y + item_size.height as usize > INVENTORY_HEIGHT {
            return false;
        }

        // Check if all required cells are empty
        for dy in 0..item_size.height as usize {
            for dx in 0..item_size.width as usize {
                let grid_idx = (start_y + dy) * INVENTORY_WIDTH + (start_x + dx);
                if self.grid[grid_idx] >= 0 {
                    // Cell occupied
                    return false;
                }
            }
        }

        true
    }

    /// Find first available slot for item of given size
    ///
    /// C++ equivalent: FindSlotForItem in Source/inv.cpp:687-737
    ///
    /// # Arguments
    /// * `item_size` - Item dimensions
    ///
    /// # Returns
    /// * `Some(slot_index)` - First available slot (0-39)
    /// * `None` - No space available
    ///
    /// # Placement Priority
    /// 1. Height 1: Last row (30-39) → Columns 9→0, Rows 2→0
    /// 2. Height 2: Columns (10-width)→0, Rows 0→2
    /// 3. Size 1×3: Slots 0-19
    /// 4. Size 2×3: Slots 0-8, 10-18
    pub fn find_empty_slot(&self, item_size: ItemSize) -> Option<usize> {
        match item_size.height {
            // Height 1 items: Last row first, then bottom-to-top, right-to-left
            1 => {
                // Last row (30-39)
                for i in 30..=39 {
                    if self.can_fit_at_slot(i, item_size) {
                        return Some(i);
                    }
                }
                // Remaining rows: columns 9→0, rows 2→0
                for x in (0..INVENTORY_WIDTH).rev() {
                    for y in (0..3).rev() {
                        let slot = y * INVENTORY_WIDTH + x;
                        if self.can_fit_at_slot(slot, item_size) {
                            return Some(slot);
                        }
                    }
                }
                None
            }

            // Height 2 items: Left-to-right, top-to-bottom
            2 => {
                let max_x = INVENTORY_WIDTH.saturating_sub(item_size.width as usize);
                for x in (0..=max_x).rev() {
                    for y in 0..3 {
                        let slot = y * INVENTORY_WIDTH + x;
                        if self.can_fit_at_slot(slot, item_size) {
                            return Some(slot);
                        }
                    }
                }
                None
            }

            // Height 3 items
            3 => {
                if item_size.width == 1 {
                    // 1×3 items: First 20 slots
                    for i in 0..20 {
                        if self.can_fit_at_slot(i, item_size) {
                            return Some(i);
                        }
                    }
                } else if item_size.width == 2 {
                    // 2×3 items: Slots 0-8, 10-18
                    for i in 0..=8 {
                        if self.can_fit_at_slot(i, item_size) {
                            return Some(i);
                        }
                    }
                    for i in 10..=18 {
                        if self.can_fit_at_slot(i, item_size) {
                            return Some(i);
                        }
                    }
                }
                None
            }

            _ => None, // Invalid height
        }
    }

    /// Add item to inventory at specific slot
    ///
    /// C++ equivalent: AddItemToInvGrid in Source/inv.cpp:150-167
    ///
    /// # Arguments
    /// * `item` - Item to add
    /// * `slot_index` - Grid position (0-39)
    /// * `item_size` - Item dimensions
    ///
    /// # Returns
    /// * `Ok(())` - Item added successfully
    /// * `Err(&str)` - Error message
    pub fn add_item(&mut self, item: Item, slot_index: usize, item_size: ItemSize) -> Result<(), &'static str> {
        if !self.can_fit_at_slot(slot_index, item_size) {
            return Err("Item does not fit at specified slot");
        }

        // Add item to items vector
        let item_index = self.items.len();
        self.items.push(Some(item));
        self.count += 1;

        // Mark grid cells (bottom-left cell stores item index, others store negative index)
        let start_x = slot_index % INVENTORY_WIDTH;
        let start_y = slot_index / INVENTORY_WIDTH;

        for dy in 0..item_size.height as usize {
            for dx in 0..item_size.width as usize {
                let grid_idx = (start_y + dy) * INVENTORY_WIDTH + (start_x + dx);

                // Bottom-left cell (highest y, lowest x) stores positive index
                if dy == (item_size.height - 1) as usize && dx == 0 {
                    self.grid[grid_idx] = item_index as i8;
                } else {
                    // Other cells store negative index
                    self.grid[grid_idx] = -(item_index as i8 + 1);
                }
            }
        }

        Ok(())
    }

    /// Remove item from inventory at grid position
    ///
    /// C++ equivalent: RemoveInvItem in Source/inv.cpp (partial)
    ///
    /// # Arguments
    /// * `slot_index` - Grid position (0-39)
    ///
    /// # Returns
    /// * `Some(Item)` - Removed item
    /// * `None` - No item at slot
    pub fn remove_item(&mut self, slot_index: usize) -> Option<Item> {
        if slot_index >= INVENTORY_SIZE {
            return None;
        }

        let grid_val = self.grid[slot_index];
        if grid_val < 0 {
            return None; // Empty slot or secondary cell
        }

        let item_index = grid_val as usize;
        let item = self.items.get_mut(item_index)?.take()?;

        // Clear grid cells occupied by this item
        for i in 0..INVENTORY_SIZE {
            if self.grid[i].abs() - 1 == item_index as i8 {
                self.grid[i] = -1;
            }
        }

        self.count -= 1;
        Some(item)
    }

    /// Auto-place item in inventory (find best slot automatically)
    ///
    /// C++ equivalent: AutoPlaceItemInInventory in Source/inv.cpp:1395-1409
    ///
    /// # Arguments
    /// * `item` - Item to add
    /// * `item_size` - Item dimensions
    ///
    /// # Returns
    /// * `Ok(slot_index)` - Slot where item was placed
    /// * `Err(&str)` - No space available
    pub fn auto_place(&mut self, item: Item, item_size: ItemSize) -> Result<usize, &'static str> {
        if let Some(slot) = self.find_empty_slot(item_size) {
            self.add_item(item, slot, item_size)?;
            Ok(slot)
        } else {
            Err("Inventory full")
        }
    }
}

/// Maximum gold per stack
pub const MAX_GOLD_PER_STACK: i32 = 5000;

impl Belt {
    /// Auto-place item in belt (find first empty slot)
    ///
    /// C++ equivalent: AutoPlaceItemInBelt in Source/inv.cpp:1323-1345
    ///
    /// # Arguments
    /// * `item` - Item to add (must be 1×1 size)
    ///
    /// # Returns
    /// * `Ok(slot)` - Belt slot where item was placed (0-7)
    /// * `Err(&str)` - Belt full or item doesn't fit
    pub fn auto_place(&mut self, item: Item) -> Result<u8, &'static str> {
        // Find first empty slot
        if let Some(slot) = self.find_empty_slot() {
            self.items[slot as usize] = Some(item);
            Ok(slot)
        } else {
            Err("Belt full")
        }
    }
}

/// Auto-place gold in inventory (stack with existing gold or create new stacks)
///
/// C++ equivalent: AddGoldToInventory in Source/inv.cpp:1489-1526
///
/// # Arguments
/// * `inventory` - Player inventory
/// * `gold_amount` - Amount of gold to add
///
/// # Returns
/// Remaining gold that couldn't be placed (0 if all placed successfully)
///
/// # Placement Strategy
/// 1. Top off existing gold stacks (max 5000 per stack)
/// 2. Last row (slots 39→30), right to left
/// 3. Remaining slots in columns, bottom-to-top, right-to-left (column 9→0, row 2→0)
pub fn auto_place_gold(inventory: &mut Inventory, mut gold_amount: i32) -> i32 {
    // Step 1: Top off existing gold stacks
    for item_opt in &mut inventory.items {
        if gold_amount <= 0 {
            break;
        }

        if let Some(item) = item_opt {
            if item.item_type == ItemType::Gold && item.quantity < MAX_GOLD_PER_STACK {
                let space_left = MAX_GOLD_PER_STACK - item.quantity;
                if gold_amount <= space_left {
                    item.quantity += gold_amount;
                    gold_amount = 0;
                } else {
                    item.quantity = MAX_GOLD_PER_STACK;
                    gold_amount -= space_left;
                }
            }
        }
    }

    // Step 2: Last row (39→30)
    for i in (30..=39).rev() {
        if gold_amount <= 0 {
            break;
        }

        if inventory.grid[i] == -1 {
            let amount_to_place = gold_amount.min(MAX_GOLD_PER_STACK);
            let mut gold_item = Item::new("Gold".to_string(), ItemType::Gold, amount_to_place);
            gold_item.quantity = amount_to_place;
            gold_item.max_stack = MAX_GOLD_PER_STACK;

            if inventory.add_item(gold_item, i, ItemSize::new(1, 1)).is_ok() {
                gold_amount -= amount_to_place;
            }
        }
    }

    // Step 3: Remaining inventory (columns 9→0, rows 2→0)
    for x in (0..INVENTORY_WIDTH).rev() {
        for y in (0..3).rev() {
            if gold_amount <= 0 {
                break;
            }

            let slot = y * INVENTORY_WIDTH + x;
            if inventory.grid[slot] == -1 {
                let amount_to_place = gold_amount.min(MAX_GOLD_PER_STACK);
                let mut gold_item = Item::new("Gold".to_string(), ItemType::Gold, amount_to_place);
                gold_item.quantity = amount_to_place;
                gold_item.max_stack = MAX_GOLD_PER_STACK;

                if inventory.add_item(gold_item, slot, ItemSize::new(1, 1)).is_ok() {
                    gold_amount -= amount_to_place;
                }
            }
        }
    }

    gold_amount
}

// ============================================================================
// Day 96: Item Usage + Integration
// ============================================================================
// - Item usage errors
// - use_potion() - HP/Mana recovery
// - use_scroll() - Spell scrolls (framework)
// - use_staff() - Staff charge consumption (framework)
// - Quest item integration hooks
//
// C++ Reference: Source/items.cpp:4190-4300 (UseItem), Source/inv.cpp:2035-2100

/// Represents reasons why an item cannot be used
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UseItemError {
    /// Item type cannot be used
    NotUsable,

    /// Item is not identified
    NotIdentified,

    /// Player requirements not met (level, class, etc.)
    RequirementsNotMet,

    /// Cannot use in current location (e.g., town)
    LocationRestricted,

    /// Staff has no charges left
    NoChargesLeft,

    /// Item not found in inventory
    ItemNotFound,

    /// Generic error with message
    Other(String),
}

impl std::fmt::Display for UseItemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UseItemError::NotUsable => write!(f, "Item cannot be used"),
            UseItemError::NotIdentified => write!(f, "Item must be identified first"),
            UseItemError::RequirementsNotMet => write!(f, "Requirements not met"),
            UseItemError::LocationRestricted => write!(f, "Cannot use here"),
            UseItemError::NoChargesLeft => write!(f, "No charges left"),
            UseItemError::ItemNotFound => write!(f, "Item not found"),
            UseItemError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for UseItemError {}

/// Item usage result
pub type UseItemResult = Result<ItemUseEffect, UseItemError>;

/// Effects from using an item
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemUseEffect {
    /// Item consumed (removed from inventory)
    Consumed,

    /// HP restored
    HealedHP(i32),

    /// Mana restored
    RestoredMana(i32),

    /// HP and Mana restored
    Rejuvenated(i32, i32),

    /// Staff charge consumed (charges_remaining)
    ChargeConsumed(i32),

    /// Scroll used (spell_id)
    ScrollUsed(u8),

    /// Book read (spell_level_increased)
    BookRead(u8),

    /// Quest item activated
    QuestItemUsed(String),

    /// No effect (item not consumed)
    NoEffect,
}

/// Use a potion (Health, Mana, Rejuvenation)
///
/// C++ equivalent: UseItem(player, mid, spellID, spellFrom) in Source/items.cpp:4196-4260
///
/// # Arguments
/// * `item` - Potion item
///
/// # Returns
/// * `Ok(ItemUseEffect)` - Effect applied
/// * `Err(UseItemError)` - Cannot use item
///
/// # Supported Potions
/// - HealthPotion: Restore 50% HP
/// - ManaPotion: Restore 50% Mana
/// - FullRejuvenation: Restore 100% HP and Mana
pub fn use_potion(item: &Item) -> UseItemResult {
    match item.item_type {
        ItemType::HealthPotion => {
            // C++ IMISC_HEAL: player.RestorePartialLife()
            // Simplified: Return effect, caller handles HP restoration
            Ok(ItemUseEffect::HealedHP(50)) // 50% heal (placeholder)
        }
        ItemType::ManaPotion => {
            // C++ IMISC_MANA: player.RestorePartialMana()
            Ok(ItemUseEffect::RestoredMana(50)) // 50% mana restore
        }
        ItemType::FullRejuvenation => {
            // C++ IMISC_FULLREJUV: Full HP + Mana restore
            Ok(ItemUseEffect::Rejuvenated(100, 100))
        }
        ItemType::Elixir => {
            // C++ IMISC_ELIXSTR/MAG/DEX/VIT: Permanent stat increase
            // Not implemented in simplified version
            Ok(ItemUseEffect::Consumed)
        }
        _ => Err(UseItemError::NotUsable),
    }
}

/// Use a scroll (spell scroll, Town Portal, Identify)
///
/// C++ equivalent: UseItem(player, IMISC_SCROLL, spellID, spellFrom) in Source/items.cpp:4270-4285
///
/// # Arguments
/// * `item` - Scroll item
/// * `spell_id` - Spell ID to cast
///
/// # Returns
/// * `Ok(ItemUseEffect::ScrollUsed)` - Scroll consumed, spell activated
/// * `Err(UseItemError)` - Cannot use scroll
///
/// # Notes
/// - Town restriction checking deferred to caller
/// - Actual spell casting deferred to spell system
pub fn use_scroll(item: &Item, spell_id: u8) -> UseItemResult {
    if item.item_type != ItemType::Scroll {
        return Err(UseItemError::NotUsable);
    }

    // C++ logic:
    // - Check if spell allowed in town (deferred)
    // - NetSendCmdLocParam3(..., CMD_SPELLXY, ...)
    // - Cast spell at target location

    Ok(ItemUseEffect::ScrollUsed(spell_id))
}

/// Use a book (learn spell or increase spell level)
///
/// C++ equivalent: UseItem(player, IMISC_BOOK, spellID, spellFrom) in Source/items.cpp:4286-4300
///
/// # Arguments
/// * `item` - Book item
/// * `spell_id` - Spell ID to learn/upgrade
/// * `current_level` - Current spell level (0 if not learned)
///
/// # Returns
/// * `Ok(ItemUseEffect::BookRead)` - New spell level
/// * `Err(UseItemError)` - Cannot use book
pub fn use_book(item: &Item, spell_id: u8, current_level: u8) -> UseItemResult {
    if item.item_type != ItemType::Book {
        return Err(UseItemError::NotUsable);
    }

    // C++ logic:
    // const uint8_t newSpellLevel = player._pSplLvl[spellID] + 1;
    // if (newSpellLevel <= MaxSpellLevel) { ... }

    const MAX_SPELL_LEVEL: u8 = 15;
    let new_level = current_level.saturating_add(1);

    if new_level > MAX_SPELL_LEVEL {
        return Err(UseItemError::Other("Spell already at max level".to_string()));
    }

    Ok(ItemUseEffect::BookRead(new_level))
}

/// Consume staff charge when casting spell
///
/// C++ equivalent: ConsumeStaffCharge(player) in Source/inv.cpp:2046-2053
///
/// # Arguments
/// * `item` - Staff item (must be ItemType::Staff)
/// * `charges` - Current charges
///
/// # Returns
/// * `Ok(ItemUseEffect::ChargeConsumed)` - Remaining charges
/// * `Err(UseItemError::NoChargesLeft)` - Staff has no charges
pub fn use_staff_charge(item: &Item, charges: i32) -> UseItemResult {
    if item.item_type != ItemType::Staff {
        return Err(UseItemError::NotUsable);
    }

    if charges <= 0 {
        return Err(UseItemError::NoChargesLeft);
    }

    // C++ logic:
    // staff._iCharges--;
    // CalcPlrInv(player, false);

    Ok(ItemUseEffect::ChargeConsumed(charges - 1))
}

/// Check if item can be used
///
/// C++ equivalent: item->isUsable() in Source/inv.cpp:2152
///
/// # Arguments
/// * `item` - Item to check
///
/// # Returns
/// True if item is usable (potion, scroll, book, staff charge, quest item)
pub fn can_use_item(item: &Item) -> bool {
    matches!(
        item.item_type,
        ItemType::HealthPotion
            | ItemType::ManaPotion
            | ItemType::FullRejuvenation
            | ItemType::Elixir
            | ItemType::Scroll
            | ItemType::Book
            | ItemType::Staff  // Requires charges check
            | ItemType::Quest
    )
}

// ============================================================================
// Tests - Day 93 + Day 94 + Day 95 + Day 96
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test Group 1: InventorySlot enum (3 tests)

    #[test]
    fn test_inventory_slot_to_index() {
        assert_eq!(InventorySlot::Head.to_index(), 0);
        assert_eq!(InventorySlot::RingLeft.to_index(), 1);
        assert_eq!(InventorySlot::RingRight.to_index(), 2);
        assert_eq!(InventorySlot::Amulet.to_index(), 3);
        assert_eq!(InventorySlot::HandLeft.to_index(), 4);
        assert_eq!(InventorySlot::HandRight.to_index(), 5);
        assert_eq!(InventorySlot::Chest.to_index(), 6);

        assert_eq!(InventorySlot::Inventory(0).to_index(), 7);
        assert_eq!(InventorySlot::Inventory(39).to_index(), 46);

        assert_eq!(InventorySlot::Belt(0).to_index(), 47);
        assert_eq!(InventorySlot::Belt(7).to_index(), 54);
    }

    #[test]
    fn test_inventory_slot_from_index() {
        assert_eq!(InventorySlot::from_index(0), Some(InventorySlot::Head));
        assert_eq!(InventorySlot::from_index(1), Some(InventorySlot::RingLeft));
        assert_eq!(InventorySlot::from_index(6), Some(InventorySlot::Chest));

        assert_eq!(InventorySlot::from_index(7), Some(InventorySlot::Inventory(0)));
        assert_eq!(InventorySlot::from_index(46), Some(InventorySlot::Inventory(39)));

        assert_eq!(InventorySlot::from_index(47), Some(InventorySlot::Belt(0)));
        assert_eq!(InventorySlot::from_index(54), Some(InventorySlot::Belt(7)));

        assert_eq!(InventorySlot::from_index(55), None);
        assert_eq!(InventorySlot::from_index(100), None);
    }

    #[test]
    fn test_inventory_slot_type_checks() {
        assert!(InventorySlot::Head.is_equipment());
        assert!(!InventorySlot::Head.is_inventory());
        assert!(!InventorySlot::Head.is_belt());

        assert!(!InventorySlot::Inventory(0).is_equipment());
        assert!(InventorySlot::Inventory(0).is_inventory());
        assert!(!InventorySlot::Inventory(0).is_belt());

        assert!(!InventorySlot::Belt(0).is_equipment());
        assert!(!InventorySlot::Belt(0).is_inventory());
        assert!(InventorySlot::Belt(0).is_belt());
    }

    // Test Group 2: Equipment struct (2 tests)

    #[test]
    fn test_equipment_creation_and_empty() {
        let equipment = Equipment::new();

        assert!(equipment.is_all_empty());
        assert!(equipment.is_empty(EquipSlot::Head));
        assert!(equipment.is_empty(EquipSlot::RingLeft));
        assert!(equipment.is_empty(EquipSlot::HandLeft));
    }

    #[test]
    fn test_equipment_clear() {
        let mut equipment = Equipment::new();

        // Add some dummy items (using None for now as we don't have Item constructor)
        // In real usage, these would be Some(Item { ... })
        // For now, just test the clear() functionality

        equipment.clear();
        assert!(equipment.is_all_empty());
    }

    // Test Group 3: Inventory struct (3 tests)

    #[test]
    fn test_inventory_creation() {
        let inventory = Inventory::new();

        assert!(inventory.is_empty());
        assert!(!inventory.is_full());
        assert_eq!(inventory.len(), 0);
        assert_eq!(inventory.count, 0);

        // All grid cells should be -1 (empty)
        for &cell in &inventory.grid {
            assert_eq!(cell, -1);
        }
    }

    #[test]
    fn test_inventory_grid_dimensions() {
        let inventory = Inventory::new();

        // Test grid access bounds
        assert!(inventory.get_at(0, 0).is_none()); // Empty
        assert!(inventory.get_at(9, 3).is_none()); // Last valid cell, empty
        assert!(inventory.get_at(10, 0).is_none()); // Out of bounds (x)
        assert!(inventory.get_at(0, 4).is_none()); // Out of bounds (y)
    }

    #[test]
    fn test_inventory_clear() {
        let mut inventory = Inventory::new();

        inventory.clear();

        assert!(inventory.is_empty());
        assert_eq!(inventory.count, 0);
        for &cell in &inventory.grid {
            assert_eq!(cell, -1);
        }
    }

    // Test Group 4: Belt struct (2 tests)

    #[test]
    fn test_belt_creation() {
        let belt = Belt::new();

        assert!(belt.is_all_empty());
        assert_eq!(belt.count(), 0);

        for i in 0..NUM_BELT_SLOTS as u8 {
            assert!(belt.is_empty(i));
            assert!(belt.get(i).is_none());
        }
    }

    #[test]
    fn test_belt_find_empty_slot() {
        let belt = Belt::new();

        // All slots empty, should return first slot (0)
        assert_eq!(belt.find_empty_slot(), Some(0));

        // Test out of bounds
        assert!(belt.get(8).is_none());
        assert!(belt.get(100).is_none());
    }

    // ========================================================================
    // Day 94 Tests: Equip/Unequip Logic (14 tests)
    // ========================================================================

    // Test Group 5: Item requirements (3 tests)

    #[test]
    fn test_item_requirements_strength() {
        let player = PlayerStats::new(HeroClass::Warrior, 20, 10, 10, 5);

        let mut weak_item = Item::new("Short Sword".to_string(), ItemType::Sword, 100);
        weak_item.required_str = 15;

        let mut strong_item = Item::new("Great Sword".to_string(), ItemType::Sword, 500);
        strong_item.required_str = 30;

        // Should pass strength check
        assert!(Equipment::check_item_requirements(&player, &weak_item).is_ok());

        // Should fail strength check
        assert_eq!(
            Equipment::check_item_requirements(&player, &strong_item),
            Err(EquipError::StrengthRequired(30))
        );
    }

    #[test]
    fn test_item_requirements_magic() {
        let player = PlayerStats::new(HeroClass::Sorcerer, 10, 25, 10, 5);

        let mut low_magic_item = Item::new("Staff of Fire".to_string(), ItemType::Staff, 200);
        low_magic_item.required_mag = 20;

        let mut high_magic_item = Item::new("Staff of Lightning".to_string(), ItemType::Staff, 1000);
        high_magic_item.required_mag = 40;

        assert!(Equipment::check_item_requirements(&player, &low_magic_item).is_ok());
        assert_eq!(
            Equipment::check_item_requirements(&player, &high_magic_item),
            Err(EquipError::MagicRequired(40))
        );
    }

    #[test]
    fn test_item_requirements_level() {
        let player = PlayerStats::new(HeroClass::Rogue, 15, 15, 20, 8);

        let mut low_level_item = Item::new("Short Bow".to_string(), ItemType::Bow, 150);
        low_level_item.required_level = 5;

        let mut high_level_item = Item::new("Windforce".to_string(), ItemType::Bow, 5000);
        high_level_item.required_level = 20;

        assert!(Equipment::check_item_requirements(&player, &low_level_item).is_ok());
        assert_eq!(
            Equipment::check_item_requirements(&player, &high_level_item),
            Err(EquipError::LevelRequired(20))
        );
    }

    // Test Group 6: Equipment slot matching (4 tests)

    #[test]
    fn test_can_equip_helm() {
        let equipment = Equipment::new();
        let player = PlayerStats::new(HeroClass::Warrior, 20, 10, 10, 5);

        let helm = Item::new("Cap".to_string(), ItemType::Helm, 100);
        let sword = Item::new("Sword".to_string(), ItemType::Sword, 100);

        // Helm can go in head slot
        assert!(equipment.can_equip(&player, &helm, EquipSlot::Head).is_ok());

        // Sword cannot go in head slot
        assert_eq!(
            equipment.can_equip(&player, &sword, EquipSlot::Head),
            Err(EquipError::InvalidSlot)
        );
    }

    #[test]
    fn test_can_equip_armor() {
        let equipment = Equipment::new();
        let player = PlayerStats::new(HeroClass::Warrior, 20, 10, 10, 5);

        let armor = Item::new("Leather Armor".to_string(), ItemType::Armor, 200);
        let helm = Item::new("Cap".to_string(), ItemType::Helm, 100);

        assert!(equipment.can_equip(&player, &armor, EquipSlot::Chest).is_ok());
        assert_eq!(
            equipment.can_equip(&player, &helm, EquipSlot::Chest),
            Err(EquipError::InvalidSlot)
        );
    }

    #[test]
    fn test_can_equip_ring() {
        let equipment = Equipment::new();
        let player = PlayerStats::new(HeroClass::Rogue, 15, 15, 20, 5);

        let ring = Item::new("Ring".to_string(), ItemType::Ring, 300);
        let amulet = Item::new("Amulet".to_string(), ItemType::Amulet, 400);

        // Ring can go in ring slots
        assert!(equipment.can_equip(&player, &ring, EquipSlot::RingLeft).is_ok());
        assert!(equipment.can_equip(&player, &ring, EquipSlot::RingRight).is_ok());

        // Amulet cannot go in ring slot
        assert_eq!(
            equipment.can_equip(&player, &amulet, EquipSlot::RingLeft),
            Err(EquipError::InvalidSlot)
        );
    }

    #[test]
    fn test_can_equip_amulet() {
        let equipment = Equipment::new();
        let player = PlayerStats::new(HeroClass::Sorcerer, 10, 25, 10, 5);

        let amulet = Item::new("Amulet of Magic".to_string(), ItemType::Amulet, 500);
        let ring = Item::new("Ring".to_string(), ItemType::Ring, 300);

        assert!(equipment.can_equip(&player, &amulet, EquipSlot::Amulet).is_ok());
        assert_eq!(
            equipment.can_equip(&player, &ring, EquipSlot::Amulet),
            Err(EquipError::InvalidSlot)
        );
    }

    // Test Group 7: Weapon wielding (4 tests)

    #[test]
    fn test_can_wield_one_handed() {
        let equipment = Equipment::new();
        let player = PlayerStats::new(HeroClass::Warrior, 25, 10, 15, 5);

        let sword = Item::new("Short Sword".to_string(), ItemType::Sword, 100);

        // Can equip one-handed weapon in empty hand
        assert!(equipment.can_equip(&player, &sword, EquipSlot::HandRight).is_ok());
        assert!(equipment.can_equip(&player, &sword, EquipSlot::HandLeft).is_ok());
    }

    #[test]
    fn test_can_wield_two_handed() {
        let mut equipment = Equipment::new();
        let player = PlayerStats::new(HeroClass::Warrior, 30, 10, 15, 5);

        let bow = Item::new("Long Bow".to_string(), ItemType::Bow, 200);
        let shield = Item::new("Shield".to_string(), ItemType::Shield, 150);

        // Can equip two-handed weapon when both hands empty
        assert!(equipment.can_equip(&player, &bow, EquipSlot::HandRight).is_ok());

        // Occupy left hand
        equipment.hand_left = Some(shield.clone());

        // Cannot equip two-handed weapon when hand occupied
        assert_eq!(
            equipment.can_equip(&player, &bow, EquipSlot::HandRight),
            Err(EquipError::TwoHandedConflict)
        );
    }

    #[test]
    fn test_dual_wield_bard() {
        let mut equipment = Equipment::new();
        let bard = PlayerStats::new(HeroClass::Bard, 20, 15, 20, 5);

        let sword1 = Item::new("Short Sword".to_string(), ItemType::Sword, 100);
        let sword2 = Item::new("Long Sword".to_string(), ItemType::Sword, 150);

        // Equip first sword in right hand
        equipment.hand_right = Some(sword1.clone());

        // Bard can dual wield swords
        assert!(equipment.can_equip(&bard, &sword2, EquipSlot::HandLeft).is_ok());
    }

    #[test]
    fn test_both_hands_occupied() {
        let mut equipment = Equipment::new();
        let player = PlayerStats::new(HeroClass::Rogue, 20, 15, 25, 5);

        let sword = Item::new("Short Sword".to_string(), ItemType::Sword, 100);
        let dagger = Item::new("Dagger".to_string(), ItemType::Sword, 80);
        let axe = Item::new("Axe".to_string(), ItemType::Axe, 120);

        // Occupy both hands
        equipment.hand_left = Some(sword.clone());
        equipment.hand_right = Some(dagger.clone());

        // Cannot equip when both hands occupied
        assert_eq!(
            equipment.can_equip(&player, &axe, EquipSlot::HandLeft),
            Err(EquipError::BothHandsOccupied)
        );
        assert_eq!(
            equipment.can_equip(&player, &axe, EquipSlot::HandRight),
            Err(EquipError::BothHandsOccupied)
        );
    }

    // Test Group 8: Equip/Unequip operations (2 tests)

    #[test]
    fn test_equip_unequip() {
        let mut equipment = Equipment::new();
        let player = PlayerStats::new(HeroClass::Warrior, 25, 10, 15, 5);

        let helm = Item::new("Cap".to_string(), ItemType::Helm, 100);
        let armor = Item::new("Leather Armor".to_string(), ItemType::Armor, 200);

        // Equip helm
        let result = equipment.equip(&player, helm.clone(), EquipSlot::Head);
        assert!(result.is_ok());
        let prev_item = result.unwrap();
        assert!(prev_item.is_none()); // No previous item

        // Helm is now equipped
        assert!(!equipment.is_empty(EquipSlot::Head));

        // Equip armor
        equipment.equip(&player, armor.clone(), EquipSlot::Chest).unwrap();

        // Unequip helm
        let unequipped = equipment.unequip(EquipSlot::Head);
        assert!(unequipped.is_some());
        assert_eq!(unequipped.unwrap().name, "Cap");

        // Head slot now empty
        assert!(equipment.is_empty(EquipSlot::Head));

        // Chest still has armor
        assert!(!equipment.is_empty(EquipSlot::Chest));
    }


    #[test]
    fn test_auto_equip() {
        let mut equipment = Equipment::new();
        let player = PlayerStats::new(HeroClass::Warrior, 25, 10, 15, 5);

        let helm = Item::new("Cap".to_string(), ItemType::Helm, 100);
        let ring = Item::new("Ring".to_string(), ItemType::Ring, 300);
        let sword = Item::new("Sword".to_string(), ItemType::Sword, 150);

        // Auto-equip helm
        let slot = equipment.auto_equip(&player, helm.clone());
        assert_eq!(slot, Ok(EquipSlot::Head));
        assert!(!equipment.is_empty(EquipSlot::Head));

        // Auto-equip ring (should go to RingLeft first)
        let slot = equipment.auto_equip(&player, ring.clone());
        assert_eq!(slot, Ok(EquipSlot::RingLeft));

        // Auto-equip weapon (should go to HandRight first)
        let slot = equipment.auto_equip(&player, sword.clone());
        assert_eq!(slot, Ok(EquipSlot::HandRight));
    }

    #[test]
    fn test_slot_occupied_error() {
        let mut equipment = Equipment::new();
        let player = PlayerStats::new(HeroClass::Warrior, 25, 10, 15, 5);

        let helm1 = Item::new("Cap".to_string(), ItemType::Helm, 100);
        let helm2 = Item::new("Great Helm".to_string(), ItemType::Helm, 300);

        // Equip first helm
        equipment.equip(&player, helm1.clone(), EquipSlot::Head).unwrap();

        // Try to equip second helm - should fail with SlotOccupied
        let result = equipment.equip(&player, helm2.clone(), EquipSlot::Head);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e, EquipError::SlotOccupied);
        }

        // can_equip should also return SlotOccupied
        assert_eq!(
            equipment.can_equip(&player, &helm2, EquipSlot::Head),
            Err(EquipError::SlotOccupied)
        );
    }

    // ========================================================================
    // Test Group 9: Day 95 Auto-Placement System (12 tests)
    // ========================================================================

    #[test]
    fn test_item_size() {
        let size_1x1 = ItemSize::new(1, 1);
        assert_eq!(size_1x1.width, 1);
        assert_eq!(size_1x1.height, 1);

        let size_2x3 = ItemSize::new(2, 3);
        assert_eq!(size_2x3.width, 2);
        assert_eq!(size_2x3.height, 3);
    }

    #[test]
    fn test_can_fit_at_slot_1x1() {
        let inventory = Inventory::new();

        // 1×1 item fits everywhere in empty inventory
        let size = ItemSize::new(1, 1);
        assert!(inventory.can_fit_at_slot(0, size)); // Top-left
        assert!(inventory.can_fit_at_slot(9, size)); // Top-right
        assert!(inventory.can_fit_at_slot(30, size)); // Bottom-left
        assert!(inventory.can_fit_at_slot(39, size)); // Bottom-right
    }

    #[test]
    fn test_can_fit_at_slot_2x3() {
        let inventory = Inventory::new();

        // 2×3 item fits at top-left (requires 2 columns, 3 rows)
        let size = ItemSize::new(2, 3);
        assert!(inventory.can_fit_at_slot(0, size));

        // Does NOT fit at slot 9 (would extend beyond right edge)
        assert!(!inventory.can_fit_at_slot(9, size));

        // Does NOT fit at slot 20 (would extend beyond bottom edge)
        assert!(!inventory.can_fit_at_slot(20, size));
    }

    #[test]
    fn test_add_item_1x1() {
        let mut inventory = Inventory::new();
        let potion = Item::new("Health Potion".to_string(), ItemType::HealthPotion, 50);

        // Add 1×1 item at slot 5
        let result = inventory.add_item(potion.clone(), 5, ItemSize::new(1, 1));
        assert!(result.is_ok());

        // Check state
        assert_eq!(inventory.count, 1);
        assert_eq!(inventory.grid[5], 0); // Item index 0
        assert_eq!(inventory.items.len(), 1);

        // Check all other cells empty
        for i in 0..INVENTORY_SIZE {
            if i != 5 {
                assert_eq!(inventory.grid[i], -1);
            }
        }
    }

    #[test]
    fn test_add_item_2x3_occupies_6_cells() {
        let mut inventory = Inventory::new();
        let armor = Item::new("Plate Mail".to_string(), ItemType::Armor, 2000);

        // Add 2×3 item at slot 0 (top-left)
        let result = inventory.add_item(armor.clone(), 0, ItemSize::new(2, 3));
        assert!(result.is_ok());

        // Grid layout (2×3 at slot 0):
        // [0,  -1, ?, ?, ?, ?, ?, ?, ?, ?]  row 0
        // [-1, -1, ?, ?, ?, ?, ?, ?, ?, ?]  row 1
        // [+0, -1, ?, ?, ?, ?, ?, ?, ?, ?]  row 2 (bottom-left stores item index)
        // [?, ?, ?, ?, ?, ?, ?, ?, ?, ?]    row 3

        // Occupied cells: 0, 1 (row 0), 10, 11 (row 1), 20, 21 (row 2)
        assert_eq!(inventory.grid[0], -1);  // Top-left: -1 (occupied)
        assert_eq!(inventory.grid[1], -1);  // Top-right of item: -1
        assert_eq!(inventory.grid[10], -1); // Middle-left: -1
        assert_eq!(inventory.grid[11], -1); // Middle-right: -1
        assert_eq!(inventory.grid[20], 0);  // Bottom-left: +0 (item index)
        assert_eq!(inventory.grid[21], -1); // Bottom-right: -1
    }

    #[test]
    fn test_find_empty_slot_height_1() {
        let mut inventory = Inventory::new();

        // Empty inventory: 1×1 item should go to slot 30 (last row first)
        let slot = inventory.find_empty_slot(ItemSize::new(1, 1));
        assert_eq!(slot, Some(30));

        // Fill slot 30
        let potion = Item::new("Potion".to_string(), ItemType::HealthPotion, 50);
        inventory.add_item(potion.clone(), 30, ItemSize::new(1, 1)).unwrap();

        // Next 1×1 should go to slot 31
        let slot = inventory.find_empty_slot(ItemSize::new(1, 1));
        assert_eq!(slot, Some(31));
    }

    #[test]
    fn test_find_empty_slot_2x3() {
        let inventory = Inventory::new();

        // Empty inventory: 2×3 item should go to slot 0
        let slot = inventory.find_empty_slot(ItemSize::new(2, 3));
        assert_eq!(slot, Some(0));
    }

    #[test]
    fn test_remove_item() {
        let mut inventory = Inventory::new();
        let sword = Item::new("Long Sword".to_string(), ItemType::Sword, 150);

        // Add item at slot 10
        inventory.add_item(sword.clone(), 10, ItemSize::new(1, 1)).unwrap();
        assert_eq!(inventory.count, 1);

        // Remove item
        let removed = inventory.remove_item(10);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "Long Sword");
        assert_eq!(inventory.count, 0);
        assert_eq!(inventory.grid[10], -1); // Cell cleared
    }

    #[test]
    fn test_auto_place_inventory() {
        let mut inventory = Inventory::new();
        let potion = Item::new("Mana Potion".to_string(), ItemType::ManaPotion, 50);

        // Auto-place first 1×1 item (should go to slot 30)
        let result = inventory.auto_place(potion.clone(), ItemSize::new(1, 1));
        assert_eq!(result, Ok(30));
        assert_eq!(inventory.count, 1);
    }

    #[test]
    fn test_auto_place_gold() {
        let mut inventory = Inventory::new();

        // Add 3000 gold
        let remaining = auto_place_gold(&mut inventory, 3000);
        assert_eq!(remaining, 0); // All gold placed
        assert_eq!(inventory.count, 1); // 1 stack created

        // Check gold stack value
        if let Some(Some(item)) = inventory.items.first() {
            assert_eq!(item.item_type, ItemType::Gold);
            assert_eq!(item.quantity, 3000);
        } else {
            panic!("Gold item not found");
        }

        // Add 8000 more gold (should create 2 stacks: 2000 + 5000, 5000 left full)
        let remaining = auto_place_gold(&mut inventory, 8000);
        assert_eq!(remaining, 0);
        assert_eq!(inventory.count, 3); // 3 stacks total (5000, 5000, 1000)
    }

    #[test]
    fn test_belt_auto_place() {
        let mut belt = Belt::new();
        let scroll = Item::new("Town Portal".to_string(), ItemType::Scroll, 200);

        // Auto-place first item (should go to slot 0)
        let result = belt.auto_place(scroll.clone());
        assert_eq!(result, Ok(0));
        assert_eq!(belt.count(), 1);

        // Auto-place second item (should go to slot 1)
        let result = belt.auto_place(scroll.clone());
        assert_eq!(result, Ok(1));
        assert_eq!(belt.count(), 2);
    }

    // ========================================================================
    // Test Group 10: Day 96 Item Usage + Integration (11 tests)
    // ========================================================================

    #[test]
    fn test_use_health_potion() {
        let potion = Item::new("Health Potion".to_string(), ItemType::HealthPotion, 50);

        let result = use_potion(&potion);
        assert!(result.is_ok());

        if let Ok(ItemUseEffect::HealedHP(amount)) = result {
            assert_eq!(amount, 50); // 50% heal
        } else {
            panic!("Expected HealedHP effect");
        }
    }

    #[test]
    fn test_use_mana_potion() {
        let potion = Item::new("Mana Potion".to_string(), ItemType::ManaPotion, 50);

        let result = use_potion(&potion);
        assert!(result.is_ok());

        if let Ok(ItemUseEffect::RestoredMana(amount)) = result {
            assert_eq!(amount, 50); // 50% mana restore
        } else {
            panic!("Expected RestoredMana effect");
        }
    }

    #[test]
    fn test_use_full_rejuvenation() {
        let rejuv = Item::new("Full Rejuvenation".to_string(), ItemType::FullRejuvenation, 200);

        let result = use_potion(&rejuv);
        assert!(result.is_ok());

        if let Ok(ItemUseEffect::Rejuvenated(hp, mana)) = result {
            assert_eq!(hp, 100);
            assert_eq!(mana, 100);
        } else {
            panic!("Expected Rejuvenated effect");
        }
    }

    #[test]
    fn test_use_invalid_potion() {
        let sword = Item::new("Sword".to_string(), ItemType::Sword, 100);

        let result = use_potion(&sword);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UseItemError::NotUsable);
    }

    #[test]
    fn test_use_scroll() {
        let scroll = Item::new("Scroll of Town Portal".to_string(), ItemType::Scroll, 200);
        let spell_id = 1; // Town Portal

        let result = use_scroll(&scroll, spell_id);
        assert!(result.is_ok());

        if let Ok(ItemUseEffect::ScrollUsed(id)) = result {
            assert_eq!(id, spell_id);
        } else {
            panic!("Expected ScrollUsed effect");
        }
    }

    #[test]
    fn test_use_non_scroll_fails() {
        let potion = Item::new("Potion".to_string(), ItemType::HealthPotion, 50);

        let result = use_scroll(&potion, 1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UseItemError::NotUsable);
    }

    #[test]
    fn test_use_book() {
        let book = Item::new("Book of Firebolt".to_string(), ItemType::Book, 500);
        let spell_id = 2; // Firebolt
        let current_level = 0; // Not learned yet

        let result = use_book(&book, spell_id, current_level);
        assert!(result.is_ok());

        if let Ok(ItemUseEffect::BookRead(new_level)) = result {
            assert_eq!(new_level, 1);
        } else {
            panic!("Expected BookRead effect");
        }
    }

    #[test]
    fn test_use_book_max_level() {
        let book = Item::new("Book of Lightning".to_string(), ItemType::Book, 500);
        let spell_id = 3;
        let current_level = 15; // Max level

        let result = use_book(&book, spell_id, current_level);
        assert!(result.is_err());

        if let Err(UseItemError::Other(msg)) = result {
            assert!(msg.contains("max level"));
        } else {
            panic!("Expected 'max level' error");
        }
    }

    #[test]
    fn test_use_staff_charge() {
        let staff = Item::new("Staff of Lightning".to_string(), ItemType::Staff, 1000);
        let charges = 10;

        let result = use_staff_charge(&staff, charges);
        assert!(result.is_ok());

        if let Ok(ItemUseEffect::ChargeConsumed(remaining)) = result {
            assert_eq!(remaining, 9);
        } else {
            panic!("Expected ChargeConsumed effect");
        }
    }

    #[test]
    fn test_use_staff_no_charges() {
        let staff = Item::new("Staff of Fireball".to_string(), ItemType::Staff, 1000);
        let charges = 0;

        let result = use_staff_charge(&staff, charges);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UseItemError::NoChargesLeft);
    }

    #[test]
    fn test_can_use_item() {
        // Usable items
        assert!(can_use_item(&Item::new("Health Potion".to_string(), ItemType::HealthPotion, 50)));
        assert!(can_use_item(&Item::new("Mana Potion".to_string(), ItemType::ManaPotion, 50)));
        assert!(can_use_item(&Item::new("Scroll".to_string(), ItemType::Scroll, 200)));
        assert!(can_use_item(&Item::new("Book".to_string(), ItemType::Book, 500)));
        assert!(can_use_item(&Item::new("Staff".to_string(), ItemType::Staff, 1000)));
        assert!(can_use_item(&Item::new("Quest Item".to_string(), ItemType::Quest, 0)));

        // Non-usable items
        assert!(!can_use_item(&Item::new("Sword".to_string(), ItemType::Sword, 100)));
        assert!(!can_use_item(&Item::new("Armor".to_string(), ItemType::Armor, 200)));
        assert!(!can_use_item(&Item::new("Helm".to_string(), ItemType::Helm, 150)));
    }

    // ========================================================================
    // Test Group 11: Two-Hand Weapon System (M69 Day 2)
    // ========================================================================

    #[test]
    fn test_change_two_hand_empty_hands() {
        let mut equipment = Equipment::new();
        let mut inventory = Inventory::new();

        let bow = Item::new("Long Bow".to_string(), ItemType::Bow, 300);

        // Both hands empty - should equip to left hand
        let result = equipment.change_two_hand_item(&mut inventory, bow.clone());
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // No previous item

        // Bow should be in left hand
        assert!(equipment.hand_left.is_some());
        assert_eq!(equipment.hand_left.as_ref().unwrap().name, "Long Bow");
    }

    #[test]
    fn test_change_two_hand_left_occupied() {
        let mut equipment = Equipment::new();
        let mut inventory = Inventory::new();

        let sword = Item::new("Short Sword".to_string(), ItemType::Sword, 100);
        let bow = Item::new("Long Bow".to_string(), ItemType::Bow, 300);

        // Occupy left hand
        equipment.hand_left = Some(sword.clone());

        // Equip bow - should return sword
        let result = equipment.change_two_hand_item(&mut inventory, bow.clone());
        assert!(result.is_ok());

        let prev = result.unwrap();
        assert!(prev.is_some());
        assert_eq!(prev.unwrap().name, "Short Sword");

        // Bow should be in left hand
        assert_eq!(equipment.hand_left.as_ref().unwrap().name, "Long Bow");
    }

    #[test]
    fn test_change_two_hand_right_occupied() {
        let mut equipment = Equipment::new();
        let mut inventory = Inventory::new();

        let shield = Item::new("Buckler".to_string(), ItemType::Shield, 80);
        let bow = Item::new("Long Bow".to_string(), ItemType::Bow, 300);

        // Occupy right hand with shield
        equipment.hand_right = Some(shield.clone());

        // Equip bow - should return shield
        let result = equipment.change_two_hand_item(&mut inventory, bow.clone());
        assert!(result.is_ok());

        let prev = result.unwrap();
        assert!(prev.is_some());
        assert_eq!(prev.unwrap().name, "Buckler");

        // Bow should be in left hand, right hand empty
        assert_eq!(equipment.hand_left.as_ref().unwrap().name, "Long Bow");
        assert!(equipment.hand_right.is_none());
    }

    #[test]
    fn test_change_two_hand_both_occupied() {
        let mut equipment = Equipment::new();
        let mut inventory = Inventory::new();

        let sword = Item::new("Short Sword".to_string(), ItemType::Sword, 100);
        let shield = Item::new("Buckler".to_string(), ItemType::Shield, 80);
        let bow = Item::new("Long Bow".to_string(), ItemType::Bow, 300);

        // Occupy both hands
        equipment.hand_left = Some(sword.clone());
        equipment.hand_right = Some(shield.clone());

        // Equip bow - should move shield to inventory (prefer shield)
        let result = equipment.change_two_hand_item(&mut inventory, bow.clone());
        assert!(result.is_ok());

        // Shield should be in inventory
        assert!(inventory.count > 0);

        // Sword should be returned (was in left hand)
        let prev = result.unwrap();
        assert!(prev.is_some());
        assert_eq!(prev.unwrap().name, "Short Sword");

        // Bow should be in left hand
        assert_eq!(equipment.hand_left.as_ref().unwrap().name, "Long Bow");
    }
}

// ============================================================================
// M69 Day 1: Inventory Drag and Paste System
// ============================================================================
// C++ Reference: Source/inv.cpp:420-560 (CheckOverlappingItems, ChangeInvItem, CheckInvPaste)
//
// This module implements drag-and-drop inventory operations:
// - check_overlapping_items: Detect items displaced by held item
// - get_displaced_item_id: Get ID of item at slot position
// - paste_item_to_inventory: Place held item into inventory grid
// - swap_inventory_item: Exchange held item with existing item
// - pick_up_from_inventory: Pick item from inventory to cursor
// - paste_item_to_belt: Place held item into belt slot
// - swap_belt_item: Exchange held item with belt item

/// Result of attempting to paste an item into inventory
#[derive(Debug, Clone)]
pub enum PasteResult {
    /// Item placed successfully (slot index)
    Placed(usize),

    /// Item swapped with existing item
    Swapped(Item),

    /// Gold merged with existing stack (remaining amount if overflow)
    GoldMerged(i32),

    /// Cannot place item (reason)
    Failed(PasteError),
}

/// Errors that can occur when pasting items
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasteError {
    /// Item doesn't fit at target location
    DoesNotFit,

    /// Multiple items would be displaced (can only swap with one)
    MultipleItemsOverlap,

    /// Invalid slot index
    InvalidSlot,

    /// Inventory is full
    InventoryFull,

    /// Item cannot be placed in belt
    NotBeltItem,

    /// Belt is full
    BeltFull,
}

impl std::fmt::Display for PasteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasteError::DoesNotFit => write!(f, "Item does not fit at target location"),
            PasteError::MultipleItemsOverlap => write!(f, "Multiple items would be displaced"),
            PasteError::InvalidSlot => write!(f, "Invalid inventory slot"),
            PasteError::InventoryFull => write!(f, "Inventory is full"),
            PasteError::NotBeltItem => write!(f, "Item cannot be placed in belt"),
            PasteError::BeltFull => write!(f, "Belt is full"),
        }
    }
}

impl std::error::Error for PasteError {}

impl Inventory {
    /// Check overlapping items at target slot
    ///
    /// C++ equivalent: CheckOverlappingItems in Source/inv.cpp:420-446
    ///
    /// Checks whether pasting an item at the given slot would overlap with
    /// existing items. Returns the ID of the single overlapped item, or
    /// error if multiple different items would be displaced.
    ///
    /// # Arguments
    /// * `slot_index` - Target grid position (0-39)
    /// * `item_size` - Size of item being pasted
    ///
    /// # Returns
    /// * `Ok(Some(item_index))` - Single item would be displaced
    /// * `Ok(None)` - No items would be displaced (empty space)
    /// * `Err(PasteError)` - Multiple items would be displaced or invalid slot
    pub fn check_overlapping_items(
        &self,
        slot_index: usize,
        item_size: ItemSize,
    ) -> Result<Option<usize>, PasteError> {
        if slot_index >= INVENTORY_SIZE {
            return Err(PasteError::InvalidSlot);
        }

        let start_x = slot_index % INVENTORY_WIDTH;
        let start_y = slot_index / INVENTORY_WIDTH;

        // Check if item extends beyond grid boundaries
        if start_x + item_size.width as usize > INVENTORY_WIDTH {
            return Err(PasteError::DoesNotFit);
        }
        if start_y + item_size.height as usize > INVENTORY_HEIGHT {
            return Err(PasteError::DoesNotFit);
        }

        let mut overlapping_id: Option<usize> = None;

        // Scan all cells the item would occupy
        for dy in 0..item_size.height as usize {
            for dx in 0..item_size.width as usize {
                let grid_idx = (start_y + dy) * INVENTORY_WIDTH + (start_x + dx);
                let cell_val = self.grid[grid_idx];

                if cell_val != -1 {
                    // Cell is occupied, get the item index
                    let item_idx = if cell_val >= 0 {
                        cell_val as usize
                    } else {
                        // Negative value: convert back to item index
                        ((-cell_val) - 1) as usize
                    };

                    if let Some(existing_id) = overlapping_id {
                        if existing_id != item_idx {
                            // Found two different items that would be displaced
                            return Err(PasteError::MultipleItemsOverlap);
                        }
                    } else {
                        overlapping_id = Some(item_idx);
                    }
                }
            }
        }

        Ok(overlapping_id)
    }

    /// Get the item at a grid position
    ///
    /// C++ equivalent: Part of GetPrevItemId in Source/inv.cpp:449-459
    ///
    /// # Arguments
    /// * `slot_index` - Grid position (0-39)
    ///
    /// # Returns
    /// * `Some(&Item)` - Reference to item at slot
    /// * `None` - Slot is empty
    pub fn get_item_at_slot(&self, slot_index: usize) -> Option<&Item> {
        if slot_index >= INVENTORY_SIZE {
            return None;
        }

        let cell_val = self.grid[slot_index];
        if cell_val == -1 {
            return None;
        }

        let item_idx = if cell_val >= 0 {
            cell_val as usize
        } else {
            ((-cell_val) - 1) as usize
        };

        self.items.get(item_idx).and_then(|opt| opt.as_ref())
    }

    /// Paste held item into inventory at specific slot
    ///
    /// C++ equivalent: ChangeInvItem in Source/inv.cpp:465-530
    ///
    /// Handles placing an item at a specific inventory position:
    /// - If slot is empty: place item
    /// - If slot has item: swap items
    /// - If gold onto gold: merge stacks
    ///
    /// # Arguments
    /// * `held_item` - Item to paste
    /// * `slot_index` - Target grid position (0-39)
    /// * `item_size` - Size of held item
    ///
    /// # Returns
    /// * `PasteResult::Placed(slot)` - Item placed at slot
    /// * `PasteResult::Swapped(item)` - Swapped, returns displaced item
    /// * `PasteResult::GoldMerged(remaining)` - Gold merged, returns overflow
    /// * `PasteResult::Failed(error)` - Operation failed
    pub fn paste_item(
        &mut self,
        held_item: Item,
        slot_index: usize,
        item_size: ItemSize,
    ) -> PasteResult {
        // Check for overlapping items
        let overlap_result = self.check_overlapping_items(slot_index, item_size);

        match overlap_result {
            Err(e) => PasteResult::Failed(e),

            Ok(None) => {
                // Empty space - just place the item
                match self.add_item(held_item, slot_index, item_size) {
                    Ok(()) => PasteResult::Placed(slot_index),
                    Err(_) => PasteResult::Failed(PasteError::DoesNotFit),
                }
            }

            Ok(Some(existing_idx)) => {
                // Something is there - check if we can merge gold
                let existing_item = self.items.get(existing_idx)
                    .and_then(|opt| opt.as_ref());

                if held_item.item_type == ItemType::Gold {
                    if let Some(existing) = existing_item {
                        if existing.item_type == ItemType::Gold {
                            // Merge gold stacks
                            return self.merge_gold(held_item, existing_idx);
                        }
                    }
                }

                // Swap items
                self.swap_with_existing(held_item, slot_index, item_size, existing_idx)
            }
        }
    }

    /// Merge gold with existing gold stack
    fn merge_gold(&mut self, held_gold: Item, existing_idx: usize) -> PasteResult {
        if let Some(Some(existing)) = self.items.get_mut(existing_idx) {
            let total = existing.quantity + held_gold.quantity;
            if total <= MAX_GOLD_PER_STACK {
                existing.quantity = total;
                PasteResult::GoldMerged(0)
            } else {
                existing.quantity = MAX_GOLD_PER_STACK;
                let overflow = total - MAX_GOLD_PER_STACK;
                PasteResult::GoldMerged(overflow)
            }
        } else {
            PasteResult::Failed(PasteError::InvalidSlot)
        }
    }

    /// Swap held item with existing item at slot
    fn swap_with_existing(
        &mut self,
        held_item: Item,
        slot_index: usize,
        held_size: ItemSize,
        existing_idx: usize,
    ) -> PasteResult {
        // Remove existing item from grid
        let displaced = self.remove_item_by_index(existing_idx);

        if let Some(displaced_item) = displaced {
            // Place held item
            match self.add_item(held_item, slot_index, held_size) {
                Ok(()) => PasteResult::Swapped(displaced_item),
                Err(_) => {
                    // Failed to place - restore displaced item (shouldn't happen)
                    // Note: In real implementation we'd need to handle this better
                    PasteResult::Failed(PasteError::DoesNotFit)
                }
            }
        } else {
            PasteResult::Failed(PasteError::InvalidSlot)
        }
    }

    /// Remove item by its index in the items vector
    fn remove_item_by_index(&mut self, item_index: usize) -> Option<Item> {
        // Clear grid cells for this item
        for i in 0..INVENTORY_SIZE {
            let cell_val = self.grid[i];
            if cell_val >= 0 && cell_val as usize == item_index {
                self.grid[i] = -1;
            } else if cell_val < -1 && ((-cell_val) - 1) as usize == item_index {
                self.grid[i] = -1;
            }
        }

        // Take item from vector
        if let Some(item_opt) = self.items.get_mut(item_index) {
            if item_opt.is_some() {
                self.count -= 1;
            }
            item_opt.take()
        } else {
            None
        }
    }

    /// Pick up item from inventory slot
    ///
    /// C++ equivalent: Part of CheckInvCut in Source/inv.cpp
    ///
    /// Removes item from inventory at specified position and returns it
    /// for the player to hold in cursor.
    ///
    /// # Arguments
    /// * `slot_index` - Grid position (0-39)
    ///
    /// # Returns
    /// * `Some((Item, ItemSize))` - Picked up item and its size
    /// * `None` - No item at slot
    pub fn pick_up_item(&mut self, slot_index: usize) -> Option<(Item, ItemSize)> {
        if slot_index >= INVENTORY_SIZE {
            return None;
        }

        let cell_val = self.grid[slot_index];
        if cell_val == -1 {
            return None;
        }

        // Get actual item index
        let item_idx = if cell_val >= 0 {
            cell_val as usize
        } else {
            ((-cell_val) - 1) as usize
        };

        // Calculate item size from grid pattern
        let item_size = self.calculate_item_size(item_idx);

        // Remove and return item
        let item = self.remove_item_by_index(item_idx)?;
        Some((item, item_size))
    }

    /// Calculate item size from its grid footprint
    fn calculate_item_size(&self, item_index: usize) -> ItemSize {
        let mut min_x = INVENTORY_WIDTH;
        let mut max_x = 0;
        let mut min_y = INVENTORY_HEIGHT;
        let mut max_y = 0;
        let mut found = false;

        for y in 0..INVENTORY_HEIGHT {
            for x in 0..INVENTORY_WIDTH {
                let idx = y * INVENTORY_WIDTH + x;
                let cell_val = self.grid[idx];

                let matches = if cell_val >= 0 {
                    cell_val as usize == item_index
                } else if cell_val != -1 {
                    ((-cell_val) - 1) as usize == item_index
                } else {
                    false
                };

                if matches {
                    found = true;
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                    min_y = min_y.min(y);
                    max_y = max_y.max(y);
                }
            }
        }

        if found {
            ItemSize {
                width: (max_x - min_x + 1) as u8,
                height: (max_y - min_y + 1) as u8,
            }
        } else {
            ItemSize::new(1, 1) // Default
        }
    }

    /// Find item containing cursor position
    ///
    /// Useful for determining which item the cursor is hovering over.
    ///
    /// # Arguments
    /// * `grid_x` - X position in grid (0-9)
    /// * `grid_y` - Y position in grid (0-3)
    ///
    /// # Returns
    /// * `Some(item_index)` - Index of item at position
    /// * `None` - No item at position
    pub fn find_item_at_position(&self, grid_x: usize, grid_y: usize) -> Option<usize> {
        if grid_x >= INVENTORY_WIDTH || grid_y >= INVENTORY_HEIGHT {
            return None;
        }

        let slot_index = grid_y * INVENTORY_WIDTH + grid_x;
        let cell_val = self.grid[slot_index];

        if cell_val == -1 {
            return None;
        }

        let item_idx = if cell_val >= 0 {
            cell_val as usize
        } else {
            ((-cell_val) - 1) as usize
        };

        Some(item_idx)
    }
}

impl Belt {
    /// Paste item to specific belt slot
    ///
    /// C++ equivalent: ChangeBeltItem in Source/inv.cpp:531-545
    ///
    /// # Arguments
    /// * `item` - Item to paste (must be 1×1)
    /// * `slot` - Belt slot (0-7)
    ///
    /// # Returns
    /// * `PasteResult::Placed(slot)` - Item placed
    /// * `PasteResult::Swapped(item)` - Swapped with existing item
    /// * `PasteResult::Failed(error)` - Cannot place item
    pub fn paste_item(&mut self, item: Item, slot: u8) -> PasteResult {
        if slot >= NUM_BELT_SLOTS as u8 {
            return PasteResult::Failed(PasteError::InvalidSlot);
        }

        // Check if item can go in belt (must be 1×1 usable item like potions)
        if !Self::can_place_in_belt(&item) {
            return PasteResult::Failed(PasteError::NotBeltItem);
        }

        let slot_idx = slot as usize;

        if let Some(existing) = self.items[slot_idx].take() {
            // Swap with existing item
            self.items[slot_idx] = Some(item);
            PasteResult::Swapped(existing)
        } else {
            // Empty slot
            self.items[slot_idx] = Some(item);
            PasteResult::Placed(slot_idx)
        }
    }

    /// Check if item can be placed in belt
    ///
    /// C++ equivalent: CanBePlacedOnBelt in Source/inv.cpp
    /// Only 1×1 usable items (potions, scrolls) can go in belt
    pub fn can_place_in_belt(item: &Item) -> bool {
        matches!(
            item.item_type,
            ItemType::HealthPotion | ItemType::ManaPotion | ItemType::Scroll
        )
    }

    /// Pick up item from belt slot
    ///
    /// # Arguments
    /// * `slot` - Belt slot (0-7)
    ///
    /// # Returns
    /// * `Some(Item)` - Picked up item
    /// * `None` - Slot was empty
    pub fn pick_up_item(&mut self, slot: u8) -> Option<Item> {
        if slot >= NUM_BELT_SLOTS as u8 {
            return None;
        }
        self.items[slot as usize].take()
    }

    /// Get item at belt slot
    pub fn get_item_at_slot(&self, slot: u8) -> Option<&Item> {
        if slot >= NUM_BELT_SLOTS as u8 {
            return None;
        }
        self.items[slot as usize].as_ref()
    }
}

// ============================================================================
// Held Item System
// ============================================================================
// Represents the item currently held by the cursor/player

/// Represents the item currently being held by the player cursor
#[derive(Debug, Clone, Default)]
pub struct HeldItem {
    /// The item being held (None if cursor is empty)
    pub item: Option<Item>,

    /// Size of held item (for grid placement)
    pub size: ItemSize,
}

impl HeldItem {
    /// Create empty held item state
    pub fn new() -> Self {
        Self {
            item: None,
            size: ItemSize::new(1, 1),
        }
    }

    /// Check if holding an item
    pub fn is_holding(&self) -> bool {
        self.item.is_some()
    }

    /// Pick up an item to cursor
    pub fn pick_up(&mut self, item: Item, size: ItemSize) {
        self.item = Some(item);
        self.size = size;
    }

    /// Put down held item
    pub fn put_down(&mut self) -> Option<Item> {
        self.size = ItemSize::new(1, 1);
        self.item.take()
    }

    /// Get reference to held item
    pub fn get(&self) -> Option<&Item> {
        self.item.as_ref()
    }

    /// Get mutable reference to held item
    pub fn get_mut(&mut self) -> Option<&mut Item> {
        self.item.as_mut()
    }

    /// Swap held item with another item
    pub fn swap(&mut self, other_item: Item, other_size: ItemSize) -> Option<Item> {
        let old = self.item.take();
        self.item = Some(other_item);
        self.size = other_size;
        old
    }
}

// ============================================================================
// M69 Day 1 Tests
// ============================================================================

#[cfg(test)]
mod inventory_drag_tests {
    use super::*;

    fn create_test_item(name: &str, item_type: ItemType) -> Item {
        Item::new(name.to_string(), item_type, 100)
    }

    #[test]
    fn test_check_overlapping_items_empty() {
        let inventory = Inventory::new();
        let result = inventory.check_overlapping_items(0, ItemSize::new(1, 1));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_check_overlapping_items_single() {
        let mut inventory = Inventory::new();
        let sword = create_test_item("Sword", ItemType::Sword);
        inventory.add_item(sword, 0, ItemSize::new(1, 2)).unwrap();

        // Check slot that overlaps with sword
        let result = inventory.check_overlapping_items(0, ItemSize::new(1, 1));
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_check_overlapping_items_out_of_bounds() {
        let inventory = Inventory::new();

        // Item extending past right edge
        let result = inventory.check_overlapping_items(9, ItemSize::new(2, 1));
        assert!(matches!(result, Err(PasteError::DoesNotFit)));

        // Item extending past bottom edge
        let result = inventory.check_overlapping_items(30, ItemSize::new(1, 2));
        assert!(matches!(result, Err(PasteError::DoesNotFit)));
    }

    #[test]
    fn test_paste_item_empty_slot() {
        let mut inventory = Inventory::new();
        let sword = create_test_item("Sword", ItemType::Sword);

        let result = inventory.paste_item(sword, 0, ItemSize::new(1, 2));
        assert!(matches!(result, PasteResult::Placed(0)));
        assert_eq!(inventory.count, 1);
    }

    #[test]
    fn test_paste_item_swap() {
        let mut inventory = Inventory::new();

        // Place first item
        let sword = create_test_item("Sword", ItemType::Sword);
        inventory.add_item(sword, 0, ItemSize::new(1, 2)).unwrap();

        // Paste second item over it
        let axe = create_test_item("Axe", ItemType::Axe);
        let result = inventory.paste_item(axe, 0, ItemSize::new(1, 2));

        if let PasteResult::Swapped(displaced) = result {
            assert_eq!(displaced.name, "Sword");
        } else {
            panic!("Expected swap result");
        }
    }

    #[test]
    fn test_gold_merge() {
        let mut inventory = Inventory::new();

        // Place gold stack
        let mut gold1 = create_test_item("Gold", ItemType::Gold);
        gold1.quantity = 1000;
        gold1.max_stack = MAX_GOLD_PER_STACK;
        inventory.add_item(gold1, 30, ItemSize::new(1, 1)).unwrap();

        // Merge more gold
        let mut gold2 = create_test_item("Gold", ItemType::Gold);
        gold2.quantity = 500;
        let result = inventory.paste_item(gold2, 30, ItemSize::new(1, 1));

        assert!(matches!(result, PasteResult::GoldMerged(0)));

        // Check total
        let merged = inventory.get_item_at_slot(30).unwrap();
        assert_eq!(merged.quantity, 1500);
    }

    #[test]
    fn test_gold_merge_overflow() {
        let mut inventory = Inventory::new();

        // Place gold stack near max
        let mut gold1 = create_test_item("Gold", ItemType::Gold);
        gold1.quantity = 4500;
        gold1.max_stack = MAX_GOLD_PER_STACK;
        inventory.add_item(gold1, 30, ItemSize::new(1, 1)).unwrap();

        // Merge more gold (causes overflow)
        let mut gold2 = create_test_item("Gold", ItemType::Gold);
        gold2.quantity = 1000;
        let result = inventory.paste_item(gold2, 30, ItemSize::new(1, 1));

        if let PasteResult::GoldMerged(overflow) = result {
            assert_eq!(overflow, 500); // 4500 + 1000 - 5000 = 500
        } else {
            panic!("Expected gold merge with overflow");
        }
    }

    #[test]
    fn test_pick_up_item() {
        let mut inventory = Inventory::new();
        let sword = create_test_item("Sword", ItemType::Sword);
        inventory.add_item(sword, 0, ItemSize::new(1, 2)).unwrap();

        let picked = inventory.pick_up_item(0);
        assert!(picked.is_some());

        let (item, size) = picked.unwrap();
        assert_eq!(item.name, "Sword");
        assert_eq!(size.width, 1);
        assert_eq!(size.height, 2);

        // Inventory should now be empty at that slot
        assert!(inventory.get_item_at_slot(0).is_none());
    }

    #[test]
    fn test_belt_paste_item() {
        let mut belt = Belt::new();
        let potion = create_test_item("Health Potion", ItemType::HealthPotion);

        let result = belt.paste_item(potion, 0);
        assert!(matches!(result, PasteResult::Placed(0)));
    }

    #[test]
    fn test_belt_paste_swap() {
        let mut belt = Belt::new();

        // Place first potion
        let hp_pot = create_test_item("Health Potion", ItemType::HealthPotion);
        belt.items[0] = Some(hp_pot);

        // Paste second potion
        let mp_pot = create_test_item("Mana Potion", ItemType::ManaPotion);
        let result = belt.paste_item(mp_pot, 0);

        if let PasteResult::Swapped(displaced) = result {
            assert_eq!(displaced.name, "Health Potion");
        } else {
            panic!("Expected swap result");
        }
    }

    #[test]
    fn test_belt_reject_non_belt_item() {
        let mut belt = Belt::new();
        let sword = create_test_item("Sword", ItemType::Sword);

        let result = belt.paste_item(sword, 0);
        assert!(matches!(result, PasteResult::Failed(PasteError::NotBeltItem)));
    }

    #[test]
    fn test_held_item() {
        let mut held = HeldItem::new();
        assert!(!held.is_holding());

        let sword = create_test_item("Sword", ItemType::Sword);
        held.pick_up(sword, ItemSize::new(1, 2));

        assert!(held.is_holding());
        assert_eq!(held.get().unwrap().name, "Sword");
        assert_eq!(held.size.height, 2);

        let put_down = held.put_down();
        assert!(put_down.is_some());
        assert!(!held.is_holding());
    }

    #[test]
    fn test_find_item_at_position() {
        let mut inventory = Inventory::new();
        let sword = create_test_item("Sword", ItemType::Sword);
        inventory.add_item(sword, 5, ItemSize::new(2, 2)).unwrap();

        // Both cells should return same item index
        let at_5 = inventory.find_item_at_position(5, 0);
        let at_6 = inventory.find_item_at_position(6, 0);
        let at_15 = inventory.find_item_at_position(5, 1);
        let at_16 = inventory.find_item_at_position(6, 1);

        assert!(at_5.is_some());
        assert_eq!(at_5, at_6);
        assert_eq!(at_5, at_15);
        assert_eq!(at_5, at_16);

        // Empty position
        let at_0 = inventory.find_item_at_position(0, 0);
        assert!(at_0.is_none());
    }
}
