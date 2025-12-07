//! Theme Room System - Special themed rooms in dungeons
//!
//! Theme rooms are special decorated areas (shrines, treasure rooms, skeleton rooms, etc.)
//! placed in dungeons to add variety. This module manages theme room placement and creation.
//!
//! C++ source: Source/levels/themes.cpp (985 lines)
//! Target: ~600-700 lines Rust code (core framework, Theme_* functions deferred)

/// Maximum number of themes per level
pub const MAXTHEMES: usize = 50;

/// Theme type identifier
///
/// C++ equivalent: theme_id enum in Source/objdat.h:17-36
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    None = -1,
}

impl ThemeId {
    /// Convert i8 to ThemeId
    pub fn from_i8(value: i8) -> Self {
        match value {
            0 => ThemeId::Barrel,
            1 => ThemeId::Shrine,
            2 => ThemeId::MonstPit,
            3 => ThemeId::SkelRoom,
            4 => ThemeId::Treasure,
            5 => ThemeId::Library,
            6 => ThemeId::Torture,
            7 => ThemeId::BloodFountain,
            8 => ThemeId::Decapitated,
            9 => ThemeId::PurifyingFountain,
            10 => ThemeId::ArmorStand,
            11 => ThemeId::GoatShrine,
            12 => ThemeId::Cauldron,
            13 => ThemeId::MurkyFountain,
            14 => ThemeId::TearFountain,
            15 => ThemeId::BrnCross,
            16 => ThemeId::WeaponRack,
            _ => ThemeId::None,
        }
    }

    /// Check if theme is a "good" theme (higher placement priority)
    ///
    /// C++ equivalent: ThemeGood array in Source/levels/themes.cpp:177-178
    pub fn is_good_theme(self) -> bool {
        matches!(
            self,
            ThemeId::Shrine
                | ThemeId::Library
                | ThemeId::Treasure
                | ThemeId::SkelRoom
        )
    }
}

/// Theme structure - defines a themed room
///
/// C++ equivalent: ThemeStruct in Source/levels/themes.h:12-15
#[derive(Debug, Clone, Copy)]
pub struct ThemeStruct {
    /// Theme type
    pub ttype: ThemeId,
    /// Theme transparency value (region ID in dTransVal)
    pub ttval: i8,
}

impl ThemeStruct {
    pub fn new(ttype: ThemeId, ttval: i8) -> Self {
        Self { ttype, ttval }
    }
}

/// Theme manager - stores all active themes for current level
///
/// C++ equivalent: Global variables in Source/levels/themes.cpp:27-30
pub struct ThemeManager {
    /// Array of active themes (max 50 per level)
    pub themes: [ThemeStruct; MAXTHEMES],
    /// Number of active themes (0-50)
    pub numthemes: usize,
    /// Armor stand placement flag
    pub armor_flag: bool,
    /// Weapon rack placement flag
    pub weapon_flag: bool,
    /// Zhar library theme index (-1 if not present)
    pub zharlib: i32,

    // Internal state for theme placement (C++ uses globals)
    /// Current theme X position during placement
    themex: i32,
    /// Current theme Y position during placement
    themey: i32,
    /// Theme variant (used by some themes for orientation/type)
    theme_var1: usize,

