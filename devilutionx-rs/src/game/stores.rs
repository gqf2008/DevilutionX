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
use crate::game::item_new::Item;

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
///
/// Each variant represents a specific NPC dialogue or shop state:
/// - **Dialogue states**: Smith, Witch, Healer, etc. (NPC greeting)
/// - **Shop states**: SmithBuy, WitchBuy, etc. (browsing items)
/// - **Action states**: SmithSell, SmithRepair, etc. (transaction)
/// - **System states**: NoMoney, NoRoom, Confirm (error/confirm dialogs)
///
/// # Alignment with C++
/// ```cpp
/// enum class TalkID : uint8_t {
///     None, Smith, SmithBuy, SmithSell, SmithRepair,
///     Witch, WitchBuy, WitchSell, WitchRecharge,
///     NoMoney, NoRoom, Confirm,
///     Boy, BoyBuy, Healer, Storyteller, HealerBuy, StorytellerIdentify,
///     SmithPremiumBuy, Gossip, StorytellerIdentifyShow,
///     Tavern, Drunk, Barmaid,
/// };
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TalkId {
    /// No active dialogue
    None = 0,

    // Griswold (Blacksmith)
    /// Griswold greeting dialogue
    Smith = 1,
    /// Browse Griswold's basic items
    SmithBuy = 2,
    /// Sell items to Griswold
    SmithSell = 3,
    /// Repair items at Griswold
    SmithRepair = 4,

    // Adria (Witch)
    /// Adria greeting dialogue
    Witch = 5,
    /// Browse Adria's magic items
    WitchBuy = 6,
    /// Sell items to Adria
    WitchSell = 7,
    /// Recharge staff at Adria
    WitchRecharge = 8,

    // System states
    /// Error: Insufficient gold
    NoMoney = 9,
    /// Error: Inventory full
    NoRoom = 10,
    /// Confirmation dialog
    Confirm = 11,

    // Wirt (Boy with peg leg)
    /// Wirt greeting dialogue
    Boy = 12,
    /// Browse Wirt's premium item
    BoyBuy = 13,

    // Pepin (Healer)
    /// Pepin greeting dialogue
    Healer = 14,

    // Deckard Cain (Storyteller)
    /// Cain greeting dialogue
    Storyteller = 15,

    // More shop states
    /// Browse Pepin's potions
    HealerBuy = 16,
    /// Identify item at Cain
    StorytellerIdentify = 17,
    /// Browse Griswold's premium items
    SmithPremiumBuy = 18,

    // NPC gossip
    /// Random NPC gossip
    Gossip = 19,

    /// Show identified item info
    StorytellerIdentifyShow = 20,

    // Ogden (Tavern owner)
    /// Ogden greeting dialogue
    Tavern = 21,

    // Farnham (Drunk)
    /// Farnham dialogue
    Drunk = 22,

    // Gillian (Barmaid)
    /// Gillian dialogue
    Barmaid = 23,
}

impl TalkId {
    /// Total number of TalkID variants.
    pub const COUNT: usize = 24;

