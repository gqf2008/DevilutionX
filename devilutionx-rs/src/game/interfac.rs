// interfac.rs - Load screen interface system
// Ported from Source/interfac.cpp (694 lines)

use std::time::{Duration, Instant};

/// Maximum progress value for progress bar
pub const MAX_PROGRESS: u32 = 534;

/// Progress step size per increment
pub const PROGRESS_STEP_SIZE: u32 = 23;

/// Progress bar height in pixels
pub const PROGRESS_HEIGHT: i32 = 22;

/// Interface mode / Custom events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InterfaceMode {
    /// Go to next level
    DiabNextLevel = 0,
    /// Go to previous level
    DiabPrevLevel,
    /// Return to level
    DiabReturnLevel,
    /// Set level (special levels)
    DiabSetLevel,
    /// Warp level (portal)
    DiabWarpLevel,
    /// Town warp down
    DiabTownWarp,
    /// Town warp up
    DiabTownWarpUp,
    /// Return to town
    DiabReTown,
    /// New game
    DiabNewGame,
    /// Load game
    DiabLoadGame,
    /// Progress update (async)
    Progress,
    /// Error event (async)
    Error,
    /// Done event (async)
    Done,
}

impl InterfaceMode {
    pub const FIRST: Self = Self::DiabNextLevel;
    pub const LAST: Self = Self::Done;

    /// Check if this is a level transition mode
    pub fn is_level_transition(&self) -> bool {
        matches!(
            self,
            Self::DiabNextLevel
                | Self::DiabPrevLevel
                | Self::DiabReturnLevel
                | Self::DiabSetLevel
                | Self::DiabWarpLevel
                | Self::DiabTownWarp
                | Self::DiabTownWarpUp
                | Self::DiabReTown
        )
    }

    /// Check if this is a game start mode
    pub fn is_game_start(&self) -> bool {
        matches!(self, Self::DiabNewGame | Self::DiabLoadGame)
    }

    /// Check if this is an async event
    pub fn is_async_event(&self) -> bool {
        matches!(self, Self::Progress | Self::Error | Self::Done)
    }
}

impl TryFrom<u8> for InterfaceMode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, <Self as TryFrom<u8>>::Error> {
        match value {
            0 => Ok(Self::DiabNextLevel),
            1 => Ok(Self::DiabPrevLevel),
            2 => Ok(Self::DiabReturnLevel),
            3 => Ok(Self::DiabSetLevel),
            4 => Ok(Self::DiabWarpLevel),
            5 => Ok(Self::DiabTownWarp),
            6 => Ok(Self::DiabTownWarpUp),
            7 => Ok(Self::DiabReTown),
            8 => Ok(Self::DiabNewGame),
            9 => Ok(Self::DiabLoadGame),
            10 => Ok(Self::Progress),
            11 => Ok(Self::Error),
            12 => Ok(Self::Done),
            _ => Err(()),
        }
    }
}

/// Cutscene types for loading screens
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Cutscene {
    Start = 0,
    Town,
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
    Level6,
    Portal,
    PortalRed,
    Gate,
}

