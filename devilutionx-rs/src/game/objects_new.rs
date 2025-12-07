//! Exact port of object system from DevilutionX
//!
//! Port of objects.h and objdat.h

#![allow(non_snake_case)]
#![allow(dead_code)]

use super::level_new::DungeonType;
use super::quest_new::QuestId;

// ============================================================================
// Constants
// ============================================================================

/// Maximum number of objects in a level
pub const MAXOBJECTS: usize = 127;

// ============================================================================
// Theme ID enum - exact port from objdat.h
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i8)]
pub enum ThemeId {
    Barrel = 0,
    Shrine = 1,
    MonstPit = 2,
    SkelRoom = 3,
    Treasure = 4,
    Library = 5,
    Torture = 6,
    BloodFountain = 7,
    Decapitated = 8,
    PurifyingFountain = 9,
    ArmorStand = 10,
    GoatShrine = 11,
    Cauldron = 12,
    MurkyFountain = 13,
    TearFountain = 14,
    BrnCross = 15,
    WeaponRack = 16,
    #[default]
    None = -1,
}

// ============================================================================
// Object ID enum - exact port from objdat.h
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i8)]
pub enum ObjectId {
    L1Light = 0,
    L1LDoor = 1,
    L1RDoor = 2,
    SkFire = 3,
    Lever = 4,
    Chest1 = 5,
    Chest2 = 6,
    Chest3 = 7,
    Candle1 = 8,
    Candle2 = 9,
    CandleO = 10,
    BannerL = 11,
    BannerM = 12,
    BannerR = 13,
    SkPile = 14,
    SkStick1 = 15,
    SkStick2 = 16,
    SkStick3 = 17,
    SkStick4 = 18,
    SkStick5 = 19,
    Crux1 = 20,
    Crux2 = 21,
    Crux3 = 22,
    Stand = 23,
    Angel = 24,
    Book2L = 25,
    BCross = 26,
    NudeW2R = 27,
    SwitchSkl = 28,
    TNudeM1 = 29,
    TNudeM2 = 30,
    TNudeM3 = 31,
    TNudeM4 = 32,
    TNudeW1 = 33,
    TNudeW2 = 34,
    TNudeW3 = 35,
    Torture1 = 36,
    Torture2 = 37,
    Torture3 = 38,
    Torture4 = 39,
    Torture5 = 40,
    Book2R = 41,
    L2LDoor = 42,
    L2RDoor = 43,
    TorchL = 44,
    TorchR = 45,
    TorchL2 = 46,
    TorchR2 = 47,
    Sarc = 48,
    FlameHole = 49,
    FlameLvr = 50,
    Water = 51,
    BookLvr = 52,
    TrapL = 53,
    TrapR = 54,
    BookShelf = 55,
    WeapRack = 56,
    Barrel = 57,
    BarrelEx = 58,
    ShrineL = 59,
    ShrineR = 60,
    SkelBook = 61,
    BookCaseL = 62,
    BookCaseR = 63,
    BookStand = 64,
    BookCandle = 65,
    BloodFtn = 66,
    Decap = 67,
    TChest1 = 68,
    TChest2 = 69,
    TChest3 = 70,
    BlindBook = 71,
    BloodBook = 72,
    Pedestal = 73,
    L3LDoor = 74,
    L3RDoor = 75,
    PurifyingFtn = 76,
    ArmorStand = 77,
    ArmorStandN = 78,
    GoatShrine = 79,
    Cauldron = 80,
    MurkyFtn = 81,
    TearFtn = 82,
    AltBoy = 83,
    MCircle1 = 84,
    MCircle2 = 85,
    StoryBook = 86,
    StoryCandle = 87,
    SteelTome = 88,
    WarArmor = 89,
    WarWeap = 90,
    TBCross = 91,
    WeaponRack = 92,
    WeaponRackN = 93,
    MushPatch = 94,
    LazStand = 95,
    SlainHero = 96,
    SignChest = 97,
    BookShelfR = 98,
    Pod = 99,
    PodEx = 100,
    Urn = 101,
    UrnEx = 102,
    L5Books = 103,
    L5Candle = 104,
    L5LDoor = 105,
    L5RDoor = 106,
    L5Lever = 107,
    L5Sarc = 108,
    
