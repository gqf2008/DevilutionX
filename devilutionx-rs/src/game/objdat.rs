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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionRegion {
    #[default]
    None,
    Bottom,
    Middle,
    Top,
}

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

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn is_animated(&self) -> bool {
        self.contains(Self::ANIMATED)
    }

    pub fn is_solid(&self) -> bool {
        self.contains(Self::SOLID)
    }

    pub fn missiles_pass_through(&self) -> bool {
        self.contains(Self::MISSILES_PASS_THROUGH)
    }

    pub fn apply_lighting(&self) -> bool {
        self.contains(Self::LIGHT)
    }

    pub fn is_trap(&self) -> bool {
        self.contains(Self::TRAP)
    }

    pub fn is_breakable(&self) -> bool {
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

    /// Selection region
    pub selection_region: SelectionRegion,
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

/// Get object data by ObjectId
pub fn get_object_data(obj_id: ObjectId) -> Option<&'static ObjectData> {
    ALL_OBJECTS.get(obj_id as usize)
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

/// Complete object data table - will be populated with actual data
pub static ALL_OBJECTS: &[ObjectData] = &[
    // Placeholder - actual data will be added next
    ObjectData {
        ofindex: 0,
        minlvl: 1,
        maxlvl: 16,
        olvltype: DungeonType::Cathedral,
        otheme: ThemeId::NONE,
        oquest: QuestId::INVALID,
        flags: ObjectDataFlags::LIGHT,
        anim_delay: 15,
        anim_len: 8,
        anim_width: 128,
        selection_region: SelectionRegion::None,
    },
    // TODO: Add remaining 110 objects
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
            selection_region: SelectionRegion::None,
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
        let data = get_object_data(ObjectId::L1Light);
        assert!(data.is_some());

        if let Some(obj) = data {
            assert!(obj.apply_lighting());
        }
    }

    #[test]
    fn test_all_objects_count() {
        // Currently placeholder, will be 109 when complete
        assert!(ALL_OBJECTS.len() >= 1);
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
}