impl Cutscene {
    /// Get the CEL file path for this cutscene
    pub fn cel_path(&self) -> &'static str {
        match self {
            Self::Start => "gendata\\cutstart",
            Self::Town => "gendata\\cuttt",
            Self::Level1 => "gendata\\cutl1d",
            Self::Level2 => "gendata\\cut2",
            Self::Level3 => "gendata\\cut3",
            Self::Level4 => "gendata\\cut4",
            Self::Level5 => "nlevels\\cutl5",
            Self::Level6 => "nlevels\\cutl6",
            Self::Portal => "gendata\\cutportl",
            Self::PortalRed => "gendata\\cutportr",
            Self::Gate => "gendata\\cutgate",
        }
    }

    /// Get the palette file path for this cutscene
    pub fn pal_path(&self) -> &'static str {
        match self {
            Self::Start => "gendata\\cutstart.pal",
            Self::Town => "gendata\\cuttt.pal",
            Self::Level1 => "gendata\\cutl1d.pal",
            Self::Level2 => "gendata\\cut2.pal",
            Self::Level3 => "gendata\\cut3.pal",
            Self::Level4 => "gendata\\cut4.pal",
            Self::Level5 => "nlevels\\cutl5.pal",
            Self::Level6 => "nlevels\\cutl6.pal",
            Self::Portal => "gendata\\cutportl.pal",
            Self::PortalRed => "gendata\\cutportr.pal",
            Self::Gate => "gendata\\cutgate.pal",
        }
    }

    /// Get widescreen CLX file path for this cutscene
    pub fn widescreen_clx_path(&self) -> &'static str {
        match self {
            Self::Start => "gendata\\cutstartw.clx",
            Self::Town => "gendata\\cutttw.clx",
            Self::Level1 => "gendata\\cutl1dw.clx",
            Self::Level2 => "gendata\\cut2w.clx",
            Self::Level3 => "gendata\\cut3w.clx",
            Self::Level4 => "gendata\\cut4w.clx",
            Self::Level5 => "nlevels\\cutl5w.clx",
            Self::Level6 => "nlevels\\cutl6w.clx",
            Self::Portal => "gendata\\cutportlw.clx",
            Self::PortalRed => "gendata\\cutportrw.clx",
            Self::Gate => "gendata\\cutgatew.clx",
        }
    }

    /// Get progress bar color index for this cutscene
    pub fn progress_id(&self) -> usize {
        match self {
            Self::Level1 => 0,
            Self::Start | Self::Town | Self::Level3 | Self::Level4
            | Self::Level5 | Self::Level6 | Self::Portal
            | Self::PortalRed | Self::Gate => 1,
            Self::Level2 => 2,
        }
    }
}

/// Dungeon type for cutscene selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonType {
    Town,
    Cathedral,
    Catacombs,
    Caves,
    Hell,
    Nest,
    Crypt,
}

impl DungeonType {
    /// Get cutscene for this dungeon type
    pub fn to_cutscene(&self) -> Cutscene {
        match self {
            Self::Town => Cutscene::Town,
            Self::Cathedral => Cutscene::Level1,
            Self::Catacombs => Cutscene::Level2,
            Self::Caves => Cutscene::Level3,
            Self::Hell => Cutscene::Level4,
            Self::Nest => Cutscene::Level6,
            Self::Crypt => Cutscene::Level5,
        }
    }
}

/// Progress bar color palette indices
pub const BAR_COLORS: [u8; 3] = [138, 43, 254];

/// Progress bar positions (x, y) for each progress_id
pub const BAR_POSITIONS: [[i32; 2]; 3] = [
    [53, 37],   // Level1
    [53, 421],  // Most cutscenes
    [53, 37],   // Level2
];

/// Progress event handler state
#[derive(Debug)]
pub struct ProgressState {
    pub load_started_at: Option<Instant>,
    pub skip_rendering: bool,
    pub done: bool,
    pub drawn_progress: u32,
    pub palette: [u32; 256],
}

impl Default for ProgressState {
    fn default() -> Self {
        Self {
            load_started_at: None,
            skip_rendering: true,
            done: false,
            drawn_progress: 0,
            palette: [0; 256],
        }
    }
}

impl ProgressState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset state for new progress
    pub fn reset(&mut self) {
        self.load_started_at = Some(Instant::now());
        self.skip_rendering = true;
        self.done = false;
        self.drawn_progress = 0;
    }

    /// Get elapsed time since load started
    pub fn elapsed(&self) -> Option<Duration> {
        self.load_started_at.map(|t| t.elapsed())
    }
}

/// Interface manager for load screens
pub struct InterfaceManager {
    /// Current progress value
    progress: u32,
    /// Progress ID (determines bar position/color)
    progress_id: usize,
    /// Is progress screen active
    is_progress: bool,
    /// Progress state
    state: ProgressState,
    /// Skip threshold in milliseconds
    skip_threshold_ms: u32,
    /// Is multiplayer
    is_multiplayer: bool,
    /// Is hellfire
    is_hellfire: bool,
    /// Current level
    current_level: u8,
    /// Set level number
    set_level_num: u8,
    /// Set level type
    set_level_type: Option<DungeonType>,
}

impl InterfaceManager {
    pub fn new() -> Self {
        Self {
            progress: 0,
            progress_id: 1,
            is_progress: false,
            state: ProgressState::new(),
            skip_threshold_ms: 0,
            is_multiplayer: false,
            is_hellfire: false,
            current_level: 0,
            set_level_num: 0,
            set_level_type: None,
        }
    }

    /// Get current progress value
    pub fn progress(&self) -> u32 {
        self.progress
    }

