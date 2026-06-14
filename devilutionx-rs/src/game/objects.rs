//! Game object system (Day 10-16)
//!
//! This module implements the complete object interaction system from DevilutionX,
//! including doors, chests, levers, books, shrines, fountains, and decorative objects.
//!
//! Reference: Source/objects.cpp, Source/objects.h

use crate::game::objdat::{ObjectId, SelectionRegion};
use crate::game::types::Point;
use crate::game::player_exact::{Player, CharacterAttribute, HeroClass};
use crate::game::shrine_effects;

// ============================================================================
// Rendering Interface (Day 17-18)
// ============================================================================

/// Rendering information for an object
///
/// This structure contains all data needed by the rendering system to draw an object.
/// It decouples the game logic (Object struct) from the rendering layer.
///
/// Reference: Source/engine/render/scrollrt.cpp DrawObject()
#[derive(Debug, Clone, Copy)]
pub struct ObjectRenderInfo {
    /// Object type (determines sprite to use)
    pub otype: ObjectId,
    /// Position in world coordinates
    pub position: Point,
    /// Current animation frame (0-indexed for sprite arrays)
    pub current_frame: i32,
    /// Sprite width for rendering
    pub animation_width: i32,
    /// Whether this object should be rendered (not deleted)
    pub should_render: bool,
    /// Whether the object is currently animating
    pub is_animated: bool,
}

// ============================================================================
// Object Structure (Day 10-11)
// ============================================================================

/// Main object structure matching C++ Object class
/// From Source/objects.h line ~80
#[derive(Debug, Clone)]
pub struct Object {
    /// Object type identifier
    pub otype: ObjectId,
    /// Position on the map
    pub position: Point,
    /// Deletion flag
    pub del_flag: bool,
    /// Animation state
    pub anim_flag: bool,
    /// Current animation frame
    pub anim_frame: i32,
    /// Animation frame length
    pub anim_len: i32,
    /// Animation frame count
    pub anim_cnt: i32,
    /// Animation delay
    pub anim_delay: i32,
    /// Animation width
    pub anim_width: i32,
    /// Selection region for interaction
    pub selection_region: SelectionRegion,
    /// Is this a trap?
    pub is_trap: bool,
    /// Random seed for this object
    pub rnd_seed: u32,
    /// Map range start (for books)
    pub ovar1: i32,
    /// Map range end (for books)
    pub ovar2: i32,
    /// Book message index
    pub book_message: i32,
    /// Quest-related flag
    pub is_quest_item: bool,
    /// Object solid flag
    pub solid: bool,
    /// Breakable flag
    pub breakable: bool,
    /// Door state (0=closed, 1=open, 2=blocked)
    pub door_state: i32,
    /// Previous tile value (for doors)
    pub pre_flag: i32,
    /// Object var3
    pub ovar3: i32,
    /// Object var4
    pub ovar4: i32,
    /// Object var5
    pub ovar5: i32,
    /// Object var6
    pub ovar6: i32,
}

impl Object {
    /// Create a new object with minimal initialization
    pub fn new(otype: ObjectId, position: Point) -> Self {
        Self {
            otype,
            position,
            del_flag: false,
            anim_flag: false,
            anim_frame: 0,
            anim_len: 0,
            anim_cnt: 0,
            anim_delay: 0,
            anim_width: 0,
            selection_region: SelectionRegion::None,
            is_trap: false,
            rnd_seed: 0,
            ovar1: 0,
            ovar2: 0,
            book_message: 0,
            is_quest_item: false,
            solid: false,
            breakable: false,
            door_state: 0,
            pre_flag: 0,
            ovar3: 0,
            ovar4: 0,
            ovar5: 0,
            ovar6: 0,
        }
    }

    // ========================================================================
    // Helper Methods (Day 10-11)
    // ========================================================================

    /// Check if object is a chest
    pub fn is_chest(&self) -> bool {
        matches!(
            self.otype,
            ObjectId::Chest1
                | ObjectId::Chest2
                | ObjectId::Chest3
                | ObjectId::TChest1
                | ObjectId::TChest2
                | ObjectId::TChest3
        )
    }

    /// Check if object is a door
    pub fn is_door(&self) -> bool {
        matches!(
            self.otype,
            ObjectId::L1LDoor
                | ObjectId::L1RDoor
                | ObjectId::L2LDoor
                | ObjectId::L2RDoor
                | ObjectId::L3LDoor
                | ObjectId::L3RDoor
                | ObjectId::L5LDoor
                | ObjectId::L5RDoor
        )
    }

    /// Check if object is a barrel
    pub fn is_barrel(&self) -> bool {
        matches!(self.otype, ObjectId::Barrel | ObjectId::BarrelEx)
    }

    /// Check if object is a shrine
    pub fn is_shrine(&self) -> bool {
        matches!(
            self.otype,
            ObjectId::ShrineL | ObjectId::ShrineR | ObjectId::GoatShrine | ObjectId::Cauldron
        )
    }

    /// Check if object is breakable
    pub fn is_breakable_object(&self) -> bool {
        self.breakable
    }

    /// Check if object is explosive
    pub fn is_explosive(&self) -> bool {
        self.is_barrel()
    }

    /// Check if object is a trapped chest
    pub fn is_trapped_chest(&self) -> bool {
        self.is_chest() && self.is_trap
    }

    /// Update animation state (basic version)
    ///
    /// Reference: Source/objects.cpp ProcessObjects() lines 4268-4278
    pub fn update_animation(&mut self) {
        if !self.anim_flag {
            return;
        }

        self.anim_cnt += 1;

        if self.anim_cnt < self.anim_delay {
            return;
        }

        self.anim_cnt = 0;
        self.anim_frame += 1;

        // Loop animation (frame wraps from len back to 1, not 0)
        if self.anim_frame > self.anim_len {
            self.anim_frame = 1;
        }
    }

    /// Stop animation at current frame
    ///
    /// Reference: Source/objects.cpp ObjectStopAnim()
    pub fn stop_animation(&mut self) {
        self.anim_flag = false;
    }

    /// Check if animation is finished (reached last frame)
    pub fn is_animation_finished(&self) -> bool {
        self.anim_frame >= self.anim_len
    }

    /// Get current animation frame (0-indexed for rendering)
    ///
    /// Converts internal 1-based frame indexing to 0-based for sprite rendering.
    /// This matches C++'s `currentSprite()` which returns `(*_oAnimData)[_oAnimFrame - 1]`
    ///
    /// Reference: Source/objects.h line 268
    pub fn get_animation_frame(&self) -> i32 {
        if self.anim_frame > 0 {
            self.anim_frame - 1  // Convert to 0-indexed
        } else {
            0
        }
    }

    /// Get render information for this object
    ///
    /// Returns data needed by the rendering system to draw this object.
    /// This matches the data accessed in C++ DrawObject() function.
    ///
    /// Reference: Source/engine/render/scrollrt.cpp DrawObject()
    pub fn get_render_info(&self) -> ObjectRenderInfo {
        ObjectRenderInfo {
            otype: self.otype,
            position: self.position,
            current_frame: self.get_animation_frame(),
            animation_width: self.anim_width,
            should_render: !self.del_flag,
            is_animated: self.anim_flag && self.anim_len > 0,
        }
    }

    /// Set animation parameters
    pub fn set_animation(&mut self, delay: i32, len: i32, start_frame: i32) {
        self.anim_delay = delay;
        self.anim_len = len;
        self.anim_frame = start_frame;
        self.anim_cnt = 0;
        self.anim_flag = true;
    }

    /// Reset animation to start
    pub fn reset_animation(&mut self) {
        self.anim_frame = 1;
        self.anim_cnt = 0;
    }

    /// Check if object can be interacted with
    pub fn can_interact_with(&self) -> bool {
        self.selection_region != SelectionRegion::None
    }

    /// Get door state as enum (uses ovar4 matching C++ _oVar4)
    pub fn get_door_state(&self) -> DoorState {
        match self.ovar4 {
            0 => DoorState::Closed,
            1 => DoorState::Open,
            2 => DoorState::Blocked,
            _ => DoorState::Closed,
        }
    }

    /// Set door state (uses ovar4 matching C++ _oVar4)
    pub fn set_door_state(&mut self, state: DoorState) {
        self.ovar4 = state as i32;
    }

    /// Get book message index
    pub fn get_book_message(&self) -> i32 {
        self.book_message
    }

    /// Set book message (renamed from initialize_book to avoid conflict)
    pub fn set_book_message(&mut self, msg_id: i32) {
        self.book_message = msg_id;
    }

    /// Set book map range
    pub fn set_book_map_range(&mut self, start: i32, end: i32) {
        self.ovar1 = start;
        self.ovar2 = end;
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::new(ObjectId::L1Light, Point::new(0, 0))
    }
}

// ============================================================================
// Object Manager (Day 10-11)
// ============================================================================

/// Object manager for maintaining the global object array
/// Matches C++ Objects[] array (MAX_OBJECTS = 127)
pub struct ObjectManager {
    objects: Vec<Option<Object>>,
}

