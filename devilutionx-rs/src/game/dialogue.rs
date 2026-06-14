//! Dialogue System
//!
//! # M14: Dialogue System (3 days, ~800 lines, ~20 tests)
//! # Day 96-97: Dialogue Foundation + Manager (600 lines, 18 tests)
//!
//! Rust port of NPC dialogue logic from `Source/towners.cpp`.
//!
//! ## C++ References
//! - `Source/towners.cpp:682-950`: NPC dialogue functions
//!   - `TalkToBlackSmith()` - Griswold dialogue
//!   - `TalkToWitch()` - Adria dialogue
//!   - `TalkToHealer()` - Pepin dialogue
//!   - `TalkToBoy()` - Wirt dialogue
//!   - `TalkToStoryteller()` - Cain dialogue
//!   - `TalkToTavern()` - Ogden dialogue
//!   - `TalkToDrunk()` - Farnham dialogue
//!   - `TalkToBarmaid()` - Gillian dialogue
//!
//! ## Scope (Day 96-97)
//! - DialogueOption (single dialogue choice)
//! - DialogueNode (dialogue tree node)
//! - DialogueManager (dialogue state management)
//! - Condition helper functions
//! - Complete dialogue trees (8 NPCs)
//!
//! ## Dependencies
//! - game::store::TalkId (M13) ✅
//! - game::towner::TownerType (M12) ✅
//! - game::player_exact::Player (M3) ✅

use super::store::TalkId;
use super::towner::TownerType;
use super::quests::{QuestId, QuestManager};
use std::collections::HashMap;

// =============================================================================
// DialoguePlayer Trait - Player Interface for Dialogue System (Day 100)
// =============================================================================

/// Player interface for the Dialogue system.
///
/// This trait defines the minimal player capabilities required by the dialogue
/// system, allowing it to query player state without depending on concrete
/// implementations.
///
/// **C++ Reference**: Implicit player queries in `TalkTo*()` functions
/// (Source/towners.cpp:682-950)
///
/// # Implementors
/// - `StubPlayer` - Testing/stub implementation (always returns false/0)
/// - `super::player::Player` - Real player implementation (via adapter)
///
/// # Example
/// ```ignore
/// fn check_dialogue_condition<P: DialoguePlayer>(player: &P) -> bool {
///     player.gold() >= 100 && player.has_visited_level(5)
/// }
/// ```
pub trait DialoguePlayer {
    /// Get player's current gold amount
    ///
    /// **C++ Reference**: `player._pGold` in Source/player.h
    fn gold(&self) -> i32;

    /// Get player's current level
    ///
    /// **C++ Reference**: `player._pLevel` in Source/player.h
    fn level(&self) -> u8;

    /// Check if player has a specific quest item by ID
    ///
    /// **C++ Reference**: `RemoveInventoryItemById(player, item_id)` in Source/inv.cpp
    ///
    /// # Arguments
    /// * `item_id` - Item ID to check (e.g., IDI_ROCK = 9)
    ///
    /// # Item ID Reference
    /// - IDI_ROCK = 9 (Rock of Enchantment)
    /// - IDI_ANVIL = 16 (Anvil of Fury)
    /// - IDI_MUSHROOM = 17 (Black Mushroom)
    /// - IDI_BRAIN = 18 (Brain)
    /// - IDI_FUNGALTM = 19 (Fungal Tome)
    /// - IDI_LAZSTAFF = 33 (Staff of Lazarus)
    fn has_item(&self, item_id: u32) -> bool;

    /// Check if player has visited a dungeon level
    ///
    /// **C++ Reference**: `player._pLvlVisited[level]` in Source/player.h
    ///
    /// # Arguments
    /// * `level` - Dungeon level (0-24)
    fn has_visited_level(&self, level: usize) -> bool;

    /// Get the number of items in inventory (for "has items to sell" checks)
    fn inventory_count(&self) -> usize;
}

// =============================================================================
// StubPlayer - Testing Implementation
// =============================================================================

/// Stub player implementation for dialogue testing.
///
/// This simple struct implements `DialoguePlayer` with configurable values,
/// allowing tests to control the player state without complex setup.
///
/// # Example
/// ```
/// # use devilutionx_rs::game::dialogue::{StubPlayer, DialoguePlayer};
/// let player = StubPlayer::new()
///     .with_gold(500)
///     .with_level(10)
///     .with_visited_levels(&[1, 2, 3]);
///
/// assert_eq!(player.gold(), 500);
/// assert!(player.has_visited_level(2));
/// assert!(!player.has_visited_level(10));
/// ```
#[derive(Debug, Clone, Default)]
pub struct StubPlayer {
    pub gold: i32,
    pub level: u8,
    pub inventory_items: usize,
    pub quest_items: Vec<u32>,
    pub visited_levels: Vec<usize>,
}

impl StubPlayer {
    /// Create a new stub player with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set gold amount (builder pattern)
    pub fn with_gold(mut self, gold: i32) -> Self {
        self.gold = gold;
        self
    }

    /// Set player level (builder pattern)
    pub fn with_level(mut self, level: u8) -> Self {
        self.level = level;
        self
    }

    /// Set inventory item count (builder pattern)
    pub fn with_inventory_count(mut self, count: usize) -> Self {
        self.inventory_items = count;
        self
    }

    /// Add quest items (builder pattern)
    pub fn with_quest_items(mut self, items: &[u32]) -> Self {
        self.quest_items = items.to_vec();
        self
    }

    /// Set visited levels (builder pattern)
    pub fn with_visited_levels(mut self, levels: &[usize]) -> Self {
        self.visited_levels = levels.to_vec();
        self
    }
}

impl DialoguePlayer for StubPlayer {
    fn gold(&self) -> i32 {
        self.gold
    }

    fn level(&self) -> u8 {
        self.level
    }

    fn has_item(&self, item_id: u32) -> bool {
        self.quest_items.contains(&item_id)
    }

    fn has_visited_level(&self, level: usize) -> bool {
        self.visited_levels.contains(&level)
    }

    fn inventory_count(&self) -> usize {
        self.inventory_items
    }
}

// =============================================================================
// Legacy Alias (for backward compatibility)
// =============================================================================

/// Legacy type alias for backward compatibility
///
/// **Deprecated**: Use `StubPlayer` directly or implement `DialoguePlayer` trait
pub type Player = StubPlayer;

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if player has specific quest item
///
/// **C++ Reference**: `RemoveInventoryItemById(player, item_id)` in Source/inv.cpp
///
/// # Arguments
/// * `player` - Any type implementing DialoguePlayer
/// * `item_id` - Item ID to check (e.g., IDI_ROCK = 9)
///
/// # Example
/// ```
/// # use devilutionx_rs::game::dialogue::{has_quest_item, StubPlayer};
/// let player = StubPlayer::new().with_quest_items(&[9, 17]); // Rock, Mushroom
/// assert!(has_quest_item(&player, 9));
/// assert!(!has_quest_item(&player, 16));
/// ```
pub fn has_quest_item<P: DialoguePlayer>(player: &P, item_id: u32) -> bool {
    player.has_item(item_id)
}

/// Check if player has visited a dungeon level
///
/// **C++ Reference**: `player._pLvlVisited[level]` in Source/player.h
///
/// # Arguments
/// * `player` - Any type implementing DialoguePlayer
/// * `level` - Dungeon level (0-24)
///
/// # Example
/// ```
/// # use devilutionx_rs::game::dialogue::{has_visited_level, StubPlayer};
/// let player = StubPlayer::new().with_visited_levels(&[1, 5, 9]);
/// assert!(has_visited_level(&player, 5));
/// assert!(!has_visited_level(&player, 10));
/// ```
pub fn has_visited_level<P: DialoguePlayer>(player: &P, level: u8) -> bool {
    player.has_visited_level(level as usize)
}

