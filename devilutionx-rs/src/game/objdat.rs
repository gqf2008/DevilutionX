//! Exact port of object data system from DevilutionX
//!
//! Contains object type definitions, theme IDs, quest IDs, and object data tables
//! Ported from Source/objdat.h and Source/objdat.cpp

#![allow(non_snake_case)]
#![allow(dead_code)]

use super::types::DungeonType;

// ============================================================================
// Selection Region
// ============================================================================

// C++ SelectionRegion flags (cursor.h:19-24): Bottom=1, Middle=2, Top=4.
pub const SEL_NONE: u8 = 0;
pub const SEL_BOTTOM: u8 = 1 << 0;
pub const SEL_MIDDLE: u8 = 1 << 1;
pub const SEL_TOP: u8 = 1 << 2;
/// "Bottom,Middle" combo used by sarcophagi/doors (value 3).
pub const SEL_BOTTOM_MIDDLE: u8 = SEL_BOTTOM | SEL_MIDDLE;
pub type SelectionRegion = u8;

// ============================================================================
// Object ID - 109 object types (OBJ_L1LIGHT to OBJ_L5SARC)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i8)]
pub enum ObjectId {
    #[default]
    Null = -1,
    L1Light = 0,
    L1LDoor,
    L1RDoor,
    SkFire,
    Lever,
    Chest1,
    Chest2,
    Chest3,
    Candle1,
    Candle2,
    CandleO,
    BannerL,
    BannerM,
    BannerR,
    SkPile,
    SkStick1,
    SkStick2,
    SkStick3,
    SkStick4,
    SkStick5,
    Crux1,
    Crux2,
    Crux3,
    Stand,
    Angel,
    Book2L,
    BCross,
    NudeW2R,
    SwitchSkl,
    TNudeM1,
    TNudeM2,
    TNudeM3,
    TNudeM4,
    TNudeW1,
    TNudeW2,
    TNudeW3,
    Torture1,
    Torture2,
    Torture3,
    Torture4,
    Torture5,
    Book2R,
    L2LDoor,
    L2RDoor,
    TorchL,
    TorchR,
    TorchL2,
    TorchR2,
    Sarc,
    FlameHole,
    FlameLvr,
    Water,
    BookLvr,
    TrapL,
    TrapR,
    Bookshelf,
    WeapRack,
    Barrel,
    BarrelEx,
    ShrineL,
    ShrineR,
    SkelBook,
    BookcaseL,
    BookcaseR,
    Bookstand,
    BookCandle,
    BloodFtn,
    Decap,
    TChest1,
    TChest2,
    TChest3,
    BlindBook,
    BloodBook,
    Pedestal,
    L3LDoor,
    L3RDoor,
    PurifyingFtn,
    ArmorStand,
    ArmorStandN,
    GoatShrine,
    Cauldron,
    MurkyFtn,
    TearFtn,
    AltBoy,
    MCircle1,
    MCircle2,
    StoryBook,
    StoryCandle,
    SteelTome,
    WarArmor,
    WarWeap,
    TBCross,
    WeaponRack,
    WeaponRackN,
    MushPatch,
    LazStand,
    SlainHero,
    SignChest,
    BookshelfR,
    Pod,
    PodEx,
    Urn,
    UrnEx,
    L5Books,
    L5Candle,
    L5LDoor,
    L5RDoor,
    L5Lever,
    L5Sarc,
}

impl ObjectId {
    pub const LAST: Self = Self::L5Sarc;
    pub const NULL: i8 = -1;

    /// Total number of object types (0-based indexing, so LAST+1)
    pub const COUNT: usize = 109;

    /// Convert from i8 to ObjectId (safe version using match)
    pub fn from_i8(value: i8) -> Option<Self> {
        // Use a lookup table approach for safety
        const IDS: [ObjectId; 109] = [
            ObjectId::L1Light, ObjectId::L1LDoor, ObjectId::L1RDoor, ObjectId::SkFire, ObjectId::Lever,
            ObjectId::Chest1, ObjectId::Chest2, ObjectId::Chest3, ObjectId::Candle1, ObjectId::Candle2,
            ObjectId::CandleO, ObjectId::BannerL, ObjectId::BannerM, ObjectId::BannerR, ObjectId::SkPile,
            ObjectId::SkStick1, ObjectId::SkStick2, ObjectId::SkStick3, ObjectId::SkStick4, ObjectId::SkStick5,
            ObjectId::Crux1, ObjectId::Crux2, ObjectId::Crux3, ObjectId::Stand, ObjectId::Angel,
            ObjectId::Book2L, ObjectId::BCross, ObjectId::NudeW2R, ObjectId::SwitchSkl, ObjectId::TNudeM1,
            ObjectId::TNudeM2, ObjectId::TNudeM3, ObjectId::TNudeM4, ObjectId::TNudeW1, ObjectId::TNudeW2,
            ObjectId::TNudeW3, ObjectId::Torture1, ObjectId::Torture2, ObjectId::Torture3, ObjectId::Torture4,
            ObjectId::Torture5, ObjectId::Book2R, ObjectId::L2LDoor, ObjectId::L2RDoor, ObjectId::TorchL,
            ObjectId::TorchR, ObjectId::TorchL2, ObjectId::TorchR2, ObjectId::Sarc, ObjectId::FlameHole,
            ObjectId::FlameLvr, ObjectId::Water, ObjectId::BookLvr, ObjectId::TrapL, ObjectId::TrapR,
            ObjectId::Bookshelf, ObjectId::WeapRack, ObjectId::Barrel, ObjectId::BarrelEx,
            ObjectId::ShrineL, ObjectId::ShrineR, ObjectId::SkelBook, ObjectId::BookcaseL, ObjectId::BookcaseR,
            ObjectId::Bookstand, ObjectId::BookCandle, ObjectId::BloodFtn, ObjectId::Decap, ObjectId::TChest1,
            ObjectId::TChest2, ObjectId::TChest3, ObjectId::BlindBook, ObjectId::BloodBook, ObjectId::Pedestal,
            ObjectId::L3LDoor, ObjectId::L3RDoor, ObjectId::PurifyingFtn, ObjectId::ArmorStand, ObjectId::ArmorStandN,
            ObjectId::GoatShrine, ObjectId::Cauldron, ObjectId::MurkyFtn, ObjectId::TearFtn, ObjectId::AltBoy,
            ObjectId::MCircle1, ObjectId::MCircle2, ObjectId::StoryBook, ObjectId::StoryCandle, ObjectId::SteelTome,
            ObjectId::WarArmor, ObjectId::WarWeap, ObjectId::TBCross, ObjectId::WeaponRack, ObjectId::WeaponRackN,
            ObjectId::MushPatch, ObjectId::LazStand, ObjectId::SlainHero, ObjectId::SignChest, ObjectId::BookshelfR,
            ObjectId::Pod, ObjectId::PodEx, ObjectId::Urn, ObjectId::UrnEx, ObjectId::L5Books,
            ObjectId::L5Candle, ObjectId::L5LDoor, ObjectId::L5RDoor, ObjectId::L5Lever, ObjectId::L5Sarc,
        ];

        if value >= 0 && value < 109 {
            Some(IDS[value as usize])
        } else {
            None
        }
    }

    /// Convert to i8
    pub fn to_i8(self) -> i8 {
        self as i8
    }
}

// ============================================================================
// Theme ID - 17 theme types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum ThemeId {
    Barrel = 0,
    Shrine,
    MonstPit,
    SkelRoom,
    Treasure,
    Library,
    Torture,
    BloodFountain,
    Decapitated,
    PurifyingFountain,
    ArmorStand,
    GoatShrine,
    Cauldron,
    MurkyFountain,
    TearFountain,
    BrnCross,
    WeaponRack,
}

impl ThemeId {
    pub const NONE: i8 = -1;

    /// Convert from i8 to ThemeId
    pub fn from_i8(value: i8) -> Option<Self> {
        match value {
            0 => Some(Self::Barrel),
            1 => Some(Self::Shrine),
            2 => Some(Self::MonstPit),
            3 => Some(Self::SkelRoom),
            4 => Some(Self::Treasure),
            5 => Some(Self::Library),
            6 => Some(Self::Torture),
            7 => Some(Self::BloodFountain),
            8 => Some(Self::Decapitated),
            9 => Some(Self::PurifyingFountain),
            10 => Some(Self::ArmorStand),
            11 => Some(Self::GoatShrine),
            12 => Some(Self::Cauldron),
            13 => Some(Self::MurkyFountain),
            14 => Some(Self::TearFountain),
            15 => Some(Self::BrnCross),
            16 => Some(Self::WeaponRack),
            _ => None,
        }
    }