impl ObjectManager {
    /// Create a new object manager
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    /// Spawn a new object
    pub fn spawn(&mut self, object: Object) -> Option<usize> {
        // Find first empty slot
        for (i, slot) in self.objects.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(object);
                return Some(i);
            }
        }

        // Add new slot if under limit (127)
        if self.objects.len() < 127 {
            self.objects.push(Some(object));
            Some(self.objects.len() - 1)
        } else {
            None
        }
    }

    /// Remove an object by index
    pub fn remove(&mut self, index: usize) {
        if let Some(slot) = self.objects.get_mut(index) {
            *slot = None;
        }
    }

    /// Get object by index
    pub fn get(&self, index: usize) -> Option<&Object> {
        self.objects.get(index).and_then(|slot| slot.as_ref())
    }

    /// Get mutable object by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Object> {
        self.objects.get_mut(index).and_then(|slot| slot.as_mut())
    }

    /// Iterate over all active objects
    pub fn iter(&self) -> impl Iterator<Item = &Object> {
        self.objects.iter().filter_map(|slot| slot.as_ref())
    }

    /// Iterate mutably over all active objects
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Object> {
        self.objects.iter_mut().filter_map(|slot| slot.as_mut())
    }

    /// Get total object count
    pub fn count(&self) -> usize {
        self.objects.iter().filter(|slot| slot.is_some()).count()
    }

    /// Process all object animations (Day 17-18)
    ///
    /// Reference: Source/objects.cpp ProcessObjects() lines 4201-4290
    pub fn process_animations(&mut self) {
        for slot in self.objects.iter_mut() {
            if let Some(object) = slot {
                // Update object-specific logic
                match object.otype {
                    // Static objects (stop animation)
                    ObjectId::Barrel
                    | ObjectId::BarrelEx
                    | ObjectId::ShrineL
                    | ObjectId::ShrineR => {
                        object.stop_animation();
                    }

                    // Doors (update door state)
                    ObjectId::L1LDoor
                    | ObjectId::L1RDoor
                    | ObjectId::L2LDoor
                    | ObjectId::L2RDoor
                    | ObjectId::L3LDoor
                    | ObjectId::L3RDoor => {
                        update_door_animation(object);
                    }

                    // Sarcophagus
                    ObjectId::Sarc | ObjectId::L5Sarc => {
                        update_sarcophagus_animation(object);
                    }

                    // Default: just update animation
                    _ => {}
                }

                // Update animation frame
                object.update_animation();
            }
        }
    }

    /// Clean up deleted objects (Day 17-18)
    ///
    /// Reference: Source/objects.cpp ProcessObjects() lines 4281-4288
    /// Remove objects marked for deletion
    ///
    /// Reference: Source/objects.cpp ProcessObjects() lines 4281-4288
    pub fn cleanup_deleted(&mut self) {
        for slot in self.objects.iter_mut() {
            if let Some(object) = slot {
                if object.del_flag {
                    *slot = None;
                }
            }
        }
    }

    /// Get render information for all active objects
    ///
    /// This method provides a view of all renderable objects for the rendering system.
    /// It's the bridge between the game logic and rendering layer.
    ///
    /// # Returns
    /// Iterator over (index, ObjectRenderInfo) for all active objects
    pub fn get_all_render_info(&self) -> impl Iterator<Item = (usize, ObjectRenderInfo)> + '_ {
        self.objects
            .iter()
            .enumerate()
            .filter_map(|(idx, slot)| {
                slot.as_ref()
                    .map(|obj| (idx, obj.get_render_info()))
            })
            .filter(|(_, info)| info.should_render)
    }

    /// Get render information for a specific object
    ///
    /// # Arguments
    /// * `index` - Object index (0 to MAX_OBJECTS-1)
    ///
    /// # Returns
    /// ObjectRenderInfo if object exists and should be rendered
    pub fn get_object_render_info(&self, index: usize) -> Option<ObjectRenderInfo> {
        self.objects
            .get(index)?
            .as_ref()
            .filter(|obj| !obj.del_flag)
            .map(|obj| obj.get_render_info())
    }
}

impl Default for ObjectManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Animation Update Functions (Day 17-18)
// ============================================================================

/// Update door animation based on state
///
/// Reference: Source/objects.cpp UpdateDoor()
pub(crate) fn update_door_animation(door: &mut Object) {
    let state = door.get_door_state();

    match state {
        DoorState::Closed => {
            // Door is closed, reset to frame 1
            if door.anim_frame != 1 {
                door.anim_frame = 1;
                door.anim_cnt = 0;
            }
        }
        DoorState::Open => {
            // Door is open, stay at last frame
            if door.anim_frame < door.anim_len {
                door.anim_frame = door.anim_len;
            }
        }
        DoorState::Blocked => {
            // Door is blocked, stop animation
            door.stop_animation();
        }
    }
}

/// Update sarcophagus animation
///
/// Reference: Source/objects.cpp UpdateSarcophagus()
pub(crate) fn update_sarcophagus_animation(sarc: &mut Object) {
    // If sarcophagus is opened (ovar2 == 1), stop animating
    if sarc.ovar2 == 1 {
        sarc.stop_animation();
    }
}

// ============================================================================
// Day 12-13: Object Interactions
// ============================================================================

/// Door states matching C++ implementation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorState {
    Closed = 0,
    Open = 1,
    Blocked = 2,
}

/// Chest item generation modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChestItemMode {
    Random = 0,  // CreateRndItem
    Useful = 1,  // CreateRndUseful
}

/// Open a door
/// Matches: void OpenDoor(Object &door)
/// From Source/objects.cpp line ~1810
pub fn open_door(door: &mut Object) {
    door.set_door_state(DoorState::Open);
    door.anim_frame = door.anim_len; // Jump to fully open (last frame)
    door.selection_region = SelectionRegion::None;

    // TODO: Update pathfinding map
    // SetDoorStateOpen(door.position);
}

/// Close a door
/// Matches: void CloseDoor(Object &door)
/// From Source/objects.cpp line ~1820
pub fn close_door(door: &mut Object) {
    door.set_door_state(DoorState::Closed);
    door.anim_frame = 1; // Back to closed position (first frame)
    door.selection_region = SelectionRegion::Bottom;

    // TODO: Update pathfinding map
    // SetDoorStateClosed(door.position);
}

/// Operate a door (toggle open/close)
/// Matches: void OperateDoor(Object &door, bool sendmsg)
/// From Source/objects.cpp line ~1760
pub fn operate_door(door: &mut Object, _send_network_msg: bool) -> bool {
    if !door.can_interact_with() {
        return false;
    }

    // Check if door is blocked
    // TODO: implement IsDoorClear() collision detection
    let is_clear = true; // Placeholder

    match door.get_door_state() {
        DoorState::Closed => {
            if is_clear {
                open_door(door);
                // TODO: PlaySfxLoc(SfxID::OperateDoor, door.position);
            } else {
                door.set_door_state(DoorState::Blocked);
            }
        }
        DoorState::Open => {
            if is_clear {
                close_door(door);
                // TODO: PlaySfxLoc(SfxID::OperateDoor, door.position);
            } else {
                door.set_door_state(DoorState::Blocked);
            }
        }
        DoorState::Blocked => {
            // Try to clear blockage
            if is_clear {
                door.set_door_state(DoorState::Closed);
            }
        }
    }

    // TODO: Network sync
    // if (sendmsg) NetSendCmdLoc(MyPlayerId, false, CMD_OPERATEOBJ, door.position);

    true
}

/// Operate a chest
/// Matches: void OperateChest(Player &player, Object &chest, bool sendmsg)
/// From Source/objects.cpp line ~2020
pub fn operate_chest(chest: &mut Object, _player_pos: Point, _send_loot_msg: bool) -> bool {
    if !chest.can_interact_with() {
        return false;
    }

    chest.selection_region = SelectionRegion::None;
    chest.anim_frame += 1;

    // TODO: PlaySfxLoc(SfxID::OperateChest, chest.position);

    // Handle trapped chests
    if chest.is_trapped_chest() {
        // Trap types: Arrow, Fire, Lightning, Gas, Poison, Explosion
        // TODO: integrate missile system
        // AddMissile(chest.position, player.position, Direction, MissileID::TrapType, ...);
        chest.is_trap = false; // Disable trap after triggered
    }

    // Generate loot
    // TODO: integrate item system
    // SetRndSeed(chest._oRndSeed);
    // CreateRndItem(chest.position, false, sendLootMsg, false);
    // or CreateRndUseful(chest.position, sendLootMsg);

    // TODO: Network sync
    // if (sendmsg) NetSendCmdParam1(false, CMD_OPERATEOBJ, chest.position);

    true
}