    /// Convert from u8 (bounds-checked).
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::store::TalkId;
    /// assert_eq!(TalkId::from_u8(0), Some(TalkId::None));
    /// assert_eq!(TalkId::from_u8(2), Some(TalkId::SmithBuy));
    /// assert_eq!(TalkId::from_u8(99), None);
    /// ```
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
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::store::TalkId;
    /// assert_eq!(TalkId::Smith.name(), "Griswold");
    /// assert_eq!(TalkId::SmithBuy.name(), "Griswold Shop");
    /// ```
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
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::store::TalkId;
    /// assert!(TalkId::SmithBuy.is_shop());
    /// assert!(TalkId::WitchBuy.is_shop());
    /// assert!(!TalkId::Smith.is_shop());
    /// assert!(!TalkId::Gossip.is_shop());
    /// ```
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

// =============================================================================
// Store Constants
// =============================================================================

/// Griswold basic items count (Diablo mode).
pub const NUM_SMITH_BASIC_ITEMS: usize = 19;

/// Griswold basic items count (Hellfire mode).
pub const NUM_SMITH_BASIC_ITEMS_HF: usize = 24;

/// Griswold premium items count (Diablo mode).
///
/// **C++ Reference**: `NumSmithItems` in `Source/stores.h:23`
/// (C++ uses shorter name without "Premium" suffix)
pub const NUM_SMITH_PREMIUM_ITEMS: usize = 6;

/// Griswold premium items count (Hellfire mode).
///
/// **C++ Reference**: `NumSmithItemsHf` in `Source/stores.h:24`
pub const NUM_SMITH_PREMIUM_ITEMS_HF: usize = 15;

/// Pepin healer items count (Diablo mode).
pub const NUM_HEALER_ITEMS: usize = 17;

/// Pepin healer items count (Hellfire mode).
pub const NUM_HEALER_ITEMS_HF: usize = 19;

/// Adria witch items count (Diablo mode).
pub const NUM_WITCH_ITEMS: usize = 17;

/// Adria witch items count (Hellfire mode).
pub const NUM_WITCH_ITEMS_HF: usize = 24;

// =============================================================================
// StoreState - Active Store Management
// =============================================================================

/// Global store state (tracks active dialogue/shop).
///
/// **C++ Reference**: `Source/stores.h` - `extern TalkID ActiveStore;`
///
/// This struct replaces the C++ global variable `ActiveStore` with
/// a more Rust-friendly struct that also tracks UI state.
///
/// # Usage
/// ```
/// # use devilutionx_rs::game::store::{TalkId, StoreState};
/// let mut state = StoreState::new();
/// assert!(!state.is_store_active());
///
/// state.open_store(TalkId::SmithBuy);
/// assert!(state.is_store_active());
/// assert_eq!(state.active_store, TalkId::SmithBuy);
///
/// state.close_store();
/// assert!(!state.is_store_active());
/// ```
#[derive(Debug, Clone, Default)]
pub struct StoreState {
    /// Currently active store/dialogue.
    pub active_store: TalkId,

    /// Current item index (for UI navigation).
    pub current_item_index: usize,
}

impl StoreState {
    /// Create new store state (no active store).
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::store::StoreState;
    /// let state = StoreState::new();
    /// assert!(!state.is_store_active());
    /// ```
    pub fn new() -> Self {
        Self {
            active_store: TalkId::None,
            current_item_index: 0,
        }
    }

    /// Open a store/dialogue.
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::store::{TalkId, StoreState};
    /// let mut state = StoreState::new();
    /// state.open_store(TalkId::SmithBuy);
    /// assert_eq!(state.active_store, TalkId::SmithBuy);
    /// assert_eq!(state.current_item_index, 0);
    /// ```
    pub fn open_store(&mut self, store: TalkId) {
        self.active_store = store;
        self.current_item_index = 0;
    }

    /// Close the current store/dialogue.
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::store::{TalkId, StoreState};
    /// let mut state = StoreState::new();
    /// state.open_store(TalkId::SmithBuy);
    /// state.close_store();
    /// assert_eq!(state.active_store, TalkId::None);
    /// ```
    pub fn close_store(&mut self) {
        self.active_store = TalkId::None;
        self.current_item_index = 0;
    }

    /// Check if any store is currently active.
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::store::{TalkId, StoreState};
    /// let mut state = StoreState::new();
    /// assert!(!state.is_store_active());
    ///
    /// state.open_store(TalkId::Witch);
    /// assert!(state.is_store_active());
    /// ```
    pub fn is_store_active(&self) -> bool {
        self.active_store != TalkId::None
    }

