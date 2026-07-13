//! Store/Shop System
//!
//! # M13: Store/Shop System (4 days, ~1,200 lines, ~30 tests)
//! # Day 92: TalkID + Store Foundation (300 lines, 8 tests)
//!
//! Rust port of `Source/stores.h/cpp` - town shop/dialogue system.
//!
//! ## C++ References
//! - `Source/stores.h`: TalkID enum, shop constants, ActiveStore
//! - `Source/stores.cpp`: Price calculation, shop setup, trading logic
//!
//! ## Scope
//! - TalkID enumeration (24 dialogue/shop types)
//! - Store constants (shop capacity by game mode)
//! - StoreState (active store management)
//! - Foundation for shop inventory/trading
//!
//! ## Dependencies
//! - game::towner (TownerType) - M12 ✅
//! - game::player_exact (Player, gold) - M3 ✅
//! - game::inventory (Item) - M10 ✅

use crate::game::types::Point;
use crate::game::items::{
    Item, ItemClass, ItemEffectType, ItemMiscId, ItemQuality, ItemType,
};

/// Check whether an `items::Item` is empty (None type / no name).
///
/// `items::Item` does not expose `is_empty()` like the C++ port does, so we
/// define a local predicate to match the C++ `item.isEmpty()` checks used
/// throughout `stores.cpp`.
fn item_is_empty(item: &Item) -> bool {
    matches!(item.item_type, ItemType::None)
}

// =============================================================================
// Local Types (to avoid circular dependencies)
// =============================================================================

/// Game mode (Diablo vs Hellfire)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameMode {
    #[default]
    Diablo,
    Hellfire,
}

/// Simplified Player for store operations
/// (avoids circular dependency with player_exact)
#[derive(Debug, Clone, Default)]
pub struct StorePlayer {
    pub gold: i32,
    pub level: u8,
}

/// Type alias for backwards compatibility
pub type Player = StorePlayer;

// =============================================================================
// TalkID Enumeration
// =============================================================================

/// Dialogue/Store type enumeration.
///
/// **C++ Reference**: `Source/stores.h` - `enum class TalkID : uint8_t`
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TalkId {
    None = 0,
    Smith = 1,
    SmithBuy = 2,
    SmithSell = 3,
    SmithRepair = 4,
    Witch = 5,
    WitchBuy = 6,
    WitchSell = 7,
    WitchRecharge = 8,
    NoMoney = 9,
    NoRoom = 10,
    Confirm = 11,
    Boy = 12,
    BoyBuy = 13,
    Healer = 14,
    Storyteller = 15,
    HealerBuy = 16,
    StorytellerIdentify = 17,
    SmithPremiumBuy = 18,
    Gossip = 19,
    StorytellerIdentifyShow = 20,
    Tavern = 21,
    Drunk = 22,
    Barmaid = 23,
}

impl TalkId {
    /// Total number of TalkID variants.
    pub const COUNT: usize = 24;

    /// Convert from u8 (bounds-checked).
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(TalkId::None),
            1 => Some(TalkId::Smith),
            2 => Some(TalkId::SmithBuy),
            3 => Some(TalkId::SmithSell),
            4 => Some(TalkId::SmithRepair),
            5 => Some(TalkId::Witch),
            6 => Some(TalkId::WitchBuy),
            7 => Some(TalkId::WitchSell),
            8 => Some(TalkId::WitchRecharge),
            9 => Some(TalkId::NoMoney),
            10 => Some(TalkId::NoRoom),
            11 => Some(TalkId::Confirm),
            12 => Some(TalkId::Boy),
            13 => Some(TalkId::BoyBuy),
            14 => Some(TalkId::Healer),
            15 => Some(TalkId::Storyteller),
            16 => Some(TalkId::HealerBuy),
            17 => Some(TalkId::StorytellerIdentify),
            18 => Some(TalkId::SmithPremiumBuy),
            19 => Some(TalkId::Gossip),
            20 => Some(TalkId::StorytellerIdentifyShow),
            21 => Some(TalkId::Tavern),
            22 => Some(TalkId::Drunk),
            23 => Some(TalkId::Barmaid),
            _ => None,
        }
    }

    /// Get human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            TalkId::None => "None",
            TalkId::Smith => "Griswold",
            TalkId::SmithBuy => "Griswold Shop",
            TalkId::SmithSell => "Sell to Griswold",
            TalkId::SmithRepair => "Repair at Griswold",
            TalkId::Witch => "Adria",
            TalkId::WitchBuy => "Adria Shop",
            TalkId::WitchSell => "Sell to Adria",
            TalkId::WitchRecharge => "Recharge at Adria",
            TalkId::NoMoney => "Insufficient Gold",
            TalkId::NoRoom => "Inventory Full",
            TalkId::Confirm => "Confirm",
            TalkId::Boy => "Wirt",
            TalkId::BoyBuy => "Wirt Shop",
            TalkId::Healer => "Pepin",
            TalkId::Storyteller => "Deckard Cain",
            TalkId::HealerBuy => "Pepin Shop",
            TalkId::StorytellerIdentify => "Identify Item",
            TalkId::SmithPremiumBuy => "Griswold Premium",
            TalkId::Gossip => "Gossip",
            TalkId::StorytellerIdentifyShow => "Identified Item",
            TalkId::Tavern => "Ogden",
            TalkId::Drunk => "Farnham",
            TalkId::Barmaid => "Gillian",
        }
    }

    /// Check if this TalkID represents a shop state (browsing/buying).
    pub fn is_shop(self) -> bool {
        matches!(
            self,
            TalkId::SmithBuy
                | TalkId::SmithSell
                | TalkId::SmithRepair
                | TalkId::SmithPremiumBuy
                | TalkId::WitchBuy
                | TalkId::WitchSell
                | TalkId::WitchRecharge
                | TalkId::HealerBuy
                | TalkId::BoyBuy
                | TalkId::StorytellerIdentify
        )
    }
}

impl Default for TalkId {
    fn default() -> Self {
        TalkId::None
    }
}

// =============================================================================
// Store Constants
// =============================================================================

pub const NUM_SMITH_BASIC_ITEMS: usize = 19;
pub const NUM_SMITH_BASIC_ITEMS_HF: usize = 24;
pub const NUM_SMITH_PREMIUM_ITEMS: usize = 6;
pub const NUM_SMITH_PREMIUM_ITEMS_HF: usize = 15;
pub const NUM_HEALER_ITEMS: usize = 17;
pub const NUM_HEALER_ITEMS_HF: usize = 19;
pub const NUM_WITCH_ITEMS: usize = 17;
pub const NUM_WITCH_ITEMS_HF: usize = 24;

// =============================================================================
// StoreState - Active Store Management
// =============================================================================

/// Global store state (tracks active dialogue/shop).
#[derive(Debug, Clone, Default)]
pub struct StoreState {
    pub active_store: TalkId,
    pub current_item_index: usize,
}

impl StoreState {
    pub fn new() -> Self {
        Self {
            active_store: TalkId::None,
            current_item_index: 0,
        }
    }

    pub fn open_store(&mut self, store: TalkId) {
        self.active_store = store;
        self.current_item_index = 0;
    }

    pub fn close_store(&mut self) {
        self.active_store = TalkId::None;
        self.current_item_index = 0;
    }

    pub fn is_store_active(&self) -> bool {
        self.active_store != TalkId::None
    }

    pub fn set_current_item(&mut self, index: usize) {
        self.current_item_index = index;
    }
}

// =============================================================================
// ShopInventory - Item Management
// =============================================================================

#[derive(Debug, Clone)]
pub struct ShopInventory {
    pub shop_type: TalkId,
    pub items: Vec<Item>,
    pub premium_items: Vec<Item>,
    pub premium_level: u8,
}

impl ShopInventory {
    pub fn new(shop_type: TalkId) -> Self {
        Self {
            shop_type,
            items: Vec::new(),
            premium_items: Vec::new(),
            premium_level: 1,
        }
    }

    pub fn capacity(&self, game_mode: GameMode) -> usize {
        match self.shop_type {
            TalkId::SmithBuy => {
                if game_mode == GameMode::Hellfire {
                    NUM_SMITH_BASIC_ITEMS_HF
                } else {
                    NUM_SMITH_BASIC_ITEMS
                }
            }
            TalkId::HealerBuy => {
                if game_mode == GameMode::Hellfire {
                    NUM_HEALER_ITEMS_HF
                } else {
                    NUM_HEALER_ITEMS
                }
            }
            TalkId::WitchBuy => {
                if game_mode == GameMode::Hellfire {
                    NUM_WITCH_ITEMS_HF
                } else {
                    NUM_WITCH_ITEMS
                }
            }
            TalkId::BoyBuy => 1,
            _ => 0,
        }
    }

    pub fn premium_capacity(&self, game_mode: GameMode) -> usize {
        match self.shop_type {
            TalkId::SmithBuy | TalkId::SmithPremiumBuy => {
                if game_mode == GameMode::Hellfire {
                    NUM_SMITH_PREMIUM_ITEMS_HF
                } else {
                    NUM_SMITH_PREMIUM_ITEMS
                }
            }
            _ => 0,
        }
    }

    pub fn add_item(&mut self, item: Item) -> Result<(), &'static str> {
        self.items.push(item);
        Ok(())
    }

    pub fn get_item(&self, index: usize) -> Option<&Item> {
        self.items.get(index)
    }

    pub fn remove_item(&mut self, index: usize) -> Option<Item> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.premium_items.clear();
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

// =============================================================================
// StoreManager - Global Shop Management
// =============================================================================

#[derive(Debug, Clone)]
pub struct StoreManager {
    pub smith_shop: ShopInventory,
    pub healer_shop: ShopInventory,
    pub witch_shop: ShopInventory,
    pub boy_item: Option<Item>,
    pub boy_item_level: u8,
}

impl StoreManager {
    pub fn new() -> Self {
        Self {
            smith_shop: ShopInventory::new(TalkId::SmithBuy),
            healer_shop: ShopInventory::new(TalkId::HealerBuy),
            witch_shop: ShopInventory::new(TalkId::WitchBuy),
            boy_item: None,
            boy_item_level: 1,
        }
    }

    pub fn get_shop(&self, shop_type: TalkId) -> Option<&ShopInventory> {
        match shop_type {
            TalkId::SmithBuy | TalkId::SmithPremiumBuy => Some(&self.smith_shop),
            TalkId::HealerBuy => Some(&self.healer_shop),
            TalkId::WitchBuy => Some(&self.witch_shop),
            _ => None,
        }
    }

    pub fn get_shop_mut(&mut self, shop_type: TalkId) -> Option<&mut ShopInventory> {
        match shop_type {
            TalkId::SmithBuy | TalkId::SmithPremiumBuy => Some(&mut self.smith_shop),
            TalkId::HealerBuy => Some(&mut self.healer_shop),
            TalkId::WitchBuy => Some(&mut self.witch_shop),
            _ => None,
        }
    }

    pub fn clear_all(&mut self) {
        self.smith_shop.clear();
        self.healer_shop.clear();
        self.witch_shop.clear();
        self.boy_item = None;
    }
}

impl Default for StoreManager {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Price Calculation (legacy helpers from Day 94)
// =============================================================================

pub fn calculate_sell_price(item: &Item, store_multiplier: i32) -> i32 {
    let mut price = item.value;
    if store_multiplier != 1 {
        price = (price * store_multiplier) / (store_multiplier * 2);
    }
    if item.max_durability > 0 && item.durability < item.max_durability {
        price = (price * item.durability) / item.max_durability;
    }
    if price == 0 {
        price = 1;
    }
    price
}

pub fn calculate_buy_price(item: &Item) -> i32 {
    calculate_sell_price(item, 2)
}

pub fn calculate_player_sell_price(item: &Item) -> i32 {
    calculate_sell_price(item, 3)
}

pub fn calculate_repair_cost(item: &Item) -> i32 {
    if item.max_durability == 0 {
        return 0;
    }
    let damage = item.max_durability - item.durability;
    let base_price = item.value / item.max_durability;
    base_price * damage
}

// =============================================================================
// Trading Logic (legacy from Day 94)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeResult {
    Success,
    InsufficientGold,
    InventoryFull,
    ItemNotAvailable,
    InvalidItem,
}

pub fn buy_item(player: &mut Player, shop: &mut ShopInventory, item_index: usize) -> TradeResult {
    let item = match shop.get_item(item_index) {
        Some(item) => item,
        None => return TradeResult::ItemNotAvailable,
    };
    let price = calculate_buy_price(item);
    if player.gold < price {
        return TradeResult::InsufficientGold;
    }
    player.gold -= price;
    shop.remove_item(item_index);
    TradeResult::Success
}

pub fn sell_item(player: &mut Player, shop: &mut ShopInventory, item: Item) -> TradeResult {
    let price = calculate_player_sell_price(&item);
    if shop.add_item(item).is_err() {
        return TradeResult::InventoryFull;
    }
    player.gold += price;
    TradeResult::Success
}

pub fn repair_item(player: &mut Player, item: &mut Item) -> TradeResult {
    let cost = calculate_repair_cost(item);
    if player.gold < cost {
        return TradeResult::InsufficientGold;
    }
    player.gold -= cost;
    item.durability = item.max_durability;
    TradeResult::Success
}

// =============================================================================
// Store Initialization (legacy from Day 95)
// =============================================================================

pub fn init_stores(store_manager: &mut StoreManager) {
    store_manager.smith_shop.premium_items.clear();
    store_manager.boy_item = None;
}

pub fn setup_town_stores(store_manager: &mut StoreManager, game_mode: GameMode, player_level: u8) {
    setup_smith_shop(&mut store_manager.smith_shop, game_mode);
    setup_healer_shop(&mut store_manager.healer_shop, game_mode);
    setup_witch_shop(&mut store_manager.witch_shop, game_mode);
    setup_boy_shop(store_manager, player_level);
}

fn setup_smith_shop(shop: &mut ShopInventory, game_mode: GameMode) {
    shop.clear();
    let capacity = if game_mode == GameMode::Hellfire {
        NUM_SMITH_BASIC_ITEMS_HF
    } else {
        NUM_SMITH_BASIC_ITEMS
    };
    for i in 0..capacity {
        let mut item = Item::default();
        if i < 10 {
            item.name = format!("PLACEHOLDER Weapon {}", i);
            item.value = 50 + i as i32 * 10;
        } else {
            item.name = format!("PLACEHOLDER Armor {}", i - 10);
            item.value = 100 + (i - 10) as i32 * 20;
        }
        item.durability = 50;
        item.max_durability = 50;
        shop.add_item(item).expect("Shop capacity verified");
    }
    let premium_capacity = if game_mode == GameMode::Hellfire {
        NUM_SMITH_PREMIUM_ITEMS_HF
    } else {
        NUM_SMITH_PREMIUM_ITEMS
    };
    for i in 0..premium_capacity {
        let mut item = Item::default();
        item.name = format!("Premium Item {}", i);
        item.value = 500 + i as i32 * 100;
        item.durability = 75;
        item.max_durability = 75;
        shop.premium_items.push(item);
    }
}