/// Operate a lever
/// Matches: void OperateLever(Object &lever, bool sendmsg)
/// From Source/objects.cpp line ~2252
pub fn operate_lever(lever: &mut Object, _send_network_msg: bool) -> bool {
    if !lever.can_interact_with() {
        return false;
    }

    lever.selection_region = SelectionRegion::None;

    // Toggle lever animation
    if lever.anim_frame == 0 {
        lever.anim_frame = 1;
    } else {
        lever.anim_frame = 0;
    }

    // TODO: PlaySfxLoc(SfxID::OperateLever, lever.position);

    // Lever effects (door opening, quest triggers, etc.)
    // TODO: integrate quest and door systems
    // UpdateLeverState(lever);

    // TODO: Network sync
    // if (sendmsg) NetSendCmdLoc(MyPlayerId, false, CMD_OPERATEOBJ, lever.position);

    true
}

/// Update lever state and trigger effects
pub fn update_lever_state(_lever: &mut Object) {
    // TODO: implement lever-specific logic
    // - Quest levers (Skeleton King, Chamber of Bone, etc.)
    // - Door levers
    // - Trap levers
}

/// Operate a book
/// Matches: void OperateBook(Player &player, Object &book, bool sendmsg)
/// From Source/objects.cpp line ~2730
pub fn operate_book(book: &mut Object, _send_network_msg: bool) -> bool {
    if !book.can_interact_with() {
        return false;
    }

    // Special books that act as levers
    if matches!(
        book.otype,
        ObjectId::BlindBook | ObjectId::BloodBook | ObjectId::SteelTome
    ) {
        book.selection_region = SelectionRegion::None;
        // TODO: trigger lever effect
        return true;
    }

    // Story books display text
    // TODO: integrate text display system
    // InitQTextMsg(book.book_message);
    // NetSendCmdLoc(MyPlayerId, false, CMD_OPERATEOBJ, book.position);

    true
}

/// General object operation dispatcher
/// Matches: void OperateObject(Player &player, Object &object)
/// From Source/objects.cpp line 4380
pub fn operate_object(
    object: &mut Object,
    player_pos: Point,
    send_network_msg: bool,
) -> bool {
    if !object.can_interact_with() {
        return false;
    }

    match object.otype {
        // Doors
        ObjectId::L1LDoor
        | ObjectId::L1RDoor
        | ObjectId::L2LDoor
        | ObjectId::L2RDoor
        | ObjectId::L3LDoor
        | ObjectId::L3RDoor
        | ObjectId::L5LDoor
        | ObjectId::L5RDoor => operate_door(object, send_network_msg),

        // Levers
        ObjectId::Lever | ObjectId::L5Lever | ObjectId::SwitchSkl => {
            operate_lever(object, send_network_msg)
        }

        // Chests
        ObjectId::Chest1
        | ObjectId::Chest2
        | ObjectId::Chest3
        | ObjectId::TChest1
        | ObjectId::TChest2
        | ObjectId::TChest3 => operate_chest(object, player_pos, send_network_msg),

        // Books
        ObjectId::Book2L
        | ObjectId::BlindBook
        | ObjectId::BloodBook
        | ObjectId::SteelTome
        | ObjectId::StoryBook
        | ObjectId::L5Books => operate_book(object, send_network_msg),

        // Shrines (Day 14-15 + Day 26-30 Player Integration)
        ObjectId::ShrineL | ObjectId::ShrineR | ObjectId::GoatShrine | ObjectId::Cauldron => {
            // TODO: Pass actual player from game state
            let mut dummy_player = Player::new();
            operate_shrine(object, player_pos, send_network_msg, &mut dummy_player)
        }

        // Fountains (Day 14-15)
        ObjectId::BloodFtn
        | ObjectId::PurifyingFtn
        | ObjectId::MurkyFtn
        | ObjectId::TearFtn => operate_fountain(object, player_pos, send_network_msg),

        // Sarcophagus (Day 14-15)
        ObjectId::Sarc | ObjectId::L5Sarc => {
            operate_sarcophagus(object, send_network_msg, send_network_msg)
        }

        // Weapon racks (Day 14-15)
        ObjectId::WeaponRack => {
            operate_weapon_rack(object, send_network_msg, send_network_msg)
        }

        // Armor stands (Day 14-15)
        ObjectId::ArmorStand | ObjectId::ArmorStandN | ObjectId::WarArmor => {
            operate_armor_stand(object, 1, send_network_msg, send_network_msg)
        }

        // Bookcases (Day 14-15)
        ObjectId::BookcaseL | ObjectId::BookcaseR => {
            operate_bookcase(object, send_network_msg, send_network_msg)
        }

        // Decapitated body (Day 14-15)
        ObjectId::Decap => operate_decapitated_body(object, send_network_msg, send_network_msg),

        _ => false,
    }
}

// ============================================================================
// Day 14-15: Special Objects - Shrines, Fountains, Sarcophagus, Decorative
// ============================================================================

/// Shrine types matching C++ shrine_type enum (Source/objects.cpp:64)
/// Total: 35 shrine types (0-34, including NumberOfShrineTypes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Fascinating = 9,      // Restore mana (was missing)
    Cryptic = 10,         // Nova spell (was at index 9)
    MagicaL2 = 11,        // Magical variant (was missing)
    Eldritch = 12,        // All shrines become potions
    Eerie = 13,           // -2 light radius
    Divine = 14,          // Full heal HP+Mana
    Holy = 15,            // Cast Phasing
    Sacred = 16,          // Cast charged bolt (was missing)
    Spiritual = 17,       // Fill inventory with gold
    Spooky = 18,          // Teleport to random player
    Abandoned = 19,       // +2 Dexterity
    Creepy = 20,          // +2 Strength
    Quiet = 21,           // +2 Vitality
    Secluded = 22,        // Reveal map
    Ornate = 23,          // Cast holy bolt (was missing)
    Glimmering = 24,      // Identify items
    Tainted = 25,         // -1 all resistances
    Oily = 26,            // +1 all resistances (Hellfire)
    Glowing = 27,         // +5% magic damage (Hellfire)
    Mendicant = 28,       // Full heal, lose gold (Hellfire)
    Sparkling = 29,       // +1 random stat
    Town = 30,            // Town portal
    Shimmering = 31,      // +2 Magic
    Solar = 32,           // Time-based stat boost (was missing)
    Murphys = 33,         // Break item or lose gold (was missing)
}

/// Operate a shrine
/// Matches: void OperateShrine(Player &player, Object &object, SfxID sType)
///
/// Day 26-30: Player integration added
pub fn operate_shrine(shrine: &mut Object, _player_pos: Point, _send_network_msg: bool, player: &mut Player) -> bool {
    if !shrine.can_interact_with() {
        return false;
    }

    shrine.selection_region = SelectionRegion::None;

    let shrine_type = match shrine.ovar1 {
        0 => ShrineType::Mysterious,
        1 => ShrineType::Hidden,
        2 => ShrineType::Gloomy,
        3 => ShrineType::Weird,
        4 => ShrineType::Magical,
        5 => ShrineType::Stone,
        6 => ShrineType::Religious,
        7 => ShrineType::Enchanted,
        8 => ShrineType::Thaumaturgic,
        9 => ShrineType::Fascinating,
        10 => ShrineType::Cryptic,
        11 => ShrineType::MagicaL2,      // Magical variant
        12 => ShrineType::Eldritch,
        13 => ShrineType::Eerie,
        14 => ShrineType::Divine,
        15 => ShrineType::Holy,
        16 => ShrineType::Sacred,
        17 => ShrineType::Spiritual,
        18 => ShrineType::Spooky,
        19 => ShrineType::Abandoned,
        20 => ShrineType::Creepy,
        21 => ShrineType::Quiet,
        22 => ShrineType::Secluded,
        23 => ShrineType::Ornate,
        24 => ShrineType::Glimmering,
        25 => ShrineType::Tainted,
        26 => ShrineType::Oily,
        27 => ShrineType::Glowing,
        28 => ShrineType::Mendicant,
        29 => ShrineType::Sparkling,
        30 => ShrineType::Town,
        31 => ShrineType::Shimmering,
        32 => ShrineType::Solar,
        33 => ShrineType::Murphys,
        _ => ShrineType::Mysterious,
    };

    apply_shrine_effect(shrine_type, shrine, player);
    true
}

