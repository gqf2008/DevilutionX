//! Trigger System - Event triggers when player enters specific tiles
//!
//! This module manages level transition triggers (stairs, warps, quest returns).
//! When a player steps on a trigger tile, it initiates level changes or special events.
//!
//! C++ source: Source/levels/trigs.cpp (948 lines)
//! Target: ~600-700 lines Rust code

use crate::levels::types::{DungeonType, MAXDUNX, MAXDUNY};

/// Maximum number of triggers per level
pub const MAXTRIGGERS: usize = 7;

/// 2D position in dungeon coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// Trigger message types (what happens when player steps on trigger)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TriggerMessage {
    /// Go to next level (stairs down)
    NextLevel = 0,
    /// Go to previous level (stairs up)
    PrevLevel = 1,
    /// Return from set level to dungeon
    ReturnLevel = 2,
    /// Town portal warp
    TownWarp = 3,
    /// Warp up from dungeon to town
    TwarpUp = 4,
}

/// Trigger structure - defines a level transition trigger
///
/// C++ equivalent: TriggerStruct in Source/levels/trigs.h
#[derive(Debug, Clone, Copy)]
pub struct TriggerStruct {
    /// Trigger position in dungeon coordinates
    pub position: Point,
    /// Trigger message type (what action to take)
    pub tmsg: TriggerMessage,
    /// Target level (only used for TownWarp)
    pub tlvl: i32,
}

impl TriggerStruct {
    pub fn new(x: i32, y: i32, tmsg: TriggerMessage, tlvl: i32) -> Self {
        Self {
            position: Point { x, y },
            tmsg,
            tlvl,
        }
    }
}

/// Trigger manager - stores all active triggers for current level
pub struct TriggerManager {
    /// Array of active triggers (max 7 per level)
    pub trigs: [TriggerStruct; MAXTRIGGERS],
    /// Number of active triggers (0-7)
    pub numtrigs: usize,
    /// Trigger flag (set when player is on trigger tile)
    pub trigflag: bool,
    /// Source level for town warp (used when warping back)
    pub twarp_from: i32,
}

impl TriggerManager {
    pub fn new() -> Self {
        Self {
            trigs: [TriggerStruct::new(0, 0, TriggerMessage::NextLevel, 0); MAXTRIGGERS],
            numtrigs: 0,
            trigflag: false,
            twarp_from: 0,
        }
    }

    /// Initialize with no triggers (clear all)
    ///
    /// C++ equivalent: InitNoTriggers() in Source/levels/trigs.cpp:76-79
    pub fn init_no_triggers(&mut self) {
        self.numtrigs = 0;
        self.trigflag = false;
    }

    /// Add a trigger to the manager
    ///
    /// # Returns
    /// `true` if added successfully, `false` if trigger array is full
    pub fn add_trigger(&mut self, position: Point, tmsg: TriggerMessage, tlvl: i32) -> bool {
        if self.numtrigs >= MAXTRIGGERS {
            return false;
        }

        self.trigs[self.numtrigs] = TriggerStruct {
            position,
            tmsg,
            tlvl,
        };
        self.numtrigs += 1;
        true
    }