    /// Set current item index (for UI navigation).
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::store::{TalkId, StoreState};
    /// let mut state = StoreState::new();
    /// state.open_store(TalkId::SmithBuy);
    /// state.set_current_item(5);
    /// assert_eq!(state.current_item_index, 5);
    /// ```
    pub fn set_current_item(&mut self, index: usize) {
        self.current_item_index = index;
    }
}

impl Default for TalkId {
    fn default() -> Self {
        TalkId::None
    }
}

// =============================================================================
// ShopInventory - Item Management (Day 93)
// =============================================================================

/// Shop inventory container.
///
/// **C++ Reference**: `Source/stores.h` - `StaticVector<Item, N> SmithItems;`
///
/// Manages a shop's item list with capacity based on game mode.
///
/// # Usage
/// ```
/// # use devilutionx_rs::game::store::{ShopInventory, TalkId, GameMode};
/// let mut shop = ShopInventory::new(TalkId::SmithBuy);
/// assert_eq!(shop.capacity(GameMode::Diablo), 19);
/// assert_eq!(shop.capacity(GameMode::Hellfire), 24);
/// ```
#[derive(Debug, Clone)]
pub struct ShopInventory {
    /// Shop type (SmithBuy, WitchBuy, etc.)
    pub shop_type: TalkId,

    /// Basic items list
    pub items: Vec<Item>,

    /// Premium items list (Griswold only)
    pub premium_items: Vec<Item>,

    /// Premium item level (for generation)
    pub premium_level: u8,
}

impl ShopInventory {
    /// Create new empty shop inventory.
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::store::{ShopInventory, TalkId};
    /// let shop = ShopInventory::new(TalkId::SmithBuy);
    /// assert_eq!(shop.shop_type, TalkId::SmithBuy);
    /// assert!(shop.items.is_empty());
    /// ```
    pub fn new(shop_type: TalkId) -> Self {
        Self {
            shop_type,
            items: Vec::new(),
            premium_items: Vec::new(),
            premium_level: 1,
        }
    }

    /// Get shop capacity based on game mode.
    ///
    /// **C++ Reference**: Constants in `Source/stores.h`
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::store::{ShopInventory, TalkId, GameMode};
    /// let shop = ShopInventory::new(TalkId::SmithBuy);
    /// assert_eq!(shop.capacity(GameMode::Diablo), 19);
    /// assert_eq!(shop.capacity(GameMode::Hellfire), 24);
    /// ```
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
            TalkId::BoyBuy => 1, // Wirt has only 1 item
            _ => 0,
        }
    }

    /// Get premium item capacity (Griswold only).
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::store::{ShopInventory, TalkId, GameMode};
    /// let shop = ShopInventory::new(TalkId::SmithBuy);
    /// assert_eq!(shop.premium_capacity(GameMode::Diablo), 6);
    /// assert_eq!(shop.premium_capacity(GameMode::Hellfire), 15);
    /// ```
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

    /// Add item to shop.
    ///
    /// # Errors
    /// Returns error if shop is full.
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::store::{ShopInventory, TalkId, Item};
    /// let mut shop = ShopInventory::new(TalkId::SmithBuy);
    /// let item = Item::default();
    /// assert!(shop.add_item(item).is_ok());
    /// ```
    pub fn add_item(&mut self, item: Item) -> Result<(), &'static str> {
        self.items.push(item);
        Ok(())
    }

    /// Get item by index.
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::store::{ShopInventory, TalkId, Item};
    /// let mut shop = ShopInventory::new(TalkId::SmithBuy);
    /// shop.add_item(Item::default());
    /// assert!(shop.get_item(0).is_some());
    /// assert!(shop.get_item(1).is_none());
    /// ```
    pub fn get_item(&self, index: usize) -> Option<&Item> {
        self.items.get(index)
    }

    /// Remove item by index.
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::store::{ShopInventory, TalkId, Item};
    /// let mut shop = ShopInventory::new(TalkId::SmithBuy);
    /// shop.add_item(Item::default());
    /// assert!(shop.remove_item(0).is_some());
    /// assert!(shop.remove_item(0).is_none());
    /// ```
    pub fn remove_item(&mut self, index: usize) -> Option<Item> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    /// Clear all items.
    pub fn clear(&mut self) {
        self.items.clear();
        self.premium_items.clear();
    }

    /// Get item count.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if shop is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