/// Apply shrine effect to player
/// Matches: void OperateShrine*(Player &player, Object &shrine, SfxID sType)
///
/// Day 26-30: Integrated with Player system via shrine_effects module
fn apply_shrine_effect(shrine_type: ShrineType, shrine: &mut Object, player: &mut Player) {
    use rand::Rng;

    // Mark shrine as activated
    shrine.ovar2 = 1;

    // Apply player effects
    match shrine_type {
        // === Stat Modification Shrines (Full Implementation) ===

        ShrineType::Mysterious => {
            // -1 all stats, +6 random stat
            let mut rng = rand::rng();
            let random_stat = match rng.random_range(0..4) {
                0 => CharacterAttribute::Strength,
                1 => CharacterAttribute::Magic,
                2 => CharacterAttribute::Dexterity,
                _ => CharacterAttribute::Vitality,
            };
            shrine_effects::apply_mysterious(player, random_stat);
        }

        ShrineType::Weird => {
            // Swap two random stats
            let mut rng = rand::rng();
            let stat1 = match rng.random_range(0..4) {
                0 => CharacterAttribute::Strength,
                1 => CharacterAttribute::Magic,
                2 => CharacterAttribute::Dexterity,
                _ => CharacterAttribute::Vitality,
            };
            let mut stat2 = match rng.random_range(0..4) {
                0 => CharacterAttribute::Strength,
                1 => CharacterAttribute::Magic,
                2 => CharacterAttribute::Dexterity,
                _ => CharacterAttribute::Vitality,
            };
            // Ensure different stats
            while stat2 as u8 == stat1 as u8 {
                stat2 = match rng.random_range(0..4) {
                    0 => CharacterAttribute::Strength,
                    1 => CharacterAttribute::Magic,
                    2 => CharacterAttribute::Dexterity,
                    _ => CharacterAttribute::Vitality,
                };
            }
            shrine_effects::apply_weird(player, stat1, stat2);
        }

        ShrineType::Abandoned => shrine_effects::apply_abandoned(player),  // +2 Dex
        ShrineType::Creepy => shrine_effects::apply_creepy(player),        // +2 Str
        ShrineType::Quiet => shrine_effects::apply_quiet(player),          // +2 Vit
        ShrineType::Shimmering => shrine_effects::apply_shimmering(player), // +2 Mag

        ShrineType::Sparkling => {
            // +2 random stat (Str/Mag/Dex/Vit)
            let mut rng = rand::rng();
            let random_stat = match rng.random_range(0..4) {
                0 => CharacterAttribute::Strength,
                1 => CharacterAttribute::Magic,
                2 => CharacterAttribute::Dexterity,
                _ => CharacterAttribute::Vitality,
            };
            shrine_effects::apply_solar(player, random_stat);
        }

        ShrineType::Solar => {
            // Time-based stat boost (+2 to stat based on hour)
            let mut rng = rand::rng();
            let random_stat = match rng.random_range(0..4) {
                0 => CharacterAttribute::Strength,
                1 => CharacterAttribute::Magic,
                2 => CharacterAttribute::Dexterity,
                _ => CharacterAttribute::Vitality,
            };
            shrine_effects::apply_solar(player, random_stat);
        }

        ShrineType::Murphys => {
            // +1 all stats (Murphy's Law implementation in C++ breaks items)
            shrine_effects::apply_murphys(player);
        }

        ShrineType::Glowing => shrine_effects::apply_murphys(player),      // +1 all stats

        // === HP/Mana Restoration Shrines (Full Implementation) ===

        ShrineType::Divine => shrine_effects::apply_divine(player),         // Full HP+Mana
        ShrineType::Fascinating => shrine_effects::apply_fascinating(player), // Restore Mana
        ShrineType::Holy => shrine_effects::apply_holy(player),             // Cast Phasing spell
        ShrineType::Mendicant => shrine_effects::apply_mendicant(player),   // Full heal, lose gold

        // === Resistance Shrines (Full Implementation) ===

        ShrineType::Tainted => shrine_effects::apply_tainted(player),       // -1 all resist
        ShrineType::Oily => shrine_effects::apply_oily(player),             // +1 all resist

        // === Light Radius Shrine (Full Implementation) ===

        ShrineType::Eerie => shrine_effects::apply_eerie(player),           // -2 light radius

        // === Item Effect Shrines (TODO: Requires Item System) ===

        ShrineType::Hidden => {
            // TODO(Item): Identify items in inventory
            shrine_effects::apply_hidden(player, 0);
        }
        ShrineType::Gloomy => {
            // TODO(Item): Damage active weapon
            shrine_effects::apply_gloomy(player);
        }
        ShrineType::Magical => {
            // TODO(Item): Recharge staves
            shrine_effects::apply_magical(player, 0);
        }
        ShrineType::MagicaL2 => {
            // MagicaL2 is same as Magical (variant in C++)
            shrine_effects::apply_magical(player, 0);
        }
        ShrineType::Stone => shrine_effects::apply_stone(player),           // Recharge items
        ShrineType::Religious => shrine_effects::apply_religious(player),   // Repair armor
        ShrineType::Enchanted => {
            // TODO(Item): Recharge all items
            shrine_effects::apply_enchanted(player, 0);
        }

        // === Spell Effect Shrines (TODO: Requires Spell System) ===

        ShrineType::Thaumaturgic => {
            // TODO(Spell): Grant all spells
            shrine_effects::apply_thaumaturgic();
        }
        ShrineType::Cryptic => shrine_effects::apply_cryptic(player),       // Cast Nova
        ShrineType::Eldritch => {
            // TODO(Spell): All shrines become potions
            shrine_effects::apply_eldritch();
        }
        ShrineType::Sacred => {
            // TODO(Spell): Cast Charged Bolt + reduce max mana
            shrine_effects::apply_sacred(player);
        }
        ShrineType::Ornate => {
            // TODO(Spell): Cast Holy Bolt + reduce max mana
            shrine_effects::apply_ornate(player);
        }
        ShrineType::Spiritual => {
            // TODO(Spell): Restore spell charges
            shrine_effects::apply_spiritual(player, 0);
        }
        ShrineType::Spooky => shrine_effects::apply_spooky(player),         // No mana cost

        // === Map Effect Shrines (TODO: Requires Map System) ===

        ShrineType::Secluded => shrine_effects::apply_secluded(player),     // Phasing
        ShrineType::Glimmering => shrine_effects::apply_glimmering(player), // Mana shield
        ShrineType::Town => shrine_effects::apply_town(player),             // Town portal
    }
}

/// Operate a fountain
/// Matches: bool OperateFountains(Player &player, Object &fountain)
pub fn operate_fountain(fountain: &mut Object, _player_pos: Point, _send_network_msg: bool) -> bool {
    if !fountain.can_interact_with() {
        return false;
    }

    let applied = match fountain.otype {
        ObjectId::BloodFtn => {
            fountain.ovar1 = 64;
            true
        }
        ObjectId::PurifyingFtn => {
            fountain.ovar1 = 64;
            true
        }
        ObjectId::MurkyFtn => {
            fountain.selection_region = SelectionRegion::None;
            fountain.ovar1 = 1;
            true
        }
        ObjectId::TearFtn => {
            fountain.selection_region = SelectionRegion::None;
            let random_value = (fountain.rnd_seed >> 16) % 12;
            let from_stat = random_value / 3;
            let mut to_stat = random_value % 3;
            if to_stat >= from_stat {
                to_stat += 1;
            }
            fountain.ovar1 = from_stat as i32;
            fountain.ovar2 = to_stat as i32;
            true
        }
        _ => false,
    };

    applied
}

/// Operate a sarcophagus
/// Matches: void OperateSarcophagus(Object &sarcophagus, bool sendmsg, bool sendLootMsg)
pub fn operate_sarcophagus(sarc: &mut Object, _send_network_msg: bool, _send_loot_msg: bool) -> bool {
    if !sarc.can_interact_with() {
        return false;
    }

    sarc.selection_region = SelectionRegion::None;
    sarc.ovar2 = 1;
    true
}

/// Operate a weapon rack
/// Matches: void OperateWeaponRack(Object &weaponRack, bool sendmsg, bool sendLootMsg)
pub fn operate_weapon_rack(rack: &mut Object, _send_network_msg: bool, _send_loot_msg: bool) -> bool {
    if !rack.can_interact_with() {
        return false;
    }

    rack.selection_region = SelectionRegion::None;
    rack.anim_frame += 1;
    rack.ovar1 = 1;
    true
}

/// Operate an armor stand
/// Matches: void OperateArmorStand(Object &armorStand, bool sendmsg, bool sendLootMsg)
pub fn operate_armor_stand(stand: &mut Object, current_level: i32, _send_network_msg: bool, _send_loot_msg: bool) -> bool {
    if !stand.can_interact_with() {
        return false;
    }

    stand.selection_region = SelectionRegion::None;
    stand.anim_frame += 1;
    stand.ovar1 = current_level;
    true
}

/// Operate a bookcase
/// Matches: void OperateBookcase(Object &bookcase, bool sendmsg, bool sendLootMsg)
pub fn operate_bookcase(bookcase: &mut Object, _send_network_msg: bool, _send_loot_msg: bool) -> bool {
    if !bookcase.can_interact_with() {
        return false;
    }

    bookcase.selection_region = SelectionRegion::None;
    bookcase.anim_frame -= 2;
    bookcase.ovar1 = 1;
    true
}

/// Operate a decapitated body
/// Matches: void OperateDecapitatedBody(Object &corpse, bool sendmsg, bool sendLootMsg)
pub fn operate_decapitated_body(corpse: &mut Object, _send_network_msg: bool, _send_loot_msg: bool) -> bool {
    if !corpse.can_interact_with() {
        return false;
    }

    corpse.selection_region = SelectionRegion::None;
    corpse.ovar1 = 1;
    true
}