    /// Initialize triggers for Town level
    ///
    /// C++ equivalent: InitTownTriggers() in Source/levels/trigs.cpp:111-147
    ///
    /// Sets up:
    /// - Cathedral entrance (25, 29) - always available
    /// - Catacombs warp (49, 21) - if level 5 warp open
    /// - Caves warp (17, 69) - if level 9 warp open
    /// - Hell warp (41, 80) - if level 13 warp open
    /// - Nest warp (80, 62) - if level 17 warp open (Hellfire)
    /// - Crypt warp (36, 24) - if level 21 warp open (Hellfire)
    pub fn init_town_triggers(&mut self, is_warp_open: &dyn Fn(DungeonType) -> bool) {
        self.numtrigs = 0;

        // Cathedral entrance (always available)
        self.add_trigger(
            Point { x: 25, y: 29 },
            TriggerMessage::NextLevel,
            0,
        );

        // Catacombs warp (level 5)
        if is_warp_open(DungeonType::Catacombs) {
            self.add_trigger(Point { x: 49, y: 21 }, TriggerMessage::TownWarp, 5);
        }

        // Caves warp (level 9)
        if is_warp_open(DungeonType::Caves) {
            self.add_trigger(Point { x: 17, y: 69 }, TriggerMessage::TownWarp, 9);
        }

        // Hell warp (level 13)
        if is_warp_open(DungeonType::Hell) {
            self.add_trigger(Point { x: 41, y: 80 }, TriggerMessage::TownWarp, 13);
        }

        // Nest warp (level 17, Hellfire)
        if is_warp_open(DungeonType::Nest) {
            self.add_trigger(Point { x: 80, y: 62 }, TriggerMessage::TownWarp, 17);
        }

        // Crypt warp (level 21, Hellfire)
        if is_warp_open(DungeonType::Crypt) {
            self.add_trigger(Point { x: 36, y: 24 }, TriggerMessage::TownWarp, 21);
        }

        self.trigflag = false;
    }