// =============================================================================
// DialogueOption - Single Dialogue Choice
// =============================================================================

/// Single dialogue option in a conversation tree.
///
/// **C++ Reference**: Implicit in `TalkToBlackSmith()` etc. (Source/towners.cpp:682-950)
///
/// Each option represents a choice the player can make in a dialogue,
/// such as "Let me see what you have to sell" or "I need healing".
///
/// # Fields
/// - `text`: Display text shown to the player
/// - `next_talk_id`: Next dialogue/shop state to transition to
/// - `condition`: Function to check if option is available
///
/// # Example
/// ```
/// # use devilutionx_rs::game::dialogue::{DialogueOption, Player};
/// # use devilutionx_rs::game::store::TalkId;
/// let option = DialogueOption::new(
///     "Let me see what you have to sell",
///     TalkId::SmithBuy,
///     |_player| true, // Always available
/// );
///
/// let player = Player::default();
/// assert!(option.is_available(&player));
/// assert_eq!(option.text, "Let me see what you have to sell");
/// assert_eq!(option.next_talk_id, TalkId::SmithBuy);
/// ```
#[derive(Debug, Clone)]
pub struct DialogueOption {
    /// Display text shown to the player.
    pub text: &'static str,

    /// Next TalkId to transition to when this option is selected.
    pub next_talk_id: TalkId,

    /// Condition function to check if this option is available.
    ///
    /// Takes a reference to the player and returns true if the option
    /// should be shown (e.g., "has items to sell" returns true only if
    /// the player has sellable items).
    pub condition: fn(&Player) -> bool,
}

impl DialogueOption {
    /// Create a new dialogue option.
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::dialogue::DialogueOption;
    /// # use devilutionx_rs::game::store::TalkId;
    /// let option = DialogueOption::new(
    ///     "Leave",
    ///     TalkId::None,
    ///     |_| true,
    /// );
    /// ```
    pub const fn new(
        text: &'static str,
        next_talk_id: TalkId,
        condition: fn(&Player) -> bool,
    ) -> Self {
        Self {
            text,
            next_talk_id,
            condition,
        }
    }

    /// Check if this dialogue option is available for the given player.
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::dialogue::{DialogueOption, Player};
    /// # use devilutionx_rs::game::store::TalkId;
    /// let option = DialogueOption::new(
    ///     "Test",
    ///     TalkId::None,
    ///     |p| p.gold >= 100,
    /// );
    ///
    /// let mut player = Player::default();
    /// player.gold = 50;
    /// assert!(!option.is_available(&player));
    ///
    /// player.gold = 150;
    /// assert!(option.is_available(&player));
    /// ```
    pub fn is_available(&self, player: &Player) -> bool {
        (self.condition)(player)
    }
}

// =============================================================================
// DialogueNode - Dialogue Tree Node
// =============================================================================

/// Dialogue tree node for NPC conversations.
///
/// **C++ Reference**: `TalkToBlackSmith()` in `Source/towners.cpp:682-737`
///
/// Each node represents a specific dialogue state (e.g., Griswold's greeting)
/// and contains multiple dialogue options the player can choose from.
///
/// # Fields
/// - `talk_id`: Current dialogue/shop state (e.g., TalkId::Smith)
/// - `greeting`: Optional greeting text shown when entering this state
/// - `options`: Array of available dialogue options
///
/// # Example
/// ```
/// # use devilutionx_rs::game::dialogue::{DialogueNode, DialogueOption, Player};
/// # use devilutionx_rs::game::store::TalkId;
/// let node = DialogueNode {
///     talk_id: TalkId::Smith,
///     greeting: Some("What can I do for ye?"),
///     options: &[
///         DialogueOption::new("Buy", TalkId::SmithBuy, |_| true),
///         DialogueOption::new("Leave", TalkId::None, |_| true),
///     ],
/// };
///
/// let player = Player::default();
/// let available = node.get_available_options(&player);
/// assert_eq!(available.len(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct DialogueNode {
    /// Current TalkId (e.g., TalkId::Smith for Griswold's greeting).
    pub talk_id: TalkId,

    /// Optional greeting text displayed when entering this dialogue state.
    pub greeting: Option<&'static str>,

    /// Array of available dialogue options.
    pub options: &'static [DialogueOption],
}

impl DialogueNode {
    /// Get all available dialogue options for the given player.
    ///
    /// Filters options based on their condition functions.
    ///
    /// # Example
    /// ```
    /// # use devilutionx_rs::game::dialogue::{DialogueNode, DialogueOption, Player};
    /// # use devilutionx_rs::game::store::TalkId;
    /// let node = DialogueNode {
    ///     talk_id: TalkId::Smith,
    ///     greeting: Some("Greetings"),
    ///     options: &[
    ///         DialogueOption::new("Option 1", TalkId::None, |_| true),
    ///         DialogueOption::new("Option 2", TalkId::None, |p| p.gold >= 100),
    ///     ],
    /// };
    ///
    /// let mut player = Player::default();
    /// player.gold = 50;
    /// assert_eq!(node.get_available_options(&player).len(), 1);
    ///
    /// player.gold = 150;
    /// assert_eq!(node.get_available_options(&player).len(), 2);
    /// ```
    pub fn get_available_options(&self, player: &Player) -> Vec<&DialogueOption> {
        self.options
            .iter()
            .filter(|opt| opt.is_available(player))
            .collect()
    }
}

// =============================================================================
// Condition Helper Functions
// =============================================================================

/// Check if player has items that can be sold to NPCs.
///
/// **C++ Reference**: Implicit in `TalkToBlackSmith()` "I have something for you" option
///
/// **STUB**: Always returns true for M14. Full implementation requires M10 Inventory.
///
/// **TODO(M15)**: Replace with actual inventory check:
/// - Check player inventory for sellable items
/// - Exclude quest items, equipped items
/// - Verify item value > 0
///
/// # Example
/// ```
/// # use devilutionx_rs::game::dialogue::{has_items_to_sell, Player};
/// let player = Player::default();
/// // Stub: always true
/// assert!(has_items_to_sell(&player));
/// ```
pub fn has_items_to_sell(_player: &Player) -> bool {
    // STUB: Always true until M10 Inventory provides full item inspection
    true
}

/// Check if player has items that need repair.
///
/// **C++ Reference**: Implicit in `TalkToBlackSmith()` "I have something that needs repair" option
///
/// **STUB**: Always returns false for M14. Full implementation requires M10 Inventory.
///
/// **TODO(M15)**: Replace with actual durability check:
/// - Check player inventory for damaged items (durability < max_durability)
/// - Exclude items with max_durability == 0 (indestructible)
/// - Calculate repair cost
///
/// # Example
/// ```
/// # use devilutionx_rs::game::dialogue::{has_items_to_repair, Player};
/// let player = Player::default();
/// // Stub: always false (no damaged items)
/// assert!(!has_items_to_repair(&player));
/// ```
pub fn has_items_to_repair(_player: &Player) -> bool {
    // STUB: Always false until M10 Inventory provides durability inspection
    false
}

/// Check if player has a staff that can be recharged.
///
/// **C++ Reference**: Implicit in `TalkToWitch()` "I need this staff recharged" option
///
/// **STUB**: Always returns false for M14. Full implementation requires M10 Inventory.
///
/// **TODO(M15)**: Replace with actual staff charge check:
/// - Check player inventory for staff items
/// - Verify staff has charges system (max_charges > 0)
/// - Check if current charges < max charges
/// - Calculate recharge cost
///
/// # Example
/// ```
/// # use devilutionx_rs::game::dialogue::{has_staff_to_recharge, Player};
/// let player = Player::default();
/// // Stub: always false (no staves)
/// assert!(!has_staff_to_recharge(&player));
/// ```
pub fn has_staff_to_recharge(_player: &Player) -> bool {
    // STUB: Always false until M10 Inventory provides staff charge inspection
    false
}