    /// Convert to i8
    pub fn to_i8(self) -> i8 {
        self as i8
    }
}

// ============================================================================
// Quest ID - 25 quest types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum QuestId {
    Rock = 0,
    Mushroom,
    Garbud,
    Zhar,
    Veil,
    Diablo,
    Butcher,
    LtBanner,
    Blind,
    Blood,
    Anvil,
    Warlord,
    SkelKing,
    PWater,
    Schamb,
    Betrayer,
    Grave,
    Farmer,
    Girl,
    Trader,
    Defiler,
    Nakrul,
    Cornstn,
    Jersey,
}

impl QuestId {
    pub const INVALID: i8 = -1;

    /// Convert from i8 to QuestId
    pub fn from_i8(value: i8) -> Option<Self> {
        if value >= 0 && value <= 23 {
            // SAFETY: value is in valid range [0, 23]
            Some(unsafe { std::mem::transmute(value) })
        } else {
            None
        }
    }

    /// Convert to i8
    pub fn to_i8(self) -> i8 {
        self as i8
    }
}

// ============================================================================
// Object Data Flags
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ObjectDataFlags(pub u8);

impl std::ops::BitOr for ObjectDataFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl ObjectDataFlags {
    pub const NONE: Self = Self(0);
    pub const ANIMATED: Self = Self(1 << 0);
    pub const SOLID: Self = Self(1 << 1);
    pub const MISSILES_PASS_THROUGH: Self = Self(1 << 2);
    pub const LIGHT: Self = Self(1 << 3);
    pub const TRAP: Self = Self(1 << 4);
    pub const BREAKABLE: Self = Self(1 << 5);

    /// Const bitwise-or, usable in `static` initializers (`|` via `BitOr`
    /// is not `const`).
    pub const fn or(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }

    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    pub const fn is_animated(&self) -> bool {
        self.contains(Self::ANIMATED)
    }

    pub const fn is_solid(&self) -> bool {
        self.contains(Self::SOLID)
    }

    pub const fn missiles_pass_through(&self) -> bool {
        self.contains(Self::MISSILES_PASS_THROUGH)
    }

    pub const fn apply_lighting(&self) -> bool {
        self.contains(Self::LIGHT)
    }

    pub const fn is_trap(&self) -> bool {
        self.contains(Self::TRAP)
    }

    pub const fn is_breakable(&self) -> bool {
        self.contains(Self::BREAKABLE)
    }
}

// ============================================================================
// Object Data Structure
// ============================================================================

#[derive(Debug, Clone)]
pub struct ObjectData {
    /// Index into object graphic list
    pub ofindex: u8,

    /// Minimum dungeon level
    pub minlvl: i8,

    /// Maximum dungeon level
    pub maxlvl: i8,

    /// Dungeon type this object appears in
    pub olvltype: DungeonType,

    /// Theme ID (or NONE)
    pub otheme: i8,

    /// Quest ID (or INVALID)
    pub oquest: i8,

    /// Object flags
    pub flags: ObjectDataFlags,

    /// Tick length of each frame in animation
    pub anim_delay: u8,

    /// Number of frames in animation
    pub anim_len: u8,

    /// Animation width
    pub anim_width: u8,

    /// Selection region flags (C++ SelectionRegion bitmask)
    pub selection_region: u8,
}

impl ObjectData {
    pub fn is_animated(&self) -> bool {
        self.flags.is_animated()
    }

    pub fn is_solid(&self) -> bool {
        self.flags.is_solid()
    }

    pub fn missiles_pass_through(&self) -> bool {
        self.flags.missiles_pass_through()
    }

    pub fn apply_lighting(&self) -> bool {
        self.flags.apply_lighting()
    }

    pub fn is_trap(&self) -> bool {
        self.flags.is_trap()
    }

    pub fn is_breakable(&self) -> bool {
        self.flags.is_breakable()
    }
}

// ============================================================================
// OBJ_TYPE_CONV - Maps from dun_object_id to ObjectId
// ============================================================================

/// Maps dungeon object IDs to ObjectId (156 entries)
pub static OBJ_TYPE_CONV: &[i8] = &[
    ObjectId::NULL, // 0
    ObjectId::Lever as i8,
    ObjectId::Crux1 as i8,
    ObjectId::Crux2 as i8,
    ObjectId::Crux3 as i8,
    ObjectId::Angel as i8,
    ObjectId::BannerL as i8,
    ObjectId::BannerM as i8,
    ObjectId::BannerR as i8,
    ObjectId::NULL,
    ObjectId::NULL, // 10
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::Book2L as i8,
    ObjectId::Book2R as i8,
    ObjectId::BCross as i8,
    ObjectId::NULL,
    ObjectId::Candle1 as i8,
    ObjectId::Candle2 as i8,
    ObjectId::CandleO as i8, // 20
    ObjectId::Cauldron as i8,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::FlameHole as i8, // 30
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::MCircle1 as i8,
    ObjectId::MCircle2 as i8,
    ObjectId::SkFire as i8,
    ObjectId::SkPile as i8,
    ObjectId::SkStick1 as i8, // 40
    ObjectId::SkStick2 as i8,
    ObjectId::SkStick3 as i8,
    ObjectId::SkStick4 as i8,
    ObjectId::SkStick5 as i8,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL, // 50
    ObjectId::SwitchSkl as i8,
    ObjectId::NULL,
    ObjectId::TrapL as i8,
    ObjectId::TrapR as i8,
    ObjectId::Torture1 as i8,
    ObjectId::Torture2 as i8,
    ObjectId::Torture3 as i8,
    ObjectId::Torture4 as i8,
    ObjectId::Torture5 as i8,
    ObjectId::NULL, // 60
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NudeW2R as i8,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::TNudeM1 as i8, // 70
    ObjectId::TNudeM2 as i8,
    ObjectId::TNudeM3 as i8,
    ObjectId::TNudeM4 as i8,
    ObjectId::TNudeW1 as i8,
    ObjectId::TNudeW2 as i8,
    ObjectId::TNudeW3 as i8,
    ObjectId::Chest1 as i8,
    ObjectId::Chest1 as i8,
    ObjectId::Chest1 as i8,
    ObjectId::Chest2 as i8, // 80
    ObjectId::Chest2 as i8,
    ObjectId::Chest2 as i8,
    ObjectId::Chest3 as i8,
    ObjectId::Chest3 as i8,
    ObjectId::Chest3 as i8,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL, // 90
    ObjectId::Pedestal as i8,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL, // 100
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::AltBoy as i8,
    ObjectId::NULL,
    ObjectId::NULL,
    ObjectId::WarArmor as i8,
    ObjectId::WarWeap as i8,
    ObjectId::TorchR2 as i8,
    ObjectId::TorchL2 as i8, // 110
    ObjectId::MushPatch as i8,
    ObjectId::Stand as i8,
    ObjectId::TorchL as i8,
    ObjectId::TorchR as i8,
    ObjectId::FlameLvr as i8,
    ObjectId::Sarc as i8,
    ObjectId::Barrel as i8,
    ObjectId::BarrelEx as i8,
    ObjectId::Bookshelf as i8,
    ObjectId::BookcaseL as i8, // 120
    ObjectId::BookcaseR as i8,
    ObjectId::ArmorStandN as i8,
    ObjectId::WeaponRackN as i8,
    ObjectId::BloodFtn as i8,
    ObjectId::PurifyingFtn as i8,
    ObjectId::ShrineL as i8,
    ObjectId::ShrineR as i8,
    ObjectId::GoatShrine as i8,
    ObjectId::MurkyFtn as i8,
    ObjectId::TearFtn as i8, // 130
    ObjectId::Decap as i8,
    ObjectId::TChest1 as i8,
    ObjectId::TChest2 as i8,
    ObjectId::TChest3 as i8,
    ObjectId::LazStand as i8,
    ObjectId::Bookstand as i8,
    ObjectId::BookshelfR as i8,
    ObjectId::Pod as i8,
    ObjectId::PodEx as i8,
    ObjectId::Urn as i8, // 140
    ObjectId::UrnEx as i8,
    ObjectId::L5Books as i8,
    ObjectId::L5Candle as i8,
    ObjectId::L5Lever as i8,
    ObjectId::L5Sarc as i8,
];

// ============================================================================
// Helper Functions
// ============================================================================