    #[default]
    Null = -1,
}

impl ObjectId {
    /// Check if this object is a barrel (or explosive barrel)
    pub fn is_barrel(&self) -> bool {
        matches!(
            self,
            ObjectId::Barrel | ObjectId::BarrelEx |
            ObjectId::Pod | ObjectId::PodEx |
            ObjectId::Urn | ObjectId::UrnEx
        )
    }
    
    /// Check if this object contains explosives
    pub fn is_explosive(&self) -> bool {
        matches!(
            self,
            ObjectId::BarrelEx | ObjectId::PodEx | ObjectId::UrnEx
        )
    }
    
    /// Check if this object is a chest
    pub fn is_chest(&self) -> bool {
        matches!(
            self,
            ObjectId::Chest1 | ObjectId::Chest2 | ObjectId::Chest3 |
            ObjectId::TChest1 | ObjectId::TChest2 | ObjectId::TChest3
        )
    }
    
    /// Check if this object is a trapped chest
    pub fn is_trapped_chest(&self) -> bool {
        matches!(
            self,
            ObjectId::TChest1 | ObjectId::TChest2 | ObjectId::TChest3
        )
    }
    
    /// Check if this object is a crucifix
    pub fn is_crux(&self) -> bool {
        matches!(
            self,
            ObjectId::Crux1 | ObjectId::Crux2 | ObjectId::Crux3
        )
    }
    
    /// Check if this object is a door
    pub fn is_door(&self) -> bool {
        matches!(
            self,
            ObjectId::L1LDoor | ObjectId::L1RDoor |
            ObjectId::L2LDoor | ObjectId::L2RDoor |
            ObjectId::L3LDoor | ObjectId::L3RDoor |
            ObjectId::L5LDoor | ObjectId::L5RDoor
        )
    }
    
    /// Check if this object is a shrine
    pub fn is_shrine(&self) -> bool {
        matches!(self, ObjectId::ShrineL | ObjectId::ShrineR)
    }
    
    /// Check if this object is a trap
    pub fn is_trap(&self) -> bool {
        matches!(self, ObjectId::TrapL | ObjectId::TrapR)
    }
}

// ============================================================================
// Selection Region enum - exact port
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum SelectionRegion {
    #[default]
    None = 0,
    Top = 1,
    Mid = 2,
    Bottom = 3,
    Full = 4,
}

// ============================================================================
// Object Data Flags - exact port from objdat.h
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ObjectDataFlags(pub u8);

impl ObjectDataFlags {
    pub const NONE: Self = Self(0);
    pub const ANIMATED: Self = Self(1 << 0);
    pub const SOLID: Self = Self(1 << 1);
    pub const MISSILES_PASS_THROUGH: Self = Self(1 << 2);
    pub const LIGHT: Self = Self(1 << 3);
    pub const TRAP: Self = Self(1 << 4);
    pub const BREAKABLE: Self = Self(1 << 5);
    
    pub fn has(&self, flag: Self) -> bool {
        (self.0 & flag.0) != 0
    }
    
    pub fn is_animated(&self) -> bool {
        self.has(Self::ANIMATED)
    }
    
    pub fn is_solid(&self) -> bool {
        self.has(Self::SOLID)
    }
    
    pub fn is_light(&self) -> bool {
        self.has(Self::LIGHT)
    }
    
    pub fn is_trap(&self) -> bool {
        self.has(Self::TRAP)
    }
    
    pub fn is_breakable(&self) -> bool {
        self.has(Self::BREAKABLE)
    }
}

// ============================================================================
// Object Data structure - exact port from objdat.h
// ============================================================================

#[derive(Debug, Clone)]
pub struct ObjectData {
    /// Object graphic index
    pub ofindex: u8,
    
    /// Minimum level this object appears on
    pub minlvl: i8,
    
    /// Maximum level this object appears on
    pub maxlvl: i8,
    
    /// Dungeon type where object appears
    pub olvltype: DungeonType,
    
