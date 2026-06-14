/// Automap System for DevilutionX-RS
///
/// This module implements the in-game map overlay functionality:
/// - AutomapTile types for different dungeon features
/// - Drawing functions for map elements
/// - Automap manager for state and rendering
/// - Minimap support
///
/// Ported from C++ automap.cpp (1,956 lines)

use crate::game::lighting::DungeonLevelType;

// ============================================================================
// Constants
// ============================================================================

/// Maximum dungeon X dimension
pub const DMAXX: usize = 40;
/// Maximum dungeon Y dimension
pub const DMAXY: usize = 40;

/// Default automap scale
pub const DEFAULT_AUTOMAP_SCALE: i32 = 50;

/// Minimum automap scale
pub const MIN_AUTOMAP_SCALE: i32 = 25;

/// Maximum automap scale
pub const MAX_AUTOMAP_SCALE: i32 = 200;

// ============================================================================
// Map Colors - matches C++ MapColors enum
// ============================================================================

/// Color used to draw the player's arrow
pub const MAP_COLOR_PLAYER: u8 = 128 + 1;  // PAL8_ORANGE + 1

/// Color for bright map lines (doors, stairs etc.)
pub const MAP_COLOR_BRIGHT: u8 = 144;  // PAL8_YELLOW

/// Color for dim map lines/dots
pub const MAP_COLOR_DIM: u8 = 160 + 8;  // PAL16_YELLOW + 8

/// Color for items on automap
pub const MAP_COLOR_ITEM: u8 = 112 + 1;  // PAL8_BLUE + 1

/// Color for activated pentagram on automap
pub const MAP_COLOR_PENTAGRAM_OPEN: u8 = 96 + 2;  // PAL8_RED + 2

/// Color for cave lava on automap
pub const MAP_COLOR_LAVA: u8 = 128 + 2;  // PAL8_ORANGE + 2

/// Color for cave water on automap
pub const MAP_COLOR_WATER: u8 = 112 + 2;  // PAL8_BLUE + 2

/// Color for hive acid on automap
pub const MAP_COLOR_ACID: u8 = 144 + 4;  // PAL8_YELLOW + 4

// ============================================================================
// Automap Tile Types - matches C++ AutomapTile::Types enum
// ============================================================================

/// Automap tile shape types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum AutomapTileType {
    #[default]
    None = 0,
    Diamond,
    Vertical,
    Horizontal,
    Cross,
    FenceVertical,
    FenceHorizontal,
    Corner,
    CaveHorizontalCross,
    CaveVerticalCross,
    CaveHorizontal,
    CaveVertical,
    CaveCross,
    Bridge,
    River,
    RiverCornerEast,
    RiverCornerNorth,
    RiverCornerSouth,
    RiverCornerWest,
    RiverForkIn,
    RiverForkOut,
    RiverLeftIn,
    RiverLeftOut,
    RiverRightIn,
    RiverRightOut,
    CaveHorizontalWoodCross,
    CaveVerticalWoodCross,
    CaveLeftCorner,
    CaveRightCorner,
    CaveBottomCorner,
    CaveHorizontalWood,
    CaveVerticalWood,
    CaveWoodCross,
    CaveRightWoodCross,
    CaveLeftWoodCross,
    HorizontalLavaThin,
    VerticalLavaThin,
    BendSouthLavaThin,
    BendWestLavaThin,
    BendEastLavaThin,
    BendNorthLavaThin,
    VerticalWallLava,
    HorizontalWallLava,
    SELava,
    SWLava,
    NELava,
    NWLava,
    SLava,
    WLava,
    ELava,
    NLava,
    Lava,
    CaveHorizontalWallLava,
    CaveVerticalWallLava,
    HorizontalBridgeLava,
    VerticalBridgeLava,
    VerticalDiamond,
    HorizontalDiamond,
    PentagramClosed,
    PentagramOpen,
}

