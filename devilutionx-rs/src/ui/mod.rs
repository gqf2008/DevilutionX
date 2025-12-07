//! UI Module - Menus, Dialogs, and HUD
//!
//! # M62: UI System + M65: DiabloUI System
//!
//! This module provides the user interface components for DevilutionX:
//! - Main menu with Single/Multi/Settings options
//! - Dialog boxes (OK, Yes/No, custom)
//! - Scrollable lists
//! - Text input
//! - Progress bars
//! - Fade effects
//! - Hero selection and creation
//! - Credits and title screens
//!
//! ## C++ References
//! - `Source/DiabloUI/*.cpp` - UI components
//! - `Source/DiabloUI/diabloui.cpp` - Core UI framework
//! - `Source/DiabloUI/mainmenu.cpp` - Main menu

#![allow(unused_imports)]

pub mod diabloui;
pub mod menu;

// Re-export main types from menu
pub use menu::{
    // UI Manager
    Menu,

    // Main Menu
    MainMenu,
    MainMenuSelection,
    MenuItem,

    // Dialogs
    Dialog,
    DialogType,
    DialogResult,

    // Widgets
    Button,
    UiList,
    ListItem,
    TextInput,
    ProgressBar,

    // Effects
    FadeState,

    // Input
    KeyCode,

    // Geometry
    Point,
    Rect,

    // Flags
    UiFlags,
};

// Re-export types from diabloui
pub use diabloui::{
    // Core types
    ArtFocus,
    DefaultStats,
    Difficulty,
    HeroClass,

    // UI items
    UiType,
    UiRect,
    UiItem,
    UiItemBase,
    UiText,
    UiArtText,
    UiArtTextButton,
    UiImageClx,
    UiButton,
    UiListItem,
    UiScrollbar,
    UiEdit,

    // UI core
    UiContext,
    UiEvent,
    UiEventResult,
    UiEventHandler,
    UiRenderer,
    UiListState,

    // Dialogs
    DialogResult as DiabloDialogResult,
    DialogType as DiabloDialogType,
    ProgressDialog,
    SelectDialog,

    // Hero selection
    HeroInfo,
    HeroSelection,
    SelHeroResult,

    // Main menu
    MainMenuResult,
    MainMenuSelection as DiabloMainMenuSelection,
};