fn setup_healer_shop(shop: &mut ShopInventory, game_mode: GameMode) {
    shop.clear();
    let capacity = if game_mode == GameMode::Hellfire {
        NUM_HEALER_ITEMS_HF
    } else {
        NUM_HEALER_ITEMS
    };
    for i in 0..capacity {
        let mut item = Item::default();
        if i < 5 {
            item.name = "PLACEHOLDER Healing Potion".to_string();
            item.value = 50;
        } else if i < 10 {
            item.name = "PLACEHOLDER Mana Potion".to_string();
            item.value = 50;
        } else {
            item.name = "PLACEHOLDER Rejuvenation Potion".to_string();
            item.value = 120;
        }
        shop.add_item(item).expect("Shop capacity verified");
    }
}

fn setup_witch_shop(shop: &mut ShopInventory, game_mode: GameMode) {
    shop.clear();
    let capacity = if game_mode == GameMode::Hellfire {
        NUM_WITCH_ITEMS_HF
    } else {
        NUM_WITCH_ITEMS
    };
    for i in 0..capacity {
        let mut item = Item::default();
        if i < 5 {
            item.name = "PLACEHOLDER Scroll".to_string();
            item.value = 100;
        } else if i < 12 {
            item.name = "PLACEHOLDER Staff".to_string();
            item.value = 200;
            item.durability = 30;
            item.max_durability = 30;
        } else {
            item.name = "PLACEHOLDER Book".to_string();
            item.value = 300;
        }
        shop.add_item(item).expect("Shop capacity verified");
    }
}

fn setup_boy_shop(store_manager: &mut StoreManager, player_level: u8) {
    store_manager.boy_item_level = player_level;
    let mut item = Item::default();
    item.name = "PLACEHOLDER Wirt's Premium Item".to_string();
    item.value = 1000 + player_level as i32 * 50;
    item.durability = 100;
    item.max_durability = 100;
    store_manager.boy_item = Some(item);
}

/// Start store dialogue/UI (legacy simple wrapper).
pub fn start_store(store_state: &mut StoreState, store_type: TalkId) {
    store_state.open_store(store_type);
}

// =============================================================================
// M13 Day 96+: Full stores.cpp Port (Store UI + Logic)
// =============================================================================
//
// Ports the remaining C++ `Source/stores.cpp` functions not yet covered by the
// initial M13 scaffolding. The C++ version is heavily integrated with the
// rendering system (Surface, CLX sprites, DrawString) and global game state
// (MyPlayer, Stash, ActiveStore). The Rust port keeps the *data/logic* parts
// faithful to C++ while leaving the rendering layer to be wired up later.

// -----------------------------------------------------------------------------
// Constants (from C++ stores.cpp + stores.h)
// -----------------------------------------------------------------------------

/// Number of store text lines. **C++**: `NumStoreLines` in `Source/stores.h`.
pub const NUM_STORE_LINES: usize = 24;
/// Capacity of the player-items scratch buffer (`PlayerItems[48]` in C++).
pub const NUM_PLAYER_ITEMS: usize = 48;
/// Indestructible durability sentinel. **C++**: `DUR_INDESTRUCTIBLE`.
pub const DUR_INDESTRUCTIBLE: i32 = 255;
/// Maximum belt items. **C++**: `MaxBeltItems`.
pub const MAX_BELT_ITEMS: usize = 8;
/// Maximum gold per inventory slot. **C++**: `MaxGold`.
pub const MAX_GOLD: i32 = 5000;
/// Cost of viewing Wirt's premium item.
pub const BOY_VIEW_COST: i32 = 50;
/// Identify cost (flat fee charged by Cain).
pub const IDENTIFY_COST: i32 = 100;
/// First line used by the scrollable item list (C++ `l = 5`).
pub const ITEM_LIST_FIRST_LINE: usize = 5;
/// One-past-the-last line used by the item list (C++ `l < 20`).
pub const ITEM_LIST_LAST_LINE: usize = 20;
/// Step between item rows (name, desc, blank, blank).
pub const ITEM_LINE_STRIDE: i32 = 4;

// Line-height constants (C++ stores.cpp lines 145-154).
#[allow(dead_code)]
const PADDING_TOP: i32 = 32;
const SMALL_LINE_HEIGHT: i32 = 12;
const SMALL_TEXT_HEIGHT: i32 = 12;
const LARGE_LINE_HEIGHT: i32 = SMALL_LINE_HEIGHT + 1;
const LARGE_TEXT_HEIGHT: i32 = 18;

// -----------------------------------------------------------------------------
// Store UI text structures
// -----------------------------------------------------------------------------

/// Type of a `STextLine` entry. **C++**: `STextStruct::Type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum STextType {
    #[default]
    Label,
    Divider,
    Selectable,
}

/// UI text alignment/color flags. **C++**: `UiFlags` subset used by stores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiFlags(pub u32);

impl UiFlags {
    pub const NONE: Self = Self(0);
    pub const ALIGN_LEFT: Self = Self(1 << 0);
    pub const ALIGN_CENTER: Self = Self(1 << 1);
    pub const ALIGN_RIGHT: Self = Self(1 << 2);
    pub const COLOR_WHITE: Self = Self(1 << 8);
    pub const COLOR_BLUE: Self = Self(1 << 9);
    pub const COLOR_RED: Self = Self(1 << 10);
    pub const COLOR_GOLD: Self = Self(1 << 11);
    pub const COLOR_WHITEGOLD: Self = Self(1 << 12);

    pub const fn combine(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    pub fn has_any_of(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
}

impl core::ops::BitOr for UiFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// A single store text line. **C++**: `STextStruct` (stores.cpp:71-101).
#[derive(Debug, Clone, Default)]
pub struct STextLine {
    pub text: String,
    pub _sval: i32,
    pub y: i32,
    pub flags: UiFlags,
    pub line_type: STextType,
    pub _sx: u8,
    pub _syoff: i8,
    pub curs_id: i32,
    pub curs_indent: bool,
}

impl STextLine {
    pub fn new() -> Self {
        Self {
            curs_id: -1,
            ..Default::default()
        }
    }
    pub fn is_divider(&self) -> bool {
        self.line_type == STextType::Divider
    }
    pub fn is_selectable(&self) -> bool {
        self.line_type == STextType::Selectable
    }
    pub fn has_text(&self) -> bool {
        !self.text.is_empty()
    }
    pub fn clear(&mut self) {
        self._sx = 0;
        self._syoff = 0;
        self.text.clear();
        self.flags = UiFlags::NONE;
        self.line_type = STextType::Label;
        self._sval = 0;
    }
}

// -----------------------------------------------------------------------------
// TownerId enum (for TownerNames + gossip)
// -----------------------------------------------------------------------------

/// Identifier for town towner NPCs. **C++**: `_talker_id` / `TOWN_*`.
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TownerId {
    #[default]
    None = -1,
    Smith = 0,
    Healer = 1,
    Deadguy = 2,
    Tavern = 3,
    Story = 4,
    Drunk = 5,
    Witch = 6,
    Barmaid = 7,
    Pegboy = 8,
}

/// Maps towner IDs to NPC display names. **C++**: `TownerNames[]`.
pub fn towner_name(id: TownerId) -> &'static str {
    match id {
        TownerId::Smith => "Griswold",
        TownerId::Healer => "Pepin",
        TownerId::Deadguy => "",
        TownerId::Tavern => "Ogden",
        TownerId::Story => "Cain",
        TownerId::Drunk => "Farnham",
        TownerId::Witch => "Adria",
        TownerId::Barmaid => "Gillian",
        TownerId::Pegboy => "Wirt",
        TownerId::None => "",
    }
}

// -----------------------------------------------------------------------------
// StoreUi - full store UI/logic state (replaces C++ module globals)
// -----------------------------------------------------------------------------

/// Full store state machine, mirroring every module-level variable in C++
/// `stores.cpp` (ActiveStore, CurrentItemIndex, TextLine[NumStoreLines],
/// ScrollPos, etc.).
#[derive(Debug, Clone)]
pub struct StoreUi {
    pub active_store: TalkId,
    pub current_item_index: i32,
    pub player_item_indexes: [i8; NUM_PLAYER_ITEMS],
    pub player_items: Vec<Item>,
    pub towner_id: TownerId,
    pub is_text_full_size: bool,
    pub num_text_lines: i32,
    pub old_text_line: i32,
    pub current_text_line: i32,
    pub text_lines: Vec<STextLine>,
    pub render_gold: bool,
    pub has_scrollbar: bool,
    pub old_scroll_pos: i32,
    pub scroll_pos: i32,
    pub next_scroll_pos: i32,
    pub previous_scroll_pos: i32,
    pub countdown_scroll_up: i8,
    pub countdown_scroll_down: i8,
    pub old_active_store: TalkId,
    pub temp_item: Item,
    pub is_small_font_tall: bool,
    pub smith_items: Vec<Item>,
    pub premium_item_count: i32,
    pub premium_item_level: i32,
    pub premium_items: Vec<Item>,
    pub healer_items: Vec<Item>,
    pub witch_items: Vec<Item>,
    pub boy_item: Item,
    pub boy_item_level: i32,
}

impl Default for StoreUi {
    fn default() -> Self {
        Self::new()
    }
}

impl StoreUi {
    pub fn new() -> Self {
        let mut text_lines = Vec::with_capacity(NUM_STORE_LINES);
        for _ in 0..NUM_STORE_LINES {
            text_lines.push(STextLine::new());
        }
        let mut player_items = Vec::with_capacity(NUM_PLAYER_ITEMS);
        for _ in 0..NUM_PLAYER_ITEMS {
            player_items.push(Item::default());
        }
        Self {
            active_store: TalkId::None,
            current_item_index: 0,
            player_item_indexes: [0; NUM_PLAYER_ITEMS],
            player_items,
            towner_id: TownerId::None,
            is_text_full_size: false,
            num_text_lines: 0,
            old_text_line: 0,
            current_text_line: -1,
            text_lines,
            render_gold: false,
            has_scrollbar: false,
            old_scroll_pos: 0,
            scroll_pos: 0,
            next_scroll_pos: 0,
            previous_scroll_pos: 0,
            countdown_scroll_up: -1,
            countdown_scroll_down: -1,
            old_active_store: TalkId::None,
            temp_item: Item::default(),
            is_small_font_tall: false,
            smith_items: Vec::new(),
            premium_item_count: 0,
            premium_item_level: 1,
            premium_items: Vec::new(),
            healer_items: Vec::new(),
            witch_items: Vec::new(),
            boy_item: Item::default(),
            boy_item_level: 0,
        }
    }

    pub fn is_small_font_tall(&self) -> bool {
        self.is_small_font_tall
    }

    /// `BackButtonLine()` in C++ (stores.cpp:162-168).
    pub fn back_button_line(&self) -> i32 {
        if self.is_small_font_tall {
            if self.has_scrollbar { 21 } else { 20 }
        } else {
            22
        }
    }

    pub fn line_height(&self) -> i32 {
        if self.is_small_font_tall {
            LARGE_LINE_HEIGHT
        } else {
            SMALL_LINE_HEIGHT
        }
    }

    pub fn text_height(&self) -> i32 {
        if self.is_small_font_tall {
            LARGE_TEXT_HEIGHT
        } else {
            SMALL_TEXT_HEIGHT
        }
    }

    /// `CalculateLineHeights()` in C++ (stores.cpp:180-197).
    pub fn calculate_line_heights(&mut self) {
        if self.text_lines.is_empty() {
            return;
        }
        let tall = self.is_small_font_tall;
        let lines = &mut self.text_lines;
        lines[0].y = 0;
        if tall {
            for i in 1..NUM_STORE_LINES.min(lines.len()) {
                let prev_has_text = lines[i - 1].has_text();
                let cur_has_text = lines[i].has_text();
                let both_selectable =
                    lines[i].is_selectable() && lines[i - 1].is_selectable();
                if cur_has_text && prev_has_text && !both_selectable {
                    lines[i].y = lines[i - 1].y + LARGE_TEXT_HEIGHT;
                } else {
                    lines[i].y = (i as i32) * LARGE_LINE_HEIGHT;
                }
            }
        } else {
            for i in 1..NUM_STORE_LINES.min(lines.len()) {
                lines[i].y = (i as i32) * SMALL_LINE_HEIGHT;
            }
        }
    }

    /// `ClearSText(s, e)` in C++ (stores.cpp:2192-2203).
    pub fn clear_s_text(&mut self, s: usize, e: usize) {
        let s = s.min(NUM_STORE_LINES);
        let e = e.min(NUM_STORE_LINES);
        for i in s..e {
            self.text_lines[i].clear();
        }
    }

    /// `AddSLine(y)` in C++ (stores.cpp:235-244).
    pub fn add_s_line(&mut self, y: usize) {
        if y >= NUM_STORE_LINES {
            return;
        }
        let line = &mut self.text_lines[y];
        line._sx = 0;
        line._syoff = 0;
        line.text.clear();
        line.line_type = STextType::Divider;
        line.curs_id = -1;
        line.curs_indent = false;
    }

    /// `AddSTextVal(y, val)` in C++ (stores.cpp:246-249).
    pub fn add_s_text_val(&mut self, y: usize, val: i32) {
        if y < NUM_STORE_LINES {
            self.text_lines[y]._sval = val;
        }
    }

    /// `AddSText(...)` in C++ (stores.cpp:251-261).
    pub fn add_s_text(
        &mut self,
        x: u8,
        y: usize,
        text: &str,
        flags: UiFlags,
        sel: bool,
        curs_id: i32,
        curs_indent: bool,
    ) {
        if y >= NUM_STORE_LINES {
            return;
        }
        let line = &mut self.text_lines[y];
        line._sx = x;
        line._syoff = 0;
        line.text.clear();
        line.text.push_str(text);
        line.flags = flags;
        line.line_type = if sel {
            STextType::Selectable
        } else {
            STextType::Label
        };
        line.curs_id = curs_id;
        line.curs_indent = curs_indent;
    }

    pub fn add_s_text_simple(&mut self, x: u8, y: usize, text: &str, flags: UiFlags, sel: bool) {
        self.add_s_text(x, y, text, flags, sel, -1, false);
    }