impl AutomapTileType {
    /// Get tile type from raw value
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Diamond,
            2 => Self::Vertical,
            3 => Self::Horizontal,
            4 => Self::Cross,
            5 => Self::FenceVertical,
            6 => Self::FenceHorizontal,
            7 => Self::Corner,
            8 => Self::CaveHorizontalCross,
            9 => Self::CaveVerticalCross,
            10 => Self::CaveHorizontal,
            11 => Self::CaveVertical,
            12 => Self::CaveCross,
            13 => Self::Bridge,
            14 => Self::River,
            15 => Self::RiverCornerEast,
            16 => Self::RiverCornerNorth,
            17 => Self::RiverCornerSouth,
            18 => Self::RiverCornerWest,
            19 => Self::RiverForkIn,
            20 => Self::RiverForkOut,
            21 => Self::RiverLeftIn,
            22 => Self::RiverLeftOut,
            23 => Self::RiverRightIn,
            24 => Self::RiverRightOut,
            25 => Self::CaveHorizontalWoodCross,
            26 => Self::CaveVerticalWoodCross,
            27 => Self::CaveLeftCorner,
            28 => Self::CaveRightCorner,
            29 => Self::CaveBottomCorner,
            30 => Self::CaveHorizontalWood,
            31 => Self::CaveVerticalWood,
            32 => Self::CaveWoodCross,
            33 => Self::CaveRightWoodCross,
            34 => Self::CaveLeftWoodCross,
            35 => Self::HorizontalLavaThin,
            36 => Self::VerticalLavaThin,
            37 => Self::BendSouthLavaThin,
            38 => Self::BendWestLavaThin,
            39 => Self::BendEastLavaThin,
            40 => Self::BendNorthLavaThin,
            41 => Self::VerticalWallLava,
            42 => Self::HorizontalWallLava,
            43 => Self::SELava,
            44 => Self::SWLava,
            45 => Self::NELava,
            46 => Self::NWLava,
            47 => Self::SLava,
            48 => Self::WLava,
            49 => Self::ELava,
            50 => Self::NLava,
            51 => Self::Lava,
            52 => Self::CaveHorizontalWallLava,
            53 => Self::CaveVerticalWallLava,
            54 => Self::HorizontalBridgeLava,
            55 => Self::VerticalBridgeLava,
            56 => Self::VerticalDiamond,
            57 => Self::HorizontalDiamond,
            58 => Self::PentagramClosed,
            59 => Self::PentagramOpen,
            _ => Self::None,
        }
    }

    /// Check if this is a lava type
    pub fn is_lava(&self) -> bool {
        matches!(self,
            Self::HorizontalLavaThin |
            Self::VerticalLavaThin |
            Self::BendSouthLavaThin |
            Self::BendWestLavaThin |
            Self::BendEastLavaThin |
            Self::BendNorthLavaThin |
            Self::VerticalWallLava |
            Self::HorizontalWallLava |
            Self::SELava |
            Self::SWLava |
            Self::NELava |
            Self::NWLava |
            Self::SLava |
            Self::WLava |
            Self::ELava |
            Self::NLava |
            Self::Lava |
            Self::CaveHorizontalWallLava |
            Self::CaveVerticalWallLava |
            Self::HorizontalBridgeLava |
            Self::VerticalBridgeLava
        )
    }

    /// Check if this is a river type
    pub fn is_river(&self) -> bool {
        matches!(self,
            Self::River |
            Self::RiverCornerEast |
            Self::RiverCornerNorth |
            Self::RiverCornerSouth |
            Self::RiverCornerWest |
            Self::RiverForkIn |
            Self::RiverForkOut |
            Self::RiverLeftIn |
            Self::RiverLeftOut |
            Self::RiverRightIn |
            Self::RiverRightOut
        )
    }

    /// Check if this is a cave type
    pub fn is_cave(&self) -> bool {
        matches!(self,
            Self::CaveHorizontalCross |
            Self::CaveVerticalCross |
            Self::CaveHorizontal |
            Self::CaveVertical |
            Self::CaveCross |
            Self::CaveHorizontalWoodCross |
            Self::CaveVerticalWoodCross |
            Self::CaveLeftCorner |
            Self::CaveRightCorner |
            Self::CaveBottomCorner |
            Self::CaveHorizontalWood |
            Self::CaveVerticalWood |
            Self::CaveWoodCross |
            Self::CaveRightWoodCross |
            Self::CaveLeftWoodCross
        )
    }
}

// ============================================================================
// Automap Tile Flags - matches C++ AutomapTile::Flags enum
// ============================================================================

bitflags::bitflags! {
    /// Flags for automap tiles
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct AutomapTileFlags: u8 {
        const VERTICAL_DOOR = 1 << 0;
        const HORIZONTAL_DOOR = 1 << 1;
        const VERTICAL_ARCH = 1 << 2;
        const HORIZONTAL_ARCH = 1 << 3;
        const VERTICAL_GRATE = 1 << 4;
        const HORIZONTAL_GRATE = 1 << 5;
        const DIRT = 1 << 6;
        const STAIRS = 1 << 7;

        /// Combined flags
        const VERTICAL_PASSAGE = Self::VERTICAL_DOOR.bits() | Self::VERTICAL_ARCH.bits() | Self::VERTICAL_GRATE.bits();
        const HORIZONTAL_PASSAGE = Self::HORIZONTAL_DOOR.bits() | Self::HORIZONTAL_ARCH.bits() | Self::HORIZONTAL_GRATE.bits();
    }
}

impl AutomapTileFlags {
    /// Check if has any vertical passage
    pub fn has_vertical_passage(&self) -> bool {
        self.intersects(Self::VERTICAL_PASSAGE)
    }

    /// Check if has any horizontal passage
    pub fn has_horizontal_passage(&self) -> bool {
        self.intersects(Self::HORIZONTAL_PASSAGE)
    }
}

// ============================================================================
// Automap Tile - combines type and flags
// ============================================================================

/// Complete automap tile definition
#[derive(Debug, Clone, Copy, Default)]
pub struct AutomapTile {
    pub tile_type: AutomapTileType,
    pub flags: AutomapTileFlags,
}

impl AutomapTile {
    /// Create new automap tile
    pub fn new(tile_type: AutomapTileType, flags: AutomapTileFlags) -> Self {
        Self { tile_type, flags }
    }

    /// Create tile with type only
    pub fn with_type(tile_type: AutomapTileType) -> Self {
        Self { tile_type, flags: AutomapTileFlags::empty() }
    }

    /// Check if tile has specific flag
    pub fn has_flag(&self, flag: AutomapTileFlags) -> bool {
        self.flags.contains(flag)
    }

    /// Check if tile has any of the specified flags
    pub fn has_any_flag(&self, flags: AutomapTileFlags) -> bool {
        self.flags.intersects(flags)
    }
}

// ============================================================================
// Automap Type - display mode
// ============================================================================

/// Automap display type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutomapType {
    #[default]
    Opaque,
    Transparent,
    Minimap,
}

// ============================================================================
// Direction for player arrow
// ============================================================================

/// Direction enum for player arrow drawing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    South = 0,
    SouthWest,
    West,
    NorthWest,
    North,
    NorthEast,
    East,
    SouthEast,
    NoDirection,
}

impl Direction {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::South,
            1 => Self::SouthWest,
            2 => Self::West,
            3 => Self::NorthWest,
            4 => Self::North,
            5 => Self::NorthEast,
            6 => Self::East,
            7 => Self::SouthEast,
            _ => Self::NoDirection,
        }
    }
}

// ============================================================================
// Point and Rectangle types
// ============================================================================

/// 2D point
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn offset(&self, dx: i32, dy: i32) -> Self {
        Self { x: self.x + dx, y: self.y + dy }
    }
}