    /// Get progress ID
    pub fn progress_id(&self) -> usize {
        self.progress_id
    }

    /// Check if progress is active
    pub fn is_progress(&self) -> bool {
        self.is_progress
    }

    /// Set skip threshold
    pub fn set_skip_threshold(&mut self, ms: u32) {
        self.skip_threshold_ms = ms;
    }

    /// Set multiplayer mode
    pub fn set_multiplayer(&mut self, multiplayer: bool) {
        self.is_multiplayer = multiplayer;
    }

    /// Set hellfire mode
    pub fn set_hellfire(&mut self, hellfire: bool) {
        self.is_hellfire = hellfire;
    }

    /// Set current level
    pub fn set_current_level(&mut self, level: u8) {
        self.current_level = level;
    }

    /// Set special level info
    pub fn set_set_level(&mut self, num: u8, level_type: DungeonType) {
        self.set_level_num = num;
        self.set_level_type = Some(level_type);
    }

    /// Get level type for a level number
    pub fn get_level_type(&self, level: u8) -> DungeonType {
        if level == 0 {
            return DungeonType::Town;
        }

        let num_levels = if self.is_hellfire { 25 } else { 17 };

        match level {
            1..=4 => DungeonType::Cathedral,
            5..=8 => DungeonType::Catacombs,
            9..=12 => DungeonType::Caves,
            13..=16 => DungeonType::Hell,
            _ if level <= num_levels && self.is_hellfire => {
                if level <= 20 {
                    DungeonType::Crypt
                } else {
                    DungeonType::Nest
                }
            }
            _ => DungeonType::Cathedral,
        }
    }

    /// Pick cutscene for interface mode
    pub fn pick_cutscene(&self, mode: InterfaceMode) -> Cutscene {
        match mode {
            InterfaceMode::DiabLoadGame | InterfaceMode::DiabNewGame => Cutscene::Start,
            InterfaceMode::DiabReTown => Cutscene::Town,
            InterfaceMode::DiabNextLevel
            | InterfaceMode::DiabPrevLevel
            | InterfaceMode::DiabTownWarp
            | InterfaceMode::DiabTownWarpUp => {
                let level = self.current_level;
                if level == 1 && mode == InterfaceMode::DiabNextLevel {
                    return Cutscene::Town;
                }
                if level == 16 && mode == InterfaceMode::DiabNextLevel {
                    return Cutscene::Gate;
                }
                self.get_level_type(level).to_cutscene()
            }
            InterfaceMode::DiabWarpLevel => Cutscene::Portal,
            InterfaceMode::DiabSetLevel | InterfaceMode::DiabReturnLevel => {
                // Handle special set levels
                if self.set_level_num == 6 {
                    // Bone Chamber
                    return Cutscene::Level2;
                }
                if self.set_level_num == 9 {
                    // Vile Betrayer
                    return Cutscene::PortalRed;
                }
                // Arena levels
                if self.is_arena_level(self.set_level_num) {
                    if mode == InterfaceMode::DiabSetLevel {
                        return self.set_level_type.unwrap_or(DungeonType::Cathedral).to_cutscene();
                    }
                    return Cutscene::Town;
                }
                Cutscene::Level1
            }
            _ => Cutscene::Level1,
        }
    }

    /// Check if level is an arena level
    fn is_arena_level(&self, level: u8) -> bool {
        // Arena levels are typically special level numbers
        level >= 50
    }

    /// Increment progress
    pub fn inc_progress(&mut self, steps: u32) -> bool {
        if !self.is_progress {
            return false;
        }

        let prev_progress = self.progress;
        self.progress += PROGRESS_STEP_SIZE * steps;

        if self.progress > MAX_PROGRESS {
            self.progress = MAX_PROGRESS;
        }

        self.progress != prev_progress
    }

    /// Complete progress to maximum
    pub fn complete_progress(&mut self) {
        if !self.is_progress {
            return;
        }

        if self.progress < MAX_PROGRESS {
            let remaining_steps = (MAX_PROGRESS - self.progress) / PROGRESS_STEP_SIZE;
            self.inc_progress(remaining_steps);
        }
    }

    /// Start showing progress screen
    pub fn start_progress(&mut self, mode: InterfaceMode) {
        self.is_progress = true;
        self.progress = 0;
        self.state.reset();

        let cutscene = self.pick_cutscene(mode);
        self.progress_id = cutscene.progress_id();
    }