/// Always returns true (for unconditional options like "Leave").
///
/// # Example
/// ```
/// # use devilutionx_rs::game::dialogue::{always_available, Player};
/// let player = Player::default();
/// assert!(always_available(&player));
/// ```
pub fn always_available(_player: &Player) -> bool {
    true
}

// =============================================================================
// Dialogue Tree Definitions
// =============================================================================

/// Griswold (Blacksmith) dialogue tree.
///
/// **C++ Reference**: `TalkToBlackSmith()` in `Source/towners.cpp:682-737`
///
/// # Dialogue Options
/// 1. "I have something for you to buy" → TalkId::SmithSell
/// 2. "I have something that needs repair" → TalkId::SmithRepair
/// 3. "Let me see what you have to sell" → TalkId::SmithBuy
/// 4. "Premium goods" → TalkId::SmithPremiumBuy
/// 5. "Leave" → TalkId::None
///
/// # C++ Code
/// ```cpp
/// void TalkToBlackSmith() {
///     // Display dialogue options based on player state
///     // Option: "I have something for you..." (sell)
///     // Option: "I have something that needs repair"
///     // Option: "Let me see what you have to sell" (buy)
///     // Option: "Premium goods"
///     // Option: "Leave"
/// }
/// ```
pub const SMITH_DIALOGUE: DialogueNode = DialogueNode {
    talk_id: TalkId::Smith,
    greeting: Some("What can I do for ye?"),
    options: &[
        DialogueOption::new(
            "I have something for you to buy",
            TalkId::SmithSell,
            has_items_to_sell,
        ),
        DialogueOption::new(
            "I have something that needs repair",
            TalkId::SmithRepair,
            has_items_to_repair,
        ),
        DialogueOption::new(
            "Let me see what you have to sell",
            TalkId::SmithBuy,
            always_available,
        ),
        DialogueOption::new(
            "Premium goods",
            TalkId::SmithPremiumBuy,
            always_available,
        ),
        DialogueOption::new("Leave", TalkId::None, always_available),
    ],
};

/// Adria (Witch) dialogue tree.
///
/// **C++ Reference**: `TalkToWitch()` in `Source/towners.cpp:739-783`
///
/// # Dialogue Options
/// 1. "I have something for you to buy" → TalkId::WitchSell
/// 2. "I need this staff recharged" → TalkId::WitchRecharge
/// 3. "Let me see your items" → TalkId::WitchBuy
/// 4. "Leave" → TalkId::None
///
/// # C++ Code
/// ```cpp
/// void TalkToWitch() {
///     // Option: "I have something for you..." (sell)
///     // Option: "I need this staff recharged" (recharge)
///     // Option: "Let me see your items" (buy)
///     // Option: "Leave"
/// }
/// ```
pub const WITCH_DIALOGUE: DialogueNode = DialogueNode {
    talk_id: TalkId::Witch,
    greeting: Some("Welcome to my shop"),
    options: &[
        DialogueOption::new(
            "I have something for you to buy",
            TalkId::WitchSell,
            has_items_to_sell,
        ),
        DialogueOption::new(
            "I need this staff recharged",
            TalkId::WitchRecharge,
            has_staff_to_recharge,
        ),
        DialogueOption::new(
            "Let me see your items",
            TalkId::WitchBuy,
            always_available,
        ),
        DialogueOption::new("Leave", TalkId::None, always_available),
    ],
};

/// Pepin (Healer) dialogue tree.
///
/// **C++ Reference**: `TalkToHealer()` in `Source/towners.cpp:785-822`
///
/// # Dialogue Options
/// 1. "I need healing" → TalkId::Healer (triggers heal action)
/// 2. "Let me see your potions" → TalkId::HealerBuy
/// 3. "Leave" → TalkId::None
///
/// # C++ Code
/// ```cpp
/// void TalkToHealer() {
///     // Option: "I need healing" (heal player)
///     // Option: "Let me see your potions" (buy)
///     // Option: "Leave"
/// }
/// ```
pub const HEALER_DIALOGUE: DialogueNode = DialogueNode {
    talk_id: TalkId::Healer,
    greeting: Some("How can I help you?"),
    options: &[
        DialogueOption::new(
            "I need healing",
            TalkId::Healer, // Same state (triggers heal action)
            always_available,
        ),
        DialogueOption::new(
            "Let me see your potions",
            TalkId::HealerBuy,
            always_available,
        ),
        DialogueOption::new("Leave", TalkId::None, always_available),
    ],
};

// =============================================================================
// Additional Dialogue Trees (Day 97)
// =============================================================================

/// Wirt (Peg-legged Boy) Dialogue Tree
///
/// **C++ Reference**: `TalkToBoy()` in `Source/towners.cpp:824-863`
///
/// # Dialogue Options
/// 1. "I have something to sell" → TalkId::BoySell
/// 2. "What have you got?" → TalkId::BoyBuy (costs 50 gold)
/// 3. "Leave" → TalkId::None
///
/// # C++ Code
/// ```cpp
/// void TalkToBoy() {
///     // Option: "I have something to sell"
///     // Option: "What have you got?" (requires 50 gold to browse)
///     // Option: "Leave"
/// }
/// ```
pub const BOY_DIALOGUE: DialogueNode = DialogueNode {
    talk_id: TalkId::Boy,
    greeting: Some("Whatcha want?"),
    options: &[
        DialogueOption::new(
            "What have you got?",
            TalkId::BoyBuy,
            always_available, // Note: Actual browsing costs 50 gold (handled by shop logic)
        ),
        DialogueOption::new("Leave", TalkId::None, always_available),
    ],
};

/// Deckard Cain (Storyteller) Dialogue Tree
///
/// **C++ Reference**: `TalkToStoryteller()` in `Source/towners.cpp:865-898`
///
/// # Dialogue Options
/// 1. "Identify an item" → TalkId::StorytellerIdentify
/// 2. "Talk" → TalkId::Storyteller (quest dialogue)
/// 3. "Leave" → TalkId::None
///
/// # C++ Code
/// ```cpp
/// void TalkToStoryteller() {
///     // Option: "Identify an item" (identification service)
///     // Option: "Talk" (quest dialogue)
///     // Option: "Leave"
/// }
/// ```
pub const STORYTELLER_DIALOGUE: DialogueNode = DialogueNode {
    talk_id: TalkId::Storyteller,
    greeting: Some("Hello, my friend. Stay awhile and listen..."),
    options: &[
        DialogueOption::new(
            "Identify an item",
            TalkId::StorytellerIdentify,
            always_available,
        ),
        DialogueOption::new(
            "Talk",
            TalkId::Storyteller, // Same state (quest dialogue)
            always_available,
        ),
        DialogueOption::new("Leave", TalkId::None, always_available),
    ],
};

/// Ogden (Tavern Owner) Dialogue Tree
///
/// **C++ Reference**: `TalkToTavern()` in `Source/towners.cpp:900-925`
///
/// # Dialogue Options
/// 1. "Talk" → TalkId::Tavern (quest/gossip)
/// 2. "Leave" → TalkId::None
///
/// # C++ Code
/// ```cpp
/// void TalkToTavern() {
///     // Option: "Talk" (quest dialogue/gossip)
///     // Option: "Leave"
/// }
/// ```
pub const TAVERN_DIALOGUE: DialogueNode = DialogueNode {
    talk_id: TalkId::Tavern,
    greeting: Some("Welcome to the Rising Sun!"),
    options: &[
        DialogueOption::new(
            "Talk",
            TalkId::Tavern, // Same state (gossip)
            always_available,
        ),
        DialogueOption::new("Leave", TalkId::None, always_available),
    ],
};