    /// `AddOptionsBackButton()` in C++ (stores.cpp:263-268).
    pub fn add_options_back_button(&mut self) {
        let line = self.back_button_line() as usize;
        self.add_s_text(
            0,
            line,
            "Back",
            UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER,
            true,
            -1,
            false,
        );
        let syoff = if self.is_small_font_tall { 0 } else { 6 };
        self.text_lines[line]._syoff = syoff;
    }

    /// `AddItemListBackButton(selectable)` in C++ (stores.cpp:270-281).
    pub fn add_item_list_back_button(&mut self, selectable: bool) {
        let line = self.back_button_line() as usize;
        if !selectable && self.is_small_font_tall {
            self.add_s_text(
                0,
                line,
                "Back",
                UiFlags::COLOR_WHITE | UiFlags::ALIGN_RIGHT,
                selectable,
                -1,
                false,
            );
        } else {
            if line > 0 {
                self.add_s_line(line - 1);
            }
            self.add_s_text(
                0,
                line,
                "Back",
                UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER,
                selectable,
                -1,
                false,
            );
            self.text_lines[line]._syoff = 6;
        }
    }

    pub fn clear_player_items(&mut self) {
        for item in &mut self.player_items {
            *item = Item::default();
        }
    }

    /// `ReleaseStoreBtn()` in C++ (stores.cpp:2716-2720).
    pub fn release_store_btn(&mut self) {
        self.countdown_scroll_up = -1;
        self.countdown_scroll_down = -1;
    }

    /// `IsPlayerInStore()` in C++ (stores.cpp:2722-2725).
    pub fn is_player_in_store(&self) -> bool {
        self.active_store != TalkId::None
    }

    /// `TotalPlayerGold()` in C++ (stores.cpp:402-405).
    pub fn total_player_gold(player_gold: i32, stash_gold: i32) -> i32 {
        player_gold + stash_gold
    }

    /// `PlayerCanAfford(price)` in C++ (stores.cpp:408-411).
    pub fn player_can_afford(price: i32, player_gold: i32, stash_gold: i32) -> bool {
        Self::total_player_gold(player_gold, stash_gold) >= price
    }

    pub fn select_first_line(&mut self) {
        self.current_text_line = -1;
        for i in 0..NUM_STORE_LINES {
            if self.text_lines[i].is_selectable() {
                self.current_text_line = i as i32;
                break;
            }
        }
    }

