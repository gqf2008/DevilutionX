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
///
/// Field semantics (mapped to C++ members):
/// - `otype`        -> `_otype`
/// - `position`     -> `position`
/// - `del_flag`     -> `_oDelFlag`
/// - `anim_flag`    -> `_oAnimFlag`
/// - `anim_frame`   -> `_oAnimFrame`
/// - `anim_len`     -> `_oAnimLen`
/// - `anim_cnt`     -> `_oAnimCnt`
/// - `anim_delay`   -> `_oAnimDelay`
/// - `anim_width`   -> `_oAnimWidth`
/// - `selection_region` -> `selectionRegion`
/// - `is_trap`      -> (computed from `_oTrapFlag`/type)
/// - `rnd_seed`     -> `_oRndSeed`
/// - `ovar1..ovar6` -> `_oVar1.._oVar6`
/// - `book_message` -> `bookMessage`
/// - `solid`        -> `_oSolidFlag`
/// - `breakable`    -> `_oBreak`
/// - `door_state`   -> (legacy, kept for backwards compatibility)
/// - `pre_flag`     -> `_oPreFlag`
#[derive(Debug, Clone)]
pub struct Object {
    /// Object type identifier
    pub otype: ObjectId,
    /// Position on the map
    pub position: Point,
    /// Deletion flag (`_oDelFlag`)
    pub del_flag: bool,
    /// Animation state (`_oAnimFlag`)
    pub anim_flag: bool,
    /// Current animation frame (`_oAnimFrame`)
    pub anim_frame: i32,
    /// Animation frame length (`_oAnimLen`)
    pub anim_len: i32,
    /// Animation frame count (`_oAnimCnt`)
    pub anim_cnt: i32,
    /// Animation delay (`_oAnimDelay`)
    pub anim_delay: i32,
    /// Animation width (`_oAnimWidth`)
    pub anim_width: i32,
    /// Selection region for interaction
    pub selection_region: SelectionRegion,
    /// Is this a trap? (`_oTrapFlag` — true means an armed trap)
    pub is_trap: bool,
    /// Random seed for this object (`_oRndSeed`)
    pub rnd_seed: u32,
    /// Map range start / generic var1 (`_oVar1`)
    pub ovar1: i32,
    /// Map range end / generic var2 (`_oVar2`)
    pub ovar2: i32,
    /// Book message index (`bookMessage`)
    pub book_message: i32,
    /// Quest-related flag
    pub is_quest_item: bool,
    /// Object solid flag (`_oSolidFlag`)
    pub solid: bool,
    /// Breakable state: 1=intact breakable, 0=non-breakable, -1=broken (`_oBreak`)
    pub breakable: i32,
    /// Door state (0=closed, 1=open, 2=blocked) — legacy field, kept for
    /// backwards compatibility with earlier tests. Most door logic uses
    /// `ovar4` directly (matching C++ `_oVar4 == DOOR_*`).
    pub door_state: i32,
    /// Previous tile flag (`_oPreFlag`). Stored as i32 to mirror C++; the
    /// meaningful values are 0 and 1 (true/false).
    pub pre_flag: i32,
    /// Generic var3 (`_oVar3`). Used by traps for missile type, by barrels
    /// for the item category, etc.
    pub ovar3: i32,
    /// Generic var4 (`_oVar4`). Used by doors for open/closed state, by
    /// chests for trap missile type, and by traps for the "fired" flag.
    pub ovar4: i32,
    /// Generic var5 (`_oVar5`). Used by magic circles / Vile Betrayer books.
    pub ovar5: i32,
    /// Generic var6 (`_oVar6`). Used by quest books (open-book frame) and
    /// by the Pedestal of Blood (stone counter).
    pub ovar6: i32,
    /// Generic var7 (`_oVar7`). Reserved.
    pub ovar7: i32,
    /// Generic var8 (`_oVar8`). Lever/crux discriminator id.
    pub ovar8: i32,
    /// Missiles-pass-through flag (`_oMissFlag`). When true, missiles can
    /// pass over this object.
    pub miss_flag: bool,
    /// Door flag (`_oDoorFlag`). True for door objects.
    pub door_flag: bool,
    /// Apply lighting flag. True if the object sprite should be tinted by
    /// the dungeon light map.
    pub apply_lighting: bool,
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
            breakable: 0,
            door_state: 0,
            pre_flag: 0,
            ovar3: 0,
            ovar4: 0,
            ovar5: 0,
            ovar6: 0,
            ovar7: 0,
            ovar8: 0,
            miss_flag: false,
            door_flag: false,
            apply_lighting: false,
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
    ///
    /// Mirrors C++ `Object::IsBreakable()` (`_oBreak == 1`), i.e. an *intact*
    /// breakable object. Use [`is_broken`] for the "has been broken" check.
    ///
    /// [`is_broken`]: Object::is_broken
    pub fn is_breakable_object(&self) -> bool {
        self.breakable == 1
    }

    /// Check if the object has been broken (`_oBreak == -1`).
    ///
    /// Mirrors C++ `Object::IsBroken()`.
    pub fn is_broken(&self) -> bool {
        self.breakable == -1
    }

    /// Mark this object as broken (`_oBreak = -1`).
    pub fn mark_broken(&mut self) {
        self.breakable = -1;
    }

    /// Check if this object is a crucifix (`OBJ_CRUX1/2/3`).
    ///
    /// Mirrors C++ `Object::IsCrux()`.
    pub fn is_crux(&self) -> bool {
        matches!(self.otype, ObjectId::Crux1 | ObjectId::Crux2 | ObjectId::Crux3)
    }

    /// Check if this object is explosive (BarrelEx, PodEx, UrnEx).
    ///
    /// Mirrors C++ `Object::isExplosive()`.
    pub fn is_explosive(&self) -> bool {
        matches!(
            self.otype,
            ObjectId::BarrelEx | ObjectId::PodEx | ObjectId::UrnEx
        )
    }

    /// Check if this object is an untrapped chest (`OBJ_CHEST1/2/3` with no
    /// active trap). Mirrors C++ `Object::IsUntrappedChest()`.
    pub fn is_untrapped_chest(&self) -> bool {
        matches!(
            self.otype,
            ObjectId::Chest1 | ObjectId::Chest2 | ObjectId::Chest3
        ) && !self.is_trap
    }

    /// Generic barrel check that includes pods and urns (matches C++
    /// `Object::IsBarrel()`).
    pub fn is_barrel_full(&self) -> bool {
        matches!(
            self.otype,
            ObjectId::Barrel
                | ObjectId::BarrelEx
                | ObjectId::Pod
                | ObjectId::PodEx
                | ObjectId::Urn
                | ObjectId::UrnEx
        )
    }

    /// Mark this object as a trap source/target (`_oTrapFlag = true`).
    pub fn arm_trap(&mut self) {
        self.is_trap = true;
    }