/// Farnham (Drunk) Dialogue Tree
///
/// **C++ Reference**: `TalkToDrunk()` in `Source/towners.cpp:927-950`
///
/// # Dialogue Options
/// 1. "Talk" → TalkId::Drunk (quest/gossip)
/// 2. "Leave" → TalkId::None
///
/// # C++ Code
/// ```cpp
/// void TalkToDrunk() {
///     // Option: "Talk" (quest dialogue/gossip)
///     // Option: "Leave"
/// }
/// ```
pub const DRUNK_DIALOGUE: DialogueNode = DialogueNode {
    talk_id: TalkId::Drunk,
    greeting: Some("Whaddya want? I got important drinkin' to do!"),
    options: &[
        DialogueOption::new(
            "Talk",
            TalkId::Drunk, // Same state (gossip)
            always_available,
        ),
        DialogueOption::new("Leave", TalkId::None, always_available),
    ],
};

/// Gillian (Barmaid) Dialogue Tree
///
/// **C++ Reference**: Implicit in towner interaction system
///
/// # Dialogue Options
/// 1. "Talk" → TalkId::Barmaid (gossip)
/// 2. "Leave" → TalkId::None
///
/// # Note
/// Gillian has minimal interaction in original game (mostly ambient dialogue)
pub const BARMAID_DIALOGUE: DialogueNode = DialogueNode {
    talk_id: TalkId::Barmaid,
    greeting: Some("Can I help you with something?"),
    options: &[
        DialogueOption::new(
            "Talk",
            TalkId::Barmaid, // Same state (gossip)
            always_available,
        ),
        DialogueOption::new("Leave", TalkId::None, always_available),
    ],
};

// =============================================================================
// Quest Dialogue Nodes (Day 98)
// =============================================================================

/// Griswold - Magic Rock Quest dialogue (start)
///
/// **C++ Reference**: `TalkToBlackSmith()` lines 293-305 (Source/towners.cpp)
///
/// # Trigger Condition
/// - Quest Q_ROCK active
/// - Player visited level 4 or 5 (Catacombs 1-2)
/// - Quest var2 == 0 (not yet acknowledged)
///
/// # C++ Code
/// ```cpp
/// if ((player._pLvlVisited[4] || player._pLvlVisited[5])
///     && Quests[Q_ROCK]._qactive != QUEST_DONE) {
///     if (Quests[Q_ROCK]._qvar2 == 0) {
///         Quests[Q_ROCK]._qvar2 = 1;
///         Quests[Q_ROCK]._qlog = true;
///         InitQTextMsg(TEXT_INFRA5);
///         return;
///     }
/// }
/// ```
#[allow(dead_code)]
pub const SMITH_QUEST_ROCK_START: DialogueNode = DialogueNode {
    talk_id: TalkId::Smith,
    greeting: Some("I sense a magical presence in the lower depths..."),
    options: &[
        DialogueOption::new(
            "Tell me more",
            TalkId::Smith, // Return to normal dialogue
            always_available,
        ),
    ],
};

/// Griswold - Magic Rock Quest dialogue (completion)
///
/// **C++ Reference**: `TalkToBlackSmith()` lines 306-313
///
/// # Trigger Condition
/// - Quest Q_ROCK var2 == 1 (acknowledged)
/// - Player has Magic Rock (IDI_ROCK = 9)
#[allow(dead_code)]
pub const SMITH_QUEST_ROCK_COMPLETE: DialogueNode = DialogueNode {
    talk_id: TalkId::Smith,
    greeting: Some("Ah, the Magic Rock! Let me forge you something special..."),
    options: &[
        DialogueOption::new(
            "Here it is",
            TalkId::Smith, // Quest completion (spawn Infravision ring)
            |p| has_quest_item(p, 9), // IDI_ROCK = 9
        ),
        DialogueOption::new("Leave", TalkId::None, always_available),
    ],
};

/// Griswold - Anvil of Fury Quest dialogue (start)
///
/// **C++ Reference**: `TalkToBlackSmith()` lines 318-330
///
/// # Trigger Condition
/// - Quest Q_ANVIL active
/// - Player visited level 9 or 10 (Hell 1-2)
/// - Quest var2 == 0
///
/// # C++ Code
/// ```cpp
/// if ((player._pLvlVisited[9] || player._pLvlVisited[10])
///     && Quests[Q_ANVIL]._qvar2 == 0) {
///     Quests[Q_ANVIL]._qvar2 = 1;
///     Quests[Q_ANVIL]._qlog = true;
///     InitQTextMsg(TEXT_ANVIL5);
///     return;
/// }
/// ```
#[allow(dead_code)]
pub const SMITH_QUEST_ANVIL_START: DialogueNode = DialogueNode {
    talk_id: TalkId::Smith,
    greeting: Some("The Anvil of Fury... I could forge Griswold's Edge with that!"),
    options: &[
        DialogueOption::new(
            "I'll find it",
            TalkId::Smith, // Quest activated
            always_available,
        ),
    ],
};

/// Griswold - Anvil of Fury Quest dialogue (completion)
///
/// **C++ Reference**: `TalkToBlackSmith()` lines 331-337
#[allow(dead_code)]
pub const SMITH_QUEST_ANVIL_COMPLETE: DialogueNode = DialogueNode {
    talk_id: TalkId::Smith,
    greeting: Some("You found the Anvil! Let me craft Griswold's Edge for you..."),
    options: &[
        DialogueOption::new(
            "Here it is",
            TalkId::Smith, // Quest completion (spawn Griswold's Edge)
            |p| has_quest_item(p, 16), // IDI_ANVIL = 16
        ),
        DialogueOption::new("Leave", TalkId::None, always_available),
    ],
};

/// Adria - Mushroom Quest dialogue (Tome given)
///
/// **C++ Reference**: `TalkToWitch()` lines 341-349 (Source/towners.cpp)
///
/// # Trigger Condition
/// - Quest Q_MUSHROOM INIT
/// - Player has Fungal Tome (IDI_FUNGALTM = 86)
///
/// # C++ Code
/// ```cpp
/// if (Quests[Q_MUSHROOM]._qactive == QUEST_INIT
///     && RemoveInventoryItemById(player, IDI_FUNGALTM)) {
///     Quests[Q_MUSHROOM]._qactive = QUEST_ACTIVE;
///     Quests[Q_MUSHROOM]._qvar1 = QS_TOMEGIVEN;
///     InitQTextMsg(TEXT_MUSH8);
///     return;
/// }
/// ```
#[allow(dead_code)]
pub const WITCH_QUEST_MUSHROOM_TOME: DialogueNode = DialogueNode {
    talk_id: TalkId::Witch,
    greeting: Some("Ah, the Fungal Tome! I can help you with this..."),
    options: &[
        DialogueOption::new(
            "Take the tome",
            TalkId::Witch, // Quest activated
            |p| has_quest_item(p, 19), // IDI_FUNGALTM = 19
        ),
        DialogueOption::new("Leave", TalkId::None, always_available),
    ],
};

/// Adria - Mushroom Quest dialogue (Mushroom delivery)
///
/// **C++ Reference**: `TalkToWitch()` lines 353-361
///
/// # Trigger Condition
/// - Quest Q_MUSHROOM active, var1 >= QS_TOMEGIVEN
/// - Player has Mushroom (IDI_MUSHROOM = 17)
#[allow(dead_code)]
pub const WITCH_QUEST_MUSHROOM_DELIVERY: DialogueNode = DialogueNode {
    talk_id: TalkId::Witch,
    greeting: Some("Have you found the mushroom I need?"),
    options: &[
        DialogueOption::new(
            "Here it is",
            TalkId::Witch, // Quest progress (brew elixir)
            |p| has_quest_item(p, 17), // IDI_MUSHROOM = 17
        ),
        DialogueOption::new("Not yet", TalkId::None, always_available),
    ],
};