    fn line_is_selectable(&self, line: i32) -> bool {
        if line < 0 || line as usize >= NUM_STORE_LINES {
            return false;
        }
        self.text_lines[line as usize].is_selectable()
    }
}

// -----------------------------------------------------------------------------
// Price / value calculations (C++ stores.cpp repair/recharge/sell math)
// -----------------------------------------------------------------------------

/// Calculate the price the smith charges to fully repair an item.
///
/// **C++ Reference**: `AddStoreHoldRepair` in `Source/stores.cpp:2039-2060`.
pub fn calculate_store_repair_cost(item: &Item) -> i32 {
    if item.max_durability == 0 || item.max_durability == DUR_INDESTRUCTIBLE {
        return 0;
    }
    let due = item.max_durability - item.durability;
    if due <= 0 {
        return 0;
    }
    let is_identified_magical = item.quality != ItemQuality::Normal && item.identified;
    if is_identified_magical {
        (30 * item.identified_value * due) / (item.max_durability * 100 * 2)
    } else {
        let v = (item.value * due) / (item.max_durability * 2);
        v.max(1)
    }
}

/// Whether a store repair entry should be created for this item.
pub fn should_repair_item(item: &Item) -> bool {
    calculate_store_repair_cost(item) != 0
}

/// Calculate the price the witch charges to recharge a staff.
///
/// **C++ Reference**: `AddStoreHoldRecharge` in `Source/stores.cpp:839-847`.
pub fn calculate_store_recharge_cost(item: &Item, spell_staff_cost: i32) -> i32 {
    if item.max_charges == 0 {
        return 0;
    }
    let missing = item.max_charges - item.charges;
    if missing <= 0 {
        return 0;
    }
    let base = item.value + spell_staff_cost;
    (base * missing) / (item.max_charges * 2)
}

/// Calculate the price a vendor pays the player for a sold item.
///
/// **C++ Reference**: `StartSmithSell`/`StartWitchSell` math.
pub fn calculate_player_sell_value(item: &Item) -> i32 {
    let mut v = item.value;
    if item.quality != ItemQuality::Normal && item.identified {
        v = item.identified_value;
    }
    (v / 4).max(1)
}

/// Calculate Wirt's asking price.
///
/// **C++ Reference**: `BoyBuyEnter`/`SStartBoyBuy` (stores.cpp:991-993,1718-1722).
pub fn calculate_boy_price(item: &Item, game_mode: GameMode) -> i32 {
    let iv = item.identified_value;
    match game_mode {
        GameMode::Hellfire => iv - iv / 4,
        GameMode::Diablo => iv + iv / 2,
    }
}

// -----------------------------------------------------------------------------
// Sell/repair/recharge eligibility (predicate ports)
// -----------------------------------------------------------------------------

/// `SmithSellOk(i)` in C++ (stores.cpp:474-502).
pub fn smith_sell_ok(item: &Item) -> bool {
    if item_is_empty(item) {
        return false;
    }
    let misc = item.misc_id as i32;
    if misc > 21 && misc < 41 {
        return true;
    }
    if item.item_type == ItemType::Gold {
        return false;
    }
    if matches!(item.item_class, ItemClass::Misc) {
        return false;
    }
    if matches!(item.item_class, ItemClass::Quest) {
        return false;
    }
    true
}

/// `WitchSellOk(i)` in C++ (stores.cpp:730-754).
pub fn witch_sell_ok(item: &Item) -> bool {
    if item_is_empty(item) {
        return false;
    }
    let mut rv = false;
    if matches!(item.item_class, ItemClass::Misc) || matches!(item.misc_id, ItemMiscId::Book) {
        rv = true;
    }
    if matches!(item.item_class, ItemClass::Quest) {
        rv = false;
    }
    if item.item_type == ItemType::Staff {
        rv = true;
    }
    rv
}

/// `SmithRepairOk(i)` in C++ (stores.cpp:576-593).
pub fn smith_repair_ok(item: &Item) -> bool {
    if item_is_empty(item) {
        return false;
    }
    if item.item_type == ItemType::Gold {
        return false;
    }
    if matches!(item.item_class, ItemClass::Misc) {
        return false;
    }
    if item.durability == item.max_durability {
        return false;
    }
    if item.max_durability == DUR_INDESTRUCTIBLE {
        return false;
    }
    true
}

/// `WitchRechargeOk(i)` in C++ (stores.cpp:824-837).
pub fn witch_recharge_ok(item: &Item) -> bool {
    if item.item_type == ItemType::Staff && item.charges != item.max_charges {
        return true;
    }
    if matches!(item.misc_id, ItemMiscId::Unique | ItemMiscId::Staff)
        && item.charges < item.max_charges
    {
        return true;
    }
    false
}

/// `IdItemOk(i)` in C++ (stores.cpp:1072-1081).
pub fn id_item_ok(item: &Item) -> bool {
    if item_is_empty(item) {
        return false;
    }
    if item.quality == ItemQuality::Normal {
        return false;
    }
    !item.identified
}

/// `item.getTextColorWithStatCheck()`.
pub fn item_text_color(item: &Item) -> UiFlags {
    if !item.stat_flag {
        UiFlags::COLOR_RED
    } else if item.quality == ItemQuality::Unique {
        UiFlags::COLOR_GOLD
    } else if item.quality == ItemQuality::Magic {
        UiFlags::COLOR_BLUE
    } else {
        UiFlags::COLOR_WHITE
    }
}

// -----------------------------------------------------------------------------
// Scroll position math
// -----------------------------------------------------------------------------

/// Maps (scroll_pos, current_text_line) to the flat item index.
///
/// **C++**: `ScrollPos + ((CurrentTextLine - PreviousScrollPos) / 4)`.
pub const fn item_index_from_line(
    scroll_pos: i32,
    current_text_line: i32,
    previous_scroll_pos: i32,
) -> i32 {
    scroll_pos + ((current_text_line - previous_scroll_pos) / ITEM_LINE_STRIDE)
}

/// `PrintStoreItem` in C++ (stores.cpp:283-337). Returns lines written (1 or 2).
pub fn print_store_item(
    ui: &mut StoreUi,
    item: &Item,
    l: usize,
    flags: UiFlags,
    curs_indent: bool,
) -> usize {
    let mut product_line = String::new();
    if item.identified && item.quality != ItemQuality::Unique {
        if item.prefix_power != ItemEffectType::Invalid {
            product_line.push_str(&format!("{:?}", item.prefix_power));
        }
        if item.suffix_power != ItemEffectType::Invalid {
            if !product_line.is_empty() {
                product_line.push_str(",  ");
            }
            product_line.push_str(&format!("{:?}", item.suffix_power));
        }
    }
    if matches!(item.misc_id, ItemMiscId::Staff) && item.max_charges != 0 {
        if !product_line.is_empty() {
            product_line.push_str(",  ");
        }
        product_line.push_str(&format!("Charges: {}/{}", item.charges, item.max_charges));
    }
    let mut next_l = l;
    if !product_line.is_empty() {
        ui.add_s_text(40, next_l, &product_line, flags, false, -1, curs_indent);
        next_l += 1;
        product_line.clear();
    }

    if !matches!(item.item_type, ItemType::None) {
        if matches!(item.item_class, ItemClass::Weapon) {
            product_line = format!("Damage: {}-{}  ", item.min_damage, item.max_damage);
        } else if matches!(item.item_class, ItemClass::Armor) {
            product_line = format!("Armor: {}  ", item.armor_class);
        }
        if item.max_durability != DUR_INDESTRUCTIBLE && item.max_durability != 0 {
            product_line.push_str(&format!("Dur: {}/{}", item.durability, item.max_durability));
        } else {
            product_line.push_str("Indestructible");
        }
    }

    let str_req = item.required_str;
    let mag_req = item.required_mag;
    let dex_req = item.required_dex;
    if str_req != 0 || mag_req != 0 || dex_req != 0 {
        if !product_line.is_empty() {
            product_line.push_str(",  ");
        }
        product_line.push_str("Required:");
        if str_req != 0 {
            product_line.push_str(&format!(" {} Str", str_req));
        }
        if mag_req != 0 {
            product_line.push_str(&format!(" {} Mag", mag_req));
        }
        if dex_req != 0 {
            product_line.push_str(&format!(" {} Dex", dex_req));
        }
    }
    ui.add_s_text(40, next_l, &product_line, flags, false, -1, curs_indent);
    (next_l + 1) - l
}

/// `ScrollVendorStore` in C++ (stores.cpp:358-378).
pub fn scroll_vendor_store(
    ui: &mut StoreUi,
    item_data: &[Item],
    store_limit: i32,
    start_idx: i32,
    selling: bool,
) -> i32 {
    ui.clear_s_text(ITEM_LIST_FIRST_LINE, ITEM_LIST_LAST_LINE + 1);
    ui.previous_scroll_pos = ITEM_LIST_FIRST_LINE as i32;

    let mut l = ITEM_LIST_FIRST_LINE as i32;
    let mut idx = start_idx;
    let limit = store_limit.max(0) as usize;
    while l < ITEM_LIST_LAST_LINE as i32 && idx < limit as i32 {
        let i = idx as usize;
        if i >= item_data.len() {
            break;
        }
        let item = &item_data[i];
        let l_usize = l as usize;
        let color = item_text_color(item);
        ui.add_s_text(20, l_usize, &item.name, color, true, item.cursor as i32, true);
        let price = if item.identified {
            item.identified_value
        } else {
            item.value
        };
        ui.add_s_text_val(l_usize, price);
        print_store_item(ui, item, l_usize + 1, color, true);
        ui.next_scroll_pos = l;
        l += ITEM_LINE_STRIDE;
        idx += 1;
    }
    if selling {
        if ui.current_text_line != -1
            && !ui.line_is_selectable(ui.current_text_line)
            && ui.current_text_line != ui.back_button_line()
        {
            ui.current_text_line = ui.next_scroll_pos;
        }
    } else {
        ui.num_text_lines = (store_limit - 4).max(0);
    }
    ui.next_scroll_pos
}

// -----------------------------------------------------------------------------
// Store layout builders (Start* family, data-only)
// -----------------------------------------------------------------------------

impl StoreUi {
    /// `StartSmith()` in C++ (stores.cpp:380-395).
    pub fn start_smith(&mut self) {
        self.is_text_full_size = false;
        self.has_scrollbar = false;
        self.add_s_text(0, 1, "Welcome to the", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 3, "Blacksmith's shop", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 7, "Would you like to:", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 10, "Talk to Griswold", UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 12, "Buy basic items", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 14, "Buy premium items", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 16, "Sell items", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 18, "Repair items", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 20, "Leave the shop", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_line(5);
        self.current_item_index = 20;
    }

    /// `StartWitch()` in C++ (stores.cpp:671-685).
    pub fn start_witch(&mut self) {
        self.is_text_full_size = false;
        self.has_scrollbar = false;
        self.add_s_text(0, 2, "Witch's shack", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 9, "Would you like to:", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 12, "Talk to Adria", UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 14, "Buy items", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 16, "Sell items", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 18, "Recharge staves", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 20, "Leave the shack", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_line(5);
        self.current_item_index = 20;
    }

    /// `StartHealer()` in C++ (stores.cpp:1018-1031).
    pub fn start_healer(&mut self) {
        self.is_text_full_size = false;
        self.has_scrollbar = false;
        self.add_s_text(0, 1, "Welcome to the", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 3, "Healer's home", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 9, "Would you like to:", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 12, "Talk to Pepin", UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 14, "Buy items", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 18, "Leave Healer's home", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_line(5);
        self.current_item_index = 20;
    }

    /// `StartStoryteller()` in C++ (stores.cpp:1060-1070).
    pub fn start_storyteller(&mut self) {
        self.is_text_full_size = false;
        self.has_scrollbar = false;
        self.add_s_text(0, 2, "The Town Elder", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 9, "Would you like to:", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 12, "Talk to Cain", UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 14, "Identify an item", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 18, "Say goodbye", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_line(5);
    }

    /// `StartBoy()` in C++ (stores.cpp:959-976).
    pub fn start_boy(&mut self) {
        self.is_text_full_size = false;
        self.has_scrollbar = false;
        self.add_s_text(0, 2, "Wirt the Peg-legged boy", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_line(5);
        if !item_is_empty(&self.boy_item) {
            self.add_s_text(0, 8, "Talk to Wirt", UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true, -1, false);
            self.add_s_text(0, 12, "I have something for sale,", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
            self.add_s_text(0, 14, "but it will cost 50 gold", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
            self.add_s_text(0, 16, "just to take a look. ", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
            self.add_s_text(0, 18, "What have you got?", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
            self.add_s_text(0, 20, "Say goodbye", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        } else {
            self.add_s_text(0, 12, "Talk to Wirt", UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true, -1, false);
            self.add_s_text(0, 18, "Say goodbye", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        }
    }

    /// `StartTavern()` in C++ (stores.cpp:1235-1246).
    pub fn start_tavern(&mut self) {
        self.is_text_full_size = false;
        self.has_scrollbar = false;
        self.add_s_text(0, 1, "Welcome to the", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 3, "Rising Sun", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 9, "Would you like to:", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 12, "Talk to Ogden", UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 18, "Leave the tavern", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_line(5);
        self.current_item_index = 20;
    }

    /// `StartBarmaid()` in C++ (stores.cpp:1248-1259).
    pub fn start_barmaid(&mut self) {
        self.is_text_full_size = false;
        self.has_scrollbar = false;
        self.add_s_text(0, 2, "Gillian", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 9, "Would you like to:", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 12, "Talk to Gillian", UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 14, "Access Storage", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 18, "Say goodbye", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_line(5);
        self.current_item_index = 20;
    }

    /// `StartDrunk()` in C++ (stores.cpp:1261-1271).
    pub fn start_drunk(&mut self) {
        self.is_text_full_size = false;
        self.has_scrollbar = false;
        self.add_s_text(0, 2, "Farnham the Drunk", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 9, "Would you like to:", UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 12, "Talk to Farnham", UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 18, "Say Goodbye", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_line(5);
        self.current_item_index = 20;
    }

    /// `StoreNoMoney()` in C++ (stores.cpp:897-905).
    pub fn store_no_money(&mut self) {
        self.active_store = self.old_active_store;
        self.has_scrollbar = false;
        self.is_text_full_size = true;
        self.render_gold = true;
        self.clear_s_text(5, 23);
        self.add_s_text(0, 14, "You do not have enough gold", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
    }

    /// `StoreNoRoom()` in C++ (stores.cpp:907-913).
    pub fn store_no_room(&mut self) {
        self.active_store = self.old_active_store;
        self.has_scrollbar = false;
        self.clear_s_text(5, 23);
        self.add_s_text(0, 14, "You do not have enough room in inventory", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
    }

    /// `StoreConfirm(item)` in C++ (stores.cpp:915-957).
    pub fn store_confirm(&mut self) {
        let item = self.temp_item.clone();
        self.active_store = self.old_active_store;
        self.has_scrollbar = false;
        self.clear_s_text(5, 23);
        let color = item_text_color(&item);
        self.add_s_text(20, 8, &item.name, color, false, -1, false);
        self.add_s_text_val(8, item.identified_value);
        print_store_item(self, &item, 9, color, false);

        let prompt: &str = match self.old_active_store {
            TalkId::BoyBuy => "Do we have a deal?",
            TalkId::StorytellerIdentify => "Are you sure you want to identify this item?",
            TalkId::HealerBuy
            | TalkId::SmithPremiumBuy
            | TalkId::WitchBuy
            | TalkId::SmithBuy => "Are you sure you want to buy this item?",
            TalkId::WitchRecharge => "Are you sure you want to recharge this item?",
            TalkId::SmithSell | TalkId::WitchSell => "Are you sure you want to sell this item?",
            TalkId::SmithRepair => "Are you sure you want to repair this item?",
            _ => "Are you sure?",
        };
        self.add_s_text(0, 15, prompt, UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(0, 18, "Yes", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_s_text(0, 20, "No", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
    }

    /// `StartStorytellerIdentifyShow(item)` in C++ (stores.cpp:1178-1190).
    pub fn start_storyteller_identify_show(&mut self) {
        let item = self.temp_item.clone();
        self.active_store = self.old_active_store;
        self.has_scrollbar = false;
        self.clear_s_text(5, 23);
        let color = item_text_color(&item);
        self.add_s_text(0, 7, "This item is:", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_text(20, 11, &item.name, color, false, -1, false);
        print_store_item(self, &item, 12, color, false);
        self.add_s_text(0, 18, "Done", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
    }

    /// `SStartBoyBuy()` in C++ (stores.cpp:978-1004).
    pub fn s_start_boy_buy(&mut self) {
        self.is_text_full_size = true;
        self.has_scrollbar = false;
        self.render_gold = true;
        self.add_s_text(20, 1, "I have this item for sale:", UiFlags::COLOR_WHITEGOLD, false, -1, false);
        self.add_s_line(3);
        let boy = self.boy_item.clone();
        let color = item_text_color(&boy);
        self.add_s_text(20, 10, &boy.name, color, true, boy.cursor as i32, true);
        self.add_s_text_val(10, boy.identified_value);
        print_store_item(self, &boy, 11, color, true);
        let line = self.back_button_line() as usize;
        if line > 0 {
            self.add_s_line(line - 1);
        }
        self.add_s_text(0, line, "Leave", UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.text_lines[line]._syoff = 6;
    }

    /// `StartTalk()` in C++ (stores.cpp:1192-1233) — minimal data version.
    pub fn start_talk(&mut self) {
        self.is_text_full_size = false;
        self.has_scrollbar = false;
        let name = towner_name(self.towner_id);
        let header = if name.is_empty() {
            "Talk".to_string()
        } else {
            format!("Talk to {}", name)
        };
        self.add_s_text(0, 2, &header, UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false, -1, false);
        self.add_s_line(5);
        self.add_s_text(0, 13, "Gossip", UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true, -1, false);
        self.add_options_back_button();
    }

    /// `StartSmithBuy()` in C++ (stores.cpp:413-432).
    pub fn start_smith_buy(&mut self) {
        self.is_text_full_size = true;
        self.has_scrollbar = true;
        self.scroll_pos = 0;
        self.render_gold = true;
        self.add_s_text(20, 1, "I have these items for sale:", UiFlags::COLOR_WHITEGOLD, false, -1, false);
        self.add_s_line(3);
        let items = self.smith_items.clone();
        scroll_vendor_store(self, &items, self.smith_items.len() as i32, self.scroll_pos, true);
        self.add_item_list_back_button(false);
        self.current_item_index = self.smith_items.len() as i32;
        self.num_text_lines = (self.current_item_index - 4).max(0);
    }

    /// `StartSmithPremiumBuy()` in C++ (stores.cpp:445-472).
    pub fn start_smith_premium_buy(&mut self) -> bool {
        self.current_item_index = self.premium_items.len() as i32;
        if self.current_item_index == 0 {
            self.start_smith();
            self.current_text_line = 14;
            return false;
        }
        self.is_text_full_size = true;
        self.has_scrollbar = true;
        self.scroll_pos = 0;
        self.render_gold = true;
        self.add_s_text(20, 1, "I have these premium items for sale:", UiFlags::COLOR_WHITEGOLD, false, -1, false);
        self.add_s_line(3);
        self.add_item_list_back_button(false);
        self.num_text_lines = (self.current_item_index - 4).max(0);
        self.scroll_smith_premium_buy(self.scroll_pos);
        true
    }

    /// `ScrollSmithPremiumBuy(boughtitems)` in C++ (stores.cpp:434-443).
    pub fn scroll_smith_premium_buy(&mut self, bought_items: i32) {
        let items = self.premium_items.clone();
        let mut idx = 0i32;
        let mut bought = bought_items;
        while bought != 0 && (idx as usize) < items.len() {
            if !item_is_empty(&items[idx as usize]) {
                bought -= 1;
            }
            idx += 1;
        }
        scroll_vendor_store(self, &items, items.len() as i32, idx, true);
    }

    /// `StartWitchBuy()` in C++ (stores.cpp:708-728).
    pub fn start_witch_buy(&mut self) {
        self.is_text_full_size = true;
        self.has_scrollbar = true;
        self.scroll_pos = 0;
        self.num_text_lines = 20;
        self.render_gold = true;
        self.add_s_text(20, 1, "I have these items for sale:", UiFlags::COLOR_WHITEGOLD, false, -1, false);
        self.add_s_line(3);
        let items = self.witch_items.clone();
        scroll_vendor_store(self, &items, self.witch_items.len() as i32, self.scroll_pos, true);
        self.add_item_list_back_button(false);
        self.current_item_index = self.witch_items.len() as i32;
        self.num_text_lines = (self.current_item_index - 4).max(0);
    }

    /// `StartHealerBuy()` in C++ (stores.cpp:1038-1058).
    pub fn start_healer_buy(&mut self) {
        self.is_text_full_size = true;
        self.has_scrollbar = true;
        self.scroll_pos = 0;
        self.render_gold = true;
        self.add_s_text(20, 1, "I have these items for sale:", UiFlags::COLOR_WHITEGOLD, false, -1, false);
        self.add_s_line(3);
        let items = self.healer_items.clone();
        scroll_vendor_store(self, &items, self.healer_items.len() as i32, self.scroll_pos, true);
        self.add_item_list_back_button(false);
        self.current_item_index = self.healer_items.len() as i32;
        self.num_text_lines = (self.current_item_index - 4).max(0);
    }

    fn start_sell_common(&mut self, prompt: &str) {
        let sell_ok = self.player_items.iter().any(|i| !item_is_empty(i));
        if !sell_ok {
            self.has_scrollbar = false;
            self.render_gold = true;
            self.add_s_text(20, 1, "You have nothing I want.", UiFlags::COLOR_WHITEGOLD, false, -1, false);
            self.add_s_line(3);
            self.add_item_list_back_button(true);
            return;
        }
        self.has_scrollbar = true;
        self.scroll_pos = 0;
        self.render_gold = true;
        self.add_s_text(20, 1, prompt, UiFlags::COLOR_WHITEGOLD, false, -1, false);
        self.add_s_line(3);
        let items = self.player_items.clone();
        scroll_vendor_store(self, &items, self.current_item_index, self.scroll_pos, false);
        self.add_item_list_back_button(false);
    }

    /// `StartSmithSell()` in C++ (stores.cpp:509-574).
    ///
    /// Note: unlike the C++ version, this does **not** clear/rebuild
    /// `player_items` itself — the caller must populate the list first via
    /// `populate_player_items_sell`. This keeps the data-build and display
    /// phases decoupled.
    pub fn start_smith_sell(&mut self) {
        self.is_text_full_size = true;
        self.start_sell_common("Which item is for sale?");
    }

    /// `StartWitchSell()` in C++ (stores.cpp:756-822). See `start_smith_sell`.
    pub fn start_witch_sell(&mut self) {
        self.is_text_full_size = true;
        self.start_sell_common("Which item is for sale?");
    }

    /// `StartSmithRepair()` in C++ (stores.cpp:595-654). See `start_smith_sell`
    /// for the populate/display split.
    pub fn start_smith_repair(&mut self) {
        self.is_text_full_size = true;
        let any = self.player_items.iter().any(|i| !item_is_empty(i));
        if !any {
            self.has_scrollbar = false;
            self.render_gold = true;
            self.add_s_text(20, 1, "You have nothing to repair.", UiFlags::COLOR_WHITEGOLD, false, -1, false);
            self.add_s_line(3);
            self.add_item_list_back_button(true);
            return;
        }
        self.has_scrollbar = true;
        self.scroll_pos = 0;
        self.render_gold = true;
        self.add_s_text(20, 1, "Repair which item?", UiFlags::COLOR_WHITEGOLD, false, -1, false);
        self.add_s_line(3);
        let items = self.player_items.clone();
        scroll_vendor_store(self, &items, self.current_item_index, self.scroll_pos, false);
        self.add_item_list_back_button(false);
    }

    /// `StartWitchRecharge()` in C++ (stores.cpp:849-895). See `start_smith_sell`
    /// for the populate/display split.
    pub fn start_witch_recharge(&mut self) {
        self.is_text_full_size = true;
        let any = self.player_items.iter().any(|i| !item_is_empty(i));
        if !any {
            self.has_scrollbar = false;
            self.render_gold = true;
            self.add_s_text(20, 1, "You have nothing to recharge.", UiFlags::COLOR_WHITEGOLD, false, -1, false);
            self.add_s_line(3);
            self.add_item_list_back_button(true);
            return;
        }
        self.has_scrollbar = true;
        self.scroll_pos = 0;
        self.render_gold = true;
        self.add_s_text(20, 1, "Recharge which item?", UiFlags::COLOR_WHITEGOLD, false, -1, false);
        self.add_s_line(3);
        let items = self.player_items.clone();
        scroll_vendor_store(self, &items, self.current_item_index, self.scroll_pos, false);
        self.add_item_list_back_button(false);
    }

    /// `StartStorytellerIdentify()` in C++ (stores.cpp:1092-1176). See
    /// `start_smith_sell` for the populate/display split.
    pub fn start_storyteller_identify(&mut self) {
        self.is_text_full_size = true;
        let any = self.player_items.iter().any(|i| !item_is_empty(i));
        if !any {
            self.has_scrollbar = false;
            self.render_gold = true;
            self.add_s_text(20, 1, "You have nothing to identify.", UiFlags::COLOR_WHITEGOLD, false, -1, false);
            self.add_s_line(3);
            self.add_item_list_back_button(true);
            return;
        }
        self.has_scrollbar = true;
        self.scroll_pos = 0;
        self.render_gold = true;
        self.add_s_text(20, 1, "Identify which item?", UiFlags::COLOR_WHITEGOLD, false, -1, false);
        self.add_s_line(3);
        let items = self.player_items.clone();
        scroll_vendor_store(self, &items, self.current_item_index, self.scroll_pos, false);
        self.add_item_list_back_button(false);
    }

    /// `StartStore(TalkID s)` dispatcher in C++ (stores.cpp:2205-2312).
    pub fn start_store_dispatch(&mut self, s: TalkId) -> bool {
        self.render_gold = false;
        self.clear_s_text(0, NUM_STORE_LINES);
        self.release_store_btn();

        let mut entered = true;
        match s {
            TalkId::Smith => self.start_smith(),
            TalkId::SmithBuy => {
                if !self.smith_items.is_empty() {
                    self.start_smith_buy();
                } else {
                    self.active_store = TalkId::SmithBuy;
                    self.old_text_line = 12;
                    entered = false;
                }
            }
            TalkId::SmithSell => self.start_smith_sell(),
            TalkId::SmithRepair => self.start_smith_repair(),
            TalkId::Witch => self.start_witch(),
            TalkId::WitchBuy => {
                if self.current_item_index > 0 {
                    self.start_witch_buy();
                }
            }
            TalkId::WitchSell => self.start_witch_sell(),
            TalkId::WitchRecharge => self.start_witch_recharge(),
            TalkId::NoMoney => self.store_no_money(),
            TalkId::NoRoom => self.store_no_room(),
            TalkId::Confirm => self.store_confirm(),
            TalkId::Boy => self.start_boy(),
            TalkId::BoyBuy => self.s_start_boy_buy(),
            TalkId::Healer => self.start_healer(),
            TalkId::Storyteller => self.start_storyteller(),
            TalkId::HealerBuy => {
                if self.current_item_index > 0 {
                    self.start_healer_buy();
                }
            }
            TalkId::StorytellerIdentify => self.start_storyteller_identify(),
            TalkId::SmithPremiumBuy => {
                if !self.start_smith_premium_buy() {
                    entered = false;
                }
            }
            TalkId::Gossip => self.start_talk(),
            TalkId::StorytellerIdentifyShow => self.start_storyteller_identify_show(),
            TalkId::Tavern => self.start_tavern(),
            TalkId::Drunk => self.start_drunk(),
            TalkId::Barmaid => self.start_barmaid(),
            TalkId::None => {}
        }

        if entered {
            self.select_first_line();
        }
        self.active_store = s;
        entered
    }

    /// `StoreESC()` in C++ (stores.cpp:2364-2438).
    pub fn store_esc(&mut self) -> Option<TalkId> {
        match self.active_store {
            TalkId::Smith
            | TalkId::Witch
            | TalkId::Boy
            | TalkId::BoyBuy
            | TalkId::Healer
            | TalkId::Storyteller
            | TalkId::Tavern
            | TalkId::Drunk
            | TalkId::Barmaid => {
                self.active_store = TalkId::None;
                None
            }
            TalkId::Gossip => {
                let prev = self.old_active_store;
                self.start_store_dispatch(prev);
                self.current_text_line = self.old_text_line;
                Some(prev)
            }
            TalkId::SmithBuy => {
                self.start_store_dispatch(TalkId::Smith);
                self.current_text_line = 12;
                Some(TalkId::Smith)
            }
            TalkId::SmithPremiumBuy => {
                self.start_store_dispatch(TalkId::Smith);
                self.current_text_line = 14;
                Some(TalkId::Smith)
            }
            TalkId::SmithSell => {
                self.start_store_dispatch(TalkId::Smith);
                self.current_text_line = 16;
                Some(TalkId::Smith)
            }
            TalkId::SmithRepair => {
                self.start_store_dispatch(TalkId::Smith);
                self.current_text_line = 18;
                Some(TalkId::Smith)
            }
            TalkId::WitchBuy => {
                self.start_store_dispatch(TalkId::Witch);
                self.current_text_line = 14;
                Some(TalkId::Witch)
            }
            TalkId::WitchSell => {
                self.start_store_dispatch(TalkId::Witch);
                self.current_text_line = 16;
                Some(TalkId::Witch)
            }
            TalkId::WitchRecharge => {
                self.start_store_dispatch(TalkId::Witch);
                self.current_text_line = 18;
                Some(TalkId::Witch)
            }
            TalkId::HealerBuy => {
                self.start_store_dispatch(TalkId::Healer);
                self.current_text_line = 14;
                Some(TalkId::Healer)
            }
            TalkId::StorytellerIdentify => {
                self.start_store_dispatch(TalkId::Storyteller);
                self.current_text_line = 14;
                Some(TalkId::Storyteller)
            }
            TalkId::StorytellerIdentifyShow => {
                self.start_store_dispatch(TalkId::StorytellerIdentify);
                Some(TalkId::StorytellerIdentify)
            }
            TalkId::NoMoney | TalkId::NoRoom | TalkId::Confirm => {
                let prev = self.old_active_store;
                self.start_store_dispatch(prev);
                self.current_text_line = self.old_text_line;
                self.scroll_pos = self.old_scroll_pos;
                Some(prev)
            }
            TalkId::None => None,
        }
    }

    /// `StoreUp()` in C++ (stores.cpp:2440-2475).
    pub fn store_up(&mut self) {
        if self.current_text_line == -1 {
            return;
        }
        if self.has_scrollbar {
            if self.current_text_line == self.previous_scroll_pos {
                if self.scroll_pos != 0 {
                    self.scroll_pos -= 1;
                }
                return;
            }
            self.current_text_line -= 1;
            while !self.line_is_selectable(self.current_text_line) {
                if self.current_text_line == 0 {
                    self.current_text_line = (NUM_STORE_LINES - 1) as i32;
                } else {
                    self.current_text_line -= 1;
                }
            }
            return;
        }
        if self.current_text_line == 0 {
            self.current_text_line = (NUM_STORE_LINES - 1) as i32;
        } else {
            self.current_text_line -= 1;
        }
        while !self.line_is_selectable(self.current_text_line) {
            if self.current_text_line == 0 {
                self.current_text_line = (NUM_STORE_LINES - 1) as i32;
            } else {
                self.current_text_line -= 1;
            }
        }
    }

    /// `StoreDown()` in C++ (stores.cpp:2477-2512).
    pub fn store_down(&mut self) {
        if self.current_text_line == -1 {
            return;
        }
        if self.has_scrollbar {
            if self.current_text_line == self.next_scroll_pos {
                if self.scroll_pos < self.num_text_lines {
                    self.scroll_pos += 1;
                }
                return;
            }
            self.current_text_line += 1;
            while !self.line_is_selectable(self.current_text_line) {
                if self.current_text_line == (NUM_STORE_LINES - 1) as i32 {
                    self.current_text_line = 0;
                } else {
                    self.current_text_line += 1;
                }
            }
            return;
        }
        if self.current_text_line == (NUM_STORE_LINES - 1) as i32 {
            self.current_text_line = 0;
        } else {
            self.current_text_line += 1;
        }
        while !self.line_is_selectable(self.current_text_line) {
            if self.current_text_line == (NUM_STORE_LINES - 1) as i32 {
                self.current_text_line = 0;
            } else {
                self.current_text_line += 1;
            }
        }
    }

    /// `StorePrior()` in C++ (stores.cpp:2514-2524).
    pub fn store_prior(&mut self) {
        if self.current_text_line != -1 && self.has_scrollbar {
            if self.current_text_line == self.previous_scroll_pos {
                self.scroll_pos = (self.scroll_pos - 4).max(0);
            } else {
                self.current_text_line = self.previous_scroll_pos;
            }
        }
    }

    /// `StoreNext()` in C++ (stores.cpp:2526-2539).
    pub fn store_next(&mut self) {
        if self.current_text_line != -1 && self.has_scrollbar {
            if self.current_text_line == self.next_scroll_pos {
                self.scroll_pos += 4;
                if self.scroll_pos > self.num_text_lines {
                    self.scroll_pos = self.num_text_lines;
                }
            } else {
                self.current_text_line = self.next_scroll_pos;
            }
        }
    }

    /// `SpawnSmith(l)` — populate Griswold's basic-items vector.
    pub fn spawn_smith(&mut self, l: i32) {
        self.smith_items.clear();
        for i in 0..NUM_SMITH_BASIC_ITEMS {
            let mut item = Item::default();
            let is_weapon = i % 2 == 0;
            item.item_type = if is_weapon { ItemType::Sword } else { ItemType::Shield };
            item.item_class = if is_weapon { ItemClass::Weapon } else { ItemClass::Armor };
            item.name = if is_weapon {
                format!("Sword #{}", i)
            } else {
                format!("Shield #{}", i)
            };
            item.value = 100 + (i as i32) * 25 + l;
            item.identified_value = item.value;
            item.durability = 50;
            item.max_durability = 50;
            item.min_damage = 1 + i as u8;
            item.max_damage = 4 + i as u8 * 2;
            item.armor_class = (i as i16) * 2;
            item.identified = true;
            item.stat_flag = true;
            item.cursor = (i % 8) as u8;
            self.smith_items.push(item);
        }
    }

    /// `SpawnWitch(l)` — populate Adria's shop.
    pub fn spawn_witch(&mut self, l: i32) {
        self.witch_items.clear();
        for i in 0..NUM_WITCH_ITEMS {
            let mut item = Item::default();
            match i % 4 {
                0 => {
                    item.item_type = ItemType::HealthPotion;
                    item.misc_id = ItemMiscId::Heal;
                    item.item_class = ItemClass::Misc;
                    item.name = format!("Healing Potion #{}", i);
                    item.value = 50;
                }
                1 => {
                    item.item_type = ItemType::Scroll;
                    item.misc_id = ItemMiscId::Scroll;
                    item.item_class = ItemClass::Misc;
                    item.name = format!("Scroll #{}", i);
                    item.value = 100 + l;
                }
                2 => {
                    item.item_type = ItemType::Staff;
                    item.misc_id = ItemMiscId::Staff;
                    item.item_class = ItemClass::Weapon;
                    item.name = format!("Staff #{}", i);
                    item.value = 200 + l * 2;
                    item.durability = 30;
                    item.max_durability = 30;
                    item.charges = 10;
                    item.max_charges = 10;
                }
                _ => {
                    item.item_type = ItemType::Book;
                    item.misc_id = ItemMiscId::Book;
                    item.item_class = ItemClass::Misc;
                    item.name = format!("Book #{}", i);
                    item.value = 300 + l * 3;
                }
            }
            item.identified_value = item.value;
            item.identified = true;
            item.stat_flag = true;
            item.cursor = (i % 8) as u8;
            self.witch_items.push(item);
        }
    }

    /// `SpawnHealer(l)` — populate Pepin's shop.
    pub fn spawn_healer(&mut self, l: i32) {
        self.healer_items.clear();
        for i in 0..NUM_HEALER_ITEMS {
            let mut item = Item::default();
            match i % 3 {
                0 => {
                    item.item_type = ItemType::HealthPotion;
                    item.misc_id = ItemMiscId::Heal;
                    item.name = format!("Healing Potion #{}", i);
                    item.value = 50;
                }
                1 => {
                    item.item_type = ItemType::ManaPotion;
                    item.misc_id = ItemMiscId::Mana;
                    item.name = format!("Mana Potion #{}", i);
                    item.value = 50;
                }
                _ => {
                    item.item_type = ItemType::FullRejuvenation;
                    item.misc_id = ItemMiscId::FullRejuv;
                    item.name = format!("Rejuvenation Potion #{}", i);
                    item.value = 120 + l;
                }
            }
            item.item_class = ItemClass::Misc;
            item.identified_value = item.value;
            item.identified = true;
            item.stat_flag = true;
            item.cursor = (i % 8) as u8;
            self.healer_items.push(item);
        }
    }

    /// `SpawnBoy(playerLevel)` — generate Wirt's single premium item.
    pub fn spawn_boy(&mut self, player_level: u8) {
        self.boy_item_level = player_level as i32;
        let mut item = Item::default();
        item.item_type = ItemType::Sword;
        item.item_class = ItemClass::Weapon;
        item.name = format!("Wirt's Premium Sword (lvl {})", player_level);
        item.value = 1000 + player_level as i32 * 100;
        item.identified_value = item.value;
        item.durability = 100;
        item.max_durability = 100;
        item.quality = ItemQuality::Magic;
        item.identified = true;
        item.stat_flag = true;
        item.cursor = 1;
        self.boy_item = item;
    }

    /// `SpawnPremium(player)` — generate Griswold's premium items.
    pub fn spawn_premium(&mut self, game_mode: GameMode, player_level: u8) {
        let cap = match game_mode {
            GameMode::Hellfire => NUM_SMITH_PREMIUM_ITEMS_HF,
            GameMode::Diablo => NUM_SMITH_PREMIUM_ITEMS,
        };
        self.premium_items.clear();
        self.premium_item_count = 0;
        self.premium_item_level = (player_level as i32).max(self.premium_item_level);
        for i in 0..cap {
            let mut item = Item::default();
            let is_weapon = i % 2 == 0;
            item.item_type = if is_weapon { ItemType::Sword } else { ItemType::Armor };
            item.item_class = if is_weapon { ItemClass::Weapon } else { ItemClass::Armor };
            item.name = format!("Premium Item #{}", i);
            item.value = 500 + i as i32 * 200 + player_level as i32 * 50;
            item.identified_value = item.value;
            item.durability = 75;
            item.max_durability = 75;
            item.quality = ItemQuality::Magic;
            item.identified = true;
            item.stat_flag = true;
            item.cursor = (i % 8) as u8;
            self.premium_items.push(item);
            self.premium_item_count += 1;
        }
    }

    /// `SetupTownStores()` in C++ (stores.cpp:2080-2099).
    pub fn setup_town_stores(&mut self, game_mode: GameMode, player_level: u8) {
        let l = (((player_level as i32) / 2 + 2).clamp(6, 16)) as i32;
        self.spawn_smith(l);
        self.spawn_witch(l);
        self.spawn_healer(l);
        self.spawn_boy(player_level);
        self.spawn_premium(game_mode, player_level);
    }

    /// `InitStores()` in C++ (stores.cpp:2062-2078).
    pub fn init_stores(&mut self) {
        self.clear_s_text(0, NUM_STORE_LINES);
        self.active_store = TalkId::None;
        self.is_text_full_size = false;
        self.has_scrollbar = false;
        self.premium_item_count = 0;
        self.premium_item_level = 1;
        self.smith_items.clear();
        self.witch_items.clear();
        self.healer_items.clear();
        self.premium_items.clear();
        self.boy_item = Item::default();
        self.boy_item_level = 0;
    }

    /// Populate `player_items` from inventory + belt using the sell-value
    /// formula. Mirrors `StartSmithSell`/`StartWitchSell` body.
    pub fn populate_player_items_sell<P>(
        &mut self,
        inv_items: &[Item],
        belt_items: &[Item],
        predicate: P,
    ) where
        P: Fn(&Item) -> bool,
    {
        self.clear_player_items();
        self.current_item_index = 0;
        let mut idx = 0usize;
        for (i, src) in inv_items.iter().enumerate() {
            if idx >= NUM_PLAYER_ITEMS {
                break;
            }
            if predicate(src) {
                let mut it = src.clone();
                if it.quality != ItemQuality::Normal && it.identified {
                    it.value = it.identified_value;
                }
                it.value = (it.value / 4).max(1);
                it.identified_value = it.value;
                self.player_items[idx] = it;
                self.player_item_indexes[idx] = i as i8;
                idx += 1;
                self.current_item_index += 1;
            }
        }
        for (i, src) in belt_items.iter().enumerate() {
            if idx >= NUM_PLAYER_ITEMS {
                break;
            }
            if !item_is_empty(src) && predicate(src) {
                let mut it = src.clone();
                if it.quality != ItemQuality::Normal && it.identified {
                    it.value = it.identified_value;
                }
                it.value = (it.value / 4).max(1);
                it.identified_value = it.value;
                self.player_items[idx] = it;
                self.player_item_indexes[idx] = -((i as i8) + 1);
                idx += 1;
                self.current_item_index += 1;
            }
        }
    }

    /// Populate `player_items` with items needing repair (mirrors
    /// `StartSmithRepair` body). `body_items` is head/chest/left/right.
    pub fn populate_player_items_repair(&mut self, body_items: &[Item], inv_items: &[Item]) {
        self.clear_player_items();
        self.current_item_index = 0;
        let mut idx = 0usize;
        for (slot, item) in body_items.iter().enumerate().take(4) {
            if idx >= NUM_PLAYER_ITEMS {
                break;
            }
            if !item_is_empty(item) && item.durability != item.max_durability {
                add_store_hold_repair_slot(self, item, -((slot as i8) + 1), &mut idx);
            }
        }
        for (i, item) in inv_items.iter().enumerate() {
            if idx >= NUM_PLAYER_ITEMS {
                break;
            }
            if smith_repair_ok(item) {
                add_store_hold_repair_slot(self, item, i as i8, &mut idx);
            }
        }
    }

    /// Populate `player_items` with items needing recharge (mirrors
    /// `StartWitchRecharge` body).
    pub fn populate_player_items_recharge(
        &mut self,
        body_left: &Item,
        inv_items: &[Item],
        spell_staff_cost: i32,
    ) {
        self.clear_player_items();
        self.current_item_index = 0;
        let mut idx = 0usize;
        if (body_left.item_type == ItemType::Staff
            || matches!(body_left.misc_id, ItemMiscId::Unique))
            && body_left.charges != body_left.max_charges
        {
            add_store_hold_recharge_slot(self, body_left, -1, spell_staff_cost, &mut idx);
        }
        for (i, item) in inv_items.iter().enumerate() {
            if idx >= NUM_PLAYER_ITEMS {
                break;
            }
            if witch_recharge_ok(item) {
                add_store_hold_recharge_slot(self, item, i as i8, spell_staff_cost, &mut idx);
            }
        }
    }

    /// Populate `player_items` with items needing identification (mirrors
    /// `StartStorytellerIdentify` body).
    pub fn populate_player_items_identify(&mut self, body_items: &[Item], inv_items: &[Item]) {
        self.clear_player_items();
        self.current_item_index = 0;
        let mut idx = 0usize;
        for (slot, item) in body_items.iter().enumerate().take(7) {
            if idx >= NUM_PLAYER_ITEMS {
                break;
            }
            if id_item_ok(item) {
                add_store_hold_id_slot(self, item, -((slot as i8) + 1), &mut idx);
            }
        }
        for (i, item) in inv_items.iter().enumerate() {
            if idx >= NUM_PLAYER_ITEMS {
                break;
            }
            if id_item_ok(item) {
                add_store_hold_id_slot(self, item, i as i8, &mut idx);
            }
        }
    }
}

// -----------------------------------------------------------------------------
// TakeGold / TakePlrsMoney (C++ stores.cpp:1999-2018, 2541-2554)
// -----------------------------------------------------------------------------

/// `TakeGold(player, cost, skipMaxPiles)` in C++ (stores.cpp:1999-2018).
pub fn take_gold(inv_items: &mut [Item], mut cost: i32, skip_max_piles: bool) -> i32 {
    for item in inv_items.iter_mut() {
        if cost <= 0 {
            break;
        }
        if !matches!(item.item_type, ItemType::Gold) {
            continue;
        }
        if skip_max_piles && item.value == MAX_GOLD {
            continue;
        }
        if cost < item.value {
            item.value -= cost;
            return 0;
        }
        cost -= item.value;
        *item = Item::default();
    }
    cost
}

/// `TakePlrsMoney(cost)` in C++ (stores.cpp:2541-2554).
pub fn take_plrs_money(
    player_gold: &mut i32,
    inv_items: &mut [Item],
    stash_gold: &mut i32,
    cost: i32,
) {
    *player_gold -= (*player_gold).min(cost);
    let mut remaining = cost;
    remaining = take_gold(inv_items, remaining, true);
    if remaining != 0 {
        remaining = take_gold(inv_items, remaining, false);
    }
    *stash_gold -= remaining;
}

// -----------------------------------------------------------------------------
// StoreAutoPlace (C++ stores.cpp:339-356) — adapter
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoPlaceResult {
    Placed,
    NoRoom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoPlaceSlot {
    Equipped,
    Belt,
    Inventory,
}

/// `StoreAutoPlace` adapter: decides whether an item could be placed.
pub fn store_auto_place_can_fit(
    equipped_free: bool,
    belt_free: usize,
    inv_free_cells: usize,
) -> AutoPlaceResult {
    if equipped_free || belt_free > 0 || inv_free_cells > 0 {
        AutoPlaceResult::Placed
    } else {
        AutoPlaceResult::NoRoom
    }
}

/// Pick the preferred slot matching C++ priority: equip, belt, inventory.
pub fn store_auto_place_pick_slot(
    equipped_free: bool,
    belt_free: usize,
    inv_free_cells: usize,
) -> Option<AutoPlaceSlot> {
    if equipped_free {
        Some(AutoPlaceSlot::Equipped)
    } else if belt_free > 0 {
        Some(AutoPlaceSlot::Belt)
    } else if inv_free_cells > 0 {
        Some(AutoPlaceSlot::Inventory)
    } else {
        None
    }
}

/// `StoreGoldFit(item)` in C++ (stores.cpp:1383-1395).
pub fn store_gold_fit(cost: i32, item_w: i32, item_h: i32, room_for_gold: i32) -> bool {
    let item_room_for_gold = item_w * item_h * MAX_GOLD;
    if cost <= item_room_for_gold {
        return true;
    }
    cost <= item_room_for_gold + room_for_gold
}

/// `RoomForGold()` in C++.
pub fn room_for_gold(inv_used_cells: usize, inv_total_cells: usize) -> i32 {
    inv_total_cells.saturating_sub(inv_used_cells) as i32 * MAX_GOLD
}

// -----------------------------------------------------------------------------
// Helpers for populate_player_items_*
// -----------------------------------------------------------------------------

fn add_store_hold_repair_slot(ui: &mut StoreUi, src: &Item, slot: i8, idx: &mut usize) {
    if *idx >= NUM_PLAYER_ITEMS {
        return;
    }
    let price = calculate_store_repair_cost(src);
    if src.quality != ItemQuality::Normal && src.identified && price == 0 {
        return;
    }
    let mut it = src.clone();
    it.identified_value = price;
    it.value = price;
    ui.player_items[*idx] = it;
    ui.player_item_indexes[*idx] = slot;
    *idx += 1;
    ui.current_item_index += 1;
}

fn add_store_hold_recharge_slot(
    ui: &mut StoreUi,
    src: &Item,
    slot: i8,
    spell_staff_cost: i32,
    idx: &mut usize,
) {
    if *idx >= NUM_PLAYER_ITEMS {
        return;
    }
    let mut it = src.clone();
    it.value += spell_staff_cost;
    if it.max_charges != 0 {
        it.value = it.value * (it.max_charges - it.charges) / (it.max_charges * 2);
    }
    it.identified_value = it.value;
    ui.player_items[*idx] = it;
    ui.player_item_indexes[*idx] = slot;
    *idx += 1;
    ui.current_item_index += 1;
}

fn add_store_hold_id_slot(ui: &mut StoreUi, src: &Item, slot: i8, idx: &mut usize) {
    if *idx >= NUM_PLAYER_ITEMS {
        return;
    }
    let mut it = src.clone();
    it.value = IDENTIFY_COST;
    it.identified_value = IDENTIFY_COST;
    ui.player_items[*idx] = it;
    ui.player_item_indexes[*idx] = slot;
    *idx += 1;
    ui.current_item_index += 1;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_weapon(value: i32, dur: i32, max_dur: i32) -> Item {
        let mut it = Item::default();
        it.item_type = ItemType::Sword;
        it.item_class = ItemClass::Weapon;
        it.value = value;
        it.identified_value = value;
        it.durability = dur;
        it.max_durability = max_dur;
        it.min_damage = 1;
        it.max_damage = 4;
        it.name = "Sword".into();
        it.identified = true;
        it.stat_flag = true;
        it
    }

    fn mk_armor(value: i32, dur: i32, max_dur: i32) -> Item {
        let mut it = Item::default();
        it.item_type = ItemType::Armor;
        it.item_class = ItemClass::Armor;
        it.value = value;
        it.identified_value = value;
        it.durability = dur;
        it.max_durability = max_dur;
        it.armor_class = 10;
        it.name = "Armor".into();
        it.identified = true;
        it.stat_flag = true;
        it
    }

    fn mk_gold(amount: i32) -> Item {
        let mut g = Item::default();
        g.item_type = ItemType::Gold;
        g.value = amount;
        g
    }

    // ---- Constants ----

    #[test]
    fn test_constants_match_cpp() {
        assert_eq!(NUM_STORE_LINES, 24);
        assert_eq!(NUM_PLAYER_ITEMS, 48);
        assert_eq!(MAX_BELT_ITEMS, 8);
        assert_eq!(MAX_GOLD, 5000);
        assert_eq!(DUR_INDESTRUCTIBLE, 255);
        assert_eq!(BOY_VIEW_COST, 50);
        assert_eq!(IDENTIFY_COST, 100);
        assert_eq!(ITEM_LIST_FIRST_LINE, 5);
        assert_eq!(ITEM_LIST_LAST_LINE, 20);
        assert_eq!(ITEM_LINE_STRIDE, 4);
    }

    // ---- TalkId ----

    #[test]
    fn test_talk_id_round_trip() {
        for i in 0..24 {
            assert!(TalkId::from_u8(i as u8).is_some(), "TalkId::from_u8({})", i);
        }
        assert_eq!(TalkId::from_u8(24), None);
    }

    #[test]
    fn test_talk_id_is_shop() {
        assert!(TalkId::SmithBuy.is_shop());
        assert!(TalkId::WitchSell.is_shop());
        assert!(!TalkId::Smith.is_shop());
        assert!(!TalkId::Gossip.is_shop());
    }

    // ---- StoreState ----

    #[test]
    fn test_store_state_open_close() {
        let mut s = StoreState::new();
        assert!(!s.is_store_active());
        s.open_store(TalkId::SmithBuy);
        assert!(s.is_store_active());
        s.close_store();
        assert!(!s.is_store_active());
    }

    // ---- UiFlags ----

    #[test]
    fn test_ui_flags_bitor() {
        let f = UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER;
        assert!(f.has_any_of(UiFlags::ALIGN_CENTER));
        assert!(f.has_any_of(UiFlags::COLOR_WHITE));
        assert!(!f.has_any_of(UiFlags::COLOR_RED));
    }

    // ---- STextLine ----

    #[test]
    fn test_stext_line_default_and_clear() {
        let mut l = STextLine::new();
        assert_eq!(l.curs_id, -1);
        assert!(!l.is_selectable());
        assert!(!l.is_divider());
        assert!(!l.has_text());
        l.text = "hi".into();
        l.line_type = STextType::Selectable;
        assert!(l.has_text());
        assert!(l.is_selectable());
        l.clear();
        assert!(!l.has_text());
        assert!(!l.is_selectable());
    }

    // ---- towner_name ----

    #[test]
    fn test_towner_names() {
        assert_eq!(towner_name(TownerId::Smith), "Griswold");
        assert_eq!(towner_name(TownerId::Healer), "Pepin");
        assert_eq!(towner_name(TownerId::Witch), "Adria");
        assert_eq!(towner_name(TownerId::Pegboy), "Wirt");
        assert_eq!(towner_name(TownerId::Deadguy), "");
        assert_eq!(towner_name(TownerId::None), "");
    }

    // ---- StoreUi construction + line math ----

    #[test]
    fn test_store_ui_new_is_clean() {
        let ui = StoreUi::new();
        assert_eq!(ui.active_store, TalkId::None);
        assert_eq!(ui.current_text_line, -1);
        assert_eq!(ui.text_lines.len(), NUM_STORE_LINES);
        assert_eq!(ui.player_items.len(), NUM_PLAYER_ITEMS);
        assert_eq!(ui.countdown_scroll_up, -1);
        assert!(!ui.is_player_in_store());
    }

    #[test]
    fn test_back_button_line_small_font() {
        let mut ui = StoreUi::new();
        ui.is_small_font_tall = false;
        assert_eq!(ui.back_button_line(), 22);
        ui.has_scrollbar = true;
        assert_eq!(ui.back_button_line(), 22);
    }

    #[test]
    fn test_back_button_line_tall_font() {
        let mut ui = StoreUi::new();
        ui.is_small_font_tall = true;
        ui.has_scrollbar = false;
        assert_eq!(ui.back_button_line(), 20);
        ui.has_scrollbar = true;
        assert_eq!(ui.back_button_line(), 21);
    }

    #[test]
    fn test_line_height_and_text_height() {
        let mut ui = StoreUi::new();
        ui.is_small_font_tall = false;
        assert_eq!(ui.line_height(), 12);
        assert_eq!(ui.text_height(), 12);
        ui.is_small_font_tall = true;
        assert_eq!(ui.line_height(), 13);
        assert_eq!(ui.text_height(), 18);
    }

    #[test]
    fn test_calculate_line_heights_small_font() {
        let mut ui = StoreUi::new();
        ui.is_small_font_tall = false;
        ui.calculate_line_heights();
        for i in 0..NUM_STORE_LINES {
            assert_eq!(ui.text_lines[i].y, (i as i32) * 12);
        }
    }

    #[test]
    fn test_calculate_line_heights_tall_font_simple() {
        let mut ui = StoreUi::new();
        ui.is_small_font_tall = true;
        ui.calculate_line_heights();
        for i in 0..NUM_STORE_LINES {
            assert_eq!(ui.text_lines[i].y, (i as i32) * LARGE_LINE_HEIGHT);
        }
    }

    // ---- clear_s_text / add_s_text / add_s_line ----

    #[test]
    fn test_add_s_text_marks_selectable() {
        let mut ui = StoreUi::new();
        ui.add_s_text(0, 5, "Hello", UiFlags::COLOR_WHITE, true, -1, false);
        assert!(ui.text_lines[5].is_selectable());
        assert_eq!(ui.text_lines[5].text, "Hello");
        ui.add_s_text(0, 5, "World", UiFlags::COLOR_WHITE, false, -1, false);
        assert!(!ui.text_lines[5].is_selectable());
        assert_eq!(ui.text_lines[5].text, "World");
    }

    #[test]
    fn test_add_s_line_marks_divider() {
        let mut ui = StoreUi::new();
        ui.add_s_text(0, 3, "x", UiFlags::COLOR_WHITE, true, -1, false);
        ui.add_s_line(3);
        assert!(ui.text_lines[3].is_divider());
        assert!(!ui.text_lines[3].has_text());
        assert_eq!(ui.text_lines[3].curs_id, -1);
    }

    #[test]
    fn test_add_s_text_val_sets_value() {
        let mut ui = StoreUi::new();
        ui.add_s_text_val(7, 1234);
        assert_eq!(ui.text_lines[7]._sval, 1234);
    }

    #[test]
    fn test_add_s_text_out_of_bounds_ignored() {
        let mut ui = StoreUi::new();
        ui.add_s_text(0, 99, "x", UiFlags::COLOR_WHITE, true, -1, false);
        ui.add_s_text_val(99, 5);
        // No panic.
    }

    // ---- add_options_back_button / add_item_list_back_button ----

    #[test]
    fn test_add_options_back_button_small_font() {
        let mut ui = StoreUi::new();
        ui.add_options_back_button();
        assert!(ui.text_lines[22].is_selectable());
        assert_eq!(ui.text_lines[22].text, "Back");
        assert_eq!(ui.text_lines[22]._syoff, 6);
    }

    #[test]
    fn test_add_item_list_back_button_selectable() {
        let mut ui = StoreUi::new();
        ui.add_item_list_back_button(true);
        let line = ui.back_button_line() as usize;
        assert!(ui.text_lines[line].is_selectable());
        assert!(ui.text_lines[line - 1].is_divider());
    }

    #[test]
    fn test_add_item_list_back_button_non_selectable_tall_font() {
        let mut ui = StoreUi::new();
        ui.is_small_font_tall = true;
        ui.add_item_list_back_button(false);
        let line = ui.back_button_line() as usize;
        assert!(!ui.text_lines[line].is_selectable());
        assert!(ui.text_lines[line].flags.has_any_of(UiFlags::ALIGN_RIGHT));
    }

    // ---- gold helpers ----

    #[test]
    fn test_total_player_gold_and_can_afford() {
        assert_eq!(StoreUi::total_player_gold(100, 50), 150);
        assert!(StoreUi::player_can_afford(150, 100, 50));
        assert!(!StoreUi::player_can_afford(151, 100, 50));
    }

    // ---- calculate_store_repair_cost ----

    #[test]
    fn test_repair_cost_normal_item() {
        let it = mk_weapon(100, 30, 50);
        assert_eq!(calculate_store_repair_cost(&it), 20);
    }

    #[test]
    fn test_repair_cost_min_one() {
        let it = mk_weapon(1, 49, 50);
        assert_eq!(calculate_store_repair_cost(&it), 1);
    }

    #[test]
    fn test_repair_cost_identified_magic() {
        let mut it = mk_weapon(1000, 25, 50);
        it.quality = ItemQuality::Magic;
        it.identified = true;
        assert_eq!(calculate_store_repair_cost(&it), 75);
    }

    #[test]
    fn test_repair_cost_zero_when_full_durability() {
        let it = mk_weapon(100, 50, 50);
        assert_eq!(calculate_store_repair_cost(&it), 0);
    }

    #[test]
    fn test_repair_cost_indestructible() {
        let mut it = mk_weapon(100, 100, DUR_INDESTRUCTIBLE);
        it.max_durability = DUR_INDESTRUCTIBLE;
        it.durability = 1;
        assert_eq!(calculate_store_repair_cost(&it), 0);
    }

    #[test]
    fn test_should_repair_item() {
        assert!(should_repair_item(&mk_weapon(100, 30, 50)));
        assert!(!should_repair_item(&mk_weapon(100, 50, 50)));
    }

    // ---- calculate_store_recharge_cost ----

    #[test]
    fn test_recharge_cost_basic() {
        let mut it = mk_weapon(200, 50, 50);
        it.item_type = ItemType::Staff;
        it.misc_id = ItemMiscId::Staff;
        it.charges = 5;
        it.max_charges = 10;
        assert_eq!(calculate_store_recharge_cost(&it, 100), 75);
    }

    #[test]
    fn test_recharge_cost_zero_when_full() {
        let mut it = mk_weapon(200, 50, 50);
        it.item_type = ItemType::Staff;
        it.charges = 10;
        it.max_charges = 10;
        assert_eq!(calculate_store_recharge_cost(&it, 100), 0);
    }

    // ---- calculate_player_sell_value ----

    #[test]
    fn test_player_sell_value_normal() {
        let it = mk_weapon(300, 50, 50);
        assert_eq!(calculate_player_sell_value(&it), 75);
    }

    #[test]
    fn test_player_sell_value_min_one() {
        let it = mk_weapon(1, 50, 50);
        assert_eq!(calculate_player_sell_value(&it), 1);
    }

    #[test]
    fn test_player_sell_value_uses_identified_value() {
        let mut it = mk_weapon(300, 50, 50);
        it.quality = ItemQuality::Magic;
        it.identified = true;
        it.identified_value = 1000;
        assert_eq!(calculate_player_sell_value(&it), 250);
    }

    // ---- calculate_boy_price ----

    #[test]
    fn test_boy_price_diablo() {
        assert_eq!(calculate_boy_price(&mk_weapon(1000, 50, 50), GameMode::Diablo), 1500);
    }

    #[test]
    fn test_boy_price_hellfire() {
        assert_eq!(calculate_boy_price(&mk_weapon(1000, 50, 50), GameMode::Hellfire), 750);
    }

    // ---- eligibility predicates ----

    #[test]
    fn test_smith_repair_ok() {
        assert!(smith_repair_ok(&mk_weapon(100, 30, 50)));
        assert!(!smith_repair_ok(&mk_weapon(100, 50, 50)));
        assert!(!smith_repair_ok(&Item::default()));
    }

    #[test]
    fn test_witch_recharge_ok() {
        let mut it = mk_weapon(200, 50, 50);
        it.item_type = ItemType::Staff;
        it.charges = 5;
        it.max_charges = 10;
        assert!(witch_recharge_ok(&it));
        it.charges = 10;
        assert!(!witch_recharge_ok(&it));
    }

    #[test]
    fn test_id_item_ok() {
        let mut it = mk_weapon(100, 50, 50);
        it.quality = ItemQuality::Magic;
        it.identified = false;
        assert!(id_item_ok(&it));
        it.identified = true;
        assert!(!id_item_ok(&it));
        it.quality = ItemQuality::Normal;
        it.identified = false;
        assert!(!id_item_ok(&it));
    }

    // ---- item_text_color ----

    #[test]
    fn test_item_text_color_variants() {
        let mut it = mk_weapon(100, 50, 50);
        it.stat_flag = false;
        assert_eq!(item_text_color(&it), UiFlags::COLOR_RED);
        it.stat_flag = true;
        it.quality = ItemQuality::Magic;
        assert_eq!(item_text_color(&it), UiFlags::COLOR_BLUE);
        it.quality = ItemQuality::Unique;
        assert_eq!(item_text_color(&it), UiFlags::COLOR_GOLD);
        it.quality = ItemQuality::Normal;
        assert_eq!(item_text_color(&it), UiFlags::COLOR_WHITE);
    }

    // ---- item_index_from_line ----

    #[test]
    fn test_item_index_from_line() {
        assert_eq!(item_index_from_line(8, 9, 5), 9);
        assert_eq!(item_index_from_line(0, 5, 5), 0);
        assert_eq!(item_index_from_line(0, 9, 5), 1);
        assert_eq!(item_index_from_line(4, 13, 5), 6);
    }

    // ---- print_store_item ----

    #[test]
    fn test_print_store_item_weapon() {
        let mut ui = StoreUi::new();
        let it = mk_weapon(100, 30, 50);
        let lines_written = print_store_item(&mut ui, &it, 10, UiFlags::COLOR_WHITE, false);
        assert!(lines_written >= 1);
        let text = &ui.text_lines[10].text;
        assert!(text.contains("Damage: 1-4"), "got: {}", text);
        assert!(text.contains("Dur: 30/50"), "got: {}", text);
    }

    #[test]
    fn test_print_store_item_armor() {
        let mut ui = StoreUi::new();
        let it = mk_armor(100, 50, 50);
        print_store_item(&mut ui, &it, 5, UiFlags::COLOR_WHITE, false);
        let text = &ui.text_lines[5].text;
        assert!(text.contains("Armor: 10"), "got: {}", text);
        assert!(text.contains("Dur: 50/50"), "got: {}", text);
    }

    #[test]
    fn test_print_store_item_indestructible() {
        let mut ui = StoreUi::new();
        let mut it = mk_weapon(100, 1, DUR_INDESTRUCTIBLE);
        it.max_durability = DUR_INDESTRUCTIBLE;
        print_store_item(&mut ui, &it, 5, UiFlags::COLOR_WHITE, false);
        let text = &ui.text_lines[5].text;
        assert!(text.contains("Indestructible"), "got: {}", text);
    }

    #[test]
    fn test_print_store_item_requirements() {
        let mut ui = StoreUi::new();
        let mut it = mk_weapon(100, 50, 50);
        it.required_str = 30;
        it.required_dex = 15;
        print_store_item(&mut ui, &it, 5, UiFlags::COLOR_WHITE, false);
        let text = &ui.text_lines[5].text;
        assert!(text.contains("Required:"), "got: {}", text);
        assert!(text.contains("30 Str"), "got: {}", text);
        assert!(text.contains("15 Dex"), "got: {}", text);
    }

    // ---- Start* builders ----

    #[test]
    fn test_start_smith_layout() {
        let mut ui = StoreUi::new();
        ui.start_smith();
        assert_eq!(ui.text_lines[1].text, "Welcome to the");
        assert_eq!(ui.text_lines[10].text, "Talk to Griswold");
        assert!(ui.text_lines[10].is_selectable());
        assert!(ui.text_lines[12].is_selectable());
        assert!(ui.text_lines[20].is_selectable());
        assert!(ui.text_lines[5].is_divider());
        assert_eq!(ui.current_item_index, 20);
    }

    #[test]
    fn test_start_witch_layout() {
        let mut ui = StoreUi::new();
        ui.start_witch();
        assert_eq!(ui.text_lines[2].text, "Witch's shack");
        assert_eq!(ui.text_lines[12].text, "Talk to Adria");
        assert!(ui.text_lines[18].is_selectable());
    }

    #[test]
    fn test_start_healer_layout() {
        let mut ui = StoreUi::new();
        ui.start_healer();
        assert_eq!(ui.text_lines[3].text, "Healer's home");
        assert_eq!(ui.text_lines[12].text, "Talk to Pepin");
    }

    #[test]
    fn test_start_storyteller_layout() {
        let mut ui = StoreUi::new();
        ui.start_storyteller();
        assert_eq!(ui.text_lines[2].text, "The Town Elder");
        assert_eq!(ui.text_lines[14].text, "Identify an item");
    }

    #[test]
    fn test_start_boy_with_item() {
        let mut ui = StoreUi::new();
        ui.spawn_boy(5);
        ui.start_boy();
        assert_eq!(ui.text_lines[2].text, "Wirt the Peg-legged boy");
        assert_eq!(ui.text_lines[18].text, "What have you got?");
        assert!(ui.text_lines[18].is_selectable());
    }

    #[test]
    fn test_start_boy_without_item() {
        let mut ui = StoreUi::new();
        ui.start_boy();
        assert_eq!(ui.text_lines[12].text, "Talk to Wirt");
        assert_eq!(ui.text_lines[18].text, "Say goodbye");
    }

    #[test]
    fn test_start_tavern_and_barmaid_and_drunk() {
        let mut ui = StoreUi::new();
        ui.start_tavern();
        assert_eq!(ui.text_lines[3].text, "Rising Sun");
        ui.start_barmaid();
        assert_eq!(ui.text_lines[14].text, "Access Storage");
        ui.start_drunk();
        assert_eq!(ui.text_lines[2].text, "Farnham the Drunk");
    }

    // ---- select_first_line ----

    #[test]
    fn test_select_first_line() {
        let mut ui = StoreUi::new();
        ui.start_smith();
        ui.select_first_line();
        assert_eq!(ui.current_text_line, 10);
    }

    // ---- store_no_money / store_no_room ----

    #[test]
    fn test_store_no_money() {
        let mut ui = StoreUi::new();
        ui.old_active_store = TalkId::SmithBuy;
        ui.store_no_money();
        assert!(ui.render_gold);
        assert!(ui.text_lines[14].is_selectable());
        assert_eq!(ui.text_lines[14].text, "You do not have enough gold");
        assert_eq!(ui.active_store, TalkId::SmithBuy);
    }

    #[test]
    fn test_store_no_room() {
        let mut ui = StoreUi::new();
        ui.old_active_store = TalkId::SmithBuy;
        ui.store_no_room();
        assert!(ui.text_lines[14].is_selectable());
        assert!(ui.text_lines[14].text.contains("room"));
    }

    // ---- start_store_dispatch ----

    #[test]
    fn test_start_store_dispatch_smith() {
        let mut ui = StoreUi::new();
        assert!(ui.start_store_dispatch(TalkId::Smith));
        assert_eq!(ui.active_store, TalkId::Smith);
        assert_eq!(ui.current_text_line, 10);
    }

    #[test]
    fn test_start_store_dispatch_smith_buy_empty_falls_back() {
        let mut ui = StoreUi::new();
        let entered = ui.start_store_dispatch(TalkId::SmithBuy);
        assert_eq!(ui.active_store, TalkId::SmithBuy);
        assert!(!entered);
    }

    #[test]
    fn test_start_store_dispatch_smith_buy_with_items() {
        let mut ui = StoreUi::new();
        ui.spawn_smith(10);
        assert!(ui.start_store_dispatch(TalkId::SmithBuy));
        assert!(ui.has_scrollbar);
        assert!(ui.render_gold);
        assert!(ui.current_item_index > 0);
    }

    #[test]
    fn test_start_store_dispatch_no_money() {
        let mut ui = StoreUi::new();
        ui.old_active_store = TalkId::Smith;
        ui.start_store_dispatch(TalkId::NoMoney);
        assert!(ui.text_lines[14].text.contains("gold"));
    }

    // ---- scroll_vendor_store ----

    #[test]
    fn test_scroll_vendor_store_fills_lines() {
        let mut ui = StoreUi::new();
        let items: Vec<Item> = (0..6).map(|i| {
            let mut it = mk_weapon(100 + i, 50, 50);
            it.name = format!("Item {}", i);
            it.cursor = i as u8;
            it
        }).collect();
        let limit = items.len() as i32;
        scroll_vendor_store(&mut ui, &items, limit, 0, true);
        assert!(ui.text_lines[5].is_selectable());
        assert_eq!(ui.text_lines[5].text, "Item 0");
        assert_eq!(ui.previous_scroll_pos, 5);
        assert!(ui.next_scroll_pos >= 5);
    }

    // ---- populate_player_items_* ----

    #[test]
    fn test_populate_player_items_sell_quarters_value() {
        let mut ui = StoreUi::new();
        let inv = vec![mk_weapon(300, 50, 50), mk_weapon(100, 50, 50)];
        ui.populate_player_items_sell(&inv, &[], smith_sell_ok);
        assert_eq!(ui.current_item_index, 2);
        assert_eq!(ui.player_items[0].value, 75);
        assert_eq!(ui.player_items[1].value, 25);
        assert_eq!(ui.player_item_indexes[0], 0);
        assert_eq!(ui.player_item_indexes[1], 1);
    }

    #[test]
    fn test_populate_player_items_sell_respects_48_cap() {
        let mut ui = StoreUi::new();
        let inv: Vec<Item> = (0..100).map(|i| {
            let mut it = mk_weapon(100 + i, 50, 50);
            it.name = format!("w{}", i);
            it
        }).collect();
        ui.populate_player_items_sell(&inv, &[], smith_sell_ok);
        assert_eq!(ui.current_item_index as usize, NUM_PLAYER_ITEMS);
    }

    #[test]
    fn test_populate_player_items_sell_belt_negative_index() {
        let mut ui = StoreUi::new();
        let belt = vec![mk_weapon(200, 50, 50)];
        ui.populate_player_items_sell(&[], &belt, smith_sell_ok);
        assert_eq!(ui.current_item_index, 1);
        assert_eq!(ui.player_item_indexes[0], -1);
    }

    #[test]
    fn test_populate_player_items_repair() {
        let mut ui = StoreUi::new();
        let body = vec![mk_armor(500, 10, 50)];
        let inv = vec![mk_weapon(100, 20, 50)];
        ui.populate_player_items_repair(&body, &inv);
        assert_eq!(ui.current_item_index, 2);
        assert_eq!(ui.player_item_indexes[0], -1);
        assert_eq!(ui.player_item_indexes[1], 0);
    }

    #[test]
    fn test_populate_player_items_repair_skips_full_durability() {
        let mut ui = StoreUi::new();
        let inv = vec![mk_weapon(100, 50, 50)];
        ui.populate_player_items_repair(&[], &inv);
        assert_eq!(ui.current_item_index, 0);
    }

    #[test]
    fn test_populate_player_items_recharge() {
        let mut ui = StoreUi::new();
        let mut staff = mk_weapon(200, 50, 50);
        staff.item_type = ItemType::Staff;
        staff.misc_id = ItemMiscId::Staff;
        staff.charges = 5;
        staff.max_charges = 10;
        ui.populate_player_items_recharge(&staff, &[], 100);
        assert_eq!(ui.current_item_index, 1);
        assert_eq!(ui.player_item_indexes[0], -1);
    }

    #[test]
    fn test_populate_player_items_identify() {
        let mut ui = StoreUi::new();
        let mut magic = mk_weapon(100, 50, 50);
        magic.quality = ItemQuality::Magic;
        magic.identified = false;
        let body: Vec<Item> = vec![magic.clone()];
        ui.populate_player_items_identify(&body, &[]);
        assert_eq!(ui.current_item_index, 1);
        assert_eq!(ui.player_items[0].value, IDENTIFY_COST);
        assert_eq!(ui.player_items[0].identified_value, IDENTIFY_COST);
    }

    // ---- StoreESC ----

    #[test]
    fn test_store_esc_from_smith_closes() {
        let mut ui = StoreUi::new();
        ui.start_store_dispatch(TalkId::Smith);
        let result = ui.store_esc();
        assert_eq!(result, None);
        assert_eq!(ui.active_store, TalkId::None);
    }

    #[test]
    fn test_store_esc_from_smith_buy_goes_back_to_smith() {
        let mut ui = StoreUi::new();
        ui.spawn_smith(10);
        ui.start_store_dispatch(TalkId::SmithBuy);
        let result = ui.store_esc();
        assert_eq!(result, Some(TalkId::Smith));
        assert_eq!(ui.active_store, TalkId::Smith);
        assert_eq!(ui.current_text_line, 12);
    }

    #[test]
    fn test_store_esc_from_confirm_restores_old() {
        let mut ui = StoreUi::new();
        ui.spawn_smith(10);
        ui.old_active_store = TalkId::SmithBuy;
        ui.active_store = TalkId::Confirm;
        ui.old_text_line = 6;
        ui.old_scroll_pos = 0;
        let result = ui.store_esc();
        assert_eq!(result, Some(TalkId::SmithBuy));
    }

    // ---- StoreUp / StoreDown ----

    #[test]
    fn test_store_down_advances() {
        let mut ui = StoreUi::new();
        ui.start_smith();
        ui.select_first_line();
        assert_eq!(ui.current_text_line, 10);
        ui.store_down();
        assert_eq!(ui.current_text_line, 12);
        ui.store_down();
        assert_eq!(ui.current_text_line, 14);
    }

    #[test]
    fn test_store_up_wraps_to_last_selectable() {
        let mut ui = StoreUi::new();
        ui.start_smith();
        ui.select_first_line();
        assert_eq!(ui.current_text_line, 10);
        ui.store_up();
        assert_eq!(ui.current_text_line, 20);
    }

    #[test]
    fn test_store_up_down_skip_non_selectable() {
        let mut ui = StoreUi::new();
        ui.start_smith();
        ui.select_first_line();
        ui.store_down();
        assert_eq!(ui.current_text_line, 12);
    }

    // ---- take_gold / take_plrs_money ----

    #[test]
    fn test_take_gold_skip_max_piles() {
        let mut inv = vec![mk_gold(5000), mk_gold(1000)];
        let remaining = take_gold(&mut inv, 400, true);
        assert_eq!(remaining, 0);
        assert_eq!(inv[0].value, 5000);
        assert_eq!(inv[1].value, 600);
    }

    #[test]
    fn test_take_gold_clears_pile_when_used_up() {
        let mut inv = vec![mk_gold(500)];
        let remaining = take_gold(&mut inv, 500, false);
        assert_eq!(remaining, 0);
        assert!(item_is_empty(&inv[0]));
    }

    #[test]
    fn test_take_gold_returns_shortfall() {
        let mut inv = vec![mk_gold(100)];
        let remaining = take_gold(&mut inv, 500, false);
        assert_eq!(remaining, 400);
    }

    #[test]
    fn test_take_plrs_money_drains_player_gold_first() {
        let mut player_gold = 200;
        let mut inv: Vec<Item> = Vec::new();
        let mut stash = 0;
        take_plrs_money(&mut player_gold, &mut inv, &mut stash, 100);
        assert_eq!(player_gold, 100);
    }

    #[test]
    fn test_take_plrs_money_falls_through_to_stash() {
        let mut player_gold = 0;
        let mut inv: Vec<Item> = Vec::new();
        let mut stash = 1000;
        take_plrs_money(&mut player_gold, &mut inv, &mut stash, 400);
        assert_eq!(stash, 600);
    }

    // ---- store_auto_place / store_gold_fit ----

    #[test]
    fn test_store_auto_place_can_fit() {
        assert_eq!(store_auto_place_can_fit(true, 0, 0), AutoPlaceResult::Placed);
        assert_eq!(store_auto_place_can_fit(false, 1, 0), AutoPlaceResult::Placed);
        assert_eq!(store_auto_place_can_fit(false, 0, 1), AutoPlaceResult::Placed);
        assert_eq!(store_auto_place_can_fit(false, 0, 0), AutoPlaceResult::NoRoom);
    }

    #[test]
    fn test_store_auto_place_pick_slot_priority() {
        assert_eq!(store_auto_place_pick_slot(true, 5, 5), Some(AutoPlaceSlot::Equipped));
        assert_eq!(store_auto_place_pick_slot(false, 5, 5), Some(AutoPlaceSlot::Belt));
        assert_eq!(store_auto_place_pick_slot(false, 0, 5), Some(AutoPlaceSlot::Inventory));
        assert_eq!(store_auto_place_pick_slot(false, 0, 0), None);
    }

    #[test]
    fn test_store_gold_fit() {
        assert!(store_gold_fit(10000, 2, 3, 0));
        assert!(store_gold_fit(40000, 1, 1, 40000));
        assert!(!store_gold_fit(50000, 1, 1, 40000));
    }

    #[test]
    fn test_room_for_gold() {
        assert_eq!(room_for_gold(5, 40), 35 * MAX_GOLD);
        assert_eq!(room_for_gold(40, 40), 0);
        assert_eq!(room_for_gold(50, 40), 0);
    }

    // ---- spawn_* ----

    #[test]
    fn test_spawn_smith_count() {
        let mut ui = StoreUi::new();
        ui.spawn_smith(10);
        assert_eq!(ui.smith_items.len(), NUM_SMITH_BASIC_ITEMS);
        assert!(ui.smith_items.iter().all(|i| !item_is_empty(i)));
    }

    #[test]
    fn test_spawn_witch_count() {
        let mut ui = StoreUi::new();
        ui.spawn_witch(10);
        assert_eq!(ui.witch_items.len(), NUM_WITCH_ITEMS);
    }

    #[test]
    fn test_spawn_healer_count() {
        let mut ui = StoreUi::new();
        ui.spawn_healer(10);
        assert_eq!(ui.healer_items.len(), NUM_HEALER_ITEMS);
    }

    #[test]
    fn test_spawn_boy_creates_item() {
        let mut ui = StoreUi::new();
        ui.spawn_boy(7);
        assert!(!item_is_empty(&ui.boy_item));
        assert_eq!(ui.boy_item_level, 7);
        assert!(ui.boy_item.value >= 1000);
    }

    #[test]
    fn test_spawn_premium_diablo_count() {
        let mut ui = StoreUi::new();
        ui.spawn_premium(GameMode::Diablo, 5);
        assert_eq!(ui.premium_items.len(), NUM_SMITH_PREMIUM_ITEMS);
        assert_eq!(ui.premium_item_count as usize, NUM_SMITH_PREMIUM_ITEMS);
    }

    #[test]
    fn test_spawn_premium_hellfire_count() {
        let mut ui = StoreUi::new();
        ui.spawn_premium(GameMode::Hellfire, 5);
        assert_eq!(ui.premium_items.len(), NUM_SMITH_PREMIUM_ITEMS_HF);
    }

    #[test]
    fn test_setup_town_stores_populates_all() {
        let mut ui = StoreUi::new();
        ui.setup_town_stores(GameMode::Diablo, 10);
        assert!(!ui.smith_items.is_empty());
        assert!(!ui.witch_items.is_empty());
        assert!(!ui.healer_items.is_empty());
        assert!(!item_is_empty(&ui.boy_item));
        assert!(!ui.premium_items.is_empty());
    }

    #[test]
    fn test_init_stores_clears_everything() {
        let mut ui = StoreUi::new();
        ui.spawn_smith(10);
        ui.spawn_boy(5);
        ui.init_stores();
        assert!(ui.smith_items.is_empty());
        assert!(ui.witch_items.is_empty());
        assert!(ui.healer_items.is_empty());
        assert!(ui.premium_items.is_empty());
        assert!(item_is_empty(&ui.boy_item));
        assert_eq!(ui.premium_item_level, 1);
        assert_eq!(ui.active_store, TalkId::None);
    }

    // ---- start_smith_sell / start_smith_repair / start_witch_recharge ----

    #[test]
    fn test_start_smith_sell_nothing_to_sell() {
        let mut ui = StoreUi::new();
        ui.start_smith_sell();
        assert_eq!(ui.text_lines[1].text, "You have nothing I want.");
        assert!(!ui.has_scrollbar);
    }

    #[test]
    fn test_start_smith_sell_with_items() {
        let mut ui = StoreUi::new();
        let inv = vec![mk_weapon(300, 50, 50)];
        ui.populate_player_items_sell(&inv, &[], smith_sell_ok);
        ui.start_smith_sell();
        assert!(ui.has_scrollbar);
        assert_eq!(ui.text_lines[1].text, "Which item is for sale?");
    }

    #[test]
    fn test_start_smith_repair_nothing_to_repair() {
        let mut ui = StoreUi::new();
        ui.start_smith_repair();
        assert_eq!(ui.text_lines[1].text, "You have nothing to repair.");
    }

    #[test]
    fn test_start_witch_recharge_nothing() {
        let mut ui = StoreUi::new();
        ui.start_witch_recharge();
        assert_eq!(ui.text_lines[1].text, "You have nothing to recharge.");
    }

    #[test]
    fn test_start_storyteller_identify_nothing() {
        let mut ui = StoreUi::new();
        ui.start_storyteller_identify();
        assert_eq!(ui.text_lines[1].text, "You have nothing to identify.");
    }

    // ---- store_confirm ----

    #[test]
    fn test_store_confirm_buy_prompt() {
        let mut ui = StoreUi::new();
        ui.temp_item = mk_weapon(500, 50, 50);
        ui.old_active_store = TalkId::SmithBuy;
        ui.store_confirm();
        assert_eq!(ui.text_lines[15].text, "Are you sure you want to buy this item?");
        assert!(ui.text_lines[18].is_selectable());
        assert!(ui.text_lines[20].is_selectable());
    }

    #[test]
    fn test_store_confirm_sell_prompt() {
        let mut ui = StoreUi::new();
        ui.temp_item = mk_weapon(500, 50, 50);
        ui.old_active_store = TalkId::SmithSell;
        ui.store_confirm();
        assert_eq!(ui.text_lines[15].text, "Are you sure you want to sell this item?");
    }

    // ---- s_start_boy_buy ----

    #[test]
    fn test_s_start_boy_buy_layout() {
        let mut ui = StoreUi::new();
        ui.spawn_boy(5);
        ui.s_start_boy_buy();
        assert!(ui.text_lines[10].is_selectable());
        let back = ui.back_button_line() as usize;
        assert_eq!(ui.text_lines[back].text, "Leave");
        assert!(ui.text_lines[back].is_selectable());
    }

    // ---- legacy-API regression ----

    #[test]
    fn test_legacy_calculate_sell_price_still_works() {
        let mut it = Item::default();
        it.value = 100;
        assert_eq!(calculate_sell_price(&it, 2), 50);
    }

    #[test]
    fn test_legacy_buy_item_trade() {
        let mut player = StorePlayer { gold: 200, level: 5 };
        let mut shop = ShopInventory::new(TalkId::SmithBuy);
        let mut it = Item::default();
        it.value = 50;
        shop.add_item(it).unwrap();
        let result = buy_item(&mut player, &mut shop, 0);
        assert_eq!(result, TradeResult::Success);
        assert!(shop.is_empty());
    }

    #[test]
    fn test_legacy_setup_town_stores_fn() {
        let mut manager = StoreManager::new();
        setup_town_stores(&mut manager, GameMode::Diablo, 5);
        assert!(!manager.smith_shop.is_empty());
    }
}