    /// Initialize triggers for Cathedral (Level 1-4)
    ///
    /// C++ equivalent: InitL1Triggers() in Source/levels/trigs.cpp:149-167
    ///
    /// Scans dPiece array for:
    /// - Tile 128: Stairs up (to Town)
    /// - Tile 114: Stairs down (to next level)
    pub fn init_l1_triggers(&mut self, d_piece: &[[u16; MAXDUNY]; MAXDUNX]) {
        self.numtrigs = 0;

        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                if d_piece[i][j] == 128 {
                    self.add_trigger(
                        Point {
                            x: i as i32,
                            y: j as i32,
                        },
                        TriggerMessage::PrevLevel,
                        0,
                    );
                }
                if d_piece[i][j] == 114 {
                    self.add_trigger(
                        Point {
                            x: i as i32,
                            y: j as i32,
                        },
                        TriggerMessage::NextLevel,
                        0,
                    );
                }
            }
        }

        self.trigflag = false;
    }

    /// Initialize triggers for Catacombs (Level 5-8)
    ///
    /// C++ equivalent: InitL2Triggers() in Source/levels/trigs.cpp:169-195
    ///
    /// Scans dPiece array for:
    /// - Tile 266: Stairs up (to previous level) - except quest chamber position
    /// - Tile 558: Town portal warp up (to Town)
    /// - Tile 270: Stairs down (to next level)
    pub fn init_l2_triggers(
        &mut self,
        d_piece: &[[u16; MAXDUNY]; MAXDUNX],
        quest_chamber_pos: Option<Point>,
    ) {
        self.numtrigs = 0;

        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                let pos = Point {
                    x: i as i32,
                    y: j as i32,
                };

                // Stairs up (but not at quest chamber position)
                if d_piece[i][j] == 266 {
                    if let Some(qpos) = quest_chamber_pos {
                        if pos == qpos {
                            continue; // Skip quest chamber entrance
                        }
                    }
                    self.add_trigger(pos, TriggerMessage::PrevLevel, 0);
                }

                // Town portal warp up
                if d_piece[i][j] == 558 {
                    self.add_trigger(pos, TriggerMessage::TwarpUp, 0);
                }

                // Stairs down
                if d_piece[i][j] == 270 {
                    self.add_trigger(pos, TriggerMessage::NextLevel, 0);
                }
            }
        }

        self.trigflag = false;
    }

    /// Initialize triggers for Caves (Level 9-12)
    ///
    /// C++ equivalent: InitL3Triggers() in Source/levels/trigs.cpp:197-219
    ///
    /// Scans dPiece array for:
    /// - Tile 170: Stairs up (to previous level)
    /// - Tile 167: Stairs down (to next level)
    /// - Tile 548: Town portal warp up (to Town)
    pub fn init_l3_triggers(&mut self, d_piece: &[[u16; MAXDUNY]; MAXDUNX]) {
        self.numtrigs = 0;

        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                let pos = Point {
                    x: i as i32,
                    y: j as i32,
                };

                // Stairs up
                if d_piece[i][j] == 170 {
                    self.add_trigger(pos, TriggerMessage::PrevLevel, 0);
                }

                // Stairs down
                if d_piece[i][j] == 167 {
                    self.add_trigger(pos, TriggerMessage::NextLevel, 0);
                }

                // Town portal warp up
                if d_piece[i][j] == 548 {
                    self.add_trigger(pos, TriggerMessage::TwarpUp, 0);
                }
            }
        }

        self.trigflag = false;
    }

    /// Initialize triggers for Hell (Level 13-16)
    ///
    /// C++ equivalent: InitL4Triggers() in Source/levels/trigs.cpp:221-253
    ///
    /// Scans dPiece array for:
    /// - Tile 82: Stairs up (to previous level)
    /// - Tile 421: Town portal warp up (to Town)
    /// - Tile 119: Stairs down (to next level)
    /// - Tile 369: Pentagram (if Betrayer quest done) - entry to Diablo
    pub fn init_l4_triggers(
        &mut self,
        d_piece: &[[u16; MAXDUNY]; MAXDUNX],
        betrayer_quest_done: bool,
    ) {
        self.numtrigs = 0;

        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                let pos = Point {
                    x: i as i32,
                    y: j as i32,
                };

                // Stairs up
                if d_piece[i][j] == 82 {
                    self.add_trigger(pos, TriggerMessage::PrevLevel, 0);
                }

                // Town portal warp up
                if d_piece[i][j] == 421 {
                    self.add_trigger(pos, TriggerMessage::TwarpUp, 0);
                }

                // Stairs down
                if d_piece[i][j] == 119 {
                    self.add_trigger(pos, TriggerMessage::NextLevel, 0);
                }
            }
        }

        // Pentagram (Betrayer quest)
        if betrayer_quest_done {
            for j in 0..MAXDUNY {
                for i in 0..MAXDUNX {
                    if d_piece[i][j] == 369 {
                        self.add_trigger(
                            Point {
                                x: i as i32,
                                y: j as i32,
                            },
                            TriggerMessage::NextLevel,
                            0,
                        );
                    }
                }
            }
        }

        self.trigflag = false;
    }

    /// Initialize triggers for Hive (Nest levels, Hellfire)
    ///
    /// C++ equivalent: InitHiveTriggers() in Source/levels/trigs.cpp:255-277
    ///
    /// Scans dPiece array for:
    /// - Tile 65: Stairs up (to previous level)
    /// - Tile 62: Stairs down (to next level)
    /// - Tile 79: Town portal warp up (to Town)
    pub fn init_hive_triggers(&mut self, d_piece: &[[u16; MAXDUNY]; MAXDUNX]) {
        self.numtrigs = 0;

        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                let pos = Point {
                    x: i as i32,
                    y: j as i32,
                };

                // Stairs up
                if d_piece[i][j] == 65 {
                    self.add_trigger(pos, TriggerMessage::PrevLevel, 0);
                }

                // Stairs down
                if d_piece[i][j] == 62 {
                    self.add_trigger(pos, TriggerMessage::NextLevel, 0);
                }

                // Town portal warp up
                if d_piece[i][j] == 79 {
                    self.add_trigger(pos, TriggerMessage::TwarpUp, 0);
                }
            }
        }

        self.trigflag = false;
    }

    /// Initialize triggers for Crypt (Hellfire)
    ///
    /// C++ equivalent: InitCryptTriggers() in Source/levels/trigs.cpp:279-301
    ///
    /// Scans dPiece array for:
    /// - Tile 183: Town portal warp up (to Town)
    /// - Tile 157: Stairs up (to previous level)
    /// - Tile 125: Stairs down (to next level)
    pub fn init_crypt_triggers(&mut self, d_piece: &[[u16; MAXDUNY]; MAXDUNX]) {
        self.numtrigs = 0;

        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                let pos = Point {
                    x: i as i32,
                    y: j as i32,
                };

                // Town portal warp up
                if d_piece[i][j] == 183 {
                    self.add_trigger(pos, TriggerMessage::TwarpUp, 0);
                }

                // Stairs up
                if d_piece[i][j] == 157 {
                    self.add_trigger(pos, TriggerMessage::PrevLevel, 0);
                }

                // Stairs down
                if d_piece[i][j] == 125 {
                    self.add_trigger(pos, TriggerMessage::NextLevel, 0);
                }
            }
        }

        self.trigflag = false;
    }

    /// Initialize triggers for Skeleton King set level
    ///
    /// C++ equivalent: InitSKingTriggers() in Source/levels/trigs.cpp:303-307
    pub fn init_sking_triggers(&mut self) {
        self.trigflag = false;
        self.numtrigs = 1;
        self.trigs[0] = TriggerStruct::new(82, 42, TriggerMessage::ReturnLevel, 0);
    }

    /// Initialize triggers for Bone Chamber set level
    ///
    /// C++ equivalent: InitSChambTriggers() in Source/levels/trigs.cpp:309-313
    pub fn init_schamb_triggers(&mut self) {
        self.trigflag = false;
        self.numtrigs = 1;
        self.trigs[0] = TriggerStruct::new(70, 39, TriggerMessage::ReturnLevel, 0);
    }

    /// Initialize triggers for Poisoned Water Supply set level
    ///
    /// C++ equivalent: InitPWaterTriggers() in Source/levels/trigs.cpp:315-319
    pub fn init_pwater_triggers(&mut self) {
        self.trigflag = false;
        self.numtrigs = 1;
        self.trigs[0] = TriggerStruct::new(30, 83, TriggerMessage::ReturnLevel, 0);
    }

    /// Initialize triggers for Vile Betrayer set level
    ///
    /// C++ equivalent: InitVPTriggers() in Source/levels/trigs.cpp:321-325
    pub fn init_vp_triggers(&mut self) {
        self.trigflag = false;
        self.numtrigs = 1;
        self.trigs[0] = TriggerStruct::new(35, 32, TriggerMessage::ReturnLevel, 0);
    }
}