/// Adria - Mushroom Quest dialogue (Brain check)
///
/// **C++ Reference**: `TalkToWitch()` lines 369-373
///
/// # Trigger Condition
/// - Quest Q_MUSHROOM var1 >= QS_MUSHGIVEN
/// - Player has Brain (IDI_BRAIN = 91)
#[allow(dead_code)]
pub const WITCH_QUEST_MUSHROOM_BRAIN: DialogueNode = DialogueNode {
    talk_id: TalkId::Witch,
    greeting: Some("Hmm, that brain will be useful for my work..."),
    options: &[
        DialogueOption::new(
            "Take the brain",
            TalkId::Witch, // Quest progress
            |p| has_quest_item(p, 18), // IDI_BRAIN = 18
        ),
        DialogueOption::new("Leave", TalkId::None, always_available),
    ],
};

/// Deckard Cain - Lazarus Staff Quest dialogue
///
/// **C++ Reference**: `TalkToStoryteller()` lines 459-468 (Source/towners.cpp)
///
/// # Trigger Condition
/// - Quest Q_BETRAYER INIT (single player)
/// - Player has Lazarus Staff (IDI_LAZSTAFF = 90)
///
/// # C++ Code
/// ```cpp
/// if (!UseMultiplayerQuests()) {
///     if (betrayerQuest._qactive == QUEST_INIT
///         && RemoveInventoryItemById(player, IDI_LAZSTAFF)) {
///         InitQTextMsg(TEXT_VILE1);
///         betrayerQuest._qactive = QUEST_ACTIVE;
///         betrayerQuest._qvar1 = 2;
///         return;
///     }
/// }
/// ```
#[allow(dead_code)]
pub const STORYTELLER_QUEST_BETRAYER_START: DialogueNode = DialogueNode {
    talk_id: TalkId::Storyteller,
    greeting: Some("Ah, the Staff of Lazarus! So the rumors are true..."),
    options: &[
        DialogueOption::new(
            "What do you know?",
            TalkId::Storyteller, // Quest activated
            |p| has_quest_item(p, 33), // IDI_LAZSTAFF = 33
        ),
        DialogueOption::new("Leave", TalkId::None, always_available),
    ],
};

/// Deckard Cain - Diablo Quest unlock dialogue
///
/// **C++ Reference**: `TalkToStoryteller()` lines 477-485
///
/// # Trigger Condition
/// - Quest Q_BETRAYER done, var1 == 7 (Lazarus killed)
#[allow(dead_code)]
pub const STORYTELLER_QUEST_DIABLO_UNLOCK: DialogueNode = DialogueNode {
    talk_id: TalkId::Storyteller,
    greeting: Some("The time has come to face Diablo himself!"),
    options: &[
        DialogueOption::new(
            "I'm ready",
            TalkId::Storyteller, // Diablo quest unlocked
            always_available,
        ),
    ],
};

// =============================================================================
// DialogueManager - Dialogue State Management (Day 97)
// =============================================================================

/// Manages all NPC dialogue trees and current dialogue state.
///
/// **C++ Reference**: Implicit in towner interaction system (Source/towners.cpp)
///
/// Provides centralized management of dialogue trees and tracks the current
/// active dialogue state.
///
/// # Usage
/// ```ignore
/// let mut manager = DialogueManager::new();
/// let player = Player::default();
///
/// // Start dialogue with Griswold
/// if let Some(node) = manager.start_dialogue(TalkId::Smith, &player) {
///     let options = node.get_available_options(&player);
///     // Display options to player...
///
///     // Player selects option 0
///     manager.select_option(0, &player).unwrap();
/// }
/// ```
#[derive(Debug)]
pub struct DialogueManager {
    /// Map: TalkId → DialogueNode
    dialogues: HashMap<TalkId, DialogueNode>,

    /// Current active dialogue (if any)
    current_dialogue: Option<TalkId>,
}

impl DialogueManager {
    /// Create a new DialogueManager with all dialogue trees registered.
    ///
    /// **C++ Reference**: Implicit initialization in towner system
    pub fn new() -> Self {
        let mut dialogues = HashMap::new();

        // Register all 8 NPC dialogue trees
        dialogues.insert(TalkId::Smith, SMITH_DIALOGUE.clone());
        dialogues.insert(TalkId::Witch, WITCH_DIALOGUE.clone());
        dialogues.insert(TalkId::Healer, HEALER_DIALOGUE.clone());
        dialogues.insert(TalkId::Boy, BOY_DIALOGUE.clone());
        dialogues.insert(TalkId::Storyteller, STORYTELLER_DIALOGUE.clone());
        dialogues.insert(TalkId::Tavern, TAVERN_DIALOGUE.clone());
        dialogues.insert(TalkId::Drunk, DRUNK_DIALOGUE.clone());
        dialogues.insert(TalkId::Barmaid, BARMAID_DIALOGUE.clone());

        Self {
            dialogues,
            current_dialogue: None,
        }
    }

    /// Get a dialogue node by TalkId.
    ///
    /// Returns `None` if the TalkId has no associated dialogue tree
    /// (e.g., TalkId::None, TalkId::SmithBuy, etc. are actions, not dialogue states).
    pub fn get_dialogue(&self, talk_id: TalkId) -> Option<&DialogueNode> {
        self.dialogues.get(&talk_id)
    }

    /// Start a new dialogue with the specified TalkId.
    ///
    /// Returns the initial DialogueNode if the TalkId is valid, otherwise `None`.
    ///
    /// # Arguments
    /// * `talk_id` - The dialogue to start (e.g., TalkId::Smith)
    /// * `player` - The player (for condition checks)
    ///
    /// **C++ Reference**: `TalkToBlackSmith()` etc. in Source/towners.cpp
    pub fn start_dialogue(&mut self, talk_id: TalkId, _player: &Player) -> Option<&DialogueNode> {
        if let Some(node) = self.dialogues.get(&talk_id) {
            self.current_dialogue = Some(talk_id);
            Some(node)
        } else {
            None
        }
    }

    /// Select a dialogue option by index (from available options).
    ///
    /// Transitions to the next dialogue state based on the selected option.
    ///
    /// # Arguments
    /// * `option_index` - Index into get_available_options() result
    /// * `player` - The player (for condition checks)
    ///
    /// # Returns
    /// * `Ok(TalkId)` - The next dialogue state
    /// * `Err(String)` - Error if no active dialogue or invalid option index
    ///
    /// **C++ Reference**: Implicit in dialogue option selection logic
    pub fn select_option(&mut self, option_index: usize, player: &Player) -> Result<TalkId, String> {
        let current = self.current_dialogue.ok_or("No active dialogue")?;

        let node = self
            .dialogues
            .get(&current)
            .ok_or("Current dialogue not found")?;

        let available_options = node.get_available_options(player);

        let selected = available_options
            .get(option_index)
            .ok_or(format!("Invalid option index: {}", option_index))?;

        let next_talk_id = selected.next_talk_id;
        self.current_dialogue = Some(next_talk_id);

        Ok(next_talk_id)
    }

    /// End the current dialogue.
    ///
    /// **C++ Reference**: Implicit when exiting dialogue UI
    pub fn end_dialogue(&mut self) {
        self.current_dialogue = None;
    }

    /// Get the current active dialogue TalkId.
    pub fn current_dialogue(&self) -> Option<TalkId> {
        self.current_dialogue
    }