    /// Theme ID for themed rooms
    pub otheme: ThemeId,
    
    /// Quest this object belongs to
    pub oquest: QuestId,
    
    /// Object flags
    pub flags: ObjectDataFlags,
    
    /// Tick length of each frame
    pub anim_delay: u8,
    
    /// Number of frames in animation
    pub anim_len: u8,
    
    /// Animation width
    pub anim_width: u8,
    
    /// Selection region
    pub selection_region: SelectionRegion,
}

// ============================================================================
// Object structure - exact port from objects.h
// ============================================================================

#[derive(Debug, Clone)]
pub struct Object {
    /// Object type ID
    pub _otype: ObjectId,
    
    /// Whether to apply lighting
    pub apply_lighting: bool,
    
    /// Is this object trapped
    pub _oTrapFlag: bool,
    
    /// Is this a door
    pub _oDoorFlag: bool,
    
    /// Object position (tile coordinates)
    pub position: (i32, i32),
    
    /// Animation flag
    pub _oAnimFlag: u32,
    
    /// Animation delay (tick length of each frame)
    pub _oAnimDelay: i32,
    
    /// Animation counter (ticks until next frame)
    pub _oAnimCnt: i32,
    
    /// Animation length (number of frames)
    pub _oAnimLen: u32,
    
    /// Current animation frame
    pub _oAnimFrame: u32,
    
    /// Animation width
    pub _oAnimWidth: u16,
    
    /// Delete flag
    pub _oDelFlag: bool,
    
    /// Breakable state: 0=not breakable, 1=breakable, -1=broken
    pub _oBreak: i8,
    
    /// Is solid (blocks movement)
    pub _oSolidFlag: bool,
    
    /// Missiles pass through
    pub _oMissFlag: bool,
    
    /// Selection region
    pub selection_region: SelectionRegion,
    
    /// Pre-render flag
    pub _oPreFlag: bool,
    
    /// Light ID
    pub _olid: i32,
    
    /// Random seed for item generation
    pub _oRndSeed: u32,
    
    // Object variables (used for various purposes)
    pub _oVar1: i32,
    pub _oVar2: i32,
    pub _oVar3: i32,
    pub _oVar4: i32,
    pub _oVar5: i32,
    pub _oVar6: u32,
    pub _oVar8: i32,
    
    /// Quest message to play when activated
    pub book_message: i16, // SpeechId
}

impl Default for Object {
    fn default() -> Self {
        Self {
            _otype: ObjectId::Null,
            apply_lighting: false,
            _oTrapFlag: false,
            _oDoorFlag: false,
            position: (0, 0),
            _oAnimFlag: 0,
            _oAnimDelay: 0,
            _oAnimCnt: 0,
            _oAnimLen: 0,
            _oAnimFrame: 0,
            _oAnimWidth: 0,
            _oDelFlag: false,
            _oBreak: 0,
            _oSolidFlag: false,
            _oMissFlag: false,
            selection_region: SelectionRegion::None,
            _oPreFlag: false,
            _olid: 0,
            _oRndSeed: 0,
            _oVar1: 0,
            _oVar2: 0,
            _oVar3: 0,
            _oVar4: 0,
            _oVar5: 0,
            _oVar6: 0,
            _oVar8: 0,
            book_message: -1, // TEXT_NONE
        }
    }
}

impl Object {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Check if the object can be broken
    pub fn is_breakable(&self) -> bool {
        self._oBreak == 1
    }
    
    /// Check if the object has been broken
    pub fn is_broken(&self) -> bool {
        self._oBreak == -1
    }
    
    /// Check if object can be interacted with
    pub fn can_interact_with(&self) -> bool {
        self.selection_region != SelectionRegion::None
    }
    
    /// Check if this object is a barrel
    pub fn is_barrel(&self) -> bool {
        self._otype.is_barrel()
    }
    
    /// Check if this object is explosive
    pub fn is_explosive(&self) -> bool {
        self._otype.is_explosive()
    }
    
    /// Check if this object is a chest
    pub fn is_chest(&self) -> bool {
        self._otype.is_chest()
    }
    