/// 2D displacement
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Displacement {
    pub dx: i32,
    pub dy: i32,
}

impl Displacement {
    pub fn new(dx: i32, dy: i32) -> Self {
        Self { dx, dy }
    }
}

/// 2D size
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

/// Rectangle
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rectangle {
    pub position: Point,
    pub size: Size,
}

impl Rectangle {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            position: Point::new(x, y),
            size: Size { width, height },
        }
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.position.x
            && point.x < self.position.x + self.size.width
            && point.y >= self.position.y
            && point.y < self.position.y + self.size.height
    }
}

// ============================================================================
// Automap Line Drawing Helpers
// ============================================================================

/// Line length for automap drawing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum AmLineLength {
    DoubleTile = -2,
    FullTile = -1,
    HalfTile = 0,
    QuarterTile = 1,
}

impl AmLineLength {
    pub fn to_pixels(&self, scale: i32) -> i32 {
        match self {
            Self::DoubleTile => scale * 2,
            Self::FullTile => scale,
            Self::HalfTile => scale / 2,
            Self::QuarterTile => scale / 4,
        }
    }
}

/// Width offset for automap drawing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmWidthOffset {
    ThreeQuartersTileLeft,
    HalfTileLeft,
    QuarterTileLeft,
    None,
    QuarterTileRight,
    HalfTileRight,
    ThreeQuartersTileRight,
}

impl AmWidthOffset {
    pub fn to_pixels(&self, scale: i32) -> i32 {
        match self {
            Self::ThreeQuartersTileLeft => -(scale * 3 / 4),
            Self::HalfTileLeft => -(scale / 2),
            Self::QuarterTileLeft => -(scale / 4),
            Self::None => 0,
            Self::QuarterTileRight => scale / 4,
            Self::HalfTileRight => scale / 2,
            Self::ThreeQuartersTileRight => scale * 3 / 4,
        }
    }
}

/// Height offset for automap drawing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmHeightOffset {
    FullTileUp,
    HalfTileUp,
    QuarterTileUp,
    None,
    QuarterTileDown,
    HalfTileDown,
    ThreeQuartersTileDown,
    FullTileDown,
}

impl AmHeightOffset {
    pub fn to_pixels(&self, scale: i32) -> i32 {
        match self {
            Self::FullTileUp => -(scale / 2),
            Self::HalfTileUp => -(scale / 4),
            Self::QuarterTileUp => -(scale / 8),
            Self::None => 0,
            Self::QuarterTileDown => scale / 8,
            Self::HalfTileDown => scale / 4,
            Self::ThreeQuartersTileDown => scale * 3 / 8,
            Self::FullTileDown => scale / 2,
        }
    }
}

// ============================================================================
// Automap Manager
// ============================================================================

/// Automap manager handles map overlay rendering and state
pub struct AutomapManager {
    /// Whether automap is currently active
    pub active: bool,

    /// Current display type
    pub map_type: AutomapType,

    /// Explored tiles view [x][y]
    pub view: [[u8; DMAXY]; DMAXX],

    /// Automap scale (zoom level)
    pub scale: i32,

    /// Minimap scale
    pub minimap_scale: i32,

    /// Automap scroll offset
    pub offset: Displacement,

    /// Minimap screen rectangle
    pub minimap_rect: Rectangle,

    /// Tile type mappings
    tile_types: [AutomapTile; 256],

    /// Current level type
    level_type: DungeonLevelType,

    /// Screen dimensions
    screen_width: i32,
    screen_height: i32,
}

impl Default for AutomapManager {
    fn default() -> Self {
        Self::new(640, 480)
    }
}

impl AutomapManager {
    /// Create new automap manager
    pub fn new(screen_width: i32, screen_height: i32) -> Self {
        let minimap_width = screen_width / 4;
        let minimap_height = minimap_width / 2;
        let minimap_padding = screen_width / 128;

        // Calculate minimap scale based on screen height
        let base_height = 480;
        let base_scale = 25;
        let factor = screen_height / base_height;
        let minimap_scale = if factor >= 8 {
            base_scale * 8
        } else {
            base_scale * factor
        };

        Self {
            active: false,
            map_type: AutomapType::Opaque,
            view: [[0; DMAXY]; DMAXX],
            scale: DEFAULT_AUTOMAP_SCALE,
            minimap_scale: minimap_scale.max(base_scale),
            offset: Displacement::new(0, 0),
            minimap_rect: Rectangle::new(
                screen_width - minimap_padding - minimap_width,
                minimap_padding,
                minimap_width,
                minimap_height,
            ),
            tile_types: [AutomapTile::default(); 256],
            level_type: DungeonLevelType::Town,
            screen_width,
            screen_height,
        }
    }

    /// Initialize automap once (on game start)
    pub fn init_once(&mut self) {
        self.active = false;
        self.scale = DEFAULT_AUTOMAP_SCALE;
    }

    /// Initialize automap for a level
    pub fn init(&mut self, level_type: DungeonLevelType, tile_data: &[u8]) {
        self.level_type = level_type;

        // Reset tile types
        self.tile_types = [AutomapTile::default(); 256];

        // Load base tile types from data
        for (i, &byte) in tile_data.iter().enumerate() {
            if i < 255 {
                self.tile_types[i + 1] = AutomapTile::with_type(AutomapTileType::from_u8(byte));
            }
        }

        // Apply level-specific overrides
        self.apply_level_overrides();

        // Clear explored view
        self.clear_view();
    }