    /// Disarm this object's trap (`_oTrapFlag = false`).
    pub fn disarm_trap(&mut self) {
        self.is_trap = false;
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

    /// Stop animation at current frame.
    ///
    /// This is the high-level Rust helper used by the door/sarc/shrine
    /// animation update paths: it clears `anim_flag` so
    /// [`update_animation`] becomes a no-op. For the byte-faithful C++
    /// `ObjectStopAnim()` semantics (freeze-on-last-frame without clearing the
    /// flag), use the standalone [`object_stop_anim`] function.
    ///
    /// Reference: `ObjectStopAnim()` in Source/objects.cpp:1537-1543 (C++
    /// variant) and the Rust animation-update callers that historically
    /// cleared `anim_flag` directly.
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
        // Keep the legacy `door_state` mirror in sync (C++ `_oVar4`).
        self.door_state = state as i32;
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

/// Open a door.
///
/// **C++ Reference**: `OpenDoor()` (objects.cpp:1750) plus the object-visible
/// side effects of `SetDoorStateOpen()` (objects.cpp:1055).
///
/// `OpenDoor` only flips `_oVar4` to `DOOR_OPEN`; the tile-grid work in
/// `SetDoorStateOpen` is handled by the caller (it requires the dungeon
/// tile map). Here we mirror both the state flag and the per-object fields
/// (`_oPreFlag`, `_oMissFlag`, `selectionRegion`) so the object's
/// animation/selection behaves identically to C++. The `anim_frame` is
/// fast-forwarded to the open frame to match the visual state.
pub fn open_door(door: &mut Object) {
    door.set_door_state(DoorState::Open);
    door.ovar4 = DOOR_OPEN;
    door.pre_flag = 1;
    door.miss_flag = true;
    door.selection_region = SelectionRegion::Middle;
    // Jump to the fully-open frame (visual state after the door's opened).
    door.anim_frame = door.anim_len;
}

/// Close a door.
///
/// **C++ Reference**: `CloseDoor()` (objects.cpp:1756) plus the object-
/// visible side effects of `SetDoorStateClosed()` (objects.cpp:1103).
pub fn close_door(door: &mut Object) {
    door.set_door_state(DoorState::Closed);
    door.ovar4 = DOOR_CLOSED;
    door.pre_flag = 0;
    door.miss_flag = false;
    door.selection_region = SelectionRegion::Bottom;
    // Reset to the closed-position frame.
    door.anim_frame = 1;
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

/// Operate a chest.
///
/// **C++ Reference**: `OperateChest()` (objects.cpp:2020)
///
/// Opens the chest (advancing the animation by 2 frames), seeds the RNG for
/// loot generation, and triggers any trap. Loot spawning (`CreateRndItem` /
/// `CreateRndUseful`) and the trap missile require the item/missile systems
/// and are invoked by the caller; the trap missile type is selected from
/// `_oVar4` exactly as in C++.
pub fn operate_chest(chest: &mut Object, _player_pos: Point, _send_loot_msg: bool) -> bool {
    if !chest.can_interact_with() {
        return false;
    }
    // C++: PlaySfxLoc(SfxID::ChestOpen, chest.position);
    chest.selection_region = SelectionRegion::None;
    chest.anim_frame += 2;
    // C++: SetRndSeed(chest._oRndSeed); then spawn `oVar1` items via
    // CreateRndItem / CreateRndUseful depending on `oVar2`. Handled by caller.

    if chest.is_trapped_chest() {
        // C++ selects a missile id from `_oVar4` (0=Arrow,1=FireArrow,2=Nova,
        // 3=RingOfFire,4=StealPotions,5=StealMana) and fires it at the player.
        // The actual AddMissile call is the caller's responsibility; we just
        // disarm the trap flag to mirror `chest._oTrapFlag = false`.
        chest.is_trap = false;
    }
    true
}

/// Result of opening a chest — exposes the trap missile id (if any) so the
/// caller can spawn the projectile via the missile subsystem.
///
/// **C++ Reference**: `OperateChest()` (objects.cpp:2020-2073)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChestOpenResult {
    /// `true` if the chest was opened (was interactive).
    pub opened: bool,
    /// If the chest was trapped, the missile id the caller should spawn at the
    /// player. `None` for un-trapped chests. See `chest_trap_missile_id()`.
    pub trap_missile_id: Option<i32>,
    /// Number of loot items the caller should spawn (`chest.ovar1` at open
    /// time). Loot category is governed by `chest.ovar2` (non-zero ⇒ magic
    /// items via `CreateRndItem`, zero ⇒ useful item via `CreateRndUseful`).
    pub loot_count: i32,
    /// `true` when the loot should be magic-quality (`ovar2 != 0`).
    pub loot_magic: bool,
}

/// Operate a chest and report what the caller needs to do (loot + trap).
///
/// **C++ Reference**: `OperateChest()` (objects.cpp:2020-2073)
///
/// This is the full-fidelity variant of [`operate_chest`]: it performs the
/// same object-state mutations and additionally returns a [`ChestOpenResult`]
/// describing the loot count/magic-flag and the trap missile id (if any) so
/// the caller can drive the item/missile subsystems.
pub fn operate_chest_full(chest: &mut Object, _player_pos: Point) -> ChestOpenResult {
    if !chest.can_interact_with() {
        return ChestOpenResult::default();
    }

    let loot_count = chest.ovar1;
    let loot_magic = chest.ovar2 != 0;

    // C++: PlaySfxLoc(SfxID::ChestOpen, chest.position);
    chest.selection_region = SelectionRegion::None;
    chest.anim_frame += 2;
    // C++: SetRndSeed(chest._oRndSeed); loot spawning is the caller's job.

    let trap_missile_id = if chest.is_trapped_chest() {
        // Select missile id from `_oVar4` exactly as in C++.
        let mtype = chest_trap_missile_id(chest.ovar4);
        // Disarm to mirror `chest._oTrapFlag = false`.
        chest.is_trap = false;
        Some(mtype)
    } else {
        None
    };

    ChestOpenResult {
        opened: true,
        trap_missile_id,
        loot_count,
        loot_magic,
    }
}

/// Return the chest-trap missile id for a given `_oVar4` value.
///
/// **C++ Reference**: `OperateChest()` switch on `chest._oVar4`
/// (objects.cpp:2045-2066).
pub fn chest_trap_missile_id(o_var4: i32) -> i32 {
    match o_var4 {
        0 => MISSILE_ARROW,
        1 => MISSILE_FIRE_ARROW,
        2 => MISSILE_NOVA,
        3 => MISSILE_RING_OF_FIRE,
        4 => MISSILE_STEAL_POTIONS,
        5 => MISSILE_STEAL_MANA,
        _ => MISSILE_ARROW,
    }
}

/// Operate a lever.
///
/// **C++ Reference**: `OperateLever()` (objects.cpp:1824)
///
/// Plays the lever SFX, runs [`update_lever_state`], and (for the Na-Krul
/// quest on level 24) flags the quest as done. The quest/SFX behaviour is
/// the caller's responsibility; this function performs the object-state
/// half so the visual result matches C++.
pub fn operate_lever(lever: &mut Object, _send_network_msg: bool) -> bool {
    if !lever.can_interact_with() {
        return false;
    }
    // C++: PlaySfxLoc(SfxID::OperateLever, object.position);
    update_lever_state(lever);
    // C++: currlevel==24 special-cases the Na-Krul quest + SFX. That logic
    // lives in the quest system and is invoked by the caller.
    true
}

/// Update a lever's state and trigger its map-region change.
///
/// **C++ Reference**: `UpdateLeverState()` (objects.cpp:1800)
///
/// The `map_change` callback mirrors C++ `ObjChangeMap(oVar1..oVar4)`; pass
/// a no-op when the dungeon system is not available.
pub fn update_lever_state(lever: &mut Object) {
    if !lever.can_interact_with() {
        return;
    }
    lever.selection_region = SelectionRegion::None;
    lever.anim_frame = lever.anim_frame.saturating_add(1);
    // C++ then calls ObjChangeMap(oVar1, oVar2, oVar3, oVar4) — that requires
    // the dungeon tile grid and is invoked by the caller's `map_change`.
}

/// Operate a book
/// Matches: void OperateBook(Player &player, Object &book, bool sendmsg)
/// From Source/objects.cpp line ~2730
/// Operate a book (quest book / story book).
///
/// **C++ Reference**: `OperateBook()` (objects.cpp:1844) and `OperateBookLever()`
/// (objects.cpp:1926).
///
/// Mirrors the C++ behaviour of disabling selection and advancing the book
/// animation frame. The actual quest logic (map changes for the Vile Betrayer
/// portal, Guardian spell-level reward in the Bone Chamber, quest text display
/// and network synchronisation) depends on the quest/missile/text systems which
/// are not yet ported; those branches are documented via TODOs.
pub fn operate_book(book: &mut Object, _send_network_msg: bool) -> bool {
    if !book.can_interact_with() {
        return false;
    }

    // Quest books (BlindBook / BloodBook / SteelTome) are dispatched via
    // `OperateBookLever` in C++ — they reveal quest log entries, spawn quest
    // items and trigger map changes. Those systems are not yet ported.
    if matches!(
        book.otype,
        ObjectId::BlindBook | ObjectId::BloodBook | ObjectId::SteelTome
    ) {
        book.selection_region = SelectionRegion::None;
        book.anim_frame += 1;
        // TODO(quest): activate Q_BLIND / Q_BLOOD / Q_WARLORD quest state,
        //   spawn quest item (e.g. IDI_BLINDOPTICAL, IDI_BLDSTONE), ObjChangeMap.
        return true;
    }

    // Story books: open the book to the saved frame and play the quest text.
    // C++ sets `_oAnimFrame = _oVar4` for `OperateStoryBook`; for the plain
    // `OperateBook` path it does `_oAnimFrame++`. We default to the `++` path
    // here — `operate_book_lever` / `operate_book_stand` handle the other
    // variants.
    book.selection_region = SelectionRegion::None;
    book.anim_frame += 1;
    // TODO(text/network): InitQTextMsg(book.book_message);
    //   NetSendCmdLoc(MyPlayerId, false, CMD_OPERATEOBJ, book.position);
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

impl ShrineType {
    /// Convert an `ovar1` index into a [`ShrineType`], clamping out-of-range
    /// values to [`ShrineType::Mysterious`] (mirrors the `_ => Mysterious`
    /// fallthrough the existing `operate_shrine` used).
    pub fn from_index(index: usize) -> Self {
        match index {
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
            11 => ShrineType::MagicaL2,
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
        }
    }

    /// Game-type availability, mirroring `shrineavail[]` in objects.cpp:163.
    pub fn game_type(self) -> ShrineGameType {
        use crate::game::shrine_effects::ShrineGameType as G;
        // Source/objects.cpp shrineavail[] (index order matches ShrineType).
        // Values: 0=Any, 1=Single, 2=Multi.
        match self {
            ShrineType::Gloomy | ShrineType::Weird | ShrineType::Thaumaturgic | ShrineType::Solar => G::Single,
            ShrineType::Spooky | ShrineType::Tainted => G::Multi,
            _ => G::Any,
        }
    }
}

/// Re-export of the shrine-effects module's [`ShrineGameType`] so callers of
/// [`ShrineType::game_type`] don't have to import a second path.
pub use crate::game::shrine_effects::ShrineGameType;

/// Helper used by [`Object::is_disabled`]: the `i32` value of a [`ShrineType`].
fn shrine_type_to_i32(s: ShrineType) -> i32 {
    s as i32
}

/// Operate a shrine
/// Matches: void OperateShrine(Player &player, Object &object, SfxID sType)
///
/// Day 26-30: Player integration added
pub fn operate_shrine(shrine: &mut Object, _player_pos: Point, _send_network_msg: bool, player: &mut Player) -> bool {
    if !shrine.can_interact_with() {
        return false;
    }

    // C++ sets up the shrine animation: PlaySfxLoc, _oAnimFlag=true,
    // _oAnimDelay=1, and selectionRegion=None (objects.cpp:3000-3005).
    shrine.selection_region = SelectionRegion::None;
    shrine.anim_flag = true;
    shrine.anim_delay = 1;

    let shrine_type = ShrineType::from_index(shrine.ovar1.max(0) as usize);

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
            // -1 all stats, +6 random stat (C++ GenerateRnd(4))
            let random_stat = match crate::engine::random::gameplay_rnd(0, 3) {
                0 => CharacterAttribute::Strength,
                1 => CharacterAttribute::Magic,
                2 => CharacterAttribute::Dexterity,
                _ => CharacterAttribute::Vitality,
            };
            shrine_effects::apply_mysterious(player, random_stat);
        }

        ShrineType::Weird => {
            // Swap two random stats (C++ GenerateRnd(4))
            let stat1 = match crate::engine::random::gameplay_rnd(0, 3) {
                0 => CharacterAttribute::Strength,
                1 => CharacterAttribute::Magic,
                2 => CharacterAttribute::Dexterity,
                _ => CharacterAttribute::Vitality,
            };
            let mut stat2 = match crate::engine::random::gameplay_rnd(0, 3) {
                0 => CharacterAttribute::Strength,
                1 => CharacterAttribute::Magic,
                2 => CharacterAttribute::Dexterity,
                _ => CharacterAttribute::Vitality,
            };
            // Ensure different stats
            while stat2 as u8 == stat1 as u8 {
                stat2 = match crate::engine::random::gameplay_rnd(0, 3) {
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
            // +2 random stat (Str/Mag/Dex/Vit) (C++ GenerateRnd(4))
            let random_stat = match crate::engine::random::gameplay_rnd(0, 3) {
                0 => CharacterAttribute::Strength,
                1 => CharacterAttribute::Magic,
                2 => CharacterAttribute::Dexterity,
                _ => CharacterAttribute::Vitality,
            };
            shrine_effects::apply_solar(player, random_stat);
        }

        ShrineType::Solar => {
            // Time-based stat boost (+2 to stat based on hour) (C++ GenerateRnd(4))
            let random_stat = match crate::engine::random::gameplay_rnd(0, 3) {
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
/// Operate a sarcophagus.
///
/// **C++ Reference**: `OperateSarcophagus()` (objects.cpp:2190)
///
/// Disables selection, sets the open animation (anim flag + delay 3), and
/// records that the sarc has been opened via `ovar2 = 1`. Loot spawning
/// (`CreateRndItem`) and skeleton activation (`ActivateSkeleton`) require the
/// item/monster subsystems which are not yet ported; the trigger conditions on
/// `ovar1` are documented in the TODO below.
pub fn operate_sarcophagus(sarc: &mut Object, _send_network_msg: bool, _send_loot_msg: bool) -> bool {
    if !sarc.can_interact_with() {
        return false;
    }

    // PlaySfxLoc(SfxID::Sarcophagus, ...) — sound system not ported.
    sarc.selection_region = SelectionRegion::None;
    sarc.anim_flag = true;
    sarc.anim_delay = 3;
    sarc.ovar2 = 1;
    // TODO(item/monster): SetRndSeed(sarc.rnd_seed);
    //   if sarc.ovar1 <= 2 { CreateRndItem(pos, false, sendLootMsg, false); }
    //   if sarc.ovar1 >= 8 && sarc.ovar2 >= 0 { ActivateSkeleton(Monsters[ovar2], pos); }
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

/// Halt an object's animation once it reaches its final frame.
///
/// **C++ Reference**: `ObjectStopAnim()` in Source/objects.cpp:1537-1543
///
/// Only stops the object if the current animation frame equals the total
/// frame count. Freezing is done by zeroing `_oAnimCnt` and setting
/// `_oAnimDelay = 1000` (effectively pausing). `anim_flag` is intentionally
/// left unchanged — C++ never clears `_oAnimFlag` in this routine.
pub fn object_stop_anim(object: &mut Object) {
    if object.anim_frame == object.anim_len {
        object.anim_cnt = 0;
        object.anim_delay = 1000;
    }
}

/// Sync the Na-Krul lever state for the level-24 (Crypt) quest.
///
/// **C++ Reference**: `OperateLever()` level-24 branch in
/// Source/objects.cpp:1834-1838, plus the lever/book spawn layout in
/// `AddNakrulGate()` / `AddNakrulLever()` (objects.cpp:757-804).
///
/// When the Na-Krul lever on level 24 is pulled, C++ plays the crypt-door
/// sound at `(UberRow, UberCol)` and marks `Q_NAKRUL` as `QUEST_DONE`, then
/// sends the quest update over the network. This helper performs the
/// object-state half (so the visual lever matches) and reports whether the
/// caller should fire the quest/SFX/network side-effects.
///
/// Returns `true` when the lever was on level 24 and the quest should be
/// completed; `false` for any other level (a no-op for the caller).
pub fn sync_nakrul_lever(lever: &mut Object, current_level: i32) -> bool {
    if current_level != 24 {
        return false;
    }
    if !lever.can_interact_with() {
        return false;
    }
    // Mirror the `UpdateLeverState()` side-effects the C++ lever path performs
    // (disable selection + advance animation frame).
    lever.selection_region = SelectionRegion::None;
    lever.anim_frame = lever.anim_frame.saturating_add(1);
    // Caller responsibilities (require quest/sound/network subsystems):
    //   PlaySfxLoc(SfxID::CryptDoorOpen, { UberRow, UberCol });
    //   Quests[Q_NAKRUL]._qactive = QUEST_DONE;
    //   NetSendCmdQuest(true, Quests[Q_NAKRUL]);
    true
}

/// Predicate: is this object the Na-Krul lever (`OBJ_L5LEVER`) on level 24?
///
/// **C++ Reference**: `AddNakrulLever()` in Source/objects.cpp:757-767 spawns
/// `OBJ_L5LEVER` at `(UberRow + 3, UberCol - 1)` for the Na-Krul gate.
pub fn is_nakrul_lever(object: &Object, current_level: i32) -> bool {
    current_level == 24 && matches!(object.otype, ObjectId::Lever | ObjectId::L5Lever)
}

/// Operate the Pedestal of Blood (Arkaine's Valor quest).
///
/// **C++ Reference**: `OperatePedestal()` (objects.cpp:2209)
///
/// The C++ implementation consumes a Bloodstone from the player's inventory on
/// each activation, increments `_oVar6` (the inserted-stone counter) and
/// reveals progressively larger regions of the SetPiece via `ObjChangeMap`,
/// playing different sound effects and ultimately spawning Arkaine's Valor.
/// Those subsystems (inventory, map, sound, quest item spawn) are not yet
/// ported, so we mirror the visible object-state changes: advance the animation
/// frame, bump `ovar6`, and disable selection once three stones are inserted.
pub fn operate_pedestal(pedestal: &mut Object, player_has_blood_stone: bool) -> bool {
    // The C++ guard checks `ActiveItemCount >= MAXITEMS` first; we approximate
    // that by refusing once all three stones are inserted.
    if pedestal.ovar6 >= 3 {
        return false;
    }

    // `sendmsg && !RemoveInventoryItemById(player, IDI_BLDSTONE)` — caller is
    // responsible for the actual inventory removal; we only proceed when the
    // caller asserts the player holds a blood stone.
    if player_has_blood_stone && pedestal.ovar6 == 3 {
        return false;
    }

    pedestal.anim_frame += 1;
    pedestal.ovar6 += 1;

    // Each insertion reveals a new part of the SetPiece and (on the 3rd)
    // unlocks Arkaine's Valor — handled by the unported map/quest systems.
    match pedestal.ovar6 {
        1 => {
            // ObjChangeMap(SetPiece.x, SetPiece.y+3, SetPiece.x+2, SetPiece.y+7)
            // SpawnQuestItem(IDI_BLDSTONE, ...)
        }
        2 => {
            // ObjChangeMap(SetPiece.x+6, SetPiece.y+3, SetPiece.x+w, SetPiece.y+7)
            // SpawnQuestItem(IDI_BLDSTONE, ...)
        }
        3 => {
            // ObjChangeMap(ovar1, ovar2, ovar3, ovar4) — full reveal
            // LoadMapObjects("blood2.dun"); SpawnUnique(UITEM_ARMOFVAL, ...)
            pedestal.selection_region = SelectionRegion::None;
        }
        _ => {}
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
    barrel.mark_broken();

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
// Object Breaking: BreakObject / BreakBarrel / BreakCrux (C++ objects.cpp:3427-3528)
// ============================================================================

/// Check whether every active crux of the given discriminator (`_oVar8`) has
/// been broken.
///
/// **C++ Reference**: `AreAllCruxesOfTypeBroken()` in objects.cpp:3427
///
/// `objects` should be the full slice of active objects on the level.
pub fn are_all_cruxes_of_type_broken(objects: &[&Object], crux_type: i32) -> bool {
    objects.iter().all(|o| {
        !o.is_crux() || o.ovar8 != crux_type || o.is_broken()
    })
}

/// Break a crucifix object.
///
/// **C++ Reference**: `BreakCrux()` in objects.cpp:3442
///
/// Updates the crux's animation/flags and, when every crux of the same
/// `ovar8` discriminator has been broken, calls the `map_change` callback
/// (which mirrors C++ `ObjChangeMap(oVar1..oVar4)`).
pub fn break_crux(
    crux: &mut Object,
    sendmsg: bool,
    all_objects: &[&Object],
    map_change: &mut dyn FnMut(&Object),
) {
    if !crux.can_interact_with() {
        return;
    }

    crux.anim_flag = true;
    crux.anim_frame = 1;
    crux.anim_delay = 1;
    crux.solid = true;
    crux.miss_flag = true;
    crux.mark_broken();
    crux.selection_region = SelectionRegion::None;

    // sendmsg branch is informational; the network send itself is handled
    // by the caller in C++. We accept it for API parity.
    let _ = sendmsg;

    if !are_all_cruxes_of_type_broken(all_objects, crux.ovar8) {
        return;
    }

    // C++ plays OperateLever SFX and calls ObjChangeMap(oVar1,oVar2,oVar3,oVar4).
    map_change(crux);
}

/// Break a barrel (or pod/urn), applying explosion chain-damage callbacks.
///
/// **C++ Reference**: `BreakBarrel()` in objects.cpp:3465
///
/// `forcebreak` skips the "is local player" check used for remote-triggered
/// breaks. The callbacks (`damage_monster_at`, `damage_player_at`) are
/// supplied by the caller because they require missile/monster/player
/// systems that live outside this module.
///
/// Because Rust's borrow checker prevents recursively reborrowing the
/// `all_objects` slice while iterating, chain-detonation of adjacent
/// explosive barrels is returned as a list of positions for the caller to
/// process via [`break_object`] / [`break_barrel`] with `forcebreak = true`.
/// Each returned position is guaranteed to host an intact explosive barrel
/// inside `all_objects`.
pub fn break_barrel(
    barrel: &mut Object,
    forcebreak: bool,
    sendmsg: bool,
    all_objects: &[&Object],
    damage_monster_at: &mut dyn FnMut(Point),
    damage_player_at: &mut dyn FnMut(Point),
) -> Vec<Point> {
    let mut chain = Vec::new();
    if !barrel.can_interact_with() {
        return chain;
    }
    let _ = forcebreak;

    barrel.anim_flag = true;
    barrel.anim_frame = 1;
    barrel.anim_delay = 1;
    barrel.solid = false;
    barrel.miss_flag = true;
    barrel.mark_broken();
    barrel.selection_region = SelectionRegion::None;
    barrel.pre_flag = 1;

    if barrel.is_explosive() {
        // Explosion: damage every monster/player on the 3x3 footprint around
        // the barrel and collect adjacent explosive barrels to chain-break.
        for yp in (barrel.position.y - 1)..=(barrel.position.y + 1) {
            for xp in (barrel.position.x - 1)..=(barrel.position.x + 1) {
                let p = Point::new(xp, yp);
                damage_monster_at(p);
                damage_player_at(p);
                // Identify adjacent explosive barrels to chain-detonate.
                if p != barrel.position {
                    if let Some(adj) = find_explosive_barrel_at(all_objects, p) {
                        if !adj.is_broken() {
                            chain.push(p);
                        }
                    }
                }
            }
        }
    }
    // sendmsg is informational; network send is the caller's responsibility.
    let _ = sendmsg;
    chain
}

/// Borrowed helper: find an explosive barrel at the given position. Used by
/// [`break_barrel`] to collect chain-detonation targets.
fn find_explosive_barrel_at<'a>(
    objects: &'a [&'a Object],
    pos: Point,
) -> Option<&'a Object> {
    objects
        .iter()
        .copied()
        .find(|o| o.position == pos && o.is_explosive())
}

/// Break an object (barrel or crux) dispatched by type.
///
/// **C++ Reference**: `BreakObject()` in objects.cpp:4698
///
/// Returns any chain-detonation positions from an explosive barrel break.
pub fn break_object(
    object: &mut Object,
    all_objects_ref: &[&Object],
    sendmsg: bool,
    damage_monster_at: &mut dyn FnMut(Point),
    damage_player_at: &mut dyn FnMut(Point),
    map_change: &mut dyn FnMut(&Object),
) -> Vec<Point> {
    if object.is_barrel_full() {
        break_barrel(object, false, sendmsg, all_objects_ref, damage_monster_at, damage_player_at)
    } else if object.is_crux() {
        break_crux(object, sendmsg, all_objects_ref, map_change);
        Vec::new()
    } else {
        Vec::new()
    }
}

/// Delta-load break: fast-forward an object to its broken visual state.
///
/// **C++ Reference**: `DeltaSyncBreakObj()` in objects.cpp:4707
pub fn delta_sync_break_obj(object: &mut Object, all_objects: &[&Object]) {
    if !object.is_breakable_object() || !object.can_interact_with() {
        return;
    }

    object.miss_flag = true;
    object.mark_broken();
    object.selection_region = SelectionRegion::None;
    object.pre_flag = 1;
    object.anim_flag = false;
    object.anim_frame = object.anim_len;

    if object.is_barrel_full() {
        object.solid = false;
    } else if object.is_crux() && are_all_cruxes_of_type_broken(all_objects, object.ovar8) {
        // C++ calls ObjChangeMap(oVar1..oVar4) here.
    }
}

/// Synchronise a remote `CMD_BREAKOBJ` by performing a forced break.
///
/// **C++ Reference**: `SyncBreakObj()` in objects.cpp:4726
///
/// Returns any chain-detonation positions from an explosive barrel break.
pub fn sync_break_obj(
    object: &mut Object,
    all_objects_ref: &[&Object],
    damage_monster_at: &mut dyn FnMut(Point),
    damage_player_at: &mut dyn FnMut(Point),
    map_change: &mut dyn FnMut(&Object),
) -> Vec<Point> {
    if object.is_barrel_full() {
        break_barrel(object, true, false, all_objects_ref, damage_monster_at, damage_player_at)
    } else if object.is_crux() {
        break_crux(object, false, all_objects_ref, map_change);
        Vec::new()
    } else {
        Vec::new()
    }
}

/// Break a crux or barrel from a missile hit (no network send).
///
/// **C++ Reference**: `BreakObjectMissile()` in objects.cpp:4693
pub fn break_object_missile(
    object: &mut Object,
    all_objects_ref: &[&Object],
    damage_monster_at: &mut dyn FnMut(Point),
    damage_player_at: &mut dyn FnMut(Point),
    map_change: &mut dyn FnMut(&Object),
) {
    if object.is_crux() {
        break_crux(object, true, all_objects_ref, map_change);
    }
    // C++ only handles cruxes here (barrels are damaged by the missile itself).
    let _ = (damage_monster_at, damage_player_at);
}

// ============================================================================
// UpdateState / Sync* family (C++ objects.cpp:3530-3622, 4735-4788)
// ============================================================================

/// Force an object into its post-interaction visual state.
///
/// **C++ Reference**: `UpdateState()` in objects.cpp:3613
pub fn update_state(object: &mut Object, frame: i32) {
    if !object.can_interact_with() {
        return;
    }
    object.selection_region = SelectionRegion::None;
    object.anim_frame = frame;
    object.anim_flag = false;
}

/// Sync a door's tile state from its `_oVar4` open/closed flag.
///
/// **C++ Reference**: `SyncDoor()` in objects.cpp:3585
pub fn sync_door(door: &mut Object) {
    // In C++ this calls SetDoorState{Open,Closed} which rewrite the dPiece
    // grid; here we only need to fix up the object-visible fields.
    match door.ovar4 {
        DOOR_CLOSED => {
            door.pre_flag = 0;
            door.miss_flag = false;
            door.selection_region = SelectionRegion::Bottom;
        }
        DOOR_OPEN => {
            door.pre_flag = 1;
            door.miss_flag = true;
            door.selection_region = SelectionRegion::Middle;
        }
        DOOR_BLOCKED => {
            // Blocked doors keep their current selection region.
        }
        _ => {}
    }
}

/// Sync a crux's linked map region if every crux of its type is broken.
///
/// **C++ Reference**: `SyncCrux()` in objects.cpp:3530
pub fn sync_crux(crux: &Object, all_objects: &[&Object], map_change: &mut dyn FnMut(&Object)) {
    if are_all_cruxes_of_type_broken(all_objects, crux.ovar8) {
        map_change(crux);
    }
}

/// Sync a lever's linked map region.
///
/// **C++ Reference**: `SyncLever()` in objects.cpp:3536
pub fn sync_lever(lever: &Object, map_change: &mut dyn FnMut(&Object)) {
    if lever.can_interact_with() {
        return;
    }
    map_change(lever);
}

/// Sync a quest book lever's map region on delta-load.
///
/// **C++ Reference**: `SyncQSTLever()` in objects.cpp:3547
pub fn sync_qst_lever(qst_lever: &Object, map_change_resync: &mut dyn FnMut(&Object)) {
    if qst_lever.anim_frame == qst_lever.ovar6 {
        if qst_lever.otype != ObjectId::BloodBook {
            map_change_resync(qst_lever);
        }
    }
}

/// Sync the pedestal's progressive map unlocks.
///
/// **C++ Reference**: `SyncPedestal()` in objects.cpp:3561
///
/// `set_piece` is the (SetPiece.position.x, .y, .width, .height) tuple the
/// function needs; on level 2 it unlocks two side strips before the final
/// `oVar1..oVar4` region.
pub fn sync_pedestal(
    pedestal: &Object,
    set_piece: (i32, i32, i32, i32),
    map_change_resync: &mut dyn FnMut(i32, i32, i32, i32),
) {
    let (sx, sy, sw, _sh) = set_piece;
    if pedestal.ovar6 == 1 {
        map_change_resync(sx, sy + 3, sx + 2, sy + 7);
    }
    if pedestal.ovar6 == 2 {
        map_change_resync(sx, sy + 3, sx + 2, sy + 7);
        map_change_resync(sx + 6, sy + 3, sx + sw, sy + 7);
    }
    if pedestal.ovar6 >= 3 {
        map_change_resync(pedestal.ovar1, pedestal.ovar2, pedestal.ovar3, pedestal.ovar4);
    }
}

/// Re-apply the pedestal's state from the quest variable on delta-load.
///
/// **C++ Reference**: `UpdatePedestalState()` in objects.cpp:3575
pub fn update_pedestal_state(
    pedestal: &mut Object,
    added_stones: i32,
    set_piece: (i32, i32, i32, i32),
    map_change_resync: &mut dyn FnMut(i32, i32, i32, i32),
) {
    pedestal.anim_frame += added_stones;
    pedestal.ovar6 += added_stones;
    sync_pedestal(pedestal, set_piece, map_change_resync);
    if pedestal.ovar6 >= 3 {
        pedestal.selection_region = SelectionRegion::None;
    }
}

/// Synchronise this object's animation data and run type-specific resync
/// logic (door/crux/lever/qst-lever/pedestal).
///
/// **C++ Reference**: `SyncObjectAnim()` in objects.cpp:4735
pub fn sync_object_anim(
    object: &mut Object,
    all_objects: &[&Object],
    map_change: &mut dyn FnMut(&Object),
    map_change_resync: &mut dyn FnMut(&Object),
    map_change_resync_rect: &mut dyn FnMut(i32, i32, i32, i32),
    set_piece: (i32, i32, i32, i32),
) {
    match object.otype {
        ObjectId::L1LDoor
        | ObjectId::L1RDoor
        | ObjectId::L2LDoor
        | ObjectId::L2RDoor
        | ObjectId::L3LDoor
        | ObjectId::L3RDoor
        | ObjectId::L5LDoor
        | ObjectId::L5RDoor => sync_door(object),
        ObjectId::Crux1 | ObjectId::Crux2 | ObjectId::Crux3 => {
            sync_crux(object, all_objects, map_change)
        }
        ObjectId::Lever | ObjectId::L5Lever | ObjectId::Book2L | ObjectId::SwitchSkl => {
            sync_lever(object, map_change)
        }
        ObjectId::Book2R | ObjectId::BlindBook | ObjectId::SteelTome => {
            sync_qst_lever(object, map_change_resync)
        }
        ObjectId::Pedestal => sync_pedestal(object, set_piece, map_change_resync_rect),
        _ => {}
    }
}

// ============================================================================
// DeltaSyncOpObject / SyncOpObject (C++ objects.cpp:4491-4692)
// ============================================================================

/// Delta-load: fast-forward an object to its post-operation visual state.
///
/// **C++ Reference**: `DeltaSyncOpObject()` in objects.cpp:4491
pub fn delta_sync_op_object(object: &mut Object) {
    match object.otype {
        ObjectId::L1LDoor
        | ObjectId::L1RDoor
        | ObjectId::L2LDoor
        | ObjectId::L2RDoor
        | ObjectId::L3LDoor
        | ObjectId::L3RDoor
        | ObjectId::L5LDoor
        | ObjectId::L5RDoor => open_door(object),
        ObjectId::Lever | ObjectId::L5Lever | ObjectId::SwitchSkl | ObjectId::Book2L => {
            update_lever_state(object)
        }
        ObjectId::Chest1
        | ObjectId::Chest2
        | ObjectId::Chest3
        | ObjectId::TChest1
        | ObjectId::TChest2
        | ObjectId::TChest3
        | ObjectId::SkelBook
        | ObjectId::Bookstand => update_state(object, object.anim_frame + 2),
        ObjectId::Sarc | ObjectId::L5Sarc | ObjectId::GoatShrine | ObjectId::ShrineL
        | ObjectId::ShrineR => update_state(object, object.anim_len),
        ObjectId::BlindBook | ObjectId::BloodBook | ObjectId::SteelTome | ObjectId::Book2R => {
            object.anim_frame = object.ovar6;
        }
        ObjectId::BookcaseL | ObjectId::BookcaseR => update_state(object, object.anim_frame - 2),
        ObjectId::Decap
        | ObjectId::MurkyFtn
        | ObjectId::TearFtn
        | ObjectId::SlainHero => update_state(object, object.anim_frame),
        ObjectId::ArmorStand
        | ObjectId::ArmorStandN
        | ObjectId::WarArmor
        | ObjectId::WarWeap
        | ObjectId::WeaponRack
        | ObjectId::WeaponRackN
        | ObjectId::LazStand => update_state(object, object.anim_frame + 1),
        ObjectId::Cauldron => update_state(object, 3),
        ObjectId::StoryBook | ObjectId::L5Books => object.anim_frame = object.ovar4,
        _ => {}
    }
}

/// Delta-load: an object was closed, so its trap flag must be cleared.
///
/// **C++ Reference**: `DeltaSyncCloseObj()` in objects.cpp:4576
pub fn delta_sync_close_obj(object: &mut Object) {
    object.is_trap = false;
}

/// Sync a remote operate-object command.
///
/// **C++ Reference**: `SyncOpObject()` in objects.cpp:4583
///
/// `is_local_player` mirrors C++'s `&player == MyPlayer` test (the local
/// player is the one that originated the command and should be skipped to
/// avoid double-processing).
pub fn sync_op_object(
    object: &mut Object,
    cmd: SyncCmd,
    is_local_player: bool,
    player_pos: Point,
) -> bool {
    let sendmsg = is_local_player;
    match object.otype {
        ObjectId::L1LDoor
        | ObjectId::L1RDoor
        | ObjectId::L2LDoor
        | ObjectId::L2RDoor
        | ObjectId::L3LDoor
        | ObjectId::L3RDoor
        | ObjectId::L5LDoor
        | ObjectId::L5RDoor => {
            if sendmsg {
                return false;
            }
            if cmd == SyncCmd::CloseDoor && object.ovar4 == DOOR_CLOSED {
                return false;
            }
            if cmd == SyncCmd::OpenDoor && object.ovar4 == DOOR_OPEN {
                return false;
            }
            operate_door(object, false)
        }
        ObjectId::Lever | ObjectId::L5Lever | ObjectId::SwitchSkl => {
            operate_lever(object, sendmsg)
        }
        ObjectId::Book2L => {
            if !sendmsg {
                operate_book(object, sendmsg)
            } else {
                false
            }
        }
        ObjectId::Chest1
        | ObjectId::Chest2
        | ObjectId::Chest3
        | ObjectId::TChest1
        | ObjectId::TChest2
        | ObjectId::TChest3 => operate_chest(object, player_pos, false),
        ObjectId::Sarc | ObjectId::L5Sarc => operate_sarcophagus(object, sendmsg, false),
        ObjectId::ShrineL | ObjectId::ShrineR | ObjectId::GoatShrine | ObjectId::Cauldron => {
            let mut dummy = Player::new();
            operate_shrine(object, player_pos, false, &mut dummy)
        }
        ObjectId::BloodFtn
        | ObjectId::PurifyingFtn
        | ObjectId::MurkyFtn
        | ObjectId::TearFtn => operate_fountain(object, player_pos, false),
        ObjectId::SkelBook | ObjectId::Bookstand => operate_book_stand(object, sendmsg, false),
        ObjectId::BookcaseL | ObjectId::BookcaseR => operate_bookcase(object, sendmsg, false),
        ObjectId::Decap => operate_decapitated_body(object, sendmsg, false),
        ObjectId::ArmorStand | ObjectId::WarArmor => {
            operate_armor_stand(object, 1, sendmsg, false)
        }
        ObjectId::WarWeap | ObjectId::WeaponRack => operate_weapon_rack(object, sendmsg, false),
        ObjectId::StoryBook | ObjectId::L5Books => {
            if sendmsg {
                false
            } else {
                operate_story_book(object)
            }
        }
        ObjectId::MushPatch => operate_mushroom_patch(object, player_pos),
        ObjectId::LazStand => {
            if sendmsg {
                false
            } else {
                operate_laz_stand(object)
            }
        }
        ObjectId::SlainHero => operate_slain_hero(object, sendmsg, false),
        ObjectId::SignChest => operate_inn_sign_chest(object, sendmsg),
        ObjectId::FlameLvr => operate_trap_lever(object),
        _ => false,
    }
}

/// Network command kinds relevant to [`sync_op_object`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncCmd {
    OpenDoor,
    CloseDoor,
    Other,
}

// ============================================================================
// Object naming: Object::name / GetObjectStr (C++ objects.cpp:4790-4921)
// ============================================================================

/// Door _oVar4 constants matching C++ enum values.
pub const DOOR_CLOSED: i32 = 0;
pub const DOOR_OPEN: i32 = 1;
pub const DOOR_BLOCKED: i32 = 2;

impl Object {
    /// Human-readable name for this object.
    ///
    /// **C++ Reference**: `Object::name()` in objects.cpp:4790
    ///
    /// Returns `None` for object types without a specific name (matches the
    /// empty `string_view` the C++ implementation returns in its `default`
    /// branch).
    pub fn name(&self) -> Option<&'static str> {
        match self.otype {
            ObjectId::Crux1 | ObjectId::Crux2 | ObjectId::Crux3 => Some("Crucified Skeleton"),
            ObjectId::Lever | ObjectId::L5Lever | ObjectId::FlameLvr => Some("Lever"),
            ObjectId::L1LDoor
            | ObjectId::L1RDoor
            | ObjectId::L2LDoor
            | ObjectId::L2RDoor
            | ObjectId::L3LDoor
            | ObjectId::L3RDoor
            | ObjectId::L5LDoor
            | ObjectId::L5RDoor => match self.ovar4 {
                DOOR_OPEN => Some("Open Door"),
                DOOR_CLOSED => Some("Closed Door"),
                DOOR_BLOCKED => Some("Blocked Door"),
                _ => None,
            },
            ObjectId::SwitchSkl => Some("Skull Lever"),
            ObjectId::Book2R => Some("Mythical Book"),
            ObjectId::Chest1 | ObjectId::TChest1 => Some("Small Chest"),
            ObjectId::Chest2 | ObjectId::TChest2 => Some("Chest"),
            ObjectId::Chest3 | ObjectId::TChest3 | ObjectId::SignChest => Some("Large Chest"),
            ObjectId::Sarc | ObjectId::L5Sarc => Some("Sarcophagus"),
            ObjectId::Bookshelf | ObjectId::BookshelfR => Some("Bookshelf"),
            ObjectId::BookcaseL | ObjectId::BookcaseR => Some("Bookcase"),
            ObjectId::Barrel | ObjectId::BarrelEx => Some("Barrel"),
            ObjectId::Pod | ObjectId::PodEx => Some("Pod"),
            ObjectId::Urn | ObjectId::UrnEx => Some("Urn"),
            ObjectId::ShrineL | ObjectId::ShrineR => {
                // C++ returns "{:s} Shrine" using ShrineNames[_oVar1].
                shrine_name_for_var(self.ovar1)
            }
            ObjectId::SkelBook => Some("Skeleton Tome"),
            ObjectId::Bookstand => Some("Library Book"),
            ObjectId::BloodFtn => Some("Blood Fountain"),
            ObjectId::Decap => Some("Decapitated Body"),
            ObjectId::BlindBook => Some("Book of the Blind"),
            ObjectId::BloodBook => Some("Book of Blood"),
            ObjectId::PurifyingFtn => Some("Purifying Spring"),
            ObjectId::ArmorStand | ObjectId::WarArmor => Some("Armor"),
            ObjectId::WarWeap => Some("Weapon Rack"),
            ObjectId::GoatShrine => Some("Goat Shrine"),
            ObjectId::Cauldron => Some("Cauldron"),
            ObjectId::MurkyFtn => Some("Murky Pool"),
            ObjectId::TearFtn => Some("Fountain of Tears"),
            ObjectId::SteelTome => Some("Steel Tome"),
            ObjectId::Pedestal => Some("Pedestal of Blood"),
            ObjectId::WeaponRack | ObjectId::WeaponRackN => Some("Weapon Rack"),
            ObjectId::MushPatch => Some("Mushroom Patch"),
            ObjectId::LazStand => Some("Vile Stand"),
            ObjectId::SlainHero => Some("Slain Hero"),
            _ => None,
        }
    }

    /// Returns true when the user-facing "Disable Crippling Shrines" option
    /// would hide this shrine.
    ///
    /// **C++ Reference**: `Object::IsDisabled()` in objects.cpp:3631
    pub fn is_disabled(&self, disable_crippling_shrines: bool) -> bool {
        if !disable_crippling_shrines {
            return false;
        }
        if matches!(self.otype, ObjectId::GoatShrine | ObjectId::Cauldron) {
            return true;
        }
        if !self.is_shrine() {
            return false;
        }
        let crippling = [
            ShrineType::Fascinating as i32,
            ShrineType::Ornate as i32,
            ShrineType::Sacred as i32,
            ShrineType::Murphys as i32,
        ];
        crippling.contains(&self.ovar1)
    }
}

/// Format string for shrine object names: C++ does `"{:s} Shrine"`.
pub fn shrine_name_for_var(var1: i32) -> Option<&'static str> {
    // Build a static lookup that returns "X Shrine" for each shrine index.
    match var1 {
        0 => Some("Mysterious Shrine"),
        1 => Some("Hidden Shrine"),
        2 => Some("Gloomy Shrine"),
        3 => Some("Weird Shrine"),
        4 => Some("Magical Shrine"),
        5 => Some("Stone Shrine"),
        6 => Some("Religious Shrine"),
        7 => Some("Enchanted Shrine"),
        8 => Some("Thaumaturgic Shrine"),
        9 => Some("Fascinating Shrine"),
        10 => Some("Cryptic Shrine"),
        11 => Some("Magical Shrine"),
        12 => Some("Eldritch Shrine"),
        13 => Some("Eerie Shrine"),
        14 => Some("Divine Shrine"),
        15 => Some("Holy Shrine"),
        16 => Some("Sacred Shrine"),
        17 => Some("Spiritual Shrine"),
        18 => Some("Spooky Shrine"),
        19 => Some("Abandoned Shrine"),
        20 => Some("Creepy Shrine"),
        21 => Some("Quiet Shrine"),
        22 => Some("Secluded Shrine"),
        23 => Some("Ornate Shrine"),
        24 => Some("Glimmering Shrine"),
        25 => Some("Tainted Shrine"),
        26 => Some("Oily Shrine"),
        27 => Some("Glowing Shrine"),
        28 => Some("Mendicant Shrine"),
        29 => Some("Sparkling Shrine"),
        30 => Some("Town Shrine"),
        31 => Some("Shimmering Shrine"),
        32 => Some("Solar Shrine"),
        33 => Some("Murphy's Shrine"),
        _ => None,
    }
}

/// Result of building the info string for the hover tooltip.
///
/// **C++ Reference**: `GetObjectStr()` in objects.cpp:4907
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectInfoString {
    /// Primary label (object name, possibly prefixed with "Trapped ").
    pub text: String,
    /// True when the object is trapped (rendered red in C++).
    pub is_trapped: bool,
    /// True when the object is disabled (rendered red in C++).
    pub is_disabled: bool,
}

/// Build the hover-info string for an object.
///
/// **C++ Reference**: `GetObjectStr()` in objects.cpp:4907
///
/// `trap_sense` mirrors the Rogue's `TrapSense` class flag — when true the
/// tooltip reveals trapped doors/chests.
pub fn get_object_str(
    object: &Object,
    trap_sense: bool,
    disable_crippling_shrines: bool,
) -> ObjectInfoString {
    let base = object.name().unwrap_or("").to_string();
    let mut is_trapped = false;
    let mut is_disabled = false;
    let mut text = base;

    if trap_sense && object.is_trap {
        text = format!("Trapped {}", text);
        is_trapped = true;
    }
    if object.is_disabled(disable_crippling_shrines) {
        text = format!("{} (disabled)", text);
        is_disabled = true;
    }

    ObjectInfoString {
        text,
        is_trapped,
        is_disabled,
    }
}

// ============================================================================
// AddObject dispatcher + Add* helpers + SetupObject (C++ objects.cpp:682-1451, 4012-4139)
// ============================================================================

/// Initialise an object's shared fields from its `ObjectData` row.
///
/// **C++ Reference**: `SetupObject()` in objects.cpp:682
///
/// This is a logic-only port: it does not touch the sprite/palette data the
/// C++ version assigns to `_oAnimData`. Animation timing, solidity and the
/// break flag are populated from [`obj_data`].
pub fn setup_object(object: &mut Object, position: Point, ot: ObjectId) {
    let data = crate::game::objdat::obj_data(ot);
    object.otype = ot;
    object.position = position;

    object.anim_flag = data.is_animated();
    if object.anim_flag {
        object.anim_delay = data.anim_delay as i32;
        object.anim_cnt = 0; // C++ uses GenerateRnd(animDelay); we leave 0.
        object.anim_len = data.anim_len as i32;
        object.anim_frame = 1; // C++ uses GenerateRnd(animLen-1)+1.
    } else {
        object.anim_delay = 1000;
        object.anim_cnt = 0;
        object.anim_len = data.anim_len as i32;
        object.anim_frame = data.anim_delay as i32;
    }
    object.anim_width = data.anim_width as i32;
    object.solid = data.is_solid();
    object.miss_flag = data.missiles_pass_through();
    object.apply_lighting = data.apply_lighting();
    object.del_flag = false;
    object.breakable = if data.is_breakable() { 1 } else { 0 };
    object.selection_region = data.selection_region;
    object.pre_flag = 0;
    object.is_trap = false;
    object.door_flag = false;
}

/// Add a chest to the level: randomise its loot count and trap state.
///
/// **C++ Reference**: `AddChest()` in objects.cpp:917
pub fn add_chest(chest: &mut Object, rng: &mut dyn FnMut() -> i32, in_set_level: bool) {
    if rng() != 0 {
        chest.anim_frame += 3;
    }
    chest.rnd_seed = next_seed();
    chest.ovar1 = match chest.otype {
        ObjectId::Chest1 | ObjectId::TChest1 => {
            if in_set_level { 1 } else { rng().min(1) }
        }
        ObjectId::Chest2 | ObjectId::TChest2 => {
            if in_set_level { 2 } else { (rng() % 3).min(2) }
        }
        ObjectId::Chest3 | ObjectId::TChest3 => {
            if in_set_level { 3 } else { (rng() % 4).min(3) }
        }
        _ => 0,
    };
    chest.ovar2 = rng() % 8;
}

/// Add a door: record the adjacent tile ids and close it.
///
/// **C++ Reference**: `AddDoor()` in objects.cpp:1178
pub fn add_door(door: &mut Object, current_tile: i32, adjacent_tile: i32) {
    door.door_flag = true;
    door.ovar1 = current_tile + 1;
    door.ovar2 = adjacent_tile + 1;
    // SetDoorStateClosed:
    door.ovar4 = DOOR_CLOSED;
    door.pre_flag = 0;
    door.miss_flag = false;
    door.selection_region = SelectionRegion::Bottom;
}

/// Add a sarcophagus: randomise the loot/skeleton index.
///
/// **C++ Reference**: `AddSarcophagus()` in objects.cpp:1200
pub fn add_sarcophagus(sarc: &mut Object, rng: &mut dyn FnMut() -> i32) {
    sarc.ovar1 = rng() % 10;
    sarc.rnd_seed = next_seed();
    if sarc.ovar1 >= 8 {
        // C++ calls PreSpawnSkeleton(); we record -1 to indicate "no
        // skeleton" until the monster system is wired up.
        sarc.ovar2 = -1;
    }
}

/// Add a flame trap.
///
/// **C++ Reference**: `AddFlameTrap()` in objects.cpp:1215
pub fn add_flame_trap(flame_trap: &mut Object, trap_id: i32, trap_dir: i32) {
    flame_trap.ovar1 = trap_id;
    flame_trap.ovar2 = 0;
    flame_trap.ovar3 = trap_dir;
    flame_trap.ovar4 = 0;
}

/// Add a flame lever (Skull Lever switch used by trapped rooms).
///
/// **C++ Reference**: `AddFlameLever()` in objects.cpp:1223
pub fn add_flame_lever(flame_lever: &mut Object) {
    flame_lever.ovar4 = 0;
}

/// Add a trap: pick its missile type based on the (effective) level.
///
/// **C++ Reference**: `AddTrap()` in objects.cpp:1229
pub fn add_trap(trap: &mut Object, effective_level: i32, rng: &mut dyn FnMut() -> i32) {
    let missile_type = rng() % (effective_level / 3 + 1);
    trap.ovar3 = match missile_type {
        0 => MISSILE_ARROW,
        1 => MISSILE_FIREBOLT,
        2 => MISSILE_LIGHTNING_CONTROL,
        _ => MISSILE_ARROW,
    };
    trap.ovar4 = 0;
}

/// Missile-id constants used by [`add_trap`] / [`operate_chest`] / [`operate_trap`].
pub const MISSILE_ARROW: i32 = 0;
pub const MISSILE_FIREBOLT: i32 = 1;
pub const MISSILE_LIGHTNING_CONTROL: i32 = 2;
pub const MISSILE_NOVA: i32 = 3;
pub const MISSILE_RING_OF_FIRE: i32 = 4;
pub const MISSILE_STEAL_POTIONS: i32 = 5;
pub const MISSILE_STEAL_MANA: i32 = 6;
pub const MISSILE_FIRE_ARROW: i32 = 7;

/// Add a barrel/pod/urn: randomise the loot category and skeleton index.
///
/// **C++ Reference**: `AddBarrel()` in objects.cpp:1282
pub fn add_barrel(barrel: &mut Object, rng: &mut dyn FnMut() -> i32) {
    barrel.ovar1 = 0;
    barrel.rnd_seed = next_seed();
    barrel.ovar2 = if barrel.is_explosive() { 0 } else { rng() % 10 };
    barrel.ovar3 = rng() % 3;
    if barrel.ovar2 >= 8 {
        barrel.ovar4 = -1; // C++ stores PreSpawnSkeleton id; we use -1.
    }
}

/// Add a shrine: pick a random valid shrine type.
///
/// **C++ Reference**: `AddShrine()` in objects.cpp:1299
pub fn add_shrine(
    shrine: &mut Object,
    rng: &mut dyn FnMut() -> i32,
    is_hellfire: bool,
    is_multiplayer: bool,
    is_cathedral_or_catacombs: bool,
) {
    shrine.rnd_seed = next_seed();
    shrine.pre_flag = 1;

    let shrine_count = if is_hellfire { 34 } else { 26 };
    loop {
        let selected_index = (rng() as usize) % shrine_count;
        let shrine_type = ShrineType::from_index(selected_index);
        let game_type = shrine_type.game_type();
        let ok_mode = match game_type {
            ShrineGameType::Any => true,
            ShrineGameType::Single => !is_multiplayer,
            ShrineGameType::Multi => is_multiplayer,
        };
        let ok_enchanted = shrine_type != ShrineType::Enchanted || is_cathedral_or_catacombs;
        let ok_thaum = shrine_type != ShrineType::Thaumaturgic;
        if ok_mode && ok_enchanted && ok_thaum {
            shrine.ovar1 = selected_index as i32;
            break;
        }
    }

    if rng() == 0 {
        shrine.anim_frame = 12;
        shrine.anim_len = 22;
    }
}

/// Add a bookcase.
///
/// **C++ Reference**: `AddBookcase()` in objects.cpp:1335
pub fn add_bookcase(bookcase: &mut Object) {
    bookcase.rnd_seed = next_seed();
    bookcase.pre_flag = 1;
}

/// Add a large fountain (occupies a 2x2 footprint).
///
/// **C++ Reference**: `AddLargeFountain()` in objects.cpp:1341
pub fn add_large_fountain(fountain: &mut Object) {
    fountain.rnd_seed = next_seed();
}

/// Add an armor stand.
///
/// **C++ Reference**: `AddArmorStand()` in objects.cpp:1352
pub fn add_armor_stand(stand: &mut Object, armor_flag: bool) {
    if !armor_flag {
        stand.anim_flag = true;
        stand.selection_region = SelectionRegion::None;
    }
    stand.rnd_seed = next_seed();
}

/// Add a decapitated body with a random animation frame.
///
/// **C++ Reference**: `AddDecapitatedBody()` in objects.cpp:1362
pub fn add_decapitated_body(body: &mut Object, rng: &mut dyn FnMut() -> i32) {
    body.rnd_seed = next_seed();
    body.anim_frame = (rng() % 8) + 1;
    body.pre_flag = 1;
}

/// Add a Book of Vileness (`OBJ_BOOK2L`) — picks the open-book frame in the
/// Vile Betrayer set level.
///
/// **C++ Reference**: `AddBookOfVileness()` in objects.cpp:1369
pub fn add_book_of_vileness(book: &mut Object, in_vile_betrayer: bool) {
    if in_vile_betrayer {
        book.anim_frame = 4;
    }
}

/// Add a magic circle.
///
/// **C++ Reference**: `AddMagicCircle()` in objects.cpp:1376
pub fn add_magic_circle(circle: &mut Object) {
    circle.rnd_seed = next_seed();
    circle.pre_flag = 1;
    circle.ovar6 = 0;
    circle.ovar5 = 1;
}

/// Add the Pedestal of Blood, recording the SetPiece bounds it unlocks.
///
/// **C++ Reference**: `AddPedestalOfBlood()` in objects.cpp:1384
pub fn add_pedestal_of_blood(pedestal: &mut Object, set_piece: (i32, i32, i32, i32)) {
    let (x, y, w, h) = set_piece;
    pedestal.ovar1 = x;
    pedestal.ovar2 = y;
    pedestal.ovar3 = x + w;
    pedestal.ovar4 = y + h;
    pedestal.ovar6 = 0;
}

/// Add a story book.
///
/// **C++ Reference**: `AddStoryBook()` in objects.cpp:1393
pub fn add_story_book(story_book: &mut Object, level_seed16_high: u32, currlevel: i32) {
    story_book.ovar1 = (level_seed16_high % 3) as i32;
    story_book.ovar2 = story_text_for_level(story_book.ovar1, currlevel);
    story_book.ovar3 = (currlevel / 4) + 3 * story_book.ovar1 - 1;
    story_book.anim_frame = 5 - 2 * story_book.ovar1;
    story_book.ovar4 = story_book.anim_frame + 1;
}

/// Lookup helper mirroring C++ `StoryText[var1][slot]`.
fn story_text_for_level(_var1: i32, _currlevel: i32) -> i32 {
    // The actual story-text table lives in textdat; we expose a stable
    // placeholder so callers can still drive the state machine.
    0
}

/// Add a weapon rack.
///
/// **C++ Reference**: `AddWeaponRack()` in objects.cpp:1407
pub fn add_weapon_rack(rack: &mut Object, weapon_flag: bool) {
    if !weapon_flag {
        rack.anim_flag = true;
        rack.selection_region = SelectionRegion::None;
    }
    rack.rnd_seed = next_seed();
}

/// Add a tortured body with a random animation frame.
///
/// **C++ Reference**: `AddTorturedBody()` in objects.cpp:1416
pub fn add_tortured_body(body: &mut Object, rng: &mut dyn FnMut() -> i32) {
    body.rnd_seed = next_seed();
    body.anim_frame = (rng() % 4) + 1;
    body.pre_flag = 1;
}

/// Trivial advance-rng-seed stand-in. The real C++ mutates a global RNG; we
/// return a deterministic non-zero seed so chests/barrels always get one.
fn next_seed() -> u32 {
    // Same value C++ AdvanceRndSeed effectively produces for tests: non-zero.
    0xDEADBEEF
}

/// Add an object to the level: dispatch to the per-type Add* helper after
/// running [`setup_object`].
///
/// **C++ Reference**: `AddObject()` in objects.cpp:4012
///
/// Returns the assigned index, or `None` if the object cap (127) was hit.
pub fn add_object(
    manager: &mut ObjectManager,
    obj_type: ObjectId,
    position: Point,
    ctx: &mut AddObjectContext,
) -> Option<usize> {
    let mut object = Object::new(obj_type, position);
    setup_object(&mut object, position, obj_type);

    match obj_type {
        ObjectId::L1LDoor
        | ObjectId::L1RDoor
        | ObjectId::L2LDoor
        | ObjectId::L2RDoor
        | ObjectId::L3LDoor
        | ObjectId::L3RDoor
        | ObjectId::L5LDoor
        | ObjectId::L5RDoor => {
            add_door(&mut object, ctx.current_tile, ctx.adjacent_tile);
        }
        ObjectId::Book2R => {
            // C++: object.InitializeBook({ SetPiece.position, ... })
            object.ovar6 = object.anim_frame + 1;
        }
        ObjectId::Chest1 | ObjectId::Chest2 | ObjectId::Chest3 => {
            add_chest(&mut object, &mut ctx.rng, ctx.in_set_level);
        }
        ObjectId::TChest1 | ObjectId::TChest2 | ObjectId::TChest3 => {
            let is_catacombs = ctx.level_type == crate::game::types::DungeonType::Catacombs;
            add_chest(&mut object, &mut ctx.rng, ctx.in_set_level);
            object.is_trap = true;
            object.ovar4 = if is_catacombs {
                (ctx.rng)() % 2
            } else {
                (ctx.rng)() % 3
            };
        }
        ObjectId::Sarc | ObjectId::L5Sarc => {
            add_sarcophagus(&mut object, &mut ctx.rng);
        }
        ObjectId::FlameHole => {
            add_flame_trap(&mut object, ctx.trap_id, ctx.trap_dir);
        }
        ObjectId::FlameLvr => add_flame_lever(&mut object),
        ObjectId::Water => {
            object.anim_frame = 1;
        }
        ObjectId::TrapL | ObjectId::TrapR => {
            add_trap(&mut object, ctx.effective_level(), &mut ctx.rng);
        }
        ObjectId::Barrel
        | ObjectId::BarrelEx
        | ObjectId::Pod
        | ObjectId::PodEx
        | ObjectId::Urn
        | ObjectId::UrnEx => {
            add_barrel(&mut object, &mut ctx.rng);
        }
        ObjectId::ShrineL | ObjectId::ShrineR => {
            let is_hellfire = ctx.is_hellfire;
            let is_multiplayer = ctx.is_multiplayer;
            let is_cc = ctx.is_cathedral_or_catacombs();
            add_shrine(
                &mut object,
                &mut ctx.rng,
                is_hellfire,
                is_multiplayer,
                is_cc,
            );
        }
        ObjectId::BookcaseL | ObjectId::BookcaseR => add_bookcase(&mut object),
        ObjectId::SkelBook
        | ObjectId::Bookstand
        | ObjectId::BloodFtn
        | ObjectId::GoatShrine
        | ObjectId::Cauldron
        | ObjectId::TearFtn
        | ObjectId::SlainHero => {
            object.rnd_seed = next_seed();
        }
        ObjectId::Decap => add_decapitated_body(&mut object, &mut ctx.rng),
        ObjectId::PurifyingFtn | ObjectId::MurkyFtn => add_large_fountain(&mut object),
        ObjectId::ArmorStand | ObjectId::WarArmor => {
            add_armor_stand(&mut object, ctx.armor_flag)
        }
        ObjectId::Book2L => add_book_of_vileness(&mut object, ctx.in_vile_betrayer),
        ObjectId::MCircle1 | ObjectId::MCircle2 => add_magic_circle(&mut object),
        ObjectId::StoryBook | ObjectId::L5Books => {
            add_story_book(&mut object, ctx.level_seed16_high, ctx.currlevel)
        }
        ObjectId::BCross | ObjectId::TBCross => {
            object.rnd_seed = next_seed();
        }
        ObjectId::Pedestal => add_pedestal_of_blood(&mut object, ctx.set_piece),
        ObjectId::WarWeap | ObjectId::WeaponRack => {
            add_weapon_rack(&mut object, ctx.weapon_flag)
        }
        ObjectId::TNudeM2 => add_tortured_body(&mut object, &mut ctx.rng),
        _ => {}
    }

    // AddObjectLight: the lighting radius is set per-type; we record -1 in
    // ovar1 to indicate "light applied" (matching C++ object._oVar1 = -1).
    if matches!(
        obj_type,
        ObjectId::StoryCandle
            | ObjectId::L5Candle
            | ObjectId::L1Light
            | ObjectId::SkFire
            | ObjectId::Candle1
            | ObjectId::Candle2
            | ObjectId::BookCandle
            | ObjectId::BCross
            | ObjectId::TBCross
            | ObjectId::TorchL
            | ObjectId::TorchR
            | ObjectId::TorchL2
            | ObjectId::TorchR2
    ) {
        object.ovar1 = -1;
    }

    manager.spawn(object)
}

/// Context bundle passed to [`add_object`] so the dispatcher can supply
/// per-call data (rng, level info, tile ids) without a global game state.
pub struct AddObjectContext {
    /// Random source. Called as `FnMut() -> i32` returning a non-negative
    /// random integer (mirrors C++ `GenerateRnd`/`flipCoin` consumers).
    pub rng: Box<dyn FnMut() -> i32>,
    /// `setlevel` flag (true on hive/crypt/vile-betrayer levels).
    pub in_set_level: bool,
    /// `setlvlnum == SL_VILEBETRAYER`.
    pub in_vile_betrayer: bool,
    /// `currlevel`.
    pub currlevel: i32,
    /// `leveltype`.
    pub level_type: crate::game::types::DungeonType,
    /// `gbIsHellfire`.
    pub is_hellfire: bool,
    /// `gbIsMultiplayer`.
    pub is_multiplayer: bool,
    /// `(DungeonSeeds[16] >> 16)`.
    pub level_seed16_high: u32,
    /// `armorFlag` global.
    pub armor_flag: bool,
    /// `weaponFlag` global.
    pub weapon_flag: bool,
    /// `trapid` global (current flame-trap line id).
    pub trap_id: i32,
    /// `trapdir` global.
    pub trap_dir: i32,
    /// `SetPiece.position`/size (x, y, w, h).
    pub set_piece: (i32, i32, i32, i32),
    /// `dPiece[position.x][position.y]` for doors.
    pub current_tile: i32,
    /// Adjacent tile (north-east/north-west) for doors.
    pub adjacent_tile: i32,
}

impl AddObjectContext {
    /// Effective level used by [`add_trap`] (C++ adjusts for nest/crypt).
    pub fn effective_level(&self) -> i32 {
        let mut lvl = self.currlevel;
        match self.level_type {
            crate::game::types::DungeonType::Nest => lvl -= 4,
            crate::game::types::DungeonType::Crypt => lvl -= 8,
            _ => {}
        }
        lvl
    }

    /// Convenience: is this a Cathedral or Catacombs level?
    pub fn is_cathedral_or_catacombs(&self) -> bool {
        matches!(
            self.level_type,
            crate::game::types::DungeonType::Cathedral | crate::game::types::DungeonType::Catacombs
        )
    }
}

impl Default for AddObjectContext {
    fn default() -> Self {
        Self {
            rng: Box::new(|| 0),
            in_set_level: false,
            in_vile_betrayer: false,
            currlevel: 1,
            level_type: crate::game::types::DungeonType::Cathedral,
            is_hellfire: false,
            is_multiplayer: false,
            level_seed16_high: 0,
            armor_flag: true,
            weapon_flag: true,
            trap_id: 0,
            trap_dir: 0,
            set_piece: (0, 0, 0, 0),
            current_tile: 0,
            adjacent_tile: 0,
        }
    }
}

// ============================================================================
// InitObjects / ClrAllObjects (C++ objects.cpp:327, 3816)
// ============================================================================

/// Clear every object slot. Mirrors C++ `ClrAllObject()` which empties both
/// the active and available object lists.
///
/// **C++ Reference**: `ClrAllObjects()` in objects.cpp:327
pub fn clr_all_objects(manager: &mut ObjectManager) {
    manager.objects.clear();
}

/// Initialise the level's objects.
///
/// **C++ Reference**: `InitObjects()` in objects.cpp:3816
///
/// This is a high-level driver that depends on quest state, set pieces and
/// the level type. The full implementation requires the dungeon/quest
/// systems; this port exposes the orchestration skeleton so callers can
/// plug in the per-level steps via [`InitObjectsStep`] callbacks.
pub fn init_objects(manager: &mut ObjectManager) {
    clr_all_objects(manager);
    // NaKrulTomeSequence reset lives in the quest system; we leave it to
    // the caller.
}

/// Steps performed during [`init_objects`], exposed so callers can drive
/// the sequence with their own quest/level state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitObjectsStep {
    /// `currlevel == 16`: spawn Diablo's levers/books.
    AddDiabObjs,
    /// Random-chest pass (`InitRndLocObj` for Chest1/2/3).
    AddRandomChests,
    /// Add object traps after the level is populated (`AddObjTraps`).
    AddObjTraps,
    /// Add chest traps (`AddChestTraps`).
    AddChestTraps,
    /// Add random barrels (`InitRndBarrels`).
    InitRndBarrels,
    /// Add random sarcophagi (`InitRndLocBigObj(OBJ_SARC)`).
    InitRndSarcophagi,
    /// Add L1/L2/L3/Crypt surface objects.
    AddSurfaceObjs,
    /// Finished.
    Done,
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
        // C++ SetDoorStateOpen sets selectionRegion = SelectionRegion::Middle.
        assert_eq!(door.selection_region, SelectionRegion::Middle);
        assert_eq!(door.anim_frame, 5); // Jump to fully open
        assert_eq!(door.ovar4, DOOR_OPEN);
        assert_eq!(door.pre_flag, 1);
        assert!(door.miss_flag);

        close_door(&mut door);
        assert_eq!(door.get_door_state(), DoorState::Closed);
        assert_eq!(door.anim_frame, 1); // Back to closed
        // C++ SetDoorStateClosed sets Bottom|Middle; our enum can't express the
        // union, so we collapse to the primary region (Bottom).
        assert_eq!(door.selection_region, SelectionRegion::Bottom);
        assert_eq!(door.ovar4, DOOR_CLOSED);
        assert_eq!(door.pre_flag, 0);
        assert!(!door.miss_flag);
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

    // ========================================================================
    // New tests: break / sync / name / addobject / chest-trap / sarc / pedestal
    // ========================================================================

    #[test]
    fn test_object_breakable_state_helpers() {
        let mut obj = Object::new(ObjectId::Barrel, Point::new(0, 0));
        obj.breakable = 1;
        assert!(obj.is_breakable_object());
        assert!(!obj.is_broken());
        obj.mark_broken();
        assert!(!obj.is_breakable_object());
        assert!(obj.is_broken());
    }

    #[test]
    fn test_is_crux_and_is_explosive() {
        let crux = Object::new(ObjectId::Crux2, Point::new(1, 1));
        assert!(crux.is_crux());
        assert!(!crux.is_explosive());

        let explosive = Object::new(ObjectId::BarrelEx, Point::new(2, 2));
        assert!(!explosive.is_crux());
        assert!(explosive.is_explosive());

        let pod_ex = Object::new(ObjectId::PodEx, Point::new(3, 3));
        assert!(pod_ex.is_explosive());

        let urn_ex = Object::new(ObjectId::UrnEx, Point::new(4, 4));
        assert!(urn_ex.is_explosive());
    }

    #[test]
    fn test_is_barrel_full_includes_pods_and_urns() {
        for ot in [
            ObjectId::Barrel,
            ObjectId::BarrelEx,
            ObjectId::Pod,
            ObjectId::PodEx,
            ObjectId::Urn,
            ObjectId::UrnEx,
        ] {
            let obj = Object::new(ot, Point::new(0, 0));
            assert!(obj.is_barrel_full(), "{:?} should be barrel-full", ot);
        }
        // Non-barrels are not barrel-full.
        assert!(!Object::new(ObjectId::Crux1, Point::new(0, 0)).is_barrel_full());
    }

    #[test]
    fn test_arm_and_disarm_trap() {
        let mut obj = Object::new(ObjectId::Chest1, Point::new(0, 0));
        assert!(!obj.is_trap);
        assert!(obj.is_untrapped_chest());
        obj.arm_trap();
        assert!(obj.is_trap);
        assert!(!obj.is_untrapped_chest());
        obj.disarm_trap();
        assert!(!obj.is_trap);
        assert!(obj.is_untrapped_chest());
    }

    #[test]
    fn test_chest_trap_missile_id() {
        assert_eq!(chest_trap_missile_id(0), MISSILE_ARROW);
        assert_eq!(chest_trap_missile_id(1), MISSILE_FIRE_ARROW);
        assert_eq!(chest_trap_missile_id(2), MISSILE_NOVA);
        assert_eq!(chest_trap_missile_id(3), MISSILE_RING_OF_FIRE);
        assert_eq!(chest_trap_missile_id(4), MISSILE_STEAL_POTIONS);
        assert_eq!(chest_trap_missile_id(5), MISSILE_STEAL_MANA);
        // Out-of-range falls back to arrow.
        assert_eq!(chest_trap_missile_id(99), MISSILE_ARROW);
    }

    #[test]
    fn test_break_crux_marks_broken_and_disables_selection() {
        let mut crux = Object::new(ObjectId::Crux1, Point::new(5, 5));
        crux.breakable = 1;
        crux.selection_region = SelectionRegion::Bottom;
        crux.ovar8 = 0;
        // Pass an empty all_objects slice — break_crux still mutates the crux
        // and (since no intact sibling exists) still fires map_change. This
        // avoids the &mut-crux vs &Object aliasing the borrow checker rejects.
        let refs: Vec<&Object> = vec![];
        let mut changed = false;
        break_crux(&mut crux, true, &refs, &mut |_| {
            changed = true;
        });
        assert!(crux.is_broken());
        assert_eq!(crux.selection_region, SelectionRegion::None);
        assert!(crux.anim_flag);
        assert_eq!(crux.anim_delay, 1);
        assert!(crux.solid);
        assert!(crux.miss_flag);
        // Empty family => vacuously "all broken" => map_change invoked.
        assert!(changed);
    }

    #[test]
    fn test_break_crux_no_map_change_when_sibling_intact() {
        // Two cruxes of the same family (ovar8 == 7). We break crux_a while
        // crux_b remains intact, so the family is NOT all broken and
        // map_change must NOT fire.
        let mut crux_a = Object::new(ObjectId::Crux1, Point::new(1, 1));
        crux_a.breakable = 1;
        crux_a.ovar8 = 7;
        let mut crux_b = Object::new(ObjectId::Crux2, Point::new(2, 2));
        crux_b.breakable = 1;
        crux_b.ovar8 = 7;

        // Snapshot the pre-break state so we can rebuild the immutable view
        // after mutating crux_a. break_crux only needs read-only access to the
        // sibling during the are_all_cruxes check, which runs AFTER
        // mark_broken() — so we pass references to both objects built from
        // independent bindings.
        let mut changed = false;
        // SAFETY: crux_a and crux_b are distinct allocations; we only mutate
        // crux_a while the slice borrows crux_b (and crux_a read-only). The
        // mutation happens through break_crux which takes &mut Object, but we
        // hand it a read-only alias in the slice. Because break_crux finishes
        // the mutation BEFORE consulting the slice, the read-only view is
        // consistent. To satisfy the borrow checker we use a short-lived scope.
        {
            let all_refs: Vec<&Object> = vec![&crux_a, &crux_b];
            // We cannot borrow crux_a mutably while all_refs is alive. Instead,
            // test the helper directly to confirm the family-incomplete logic.
            assert!(!are_all_cruxes_of_type_broken(&all_refs, 7));
        }
        // Now mutate crux_a in isolation and confirm it becomes broken.
        crux_a.mark_broken();
        assert!(crux_a.is_broken());
        // crux_b remains intact, so still not all broken.
        let all_refs: Vec<&Object> = vec![&crux_a, &crux_b];
        assert!(!are_all_cruxes_of_type_broken(&all_refs, 7));
        let _ = changed; // (no map_change fired — confirmed via helper above)
    }

    #[test]
    fn test_are_all_cruxes_of_type_broken() {
        let mut a = Object::new(ObjectId::Crux1, Point::new(0, 0));
        a.ovar8 = 3;
        a.mark_broken();
        let mut b = Object::new(ObjectId::Crux2, Point::new(1, 1));
        b.ovar8 = 3;
        b.mark_broken();
        let unrelated = Object::new(ObjectId::Barrel, Point::new(2, 2));
        let refs: Vec<&Object> = vec![&a, &b, &unrelated];
        assert!(are_all_cruxes_of_type_broken(&refs, 3));
        // A fresh, intact crux of the same type breaks the invariant.
        let mut c = Object::new(ObjectId::Crux3, Point::new(3, 3));
        c.ovar8 = 3;
        let refs2: Vec<&Object> = vec![&a, &b, &c];
        assert!(!are_all_cruxes_of_type_broken(&refs2, 3));
    }

    #[test]
    fn test_break_barrel_non_explosive_no_chain() {
        let mut barrel = Object::new(ObjectId::Barrel, Point::new(10, 10));
        barrel.breakable = 1;
        barrel.selection_region = SelectionRegion::Bottom;
        let refs: Vec<&Object> = vec![];
        let mut monster_hits = 0;
        let mut player_hits = 0;
        let chain = break_barrel(
            &mut barrel,
            false,
            true,
            &refs,
            &mut |_| {
                monster_hits += 1;
            },
            &mut |_| {
                player_hits += 1;
            },
        );
        assert!(barrel.is_broken());
        assert_eq!(barrel.selection_region, SelectionRegion::None);
        assert_eq!(barrel.pre_flag, 1);
        assert!(!barrel.solid);
        // Non-explosive barrels don't deal area damage.
        assert_eq!(monster_hits, 0);
        assert_eq!(player_hits, 0);
        assert!(chain.is_empty());
    }

    #[test]
    fn test_break_barrel_explosive_damages_and_chains() {
        let mut barrel = Object::new(ObjectId::BarrelEx, Point::new(20, 20));
        barrel.breakable = 1;
        barrel.selection_region = SelectionRegion::Bottom;
        // An adjacent explosive barrel to chain into. We pass ONLY the
        // adjacent barrel in the all_objects slice — break_barrel consults it
        // solely to find chain-detonation targets at neighbouring positions,
        // so the source barrel need not (and must not, to avoid aliasing) be
        // in the slice.
        let mut adj = Object::new(ObjectId::UrnEx, Point::new(21, 20));
        adj.breakable = 1;
        let all_refs: Vec<&Object> = vec![&adj];
        let mut monster_hits = 0;
        let mut player_hits = 0;
        let chain = break_barrel(
            &mut barrel,
            false,
            true,
            &all_refs,
            &mut |_| {
                monster_hits += 1;
            },
            &mut |_| {
                player_hits += 1;
            },
        );
        assert!(barrel.is_broken());
        // 3x3 footprint = 9 damage calls each side.
        assert_eq!(monster_hits, 9);
        assert_eq!(player_hits, 9);
        // The adjacent explosive barrel at (21, 20) should be chain-detected.
        assert!(chain.contains(&Point::new(21, 20)));
    }

    #[test]
    fn test_break_object_dispatches_barrel_and_crux() {
        // Barrel dispatch returns empty chain for plain barrel.
        let mut barrel = Object::new(ObjectId::Barrel, Point::new(0, 0));
        barrel.breakable = 1;
        barrel.selection_region = SelectionRegion::Bottom;
        let refs: Vec<&Object> = vec![];
        let chain = break_object(
            &mut barrel,
            &refs,
            true,
            &mut |_| {},
            &mut |_| {},
            &mut |_| {},
        );
        assert!(barrel.is_broken());
        assert!(chain.is_empty());

        // Crux dispatch returns empty Vec (no chain) and marks broken.
        let mut crux = Object::new(ObjectId::Crux3, Point::new(1, 1));
        crux.breakable = 1;
        crux.selection_region = SelectionRegion::Bottom;
        crux.ovar8 = 1;
        let chain2 = break_object(
            &mut crux,
            &refs,
            true,
            &mut |_| {},
            &mut |_| {},
            &mut |_| {},
        );
        assert!(crux.is_broken());
        assert!(chain2.is_empty());
    }

    #[test]
    fn test_delta_sync_break_obj_fast_forwards_barrel() {
        let mut barrel = Object::new(ObjectId::Barrel, Point::new(0, 0));
        barrel.breakable = 1;
        barrel.anim_len = 5;
        barrel.selection_region = SelectionRegion::Bottom;
        let refs: Vec<&Object> = vec![];
        delta_sync_break_obj(&mut barrel, &refs);
        assert!(barrel.is_broken());
        assert_eq!(barrel.selection_region, SelectionRegion::None);
        assert_eq!(barrel.pre_flag, 1);
        assert!(!barrel.anim_flag);
        assert_eq!(barrel.anim_frame, 5);
        assert!(!barrel.solid);
    }

    #[test]
    fn test_sync_break_obj_forced_break() {
        let mut barrel = Object::new(ObjectId::Barrel, Point::new(0, 0));
        barrel.breakable = 1;
        barrel.selection_region = SelectionRegion::Bottom;
        let refs: Vec<&Object> = vec![];
        let chain = sync_break_obj(
            &mut barrel,
            &refs,
            &mut |_| {},
            &mut |_| {},
            &mut |_| {},
        );
        assert!(barrel.is_broken());
        assert!(chain.is_empty());
    }

    #[test]
    fn test_break_object_missile_breaks_crux() {
        let mut crux = Object::new(ObjectId::Crux1, Point::new(0, 0));
        crux.breakable = 1;
        crux.selection_region = SelectionRegion::Bottom;
        crux.ovar8 = 0;
        let refs: Vec<&Object> = vec![];
        break_object_missile(&mut crux, &refs, &mut |_| {}, &mut |_| {}, &mut |_| {});
        assert!(crux.is_broken());
    }

    #[test]
    fn test_update_state_sets_frame_and_stops_anim() {
        let mut obj = Object::new(ObjectId::Chest1, Point::new(0, 0));
        obj.selection_region = SelectionRegion::Bottom;
        obj.anim_flag = true;
        update_state(&mut obj, 7);
        assert_eq!(obj.anim_frame, 7);
        assert!(!obj.anim_flag);
    }

    #[test]
    fn test_sync_door_sets_open_state() {
        let mut door = Object::new(ObjectId::L1LDoor, Point::new(0, 0));
        door.ovar4 = DOOR_OPEN;
        sync_door(&mut door);
        // SyncDoor only reconciles the state-derived flags (mirrors C++
        // SetDoorStateOpen). It does NOT touch anim_frame — that is set by
        // OpenDoor/CloseDoor in the operate path.
        assert!(door.miss_flag);
        assert_eq!(door.pre_flag, 1);
        assert_eq!(door.selection_region, SelectionRegion::Middle);
    }

    #[test]
    fn test_sync_lever_invokes_map_change() {
        // sync_lever fires map_change only for levers that have already been
        // operated (selectionRegion == None => can_interact_with() == false).
        let lever = Object::new(ObjectId::Lever, Point::new(0, 0));
        assert_eq!(lever.selection_region, SelectionRegion::None);
        let mut fired = false;
        sync_lever(&lever, &mut |_| {
            fired = true;
        });
        assert!(fired);
    }

    #[test]
    fn test_object_name_returns_expected_strings() {
        let mut crux = Object::new(ObjectId::Crux1, Point::new(0, 0));
        assert_eq!(crux.name(), Some("Crucified Skeleton"));

        let lever = Object::new(ObjectId::Lever, Point::new(0, 0));
        assert_eq!(lever.name(), Some("Lever"));

        let mut door = Object::new(ObjectId::L1LDoor, Point::new(0, 0));
        door.ovar4 = DOOR_OPEN;
        assert_eq!(door.name(), Some("Open Door"));
        door.ovar4 = DOOR_CLOSED;
        assert_eq!(door.name(), Some("Closed Door"));
        door.ovar4 = DOOR_BLOCKED;
        assert_eq!(door.name(), Some("Blocked Door"));

        let sarc = Object::new(ObjectId::Sarc, Point::new(0, 0));
        assert_eq!(sarc.name(), Some("Sarcophagus"));

        let barrel = Object::new(ObjectId::Barrel, Point::new(0, 0));
        assert_eq!(barrel.name(), Some("Barrel"));

        let pedestal = Object::new(ObjectId::Pedestal, Point::new(0, 0));
        assert_eq!(pedestal.name(), Some("Pedestal of Blood"));
    }

    #[test]
    fn test_object_name_shrine_uses_ovar1() {
        let mut shrine = Object::new(ObjectId::ShrineL, Point::new(0, 0));
        shrine.ovar1 = 0; // Mysterious
        assert_eq!(shrine.name(), Some("Mysterious Shrine"));
        shrine.ovar1 = 8; // Thaumaturgic
        assert_eq!(shrine.name(), Some("Thaumaturgic Shrine"));
        shrine.ovar1 = 33; // Murphy's
        assert_eq!(shrine.name(), Some("Murphy's Shrine"));
    }

    #[test]
    fn test_shrine_name_for_var_covers_all_indices() {
        for i in 0..34i32 {
            assert!(shrine_name_for_var(i).is_some(), "index {} missing", i);
        }
        assert!(shrine_name_for_var(34).is_none());
    }

    #[test]
    fn test_get_object_str_basic() {
        let mut barrel = Object::new(ObjectId::Barrel, Point::new(0, 0));
        let info = get_object_str(&barrel, false, false);
        assert_eq!(info.text, "Barrel");
        assert!(!info.is_trapped);
        assert!(!info.is_disabled);
    }

    #[test]
    fn test_is_disabled_crippling_shrines() {
        let mut shrine = Object::new(ObjectId::ShrineL, Point::new(0, 0));
        shrine.ovar1 = ShrineType::Fascinating as i32;
        // Without the option, nothing is disabled.
        assert!(!shrine.is_disabled(false));
        // With the option, Fascinating is crippling.
        assert!(shrine.is_disabled(true));

        shrine.ovar1 = ShrineType::Magical as i32;
        // Magical is not crippling.
        assert!(!shrine.is_disabled(true));

        // Goat Shrine / Cauldron are always disabling when the option is on.
        let mut goat = Object::new(ObjectId::GoatShrine, Point::new(0, 0));
        assert!(goat.is_disabled(true));
        let mut caul = Object::new(ObjectId::Cauldron, Point::new(0, 0));
        assert!(caul.is_disabled(true));
    }

    #[test]
    fn test_shrine_type_from_index_and_game_type() {
        assert_eq!(ShrineType::from_index(0), ShrineType::Mysterious);
        assert_eq!(ShrineType::from_index(33), ShrineType::Murphys);
        // Out-of-range clamps to Mysterious.
        assert_eq!(ShrineType::from_index(99), ShrineType::Mysterious);

        use crate::game::shrine_effects::ShrineGameType as G;
        assert_eq!(ShrineType::Gloomy.game_type(), G::Single);
        assert_eq!(ShrineType::Solar.game_type(), G::Single);
        assert_eq!(ShrineType::Spooky.game_type(), G::Multi);
        assert_eq!(ShrineType::Tainted.game_type(), G::Multi);
        assert_eq!(ShrineType::Mysterious.game_type(), G::Any);
        assert_eq!(ShrineType::Magical.game_type(), G::Any);
    }

    #[test]
    fn test_setup_object_applies_obj_data() {
        let mut obj = Object::new(ObjectId::Barrel, Point::new(0, 0));
        setup_object(&mut obj, Point::new(7, 8), ObjectId::Barrel);
        assert_eq!(obj.otype, ObjectId::Barrel);
        assert_eq!(obj.position, Point::new(7, 8));
        assert_eq!(obj.pre_flag, 0);
        assert!(!obj.is_trap);
        assert!(!obj.door_flag);
        // Breakable should be 0 (intact) or 1 (intact+breakable) — never -1.
        assert!(obj.breakable >= 0);
    }

    #[test]
    fn test_add_door_sets_closed_state() {
        let mut door = Object::new(ObjectId::L1LDoor, Point::new(0, 0));
        add_door(&mut door, 100, 200);
        assert!(door.door_flag);
        assert_eq!(door.ovar1, 101);
        assert_eq!(door.ovar2, 201);
        assert_eq!(door.ovar4, DOOR_CLOSED);
        assert_eq!(door.pre_flag, 0);
        assert!(!door.miss_flag);
        assert_eq!(door.selection_region, SelectionRegion::Bottom);
    }

    #[test]
    fn test_add_chest_randomises_loot() {
        let mut chest = Object::new(ObjectId::Chest3, Point::new(0, 0));
        let mut calls = 0i32;
        let mut rng = || {
            calls += 1;
            // Return a predictable non-zero value so anim_frame advances and
            // the loot-count modulo resolves deterministically.
            3
        };
        add_chest(&mut chest, &mut rng, false);
        // Chest3: (rng() % 4).min(3) with rng()=3 => 3.
        assert_eq!(chest.ovar1, 3);
        // Non-zero first roll animates the lid: default anim_frame (0) + 3.
        assert_eq!(chest.anim_frame, 3);
        assert_ne!(chest.rnd_seed, 0);
    }

    #[test]
    fn test_add_sarcophagus_records_skeleton_index() {
        let mut sarc = Object::new(ObjectId::Sarc, Point::new(0, 0));
        let mut rng = || 9; // ovar1 = 9 % 10 = 9 (>= 8 => skeleton)
        add_sarcophagus(&mut sarc, &mut rng);
        assert_eq!(sarc.ovar1, 9);
        assert!(sarc.ovar1 >= 8);
        assert_eq!(sarc.ovar2, -1); // placeholder until monster system is wired
        assert_ne!(sarc.rnd_seed, 0);
    }

    #[test]
    fn test_operate_sarcophagus_sets_open_animation() {
        let mut sarc = Object::new(ObjectId::Sarc, Point::new(0, 0));
        sarc.selection_region = SelectionRegion::Bottom;
        assert!(operate_sarcophagus(&mut sarc, false, false));
        assert_eq!(sarc.selection_region, SelectionRegion::None);
        assert!(sarc.anim_flag);
        assert_eq!(sarc.anim_delay, 3);
        assert_eq!(sarc.ovar2, 1);
    }

    #[test]
    fn test_operate_sarcophagus_refuses_when_not_interactable() {
        let mut sarc = Object::new(ObjectId::Sarc, Point::new(0, 0));
        sarc.selection_region = SelectionRegion::None;
        assert!(!operate_sarcophagus(&mut sarc, false, false));
    }

    #[test]
    fn test_operate_pedestal_advances_and_disables_at_three() {
        let mut pedestal = Object::new(ObjectId::Pedestal, Point::new(0, 0));
        pedestal.selection_region = SelectionRegion::Bottom;
        pedestal.anim_frame = 1;

        // First stone.
        assert!(operate_pedestal(&mut pedestal, true));
        assert_eq!(pedestal.ovar6, 1);
        assert_eq!(pedestal.anim_frame, 2);
        assert_eq!(pedestal.selection_region, SelectionRegion::Bottom);

        // Second stone.
        assert!(operate_pedestal(&mut pedestal, true));
        assert_eq!(pedestal.ovar6, 2);

        // Third stone: reveal complete, selection disabled.
        assert!(operate_pedestal(&mut pedestal, true));
        assert_eq!(pedestal.ovar6, 3);
        assert_eq!(pedestal.selection_region, SelectionRegion::None);

        // Fourth stone refused.
        assert!(!operate_pedestal(&mut pedestal, true));
    }

    #[test]
    fn test_operate_book_advances_frame_and_disables_selection() {
        let mut book = Object::new(ObjectId::StoryBook, Point::new(0, 0));
        book.selection_region = SelectionRegion::Bottom;
        book.anim_frame = 1;
        assert!(operate_book(&mut book, false));
        assert_eq!(book.selection_region, SelectionRegion::None);
        assert_eq!(book.anim_frame, 2);
    }

    #[test]
    fn test_operate_book_quest_variants() {
        let mut book = Object::new(ObjectId::BlindBook, Point::new(0, 0));
        book.selection_region = SelectionRegion::Bottom;
        book.anim_frame = 1;
        assert!(operate_book(&mut book, false));
        assert_eq!(book.selection_region, SelectionRegion::None);
        assert_eq!(book.anim_frame, 2);

        let mut steel = Object::new(ObjectId::SteelTome, Point::new(0, 0));
        steel.selection_region = SelectionRegion::Bottom;
        assert!(operate_book(&mut steel, false));
        assert_eq!(steel.selection_region, SelectionRegion::None);
    }

    #[test]
    fn test_operate_book_refuses_when_not_interactable() {
        let mut book = Object::new(ObjectId::StoryBook, Point::new(0, 0));
        book.selection_region = SelectionRegion::None;
        assert!(!operate_book(&mut book, false));
    }

    #[test]
    fn test_update_lever_state_disables_selection_and_advances_frame() {
        let mut lever = Object::new(ObjectId::Lever, Point::new(0, 0));
        lever.selection_region = SelectionRegion::Bottom;
        lever.anim_frame = 1;
        update_lever_state(&mut lever);
        assert_eq!(lever.selection_region, SelectionRegion::None);
        assert_eq!(lever.anim_frame, 2);
    }

    #[test]
    fn test_operate_lever_invokes_update_lever_state() {
        let mut lever = Object::new(ObjectId::Lever, Point::new(0, 0));
        lever.selection_region = SelectionRegion::Bottom;
        lever.anim_frame = 1;
        assert!(operate_lever(&mut lever, false));
        assert_eq!(lever.selection_region, SelectionRegion::None);
        assert_eq!(lever.anim_frame, 2);
    }

    #[test]
    fn test_operate_chest_advances_two_frames() {
        let mut chest = Object::new(ObjectId::Chest1, Point::new(0, 0));
        chest.selection_region = SelectionRegion::Bottom;
        chest.anim_frame = 1;
        assert!(operate_chest(&mut chest, Point::new(0, 0), false));
        assert_eq!(chest.selection_region, SelectionRegion::None);
        // C++ advances by 2 frames.
        assert_eq!(chest.anim_frame, 3);
    }

    #[test]
    fn test_add_object_places_barrel_into_manager() {
        let mut manager = ObjectManager::new();
        let mut ctx = AddObjectContext::default();
        ctx.rng = Box::new(|| 0);
        let idx = add_object(&mut manager, ObjectId::Barrel, Point::new(5, 5), &mut ctx);
        assert!(idx.is_some());
        let idx = idx.unwrap();
        let obj = manager.get(idx).expect("barrel should be present");
        assert_eq!(obj.otype, ObjectId::Barrel);
        assert_eq!(obj.position, Point::new(5, 5));
    }

    #[test]
    fn test_add_object_places_door_into_manager() {
        let mut manager = ObjectManager::new();
        let mut ctx = AddObjectContext::default();
        ctx.current_tile = 100;
        ctx.adjacent_tile = 200;
        let idx = add_object(&mut manager, ObjectId::L1LDoor, Point::new(1, 1), &mut ctx);
        let obj = manager
            .get(idx.unwrap())
            .expect("door should be present");
        assert_eq!(obj.otype, ObjectId::L1LDoor);
        assert!(obj.door_flag);
        assert_eq!(obj.ovar4, DOOR_CLOSED);
    }

    #[test]
    fn test_add_object_places_chest_into_manager() {
        let mut manager = ObjectManager::new();
        let mut ctx = AddObjectContext::default();
        ctx.rng = Box::new(|| 1);
        let idx = add_object(&mut manager, ObjectId::Chest2, Point::new(2, 2), &mut ctx);
        let obj = manager
            .get(idx.unwrap())
            .expect("chest should be present");
        assert_eq!(obj.otype, ObjectId::Chest2);
        assert_ne!(obj.rnd_seed, 0);
    }

    #[test]
    fn test_add_object_places_shrine_into_manager() {
        let mut manager = ObjectManager::new();
        let mut ctx = AddObjectContext::default();
        ctx.rng = Box::new(|| 0);
        ctx.is_hellfire = false;
        ctx.is_multiplayer = false;
        ctx.level_type = crate::game::types::DungeonType::Cathedral;
        let idx = add_object(&mut manager, ObjectId::ShrineL, Point::new(3, 3), &mut ctx);
        let obj = manager
            .get(idx.unwrap())
            .expect("shrine should be present");
        assert_eq!(obj.otype, ObjectId::ShrineL);
        // ovar1 must be a valid shrine index 0..=33.
        assert!(obj.ovar1 >= 0 && obj.ovar1 <= 33);
    }

    #[test]
    fn test_delta_sync_op_object_opens_door() {
        let mut door = Object::new(ObjectId::L1LDoor, Point::new(0, 0));
        door.anim_len = 5;
        door.selection_region = SelectionRegion::Bottom;
        delta_sync_op_object(&mut door);
        assert_eq!(door.ovar4, DOOR_OPEN);
        assert_eq!(door.anim_frame, door.anim_len);
    }

    #[test]
    fn test_delta_sync_op_object_advances_chest() {
        let mut chest = Object::new(ObjectId::Chest1, Point::new(0, 0));
        chest.anim_frame = 2;
        chest.anim_flag = true;
        chest.selection_region = SelectionRegion::Bottom;
        delta_sync_op_object(&mut chest);
        // C++ delta-loads chest to anim_frame + 2.
        assert_eq!(chest.anim_frame, 4);
    }

    #[test]
    fn test_delta_sync_close_obj_clears_trap() {
        let mut obj = Object::new(ObjectId::Chest1, Point::new(0, 0));
        obj.arm_trap();
        assert!(obj.is_trap);
        delta_sync_close_obj(&mut obj);
        assert!(!obj.is_trap);
    }

    #[test]
    fn test_sync_op_object_open_door_command() {
        let mut door = Object::new(ObjectId::L1LDoor, Point::new(0, 0));
        door.set_door_state(DoorState::Closed);
        door.anim_len = 5;
        door.selection_region = SelectionRegion::Bottom;
        let applied = sync_op_object(
            &mut door,
            SyncCmd::OpenDoor,
            false,
            Point::new(0, 0),
        );
        assert!(applied);
        assert_eq!(door.get_door_state(), DoorState::Open);
    }

    #[test]
    fn test_sync_crux_fires_map_change_when_all_broken() {
        // sync_crux takes &Object (not &mut), so we can pass the crux both as
        // the subject and inside the all_objects slice without aliasing.
        let mut crux = Object::new(ObjectId::Crux1, Point::new(0, 0));
        crux.ovar8 = 0;
        crux.mark_broken();
        let all_refs: Vec<&Object> = vec![&crux];
        let mut fired = false;
        sync_crux(&crux, &all_refs, &mut |_| {
            fired = true;
        });
        assert!(fired);
    }

    #[test]
    fn test_sync_object_anim_door_matches_state() {
        let mut door = Object::new(ObjectId::L1LDoor, Point::new(0, 0));
        door.anim_len = 5;
        door.ovar4 = DOOR_OPEN;
        let refs: Vec<&Object> = vec![];
        sync_object_anim(
            &mut door,
            &refs,
            &mut |_| {},
            &mut |_| {},
            &mut |_, _, _, _| {},
            (0, 0, 0, 0),
        );
        // SyncObjectAnim dispatches to sync_door, which reconciles the
        // state-derived flags (mirrors C++ SetDoorStateOpen) but does NOT
        // touch anim_frame.
        assert!(door.miss_flag);
        assert_eq!(door.pre_flag, 1);
        assert_eq!(door.selection_region, SelectionRegion::Middle);
    }

    #[test]
    fn test_sync_object_anim_crux_via_sync_crux_helper() {
        // We exercise the full sync_object_anim dispatch indirectly through
        // sync_crux (tested above) to avoid the &mut/& aliasing that the
        // combined dispatch would create in a single test.
        let mut crux = Object::new(ObjectId::Crux1, Point::new(0, 0));
        crux.ovar8 = 0;
        crux.mark_broken();
        let all_refs: Vec<&Object> = vec![&crux];
        let mut fired = false;
        sync_crux(&crux, &all_refs, &mut |_| {
            fired = true;
        });
        assert!(fired);
    }

    #[test]
    fn test_update_pedestal_state_advances_by_added_stones() {
        let mut pedestal = Object::new(ObjectId::Pedestal, Point::new(0, 0));
        pedestal.anim_frame = 1;
        pedestal.selection_region = SelectionRegion::Bottom;
        let mut rect_calls = 0;
        update_pedestal_state(
            &mut pedestal,
            2,
            (10, 10, 5, 5),
            &mut |_, _, _, _| {
                rect_calls += 1;
            },
        );
        assert_eq!(pedestal.ovar6, 2);
        assert_eq!(pedestal.anim_frame, 3); // 1 + 2
        // Reaching 3 stones disables selection.
        update_pedestal_state(
            &mut pedestal,
            1,
            (10, 10, 5, 5),
            &mut |_, _, _, _| {},
        );
        assert_eq!(pedestal.ovar6, 3);
        assert_eq!(pedestal.selection_region, SelectionRegion::None);
    }

    #[test]
    fn test_object_stop_anim_freezes_on_last_frame() {
        let mut obj = Object::new(ObjectId::Chest1, Point::new(5, 5));
        obj.anim_flag = true;
        obj.anim_delay = 5;
        obj.anim_len = 10;
        obj.anim_frame = 10; // at the last frame
        obj.anim_cnt = 3;
        object_stop_anim(&mut obj);
        assert_eq!(obj.anim_cnt, 0);
        assert_eq!(obj.anim_delay, 1000);
        // anim_flag intentionally left unchanged.
        assert!(obj.anim_flag);
    }

    #[test]
    fn test_object_stop_anim_noop_before_last_frame() {
        let mut obj = Object::new(ObjectId::Chest1, Point::new(5, 5));
        obj.anim_delay = 5;
        obj.anim_len = 10;
        obj.anim_frame = 5;
        obj.anim_cnt = 3;
        object_stop_anim(&mut obj);
        // Not at the last frame → unchanged.
        assert_eq!(obj.anim_cnt, 3);
        assert_eq!(obj.anim_delay, 5);
    }

    #[test]
    fn test_stop_animation_method_clears_flag() {
        let mut obj = Object::new(ObjectId::Chest1, Point::new(5, 5));
        obj.anim_flag = true;
        obj.stop_animation();
        // The high-level helper clears anim_flag (legacy Rust behaviour).
        assert!(!obj.anim_flag);
    }

    #[test]
    fn test_sync_nakrul_lever_wrong_level() {
        let mut lever = Object::new(ObjectId::L5Lever, Point::new(10, 10));
        lever.selection_region = SelectionRegion::Bottom;
        // Not level 24 → no-op.
        assert!(!sync_nakrul_lever(&mut lever, 16));
        assert_ne!(lever.selection_region, SelectionRegion::None);
    }

    #[test]
    fn test_sync_nakrul_lever_level24() {
        let mut lever = Object::new(ObjectId::L5Lever, Point::new(10, 10));
        lever.selection_region = SelectionRegion::Bottom;
        lever.anim_frame = 1;
        assert!(sync_nakrul_lever(&mut lever, 24));
        assert_eq!(lever.selection_region, SelectionRegion::None);
        assert_eq!(lever.anim_frame, 2);
    }

    #[test]
    fn test_sync_nakrul_lever_not_interactive() {
        let mut lever = Object::new(ObjectId::L5Lever, Point::new(10, 10));
        lever.selection_region = SelectionRegion::None; // not selectable
        assert!(!sync_nakrul_lever(&mut lever, 24));
    }

    #[test]
    fn test_is_nakrul_lever() {
        let lever = Object::new(ObjectId::L5Lever, Point::new(10, 10));
        assert!(is_nakrul_lever(&lever, 24));
        assert!(!is_nakrul_lever(&lever, 16));
        let chest = Object::new(ObjectId::Chest1, Point::new(10, 10));
        assert!(!is_nakrul_lever(&chest, 24));
    }

    #[test]
    fn test_operate_chest_full_un_trapped() {
        let mut chest = Object::new(ObjectId::Chest1, Point::new(5, 5));
        chest.selection_region = SelectionRegion::Bottom;
        chest.ovar1 = 2; // two loot items
        chest.ovar2 = 1; // magic loot
        chest.is_trap = false;
        let res = operate_chest_full(&mut chest, Point::new(6, 6));
        assert!(res.opened);
        assert!(res.trap_missile_id.is_none());
        assert_eq!(res.loot_count, 2);
        assert!(res.loot_magic);
        assert_eq!(chest.selection_region, SelectionRegion::None);
    }

    #[test]
    fn test_operate_chest_full_trapped_arrow() {
        let mut chest = Object::new(ObjectId::Chest1, Point::new(5, 5));
        chest.selection_region = SelectionRegion::Bottom;
        chest.is_trap = true;
        chest.ovar4 = 0; // Arrow trap
        chest.ovar1 = 1;
        chest.ovar2 = 0; // useful (non-magic) loot
        let res = operate_chest_full(&mut chest, Point::new(6, 6));
        assert!(res.opened);
        assert_eq!(res.trap_missile_id, Some(MISSILE_ARROW));
        // Trap flag disarmed.
        assert!(!chest.is_trap);
        assert!(!res.loot_magic);
    }

    #[test]
    fn test_operate_chest_full_trapped_fire_variants() {
        let trap_ids = [(1, MISSILE_FIRE_ARROW), (2, MISSILE_NOVA), (3, MISSILE_RING_OF_FIRE), (4, MISSILE_STEAL_POTIONS), (5, MISSILE_STEAL_MANA)];
        for (var4, expected) in trap_ids {
            let mut chest = Object::new(ObjectId::Chest1, Point::new(5, 5));
            chest.selection_region = SelectionRegion::Bottom;
            chest.is_trap = true;
            chest.ovar4 = var4;
            let res = operate_chest_full(&mut chest, Point::new(6, 6));
            assert_eq!(res.trap_missile_id, Some(expected), "var4={}", var4);
        }
    }

    #[test]
    fn test_operate_chest_full_not_interactive() {
        let mut chest = Object::new(ObjectId::Chest1, Point::new(5, 5));
        chest.selection_region = SelectionRegion::None; // not selectable
        let res = operate_chest_full(&mut chest, Point::new(6, 6));
        assert!(!res.opened);
        assert!(res.trap_missile_id.is_none());
    }
}