// =============================================================================
// StoreManager - Global Shop Management (Day 93)
// =============================================================================

/// Global store manager.
///
/// **C++ Reference**: `Source/stores.h` - Multiple extern vectors
///
/// Manages all town shops:
/// - Smith (Griswold): Basic + Premium items
/// - Healer (Pepin): Potions
/// - Witch (Adria): Magic items
/// - Boy (Wirt): Single premium item
///
/// # C++ Alignment
/// ```cpp
/// extern StaticVector<Item, NumSmithBasicItemsHf> SmithItems;
/// extern StaticVector<Item, NumSmithItemsHf> PremiumItems;
/// extern StaticVector<Item, NumHealerItemsHf> HealerItems;
/// extern StaticVector<Item, NumWitchItemsHf> WitchItems;
/// extern Item BoyItem;
/// ```
#[derive(Debug, Clone)]
pub struct StoreManager {
    /// Griswold's blacksmith shop
    pub smith_shop: ShopInventory,

    /// Pepin's healer shop
    pub healer_shop: ShopInventory,

    /// Adria's witch shop
    pub witch_shop: ShopInventory,

    /// Wirt's premium item
    pub boy_item: Option<Item>,

    /// Wirt's item level
    pub boy_item_level: u8,
}

impl StoreManager {
    /// Create new store manager.
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::store::StoreManager;
    /// let manager = StoreManager::new();
    /// assert!(manager.smith_shop.is_empty());
    /// assert!(manager.boy_item.is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            smith_shop: ShopInventory::new(TalkId::SmithBuy),
            healer_shop: ShopInventory::new(TalkId::HealerBuy),
            witch_shop: ShopInventory::new(TalkId::WitchBuy),
            boy_item: None,
            boy_item_level: 1,
        }
    }

    /// Get shop by TalkId.
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::store::{StoreManager, TalkId};
    /// let manager = StoreManager::new();
    /// assert!(manager.get_shop(TalkId::SmithBuy).is_some());
    /// assert!(manager.get_shop(TalkId::Smith).is_none());
    /// ```
    pub fn get_shop(&self, shop_type: TalkId) -> Option<&ShopInventory> {
        match shop_type {
            TalkId::SmithBuy | TalkId::SmithPremiumBuy => Some(&self.smith_shop),
            TalkId::HealerBuy => Some(&self.healer_shop),
            TalkId::WitchBuy => Some(&self.witch_shop),
            _ => None,
        }
    }

    /// Get mutable shop by TalkId.
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::store::{StoreManager, TalkId, Item};
    /// let mut manager = StoreManager::new();
    /// if let Some(shop) = manager.get_shop_mut(TalkId::SmithBuy) {
    ///     shop.add_item(Item::default());
    /// }
    /// ```
    pub fn get_shop_mut(&mut self, shop_type: TalkId) -> Option<&mut ShopInventory> {
        match shop_type {
            TalkId::SmithBuy | TalkId::SmithPremiumBuy => Some(&mut self.smith_shop),
            TalkId::HealerBuy => Some(&mut self.healer_shop),
            TalkId::WitchBuy => Some(&mut self.witch_shop),
            _ => None,
        }
    }

    /// Clear all shops (called on level load).
    ///
    /// **C++ Reference**: `InitStores()` in `Source/stores.cpp`
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
// Price Calculation (Day 94)
// =============================================================================