/// Get object data by ObjectId. Mirrors C++ `ObjData[id]` / `AllObjects[id]`.
/// Returns `None` only for the sentinel `ObjectId::Null` (-1); every valid
/// discriminant in [0, 108] is guaranteed a table entry.
pub fn get_object_data(obj_id: ObjectId) -> Option<&'static ObjectData> {
    if obj_id == ObjectId::Null {
        return None;
    }
    ALL_OBJECTS.get(obj_id as usize)
}

/// Direct indexed accessor mirroring C++ `AllObjects[id]`. Panics on
/// out-of-range / `ObjectId::Null`. Equivalent to C++'s unchecked indexing
/// into `AllObjects` after the `OBJ_LAST+1 == size()` assert.
pub fn obj_data(obj_id: ObjectId) -> &'static ObjectData {
    &ALL_OBJECTS[obj_id as usize]
}

/// Check if object is a shrine
pub fn is_shrine(obj_id: ObjectId) -> bool {
    matches!(obj_id, ObjectId::ShrineL | ObjectId::ShrineR | ObjectId::GoatShrine)
}

/// Check if object is a chest
pub fn is_chest(obj_id: ObjectId) -> bool {
    matches!(
        obj_id,
        ObjectId::Chest1
            | ObjectId::Chest2
            | ObjectId::Chest3
            | ObjectId::TChest1
            | ObjectId::TChest2
            | ObjectId::TChest3
            | ObjectId::SignChest
    )
}