    /// Apply level-specific tile type overrides
    fn apply_level_overrides(&mut self) {
        match self.level_type {
            DungeonLevelType::Catacombs => {
                self.tile_types[41] = AutomapTile::with_type(AutomapTileType::FenceHorizontal);
            }
            DungeonLevelType::Town | DungeonLevelType::Caves | DungeonLevelType::Nest => {
                self.tile_types[4] = AutomapTile::with_type(AutomapTileType::CaveBottomCorner);
                self.tile_types[12] = AutomapTile::with_type(AutomapTileType::CaveRightCorner);
                self.tile_types[13] = AutomapTile::with_type(AutomapTileType::CaveLeftCorner);

                if self.level_type == DungeonLevelType::Caves {
                    self.apply_caves_overrides();
                } else if self.level_type == DungeonLevelType::Nest {
                    self.apply_nest_overrides();
                }
            }
            DungeonLevelType::Hell => {
                self.tile_types[51] = AutomapTile::with_type(AutomapTileType::VerticalDiamond);
                self.tile_types[55] = AutomapTile::with_type(AutomapTileType::HorizontalDiamond);
                self.tile_types[102] = AutomapTile::with_type(AutomapTileType::PentagramClosed);
                self.tile_types[111] = AutomapTile::with_type(AutomapTileType::PentagramOpen);
            }
            _ => {}
        }
    }

    /// Apply Caves-specific overrides
    fn apply_caves_overrides(&mut self) {
        // Wood structures
        self.tile_types[129] = AutomapTile::with_type(AutomapTileType::CaveHorizontalWoodCross);
        self.tile_types[131] = AutomapTile::with_type(AutomapTileType::CaveHorizontalWoodCross);
        self.tile_types[133] = AutomapTile::with_type(AutomapTileType::CaveHorizontalWood);
        self.tile_types[135] = AutomapTile::with_type(AutomapTileType::CaveHorizontalWood);
        self.tile_types[150] = AutomapTile::with_type(AutomapTileType::CaveHorizontalWood);
        self.tile_types[145] = AutomapTile::new(
            AutomapTileType::CaveHorizontalWood,
            AutomapTileFlags::VERTICAL_DOOR,
        );
        self.tile_types[147] = AutomapTile::new(
            AutomapTileType::CaveHorizontalWood,
            AutomapTileFlags::VERTICAL_DOOR,
        );
        self.tile_types[130] = AutomapTile::with_type(AutomapTileType::CaveVerticalWoodCross);
        self.tile_types[132] = AutomapTile::with_type(AutomapTileType::CaveVerticalWoodCross);
        self.tile_types[134] = AutomapTile::with_type(AutomapTileType::CaveVerticalWood);
        self.tile_types[136] = AutomapTile::with_type(AutomapTileType::CaveVerticalWood);
        self.tile_types[151] = AutomapTile::with_type(AutomapTileType::CaveVerticalWood);
        self.tile_types[146] = AutomapTile::new(
            AutomapTileType::CaveVerticalWood,
            AutomapTileFlags::HORIZONTAL_DOOR,
        );
        self.tile_types[148] = AutomapTile::new(
            AutomapTileType::CaveVerticalWood,
            AutomapTileFlags::HORIZONTAL_DOOR,
        );
        self.tile_types[137] = AutomapTile::with_type(AutomapTileType::CaveWoodCross);
        self.tile_types[140] = AutomapTile::with_type(AutomapTileType::CaveWoodCross);
        self.tile_types[141] = AutomapTile::with_type(AutomapTileType::CaveWoodCross);
        self.tile_types[142] = AutomapTile::with_type(AutomapTileType::CaveWoodCross);
        self.tile_types[138] = AutomapTile::with_type(AutomapTileType::CaveRightWoodCross);
        self.tile_types[139] = AutomapTile::with_type(AutomapTileType::CaveLeftWoodCross);

        // Lava tiles
        self.tile_types[14] = AutomapTile::with_type(AutomapTileType::HorizontalLavaThin);
        self.tile_types[15] = AutomapTile::with_type(AutomapTileType::HorizontalLavaThin);
        self.tile_types[16] = AutomapTile::with_type(AutomapTileType::VerticalLavaThin);
        self.tile_types[17] = AutomapTile::with_type(AutomapTileType::VerticalLavaThin);
        self.tile_types[18] = AutomapTile::with_type(AutomapTileType::BendSouthLavaThin);
        self.tile_types[19] = AutomapTile::with_type(AutomapTileType::BendWestLavaThin);
        self.tile_types[20] = AutomapTile::with_type(AutomapTileType::BendEastLavaThin);
        self.tile_types[21] = AutomapTile::with_type(AutomapTileType::BendNorthLavaThin);
        self.tile_types[22] = AutomapTile::with_type(AutomapTileType::VerticalWallLava);
        self.tile_types[23] = AutomapTile::with_type(AutomapTileType::HorizontalWallLava);
        self.tile_types[24] = AutomapTile::with_type(AutomapTileType::SELava);
        self.tile_types[25] = AutomapTile::with_type(AutomapTileType::SWLava);
        self.tile_types[26] = AutomapTile::with_type(AutomapTileType::NELava);
        self.tile_types[27] = AutomapTile::with_type(AutomapTileType::NWLava);
        self.tile_types[28] = AutomapTile::with_type(AutomapTileType::SLava);
        self.tile_types[29] = AutomapTile::with_type(AutomapTileType::WLava);
        self.tile_types[30] = AutomapTile::with_type(AutomapTileType::ELava);
        self.tile_types[31] = AutomapTile::with_type(AutomapTileType::NLava);
        for i in 32..=40 {
            self.tile_types[i] = AutomapTile::with_type(AutomapTileType::Lava);
        }
        self.tile_types[41] = AutomapTile::with_type(AutomapTileType::CaveHorizontalWallLava);
        self.tile_types[42] = AutomapTile::with_type(AutomapTileType::CaveVerticalWallLava);
        self.tile_types[43] = AutomapTile::with_type(AutomapTileType::HorizontalBridgeLava);
        self.tile_types[44] = AutomapTile::with_type(AutomapTileType::VerticalBridgeLava);
    }