    /// Check if this is a trapped chest with active trap
    pub fn is_trapped_chest(&self) -> bool {
        self._otype.is_trapped_chest() && self._oTrapFlag
    }
    
    /// Check if this is an untrapped chest
    pub fn is_untrapped_chest(&self) -> bool {
        matches!(
            self._otype,
            ObjectId::Chest1 | ObjectId::Chest2 | ObjectId::Chest3
        ) && !self._oTrapFlag
    }
    
    /// Check if this is a crucifix
    pub fn is_crux(&self) -> bool {
        self._otype.is_crux()
    }
    
    /// Check if this is a door
    pub fn is_door(&self) -> bool {
        self._otype.is_door()
    }
    
    /// Check if this is a shrine
    pub fn is_shrine(&self) -> bool {
        self._otype.is_shrine()
    }
    
    /// Check if this is a trap
    pub fn is_trap(&self) -> bool {
        self._otype.is_trap()
    }
    
    /// Set map range for objects that affect map regions
    pub fn set_map_range(&mut self, top_left: (i32, i32), bottom_right: (i32, i32)) {
        self._oVar1 = top_left.0;
        self._oVar2 = top_left.1;
        self._oVar3 = bottom_right.0;
        self._oVar4 = bottom_right.1;
    }
    
    /// Initialize book object
    pub fn initialize_book(&mut self, map_range: ((i32, i32), (i32, i32))) {
        self.set_map_range(map_range.0, map_range.1);
        self._oVar6 = self._oAnimFrame + 1; // Save open book frame
    }
    
    /// Initialize quest book
    pub fn initialize_quest_book(&mut self, map_range: ((i32, i32), (i32, i32)), lever_id: i32, message: i16) {
        self.initialize_book(map_range);
        self._oVar8 = lever_id;
        self.book_message = message;
    }
}

// ============================================================================
// Object Manager
// ============================================================================

#[derive(Debug, Clone)]
pub struct ObjectManager {
    /// All objects in the level
    pub objects: Vec<Object>,
    
    /// Available object slots
    pub available_objects: Vec<usize>,
    
    /// Active object indices
    pub active_objects: Vec<usize>,
    
    /// Count of active objects
    pub active_object_count: usize,
    
    /// Loading map objects flag
    pub loading_map_objects: bool,
}

impl Default for ObjectManager {
    fn default() -> Self {
        let mut available = Vec::with_capacity(MAXOBJECTS);
        for i in 0..MAXOBJECTS {
            available.push(i);
        }
        
        Self {
            objects: vec![Object::default(); MAXOBJECTS],
            available_objects: available,
            active_objects: Vec::with_capacity(MAXOBJECTS),
            active_object_count: 0,
            loading_map_objects: false,
        }
    }
}