    /// Get quest-specific dialogue for NPC (Day 98)
    ///
    /// **C++ Reference**: Quest check logic in `TalkToX()` functions
    ///
    /// Checks for active quests that should trigger special dialogue
    /// instead of the normal NPC dialogue tree.
    ///
    /// # Arguments
    /// * `npc` - NPC type (e.g., TownerType::Smith)
    /// * `_player` - Player state (for quest checks)
    ///
    /// # Returns
    /// * `Some(&DialogueNode)` - Quest dialogue if active quest found
    /// * `None` - No active quest dialogue (use normal dialogue)
    ///
    /// # Example
    /// ```ignore
    /// let manager = DialogueManager::new();
    /// let player = Player::default();
    ///
    /// // Check for quest dialogue first
    /// if let Some(quest_node) = manager.get_quest_dialogue(TownerType::Smith, &player) {
    ///     // Display quest dialogue
    ///     display_greeting(quest_node.greeting.unwrap_or(""));
    /// } else {
    ///     // Display normal dialogue
    ///     manager.start_dialogue(TalkId::Smith, &player);
    /// }
    /// ```
    pub fn get_quest_dialogue(
        &self,
        npc: TownerType,
        _player: &Player,
        quest_manager: &QuestManager,
    ) -> Option<&DialogueNode> {
        match npc {
            TownerType::Smith => {
                // Check Q_ROCK (Magic Rock quest)
                if quest_manager.is_active(QuestId::Rock) {
                    return Some(&SMITH_QUEST_ROCK_START);
                }
                // Check Q_ANVIL (Anvil of Fury quest)
                if quest_manager.is_active(QuestId::Anvil) {
                    return Some(&SMITH_QUEST_ANVIL_START);
                }
            }
            TownerType::Witch => {
                // Check Q_MUSHROOM (Mushroom quest)
                if quest_manager.is_active(QuestId::Mushroom) {
                    return Some(&WITCH_QUEST_MUSHROOM_TOME);
                }
            }
            TownerType::Story => {
                // Check Q_BETRAYER (Lazarus quest)
                if quest_manager.is_active(QuestId::Betrayer) {
                    return Some(&STORYTELLER_QUEST_BETRAYER_START);
                }
            }
            _ => {}
        }
        None
    }
}