    /// Apply Nest-specific overrides
    fn apply_nest_overrides(&mut self) {
        self.tile_types[102] = AutomapTile::with_type(AutomapTileType::HorizontalLavaThin);
        self.tile_types[103] = AutomapTile::with_type(AutomapTileType::HorizontalLavaThin);
        self.tile_types[108] = AutomapTile::with_type(AutomapTileType::HorizontalLavaThin);
        self.tile_types[104] = AutomapTile::with_type(AutomapTileType::VerticalLavaThin);
        self.tile_types[105] = AutomapTile::with_type(AutomapTileType::VerticalLavaThin);
        self.tile_types[107] = AutomapTile::with_type(AutomapTileType::VerticalLavaThin);
        self.tile_types[112] = AutomapTile::with_type(AutomapTileType::BendSouthLavaThin);
        self.tile_types[113] = AutomapTile::with_type(AutomapTileType::BendWestLavaThin);
        self.tile_types[110] = AutomapTile::with_type(AutomapTileType::BendEastLavaThin);
        self.tile_types[111] = AutomapTile::with_type(AutomapTileType::BendNorthLavaThin);
        self.tile_types[134] = AutomapTile::with_type(AutomapTileType::VerticalWallLava);
        self.tile_types[135] = AutomapTile::with_type(AutomapTileType::HorizontalWallLava);
        self.tile_types[118] = AutomapTile::with_type(AutomapTileType::SELava);
        self.tile_types[119] = AutomapTile::with_type(AutomapTileType::SWLava);
        self.tile_types[120] = AutomapTile::with_type(AutomapTileType::NELava);
        self.tile_types[121] = AutomapTile::with_type(AutomapTileType::NWLava);
        self.tile_types[106] = AutomapTile::with_type(AutomapTileType::SLava);
        self.tile_types[114] = AutomapTile::with_type(AutomapTileType::WLava);
        self.tile_types[130] = AutomapTile::with_type(AutomapTileType::ELava);
        self.tile_types[122] = AutomapTile::with_type(AutomapTileType::NLava);
        self.tile_types[117] = AutomapTile::with_type(AutomapTileType::Lava);
        self.tile_types[124] = AutomapTile::with_type(AutomapTileType::Lava);
        for i in [126, 127, 128, 129, 131, 132, 133] {
            self.tile_types[i] = AutomapTile::with_type(AutomapTileType::Lava);
        }
        self.tile_types[136] = AutomapTile::with_type(AutomapTileType::CaveHorizontalWallLava);
        self.tile_types[137] = AutomapTile::with_type(AutomapTileType::CaveVerticalWallLava);
        self.tile_types[115] = AutomapTile::with_type(AutomapTileType::HorizontalBridgeLava);
        self.tile_types[116] = AutomapTile::with_type(AutomapTileType::VerticalBridgeLava);
    }

    /// Clear explored view
    pub fn clear_view(&mut self) {
        self.view = [[0; DMAXY]; DMAXX];
    }

    /// Start automap display
    pub fn start(&mut self) {
        self.offset = Displacement::new(0, 0);
        self.active = true;
    }

    /// Stop automap display
    pub fn stop(&mut self) {
        self.active = false;
    }

    /// Toggle automap display
    pub fn toggle(&mut self) {
        if self.active {
            self.stop();
        } else {
            self.start();
        }
    }

    /// Move automap view up
    pub fn move_up(&mut self) {
        self.offset.dy -= 1;
    }

    /// Move automap view down
    pub fn move_down(&mut self) {
        self.offset.dy += 1;
    }

    /// Move automap view left
    pub fn move_left(&mut self) {
        self.offset.dx -= 1;
    }

    /// Move automap view right
    pub fn move_right(&mut self) {
        self.offset.dx += 1;
    }

    /// Zoom in
    pub fn zoom_in(&mut self) {
        if self.scale < MAX_AUTOMAP_SCALE {
            self.scale += 25;
        }
    }

    /// Zoom out
    pub fn zoom_out(&mut self) {
        if self.scale > MIN_AUTOMAP_SCALE {
            self.scale -= 25;
        }
    }

    /// Set automap view at position
    pub fn set_view(&mut self, x: usize, y: usize) {
        if x < DMAXX && y < DMAXY {
            self.view[x][y] = 1;
        }
    }

    /// Check if position is explored
    pub fn is_explored(&self, x: usize, y: usize) -> bool {
        if x < DMAXX && y < DMAXY {
            self.view[x][y] != 0
        } else {
            false
        }
    }

    /// Get tile type at index
    pub fn get_tile(&self, index: usize) -> AutomapTile {
        if index < 256 {
            self.tile_types[index]
        } else {
            AutomapTile::default()
        }
    }

    /// Get screen center for automap
    pub fn get_screen_center(&self) -> Point {
        Point::new(self.screen_width / 2, self.screen_height / 2)
    }

    /// Calculate tile screen position
    pub fn tile_to_screen(&self, tile_x: i32, tile_y: i32, player_x: i32, player_y: i32) -> Point {
        let center = self.get_screen_center();
        let dx = tile_x - player_x - self.offset.dx;
        let dy = tile_y - player_y - self.offset.dy;

        // Isometric projection
        let screen_x = center.x + (dx - dy) * self.scale / 2;
        let screen_y = center.y + (dx + dy) * self.scale / 4;

        Point::new(screen_x, screen_y)
    }

    /// Get color for lava based on level type
    pub fn get_lava_color(&self) -> u8 {
        match self.level_type {
            DungeonLevelType::Nest => MAP_COLOR_ACID,
            _ => MAP_COLOR_LAVA,
        }
    }

    /// Get color for water based on level type
    pub fn get_water_color(&self) -> u8 {
        MAP_COLOR_WATER
    }
}