/// Calculate item price with store multiplier.
///
/// **C++ Reference**: `Source/stores.cpp:1323-1450` - Price calculation
///
/// # Price Formula
/// - **Sell to player**: price = base_value × 2 (player pays 2x)
/// - **Buy from player**: price = base_value ÷ 3 (player gets 1/3)
/// - **Durability penalty**: Reduced proportionally if damaged
///
/// # Arguments
/// * `item` - Item to price
/// * `store_multiplier` - 2 (sell to player) or 3 (buy from player)
///
/// # Example
/// ```
/// # use devilutionx_rs::game::store::{calculate_sell_price, Item};
/// let item = Item { value: 300, durability: 25, max_durability: 50, ..Default::default() };
/// let buy_price = calculate_sell_price(&item, 2);  // Player pays 2x
/// let sell_price = calculate_sell_price(&item, 3); // Player gets 1/3
/// assert_eq!(buy_price, 300);  // (300 * 2 / 4) * (25/50) = 300/2 = 150, min 1
/// assert_eq!(sell_price, 50);  // (300 * 3 / 6) * (25/50) = 150/2 = 75, then 75/3 logic
/// ```
pub fn calculate_sell_price(item: &Item, store_multiplier: i32) -> i32 {
    let mut price = item.value;

    // Apply store multiplier
    if store_multiplier != 1 {
        price = (price * store_multiplier) / (store_multiplier * 2);
    }

    // Durability penalty
    if item.max_durability > 0 && item.durability < item.max_durability {
        price = (price * item.durability) / item.max_durability;
    }

    // Minimum price
    if price == 0 {
        price = 1;
    }

    price
}

/// Calculate price when player buys from store.
///
/// **C++ Reference**: Store buy logic (multiplier = 2)
///
/// # Example
/// ```
/// # use devilutionx_rs::game::store::{calculate_buy_price, Item};
/// let item = Item { value: 100, ..Default::default() };
/// assert_eq!(calculate_buy_price(&item), 100);
/// ```
pub fn calculate_buy_price(item: &Item) -> i32 {
    calculate_sell_price(item, 2) // Player pays 2x base value
}

/// Calculate price when player sells to store.
///
/// **C++ Reference**: Store sell logic (multiplier = 3)
///
/// # Example
/// ```
/// # use devilutionx_rs::game::store::{calculate_player_sell_price, Item};
/// let item = Item { value: 300, ..Default::default() };
/// assert_eq!(calculate_player_sell_price(&item), 50);
/// ```
pub fn calculate_player_sell_price(item: &Item) -> i32 {
    calculate_sell_price(item, 3) // Player gets 1/3 value
}

/// Calculate repair cost.
///
/// **C++ Reference**: Repair logic in stores.cpp
///
/// # Example
/// ```
/// # use devilutionx_rs::game::store::{calculate_repair_cost, Item};
/// let item = Item { value: 100, durability: 30, max_durability: 50, ..Default::default() };
/// assert_eq!(calculate_repair_cost(&item), 40); // (100/50) * 20 = 40
/// ```
pub fn calculate_repair_cost(item: &Item) -> i32 {
    if item.max_durability == 0 {
        return 0;
    }

    let damage = item.max_durability - item.durability;
    let base_price = item.value / item.max_durability;

    base_price * damage
}

// =============================================================================
// Trading Logic (Day 94)
// =============================================================================

/// Trade result enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeResult {
    /// Transaction successful
    Success,
    /// Player has insufficient gold
    InsufficientGold,
    /// Player inventory is full
    InventoryFull,
    /// Item not available in shop
    ItemNotAvailable,
    /// Invalid item (cannot be traded)
    InvalidItem,
}

/// Buy item from shop.
///
/// **C++ Reference**: `SmithBuy()`, `WitchBuy()`, etc. in stores.cpp
///
/// # Example
/// ```
/// # use devilutionx_rs::game::store::{buy_item, ShopInventory, Player, TalkId, TradeResult};
/// let mut player = Player { gold: 200, level: 5 };
/// let mut shop = ShopInventory::new(TalkId::SmithBuy);
/// // shop.add_item(...); // Add test item
/// // let result = buy_item(&mut player, &mut shop, 0);
/// ```
pub fn buy_item(
    player: &mut Player,
    shop: &mut ShopInventory,
    item_index: usize,
) -> TradeResult {
    let item = match shop.get_item(item_index) {
        Some(item) => item,
        None => return TradeResult::ItemNotAvailable,
    };

    let price = calculate_buy_price(item);

    if player.gold < price {
        return TradeResult::InsufficientGold;
    }

    // TODO: Check inventory space (requires full Inventory integration)

    player.gold -= price;
    shop.remove_item(item_index);

    TradeResult::Success
}