impl Default for DialogueManager {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Helper: TownerType → TalkId Mapping (Day 97)
// =============================================================================

/// Convert TownerType to initial TalkId for dialogue.
///
/// **C++ Reference**: Implicit in NPC interaction logic
///
/// # Returns
/// * `Some(TalkId)` - For NPCs with dialogue trees
/// * `None` - For NPCs without dialogue (e.g., DeadGuy, Cow)
///
/// # Example
/// ```ignore
/// let talk_id = towner_to_talk_id(TownerType::Smith);
/// assert_eq!(talk_id, Some(TalkId::Smith));
/// ```
pub fn towner_to_talk_id(towner_type: TownerType) -> Option<TalkId> {
    match towner_type {
        TownerType::Smith => Some(TalkId::Smith),
        TownerType::Healer => Some(TalkId::Healer),
        TownerType::Witch => Some(TalkId::Witch),
        TownerType::PegBoy => Some(TalkId::Boy),
        TownerType::Story => Some(TalkId::Storyteller),
        TownerType::Tavern => Some(TalkId::Tavern),
        TownerType::Drunk => Some(TalkId::Drunk),
        TownerType::Barmaid => Some(TalkId::Barmaid),
        // Non-dialogue NPCs
        TownerType::DeadGuy | TownerType::Cow | TownerType::Farmer | TownerType::Girl | TownerType::CowFarmer => None,
    }
}

// =============================================================================
// Integration Stubs (Day 98) - To be replaced by M13/M17 systems
// =============================================================================

/// Display NPC greeting text (stub for Day 98)
///
/// **C++ Reference**: `TownerTalk(_speech_id)` in Source/towners.cpp
///
/// **TODO(M17)**: Replace with full UI implementation:
/// - Display text in dialogue box UI
/// - Play voice line (audio system)
/// - Show NPC portrait animation
///
/// # Current Implementation
/// Prints to console (development stub)
///
/// # Arguments
/// * `greeting` - NPC greeting text to display
///
/// # Example
/// ```
/// # use devilutionx_rs::game::dialogue::display_greeting;
/// display_greeting("What can I do for ye?");
/// ```
pub fn display_greeting(greeting: &str) {
    println!("[NPC]: {}", greeting);
}

/// Trigger shop UI (stub for Day 98)
///
/// **C++ Reference**: `StartStore(TalkID)` in Source/stores.cpp
///
/// **TODO(M13)**: Replace with actual M13 Shop integration:
/// - Call Shop::open(talk_id)
/// - Populate item list from shop inventory
/// - Enable buy/sell/repair UI actions
///
/// # Current Implementation
/// Prints intent (development stub)
///
/// # Arguments
/// * `talk_id` - Shop state to open (e.g., TalkId::SmithBuy)
///
/// # Example
/// ```
/// # use devilutionx_rs::game::dialogue::trigger_shop;
/// # use devilutionx_rs::game::store::TalkId;
/// trigger_shop(TalkId::SmithBuy);
/// ```
pub fn trigger_shop(talk_id: TalkId) {
    println!("[SHOP]: Opening shop for {:?}", talk_id);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::quests::QuestState;

    // =========================================================================
    // DialogueOption Tests
    // =========================================================================

    #[test]
    fn test_dialogue_option_available() {
        let option = DialogueOption::new(
            "Test Option",
            TalkId::Smith,
            |p| p.gold >= 100, // Requires 100+ gold
        );

        let mut player = Player::default();
        player.gold = 150;

        assert!(option.is_available(&player));
        assert_eq!(option.text, "Test Option");
        assert_eq!(option.next_talk_id, TalkId::Smith);
    }

    #[test]
    fn test_dialogue_option_unavailable() {
        let option = DialogueOption::new(
            "Expensive Option",
            TalkId::SmithBuy,
            |p| p.gold >= 1000, // Requires 1000+ gold
        );

        let mut player = Player::default();
        player.gold = 50; // Not enough gold

        assert!(!option.is_available(&player));
    }

    // =========================================================================
    // DialogueNode Tests
    // =========================================================================

    #[test]
    fn test_dialogue_node_filter_options() {
        const TEST_OPTIONS: &[DialogueOption] = &[
            DialogueOption::new("Always", TalkId::None, |_| true),
            DialogueOption::new("Rich Only", TalkId::None, |p| p.gold >= 100),
            DialogueOption::new("Never", TalkId::None, |_| false),
        ];

        let node = DialogueNode {
            talk_id: TalkId::Smith,
            greeting: Some("Greetings"),
            options: TEST_OPTIONS,
        };

        let mut player = Player::default();
        player.gold = 50;

        let available = node.get_available_options(&player);
        assert_eq!(available.len(), 1); // Only "Always" option
        assert_eq!(available[0].text, "Always");
    }

    #[test]
    fn test_dialogue_node_all_available() {
        const TEST_OPTIONS: &[DialogueOption] = &[
            DialogueOption::new("Option 1", TalkId::None, |_| true),
            DialogueOption::new("Option 2", TalkId::None, |_| true),
            DialogueOption::new("Option 3", TalkId::None, |_| true),
        ];

        let node = DialogueNode {
            talk_id: TalkId::Healer,
            greeting: Some("Hello"),
            options: TEST_OPTIONS,
        };

        let player = Player::default();
        let available = node.get_available_options(&player);
        assert_eq!(available.len(), 3);
    }

    #[test]
    fn test_dialogue_node_none_available() {
        const TEST_OPTIONS: &[DialogueOption] = &[
            DialogueOption::new("Option 1", TalkId::None, |_| false),
            DialogueOption::new("Option 2", TalkId::None, |_| false),
        ];

        let node = DialogueNode {
            talk_id: TalkId::Witch,
            greeting: None,
            options: TEST_OPTIONS,
        };

        let player = Player::default();
        let available = node.get_available_options(&player);
        assert_eq!(available.len(), 0);
    }

    // =========================================================================
    // Condition Helper Tests
    // =========================================================================

    #[test]
    fn test_has_items_to_sell_stub() {
        let player = Player::default();
        // Stub: always true
        assert!(has_items_to_sell(&player));
    }

    #[test]
    fn test_has_items_to_repair_stub() {
        let player = Player::default();
        // Stub: always false (no damaged items)
        assert!(!has_items_to_repair(&player));
    }

    #[test]
    fn test_has_staff_to_recharge_stub() {
        let player = Player::default();
        // Stub: always false (no staves)
        assert!(!has_staff_to_recharge(&player));
    }

    // =========================================================================
    // Dialogue Tree Tests
    // =========================================================================

    #[test]
    fn test_smith_dialogue_structure() {
        let player = Player::default();
        let available = SMITH_DIALOGUE.get_available_options(&player);

        // Stub: has_items_to_sell=true, has_items_to_repair=false
        // Expected: Sell (true), Repair (false), Buy (true), Premium (true), Leave (true)
        // = 4 available options
        assert_eq!(available.len(), 4);
        assert_eq!(SMITH_DIALOGUE.greeting, Some("What can I do for ye?"));
        assert_eq!(SMITH_DIALOGUE.talk_id, TalkId::Smith);
    }

    #[test]
    fn test_witch_dialogue_structure() {
        let player = Player::default();
        let available = WITCH_DIALOGUE.get_available_options(&player);

        // Stub: has_items_to_sell=true, has_staff_to_recharge=false
        // Expected: Sell (true), Recharge (false), Buy (true), Leave (true)
        // = 3 available options
        assert_eq!(available.len(), 3);
        assert_eq!(WITCH_DIALOGUE.greeting, Some("Welcome to my shop"));
        assert_eq!(WITCH_DIALOGUE.talk_id, TalkId::Witch);
    }

    #[test]
    fn test_healer_dialogue_structure() {
        let player = Player::default();
        let available = HEALER_DIALOGUE.get_available_options(&player);

        // All options always available
        assert_eq!(available.len(), 3);
        assert_eq!(HEALER_DIALOGUE.greeting, Some("How can I help you?"));
        assert_eq!(HEALER_DIALOGUE.talk_id, TalkId::Healer);
    }

    // =========================================================================
    // DialogueManager Tests (Day 97)
    // =========================================================================

    #[test]
    fn test_dialogue_manager_creation() {
        let manager = DialogueManager::new();

        // Verify all 8 dialogues are registered
        assert_eq!(manager.dialogues.len(), 8);

        // Verify no active dialogue initially
        assert!(manager.current_dialogue.is_none());
    }

    #[test]
    fn test_dialogue_manager_get_dialogue() {
        let manager = DialogueManager::new();

        // Test existing dialogues
        assert!(manager.get_dialogue(TalkId::Smith).is_some());
        assert!(manager.get_dialogue(TalkId::Witch).is_some());
        assert!(manager.get_dialogue(TalkId::Healer).is_some());
        assert!(manager.get_dialogue(TalkId::Boy).is_some());
        assert!(manager.get_dialogue(TalkId::Storyteller).is_some());
        assert!(manager.get_dialogue(TalkId::Tavern).is_some());
        assert!(manager.get_dialogue(TalkId::Drunk).is_some());
        assert!(manager.get_dialogue(TalkId::Barmaid).is_some());

        // Test non-existent dialogue
        assert!(manager.get_dialogue(TalkId::None).is_none());
    }

    #[test]
    fn test_dialogue_manager_start_dialogue() {
        let mut manager = DialogueManager::new();
        let player = Player::default();

        // Start smith dialogue
        let node = manager.start_dialogue(TalkId::Smith, &player);
        assert!(node.is_some());

        // Check current dialogue (separate borrow)
        assert_eq!(manager.current_dialogue, Some(TalkId::Smith));

        // Re-get node to verify structure
        let node = manager.get_dialogue(TalkId::Smith).unwrap();
        assert_eq!(node.talk_id, TalkId::Smith);
        assert_eq!(node.greeting, Some("What can I do for ye?"));
    }

    #[test]
    fn test_dialogue_manager_select_option() {
        let mut manager = DialogueManager::new();
        let player = Player::default();

        // Start smith dialogue
        manager.start_dialogue(TalkId::Smith, &player);

        // Select "Buy" option. Smith options are [Sell, Repair, Buy, PremiumBuy],
        // but Repair is unavailable for the default player so it is filtered out,
        // leaving [Sell, Buy, PremiumBuy] — Buy is at index 1.
        let result = manager.select_option(1, &player);
        assert!(result.is_ok());
        assert_eq!(manager.current_dialogue, Some(TalkId::SmithBuy));
    }

    #[test]
    fn test_dialogue_manager_select_invalid_option() {
        let mut manager = DialogueManager::new();
        let player = Player::default();

        // Start smith dialogue
        manager.start_dialogue(TalkId::Smith, &player);

        // Select invalid option index
        let result = manager.select_option(999, &player);
        assert!(result.is_err());
    }

    #[test]
    fn test_dialogue_manager_end_dialogue() {
        let mut manager = DialogueManager::new();
        let player = Player::default();

        // Start dialogue
        manager.start_dialogue(TalkId::Smith, &player);
        assert!(manager.current_dialogue.is_some());

        // End dialogue
        manager.end_dialogue();
        assert!(manager.current_dialogue.is_none());
    }

    #[test]
    fn test_towner_to_talk_id() {
        assert_eq!(towner_to_talk_id(TownerType::Smith), Some(TalkId::Smith));
        assert_eq!(towner_to_talk_id(TownerType::Witch), Some(TalkId::Witch));
        assert_eq!(towner_to_talk_id(TownerType::Healer), Some(TalkId::Healer));
        assert_eq!(towner_to_talk_id(TownerType::PegBoy), Some(TalkId::Boy));
        assert_eq!(towner_to_talk_id(TownerType::Story), Some(TalkId::Storyteller));
        assert_eq!(towner_to_talk_id(TownerType::Tavern), Some(TalkId::Tavern));
        assert_eq!(towner_to_talk_id(TownerType::Drunk), Some(TalkId::Drunk));
        assert_eq!(towner_to_talk_id(TownerType::Barmaid), Some(TalkId::Barmaid));

        // Non-dialogue towners
        assert_eq!(towner_to_talk_id(TownerType::DeadGuy), None);
        assert_eq!(towner_to_talk_id(TownerType::Cow), None);
    }

    // =========================================================================
    // Quest Dialogue Tests (Day 98)
    // =========================================================================

    #[test]
    fn test_quest_dialogue_smith_rock() {
        let manager = DialogueManager::new();
        let player = Player::default();
        let mut quest_manager = QuestManager::new();

        // Test 1: Quest inactive - should return None
        let quest_node = manager.get_quest_dialogue(TownerType::Smith, &player, &quest_manager);
        assert!(quest_node.is_none());

        // Test 2: Activate Rock quest - should return quest dialogue
        quest_manager.set_state(QuestId::Rock, QuestState::Active);
        let quest_node = manager.get_quest_dialogue(TownerType::Smith, &player, &quest_manager);
        assert!(quest_node.is_some());
        assert_eq!(quest_node.unwrap().talk_id, TalkId::Smith);
    }

    #[test]
    fn test_quest_dialogue_witch_mushroom() {
        let manager = DialogueManager::new();
        let player = Player::default();
        let mut quest_manager = QuestManager::new();

        // Test 1: Quest inactive
        let quest_node = manager.get_quest_dialogue(TownerType::Witch, &player, &quest_manager);
        assert!(quest_node.is_none());

        // Test 2: Activate Mushroom quest
        quest_manager.set_state(QuestId::Mushroom, QuestState::Active);
        let quest_node = manager.get_quest_dialogue(TownerType::Witch, &player, &quest_manager);
        assert!(quest_node.is_some());
        assert_eq!(quest_node.unwrap().talk_id, TalkId::Witch);
    }

    #[test]
    fn test_quest_dialogue_storyteller_betrayer() {
        let manager = DialogueManager::new();
        let player = Player::default();
        let mut quest_manager = QuestManager::new();

        // Test 1: Quest inactive
        let quest_node = manager.get_quest_dialogue(TownerType::Story, &player, &quest_manager);
        assert!(quest_node.is_none());

        // Test 2: Activate Betrayer quest
        quest_manager.set_state(QuestId::Betrayer, QuestState::Active);
        let quest_node = manager.get_quest_dialogue(TownerType::Story, &player, &quest_manager);
        assert!(quest_node.is_some());
        assert_eq!(quest_node.unwrap().talk_id, TalkId::Storyteller);
    }

    #[test]
    fn test_full_dialogue_flow() {
        let mut manager = DialogueManager::new();
        let player = Player::default();
        let quest_manager = QuestManager::new();

        // 1. Check for quest dialogue (should be none - no active quests)
        let quest_node = manager.get_quest_dialogue(TownerType::Smith, &player, &quest_manager);
        assert!(quest_node.is_none());

        // 2. Start normal dialogue
        let node = manager.start_dialogue(TalkId::Smith, &player);
        assert!(node.is_some());

        // 3. Get available options
        let node = node.unwrap();
        let options = node.get_available_options(&player);
        assert!(options.len() > 0);

        // 4. Find and select "Buy" option
        let buy_option_index = options
            .iter()
            .position(|opt| opt.next_talk_id == TalkId::SmithBuy)
            .expect("SmithBuy option should be available");

        let result = manager.select_option(buy_option_index, &player);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TalkId::SmithBuy);

        // 5. Trigger shop (stub)
        trigger_shop(TalkId::SmithBuy);

        // 6. End dialogue
        manager.end_dialogue();
        assert!(manager.current_dialogue.is_none());
    }