impl ObjectManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Clear all objects
    pub fn clear(&mut self) {
        self.objects.fill(Object::default());
        self.available_objects.clear();
        for i in 0..MAXOBJECTS {
            self.available_objects.push(i);
        }
        self.active_objects.clear();
        self.active_object_count = 0;
    }
    
    /// Add an object at a position
    pub fn add_object(&mut self, obj_type: ObjectId, position: (i32, i32)) -> Option<usize> {
        if self.available_objects.is_empty() {
            return None;
        }
        
        let idx = self.available_objects.pop()?;
        
        let obj = &mut self.objects[idx];
        obj._otype = obj_type;
        obj.position = position;
        obj._oDelFlag = false;
        
        self.active_objects.push(idx);
        self.active_object_count += 1;
        
        Some(idx)
    }
    
    /// Delete an object
    pub fn delete_object(&mut self, idx: usize) {
        if idx >= MAXOBJECTS {
            return;
        }
        
        self.objects[idx]._oDelFlag = true;
        
        // Remove from active list
        if let Some(pos) = self.active_objects.iter().position(|&x| x == idx) {
            self.active_objects.remove(pos);
            self.active_object_count -= 1;
        }
        
        // Return to available pool
        self.available_objects.push(idx);
    }
    
    /// Find object at position
    pub fn find_object_at_position(&self, position: (i32, i32)) -> Option<usize> {
        for &idx in &self.active_objects {
            let obj = &self.objects[idx];
            if obj.position == position && !obj._oDelFlag {
                return Some(idx);
            }
        }
        None
    }
    
    /// Get object by index
    pub fn get_object(&self, idx: usize) -> Option<&Object> {
        if idx < MAXOBJECTS && !self.objects[idx]._oDelFlag {
            Some(&self.objects[idx])
        } else {
            None
        }
    }
    
    /// Get mutable object by index
    pub fn get_object_mut(&mut self, idx: usize) -> Option<&mut Object> {
        if idx < MAXOBJECTS && !self.objects[idx]._oDelFlag {
            Some(&mut self.objects[idx])
        } else {
            None
        }
    }
    
    /// Process all objects (animation updates)
    pub fn process_objects(&mut self) {
        for &idx in &self.active_objects {
            let obj = &mut self.objects[idx];
            
            if obj._oAnimFlag != 0 {
                obj._oAnimCnt += 1;
                if obj._oAnimCnt >= obj._oAnimDelay {
                    obj._oAnimCnt = 0;
                    obj._oAnimFrame += 1;
                    if obj._oAnimFrame > obj._oAnimLen {
                        obj._oAnimFrame = 1;
                    }
                }
            }
        }
    }
    
    /// Check if position is blocked by object
    pub fn is_position_blocked(&self, position: (i32, i32)) -> bool {
        for &idx in &self.active_objects {
            let obj = &self.objects[idx];
            if obj.position == position && obj._oSolidFlag && !obj._oDelFlag {
                return true;
            }
        }
        false
    }
}

// ============================================================================
// Shrine types - for shrine effect lookup
// ============================================================================

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
    Eldritch = 11,
    Eerie = 12,
    Divine = 13,
    Holy = 14,
    Sacred = 15,
    Spiritual = 16,
    Spooky = 17,
    Abandoned = 18,
    Creepy = 19,
    Quiet = 20,
    Secluded = 21,
    Ornate = 22,
    Glimmering = 23,
    Tainted = 24,
    // Hellfire shrines
    Oily = 25,
    Glowing = 26,
    Mendicant = 27,
    Sparkling = 28,
    Town = 29,
    Shimmering = 30,
    Solar = 31,
    Murphy = 32,
}

/// Shrine names for display
pub static SHRINE_NAMES: &[&str] = &[
    "Mysterious", "Hidden", "Gloomy", "Weird", "Magical",
    "Stone", "Religious", "Enchanted", "Thaumaturgic", "Fascinating",
    "Cryptic", "Eldritch", "Eerie", "Divine", "Holy",
    "Sacred", "Spiritual", "Spooky", "Abandoned", "Creepy",
    "Quiet", "Secluded", "Ornate", "Glimmering", "Tainted",
    // Hellfire
    "Oily", "Glowing", "Mendicant", "Sparkling", "Town",
    "Shimmering", "Solar", "Murphy's",
];

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_object_manager() {
        let mut manager = ObjectManager::new();
        
        // Add object
        let idx = manager.add_object(ObjectId::Barrel, (10, 10));
        assert!(idx.is_some());
        
        let idx = idx.unwrap();
        let obj = manager.get_object(idx).unwrap();
        assert_eq!(obj._otype, ObjectId::Barrel);
        assert_eq!(obj.position, (10, 10));
        
        // Find by position
        let found = manager.find_object_at_position((10, 10));
        assert_eq!(found, Some(idx));
        
        // Delete
        manager.delete_object(idx);
        let found = manager.find_object_at_position((10, 10));
        assert!(found.is_none());
    }
    
    #[test]
    fn test_object_type_checks() {
        assert!(ObjectId::Barrel.is_barrel());
        assert!(ObjectId::BarrelEx.is_explosive());
        assert!(ObjectId::Chest1.is_chest());
        assert!(ObjectId::TChest1.is_trapped_chest());
        assert!(ObjectId::L1LDoor.is_door());
        assert!(ObjectId::ShrineL.is_shrine());
    }
}