    /// End progress screen
    pub fn end_progress(&mut self) {
        self.is_progress = false;
        self.state.done = true;
    }

    /// Check if should skip rendering
    pub fn check_skip_rendering(&mut self) -> bool {
        if !self.state.skip_rendering {
            return false;
        }

        if let Some(elapsed) = self.state.elapsed() {
            let threshold = Duration::from_millis(self.skip_threshold_ms as u64);
            if elapsed > threshold {
                self.state.skip_rendering = false;
                return false;
            }
        }

        true
    }

    /// Get progress bar rectangle
    pub fn get_progress_bar_rect(&self, ui_x: i32, ui_y: i32) -> ProgressBarRect {
        let pos = BAR_POSITIONS[self.progress_id];
        ProgressBarRect {
            x: pos[0] + ui_x,
            y: pos[1] + ui_y,
            width: self.progress as i32,
            height: PROGRESS_HEIGHT,
            color_index: BAR_COLORS[self.progress_id],
        }
    }

    /// Get cutscene info for current progress
    pub fn get_cutscene_info(&self, mode: InterfaceMode) -> CutsceneInfo {
        let cutscene = self.pick_cutscene(mode);
        CutsceneInfo {
            cutscene,
            cel_path: cutscene.cel_path().to_string(),
            pal_path: cutscene.pal_path().to_string(),
            widescreen_path: cutscene.widescreen_clx_path().to_string(),
            progress_id: cutscene.progress_id(),
        }
    }
}

impl Default for InterfaceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress bar rectangle info
#[derive(Debug, Clone, Copy)]
pub struct ProgressBarRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub color_index: u8,
}

/// Cutscene information
#[derive(Debug, Clone)]
pub struct CutsceneInfo {
    pub cutscene: Cutscene,
    pub cel_path: String,
    pub pal_path: String,
    pub widescreen_path: String,
    pub progress_id: usize,
}

/// Entry type for loading levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    /// Main entry (new level)
    Main,
    /// Previous level entry
    Prev,
    /// Set level entry
    SetLevel,
    /// Return level entry
    ReturnLevel,
    /// Warp level entry
    WarpLevel,
    /// Town warp down
    TownWarpDown,
    /// Town warp up
    TownWarpUp,
}

/// Level loading context
#[derive(Debug, Clone)]
pub struct LoadContext {
    pub mode: InterfaceMode,
    pub entry_type: EntryType,
    pub target_level: u8,
    pub target_level_type: DungeonType,
    pub is_set_level: bool,
    pub return_level: u8,
    pub return_level_type: DungeonType,
}

impl LoadContext {
    /// Create load context from interface mode
    pub fn from_mode(mode: InterfaceMode, manager: &InterfaceManager) -> Self {
        let (entry_type, target_level, is_set_level) = match mode {
            InterfaceMode::DiabNewGame | InterfaceMode::DiabLoadGame => {
                (EntryType::Main, 0, false)
            }
            InterfaceMode::DiabNextLevel => {
                (EntryType::Main, manager.current_level + 1, false)
            }
            InterfaceMode::DiabPrevLevel => {
                (EntryType::Prev, manager.current_level - 1, false)
            }
            InterfaceMode::DiabSetLevel => {
                (EntryType::SetLevel, manager.set_level_num, true)
            }
            InterfaceMode::DiabReturnLevel => {
                (EntryType::ReturnLevel, manager.current_level, false)
            }
            InterfaceMode::DiabWarpLevel => {
                (EntryType::WarpLevel, manager.current_level, false)
            }
            InterfaceMode::DiabTownWarp => {
                (EntryType::TownWarpDown, manager.current_level, false)
            }
            InterfaceMode::DiabTownWarpUp => {
                (EntryType::TownWarpUp, manager.current_level, false)
            }
            InterfaceMode::DiabReTown => {
                (EntryType::Main, manager.current_level, false)
            }
            _ => (EntryType::Main, 0, false),
        };

        let target_level_type = if is_set_level {
            manager.set_level_type.unwrap_or(DungeonType::Cathedral)
        } else {
            manager.get_level_type(target_level)
        };

        Self {
            mode,
            entry_type,
            target_level,
            target_level_type,
            is_set_level,
            return_level: 0,
            return_level_type: DungeonType::Town,
        }
    }
}