/// Operate a mushroom patch (quest item)
///
/// **C++ Reference**: `OperateMushroomPatch()` in objects.cpp:2075
pub fn operate_mushroom_patch(patch: &mut Object, _player_pos: Point) -> bool {
    if !patch.can_interact_with() {
        return false;
    }

    patch.selection_region = SelectionRegion::None;
    patch.ovar1 = 1;
    // In C++: Creates IMISC_FUNGALTM item at player position
    true
}

/// Operate a slain hero corpse (gives random equipment)
///
/// **C++ Reference**: `OperateSlainHero()` in objects.cpp:2135
pub fn operate_slain_hero(corpse: &mut Object, _send_network_msg: bool, _send_loot_msg: bool) -> bool {
    if corpse.ovar1 != 0 {
        return false;
    }

    corpse.selection_region = SelectionRegion::None;
    corpse.ovar1 = 1;
    // In C++: Spawns random equipment based on item type
    true
}

/// Operate a trap lever
///
/// **C++ Reference**: `OperateTrapLever()` in objects.cpp:2162
pub fn operate_trap_lever(lever: &mut Object) -> bool {
    if !lever.can_interact_with() {
        return false;
    }

    // Toggle trap state
    lever.anim_frame = 2;
    lever.ovar1 = if lever.ovar1 == 0 { 1 } else { 0 };
    true
}

/// Operate a pedestal (Blood Stone quest)
///
/// **C++ Reference**: `OperatePedestal()` in objects.cpp:2209
pub fn operate_pedestal(pedestal: &mut Object, _player_has_blood_stone: bool) -> bool {
    if pedestal.ovar1 >= 3 {
        return false;
    }

    // Each activation adds one blood stone
    pedestal.ovar1 += 1;
    if pedestal.ovar1 >= 3 {
        pedestal.selection_region = SelectionRegion::None;
    }
    true
}

/// Operate Inn Sign chest (Cornerstone of the World quest)
///
/// **C++ Reference**: `OperateInnSignChest()` in objects.cpp:2106
pub fn operate_inn_sign_chest(chest: &mut Object, _send_network_msg: bool) -> bool {
    if chest.ovar1 != 0 {
        return false;
    }

    chest.selection_region = SelectionRegion::None;
    chest.ovar1 = 1;
    // In C++: Spawns IMISC_NOTE item
    true
}

/// Operate story book (displays message)
///
/// **C++ Reference**: `OperateStoryBook()` in objects.cpp:1920 area
pub fn operate_story_book(book: &mut Object) -> bool {
    if !book.can_interact_with() {
        return false;
    }

    // Display book message (book_message field contains text ID)
    book.ovar1 = 1;
    true
}

/// Operate tortured body (decoration, may drop items)
///
/// **C++ Reference**: `void OperateTorturedBody()` pattern
pub fn operate_tortured_body(body: &mut Object, _send_loot_msg: bool) -> bool {
    if body.ovar1 != 0 {
        return false;
    }

    body.selection_region = SelectionRegion::None;
    body.ovar1 = 1;
    // May drop gold or items
    true
}

/// Operate magic circle (teleportation)
///
/// **C++ Reference**: `OperateMagicCircle()` pattern
pub fn operate_magic_circle(circle: &mut Object) -> bool {
    if !circle.can_interact_with() {
        return false;
    }

    // Teleport player to destination
    circle.ovar1 = 1;
    true
}

/// Operate a barrel
///
/// **C++ Reference**: `OperateBarrel()` in objects.cpp
pub fn operate_barrel(barrel: &mut Object, _send_network_msg: bool) -> bool {
    if barrel.ovar1 != 0 {
        return false;
    }

    barrel.selection_region = SelectionRegion::None;
    barrel.ovar1 = 1;
    barrel.breakable = false;

    // In C++: Plays crash sound, spawns items/gold/nothing
    // Also can spawn explosion trap (small chance)
    true
}

/// Operate a goat shrine
///
/// **C++ Reference**: `OperateGoatShrine()` in objects.cpp
pub fn operate_goat_shrine(shrine: &mut Object) -> bool {
    if shrine.ovar1 != 0 {
        return false;
    }

    shrine.selection_region = SelectionRegion::None;
    shrine.ovar1 = 1;

    // Goat shrine drops random scroll
    true
}

/// Operate a cauldron
///
/// **C++ Reference**: `OperateCauldron()` in objects.cpp
pub fn operate_cauldron(cauldron: &mut Object) -> bool {
    if cauldron.ovar1 != 0 {
        return false;
    }

    cauldron.selection_region = SelectionRegion::None;
    cauldron.ovar1 = 1;

    // Cauldron gives random potion effect
    true
}

/// Operate Magical shrine - casts ManaShield on the player
///
/// **C++ Reference**: `OperateShrineMagical()` in objects.cpp
pub fn operate_shrine_magical(_player: &Player) {
    // AddMissile(player.position.tile, player.position.tile, player._pdir,
    //            MissileID::ManaShield, TARGET_MONSTERS, player, 0, 2 * leveltype);
    // Casts ManaShield missile on player
}

/// Operate Stone shrine - recharges all staff items
///
/// **C++ Reference**: `OperateShrineStone()` in objects.cpp
pub fn operate_shrine_stone(_player: &mut Player) {
    // Recharge all staff items to max charges
    // In C++: loops through PlayerItemsRange and restores _iCharges
    // for item in player.inv_body.iter_mut() { ... }
}

/// Operate Eerie shrine - +2 Magic
///
/// **C++ Reference**: `OperateShrineEerie()` in objects.cpp
pub fn operate_shrine_eerie(player: &mut Player) {
    player._p_base_mag += 2;
    // CheckStats(player);
    // CalcPlrInv(player, true);
}

/// Operate Divine shrine - full restore HP/Mana and spawn potions
///
/// **C++ Reference**: `OperateShrineDivine()` in objects.cpp
pub fn operate_shrine_divine(player: &mut Player, _current_level: i32) {
    player._p_hit_points = player._p_max_hp;
    player._p_mana = player._p_max_mana;
    // If level < 4: spawn FullMana + FullHeal potions
    // Else: spawn 2x FullRejuv potions
}

/// Operate Holy shrine - casts Phasing spell
///
/// **C++ Reference**: `OperateShrineHoly()` in objects.cpp
pub fn operate_shrine_holy(_player: &Player) {
    // AddMissile for Phasing effect - teleport the player
}

/// Operate Spiritual shrine - fills empty inventory with gold
///
/// **C++ Reference**: `OperateShrineSpiritual()` in objects.cpp
pub fn operate_shrine_spiritual(player: &mut Player, level_type: i32) {
    // For each empty slot in inventory, create gold stack
    // Gold amount = 5 * leveltype + rng(10 * leveltype)
    let _gold_amount = 5 * level_type;
    player._p_gold += 100; // Simplified: add some gold
}

/// Operate Spooky shrine - heals other players, warns activator
///
/// **C++ Reference**: `OperateShrineSpooky()` in objects.cpp
pub fn operate_shrine_spooky(_player: &Player) {
    // If activated by MyPlayer: show warning message
    // Otherwise: fully restore HP and Mana of MyPlayer
}

/// Operate Abandoned shrine - +2 Dexterity
///
/// **C++ Reference**: `OperateShrineAbandoned()` in objects.cpp
pub fn operate_shrine_abandoned(player: &mut Player) {
    player._p_base_dex += 2;
    // CheckStats(player);
    // CalcPlrInv(player, true);
}

/// Operate Creepy shrine - +2 Strength
///
/// **C++ Reference**: `OperateShrineCreepy()` in objects.cpp
pub fn operate_shrine_creepy(player: &mut Player) {
    player._p_base_str += 2;
    // CheckStats(player);
    // CalcPlrInv(player, true);
}

/// Operate Quiet shrine - +2 Vitality
///
/// **C++ Reference**: `OperateShrineQuiet()` in objects.cpp
pub fn operate_shrine_quiet(player: &mut Player) {
    player._p_base_vit += 2;
    // CheckStats(player);
    // CalcPlrInv(player, true);
}

/// Operate Secluded shrine - reveals entire automap
///
/// **C++ Reference**: `OperateShrineSecluded()` in objects.cpp
pub fn operate_shrine_secluded() {
    // For each x,y in dungeon:
    //   UpdateAutomapExplorer({ x, y }, MAP_EXP_SHRINE);
    // Reveals the entire map
}

/// Operate Glimmering shrine - identifies all magic items
///
/// **C++ Reference**: `OperateShrineGlimmering()` in objects.cpp
pub fn operate_shrine_glimmering(_player: &mut Player) {
    // For each item in player's inventory:
    //   if item.is_magical && !item.identified:
    //       item.identified = true;
    // In C++: loops through PlayerItemsRange
}

/// Operate Tainted shrine - affects other players' stats
///
/// **C++ Reference**: `OperateShrineTainted()` in objects.cpp
pub fn operate_shrine_tainted() {
    // If activator == MyPlayer: show warning
    // Otherwise: randomly modify MyPlayer's stats (+1 to one, -1 to others)
}