/// Check if object is a door
pub fn is_door(obj_id: ObjectId) -> bool {
    matches!(
        obj_id,
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

/// Check if object is breakable
pub fn is_breakable(obj_id: ObjectId) -> bool {
    matches!(
        obj_id,
        ObjectId::Barrel
            | ObjectId::BarrelEx
            | ObjectId::Pod
            | ObjectId::PodEx
            | ObjectId::Urn
            | ObjectId::UrnEx
    )
}

// ============================================================================
// ALL_OBJECTS - Complete object data table (109 objects)
// ============================================================================
//
// This table is a direct port of the data in
// `assets/txtdata/objects/objdat.tsv`, which is what the C++ runtime loads
// into `AllObjects` via `LoadObjectData()` (Source/objdat.cpp). The row order
// matches the ObjectId discriminant order (OBJ_L1LIGHT=0 .. OBJ_L5SARC=108),
// and the C++ code asserts `OBJ_LAST + 1 == AllObjects.size()`.
//
// Field semantics (matching the TSV parser in objdat.cpp):
//   ofindex        - Index into ObjMasterLoadList, assigned by first-seen
//                    dedup of the `file` column (0..61 here, 62 unique gfx).
//   minlvl/maxlvl  - Parsed as int8; on objects these gate which dungeon
//                    level the type may spawn on.
//   olvltype       - `levelType` column; empty => DTYPE_NONE => DungeonType::None.
//   otheme         - `theme` column; empty => THEME_NONE (-1). Stored as raw
//                    i8 to mirror the C++ `theme_id otheme` field (which can
//                    hold THEME_NONE).
//   oquest         - `quest` column; empty => Q_INVALID (-1). Stored as raw i8.
//   flags          - `flags` column, parsed as an enum-flag list (comma sep).
//   anim_delay     - `animDelay` (u8). Tick length of each animation frame.
//   anim_len       - `animLen` (u8). Number of frames in current animation.
//   anim_width     - `animWidth` (u8).
//   selection_region - `selectionRegion`; C++ reads it as an enum-flag list
//                    but every row in the TSV is either empty (=> None) or a
//                    single value, so a plain SelectionRegion is exact.

/// Complete object data table — 109 entries, indexed by ObjectId.
pub static ALL_OBJECTS: &[ObjectData] = &[
    // 0  OBJ_L1LIGHT  (l1braz)  Animated,Solid,MissilesPassThrough
    ObjectData { ofindex: 0, minlvl: 0, maxlvl: 0, olvltype: DungeonType::Cathedral, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::SOLID).or(ObjectDataFlags::MISSILES_PASS_THROUGH), anim_delay: 1, anim_len: 26, anim_width: 64, selection_region: SEL_NONE },
    // 1  OBJ_L1LDOOR  (l1doors)  Light,Trap  Bottom,Middle
    ObjectData { ofindex: 1, minlvl: 0, maxlvl: 0, olvltype: DungeonType::Cathedral, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::LIGHT.or(ObjectDataFlags::TRAP), anim_delay: 1, anim_len: 0, anim_width: 64, selection_region: SEL_BOTTOM_MIDDLE },
    // 2  OBJ_L1RDOOR  (l1doors)  Light,Trap  Bottom,Middle
    ObjectData { ofindex: 1, minlvl: 0, maxlvl: 0, olvltype: DungeonType::Cathedral, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::LIGHT.or(ObjectDataFlags::TRAP), anim_delay: 2, anim_len: 0, anim_width: 64, selection_region: SEL_BOTTOM_MIDDLE },
    // 3  OBJ_SKFIRE  (skulfire)  Animated,Solid,MissilesPassThrough  THEME_SKELROOM
    ObjectData { ofindex: 2, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::SkelRoom as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::SOLID).or(ObjectDataFlags::MISSILES_PASS_THROUGH), anim_delay: 2, anim_len: 11, anim_width: 96, selection_region: SEL_NONE },
    // 4  OBJ_LEVER  (lever)  Solid,MissilesPassThrough,Light,Trap  Bottom
    ObjectData { ofindex: 3, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::TRAP), anim_delay: 1, anim_len: 1, anim_width: 96, selection_region: SEL_BOTTOM },
    // 5  OBJ_CHEST1  (chest1)  Solid,MissilesPassThrough,Light,Trap  Bottom
    ObjectData { ofindex: 4, minlvl: 1, maxlvl: 24, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::TRAP), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM },
    // 6  OBJ_CHEST2  (chest2)  Solid,MissilesPassThrough,Light,Trap  Bottom
    ObjectData { ofindex: 5, minlvl: 1, maxlvl: 24, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::TRAP), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM },
    // 7  OBJ_CHEST3  (chest3)  Solid,MissilesPassThrough,Light,Trap  Bottom
    ObjectData { ofindex: 6, minlvl: 1, maxlvl: 24, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::TRAP), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM },
    // 8  OBJ_CANDLE1  (l1braz)  no flags
    ObjectData { ofindex: 0, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::NONE, anim_delay: 0, anim_len: 0, anim_width: 0, selection_region: SEL_NONE },
    // 9  OBJ_CANDLE2  (candle2)  Animated,Solid,MissilesPassThrough,Light  THEME_SHRINE Q_PWATER
    ObjectData { ofindex: 7, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::Shrine as i8, oquest: QuestId::PWater as i8, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::SOLID).or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 4, anim_width: 96, selection_region: SEL_NONE },
    // 10  OBJ_CANDLEO  (l1braz)  no flags
    ObjectData { ofindex: 0, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::NONE, anim_delay: 0, anim_len: 0, anim_width: 0, selection_region: SEL_NONE },
    // 11  OBJ_BANNERL  (banner)  Solid,MissilesPassThrough,Light  THEME_SKELROOM
    ObjectData { ofindex: 8, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::SkelRoom as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 0, anim_width: 96, selection_region: SEL_NONE },
    // 12  OBJ_BANNERM  (banner)  Solid,MissilesPassThrough,Light  THEME_SKELROOM
    ObjectData { ofindex: 8, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::SkelRoom as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_NONE },
    // 13  OBJ_BANNERR  (banner)  Solid,MissilesPassThrough,Light  THEME_SKELROOM
    ObjectData { ofindex: 8, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::SkelRoom as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 3, anim_len: 0, anim_width: 96, selection_region: SEL_NONE },
    // 14  OBJ_SKPILE  (skulpile)  Solid,MissilesPassThrough,Light
    ObjectData { ofindex: 9, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 1, anim_width: 96, selection_region: SEL_NONE },
    // 15  OBJ_SKSTICK1  (l1braz)  no flags
    ObjectData { ofindex: 0, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::NONE, anim_delay: 0, anim_len: 0, anim_width: 0, selection_region: SEL_NONE },
    // 16  OBJ_SKSTICK2  (l1braz)  no flags
    ObjectData { ofindex: 0, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::NONE, anim_delay: 0, anim_len: 0, anim_width: 0, selection_region: SEL_NONE },
    // 17  OBJ_SKSTICK3  (l1braz)  no flags
    ObjectData { ofindex: 0, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::NONE, anim_delay: 0, anim_len: 0, anim_width: 0, selection_region: SEL_NONE },
    // 18  OBJ_SKSTICK4  (l1braz)  no flags
    ObjectData { ofindex: 0, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::NONE, anim_delay: 0, anim_len: 0, anim_width: 0, selection_region: SEL_NONE },
    // 19  OBJ_SKSTICK5  (l1braz)  no flags
    ObjectData { ofindex: 0, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::NONE, anim_delay: 0, anim_len: 0, anim_width: 0, selection_region: SEL_NONE },
    // 20  OBJ_CRUX1  (cruxsk1)  Solid,Light,Breakable  Bottom,Middle
    ObjectData { ofindex: 10, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::BREAKABLE), anim_delay: 1, anim_len: 15, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 21  OBJ_CRUX2  (cruxsk2)  Solid,Light,Breakable  Bottom,Middle
    ObjectData { ofindex: 11, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::BREAKABLE), anim_delay: 1, anim_len: 15, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 22  OBJ_CRUX3  (cruxsk3)  Solid,Light,Breakable  Bottom,Middle
    ObjectData { ofindex: 12, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::BREAKABLE), anim_delay: 1, anim_len: 15, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 23  OBJ_STAND  (rockstan)  Solid,MissilesPassThrough,Light
    ObjectData { ofindex: 13, minlvl: 5, maxlvl: 5, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_NONE },
    // 24  OBJ_ANGEL  (angel)  Solid,Light
    ObjectData { ofindex: 14, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_NONE },
    // 25  OBJ_BOOK2L  (book2)  Solid,MissilesPassThrough,Light  Bottom,Middle
    ObjectData { ofindex: 15, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 26  OBJ_BCROSS  (burncros)  Animated,Solid
    ObjectData { ofindex: 16, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::SOLID), anim_delay: 0, anim_len: 10, anim_width: 160, selection_region: SEL_NONE },
    // 27  OBJ_NUDEW2R  (nude2)  Animated,Solid,Light
    ObjectData { ofindex: 17, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::SOLID).or(ObjectDataFlags::LIGHT), anim_delay: 3, anim_len: 6, anim_width: 128, selection_region: SEL_NONE },
    // 28  OBJ_SWITCHSKL  (switch4)  Solid,MissilesPassThrough,Light,Trap  Bottom
    ObjectData { ofindex: 18, minlvl: 16, maxlvl: 16, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::TRAP), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM },
    // 29  OBJ_TNUDEM1  (tnudem)  Solid,Light  Q_BUTCHER
    ObjectData { ofindex: 19, minlvl: 13, maxlvl: 15, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Butcher as i8, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 128, selection_region: SEL_NONE },
    // 30  OBJ_TNUDEM2  (tnudem)  Solid,Light  THEME_TORTURE Q_BUTCHER
    ObjectData { ofindex: 19, minlvl: 13, maxlvl: 15, olvltype: DungeonType::None, otheme: ThemeId::Torture as i8, oquest: QuestId::Butcher as i8, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 0, anim_width: 128, selection_region: SEL_NONE },
    // 31  OBJ_TNUDEM3  (tnudem)  Solid,Light  THEME_TORTURE Q_BUTCHER
    ObjectData { ofindex: 19, minlvl: 13, maxlvl: 15, olvltype: DungeonType::None, otheme: ThemeId::Torture as i8, oquest: QuestId::Butcher as i8, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 3, anim_len: 0, anim_width: 128, selection_region: SEL_NONE },
    // 32  OBJ_TNUDEM4  (tnudem)  Solid,Light  THEME_TORTURE Q_BUTCHER
    ObjectData { ofindex: 19, minlvl: 13, maxlvl: 15, olvltype: DungeonType::None, otheme: ThemeId::Torture as i8, oquest: QuestId::Butcher as i8, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 4, anim_len: 0, anim_width: 128, selection_region: SEL_NONE },
    // 33  OBJ_TNUDEW1  (tnudew)  Solid,Light  THEME_TORTURE Q_BUTCHER
    ObjectData { ofindex: 20, minlvl: 13, maxlvl: 15, olvltype: DungeonType::None, otheme: ThemeId::Torture as i8, oquest: QuestId::Butcher as i8, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 128, selection_region: SEL_NONE },
    // 34  OBJ_TNUDEW2  (tnudew)  Solid,Light  THEME_TORTURE Q_BUTCHER
    ObjectData { ofindex: 20, minlvl: 13, maxlvl: 15, olvltype: DungeonType::None, otheme: ThemeId::Torture as i8, oquest: QuestId::Butcher as i8, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 0, anim_width: 128, selection_region: SEL_NONE },
    // 35  OBJ_TNUDEW3  (tnudew)  Solid,Light  THEME_TORTURE Q_BUTCHER
    ObjectData { ofindex: 20, minlvl: 13, maxlvl: 15, olvltype: DungeonType::None, otheme: ThemeId::Torture as i8, oquest: QuestId::Butcher as i8, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 3, anim_len: 0, anim_width: 128, selection_region: SEL_NONE },
    // 36  OBJ_TORTURE1  (tsoul)  MissilesPassThrough,Light  Q_BUTCHER
    ObjectData { ofindex: 21, minlvl: 13, maxlvl: 15, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Butcher as i8, flags: ObjectDataFlags::MISSILES_PASS_THROUGH.or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 128, selection_region: SEL_NONE },
    // 37  OBJ_TORTURE2  (tsoul)  MissilesPassThrough,Light  Q_BUTCHER
    ObjectData { ofindex: 21, minlvl: 13, maxlvl: 15, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Butcher as i8, flags: ObjectDataFlags::MISSILES_PASS_THROUGH.or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 0, anim_width: 128, selection_region: SEL_NONE },
    // 38  OBJ_TORTURE3  (tsoul)  MissilesPassThrough,Light  Q_BUTCHER
    ObjectData { ofindex: 21, minlvl: 13, maxlvl: 15, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Butcher as i8, flags: ObjectDataFlags::MISSILES_PASS_THROUGH.or(ObjectDataFlags::LIGHT), anim_delay: 3, anim_len: 0, anim_width: 128, selection_region: SEL_NONE },
    // 39  OBJ_TORTURE4  (tsoul)  MissilesPassThrough,Light  Q_BUTCHER
    ObjectData { ofindex: 21, minlvl: 13, maxlvl: 15, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Butcher as i8, flags: ObjectDataFlags::MISSILES_PASS_THROUGH.or(ObjectDataFlags::LIGHT), anim_delay: 4, anim_len: 0, anim_width: 128, selection_region: SEL_NONE },
    // 40  OBJ_TORTURE5  (tsoul)  MissilesPassThrough,Light  Q_BUTCHER
    ObjectData { ofindex: 21, minlvl: 13, maxlvl: 15, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Butcher as i8, flags: ObjectDataFlags::MISSILES_PASS_THROUGH.or(ObjectDataFlags::LIGHT), anim_delay: 5, anim_len: 0, anim_width: 128, selection_region: SEL_NONE },
    // 41  OBJ_BOOK2R  (book2)  Solid,MissilesPassThrough,Light  Bottom,Middle
    ObjectData { ofindex: 15, minlvl: 6, maxlvl: 6, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 4, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 42  OBJ_L2LDOOR  (l2doors)  Light,Trap  DTYPE_CATACOMBS  Bottom,Middle
    ObjectData { ofindex: 22, minlvl: 0, maxlvl: 0, olvltype: DungeonType::Catacombs, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::LIGHT.or(ObjectDataFlags::TRAP), anim_delay: 1, anim_len: 0, anim_width: 64, selection_region: SEL_BOTTOM_MIDDLE },
    // 43  OBJ_L2RDOOR  (l2doors)  Light,Trap  DTYPE_CATACOMBS  Bottom,Middle
    ObjectData { ofindex: 22, minlvl: 0, maxlvl: 0, olvltype: DungeonType::Catacombs, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::LIGHT.or(ObjectDataFlags::TRAP), anim_delay: 2, anim_len: 0, anim_width: 64, selection_region: SEL_BOTTOM_MIDDLE },
    // 44  OBJ_TORCHL  (wtorch4)  Animated,MissilesPassThrough
    ObjectData { ofindex: 23, minlvl: 5, maxlvl: 8, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::MISSILES_PASS_THROUGH), anim_delay: 1, anim_len: 9, anim_width: 96, selection_region: SEL_NONE },
    // 45  OBJ_TORCHR  (wtorch3)  Animated,MissilesPassThrough
    ObjectData { ofindex: 24, minlvl: 5, maxlvl: 8, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::MISSILES_PASS_THROUGH), anim_delay: 1, anim_len: 9, anim_width: 96, selection_region: SEL_NONE },
    // 46  OBJ_TORCHL2  (wtorch1)  Animated,MissilesPassThrough
    ObjectData { ofindex: 25, minlvl: 5, maxlvl: 8, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::MISSILES_PASS_THROUGH), anim_delay: 1, anim_len: 9, anim_width: 96, selection_region: SEL_NONE },
    // 47  OBJ_TORCHR2  (wtorch2)  Animated,MissilesPassThrough
    ObjectData { ofindex: 26, minlvl: 5, maxlvl: 8, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::MISSILES_PASS_THROUGH), anim_delay: 1, anim_len: 9, anim_width: 96, selection_region: SEL_NONE },
    // 48  OBJ_SARC  (sarc)  Solid,MissilesPassThrough,Light,Trap  Bottom,Middle
    ObjectData { ofindex: 27, minlvl: 1, maxlvl: 4, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::TRAP), anim_delay: 1, anim_len: 5, anim_width: 128, selection_region: SEL_BOTTOM_MIDDLE },
    // 49  OBJ_FLAMEHOLE  (flame1)  MissilesPassThrough,Light
    ObjectData { ofindex: 28, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::MISSILES_PASS_THROUGH.or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 20, anim_width: 96, selection_region: SEL_NONE },
    // 50  OBJ_FLAMELVR  (lever)  Solid,MissilesPassThrough,Light,Trap  Bottom
    ObjectData { ofindex: 3, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::TRAP), anim_delay: 1, anim_len: 2, anim_width: 96, selection_region: SEL_BOTTOM },
    // 51  OBJ_WATER  (miniwatr)  Animated,Solid,Light
    ObjectData { ofindex: 29, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::SOLID).or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 10, anim_width: 64, selection_region: SEL_NONE },
    // 52  OBJ_BOOKLVR  (book1)  Solid,MissilesPassThrough,Light  Bottom,Middle
    ObjectData { ofindex: 30, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 53  OBJ_TRAPL  (traphole)  MissilesPassThrough,Light
    ObjectData { ofindex: 31, minlvl: 1, maxlvl: 24, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::MISSILES_PASS_THROUGH.or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 64, selection_region: SEL_NONE },
    // 54  OBJ_TRAPR  (traphole)  MissilesPassThrough,Light
    ObjectData { ofindex: 31, minlvl: 1, maxlvl: 24, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::MISSILES_PASS_THROUGH.or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 0, anim_width: 64, selection_region: SEL_NONE },
    // 55  OBJ_BOOKSHELF  (bcase)  Solid,Light
    ObjectData { ofindex: 32, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_NONE },
    // 56  OBJ_WEAPRACK  (weapstnd)  Solid,Light
    ObjectData { ofindex: 33, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_NONE },
    // 57  OBJ_BARREL  (barrel)  Solid,MissilesPassThrough,Light,Breakable  Bottom,Middle
    ObjectData { ofindex: 34, minlvl: 1, maxlvl: 15, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::BREAKABLE), anim_delay: 1, anim_len: 9, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 58  OBJ_BARRELEX  (barrelex)  Solid,MissilesPassThrough,Light,Breakable  Bottom,Middle
    ObjectData { ofindex: 35, minlvl: 1, maxlvl: 15, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::BREAKABLE), anim_delay: 1, anim_len: 10, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 59  OBJ_SHRINEL  (lshrineg)  Light  THEME_SHRINE  Bottom,Middle
    ObjectData { ofindex: 36, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::Shrine as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::LIGHT, anim_delay: 1, anim_len: 11, anim_width: 128, selection_region: SEL_BOTTOM_MIDDLE },
    // 60  OBJ_SHRINER  (rshrineg)  Light  THEME_SHRINE  Bottom,Middle
    ObjectData { ofindex: 37, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::Shrine as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::LIGHT, anim_delay: 1, anim_len: 11, anim_width: 128, selection_region: SEL_BOTTOM_MIDDLE },
    // 61  OBJ_SKELBOOK  (book2)  Solid,MissilesPassThrough,Light  THEME_SKELROOM  Bottom,Middle
    ObjectData { ofindex: 15, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::SkelRoom as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 4, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 62  OBJ_BOOKCASEL  (bcase)  Solid,Light  THEME_LIBRARY  Bottom,Middle
    ObjectData { ofindex: 32, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::Library as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 3, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 63  OBJ_BOOKCASER  (bcase)  Solid,Light  THEME_LIBRARY  Bottom,Middle
    ObjectData { ofindex: 32, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::Library as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 4, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 64  OBJ_BOOKSTAND  (book2)  Solid,MissilesPassThrough,Light  THEME_LIBRARY  Bottom,Middle
    ObjectData { ofindex: 15, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::Library as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 65  OBJ_BOOKCANDLE  (candle2)  Animated,Solid,MissilesPassThrough,Light  THEME_LIBRARY
    ObjectData { ofindex: 7, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::Library as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::SOLID).or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 4, anim_width: 96, selection_region: SEL_NONE },
    // 66  OBJ_BLOODFTN  (bloodfnt)  Animated,Solid,MissilesPassThrough,Light  THEME_BLOODFOUNTAIN  Bottom,Middle
    ObjectData { ofindex: 38, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::BloodFountain as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::SOLID).or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 10, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 67  OBJ_DECAP  (decap)  Solid,MissilesPassThrough,Light  THEME_DECAPITATED  Bottom
    ObjectData { ofindex: 39, minlvl: 13, maxlvl: 15, olvltype: DungeonType::None, otheme: ThemeId::Decapitated as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM },
    // 68  OBJ_TCHEST1  (chest1)  Solid,MissilesPassThrough,Light,Trap  Bottom
    ObjectData { ofindex: 4, minlvl: 1, maxlvl: 24, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::TRAP), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM },
    // 69  OBJ_TCHEST2  (chest2)  Solid,MissilesPassThrough,Light,Trap  Bottom
    ObjectData { ofindex: 5, minlvl: 1, maxlvl: 24, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::TRAP), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM },
    // 70  OBJ_TCHEST3  (chest3)  Solid,MissilesPassThrough,Light,Trap  Bottom
    ObjectData { ofindex: 6, minlvl: 1, maxlvl: 24, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::TRAP), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM },
    // 71  OBJ_BLINDBOOK  (book1)  Solid,MissilesPassThrough,Light  Q_BLIND  Bottom,Middle
    ObjectData { ofindex: 30, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Blind as i8, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 72  OBJ_BLOODBOOK  (book1)  Solid,MissilesPassThrough,Light  Q_BLOOD  Bottom,Middle
    ObjectData { ofindex: 30, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Blood as i8, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 4, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 73  OBJ_PEDESTAL  (pedistl)  Solid,MissilesPassThrough,Light  Q_BLOOD  Bottom,Middle
    ObjectData { ofindex: 40, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Blood as i8, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 74  OBJ_L3LDOOR  (l3doors)  Light,Trap  DTYPE_CAVES  Bottom,Middle
    ObjectData { ofindex: 41, minlvl: 0, maxlvl: 0, olvltype: DungeonType::Caves, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::LIGHT.or(ObjectDataFlags::TRAP), anim_delay: 1, anim_len: 0, anim_width: 64, selection_region: SEL_BOTTOM_MIDDLE },
    // 75  OBJ_L3RDOOR  (l3doors)  Light,Trap  DTYPE_CAVES  Bottom,Middle
    ObjectData { ofindex: 41, minlvl: 0, maxlvl: 0, olvltype: DungeonType::Caves, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::LIGHT.or(ObjectDataFlags::TRAP), anim_delay: 2, anim_len: 0, anim_width: 64, selection_region: SEL_BOTTOM_MIDDLE },
    // 76  OBJ_PURIFYINGFTN  (pfountn)  Animated,Solid,MissilesPassThrough,Light  THEME_PURIFYINGFOUNTAIN  Bottom,Middle
    ObjectData { ofindex: 42, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::PurifyingFountain as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::SOLID).or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 10, anim_width: 128, selection_region: SEL_BOTTOM_MIDDLE },
    // 77  OBJ_ARMORSTAND  (armstand)  Solid,Light  THEME_ARMORSTAND  Bottom,Middle
    ObjectData { ofindex: 43, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::ArmorStand as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 78  OBJ_ARMORSTANDN  (armstand)  Solid,Light  THEME_ARMORSTAND
    ObjectData { ofindex: 43, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::ArmorStand as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 0, anim_width: 96, selection_region: SEL_NONE },
    // 79  OBJ_GOATSHRINE  (goatshrn)  Animated,Solid,MissilesPassThrough,Light  THEME_GOATSHRINE  Bottom,Middle
    ObjectData { ofindex: 44, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::GoatShrine as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::SOLID).or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 10, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 80  OBJ_CAULDRON  (cauldren)  Solid,Light  Bottom,Middle
    ObjectData { ofindex: 45, minlvl: 13, maxlvl: 15, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 81  OBJ_MURKYFTN  (mfountn)  Animated,Solid,MissilesPassThrough,Light  THEME_MURKYFOUNTAIN  Bottom,Middle
    ObjectData { ofindex: 46, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::MurkyFountain as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::SOLID).or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 10, anim_width: 128, selection_region: SEL_BOTTOM_MIDDLE },
    // 82  OBJ_TEARFTN  (tfountn)  Animated,Solid,MissilesPassThrough,Light  THEME_TEARFOUNTAIN  Bottom,Middle
    ObjectData { ofindex: 47, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::TearFountain as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::SOLID).or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 4, anim_width: 128, selection_region: SEL_BOTTOM_MIDDLE },
    // 83  OBJ_ALTBOY  (altboy)  Solid,MissilesPassThrough,Light  Q_BETRAYER
    ObjectData { ofindex: 48, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Betrayer as i8, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 128, selection_region: SEL_NONE },
    // 84  OBJ_MCIRCLE1  (mcirl)  MissilesPassThrough,Light  Q_BETRAYER
    ObjectData { ofindex: 49, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Betrayer as i8, flags: ObjectDataFlags::MISSILES_PASS_THROUGH.or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_NONE },
    // 85  OBJ_MCIRCLE2  (mcirl)  MissilesPassThrough,Light  Q_BETRAYER
    ObjectData { ofindex: 49, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Betrayer as i8, flags: ObjectDataFlags::MISSILES_PASS_THROUGH.or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_NONE },
    // 86  OBJ_STORYBOOK  (bkslbrnt)  Solid,MissilesPassThrough,Light  Bottom,Middle
    ObjectData { ofindex: 50, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 87  OBJ_STORYCANDLE  (candle2)  Animated,Solid,MissilesPassThrough,Light  Q_BETRAYER
    ObjectData { ofindex: 7, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Betrayer as i8, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::SOLID).or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 4, anim_width: 96, selection_region: SEL_NONE },
    // 88  OBJ_STEELTOME  (book1)  Solid,MissilesPassThrough,Light  Q_WARLORD  Bottom,Middle
    ObjectData { ofindex: 30, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Warlord as i8, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 4, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 89  OBJ_WARARMOR  (armstand)  Solid,Light  Q_WARLORD  Bottom,Middle
    ObjectData { ofindex: 43, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Warlord as i8, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 90  OBJ_WARWEAP  (weapstnd)  Solid,Light  Q_WARLORD  Bottom,Middle
    ObjectData { ofindex: 33, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Warlord as i8, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 91  OBJ_TBCROSS  (burncros)  Animated,Solid  THEME_BRNCROSS
    ObjectData { ofindex: 16, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::BrnCross as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::SOLID), anim_delay: 0, anim_len: 10, anim_width: 160, selection_region: SEL_NONE },
    // 92  OBJ_WEAPONRACK  (weapstnd)  Solid,Light  THEME_WEAPONRACK  Bottom,Middle
    ObjectData { ofindex: 33, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::WeaponRack as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 93  OBJ_WEAPONRACKN  (weapstnd)  Solid,Light  THEME_WEAPONRACK
    ObjectData { ofindex: 33, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::WeaponRack as i8, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 0, anim_width: 96, selection_region: SEL_NONE },
    // 94  OBJ_MUSHPATCH  (mushptch)  Solid,MissilesPassThrough,Light,Trap  Q_MUSHROOM  Bottom,Middle
    ObjectData { ofindex: 51, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Mushroom as i8, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::TRAP), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 95  OBJ_LAZSTAND  (lzstand)  Solid,Light  Q_BETRAYER  Bottom,Middle
    ObjectData { ofindex: 52, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::Betrayer as i8, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 128, selection_region: SEL_BOTTOM_MIDDLE },
    // 96  OBJ_SLAINHERO  (decap)  Solid,MissilesPassThrough,Light  Bottom
    ObjectData { ofindex: 39, minlvl: 9, maxlvl: 9, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM },
    // 97  OBJ_SIGNCHEST  (chest3)  Solid,MissilesPassThrough,Light  Bottom
    ObjectData { ofindex: 6, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM },
    // 98  OBJ_BOOKSHELFR  (bcase)  Solid,Light
    ObjectData { ofindex: 32, minlvl: 0, maxlvl: 0, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 0, anim_width: 96, selection_region: SEL_NONE },
    // 99  OBJ_POD  (l6pod1)  Solid,MissilesPassThrough,Light,Breakable  Bottom,Middle
    ObjectData { ofindex: 53, minlvl: 17, maxlvl: 20, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::BREAKABLE), anim_delay: 1, anim_len: 9, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 100  OBJ_PODEX  (l6pod2)  Solid,MissilesPassThrough,Light,Breakable  Bottom,Middle
    ObjectData { ofindex: 54, minlvl: 17, maxlvl: 20, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::BREAKABLE), anim_delay: 1, anim_len: 10, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 101  OBJ_URN  (urn)  Solid,MissilesPassThrough,Light,Breakable  Bottom,Middle
    ObjectData { ofindex: 55, minlvl: 21, maxlvl: 24, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::BREAKABLE), anim_delay: 1, anim_len: 9, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 102  OBJ_URNEX  (urnexpld)  Solid,MissilesPassThrough,Light,Breakable  Bottom,Middle
    ObjectData { ofindex: 56, minlvl: 21, maxlvl: 24, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::BREAKABLE), anim_delay: 1, anim_len: 10, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 103  OBJ_L5BOOKS  (l5books)  Solid,MissilesPassThrough,Light  Bottom,Middle
    ObjectData { ofindex: 57, minlvl: 21, maxlvl: 24, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 1, anim_len: 0, anim_width: 96, selection_region: SEL_BOTTOM_MIDDLE },
    // 104  OBJ_L5CANDLE  (l5light)  Animated,Solid,MissilesPassThrough,Light
    ObjectData { ofindex: 58, minlvl: 21, maxlvl: 23, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::ANIMATED.or(ObjectDataFlags::SOLID).or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT), anim_delay: 2, anim_len: 4, anim_width: 96, selection_region: SEL_NONE },
    // 105  OBJ_L5LDOOR  (l5door)  Light,Trap  DTYPE_CRYPT  Bottom,Middle
    ObjectData { ofindex: 59, minlvl: 0, maxlvl: 0, olvltype: DungeonType::Crypt, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::LIGHT.or(ObjectDataFlags::TRAP), anim_delay: 1, anim_len: 0, anim_width: 64, selection_region: SEL_BOTTOM_MIDDLE },
    // 106  OBJ_L5RDOOR  (l5door)  Light,Trap  DTYPE_CRYPT  Bottom,Middle
    ObjectData { ofindex: 59, minlvl: 0, maxlvl: 0, olvltype: DungeonType::Crypt, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::LIGHT.or(ObjectDataFlags::TRAP), anim_delay: 2, anim_len: 0, anim_width: 64, selection_region: SEL_BOTTOM_MIDDLE },
    // 107  OBJ_L5LEVER  (l5lever)  Solid,MissilesPassThrough,Light,Trap  Bottom
    ObjectData { ofindex: 60, minlvl: 24, maxlvl: 24, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::TRAP), anim_delay: 1, anim_len: 1, anim_width: 96, selection_region: SEL_BOTTOM },
    // 108  OBJ_L5SARC  (l5sarco)  Solid,MissilesPassThrough,Light,Trap  Bottom,Middle
    ObjectData { ofindex: 61, minlvl: 21, maxlvl: 24, olvltype: DungeonType::None, otheme: ThemeId::NONE, oquest: QuestId::INVALID, flags: ObjectDataFlags::SOLID.or(ObjectDataFlags::MISSILES_PASS_THROUGH).or(ObjectDataFlags::LIGHT).or(ObjectDataFlags::TRAP), anim_delay: 1, anim_len: 5, anim_width: 128, selection_region: SEL_BOTTOM_MIDDLE },
];

/// `Bottom,Middle` selection region. The C++ TSV parser reads
/// `selectionRegion` as an enum-flag list; in the upstream data the only
/// multi-value entry is the combination `Bottom,Middle`, so we collapse it
/// into the `Middle` discriminant (matching how callers treat the door/book
/// hit region). Single values and empty cells map directly.
/// Mirror of the C++ `ObjMasterLoadList`: the deduplicated object graphic
/// filenames in first-seen order. `ALL_OBJECTS[i].ofindex` indexes into this.
pub static OBJ_MASTER_LOAD_LIST: &[&str] = &[
    "l1braz",     // 0
    "l1doors",    // 1
    "skulfire",   // 2
    "lever",      // 3
    "chest1",     // 4
    "chest2",     // 5
    "chest3",     // 6
    "candle2",    // 7
    "banner",     // 8
    "skulpile",   // 9
    "cruxsk1",    // 10
    "cruxsk2",    // 11
    "cruxsk3",    // 12
    "rockstan",   // 13
    "angel",      // 14
    "book2",      // 15
    "burncros",   // 16
    "nude2",      // 17
    "switch4",    // 18
    "tnudem",     // 19
    "tnudew",     // 20
    "tsoul",      // 21
    "l2doors",    // 22
    "wtorch4",    // 23
    "wtorch3",    // 24
    "wtorch1",    // 25
    "wtorch2",    // 26
    "sarc",       // 27
    "flame1",     // 28
    "miniwatr",   // 29
    "book1",      // 30
    "traphole",   // 31
    "bcase",      // 32
    "weapstnd",   // 33
    "barrel",     // 34
    "barrelex",   // 35
    "lshrineg",   // 36
    "rshrineg",   // 37
    "bloodfnt",   // 38
    "decap",      // 39
    "pedistl",    // 40
    "l3doors",    // 41
    "pfountn",    // 42
    "armstand",   // 43
    "goatshrn",   // 44
    "cauldren",   // 45
    "mfountn",    // 46
    "tfountn",    // 47
    "altboy",     // 48
    "mcirl",      // 49
    "bkslbrnt",   // 50
    "mushptch",   // 51
    "lzstand",    // 52
    "l6pod1",     // 53
    "l6pod2",     // 54
    "urn",        // 55
    "urnexpld",   // 56
    "l5books",    // 57
    "l5light",    // 58
    "l5door",     // 59
    "l5lever",    // 60
    "l5sarco",    // 61
];

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_id_count() {
        assert_eq!(ObjectId::COUNT, 109);
        assert_eq!(ObjectId::LAST.to_i8(), 108);  // L5Sarc is the 109th element (index 108)
    }

    #[test]
    fn test_object_id_conversion() {
        assert_eq!(ObjectId::from_i8(0), Some(ObjectId::L1Light));
        assert_eq!(ObjectId::from_i8(108), Some(ObjectId::L5Sarc));  // Last valid index
        assert_eq!(ObjectId::from_i8(-1), None);
        assert_eq!(ObjectId::from_i8(109), None);  // Beyond last valid index

        assert_eq!(ObjectId::L1Light.to_i8(), 0);
        assert_eq!(ObjectId::L5Sarc.to_i8(), 108);  // Index 108
    }

    #[test]
    fn test_theme_id_conversion() {
        assert_eq!(ThemeId::from_i8(0), Some(ThemeId::Barrel));
        assert_eq!(ThemeId::from_i8(16), Some(ThemeId::WeaponRack));
        assert_eq!(ThemeId::from_i8(-1), None);
        assert_eq!(ThemeId::from_i8(17), None);

        assert_eq!(ThemeId::Barrel.to_i8(), 0);
        assert_eq!(ThemeId::WeaponRack.to_i8(), 16);
    }

    #[test]
    fn test_quest_id_conversion() {
        assert_eq!(QuestId::from_i8(0), Some(QuestId::Rock));
        assert_eq!(QuestId::from_i8(23), Some(QuestId::Jersey));
        assert_eq!(QuestId::from_i8(-1), None);
        assert_eq!(QuestId::from_i8(24), None);

        assert_eq!(QuestId::Rock.to_i8(), 0);
        assert_eq!(QuestId::Jersey.to_i8(), 23);
    }

    #[test]
    fn test_object_data_flags() {
        let flags = ObjectDataFlags::ANIMATED | ObjectDataFlags::SOLID;

        assert!(flags.is_animated());
        assert!(flags.is_solid());
        assert!(!flags.is_trap());
        assert!(!flags.is_breakable());
    }

    #[test]
    fn test_object_data_flags_methods() {
        let data = ObjectData {
            ofindex: 0,
            minlvl: 1,
            maxlvl: 16,
            olvltype: DungeonType::Cathedral,
            otheme: ThemeId::NONE,
            oquest: QuestId::INVALID,
            flags: ObjectDataFlags::LIGHT | ObjectDataFlags::ANIMATED,
            anim_delay: 15,
            anim_len: 8,
            anim_width: 128,
            selection_region: SEL_NONE,
        };

        assert!(data.is_animated());
        assert!(data.apply_lighting());
        assert!(!data.is_solid());
        assert!(!data.is_trap());
    }

    #[test]
    fn test_obj_type_conv_length() {
        assert_eq!(OBJ_TYPE_CONV.len(), 146);
    }

    #[test]
    fn test_obj_type_conv_mapping() {
        // Test some key mappings
        assert_eq!(OBJ_TYPE_CONV[1], ObjectId::Lever.to_i8());
        assert_eq!(OBJ_TYPE_CONV[5], ObjectId::Angel.to_i8());
        assert_eq!(OBJ_TYPE_CONV[18], ObjectId::Candle1.to_i8());
        assert_eq!(OBJ_TYPE_CONV[145], ObjectId::L5Sarc.to_i8());
    }

    #[test]
    fn test_is_shrine() {
        assert!(is_shrine(ObjectId::ShrineL));
        assert!(is_shrine(ObjectId::ShrineR));
        assert!(is_shrine(ObjectId::GoatShrine));
        assert!(!is_shrine(ObjectId::Chest1));
        assert!(!is_shrine(ObjectId::Barrel));
    }

    #[test]
    fn test_is_chest() {
        assert!(is_chest(ObjectId::Chest1));
        assert!(is_chest(ObjectId::Chest2));
        assert!(is_chest(ObjectId::Chest3));
        assert!(is_chest(ObjectId::TChest1));
        assert!(!is_chest(ObjectId::Barrel));
        assert!(!is_chest(ObjectId::ShrineL));
    }

    #[test]
    fn test_is_door() {
        assert!(is_door(ObjectId::L1LDoor));
        assert!(is_door(ObjectId::L1RDoor));
        assert!(is_door(ObjectId::L5LDoor));
        assert!(is_door(ObjectId::L5RDoor));
        assert!(!is_door(ObjectId::Barrel));
        assert!(!is_door(ObjectId::Chest1));
    }

    #[test]
    fn test_is_breakable() {
        assert!(is_breakable(ObjectId::Barrel));
        assert!(is_breakable(ObjectId::BarrelEx));
        assert!(is_breakable(ObjectId::Pod));
        assert!(is_breakable(ObjectId::Urn));
        assert!(!is_breakable(ObjectId::Chest1));
        assert!(!is_breakable(ObjectId::ShrineL));
    }

    #[test]
    fn test_get_object_data() {
        // L1LIGHT: Animated,Solid,MissilesPassThrough — no Light flag here.
        let data = get_object_data(ObjectId::L1Light).unwrap();
        assert!(data.is_animated());
        assert!(data.is_solid());
        assert!(data.missiles_pass_through());
        assert!(!data.apply_lighting());

        // Null sentinel returns None.
        assert!(get_object_data(ObjectId::Null).is_none());
    }

    #[test]
    fn test_all_objects_count() {
        // Must match ObjectId variant count exactly (C++ assert:
        // OBJ_LAST + 1 == AllObjects.size()).
        assert_eq!(ALL_OBJECTS.len(), ObjectId::COUNT);
    }

    #[test]
    fn test_object_id_enum_values() {
        // Verify key enum values match C++ objdat.h indices
        assert_eq!(ObjectId::L1Light.to_i8(), 0);
        assert_eq!(ObjectId::Lever.to_i8(), 4);
        assert_eq!(ObjectId::Chest1.to_i8(), 5);
        assert_eq!(ObjectId::Barrel.to_i8(), 57);
        assert_eq!(ObjectId::L5Sarc.to_i8(), 108);  // Index 108 (109th element)
    }

    // ------------------------------------------------------------------
    // C++-consistency spot checks against objdat.tsv / objdat.h semantics
    // ------------------------------------------------------------------

    #[test]
    fn test_accessor_no_oob_for_every_object_id() {
        // Every valid discriminant must yield a table entry (mirrors the
        // C++ `OBJ_LAST+1 == AllObjects.size()` invariant).
        let mut hits = 0;
        for i in 0..(ObjectId::COUNT as i8) {
            let id = ObjectId::from_i8(i).unwrap();
            let d = obj_data(id);
            // every entry's ofindex must be a valid graphic index
            assert!(
                (d.ofindex as usize) < OBJ_MASTER_LOAD_LIST.len(),
                "object {:?} has ofindex {} out of range",
                id,
                d.ofindex
            );
            hits += 1;
        }
        assert_eq!(hits, ObjectId::COUNT);
    }

    #[test]
    fn test_l1ldoor_is_door_and_solid_semantics() {
        // OBJ_L1LDOOR: flags = Light,Trap ; selectionRegion = Bottom,Middle
        let d = obj_data(ObjectId::L1LDoor);
        assert!(is_door(ObjectId::L1LDoor));
        assert!(d.apply_lighting());
        assert!(d.is_trap());
        // The C++ table does NOT mark doors Solid — solidity is derived
        // elsewhere (door open/closed state), so is_solid() is false here.
        assert!(!d.is_solid());
        assert_eq!(d.anim_delay, 1);
        assert_eq!(d.anim_width, 64);
        assert_eq!(d.olvltype, DungeonType::Cathedral);
        // Door hit region is the Bottom,Middle combination.
        assert_eq!(d.selection_region, SEL_BOTTOM_MIDDLE);
    }

    #[test]
    fn test_l1rdoor_anim_delay_differs() {
        // OBJ_L1RDOOR is the same as L1LDOOR but animDelay=2.
        let d = obj_data(ObjectId::L1RDoor);
        assert_eq!(d.anim_delay, 2);
        assert_eq!(d.olvltype, DungeonType::Cathedral);
        assert!(d.is_trap());
    }

    #[test]
    fn test_chests_share_gfx_but_distinct_flags() {
        // Chest1/2/3 each have their own graphic (chest1/chest2/chest3).
        let c1 = obj_data(ObjectId::Chest1);
        let c2 = obj_data(ObjectId::Chest2);
        let c3 = obj_data(ObjectId::Chest3);
        assert_eq!(c1.ofindex, 4);
        assert_eq!(c2.ofindex, 5);
        assert_eq!(c3.ofindex, 6);
        for c in [c1, c2, c3] {
            assert!(c.is_solid());
            assert!(c.apply_lighting());
            assert!(c.is_trap());
            assert!(c.missiles_pass_through());
            assert_eq!(c.selection_region, SEL_BOTTOM);
            assert_eq!(c.minlvl, 1);
            assert_eq!(c.maxlvl, 24);
        }
    }

    #[test]
    fn test_torch_properties() {
        // OBJ_TORCHL: Animated,MissilesPassThrough — NOT Solid, NOT Light.
        let t = obj_data(ObjectId::TorchL);
        assert!(t.is_animated());
        assert!(t.missiles_pass_through());
        assert!(!t.is_solid());
        assert!(!t.apply_lighting());
        assert_eq!(t.minlvl, 5);
        assert_eq!(t.maxlvl, 8);
        assert_eq!(t.anim_len, 9);
        // 4 torches use 4 distinct wtorch graphics.
        let ids = [
            ObjectId::TorchL,
            ObjectId::TorchR,
            ObjectId::TorchL2,
            ObjectId::TorchR2,
        ];
        let gfx: Vec<u8> = ids.iter().map(|id| obj_data(*id).ofindex).collect();
        let uniq: std::collections::HashSet<u8> = gfx.iter().copied().collect();
        assert_eq!(uniq.len(), 4, "each torch has a unique graphic");
    }

    #[test]
    fn test_breakable_objects_match_helper() {
        // The breakable set in the TSV is Barrel, BarrelEx, Crux1-3,
        // Pod, PodEx, Urn, UrnEx. Verify the data flag and that the
        // is_breakable() helper agrees for the classic breakables.
        for id in [ObjectId::Barrel, ObjectId::BarrelEx, ObjectId::Pod, ObjectId::PodEx, ObjectId::Urn, ObjectId::UrnEx] {
            assert!(obj_data(id).is_breakable(), "{:?} should be breakable", id);
        }
        for id in [ObjectId::Crux1, ObjectId::Crux2, ObjectId::Crux3] {
            assert!(obj_data(id).is_breakable(), "{:?} should be breakable", id);
        }
        // Non-breakable sanity.
        assert!(!obj_data(ObjectId::Chest1).is_breakable());
    }

    #[test]
    fn test_theme_and_quest_fields() {
        // Candle2: THEME_SHRINE + Q_PWATER.
        let c = obj_data(ObjectId::Candle2);
        assert_eq!(c.otheme, ThemeId::Shrine.to_i8());
        assert_eq!(c.oquest, QuestId::PWater.to_i8());

        // TnudeM1: no theme, Q_BUTCHER.
        let t = obj_data(ObjectId::TNudeM1);
        assert_eq!(t.otheme, ThemeId::NONE);
        assert_eq!(t.oquest, QuestId::Butcher.to_i8());
        assert_eq!(t.minlvl, 13);
        assert_eq!(t.maxlvl, 15);

        // Pedestal: Q_BLOOD.
        let p = obj_data(ObjectId::Pedestal);
        assert_eq!(p.oquest, QuestId::Blood.to_i8());

        // StoryCandle: Q_BETRAYER.
        let s = obj_data(ObjectId::StoryCandle);
        assert_eq!(s.oquest, QuestId::Betrayer.to_i8());

        // SteelTome: Q_WARLORD.
        let st = obj_data(ObjectId::SteelTome);
        assert_eq!(st.oquest, QuestId::Warlord.to_i8());

        // Barrel: no theme, no quest.
        let b = obj_data(ObjectId::Barrel);
        assert_eq!(b.otheme, ThemeId::NONE);
        assert_eq!(b.oquest, QuestId::INVALID);
    }

    #[test]
    fn test_level_type_per_dungeon_set() {
        assert_eq!(obj_data(ObjectId::L1LDoor).olvltype, DungeonType::Cathedral);
        assert_eq!(obj_data(ObjectId::L2LDoor).olvltype, DungeonType::Catacombs);
        assert_eq!(obj_data(ObjectId::L3LDoor).olvltype, DungeonType::Caves);
        assert_eq!(obj_data(ObjectId::L5LDoor).olvltype, DungeonType::Crypt);
        // Non-door objects with empty levelType => None.
        assert_eq!(obj_data(ObjectId::Barrel).olvltype, DungeonType::None);
        assert_eq!(obj_data(ObjectId::SkFire).olvltype, DungeonType::None);
    }

    #[test]
    fn test_ofindex_dedup_matches_master_load_list() {
        // ofindex must point at the correct dedup'd graphic name.
        assert_eq!(OBJ_MASTER_LOAD_LIST[obj_data(ObjectId::L1Light).ofindex as usize], "l1braz");
        // All four l1braz-backed objects share ofindex 0.
        assert_eq!(obj_data(ObjectId::L1Light).ofindex, 0);
        assert_eq!(obj_data(ObjectId::Candle1).ofindex, 0);
        assert_eq!(obj_data(ObjectId::CandleO).ofindex, 0);
        assert_eq!(obj_data(ObjectId::SkStick1).ofindex, 0);
        // book2 ofindex is 15, shared by Book2L/Book2R/SkelBook/Bookstand.
        assert_eq!(obj_data(ObjectId::Book2L).ofindex, 15);
        assert_eq!(obj_data(ObjectId::Book2R).ofindex, 15);
        assert_eq!(obj_data(ObjectId::SkelBook).ofindex, 15);
        assert_eq!(obj_data(ObjectId::Bookstand).ofindex, 15);
        // Final object L5SARC gets the last fresh graphic id.
        assert_eq!(obj_data(ObjectId::L5Sarc).ofindex, 61);
        assert_eq!(OBJ_MASTER_LOAD_LIST[61], "l5sarco");
        assert_eq!(OBJ_MASTER_LOAD_LIST.len(), 62);
    }

    #[test]
    fn test_door_animwidth_is_64() {
        // All doors across L1/L2/L3/L5 use animWidth 64.
        for id in [
            ObjectId::L1LDoor, ObjectId::L1RDoor,
            ObjectId::L2LDoor, ObjectId::L2RDoor,
            ObjectId::L3LDoor, ObjectId::L3RDoor,
            ObjectId::L5LDoor, ObjectId::L5RDoor,
        ] {
            assert_eq!(obj_data(id).anim_width, 64, "{:?} animWidth", id);
            assert!(is_door(id));
        }
    }
}