// ============================================================================
// Drawing Functions (stubs - actual rendering depends on graphics backend)
// ============================================================================

/// Draw context for automap rendering
pub struct AutomapDrawContext {
    pub scale: i32,
    pub color_player: u8,
    pub color_bright: u8,
    pub color_dim: u8,
    pub color_item: u8,
    pub color_lava: u8,
    pub color_water: u8,
}

impl Default for AutomapDrawContext {
    fn default() -> Self {
        Self {
            scale: DEFAULT_AUTOMAP_SCALE,
            color_player: MAP_COLOR_PLAYER,
            color_bright: MAP_COLOR_BRIGHT,
            color_dim: MAP_COLOR_DIM,
            color_item: MAP_COLOR_ITEM,
            color_lava: MAP_COLOR_LAVA,
            color_water: MAP_COLOR_WATER,
        }
    }
}

/// Pixel buffer for drawing (abstraction over actual surface)
pub struct PixelBuffer {
    width: i32,
    height: i32,
    pixels: Vec<u8>,
}

impl PixelBuffer {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width * height) as usize],
        }
    }

    /// Set pixel at position
    pub fn set_pixel(&mut self, x: i32, y: i32, color: u8) {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            self.pixels[(y * self.width + x) as usize] = color;
        }
    }

    /// Get pixel at position
    pub fn get_pixel(&self, x: i32, y: i32) -> u8 {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            self.pixels[(y * self.width + x) as usize]
        } else {
            0
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }
}

/// Draw a diamond shape
pub fn draw_diamond(buffer: &mut PixelBuffer, center: Point, scale: i32, color: u8) {
    let half = scale / 2;
    let quarter = scale / 4;

    // Top to right
    draw_line_ne(buffer, center.offset(-half, -quarter), half, color);
    // Top to left
    draw_line_se(buffer, center.offset(-half, -quarter), half, color);
    // Bottom to right
    draw_line_se(buffer, center.offset(0, -half), half, color);
    // Bottom to left
    draw_line_ne(buffer, center.offset(0, 0), half, color);
}

/// Draw line going northeast
pub fn draw_line_ne(buffer: &mut PixelBuffer, start: Point, length: i32, color: u8) {
    let mut x = start.x;
    let mut y = start.y;

    for _ in 0..length {
        buffer.set_pixel(x, y, color);
        x += 1;
        if x % 2 == 0 {
            y -= 1;
        }
    }
}

/// Draw line going southeast
pub fn draw_line_se(buffer: &mut PixelBuffer, start: Point, length: i32, color: u8) {
    let mut x = start.x;
    let mut y = start.y;

    for _ in 0..length {
        buffer.set_pixel(x, y, color);
        x += 1;
        if x % 2 == 0 {
            y += 1;
        }
    }
}

/// Draw line going southwest
pub fn draw_line_sw(buffer: &mut PixelBuffer, start: Point, length: i32, color: u8) {
    let mut x = start.x;
    let mut y = start.y;

    for _ in 0..length {
        buffer.set_pixel(x, y, color);
        x -= 1;
        if x % 2 == 0 {
            y += 1;
        }
    }
}

/// Draw line going northwest
pub fn draw_line_nw(buffer: &mut PixelBuffer, start: Point, length: i32, color: u8) {
    let mut x = start.x;
    let mut y = start.y;

    for _ in 0..length {
        buffer.set_pixel(x, y, color);
        x -= 1;
        if x % 2 == 0 {
            y -= 1;
        }
    }
}

/// Draw horizontal line
pub fn draw_line_horizontal(buffer: &mut PixelBuffer, start: Point, length: i32, color: u8) {
    for i in 0..length {
        buffer.set_pixel(start.x + i, start.y, color);
    }
}

/// Draw vertical line
pub fn draw_line_vertical(buffer: &mut PixelBuffer, start: Point, length: i32, color: u8) {
    for i in 0..length {
        buffer.set_pixel(start.x, start.y + i, color);
    }
}

/// Draw player arrow based on direction
pub fn draw_player_arrow(buffer: &mut PixelBuffer, center: Point, direction: Direction, scale: i32, color: u8) {
    let half = scale / 2;
    let quarter = scale / 4;

    match direction {
        Direction::North => {
            draw_line_vertical(buffer, center.offset(0, -half), half, color);
            draw_line_ne(buffer, center.offset(0, -half), quarter, color);
            draw_line_nw(buffer, center.offset(0, -half), quarter, color);
        }
        Direction::NorthEast => {
            draw_line_ne(buffer, center, half, color);
            draw_line_vertical(buffer, center.offset(half, -quarter), quarter, color);
            draw_line_horizontal(buffer, center.offset(quarter, -quarter), quarter, color);
        }
        Direction::East => {
            draw_line_horizontal(buffer, center, half, color);
            draw_line_ne(buffer, center.offset(half, 0), -quarter, color);
            draw_line_se(buffer, center.offset(half, 0), -quarter, color);
        }
        Direction::SouthEast => {
            draw_line_se(buffer, center, half, color);
            draw_line_vertical(buffer, center.offset(half, quarter), -quarter, color);
            draw_line_horizontal(buffer, center.offset(quarter, quarter), quarter, color);
        }
        Direction::South => {
            draw_line_vertical(buffer, center, half, color);
            draw_line_se(buffer, center.offset(0, half), -quarter, color);
            draw_line_sw(buffer, center.offset(0, half), -quarter, color);
        }
        Direction::SouthWest => {
            draw_line_sw(buffer, center, half, color);
            draw_line_vertical(buffer, center.offset(-half, quarter), -quarter, color);
            draw_line_horizontal(buffer, center.offset(-quarter, quarter), -quarter, color);
        }
        Direction::West => {
            draw_line_horizontal(buffer, center.offset(-half, 0), half, color);
            draw_line_nw(buffer, center.offset(-half, 0), quarter, color);
            draw_line_sw(buffer, center.offset(-half, 0), quarter, color);
        }
        Direction::NorthWest => {
            draw_line_nw(buffer, center, half, color);
            draw_line_vertical(buffer, center.offset(-half, -quarter), quarter, color);
            draw_line_horizontal(buffer, center.offset(-quarter, -quarter), -quarter, color);
        }
        Direction::NoDirection => {}
    }
}