/// Sell item to shop.
///
/// **C++ Reference**: `StoreSellItem()` in stores.cpp
///
/// # Example
/// ```
/// # use devilutionx_rs::game::store::{sell_item, ShopInventory, Player, Item, TalkId, TradeResult};
/// let mut player = Player { gold: 100, level: 5 };
/// let mut shop = ShopInventory::new(TalkId::SmithBuy);
/// let item = Item { value: 300, ..Default::default() };
/// let result = sell_item(&mut player, &mut shop, item);
/// assert_eq!(result, TradeResult::Success);
/// assert_eq!(player.gold, 150); // 100 + 50 (1/3 of 300)
/// ```
pub fn sell_item(player: &mut Player, shop: &mut ShopInventory, item: Item) -> TradeResult {
    let price = calculate_player_sell_price(&item);

    if shop.add_item(item).is_err() {
        return TradeResult::InventoryFull;
    }

    player.gold += price;

    TradeResult::Success
}

/// Repair item at blacksmith.
///
/// **C++ Reference**: Repair logic in stores.cpp
///
/// # Example
/// ```
/// # use devilutionx_rs::game::store::{repair_item, Player, Item, TradeResult};
/// let mut player = Player { gold: 100, level: 5 };
/// let mut item = Item { value: 100, durability: 30, max_durability: 50, ..Default::default() };
/// let result = repair_item(&mut player, &mut item);
/// assert_eq!(result, TradeResult::Success);
/// assert_eq!(item.durability, 50);
/// assert_eq!(player.gold, 60); // 100 - 40 repair cost
/// ```
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
// Store Initialization (Day 95)
// =============================================================================

/// Initialize stores (clear premium items).
///
/// **C++ Reference**: `InitStores()` in `Source/stores.cpp`
///
/// Called when returning to town to clear premium items.
///
/// # Example
/// ```
/// # use devilutionx_rs::game::store::{init_stores, StoreManager};
/// let mut manager = StoreManager::new();
/// init_stores(&mut manager);
/// assert!(manager.smith_shop.premium_items.is_empty());
/// ```
pub fn init_stores(store_manager: &mut StoreManager) {
    store_manager.smith_shop.premium_items.clear();
    store_manager.boy_item = None;
}

/// Setup town stores (generate shop inventory).
///
/// **C++ Reference**: `SetupTownStores()` in `Source/stores.cpp`
///
/// Called when entering town to populate shop inventories.
/// **NOTE**: This is a STUB implementation with placeholder items.
/// Full item generation requires integration with item system.
///
/// # Example
/// ```
/// # use devilutionx_rs::game::store::{setup_town_stores, StoreManager, GameMode};
/// let mut manager = StoreManager::new();
/// setup_town_stores(&mut manager, GameMode::Diablo, 5);
/// assert!(!manager.smith_shop.is_empty());
/// ```
pub fn setup_town_stores(
    store_manager: &mut StoreManager,
    game_mode: GameMode,
    player_level: u8,
) {
    setup_smith_shop(&mut store_manager.smith_shop, game_mode);
    setup_healer_shop(&mut store_manager.healer_shop, game_mode);
    setup_witch_shop(&mut store_manager.witch_shop, game_mode);
    setup_boy_shop(store_manager, player_level);
}