/// Operate Oily shrine - +2 to primary stat, spawns firewall
///
/// **C++ Reference**: `OperateShrineOily()` in objects.cpp
pub fn operate_shrine_oily(player: &mut Player) {
    // Increase primary stat by 2 based on class:
    // Warrior: +2 Str
    // Rogue: +2 Dex
    // Sorcerer: +2 Mag
    // Barbarian: +2 Vit
    // Monk: +1 Str, +1 Dex
    // Bard: +1 Dex, +1 Mag
    match player._p_class {
        HeroClass::Warrior => player._p_base_str += 2,
        HeroClass::Rogue => player._p_base_dex += 2,
        HeroClass::Sorcerer => player._p_base_mag += 2,
        HeroClass::Barbarian => player._p_base_vit += 2,
        HeroClass::Monk => {
            player._p_base_str += 1;
            player._p_base_dex += 1;
        }
        HeroClass::Bard => {
            player._p_base_dex += 1;
            player._p_base_mag += 1;
        }
    }
    // Also spawns FireWall missile targeting player
}

/// Operate Glowing shrine - gain Magic based on XP, lose 5% XP
///
/// **C++ Reference**: `OperateShrineGlowing()` in objects.cpp
pub fn operate_shrine_glowing(player: &mut Player) {
    // Add min(experience/1000, 5) to Magic
    let magic_gain = (player._p_experience / 1000).min(5) as i32;
    player._p_base_mag += magic_gain;

    // Take 5% XP (or all if < 5000)
    if player._p_experience > 5000 {
        player._p_experience = (player._p_experience as f64 * 0.95) as u32;
    } else {
        player._p_experience = 0;
    }
}

/// Operate Mendicant shrine - trade gold for XP
///
/// **C++ Reference**: `OperateShrineMendicant()` in objects.cpp
pub fn operate_shrine_mendicant(player: &mut Player) {
    let gold = player._p_gold / 2;
    player._p_experience += gold as u32;
    player._p_gold -= gold;
}

/// Operate Sparkling shrine - gain XP, triggers flash trap
///
/// **C++ Reference**: `OperateShrineSparkling()` in objects.cpp
pub fn operate_shrine_sparkling(player: &mut Player, current_level: i32) {
    player._p_experience += (1000 * current_level) as u32;
    // Spawns FlashBottom missile targeting player
}

/// Operate Town shrine - spawns town portal
///
/// **C++ Reference**: `OperateShrineTown()` in objects.cpp
pub fn operate_shrine_town() {
    // AddMissile for TownPortal near the shrine
}

/// Operate Shimmering shrine - restore full mana
///
/// **C++ Reference**: `OperateShrineShimmering()` in objects.cpp
pub fn operate_shrine_shimmering(player: &mut Player) {
    player._p_mana = player._p_max_mana;
}

/// Operate Solar shrine - stat bonus based on time of day
///
/// **C++ Reference**: `OperateShrineSolar()` in objects.cpp
pub fn operate_shrine_solar(player: &mut Player) {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Get current hour (simplified)
    let hour = if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
        ((duration.as_secs() / 3600) % 24) as i32
    } else {
        20 // Default to night time
    };

    // Apply stat based on time of day
    if hour >= 20 || hour < 4 {
        player._p_base_vit += 2; // Night (8pm-4am)
    } else if hour >= 18 {
        player._p_base_mag += 2; // Evening (6pm-8pm)
    } else if hour >= 12 {
        player._p_base_str += 2; // Afternoon (12pm-6pm)
    } else {
        player._p_base_dex += 2; // Morning (4am-12pm)
    }
}

/// Operate Murphy's shrine - damages items or takes gold
///
/// **C++ Reference**: `OperateShrineMurphys()` in objects.cpp
pub fn operate_shrine_murphys(player: &mut Player) {
    // Try to halve durability of a random equipped item
    // If no item was damaged, take 1/3 of player's gold
    let lost_gold = player._p_gold / 3;
    player._p_gold -= lost_gold;
}

/// Operate Book Lever - quest book interaction
///
/// **C++ Reference**: `OperateBookLever()` in objects.cpp
pub fn operate_book_lever(book: &mut Object, _send_msg: bool) -> bool {
    if book.ovar1 != 0 {
        return false;
    }
    book.ovar1 = 1;
    // Quest-related: triggers events based on quest state
    true
}

/// Operate Chamber of Bone book
///
/// **C++ Reference**: `OperateChamberOfBoneBook()` in objects.cpp
pub fn operate_chamber_of_bone_book(book: &mut Object, _send_msg: bool) -> bool {
    if book.ovar1 != 0 {
        return false;
    }
    book.ovar1 = 1;
    // Triggers Chamber of Bone quest events
    true
}

/// Operate Book Stand - drops a book item
///
/// **C++ Reference**: `OperateBookStand()` in objects.cpp
pub fn operate_book_stand(stand: &mut Object, _send_msg: bool, _send_loot_msg: bool) -> bool {
    if stand.ovar1 != 0 {
        return false;
    }
    stand.selection_region = SelectionRegion::None;
    stand.ovar1 = 1;
    // Spawns book item
    true
}

/// Operate Laz's Stand - quest object
///
/// **C++ Reference**: `OperateLazStand()` in objects.cpp
pub fn operate_laz_stand(stand: &mut Object) -> bool {
    if stand.ovar1 != 0 {
        return false;
    }
    stand.selection_region = SelectionRegion::None;
    stand.ovar1 = 1;
    // Quest-related: part of Lazarus quest
    true
}