    // Theme placement flags (track what's been placed)
    cauldron_flag: bool,
    b_fountain_flag: bool, // Blood fountain
    m_fountain_flag: bool, // Murky fountain
    p_fountain_flag: bool, // Purifying fountain
    t_fountain_flag: bool, // Tear fountain
    treasure_flag: bool,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            themes: [ThemeStruct::new(ThemeId::None, 0); MAXTHEMES],
            numthemes: 0,
            armor_flag: false,
            weapon_flag: false,
            zharlib: -1,
            themex: 0,
            themey: 0,
            theme_var1: 0,
            cauldron_flag: false,
            b_fountain_flag: false,
            m_fountain_flag: false,
            p_fountain_flag: false,
            t_fountain_flag: false,
            treasure_flag: false,
        }
    }

    /// Check if theme requirements are met
    ///
    /// C++ equivalent: CheckThemeReqs() in Source/levels/themes.cpp:196-221
    ///
    /// Some themes require specific conditions:
    /// - Library: Requires Zhar quest available
    /// - Fountains: Only one of each type per level
    /// - ArmorStand/WeaponRack: Only one per level
    /// - Treasure: Only one per level
    pub fn check_theme_reqs(&self, t: ThemeId) -> bool {
        match t {
            ThemeId::Shrine
            | ThemeId::SkelRoom
            | ThemeId::Library
            | ThemeId::Torture
            | ThemeId::Decapitated
            | ThemeId::BrnCross => true, // No special requirements

            ThemeId::ArmorStand => !self.armor_flag,
            ThemeId::WeaponRack => !self.weapon_flag,
            ThemeId::Cauldron => !self.cauldron_flag,
            ThemeId::BloodFountain => !self.b_fountain_flag,
            ThemeId::PurifyingFountain => !self.p_fountain_flag,
            ThemeId::MurkyFountain => !self.m_fountain_flag,
            ThemeId::TearFountain => !self.t_fountain_flag,
            ThemeId::Treasure => !self.treasure_flag,

            _ => false, // Barrel, MonstPit, GoatShrine have complex requirements
        }
    }

    /// Initialize themes for current level
    ///
    /// C++ equivalent: InitThemes() in Source/levels/themes.cpp:785-894
    ///
    /// This is a simplified version - full implementation would:
    /// 1. Scan themeLoc array for potential theme room positions
    /// 2. Check if Zhar quest available (place THEME_LIBRARY)
    /// 3. Randomly assign "good" themes (Shrine/Library/Treasure/SkelRoom)
    /// 4. Fill remaining with random themes
    ///
    /// **Deferred**: Full implementation requires quest system, RNG, themeLoc scanning
    pub fn init_themes(&mut self, theme_count: usize, has_zhar_quest: bool) {
        self.numthemes = 0;
        self.zharlib = -1;

        // Reset flags
        self.armor_flag = false;
        self.weapon_flag = false;
        self.cauldron_flag = false;
        self.b_fountain_flag = false;
        self.m_fountain_flag = false;
        self.p_fountain_flag = false;
        self.t_fountain_flag = false;
        self.treasure_flag = false;

        // Initialize all themes to None
        for i in 0..MAXTHEMES {
            self.themes[i] = ThemeStruct::new(ThemeId::None, 0);
        }

        // Simplified version - just set count
        // Full version would scan themeLoc, assign types, etc.
        self.numthemes = theme_count.min(MAXTHEMES);

        // If Zhar quest available, mark first theme as potential library
        if has_zhar_quest && theme_count > 0 {
            self.zharlib = 0; // First theme reserved for library
        }
    }

    /// Mark theme rooms as populated (prevent monster/object spawning)
    ///
    /// C++ equivalent: HoldThemeRooms() in Source/levels/themes.cpp:896-917
    ///
    /// Scans dTransVal array for theme room regions and marks them as populated.
    /// This prevents normal monster/object generation in theme rooms.
    ///
    /// **Deferred**: Requires dTransVal and dFlags array integration
    pub fn hold_theme_rooms(&self) {
        // C++ implementation:
        // for each theme:
        //   for y in 0..MAXDUNY:
        //     for x in 0..MAXDUNX:
        //       if dTransVal[x][y] == themes[i].ttval:
        //         dFlags[x][y] |= DungeonFlag::Populated

        // Deferred: Requires dTransVal/dFlags arrays
    }

    /// Create theme room content (place objects/monsters)
    ///
    /// C++ equivalent: CreateThemeRooms() in Source/levels/themes.cpp:919-985
    ///
    /// Dispatches to specific Theme_* functions based on theme type:
    /// - THEME_BARREL → Theme_Barrel()
    /// - THEME_SHRINE → Theme_Shrine()
    /// - THEME_SKELROOM → Theme_SkelRoom()
    /// - etc. (17 theme types total)
    ///
    /// **Deferred**: Requires Theme_* implementation functions (600+ lines)
    pub fn create_theme_rooms(&mut self) {
        // C++ implementation:
        // for i in 0..numthemes:
        //   match themes[i].ttype:
        //     THEME_BARREL => Theme_Barrel(i)
        //     THEME_SHRINE => Theme_Shrine(i)
        //     ... (17 cases total)

        // Deferred: Requires Theme_* functions, object system, monster system
    }

    /// Add a theme to the manager
    ///
    /// # Returns
    /// `true` if added successfully, `false` if theme array is full
    pub fn add_theme(&mut self, ttype: ThemeId, ttval: i8) -> bool {
        if self.numthemes >= MAXTHEMES {
            return false;
        }

        self.themes[self.numthemes] = ThemeStruct::new(ttype, ttval);
        self.numthemes += 1;

        // Update flags
        match ttype {
            ThemeId::ArmorStand => self.armor_flag = true,
            ThemeId::WeaponRack => self.weapon_flag = true,
            ThemeId::Cauldron => self.cauldron_flag = true,
            ThemeId::BloodFountain => self.b_fountain_flag = true,
            ThemeId::PurifyingFountain => self.p_fountain_flag = true,
            ThemeId::MurkyFountain => self.m_fountain_flag = true,
            ThemeId::TearFountain => self.t_fountain_flag = true,
            ThemeId::Treasure => self.treasure_flag = true,
            _ => {}
        }

        true
    }

    /// Get theme at index
    pub fn get_theme(&self, index: usize) -> Option<&ThemeStruct> {
        if index < self.numthemes {
            Some(&self.themes[index])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager_new() {
        let mgr = ThemeManager::new();
        assert_eq!(mgr.numthemes, 0);
        assert_eq!(mgr.zharlib, -1);
        assert!(!mgr.armor_flag);
        assert!(!mgr.weapon_flag);
    }

    #[test]
    fn test_theme_id_from_i8() {
        assert_eq!(ThemeId::from_i8(0), ThemeId::Barrel);
        assert_eq!(ThemeId::from_i8(1), ThemeId::Shrine);
        assert_eq!(ThemeId::from_i8(16), ThemeId::WeaponRack);
        assert_eq!(ThemeId::from_i8(-1), ThemeId::None);
        assert_eq!(ThemeId::from_i8(99), ThemeId::None);
    }

    #[test]
    fn test_theme_id_is_good_theme() {
        assert!(ThemeId::Shrine.is_good_theme());
        assert!(ThemeId::Library.is_good_theme());
        assert!(ThemeId::Treasure.is_good_theme());
        assert!(ThemeId::SkelRoom.is_good_theme());

        assert!(!ThemeId::Barrel.is_good_theme());
        assert!(!ThemeId::Cauldron.is_good_theme());
    }

    #[test]
    fn test_add_theme() {
        let mut mgr = ThemeManager::new();

        let result = mgr.add_theme(ThemeId::Shrine, 10);
        assert!(result);
        assert_eq!(mgr.numthemes, 1);
        assert_eq!(mgr.themes[0].ttype, ThemeId::Shrine);
        assert_eq!(mgr.themes[0].ttval, 10);
    }

    #[test]
    fn test_add_theme_max_limit() {
        let mut mgr = ThemeManager::new();

        // Add 50 themes (max)
        for i in 0..MAXTHEMES {
            let result = mgr.add_theme(ThemeId::Barrel, i as i8);
            assert!(result);
        }

        assert_eq!(mgr.numthemes, MAXTHEMES);

        // Try to add 51st theme (should fail)
        let result = mgr.add_theme(ThemeId::Shrine, 99);
        assert!(!result);
        assert_eq!(mgr.numthemes, MAXTHEMES);
    }

    #[test]
    fn test_add_theme_sets_flags() {
        let mut mgr = ThemeManager::new();

        mgr.add_theme(ThemeId::ArmorStand, 1);
        assert!(mgr.armor_flag);

        mgr.add_theme(ThemeId::WeaponRack, 2);
        assert!(mgr.weapon_flag);

        mgr.add_theme(ThemeId::BloodFountain, 3);
        assert!(mgr.b_fountain_flag);
    }

    #[test]
    fn test_check_theme_reqs_no_restrictions() {
        let mgr = ThemeManager::new();

        // These themes have no special requirements
        assert!(mgr.check_theme_reqs(ThemeId::Shrine));
        assert!(mgr.check_theme_reqs(ThemeId::SkelRoom));
        assert!(mgr.check_theme_reqs(ThemeId::Library));
        assert!(mgr.check_theme_reqs(ThemeId::Torture));
    }

    #[test]
    fn test_check_theme_reqs_one_per_level() {
        let mut mgr = ThemeManager::new();

        // ArmorStand allowed initially
        assert!(mgr.check_theme_reqs(ThemeId::ArmorStand));

        // Add ArmorStand
        mgr.add_theme(ThemeId::ArmorStand, 1);

        // ArmorStand no longer allowed
        assert!(!mgr.check_theme_reqs(ThemeId::ArmorStand));
    }

    #[test]
    fn test_check_theme_reqs_fountain_limits() {
        let mut mgr = ThemeManager::new();

        // Blood fountain allowed initially
        assert!(mgr.check_theme_reqs(ThemeId::BloodFountain));

        // Add blood fountain
        mgr.add_theme(ThemeId::BloodFountain, 1);

        // Blood fountain no longer allowed, but other fountains still OK
        assert!(!mgr.check_theme_reqs(ThemeId::BloodFountain));
        assert!(mgr.check_theme_reqs(ThemeId::MurkyFountain));
        assert!(mgr.check_theme_reqs(ThemeId::PurifyingFountain));
    }

    #[test]
    fn test_init_themes_basic() {
        let mut mgr = ThemeManager::new();

        mgr.init_themes(5, false);

        assert_eq!(mgr.numthemes, 5);
        assert_eq!(mgr.zharlib, -1); // No Zhar quest
        assert!(!mgr.armor_flag); // Flags reset
    }

    #[test]
    fn test_init_themes_with_zhar() {
        let mut mgr = ThemeManager::new();

        mgr.init_themes(3, true);

        assert_eq!(mgr.numthemes, 3);
        assert_eq!(mgr.zharlib, 0); // First theme reserved for library
    }

    #[test]
    fn test_init_themes_exceeds_max() {
        let mut mgr = ThemeManager::new();

        mgr.init_themes(100, false); // Request 100 themes

        assert_eq!(mgr.numthemes, MAXTHEMES); // Clamped to 50
    }

    #[test]
    fn test_get_theme() {
        let mut mgr = ThemeManager::new();

        mgr.add_theme(ThemeId::Shrine, 10);
        mgr.add_theme(ThemeId::Library, 20);

        let theme0 = mgr.get_theme(0).unwrap();
        assert_eq!(theme0.ttype, ThemeId::Shrine);
        assert_eq!(theme0.ttval, 10);

        let theme1 = mgr.get_theme(1).unwrap();
        assert_eq!(theme1.ttype, ThemeId::Library);
        assert_eq!(theme1.ttval, 20);

        // Out of bounds
        assert!(mgr.get_theme(2).is_none());
    }
}