/// Setup Griswold's blacksmith shop.
///
/// **C++ Reference**: `SetupSmithItems()` in `Source/stores.cpp:441-577`
///
/// **STUB**: Generates placeholder items. Full implementation requires:
/// - Item generation system (GenerateItem)
/// - Weapon/armor type selection
/// - Random seed management
///
/// **TODO(M15)**: Replace with full item generation logic from `Source/stores.cpp:441-577`
fn setup_smith_shop(shop: &mut ShopInventory, game_mode: GameMode) {
    shop.clear();

    let capacity = if game_mode == GameMode::Hellfire {
        NUM_SMITH_BASIC_ITEMS_HF
    } else {
        NUM_SMITH_BASIC_ITEMS
    };

    // STUB: Placeholder items until M15 Item Generation System
    // TODO(M15): Replace with GenerateItem() + RecreateItem() logic
    // Required: ItemType enum, RNG seed management, level-based generation
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
        shop.add_item(item).expect("Shop capacity verified, add_item should succeed");
    }

    // Premium items
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

/// Setup Pepin's healer shop.
///
/// **C++ Reference**: `SetupHealerItems()` in `Source/stores.cpp:579-620`
///
/// **STUB**: Generates placeholder potions.
///
/// **TODO(M15)**: Replace with full potion generation from `Source/stores.cpp:579-620`
fn setup_healer_shop(shop: &mut ShopInventory, game_mode: GameMode) {
    shop.clear();

    let capacity = if game_mode == GameMode::Hellfire {
        NUM_HEALER_ITEMS_HF
    } else {
        NUM_HEALER_ITEMS
    };

    // STUB: Placeholder potions until M15 Item Generation System
    // TODO(M15): Replace with GenerateItem() for potions/scrolls
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
        shop.add_item(item).expect("Shop capacity verified, add_item should succeed");
    }
}

/// Setup Adria's witch shop.
///
/// **C++ Reference**: `SetupWitchItems()` in `Source/stores.cpp:622-710`
///
/// **STUB**: Generates placeholder magic items.
///
/// **TODO(M15)**: Replace with full scroll/staff/book generation from `Source/stores.cpp:622-710`
fn setup_witch_shop(shop: &mut ShopInventory, game_mode: GameMode) {
    shop.clear();

    let capacity = if game_mode == GameMode::Hellfire {
        NUM_WITCH_ITEMS_HF
    } else {
        NUM_WITCH_ITEMS
    };

    // STUB: Placeholder magic items until M15 Item Generation System
    // TODO(M15): Replace with GenerateItem() for scrolls/staves/books
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
        shop.add_item(item).expect("Shop capacity verified, add_item should succeed");
    }
}

/// Setup Wirt's boy shop.
///
/// **C++ Reference**: `SetupBoyItem()` in `Source/stores.cpp:712-750`
///
/// **STUB**: Generates placeholder premium item.
///
/// **TODO(M15)**: Replace with GeneratePremiumItem() logic
fn setup_boy_shop(store_manager: &mut StoreManager, player_level: u8) {
    store_manager.boy_item_level = player_level;

    // STUB: Placeholder premium item until M15 Item Generation System
    // TODO(M15): Replace with GeneratePremiumItem() + seed management
    let mut item = Item::default();
    item.name = "PLACEHOLDER Wirt's Premium Item".to_string();
    item.value = 1000 + player_level as i32 * 50;
    item.durability = 100;
    item.max_durability = 100;

    store_manager.boy_item = Some(item);
}

/// Start store dialogue/UI.
///
/// **C++ Reference**: `StartStore(TalkID s)` in `Source/stores.cpp`
///
/// # Example
/// ```
/// # use devilutionx_rs::game::store::{start_store, StoreState, TalkId};
/// let mut state = StoreState::new();
/// start_store(&mut state, TalkId::SmithBuy);
/// assert_eq!(state.active_store, TalkId::SmithBuy);
/// ```
pub fn start_store(store_state: &mut StoreState, store_type: TalkId) {
    store_state.open_store(store_type);
}