/// Draw dirt pattern (16 dots)
pub fn draw_dirt(buffer: &mut PixelBuffer, center: Point, scale: i32, color: u8) {
    let half = scale / 2;
    let quarter = scale / 4;
    let eighth = scale / 8;

    // 16 evenly spaced dots
    buffer.set_pixel(center.x - half - quarter, center.y + eighth, color);
    buffer.set_pixel(center.x - half, center.y, color);
    buffer.set_pixel(center.x - half, center.y + quarter, color);
    buffer.set_pixel(center.x - quarter, center.y - eighth, color);
    buffer.set_pixel(center.x - quarter, center.y + eighth, color);
    buffer.set_pixel(center.x - quarter, center.y + quarter + eighth, color);
    buffer.set_pixel(center.x, center.y - quarter, color);
    buffer.set_pixel(center.x, center.y, color);
    buffer.set_pixel(center.x, center.y + quarter, color);
    buffer.set_pixel(center.x, center.y + half, color);
    buffer.set_pixel(center.x + quarter, center.y - eighth, color);
    buffer.set_pixel(center.x + quarter, center.y + eighth, color);
    buffer.set_pixel(center.x + quarter, center.y + quarter + eighth, color);
    buffer.set_pixel(center.x + half, center.y, color);
    buffer.set_pixel(center.x + half, center.y + quarter, color);
    buffer.set_pixel(center.x + half + quarter, center.y + eighth, color);
}