/// Operate Trap object - triggers trap effects
///
/// **C++ Reference**: `OperateTrap()` in objects.cpp
pub fn operate_trap(trap: &mut Object) -> bool {
    if !trap.is_trap {
        return false;
    }
    // Spawns various trap missiles based on trap type:
    // Arrow, Firebolt, Lightning, etc.
    true
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_creation() {
        let obj = Object::new(ObjectId::L1Light, Point::new(10, 20));
        assert_eq!(obj.otype, ObjectId::L1Light);
        assert_eq!(obj.position, Point::new(10, 20));
        assert!(!obj.del_flag);
    }

    #[test]
    fn test_object_default() {
        let obj = Object::default();
        assert_eq!(obj.position, Point::new(0, 0));
        assert_eq!(obj.anim_frame, 0);
    }

    #[test]
    fn test_object_manager() {
        let mut mgr = ObjectManager::new();
        let obj = Object::new(ObjectId::L1Light, Point::new(5, 5));

        let idx = mgr.spawn(obj).unwrap();
        assert_eq!(mgr.count(), 1);

        mgr.remove(idx);
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_is_chest() {
        let mut obj = Object::new(ObjectId::L1Light, Point::new(0, 0));
        obj.otype = ObjectId::Chest1;
        assert!(obj.is_chest());
    }

    #[test]
    fn test_is_door() {
        let mut obj = Object::new(ObjectId::L1Light, Point::new(0, 0));
        obj.otype = ObjectId::L1LDoor;
        assert!(obj.is_door());
    }

    #[test]
    fn test_door_states() {
        let mut door = Object::new(ObjectId::L1Light, Point::new(0, 0));
        door.set_door_state(DoorState::Open);
        assert_eq!(door.get_door_state(), DoorState::Open);
        assert_eq!(door.ovar4, 1); // Uses ovar4, not door_state
    }

    #[test]
    fn test_open_close_door() {
        let mut door = Object::new(ObjectId::L1Light, Point::new(0, 0));
        door.otype = ObjectId::L1LDoor;
        door.set_door_state(DoorState::Closed);
        door.anim_len = 5;
        door.selection_region = SelectionRegion::Bottom;

        open_door(&mut door);
        assert_eq!(door.get_door_state(), DoorState::Open);
        assert_eq!(door.selection_region, SelectionRegion::None);
        assert_eq!(door.anim_frame, 5); // Jump to fully open

        close_door(&mut door);
        assert_eq!(door.get_door_state(), DoorState::Closed);
        assert_eq!(door.anim_frame, 1); // Back to closed
    }

    #[test]
    fn test_operate_door() {
        let mut door = Object::new(ObjectId::L1Light, Point::new(0, 0));
        door.otype = ObjectId::L2RDoor;
        door.set_door_state(DoorState::Closed);
        door.anim_len = 5;
        door.selection_region = SelectionRegion::Bottom;

        let result = operate_door(&mut door, false);
        assert!(result);
        assert_eq!(door.get_door_state(), DoorState::Open);
    }

    #[test]
    fn test_operate_chest() {
        let mut chest = Object::new(ObjectId::L1Light, Point::new(0, 0));
        chest.otype = ObjectId::Chest2;
        chest.selection_region = SelectionRegion::Bottom;

        let result = operate_chest(&mut chest, Point::new(1, 1), false);
        assert!(result);
        assert_eq!(chest.selection_region, SelectionRegion::None);
    }

    #[test]
    fn test_operate_lever() {
        let mut lever = Object::new(ObjectId::L1Light, Point::new(0, 0));
        lever.otype = ObjectId::SwitchSkl;
        lever.selection_region = SelectionRegion::Middle;

        let result = operate_lever(&mut lever, false);
        assert!(result);
        assert_eq!(lever.selection_region, SelectionRegion::None);
    }

    #[test]
    fn test_operate_book() {
        let mut book = Object::new(ObjectId::L1Light, Point::new(0, 0));
        book.otype = ObjectId::StoryBook;
        book.selection_region = SelectionRegion::Bottom;
        book.book_message = 100;

        let result = operate_book(&mut book, false);
        assert!(result);
    }

    #[test]
    fn test_operate_shrine() {
        let mut shrine = Object::new(ObjectId::L1Light, Point::new(0, 0));
        shrine.otype = ObjectId::ShrineL;
        shrine.selection_region = SelectionRegion::Bottom;
        shrine.ovar1 = 20;

        let mut player = Player::new();
        let result = operate_shrine(&mut shrine, Point::new(0, 0), false, &mut player);
        assert!(result);
        assert_eq!(shrine.selection_region, SelectionRegion::None);
        assert_eq!(shrine.ovar2, 1);
    }

    #[test]
    fn test_operate_fountain_blood() {
        let mut fountain = Object::new(ObjectId::L1Light, Point::new(0, 0));
        fountain.otype = ObjectId::BloodFtn;
        fountain.selection_region = SelectionRegion::Bottom;

        let result = operate_fountain(&mut fountain, Point::new(0, 0), false);
        assert!(result);
        assert_eq!(fountain.ovar1, 64);
    }

    #[test]
    fn test_operate_fountain_tear() {
        let mut fountain = Object::new(ObjectId::L1Light, Point::new(0, 0));
        fountain.otype = ObjectId::TearFtn;
        fountain.selection_region = SelectionRegion::Bottom;
        fountain.rnd_seed = 0x12345678;

        let result = operate_fountain(&mut fountain, Point::new(0, 0), false);
        assert!(result);
        assert_eq!(fountain.selection_region, SelectionRegion::None);
    }

    #[test]
    fn test_operate_sarcophagus() {
        let mut sarc = Object::new(ObjectId::L1Light, Point::new(0, 0));
        sarc.otype = ObjectId::Sarc;
        sarc.selection_region = SelectionRegion::Bottom;

        let result = operate_sarcophagus(&mut sarc, false, false);
        assert!(result);
        assert_eq!(sarc.selection_region, SelectionRegion::None);
        assert_eq!(sarc.ovar2, 1);
    }

    #[test]
    fn test_operate_weapon_rack() {
        let mut rack = Object::new(ObjectId::L1Light, Point::new(0, 0));
        rack.otype = ObjectId::WeaponRack;
        rack.selection_region = SelectionRegion::Bottom;
        rack.anim_frame = 0;

        let result = operate_weapon_rack(&mut rack, false, false);
        assert!(result);
        assert_eq!(rack.selection_region, SelectionRegion::None);
        assert_eq!(rack.anim_frame, 1);
        assert_eq!(rack.ovar1, 1);
    }

    #[test]
    fn test_operate_armor_stand() {
        let mut stand = Object::new(ObjectId::L1Light, Point::new(0, 0));
        stand.otype = ObjectId::ArmorStand;
        stand.selection_region = SelectionRegion::Bottom;
        stand.anim_frame = 0;

        let result = operate_armor_stand(&mut stand, 5, false, false);
        assert!(result);
        assert_eq!(stand.selection_region, SelectionRegion::None);
        assert_eq!(stand.anim_frame, 1);
        assert_eq!(stand.ovar1, 5);
    }

    #[test]
    fn test_operate_bookcase() {
        let mut bookcase = Object::new(ObjectId::L1Light, Point::new(0, 0));
        bookcase.otype = ObjectId::BookcaseL;
        bookcase.selection_region = SelectionRegion::Bottom;
        bookcase.anim_frame = 10;

        let result = operate_bookcase(&mut bookcase, false, false);
        assert!(result);
        assert_eq!(bookcase.selection_region, SelectionRegion::None);
        assert_eq!(bookcase.anim_frame, 8);
        assert_eq!(bookcase.ovar1, 1);
    }

    #[test]
    fn test_operate_decapitated_body() {
        let mut corpse = Object::new(ObjectId::L1Light, Point::new(0, 0));
        corpse.otype = ObjectId::Decap;
        corpse.selection_region = SelectionRegion::Bottom;

        let result = operate_decapitated_body(&mut corpse, false, false);
        assert!(result);
        assert_eq!(corpse.selection_region, SelectionRegion::None);
        assert_eq!(corpse.ovar1, 1);
    }

    #[test]
    fn test_operate_object_door() {
        let mut door = Object::new(ObjectId::L1Light, Point::new(0, 0));
        door.otype = ObjectId::L3LDoor;
        door.selection_region = SelectionRegion::Bottom;
        door.anim_len = 5;

        let result = operate_object(&mut door, Point::new(0, 0), false);
        assert!(result);
        assert_eq!(door.get_door_state(), DoorState::Open);
    }

    #[test]
    fn test_operate_object_shrine() {
        let mut shrine = Object::new(ObjectId::L1Light, Point::new(0, 0));
        shrine.otype = ObjectId::GoatShrine;
        shrine.selection_region = SelectionRegion::Bottom;
        shrine.ovar1 = 5;

        let result = operate_object(&mut shrine, Point::new(0, 0), false);
        assert!(result);
        assert_eq!(shrine.selection_region, SelectionRegion::None);
    }

    #[test]
    fn test_operate_object_fountain() {
        let mut fountain = Object::new(ObjectId::L1Light, Point::new(0, 0));
        fountain.otype = ObjectId::MurkyFtn;
        fountain.selection_region = SelectionRegion::Bottom;

        let result = operate_object(&mut fountain, Point::new(0, 0), false);
        assert!(result);
        assert_eq!(fountain.selection_region, SelectionRegion::None);
    }

    // ========================================================================
    // Day 17-18: Animation System Tests
    // ========================================================================

    #[test]
    fn test_animation_basic_update() {
        let mut obj = Object::new(ObjectId::L1Light, Point::new(0, 0));
        obj.set_animation(3, 10, 1); // delay=3, len=10, start=1

        assert_eq!(obj.anim_frame, 1);
        assert_eq!(obj.anim_cnt, 0);
        assert!(obj.anim_flag);

        // First tick: cnt++, no frame change
        obj.update_animation();
        assert_eq!(obj.anim_frame, 1);
        assert_eq!(obj.anim_cnt, 1);

        // Second tick: cnt++, no frame change
        obj.update_animation();
        assert_eq!(obj.anim_frame, 1);
        assert_eq!(obj.anim_cnt, 2);

        // Third tick: cnt reaches delay, frame advances, cnt resets
        obj.update_animation();
        assert_eq!(obj.anim_frame, 2);
        assert_eq!(obj.anim_cnt, 0);
    }

    #[test]
    fn test_animation_loop() {
        let mut obj = Object::new(ObjectId::L1Light, Point::new(0, 0));
        obj.set_animation(1, 5, 1); // delay=1, len=5, start=1

        // Advance to last frame
        for _ in 0..4 {
            obj.update_animation();
        }
        assert_eq!(obj.anim_frame, 5);

        // Next update should loop back to 1
        obj.update_animation();
        assert_eq!(obj.anim_frame, 1);
    }

    #[test]
    fn test_animation_stop() {
        let mut obj = Object::new(ObjectId::L1Light, Point::new(0, 0));
        obj.set_animation(1, 10, 1);
        assert!(obj.anim_flag);

        obj.stop_animation();
        assert!(!obj.anim_flag);

        // Animation should not update when stopped
        let prev_frame = obj.anim_frame;
        obj.update_animation();
        assert_eq!(obj.anim_frame, prev_frame);
    }

    #[test]
    fn test_animation_is_finished() {
        let mut obj = Object::new(ObjectId::L1Light, Point::new(0, 0));
        obj.set_animation(1, 5, 1);

        assert!(!obj.is_animation_finished());

        // Advance to last frame
        for _ in 0..4 {
            obj.update_animation();
        }
        assert!(obj.is_animation_finished());
    }

    #[test]
    fn test_get_animation_frame() {
        let mut obj = Object::new(ObjectId::L1Light, Point::new(0, 0));
        obj.anim_frame = 0;
        assert_eq!(obj.get_animation_frame(), 0);

        obj.anim_frame = 1;
        assert_eq!(obj.get_animation_frame(), 0); // 1-indexed to 0-indexed

        obj.anim_frame = 5;
        assert_eq!(obj.get_animation_frame(), 4);
    }

    #[test]
    fn test_reset_animation() {
        let mut obj = Object::new(ObjectId::L1Light, Point::new(0, 0));
        obj.set_animation(3, 10, 5);

        // Advance animation (not exact multiples of delay)
        for _ in 0..7 {
            obj.update_animation();
        }
        assert!(obj.anim_frame > 1);
        // After 7 updates with delay=3: cnt will be 1 (7 % 3 = 1)
        // Frames advanced: 7/3 = 2 times, so frame = 5+2 = 7

        // Reset should go back to frame 1
        obj.reset_animation();
        assert_eq!(obj.anim_frame, 1);
        assert_eq!(obj.anim_cnt, 0);
    }

    #[test]
    fn test_door_animation_closed() {
        let mut door = Object::new(ObjectId::L1LDoor, Point::new(0, 0));
        door.set_animation(3, 5, 3);
        door.ovar1 = 0; // Closed state

        update_door_animation(&mut door);
        assert_eq!(door.anim_frame, 1); // Should reset to closed position
    }

    #[test]
    fn test_door_animation_open() {
        let mut door = Object::new(ObjectId::L1LDoor, Point::new(0, 0));
        door.set_animation(3, 5, 1);
        door.set_door_state(DoorState::Open); // Uses ovar4

        update_door_animation(&mut door);
        assert_eq!(door.anim_frame, 5); // Should go to fully open
    }

    #[test]
    fn test_door_animation_blocked() {
        let mut door = Object::new(ObjectId::L1LDoor, Point::new(0, 0));
        door.set_animation(3, 5, 3);
        door.set_door_state(DoorState::Blocked); // Uses ovar4

        update_door_animation(&mut door);
        assert!(!door.anim_flag); // Should stop animation
    }

    #[test]
    fn test_sarcophagus_animation_opened() {
        let mut sarc = Object::new(ObjectId::Sarc, Point::new(0, 0));
        sarc.set_animation(2, 8, 1);
        sarc.ovar2 = 1; // Opened flag

        update_sarcophagus_animation(&mut sarc);
        assert!(!sarc.anim_flag); // Should stop animation when opened
    }

    #[test]
    fn test_sarcophagus_animation_closed() {
        let mut sarc = Object::new(ObjectId::Sarc, Point::new(0, 0));
        sarc.set_animation(2, 8, 1);
        sarc.ovar2 = 0; // Not opened

        update_sarcophagus_animation(&mut sarc);
        assert!(sarc.anim_flag); // Should still animate
    }

    #[test]
    fn test_object_manager_process_animations() {
        let mut manager = ObjectManager::new();

        // Add animated door (delay=2 to avoid immediate frame advance)
        let mut door = Object::new(ObjectId::L1LDoor, Point::new(0, 0));
        door.set_animation(2, 5, 1); // delay=2
        door.set_door_state(DoorState::Closed); // Uses ovar4
        manager.spawn(door);

        // Add barrel (should stop animation)
        let mut barrel = Object::new(ObjectId::Barrel, Point::new(1, 1));
        barrel.set_animation(2, 4, 1);
        manager.spawn(barrel);

        // Process animations
        manager.process_animations();

        // Check door was updated (should stay at frame 1 because delay=2)
        let door = manager.get(0).unwrap();
        assert_eq!(door.anim_frame, 1); // Reset to closed, cnt=1 (not advanced yet)

        // Check barrel animation was stopped
        let barrel = manager.get(1).unwrap();
        assert!(!barrel.anim_flag);
    }

    #[test]
    fn test_object_manager_cleanup_deleted() {
        let mut manager = ObjectManager::new();

        // Add two objects
        let obj1 = Object::new(ObjectId::L1Light, Point::new(0, 0));
        let mut obj2 = Object::new(ObjectId::Barrel, Point::new(1, 1));
        obj2.del_flag = true; // Mark for deletion

        manager.spawn(obj1);
        manager.spawn(obj2);

        assert_eq!(manager.count(), 2);

        // Cleanup deleted objects
        manager.cleanup_deleted();

        assert_eq!(manager.count(), 1);
        assert!(manager.get(0).is_some());
        assert!(manager.get(1).is_none());
    }

    #[test]
    fn test_door_opening_sequence() {
        let mut door = Object::new(ObjectId::L2RDoor, Point::new(5, 5));
        door.set_animation(2, 6, 1); // delay=2, len=6
        door.selection_region = SelectionRegion::Bottom;

        // Door starts closed
        assert_eq!(door.get_door_state(), DoorState::Closed);

        // Operate door (should open)
        let result = operate_door(&mut door, false);
        assert!(result);
        assert_eq!(door.get_door_state(), DoorState::Open);
        assert_eq!(door.anim_frame, 6); // Jump to open position

        // Update door animation (should stay at open frame)
        update_door_animation(&mut door);
        assert_eq!(door.anim_frame, 6);
    }

    #[test]
    fn test_shrine_animation_stopped() {
        let mut manager = ObjectManager::new();

        let mut shrine = Object::new(ObjectId::ShrineR, Point::new(3, 3));
        shrine.set_animation(3, 10, 1);
        manager.spawn(shrine);

        manager.process_animations();

        let shrine = manager.get(0).unwrap();
        assert!(!shrine.anim_flag); // Shrines stop animation
    }

    // ========================================================================
    // Rendering Integration Tests (Day 17-18)
    // ========================================================================

    #[test]
    fn test_object_render_info() {
        let mut obj = Object::new(ObjectId::Barrel, Point::new(10, 15));
        obj.set_animation(2, 8, 1);
        obj.anim_width = 64;

        let info = obj.get_render_info();

        assert_eq!(info.otype, ObjectId::Barrel);
        assert_eq!(info.position.x, 10);
        assert_eq!(info.position.y, 15);
        assert_eq!(info.current_frame, 0); // Frame 1 in internal -> 0 in render
        assert_eq!(info.animation_width, 64);
        assert!(info.should_render);
        assert!(info.is_animated);
    }

    #[test]
    fn test_render_info_deleted_object() {
        let mut obj = Object::new(ObjectId::Chest1, Point::new(5, 5));
        obj.del_flag = true;

        let info = obj.get_render_info();

        assert!(!info.should_render); // Deleted objects shouldn't render
    }

    #[test]
    fn test_render_info_animation_frame_conversion() {
        let mut obj = Object::new(ObjectId::L1LDoor, Point::new(8, 8));
        obj.set_animation(2, 6, 1);

        // Internal frame 1 -> render frame 0
        assert_eq!(obj.get_animation_frame(), 0);

        // Advance animation
        obj.anim_frame = 3;
        assert_eq!(obj.get_animation_frame(), 2); // Frame 3 -> 2

        obj.anim_frame = 6;
        assert_eq!(obj.get_animation_frame(), 5); // Frame 6 -> 5
    }

    #[test]
    fn test_manager_get_all_render_info() {
        let mut manager = ObjectManager::new();

        // Add objects
        manager.spawn(Object::new(ObjectId::Barrel, Point::new(1, 1)));
        manager.spawn(Object::new(ObjectId::Chest1, Point::new(2, 2)));

        // Mark one as deleted
        if let Some(obj) = manager.get_mut(1) {
            obj.del_flag = true;
        }

        manager.spawn(Object::new(ObjectId::ShrineL, Point::new(3, 3)));

        // Get render info
        let render_infos: Vec<_> = manager.get_all_render_info().collect();

        // Should only get non-deleted objects
        assert_eq!(render_infos.len(), 2);
        assert_eq!(render_infos[0].1.otype, ObjectId::Barrel);
        assert_eq!(render_infos[1].1.otype, ObjectId::ShrineL);
    }

    #[test]
    fn test_manager_get_object_render_info() {
        let mut manager = ObjectManager::new();

        let mut door = Object::new(ObjectId::L2LDoor, Point::new(7, 7));
        door.set_animation(2, 6, 1);
        manager.spawn(door);

        // Get render info for existing object
        let info = manager.get_object_render_info(0);
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.otype, ObjectId::L2LDoor);
        assert_eq!(info.current_frame, 0);

        // Get render info for non-existent object
        let info = manager.get_object_render_info(99);
        assert!(info.is_none());
    }

    #[test]
    fn test_render_info_with_animation_update() {
        let mut obj = Object::new(ObjectId::Lever, Point::new(4, 4));
        obj.set_animation(1, 4, 1); // delay=1, len=4

        // Initial state
        let info = obj.get_render_info();
        assert_eq!(info.current_frame, 0);
        assert!(info.is_animated);

        // Update animation
        obj.update_animation();
        let info = obj.get_render_info();
        assert_eq!(info.current_frame, 1); // Frame 2 -> 1

        obj.update_animation();
        let info = obj.get_render_info();
        assert_eq!(info.current_frame, 2); // Frame 3 -> 2

        obj.update_animation();
        let info = obj.get_render_info();
        assert_eq!(info.current_frame, 3); // Frame 4 -> 3

        // Should loop back
        obj.update_animation();
        let info = obj.get_render_info();
        assert_eq!(info.current_frame, 0); // Frame 1 -> 0
    }

    #[test]
    fn test_render_info_stopped_animation() {
        let mut obj = Object::new(ObjectId::Bookstand, Point::new(6, 6));
        obj.set_animation(2, 5, 1);

        assert!(obj.get_render_info().is_animated);

        // Stop animation
        obj.stop_animation();

        let info = obj.get_render_info();
        assert!(!info.is_animated); // anim_flag is false
    }
}