/// Check if a town warp is open (player can use portal to dungeon)
///
/// C++ equivalent: IsWarpOpen() in Source/levels/trigs.cpp:81-109
///
/// # Arguments
/// * `dtype` - Dungeon type to check
/// * `is_spawn` - Is this Diablo Shareware? (limits access to level 2)
/// * `is_multiplayer` - Is multiplayer mode active?
/// * `is_hellfire` - Is Hellfire expansion active?
/// * `player_level` - Player character level
/// * `town_warps` - Player town warps bitmask (bit 0=Catacombs, 1=Caves, 2=Hell)
/// * `farmer_quest_done` - Is Farmer quest done? (for Nest)
/// * `grave_quest_done` - Is Grave quest done? (for Crypt)
///
/// # Returns
/// `true` if the warp is open and player can use it
pub fn is_warp_open(
    dtype: DungeonType,
    is_spawn: bool,
    is_multiplayer: bool,
    is_hellfire: bool,
    player_level: i32,
    town_warps: u8,
    farmer_quest_done: bool,
    grave_quest_done: bool,
) -> bool {
    if is_spawn {
        return false;
    }

    // In multiplayer, all warps are open (except Nest, which requires quest)
    if is_multiplayer && dtype != DungeonType::Nest {
        return true;
    }

    // Check town warps bitmask (single player saves)
    match dtype {
        DungeonType::Catacombs if (town_warps & 1) != 0 => return true,
        DungeonType::Caves if (town_warps & 2) != 0 => return true,
        DungeonType::Hell if (town_warps & 4) != 0 => return true,
        _ => {}
    }

    // Hellfire level requirements
    if is_hellfire {
        match dtype {
            DungeonType::Catacombs if player_level >= 10 => return true,
            DungeonType::Caves if player_level >= 15 => return true,
            DungeonType::Hell if player_level >= 20 => return true,
            DungeonType::Nest if farmer_quest_done => return true,
            DungeonType::Crypt if grave_quest_done => return true,
            _ => {}
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_manager_new() {
        let mgr = TriggerManager::new();
        assert_eq!(mgr.numtrigs, 0);
        assert_eq!(mgr.trigflag, false);
        assert_eq!(mgr.twarp_from, 0);
    }

    #[test]
    fn test_init_no_triggers() {
        let mut mgr = TriggerManager::new();
        mgr.numtrigs = 5;
        mgr.trigflag = true;

        mgr.init_no_triggers();

        assert_eq!(mgr.numtrigs, 0);
        assert_eq!(mgr.trigflag, false);
    }

    #[test]
    fn test_add_trigger() {
        let mut mgr = TriggerManager::new();

        let result = mgr.add_trigger(Point { x: 10, y: 20 }, TriggerMessage::NextLevel, 0);
        assert!(result);
        assert_eq!(mgr.numtrigs, 1);
        assert_eq!(mgr.trigs[0].position, Point { x: 10, y: 20 });
        assert_eq!(mgr.trigs[0].tmsg, TriggerMessage::NextLevel);
    }

    #[test]
    fn test_add_trigger_max_limit() {
        let mut mgr = TriggerManager::new();

        // Add 7 triggers (max)
        for i in 0..MAXTRIGGERS {
            let result = mgr.add_trigger(
                Point { x: i as i32, y: 0 },
                TriggerMessage::NextLevel,
                0,
            );
            assert!(result);
        }

        assert_eq!(mgr.numtrigs, MAXTRIGGERS);

        // Try to add 8th trigger (should fail)
        let result = mgr.add_trigger(Point { x: 99, y: 99 }, TriggerMessage::NextLevel, 0);
        assert!(!result);
        assert_eq!(mgr.numtrigs, MAXTRIGGERS);
    }

    #[test]
    fn test_init_town_triggers_all_closed() {
        let mut mgr = TriggerManager::new();
        let is_warp_open = |_: DungeonType| false;

        mgr.init_town_triggers(&is_warp_open);

        // Only Cathedral entrance should be present
        assert_eq!(mgr.numtrigs, 1);
        assert_eq!(mgr.trigs[0].position, Point { x: 25, y: 29 });
        assert_eq!(mgr.trigs[0].tmsg, TriggerMessage::NextLevel);
    }

    #[test]
    fn test_init_town_triggers_all_open() {
        let mut mgr = TriggerManager::new();
        let is_warp_open = |_: DungeonType| true;

        mgr.init_town_triggers(&is_warp_open);

        // Cathedral + 5 warps = 6 triggers
        assert_eq!(mgr.numtrigs, 6);

        // Check specific warp positions
        assert_eq!(mgr.trigs[0].position, Point { x: 25, y: 29 }); // Cathedral
        assert_eq!(mgr.trigs[1].position, Point { x: 49, y: 21 }); // Catacombs
        assert_eq!(mgr.trigs[1].tlvl, 5);
        assert_eq!(mgr.trigs[2].position, Point { x: 17, y: 69 }); // Caves
        assert_eq!(mgr.trigs[2].tlvl, 9);
        assert_eq!(mgr.trigs[3].position, Point { x: 41, y: 80 }); // Hell
        assert_eq!(mgr.trigs[3].tlvl, 13);
    }

    #[test]
    fn test_init_l1_triggers() {
        let mut mgr = TriggerManager::new();
        let mut d_piece = [[0u16; MAXDUNY]; MAXDUNX];

        // Place stairs up and down
        d_piece[10][20] = 128; // Stairs up
        d_piece[30][40] = 114; // Stairs down

        mgr.init_l1_triggers(&d_piece);

        assert_eq!(mgr.numtrigs, 2);
        assert_eq!(mgr.trigs[0].position, Point { x: 10, y: 20 });
        assert_eq!(mgr.trigs[0].tmsg, TriggerMessage::PrevLevel);
        assert_eq!(mgr.trigs[1].position, Point { x: 30, y: 40 });
        assert_eq!(mgr.trigs[1].tmsg, TriggerMessage::NextLevel);
    }

    #[test]
    fn test_init_l2_triggers_with_quest_chamber() {
        let mut mgr = TriggerManager::new();
        let mut d_piece = [[0u16; MAXDUNY]; MAXDUNX];

        // Place stairs and warp
        d_piece[10][20] = 266; // Stairs up (at quest chamber position)
        d_piece[15][25] = 266; // Stairs up (normal)
        d_piece[30][40] = 558; // Town warp
        d_piece[50][60] = 270; // Stairs down

        let quest_pos = Some(Point { x: 10, y: 20 });

        mgr.init_l2_triggers(&d_piece, quest_pos);

        // Should have 3 triggers (quest chamber entrance excluded)
        assert_eq!(mgr.numtrigs, 3);

        // First trigger should be normal stairs up (not quest chamber)
        assert_eq!(mgr.trigs[0].position, Point { x: 15, y: 25 });
        assert_eq!(mgr.trigs[0].tmsg, TriggerMessage::PrevLevel);
    }

    #[test]
    fn test_init_l4_triggers_with_betrayer() {
        let mut mgr = TriggerManager::new();
        let mut d_piece = [[0u16; MAXDUNY]; MAXDUNX];

        // Place normal stairs
        d_piece[10][20] = 82; // Stairs up
        d_piece[30][40] = 119; // Stairs down
        d_piece[50][60] = 369; // Pentagram

        mgr.init_l4_triggers(&d_piece, true); // Betrayer quest done

        // Should have 4 triggers (up + down + pentagram)
        assert_eq!(mgr.numtrigs, 3);

        // Pentagram should be added
        assert!(mgr
            .trigs
            .iter()
            .take(mgr.numtrigs)
            .any(|t| t.position == Point { x: 50, y: 60 }));
    }

    #[test]
    fn test_is_warp_open_spawn_blocked() {
        let result = is_warp_open(
            DungeonType::Catacombs,
            true,  // is_spawn (shareware)
            false, // is_multiplayer
            false, // is_hellfire
            20,    // player_level
            0xFF,  // town_warps (all bits set)
            true,  // farmer_quest_done
            true,  // grave_quest_done
        );

        assert!(!result); // Shareware blocks all warps
    }

    #[test]
    fn test_is_warp_open_multiplayer() {
        let result = is_warp_open(
            DungeonType::Catacombs,
            false, // is_spawn
            true,  // is_multiplayer
            false, // is_hellfire
            1,     // player_level (low)
            0,     // town_warps (none)
            false, // farmer_quest_done
            false, // grave_quest_done
        );

        assert!(result); // Multiplayer opens all warps except Nest
    }

    #[test]
    fn test_is_warp_open_hellfire_level_requirement() {
        // Catacombs requires level 10
        let result_low = is_warp_open(
            DungeonType::Catacombs,
            false, // is_spawn
            false, // is_multiplayer
            true,  // is_hellfire
            9,     // player_level (too low)
            0,     // town_warps
            false, // farmer_quest_done
            false, // grave_quest_done
        );
        assert!(!result_low);

        let result_ok = is_warp_open(
            DungeonType::Catacombs,
            false, // is_spawn
            false, // is_multiplayer
            true,  // is_hellfire
            10,    // player_level (meets requirement)
            0,     // town_warps
            false, // farmer_quest_done
            false, // grave_quest_done
        );
        assert!(result_ok);
    }

    #[test]
    fn test_is_warp_open_nest_quest_requirement() {
        let result_no_quest = is_warp_open(
            DungeonType::Nest,
            false, // is_spawn
            false, // is_multiplayer
            true,  // is_hellfire
            20,    // player_level (high)
            0xFF,  // town_warps (all bits)
            false, // farmer_quest_done (NOT done)
            false, // grave_quest_done
        );
        assert!(!result_no_quest); // Nest requires Farmer quest

        let result_quest_done = is_warp_open(
            DungeonType::Nest,
            false, // is_spawn
            false, // is_multiplayer
            true,  // is_hellfire
            20,    // player_level
            0,     // town_warps
            true,  // farmer_quest_done (DONE)
            false, // grave_quest_done
        );
        assert!(result_quest_done);
    }
}