/// Draw bridge pattern
pub fn draw_bridge(buffer: &mut PixelBuffer, center: Point, scale: i32, color: u8) {
    let quarter = scale / 4;
    let half = scale / 2;

    buffer.set_pixel(center.x, center.y, color);
    buffer.set_pixel(center.x + quarter, center.y - quarter / 2, color);
    buffer.set_pixel(center.x + quarter, center.y + quarter / 2, color);
    buffer.set_pixel(center.x + half, center.y, color);
    buffer.set_pixel(center.x + half, center.y + quarter, color);
    buffer.set_pixel(center.x + half + quarter, center.y + quarter / 2, color);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_automap_tile_type() {
        assert_eq!(AutomapTileType::from_u8(0), AutomapTileType::None);
        assert_eq!(AutomapTileType::from_u8(1), AutomapTileType::Diamond);
        assert_eq!(AutomapTileType::from_u8(51), AutomapTileType::Lava);
        assert_eq!(AutomapTileType::from_u8(59), AutomapTileType::PentagramOpen);
        assert_eq!(AutomapTileType::from_u8(255), AutomapTileType::None);
    }

    #[test]
    fn test_tile_type_categories() {
        assert!(AutomapTileType::Lava.is_lava());
        assert!(AutomapTileType::SELava.is_lava());
        assert!(!AutomapTileType::Diamond.is_lava());

        assert!(AutomapTileType::River.is_river());
        assert!(AutomapTileType::RiverCornerNorth.is_river());
        assert!(!AutomapTileType::Lava.is_river());

        assert!(AutomapTileType::CaveCross.is_cave());
        assert!(AutomapTileType::CaveHorizontalWood.is_cave());
        assert!(!AutomapTileType::Diamond.is_cave());
    }

    #[test]
    fn test_automap_tile_flags() {
        let flags = AutomapTileFlags::VERTICAL_DOOR | AutomapTileFlags::STAIRS;
        assert!(flags.contains(AutomapTileFlags::VERTICAL_DOOR));
        assert!(flags.contains(AutomapTileFlags::STAIRS));
        assert!(!flags.contains(AutomapTileFlags::HORIZONTAL_DOOR));

        assert!(flags.has_vertical_passage());
        assert!(!flags.has_horizontal_passage());
    }

    #[test]
    fn test_automap_tile() {
        let tile = AutomapTile::new(AutomapTileType::Diamond, AutomapTileFlags::STAIRS);
        assert_eq!(tile.tile_type, AutomapTileType::Diamond);
        assert!(tile.has_flag(AutomapTileFlags::STAIRS));
        assert!(!tile.has_flag(AutomapTileFlags::DIRT));
    }

    #[test]
    fn test_automap_manager_init() {
        let manager = AutomapManager::new(640, 480);
        assert!(!manager.active);
        assert_eq!(manager.scale, DEFAULT_AUTOMAP_SCALE);
        assert_eq!(manager.map_type, AutomapType::Opaque);
    }

    #[test]
    fn test_automap_manager_toggle() {
        let mut manager = AutomapManager::new(640, 480);
        assert!(!manager.active);

        manager.toggle();
        assert!(manager.active);

        manager.toggle();
        assert!(!manager.active);
    }

    #[test]
    fn test_automap_manager_zoom() {
        let mut manager = AutomapManager::new(640, 480);
        let initial_scale = manager.scale;

        manager.zoom_in();
        assert_eq!(manager.scale, initial_scale + 25);

        manager.zoom_out();
        assert_eq!(manager.scale, initial_scale);

        // Test limits
        manager.scale = MAX_AUTOMAP_SCALE;
        manager.zoom_in();
        assert_eq!(manager.scale, MAX_AUTOMAP_SCALE);

        manager.scale = MIN_AUTOMAP_SCALE;
        manager.zoom_out();
        assert_eq!(manager.scale, MIN_AUTOMAP_SCALE);
    }

    #[test]
    fn test_automap_manager_movement() {
        let mut manager = AutomapManager::new(640, 480);
        assert_eq!(manager.offset.dx, 0);
        assert_eq!(manager.offset.dy, 0);

        manager.move_up();
        assert_eq!(manager.offset.dy, -1);

        manager.move_down();
        assert_eq!(manager.offset.dy, 0);

        manager.move_left();
        assert_eq!(manager.offset.dx, -1);

        manager.move_right();
        assert_eq!(manager.offset.dx, 0);
    }

    #[test]
    fn test_automap_manager_view() {
        let mut manager = AutomapManager::new(640, 480);

        assert!(!manager.is_explored(5, 5));
        manager.set_view(5, 5);
        assert!(manager.is_explored(5, 5));

        // Out of bounds
        assert!(!manager.is_explored(100, 100));
        manager.set_view(100, 100); // Should not crash
    }

    #[test]
    fn test_point_and_rectangle() {
        let p = Point::new(10, 20);
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);

        let p2 = p.offset(5, -5);
        assert_eq!(p2.x, 15);
        assert_eq!(p2.y, 15);

        let rect = Rectangle::new(0, 0, 100, 50);
        assert!(rect.contains(Point::new(50, 25)));
        assert!(!rect.contains(Point::new(100, 50)));
        assert!(!rect.contains(Point::new(-1, 0)));
    }

    #[test]
    fn test_direction() {
        assert_eq!(Direction::from_u8(0), Direction::South);
        assert_eq!(Direction::from_u8(4), Direction::North);
        assert_eq!(Direction::from_u8(8), Direction::NoDirection);
        assert_eq!(Direction::from_u8(255), Direction::NoDirection);
    }

    #[test]
    fn test_pixel_buffer() {
        let mut buffer = PixelBuffer::new(100, 100);
        assert_eq!(buffer.get_pixel(50, 50), 0);

        buffer.set_pixel(50, 50, 128);
        assert_eq!(buffer.get_pixel(50, 50), 128);

        // Out of bounds should not crash
        buffer.set_pixel(-1, 50, 255);
        buffer.set_pixel(50, 200, 255);
        assert_eq!(buffer.get_pixel(-1, 50), 0);
        assert_eq!(buffer.get_pixel(50, 200), 0);
    }

    #[test]
    fn test_line_length() {
        let scale = 100;
        assert_eq!(AmLineLength::DoubleTile.to_pixels(scale), 200);
        assert_eq!(AmLineLength::FullTile.to_pixels(scale), 100);
        assert_eq!(AmLineLength::HalfTile.to_pixels(scale), 50);
        assert_eq!(AmLineLength::QuarterTile.to_pixels(scale), 25);
    }

    #[test]
    fn test_width_offset() {
        let scale = 100;
        assert_eq!(AmWidthOffset::HalfTileLeft.to_pixels(scale), -50);
        assert_eq!(AmWidthOffset::None.to_pixels(scale), 0);
        assert_eq!(AmWidthOffset::HalfTileRight.to_pixels(scale), 50);
    }

    #[test]
    fn test_height_offset() {
        let scale = 100;
        assert_eq!(AmHeightOffset::FullTileUp.to_pixels(scale), -50);
        assert_eq!(AmHeightOffset::None.to_pixels(scale), 0);
        assert_eq!(AmHeightOffset::FullTileDown.to_pixels(scale), 50);
    }

    #[test]
    fn test_draw_context() {
        let ctx = AutomapDrawContext::default();
        assert_eq!(ctx.scale, DEFAULT_AUTOMAP_SCALE);
        assert_eq!(ctx.color_player, MAP_COLOR_PLAYER);
        assert_eq!(ctx.color_bright, MAP_COLOR_BRIGHT);
    }

    #[test]
    fn test_caves_overrides() {
        let mut manager = AutomapManager::new(640, 480);
        manager.init(DungeonLevelType::Caves, &[]);

        // Check that caves-specific tiles are set
        assert_eq!(manager.get_tile(14).tile_type, AutomapTileType::HorizontalLavaThin);
        assert_eq!(manager.get_tile(32).tile_type, AutomapTileType::Lava);
        assert_eq!(manager.get_tile(137).tile_type, AutomapTileType::CaveWoodCross);
    }

    #[test]
    fn test_hell_overrides() {
        let mut manager = AutomapManager::new(640, 480);
        manager.init(DungeonLevelType::Hell, &[]);

        assert_eq!(manager.get_tile(51).tile_type, AutomapTileType::VerticalDiamond);
        assert_eq!(manager.get_tile(55).tile_type, AutomapTileType::HorizontalDiamond);
        assert_eq!(manager.get_tile(102).tile_type, AutomapTileType::PentagramClosed);
        assert_eq!(manager.get_tile(111).tile_type, AutomapTileType::PentagramOpen);
    }

    #[test]
    fn test_tile_to_screen() {
        let manager = AutomapManager::new(640, 480);
        let center = manager.get_screen_center();

        // Player at same position should be at center
        let screen_pos = manager.tile_to_screen(10, 10, 10, 10);
        assert_eq!(screen_pos.x, center.x);
        assert_eq!(screen_pos.y, center.y);

        // +x tile projects right and down (Diablo isometric: +x is screen-SE,
        // not NE). A true NE tile would be (+x, -y).
        let screen_pos = manager.tile_to_screen(11, 10, 10, 10);
        assert!(screen_pos.x > center.x);
        assert!(screen_pos.y > center.y);
    }

    #[test]
    fn test_lava_color() {
        let mut manager = AutomapManager::new(640, 480);

        manager.level_type = DungeonLevelType::Caves;
        assert_eq!(manager.get_lava_color(), MAP_COLOR_LAVA);

        manager.level_type = DungeonLevelType::Nest;
        assert_eq!(manager.get_lava_color(), MAP_COLOR_ACID);
    }
}