// Free functions matching C++ API

/// Increment progress by steps
pub fn inc_progress(manager: &mut InterfaceManager, steps: u32) -> bool {
    manager.inc_progress(steps)
}

/// Complete progress to maximum
pub fn complete_progress(manager: &mut InterfaceManager) {
    manager.complete_progress()
}

/// Show progress screen
pub fn show_progress(manager: &mut InterfaceManager, mode: InterfaceMode) {
    manager.start_progress(mode)
}

/// Interface message pump (process pending events)
pub fn interface_msg_pump() {
    // In Rust, this would integrate with the event system
    // Placeholder for event processing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interface_mode_values() {
        assert_eq!(InterfaceMode::DiabNextLevel as u8, 0);
        assert_eq!(InterfaceMode::DiabLoadGame as u8, 9);
        assert_eq!(InterfaceMode::Done as u8, 12);
    }

    #[test]
    fn test_interface_mode_try_from() {
        assert_eq!(InterfaceMode::try_from(0), Ok(InterfaceMode::DiabNextLevel));
        assert_eq!(InterfaceMode::try_from(9), Ok(InterfaceMode::DiabLoadGame));
        assert_eq!(InterfaceMode::try_from(12), Ok(InterfaceMode::Done));
        assert!(InterfaceMode::try_from(13).is_err());
    }

    #[test]
    fn test_interface_mode_is_level_transition() {
        assert!(InterfaceMode::DiabNextLevel.is_level_transition());
        assert!(InterfaceMode::DiabPrevLevel.is_level_transition());
        assert!(InterfaceMode::DiabTownWarp.is_level_transition());
        assert!(!InterfaceMode::DiabNewGame.is_level_transition());
        assert!(!InterfaceMode::Progress.is_level_transition());
    }

    #[test]
    fn test_interface_mode_is_game_start() {
        assert!(InterfaceMode::DiabNewGame.is_game_start());
        assert!(InterfaceMode::DiabLoadGame.is_game_start());
        assert!(!InterfaceMode::DiabNextLevel.is_game_start());
    }

    #[test]
    fn test_interface_mode_is_async_event() {
        assert!(InterfaceMode::Progress.is_async_event());
        assert!(InterfaceMode::Error.is_async_event());
        assert!(InterfaceMode::Done.is_async_event());
        assert!(!InterfaceMode::DiabNewGame.is_async_event());
    }

    #[test]
    fn test_cutscene_paths() {
        assert_eq!(Cutscene::Start.cel_path(), "gendata\\cutstart");
        assert_eq!(Cutscene::Start.pal_path(), "gendata\\cutstart.pal");
        assert_eq!(Cutscene::Start.widescreen_clx_path(), "gendata\\cutstartw.clx");
    }

    #[test]
    fn test_cutscene_progress_id() {
        assert_eq!(Cutscene::Level1.progress_id(), 0);
        assert_eq!(Cutscene::Level2.progress_id(), 2);
        assert_eq!(Cutscene::Start.progress_id(), 1);
        assert_eq!(Cutscene::Town.progress_id(), 1);
    }

    #[test]
    fn test_dungeon_type_to_cutscene() {
        assert_eq!(DungeonType::Town.to_cutscene(), Cutscene::Town);
        assert_eq!(DungeonType::Cathedral.to_cutscene(), Cutscene::Level1);
        assert_eq!(DungeonType::Catacombs.to_cutscene(), Cutscene::Level2);
        assert_eq!(DungeonType::Caves.to_cutscene(), Cutscene::Level3);
        assert_eq!(DungeonType::Hell.to_cutscene(), Cutscene::Level4);
        assert_eq!(DungeonType::Crypt.to_cutscene(), Cutscene::Level5);
        assert_eq!(DungeonType::Nest.to_cutscene(), Cutscene::Level6);
    }

    #[test]
    fn test_bar_colors() {
        assert_eq!(BAR_COLORS[0], 138);
        assert_eq!(BAR_COLORS[1], 43);
        assert_eq!(BAR_COLORS[2], 254);
    }

    #[test]
    fn test_bar_positions() {
        assert_eq!(BAR_POSITIONS[0], [53, 37]);
        assert_eq!(BAR_POSITIONS[1], [53, 421]);
        assert_eq!(BAR_POSITIONS[2], [53, 37]);
    }

    #[test]
    fn test_progress_state_default() {
        let state = ProgressState::default();
        assert!(state.load_started_at.is_none());
        assert!(state.skip_rendering);
        assert!(!state.done);
        assert_eq!(state.drawn_progress, 0);
    }

    #[test]
    fn test_progress_state_reset() {
        let mut state = ProgressState::default();
        state.done = true;
        state.drawn_progress = 100;

        state.reset();

        assert!(state.load_started_at.is_some());
        assert!(state.skip_rendering);
        assert!(!state.done);
        assert_eq!(state.drawn_progress, 0);
    }

    #[test]
    fn test_interface_manager_new() {
        let manager = InterfaceManager::new();
        assert_eq!(manager.progress(), 0);
        assert_eq!(manager.progress_id(), 1);
        assert!(!manager.is_progress());
    }

    #[test]
    fn test_interface_manager_inc_progress() {
        let mut manager = InterfaceManager::new();
        manager.is_progress = true;

        assert!(manager.inc_progress(1));
        assert_eq!(manager.progress(), PROGRESS_STEP_SIZE);

        assert!(manager.inc_progress(2));
        assert_eq!(manager.progress(), PROGRESS_STEP_SIZE * 3);
    }

    #[test]
    fn test_interface_manager_inc_progress_not_active() {
        let mut manager = InterfaceManager::new();
        // is_progress is false by default

        assert!(!manager.inc_progress(1));
        assert_eq!(manager.progress(), 0);
    }

    #[test]
    fn test_interface_manager_inc_progress_max() {
        let mut manager = InterfaceManager::new();
        manager.is_progress = true;

        // Progress beyond max
        manager.inc_progress(100);
        assert_eq!(manager.progress(), MAX_PROGRESS);
    }

    #[test]
    fn test_interface_manager_complete_progress() {
        let mut manager = InterfaceManager::new();
        manager.is_progress = true;
        manager.progress = 100;

        manager.complete_progress();

        // Should be at or near MAX_PROGRESS
        assert!(manager.progress() >= MAX_PROGRESS - PROGRESS_STEP_SIZE);
    }

    #[test]
    fn test_interface_manager_start_progress() {
        let mut manager = InterfaceManager::new();

        manager.start_progress(InterfaceMode::DiabNewGame);

        assert!(manager.is_progress());
        assert_eq!(manager.progress(), 0);
        assert!(manager.state.load_started_at.is_some());
    }

    #[test]
    fn test_interface_manager_end_progress() {
        let mut manager = InterfaceManager::new();
        manager.is_progress = true;

        manager.end_progress();

        assert!(!manager.is_progress());
        assert!(manager.state.done);
    }

    #[test]
    fn test_interface_manager_get_level_type() {
        let mut manager = InterfaceManager::new();

        assert_eq!(manager.get_level_type(0), DungeonType::Town);
        assert_eq!(manager.get_level_type(1), DungeonType::Cathedral);
        assert_eq!(manager.get_level_type(4), DungeonType::Cathedral);
        assert_eq!(manager.get_level_type(5), DungeonType::Catacombs);
        assert_eq!(manager.get_level_type(8), DungeonType::Catacombs);
        assert_eq!(manager.get_level_type(9), DungeonType::Caves);
        assert_eq!(manager.get_level_type(12), DungeonType::Caves);
        assert_eq!(manager.get_level_type(13), DungeonType::Hell);
        assert_eq!(manager.get_level_type(16), DungeonType::Hell);

        // Hellfire levels
        manager.is_hellfire = true;
        assert_eq!(manager.get_level_type(17), DungeonType::Crypt);
        assert_eq!(manager.get_level_type(21), DungeonType::Nest);
    }

    #[test]
    fn test_interface_manager_pick_cutscene() {
        let mut manager = InterfaceManager::new();

        assert_eq!(manager.pick_cutscene(InterfaceMode::DiabNewGame), Cutscene::Start);
        assert_eq!(manager.pick_cutscene(InterfaceMode::DiabLoadGame), Cutscene::Start);
        assert_eq!(manager.pick_cutscene(InterfaceMode::DiabReTown), Cutscene::Town);
        assert_eq!(manager.pick_cutscene(InterfaceMode::DiabWarpLevel), Cutscene::Portal);

        // Level transition
        manager.current_level = 5;
        assert_eq!(manager.pick_cutscene(InterfaceMode::DiabNextLevel), Cutscene::Level2);

        // Special case: level 1 going next
        manager.current_level = 1;
        assert_eq!(manager.pick_cutscene(InterfaceMode::DiabNextLevel), Cutscene::Town);

        // Special case: level 16 gate
        manager.current_level = 16;
        assert_eq!(manager.pick_cutscene(InterfaceMode::DiabNextLevel), Cutscene::Gate);
    }

    #[test]
    fn test_interface_manager_get_progress_bar_rect() {
        let mut manager = InterfaceManager::new();
        manager.progress = 100;
        manager.progress_id = 1;

        let rect = manager.get_progress_bar_rect(0, 0);

        assert_eq!(rect.x, 53);
        assert_eq!(rect.y, 421);
        assert_eq!(rect.width, 100);
        assert_eq!(rect.height, PROGRESS_HEIGHT);
        assert_eq!(rect.color_index, BAR_COLORS[1]);
    }

    #[test]
    fn test_interface_manager_get_progress_bar_rect_with_offset() {
        let mut manager = InterfaceManager::new();
        manager.progress = 200;
        manager.progress_id = 0;

        let rect = manager.get_progress_bar_rect(100, 50);

        assert_eq!(rect.x, 53 + 100);
        assert_eq!(rect.y, 37 + 50);
    }

    #[test]
    fn test_cutscene_info() {
        let manager = InterfaceManager::new();
        let info = manager.get_cutscene_info(InterfaceMode::DiabNewGame);

        assert_eq!(info.cutscene, Cutscene::Start);
        assert_eq!(info.cel_path, "gendata\\cutstart");
        assert_eq!(info.pal_path, "gendata\\cutstart.pal");
    }

    #[test]
    fn test_entry_type() {
        assert_ne!(EntryType::Main, EntryType::Prev);
        assert_ne!(EntryType::SetLevel, EntryType::ReturnLevel);
    }

    #[test]
    fn test_load_context_from_mode_new_game() {
        let manager = InterfaceManager::new();
        let ctx = LoadContext::from_mode(InterfaceMode::DiabNewGame, &manager);

        assert_eq!(ctx.mode, InterfaceMode::DiabNewGame);
        assert_eq!(ctx.entry_type, EntryType::Main);
        assert_eq!(ctx.target_level, 0);
        assert!(!ctx.is_set_level);
    }

    #[test]
    fn test_load_context_from_mode_next_level() {
        let mut manager = InterfaceManager::new();
        manager.current_level = 5;

        let ctx = LoadContext::from_mode(InterfaceMode::DiabNextLevel, &manager);

        assert_eq!(ctx.entry_type, EntryType::Main);
        assert_eq!(ctx.target_level, 6);
    }

    #[test]
    fn test_load_context_from_mode_prev_level() {
        let mut manager = InterfaceManager::new();
        manager.current_level = 5;

        let ctx = LoadContext::from_mode(InterfaceMode::DiabPrevLevel, &manager);

        assert_eq!(ctx.entry_type, EntryType::Prev);
        assert_eq!(ctx.target_level, 4);
    }

    #[test]
    fn test_load_context_from_mode_set_level() {
        let mut manager = InterfaceManager::new();
        manager.set_level_num = 10;
        manager.set_level_type = Some(DungeonType::Catacombs);

        let ctx = LoadContext::from_mode(InterfaceMode::DiabSetLevel, &manager);

        assert_eq!(ctx.entry_type, EntryType::SetLevel);
        assert_eq!(ctx.target_level, 10);
        assert!(ctx.is_set_level);
        assert_eq!(ctx.target_level_type, DungeonType::Catacombs);
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_PROGRESS, 534);
        assert_eq!(PROGRESS_STEP_SIZE, 23);
        assert_eq!(PROGRESS_HEIGHT, 22);
    }

    #[test]
    fn test_free_functions() {
        let mut manager = InterfaceManager::new();

        show_progress(&mut manager, InterfaceMode::DiabNewGame);
        assert!(manager.is_progress());

        assert!(inc_progress(&mut manager, 1));
        assert_eq!(manager.progress(), PROGRESS_STEP_SIZE);

        complete_progress(&mut manager);
        assert!(manager.progress() >= MAX_PROGRESS - PROGRESS_STEP_SIZE);
    }
}