    #[test]
    fn test_shop_trigger_stub() {
        // Test stub functions (should not panic)
        trigger_shop(TalkId::SmithBuy);
        trigger_shop(TalkId::SmithSell);
        trigger_shop(TalkId::WitchBuy);
        trigger_shop(TalkId::HealerBuy);
        trigger_shop(TalkId::BoyBuy);
    }

    #[test]
    fn test_display_greeting_stub() {
        // Test stub function (should not panic)
        display_greeting("What can I do for ye?");
        display_greeting("Welcome to my shop");
        display_greeting("How can I help you?");
        display_greeting("Whatcha want?");
    }

    #[test]
    fn test_multi_npc_dialogue_switch() {
        let mut manager = DialogueManager::new();
        let player = Player::default();

        // Start Smith dialogue
        manager.start_dialogue(TalkId::Smith, &player);
        assert_eq!(manager.current_dialogue, Some(TalkId::Smith));

        // Switch to Witch dialogue (simulates talking to different NPC)
        manager.start_dialogue(TalkId::Witch, &player);
        assert_eq!(manager.current_dialogue, Some(TalkId::Witch));

        // Switch to Healer dialogue
        manager.start_dialogue(TalkId::Healer, &player);
        assert_eq!(manager.current_dialogue, Some(TalkId::Healer));

        // End dialogue
        manager.end_dialogue();
        assert!(manager.current_dialogue.is_none());
    }

    #[test]
    fn test_quest_item_check_stub() {
        let player = Player::default();

        // STUB: Always false until M10 Inventory
        assert!(!has_quest_item(&player, 9));  // IDI_ROCK = 9
        assert!(!has_quest_item(&player, 16)); // IDI_ANVIL = 16
        assert!(!has_quest_item(&player, 17)); // IDI_MUSHROOM = 17
        assert!(!has_quest_item(&player, 33)); // IDI_LAZSTAFF = 33
    }

    #[test]
    fn test_level_visited_check_stub() {
        let player = Player::default();

        // STUB: Always false until M3 Player extends
        assert!(!has_visited_level(&player, 4));  // Catacombs 1
        assert!(!has_visited_level(&player, 5));  // Catacombs 2
        assert!(!has_visited_level(&player, 9));  // Hell 1
        assert!(!has_visited_level(&player, 10)); // Hell 2
    }

    // =========================================================================
    // DialoguePlayer Trait Tests (Day 100 Integration)
    // =========================================================================

    #[test]
    fn test_stub_player_builder() {
        // Test builder pattern
        let player = StubPlayer::new()
            .with_gold(500)
            .with_level(10)
            .with_inventory_count(5)
            .with_quest_items(&[9, 17, 33])
            .with_visited_levels(&[1, 5, 9]);

        assert_eq!(player.gold(), 500);
        assert_eq!(player.level(), 10);
        assert_eq!(player.inventory_count(), 5);

        // Quest items
        assert!(player.has_item(9));   // Rock
        assert!(player.has_item(17));  // Mushroom
        assert!(player.has_item(33));  // LazStaff
        assert!(!player.has_item(16)); // Anvil (not added)

        // Visited levels
        assert!(player.has_visited_level(1));
        assert!(player.has_visited_level(5));
        assert!(player.has_visited_level(9));
        assert!(!player.has_visited_level(10)); // Not visited
    }

    #[test]
    fn test_dialogue_with_quest_items() {
        // Player with Rock (IDI_ROCK = 9)
        let player = StubPlayer::new().with_quest_items(&[9]);

        // Rock quest dialogue should detect item
        let rock_option = DialogueOption::new(
            "I found the rock",
            TalkId::Smith,
            |p| has_quest_item(p, 9),
        );

        assert!(rock_option.is_available(&player));

        // Anvil quest should NOT be available (no anvil)
        let anvil_option = DialogueOption::new(
            "I found the anvil",
            TalkId::Smith,
            |p| has_quest_item(p, 16),
        );

        assert!(!anvil_option.is_available(&player));
    }

    #[test]
    fn test_dialogue_with_visited_levels() {
        // Player visited levels 1-5 and 9
        let player = StubPlayer::new().with_visited_levels(&[1, 2, 3, 4, 5, 9]);

        // Catacombs story option should be available (level 5 visited)
        let catacombs_option = DialogueOption::new(
            "Tell me about the catacombs",
            TalkId::Storyteller,
            |p| has_visited_level(p, 5),
        );

        assert!(catacombs_option.is_available(&player));

        // Hell story option should be available (level 9 visited)
        let hell_option = DialogueOption::new(
            "Tell me about Hell",
            TalkId::Storyteller,
            |p| has_visited_level(p, 9),
        );

        assert!(hell_option.is_available(&player));

        // Level 10 not visited
        let level10_option = DialogueOption::new(
            "Tell me about level 10",
            TalkId::Storyteller,
            |p| has_visited_level(p, 10),
        );

        assert!(!level10_option.is_available(&player));
    }

    #[test]
    fn test_trait_generic_function() {
        // Test with StubPlayer (trait)
        fn check_player_status<P: DialoguePlayer>(player: &P) -> (i32, u8, bool) {
            (player.gold(), player.level(), player.has_visited_level(5))
        }

        let stub = StubPlayer::new()
            .with_gold(1000)
            .with_level(15)
            .with_visited_levels(&[5]);

        let (gold, level, visited) = check_player_status(&stub);
        assert_eq!(gold, 1000);
        assert_eq!(level, 15);
        assert!(visited);
    }

    #[test]
    fn test_dialogue_player_trait_bounds() {
        // Verify has_quest_item works with trait bounds
        fn quest_check<P: DialoguePlayer>(player: &P, item_id: u32) -> bool {
            has_quest_item(player, item_id)
        }

        let player = StubPlayer::new().with_quest_items(&[9, 17]);
        assert!(quest_check(&player, 9));
        assert!(quest_check(&player, 17));
        assert!(!quest_check(&player, 16));
    }
}
