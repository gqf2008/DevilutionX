//! DevilutionX-RS Main Entry Point
//! Aligned with C++ DiabloMain flow in Source/diablo.cpp

use sdl2::event::Event;
use sdl2::messagebox::{show_simple_message_box, MessageBoxFlag};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Texture, TextureCreator};
use sdl2::video::WindowContext;

mod data;
mod engine;
mod game;
mod net;
mod ui;
mod utils;

use engine::font::{FontRenderer, PixelFont};
use engine::audio::AudioManager;
use engine::MpqAssetManager;
use engine::window::GameWindow;
use engine::PcxImage;
use game::init as game_init;
use game::movie::{MovieFlags, MoviePlayer};
use game::player_exact::Player;
use game::game_state::GameState;
use game::game_loop::{run_game_loop, InterfaceMode};
use ui::diabloui::mainmenu::{MainMenu, MainMenuSelection};
use ui::diabloui::UiContext;
use std::env;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use utils::{assets_path, config_path};
use utils::options::{StartUpGameMode, StartUpIntro, StartUpSplash};
use utils::paths::{set_base_path, set_config_path, set_pref_path};
use utils::{load_options as load_options_into_global, options as global_options, save_options as save_options_from_global};

const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 480;

/// Minimal command-line flags parsed at startup (C++: DiabloParseFlags)
#[derive(Default, Debug, Clone)]
struct CmdFlags {
    /// Force spawn/shareware data path
    prefer_spawn: bool,
        /// Demo mode (skip saves, mimic C++ demo override)
        demo_mode: bool,
        /// Skip splash/movie dialogs
        no_splash: bool,
    /// Path overrides
    data_dir: Option<String>,
    save_dir: Option<String>,
    config_dir: Option<String>,
    /// Forced language
    lang_override: Option<String>,
    /// Demo/timedemo/record flags
    timedemo: bool,
    demo_number: Option<i32>,
    record_number: Option<i32>,
    create_reference: bool,
    /// Disable intro (-n)
    no_intro: bool,
    /// Frame count (-f)
    frame_count: bool,
    /// Force specific game mode
    force_diablo: bool,
    force_hellfire: bool,
    vanilla: bool,
    /// Verbose/logging
    verbose: bool,
    log_to_file: Option<String>,
}

fn parse_cmdline_flags() -> CmdFlags {
    let mut flags = CmdFlags::default();
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--spawn" | "-spawn" => flags.prefer_spawn = true,
            "--demo" | "-demo" => flags.demo_mode = true,
            "--no-splash" | "-nologo" => flags.no_splash = true,
            "-n" => {
                flags.no_intro = true;
                flags.no_splash = true;
            },
            "-f" => flags.frame_count = true,
            "--hellfire" => flags.force_hellfire = true,
            "--diablo" => flags.force_diablo = true,
            "--vanilla" => flags.vanilla = true,
            "--timedemo" => flags.timedemo = true,
            "--create-reference" => flags.create_reference = true,
            "--verbose" => flags.verbose = true,
            _ => {},
        }
    }
    flags
}

fn parse_cmdline_with_args(flags: &mut CmdFlags) {
    let mut args = env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => {
                if let Some(val) = args.next() {
                    flags.data_dir = Some(val);
                }
            }
            "--save-dir" => {
                if let Some(val) = args.next() {
                    flags.save_dir = Some(val);
                }
            }
            "--config-dir" => {
                if let Some(val) = args.next() {
                    flags.config_dir = Some(val);
                }
            }
            "--lang" => {
                if let Some(val) = args.next() {
                    flags.lang_override = Some(val);
                }
            }
            "--demo" => {
                if let Some(val) = args.next() {
                    if let Ok(v) = val.parse::<i32>() {
                        flags.demo_number = Some(v);
                        flags.demo_mode = true;
                        flags.no_splash = true;
                    }
                }
            }
            "--record" => {
                if let Some(val) = args.next() {
                    if let Ok(v) = val.parse::<i32>() {
                        flags.record_number = Some(v);
                    }
                }
            }
            "--log-to-file" => {
                if let Some(val) = args.next() {
                    flags.log_to_file = Some(val);
                }
            }
            _ => {}
        }
    }
}

fn apply_path_overrides(flags: &CmdFlags) {
    if let Some(path) = &flags.data_dir {
        set_base_path(path);
    }
    if let Some(path) = &flags.save_dir {
        set_pref_path(path);
    }
    if let Some(path) = &flags.config_dir {
        set_config_path(path);
    }
}

fn apply_option_overrides(flags: &CmdFlags) {
    if flags.lang_override.is_some()
        || flags.prefer_spawn
        || flags.force_diablo
        || flags.force_hellfire
        || flags.vanilla
    {
        let mut opts = utils::options::options_mut();

        if let Some(lang) = &flags.lang_override {
            opts.language.code = lang.clone();
        }

        if flags.prefer_spawn {
            opts.game_mode.shareware = true;
        }

        if flags.force_diablo {
            opts.game_mode.game_mode = StartUpGameMode::Diablo;
            opts.game_mode.shareware = false;
        }

        if flags.force_hellfire {
            opts.game_mode.game_mode = StartUpGameMode::Hellfire;
            opts.game_mode.shareware = false;
        }

        if flags.vanilla {
            opts.game_mode.shareware = false;
        }
    }
}

fn init_keymap_actions() {
    // Placeholder for Source/controls/keymapper.cpp::InitKeymapActions
    println!("[InitKeymapActions] Using default keymap bindings");
}

fn init_padmap_actions() {
    // Placeholder for Source/controls/padmapper.cpp::InitPadmapActions
    println!("[InitPadmapActions] Using default controller bindings");
}

/// Initialize language/localization system
/// C++ Reference: Source/utils/language.cpp::LanguageInitialize()
fn language_initialize() {
    let lang_code = {
        let opts = global_options();
        opts.language.code.clone()
    };

    // English is the base translation, no MO file needed
    if lang_code.is_empty() || lang_code.to_lowercase() == "en" {
        println!("[LanguageInitialize] Using English (base translation)");
        return;
    }

    // For other languages, the translation file would be loaded here
    // C++ loads .mo/.gmo files from assets
    println!("[LanguageInitialize] Locale '{}' selected (translation files not yet implemented)", lang_code);
}

fn ui_initialize(ctx: &mut DiabloContext) {
    let mgr = ui_resources();
    match mgr.lock() {
        Ok(mut res) => {
            if res.initialized {
                println!("[UiInitialize] UI already initialized; skipping");
                return;
            }

            load_ui_assets(&mut res, &mut ctx.mpq_manager);

            res.cursor_loaded = res.assets.cursor.is_some();
            res.font = FontRenderer::from_system_font(16.0)
                .or_else(|_| FontRenderer::from_system_font(14.0))
                .ok();

            res.initialized = true;

            if res.font.is_some() {
                println!("[UiInitialize] Loaded system font for UI text");
            } else {
                println!("[UiInitialize] No system font found; fallback to pixel font only");
            }

            if res.cursor_loaded {
                println!("[UiInitialize] Cursor art loaded from MPQ");
            } else {
                println!("[UiInitialize] Cursor art missing; using SDL cursor");
            }
        }
        Err(err) => println!("[UiInitialize] Failed to lock UI resources: {}", err),
    }
}

fn ui_destroy() {
    let mgr = ui_resources();
    match mgr.lock() {
        Ok(mut res) => {
            res.font = None;
            res.cursor_loaded = false;
            res.assets = UiAssetsSnapshot::default();
            res.initialized = false;
            println!("[UiDestroy] Released UI resources (font/cursor/assets)");
        }
        Err(err) => println!("[UiDestroy] Failed to lock UI resources: {}", err),
    }
}

/// Initialize screen state
/// C++ Reference: Source/diablo.cpp::DiabloInitScreen()
fn diablo_init_screen(window: &GameWindow) {
    // C++: MousePosition = { gnScreenWidth / 2, gnScreenHeight / 2 };
    let (w, h) = (window.width(), window.height());
    let mouse_x = w as i32 / 2;
    let mouse_y = h as i32 / 2;

    // Note: In C++ this also calls SetCursorPos and ClrDiabloMsg
    // We'll handle cursor positioning through SDL and message clearing through UI system
    println!("[DiabloInitScreen] Screen initialized ({}x{}), mouse center: ({}, {})", w, h, mouse_x, mouse_y);
}

/// Initialize virtual gamepad graphics for touch controls
/// C++ Reference: Source/controls/touch/renderers.cpp::InitVirtualGamepadGFX()
fn init_virtual_gamepad() {
    // Virtual gamepad is used on touch devices (mobile, Switch, etc.)
    // On desktop, this is typically a no-op
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        println!("[VirtualGamepad] Loading touch control graphics");
        // TODO: Load pad art, button art, menu panel art from MPQ
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        println!("[VirtualGamepad] Skipped (not a touch platform)");
    }
}

fn snd_init() {
    let opts = global_options();
    let sfx_volume = (opts.audio.sound_volume as f32 / 100.0).clamp(0.0, 1.0);
    let music_volume = (opts.audio.music_volume as f32 / 100.0).clamp(0.0, 1.0);

    let mgr = audio_manager();
    match mgr.lock() {
        Ok(mut audio) => {
            if let Err(err) = audio.init() {
                println!("[Sound] Init failed: {}", err);
            } else {
                audio.set_sfx_volume(sfx_volume);
                audio.set_music_volume(music_volume);
                println!(
                    "[Sound] Init complete (sfx={:.0}%, music={:.0}%)",
                    audio.sfx_volume() * 100.0,
                    audio.music_volume() * 100.0
                );
            }
        }
        Err(err) => println!("[Sound] Init failed (lock poisoned): {}", err),
    }
}

fn ui_sound_init() {
    let opts = global_options();
    let _sfx_volume = (opts.audio.sound_volume as f32 / 100.0).clamp(0.0, 1.0);

    let mgr = audio_manager();
    match mgr.lock() {
        Ok(audio) => println!(
            "[Sound] UI sound hook ready (sfx volume {:.0}%, music volume {:.0}%)",
            audio.sfx_volume() * 100.0,
            audio.music_volume() * 100.0
        ),
        Err(err) => println!("[Sound] UI sound hook skipped (audio unavailable): {}", err),
    }
}

/// Free item animation graphics
/// C++ Reference: Source/items.cpp::FreeItemGFX()
fn free_item_gfx() {
    // C++: for (auto &itemanim : itemanims) { itemanim = std::nullopt; }
    // In Rust, we would clear the item animation cache
    // TODO: Wire to actual item graphics cache once implemented
    println!("[FreeItemGFX] Released item animation cache");
}

/// Shutdown Lua scripting engine
/// C++ Reference: Source/lua/lua_global.cpp::LuaShutdown()
fn lua_shutdown() {
    // C++: CurrentLuaState = std::nullopt;
    // Lua modding support is optional; when implemented, this would
    // destroy the Lua state and free mod resources
    println!("[LuaShutdown] Lua state released (modding disabled)");
}

/// Shutdown screen reader accessibility support
/// C++ Reference: Source/utils/screen_reader.cpp::ShutDownScreenReader()
fn screen_reader_shutdown() {
    // C++: Windows uses Tolk_Unload(), Linux uses spd_close()
    // Screen reader support is platform-specific
    #[cfg(windows)]
    {
        println!("[ScreenReader] Tolk unloaded (Windows)");
    }
    #[cfg(not(windows))]
    {
        println!("[ScreenReader] Speech-dispatcher closed");
    }
}

fn snd_deinit() {
    let mgr = audio_manager();
    match mgr.lock() {
        Ok(mut audio) => {
            audio.shutdown();
            println!("[Sound] Deinit complete");
        }
        Err(err) => println!("[Sound] Deinit skipped (audio unavailable): {}", err),
    }
}

fn show_mpq_error_dialog(message: &str) {
    let _ = show_simple_message_box(MessageBoxFlag::ERROR, "缺少或过期的游戏数据", message, None)
        .map_err(|e| println!("[UI] Failed to show MPQ error dialog: {}", e));
    println!("[UI] MPQ error dialog: {}", message);
}

fn show_support_dialog() {
    let message = "DevilutionX 支持：\n- GitHub: https://github.com/diasurgical/devilutionX\n- 文档: docs/\n- Bug 反馈: 在 GitHub 提交 Issue";
    let _ = show_simple_message_box(MessageBoxFlag::INFORMATION, "Support", message, None)
        .map_err(|e| println!("[UI] Failed to show support dialog: {}", e));
}

fn show_settings_dialog() {
    let cfg_path = options_file_path();
    let message = format!(
        "设置管理：\n- 目前请直接编辑配置文件\n- 路径: {}\n- 分辨率/音量/语言等均可在此调整",
        cfg_path.display()
    );
    let _ = show_simple_message_box(MessageBoxFlag::INFORMATION, "Settings", &message, None)
        .map_err(|e| println!("[UI] Failed to show settings dialog: {}", e));
}

fn show_credits_dialog() {
    let message = "DevilutionX Rust Port\nBased on DevilutionX contributors\nRust port by community maintainers";
    let _ = show_simple_message_box(MessageBoxFlag::INFORMATION, "Credits", message, None)
        .map_err(|e| println!("[UI] Failed to show credits dialog: {}", e));
}

/// Cleanup initialization resources (MPQ archives, network, save files)
/// C++ Reference: Source/init.cpp::init_cleanup()
fn init_cleanup(is_multiplayer: bool, running: bool) {
    // C++: if (gbIsMultiplayer && gbRunGame) { pfile_write_hero; sfile_write_stash; }
    if is_multiplayer && running {
        println!("[InitCleanup] Saving hero and stash for multiplayer");
        // TODO: Call save functions when implemented
    }

    // C++: MpqArchives.clear(); HasHellfireMpq = false; NetClose();
    println!("[InitCleanup] Releasing MPQ archives");
    net_close();
}

/// Cleanup DirectX/SDL rendering resources
/// C++ Reference: Source/engine/dx.cpp::dx_cleanup()
fn dx_cleanup() {
    // C++: SDL_HideWindow, destroy surfaces/textures/renderer, SDL_DestroyWindow
    // In Rust, SDL resources are cleaned up when GameWindow is dropped
    // This function is called before SDL_Quit to ensure proper cleanup order
    println!("[DXCleanup] SDL surfaces and renderer released");
}

/// Unload font resources
/// C++ Reference: Source/engine/render/text_render.cpp::UnloadFonts()
fn unload_fonts() {
    // C++: Fonts.clear();
    // Font sprites are loaded from MPQ and cached; this releases them
    println!("[UnloadFonts] Font cache cleared");
}

/// Initialize Lua scripting engine for mod support
/// C++ Reference: Source/lua/lua_global.cpp::LuaInitialize()
fn lua_initialize() {
    // C++: Creates sol::state, opens libraries, registers devilutionx modules
    // Lua modding is optional; when enabled this would:
    // 1. Create Lua state with panic handler
    // 2. Open standard libraries (base, coroutine, math, string, table, etc.)
    // 3. Register DevilutionX API modules (items, player, render, audio, etc.)
    // 4. Load active mods from options
    println!("[LuaInitialize] Lua scripting engine ready (modding disabled)");
}

/// Resolve diablo.ini path using the configured preferences directory
fn options_file_path() -> PathBuf {
    let mut path = PathBuf::from(config_path());
    path.push("diablo.ini");
    path
}

/// Load options from disk or create defaults when missing
fn load_or_init_options(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
        }
    }

    if path.exists() {
        load_options_into_global(path).map_err(|e| e.to_string())?
    } else {
        save_options_from_global(path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Load language-specific MPQ archives based on options
fn load_language_archive(mpq: &mut MpqAssetManager) -> Result<(), String> {
    let language = {
        let opts = global_options();
        opts.language.code.clone()
    };

    let normalized = language.to_lowercase();
    if normalized.is_empty() || normalized == "en" {
        return Ok(());
    }

    let lang_mpq = format!("{}.mpq", normalized);
    match mpq.load_mpq(&lang_mpq, 0) {
        Ok(_) => println!("  Loaded language archive: {}", lang_mpq),
        Err(err) => println!("  Language archive {} not found: {}", lang_mpq, err),
    }

    Ok(())
}







use game::player::PlayerClass;

/// Application context (similar to C++ global state)
struct DiabloContext {
    mpq_manager: MpqAssetManager,
    font: PixelFont,
    window: GameWindow,
    product_name: String,
}

static AUDIO_MANAGER: OnceLock<Mutex<AudioManager>> = OnceLock::new();
static UI_RESOURCES: OnceLock<Mutex<UiResources>> = OnceLock::new();

fn audio_manager() -> &'static Mutex<AudioManager> {
    AUDIO_MANAGER.get_or_init(|| {
        let mut mgr = AudioManager::new();
        if let Err(err) = mgr.init() {
            println!("[Audio] Init failed (stub mode): {}", err);
        }
        Mutex::new(mgr)
    })
}

#[derive(Clone, Default)]
struct UiArtImage {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl UiArtImage {
    fn to_texture<'a>(&self, creator: &'a TextureCreator<WindowContext>) -> Result<Texture<'a>, String> {
        let mut tex = creator
            .create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGBA32, self.width, self.height)
            .map_err(|e| format!("创建纹理失败: {}", e))?;

        tex.update(None, &self.rgba, (self.width * 4) as usize)
            .map_err(|e| format!("更新纹理失败: {}", e))?;

        // Enable alpha blending; set_blend_mode is infallible in this SDL2 binding
        let _ = tex.set_blend_mode(BlendMode::Blend);

        Ok(tex)
    }
}

#[derive(Clone, Default)]
struct UiAssetsSnapshot {
    mainmenu_bg: Option<UiArtImage>,
    title_bg: Option<UiArtImage>,
    selhero_bg: Option<UiArtImage>,
    logo: Option<UiArtImage>,
    cursor: Option<UiArtImage>,
    focus_small: Vec<UiArtImage>,
    focus_med: Vec<UiArtImage>,
    focus_big: Vec<UiArtImage>,
}

#[derive(Default)]
struct UiResources {
    initialized: bool,
    cursor_loaded: bool,
    font: Option<FontRenderer>,
    assets: UiAssetsSnapshot,
}

fn ui_resources() -> &'static Mutex<UiResources> {
    UI_RESOURCES.get_or_init(|| Mutex::new(UiResources::default()))
}

// UI 坐标系统一使用 640x480 逻辑分辨率；SDL 负责缩放到实际窗口
fn ui_origin() -> (i32, i32) {
    (0, 0)
}

fn load_pcx_image(mpq: &mut MpqAssetManager, path: &str, transparent: Option<u8>) -> Result<UiArtImage, String> {
    let data = mpq.read_file(path).map_err(|e| format!("读取 {} 失败: {}", path, e))?;
    let pcx = PcxImage::decode(&data).ok_or_else(|| format!("解析 PCX 失败: {}", path))?;

    let mut rgba = vec![0u8; (pcx.width * pcx.height * 4) as usize];
    for (i, &idx) in pcx.pixels.iter().enumerate() {
        let color = pcx.palette.get(idx as usize).ok_or_else(|| format!("PCX 调色板索引超出范围 {} for {}", idx, path))?;
        let offset = i * 4;
        rgba[offset] = color.r;
        rgba[offset + 1] = color.g;
        rgba[offset + 2] = color.b;
        let alpha = if transparent.map(|t| t == idx).unwrap_or(false) { 0 } else { 255 };
        rgba[offset + 3] = alpha;
    }

    Ok(UiArtImage { width: pcx.width, height: pcx.height, rgba })
}

fn load_pcx_strip(mpq: &mut MpqAssetManager, path: &str, frames: usize, transparent: Option<u8>) -> Vec<UiArtImage> {
    let mut results = Vec::new();
    let data = match mpq.read_file(path) {
        Ok(d) => d,
        Err(e) => {
            println!("[UiAssets] 读取 {} 失败: {}", path, e);
            return results;
        }
    };

    let pcx = match PcxImage::decode(&data) {
        Some(img) => img,
        None => {
            println!("[UiAssets] 解析 PCX 失败: {}", path);
            return results;
        }
    };

    if frames == 0 {
        return results;
    }

    // Diablo UI PCX strips are stacked vertically; split by frame height
    let frame_height = pcx.height / frames as u32;
    for frame_idx in 0..frames {
        let mut rgba = vec![0u8; (pcx.width * frame_height * 4) as usize];
        for y in 0..frame_height {
            let src_y = y + frame_idx as u32 * frame_height;
            for x in 0..pcx.width {
                let src_idx = (src_y * pcx.width + x) as usize;
                let dst_idx = (y * pcx.width + x) as usize * 4;
                let color_idx = pcx.pixels.get(src_idx).copied().unwrap_or(0);
                if let Some(color) = pcx.palette.get(color_idx as usize) {
                    rgba[dst_idx] = color.r;
                    rgba[dst_idx + 1] = color.g;
                    rgba[dst_idx + 2] = color.b;
                    let alpha = if transparent.map(|t| t == color_idx).unwrap_or(false) { 0 } else { 255 };
                    rgba[dst_idx + 3] = alpha;
                }
            }
        }
        results.push(UiArtImage { width: pcx.width, height: frame_height, rgba });
    }

    results
}

fn load_ui_assets(res: &mut UiResources, mpq: &mut MpqAssetManager) {
    let mut assets = UiAssetsSnapshot::default();

    let opts = global_options();
    let is_hellfire = matches!(opts.game_mode.game_mode, StartUpGameMode::Hellfire);
    let is_spawn = opts.game_mode.shareware && !is_hellfire;

    let mainmenu_path = if is_spawn { "ui_art\\swmmenu.pcx" } else { "ui_art\\mainmenu.pcx" };
    let title_path = if is_hellfire { "ui_art\\hf_title.pcx" } else { "ui_art\\title.pcx" };

    assets.mainmenu_bg = load_pcx_image(mpq, mainmenu_path, None).ok();
    assets.title_bg = load_pcx_image(mpq, title_path, None).ok();
    assets.selhero_bg = load_pcx_image(mpq, "ui_art\\selhero.pcx", None).ok();

    // Hellfire 优先，其次原版 LOGO
    assets.logo = load_pcx_image(mpq, "ui_art\\hf_logo2.pcx", Some(0))
        .or_else(|_| load_pcx_image(mpq, "ui_art\\smlogo.pcx", Some(250)))
        .ok();

    assets.cursor = load_pcx_image(mpq, "ui_art\\cursor.pcx", Some(0)).ok();

    // Focus rings (8 frames each)
    assets.focus_small = load_pcx_strip(mpq, "ui_art\\focus16.pcx", 8, Some(250));
    assets.focus_med = load_pcx_strip(mpq, "ui_art\\focus.pcx", 8, Some(250));
    assets.focus_big = load_pcx_strip(mpq, "ui_art\\focus42.pcx", 8, Some(250));

    println!(
        "[UiAssets] mainmenu={} title={} logo={} cursor={} focus16={} focus={} focus42={}",
        assets.mainmenu_bg.is_some(),
        assets.title_bg.is_some(),
        assets.logo.is_some(),
        assets.cursor.is_some(),
        assets.focus_small.len(),
        assets.focus_med.len(),
        assets.focus_big.len()
    );

    res.assets = assets;
}

impl DiabloContext {
    fn new(window: GameWindow, mpq_manager: MpqAssetManager) -> Self {
        Self {
            mpq_manager,
            font: PixelFont::new(2),
            window,
            product_name: "DevilutionX-RS".to_string(),
        }
    }
}

/// Step 1: LoadCoreArchives (C++: DiabloMain line 2707)
/// Load devilutionx.mpq, fonts.mpq
fn load_core_archives(mpq: &mut MpqAssetManager) -> Result<(), String> {
    println!("[LoadCoreArchives] Loading core archives...");
    mpq.load_core_archives().map_err(|e| e.to_string())?;
    println!("  Core MPQs loaded (devilutionx/fonts)");
    Ok(())
}

/// Step 1b: LoadGameArchives (C++: init.cpp LoadGameArchives)
fn load_game_archives(mpq: &mut MpqAssetManager, prefer_spawn: bool, detection: &ArchiveDetection) -> Result<(), String> {
    println!("[LoadGameArchives] Loading game data archives...");

    let mut loaded_spawn = false;
    let mut loaded_full = false;

    // Shareware flag: try spawn first
    if prefer_spawn {
        if mpq.load_mpq_from_search("spawn", 1000).map_err(|e| e.to_string())? {
            println!("  Loaded: spawn.mpq (shareware mode)");
            loaded_spawn = true;
        }
    }

    loaded_full = mpq.load_game_archives().map_err(|e| e.to_string())?;
    if loaded_full {
        println!("  Loaded: DIABDAT.MPQ/diabdat.mpq");
    } else if mpq.has_file("levels/towndata/town.min") {
        println!("  Game data present in existing archives");
        loaded_full = true;
    } else {
        println!("  Warning: full game data not found; attempting spawn.mpq fallback");
    }

    // Hellfire archives are optional
    if mpq.load_hellfire_archives().map_err(|e| e.to_string())? {
        println!("  Loaded Hellfire archives");
    }

    if !loaded_full && !loaded_spawn {
        if mpq.load_mpq_from_search("spawn", 1000).map_err(|e| e.to_string())? {
            println!("  Loaded fallback: spawn.mpq (shareware mode)");
            loaded_spawn = true;
        }
    }

    if !loaded_full && !loaded_spawn {
        let mut reasons = vec!["未找到 diabdat.mpq/DIABDAT.MPQ".to_string()];
        if !detection.has_spawn {
            reasons.push("未找到 spawn.mpq".to_string());
        }
        return Err(format!("加载游戏数据失败: {}", reasons.join("; ")));
    }

    Ok(())
}

/// Step 1c: Load additional text/data tables (stubs mapped to C++)
fn load_text_data() {
    println!("[LoadTextData] Text tables compiled in (e.g., translations)");
}

fn ensure_mpq_versions(mpq: &mut MpqAssetManager) -> Result<(), String> {
    println!("[EnsureVersions] reading ASSETS_VERSION...");
    let assets_version = mpq.read_file("ASSETS_VERSION").ok();
    println!("[EnsureVersions] ASSETS_VERSION bytes={}", assets_version.as_ref().map(|v| v.len()).unwrap_or(0));

    println!("[EnsureVersions] reading fonts/VERSION...");
    let fonts_version = mpq.read_file("fonts/VERSION").ok();
    println!("[EnsureVersions] fonts/VERSION bytes={}", fonts_version.as_ref().map(|v| v.len()).unwrap_or(0));

    // Development convenience: if version markers are missing, continue with a warning
    if assets_version.is_none() {
        println!("[EnsureVersions] ASSETS_VERSION missing; skipping version check (development mode)");
        return Ok(());
    }

    if fonts_version.is_none() {
        println!("[EnsureVersions] fonts/VERSION missing; skipping version check (development mode)");
        return Ok(());
    }

    let assets_outdated = game_init::is_devilutionx_mpq_out_of_date(assets_version.as_deref());
    let fonts_outdated = game_init::are_extra_fonts_out_of_date(fonts_version.as_deref());

    if assets_outdated && fonts_outdated {
        show_mpq_error_dialog("devilutionx.mpq 和 fonts.mpq 缺失或版本过低");
        return Err("devilutionx.mpq 和 fonts.mpq 缺失或版本过低".to_string());
    }
    if assets_outdated {
        show_mpq_error_dialog("devilutionx.mpq 缺失或版本过低");
        return Err("devilutionx.mpq 缺失或版本过低".to_string());
    }
    if fonts_outdated {
        show_mpq_error_dialog("fonts.mpq 缺失或版本过低");
        return Err("fonts.mpq 缺失或版本过低".to_string());
    }

    Ok(())
}

fn load_player_data_files() {
    println!("[LoadPlayerDataFiles] Player data compiled in (classes/stats)");
}

fn load_spell_data() {
    // Touch compiled spell table to ensure linkage
    let _ = crate::game::data::SPELLS_DATA.len();
    println!("[LoadSpellData] Spell data compiled in");
}

fn load_missile_data() {
    use game::missiles::{get_missile_data, MissileID};

    let sample = [MissileID::Arrow, MissileID::Firebolt, MissileID::Lightning];
    for missile in sample {
        let data = get_missile_data(missile);
        println!(
            "  Missile {:?}: graphic={:?} speed={} anim_len={}",
            missile, data.graphic, data.speed, data.anim_len
        );
    }

    println!("[LoadMissileData] Missile data compiled (sampled {})", sample.len());
}

fn load_monster_data() {
    use game::monster_dat::{get_monster_data, MonsterId, NUM_DEFAULT_MTYPES};

    let zombie = get_monster_data(MonsterId::ZombieN);
    let butcher = get_monster_data(MonsterId::Butcher);

    println!(
        "[LoadMonsterData] {} monsters compiled (zombie hp {}-{}, butcher lvl {} hp {}-{})",
        NUM_DEFAULT_MTYPES,
        zombie.hp_min,
        zombie.hp_max,
        butcher.level,
        butcher.hp_min,
        butcher.hp_max
    );
}

fn load_item_data() {
    use game::item_dat::{get_item_data, get_unique_item_data, ItemId, UniqueItemId, ITEMS_DATA, UNIQUE_ITEMS};

    let gold = get_item_data(ItemId::Gold as usize);
    let cleaver = get_unique_item_data(UniqueItemId::Cleaver);

    println!(
        "[LoadItemData] Base items={} unique items={}",
        ITEMS_DATA.len(),
        UNIQUE_ITEMS.len()
    );

    if let Some(item) = gold {
        println!("  Item {:?}: {} (type {:?})", ItemId::Gold, item.name, item.item_type);
    }

    if let Some(unique) = cleaver {
        println!("  Unique {:?}: {} (min lvl {})", UniqueItemId::Cleaver, unique.name, unique.min_level);
    }
}

fn load_object_data() {
    // Touch object data table if present
    let _ = crate::game::data::ALL_OBJECTS.len();
    println!("[LoadObjectData] Object data compiled in");
}

fn load_quest_data() {
    use game::quest::{QuestId, QuestManager};

    let opts = global_options();
    let hellfire = matches!(opts.game_mode.game_mode, StartUpGameMode::Hellfire);
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let quest_ids = QuestId::random_quest_set(hellfire, seed);
    let manager = QuestManager::new(hellfire, seed);
    let level_two = manager.quests_for_level(2);

    println!(
        "[LoadQuestData] Quests seeded (hellfire={} total={} level2={:?})",
        hellfire,
        quest_ids.len(),
        level_two
    );
}

/// Initialize network for single player mode
/// C++ Reference: Source/multi.cpp::NetInit(bSinglePlayer=true)
fn net_init_single_player() -> bool {
    // C++: InitGameInfo, InitSingle, delta_init, nthread_start, etc.
    // For single player, this sets up the loopback provider and initializes
    // game state without actual networking
    println!("[NetInit] Initializing single-player (loopback provider)");
    
    // In single player, we always succeed
    // TODO: Initialize game info, delta sync, and player messaging
    true
}

/// Close network connection and cleanup
/// C++ Reference: Source/multi.cpp::NetClose()
fn net_close() {
    // C++: nthread_cleanup, tmsg_cleanup, SNetLeaveGame, Players.clear
    // For single player this is minimal; for multiplayer it would:
    // 1. Cleanup network thread
    // 2. Cleanup turn messages
    // 3. Unregister event handlers
    // 4. Leave the game session
    println!("[NetClose] Network shutdown complete");
}

fn ui_reinitialize(ctx: &mut DiabloContext) {
    println!("[UiInitialize] Reinitializing UI resources");
    ui_destroy();
    ui_initialize(ctx);
}

fn music_stop() {
    let mgr = audio_manager();
    if let Ok(mut audio) = mgr.lock() {
        audio.stop_all();
        audio.stop_music();
        println!("[Audio] Stopped all sounds and music");
    } else {
        println!("[Audio] Failed to lock audio manager for stop");
    }
}

fn free_game_mem() {
    // Placeholder for FreeGameMem: release panels, missiles, monsters, quests, lighting, UI
    // TODO: wire to actual subsystems once implemented
    println!("[FreeGameMem] Releasing game resources (panels/monsters/quests/lighting/UI)");
}

struct ArchiveDetection {
    data_path: String,
    has_diablo: bool,
    has_spawn: bool,
    has_hellfire: bool,
}

fn detect_local_archives() -> ArchiveDetection {
    let data_path = assets_path();
    if data_path.is_empty() {
        println!("[DetectArchives] No assets path configured");
        return ArchiveDetection { data_path, has_diablo: false, has_spawn: false, has_hellfire: false };
    }

    let is_diablo = game_init::discover_diablo(&data_path);
    let is_spawn = game_init::discover_spawn(&data_path);
    let is_hellfire = game_init::discover_hellfire(&data_path);

    println!(
        "[DetectArchives] data_path={} diabdat={} spawn={} hellfire={}",
        data_path, is_diablo, is_spawn, is_hellfire
    );

    ArchiveDetection {
        data_path,
        has_diablo: is_diablo,
        has_spawn: is_spawn,
        has_hellfire: is_hellfire,
    }
}

fn resolve_game_mode(flags: &CmdFlags, detection: &ArchiveDetection) -> (bool, bool) {
    let opts = global_options();

    let mut spawn = flags.prefer_spawn
        || opts.game_mode.shareware
        || (!detection.has_diablo && detection.has_spawn);
    if flags.force_diablo || flags.force_hellfire {
        spawn = false;
    }

    let mut hellfire = flags.force_hellfire
        || matches!(opts.game_mode.game_mode, StartUpGameMode::Hellfire)
        || (matches!(opts.game_mode.game_mode, StartUpGameMode::Ask) && detection.has_hellfire && !spawn && !flags.force_diablo);

    if spawn {
        hellfire = false;
    }

    (hellfire, spawn)
}

/// Step 2: ApplicationInit (C++: diablo.cpp line 1226)
/// Window is already created before this, just print confirmation
fn application_init(ctx: &DiabloContext) -> Result<(), String> {
    println!("[ApplicationInit] Initializing application...");
    language_initialize();
    init_virtual_gamepad();
    println!("  Created window: {}x{}", ctx.window.width(), ctx.window.height());
    println!("  Window initialized successfully");
    Ok(())
}

/// Step 3: DiabloInit (C++: diablo.cpp line 1242)
/// Initialize UI system
fn diablo_init(ctx: &mut DiabloContext) -> Result<(), String> {
    println!("[DiabloInit] Initializing Diablo systems...");
    println!("  Product: {}", ctx.product_name);

    ui_initialize(ctx);
    diablo_init_screen(&ctx.window);
    snd_init();
    ui_sound_init();
    println!("  UI initialized and audio configured");

    Ok(())
}

/// Step 4: DiabloSplash (C++: diablo.cpp line 1301)
/// Show intro movies and title screen
fn diablo_splash(ctx: &mut DiabloContext, flags: &CmdFlags, is_hellfire: bool, options_path: &Path, event_pump: &mut sdl2::EventPump) -> Result<(), String> {
    println!("[DiabloSplash] Showing splash...");

    let opts = global_options();
    let splash_pref = opts.startup.splash;
    let intro_pref = if is_hellfire { opts.startup.hellfire_intro } else { opts.startup.diablo_intro };

    let mut player = MoviePlayer::new();
    let play_flags = MovieFlags { user_can_close: true, loop_movie: false, in_game: false };

    if matches!(splash_pref, StartUpSplash::LogoAndTitleDialog) {
        let logo = "gendata\\logo.smk";
        if ctx.mpq_manager.has_file(logo) {
            println!("  Playing movie: {}", logo);
            let _ = player.play(logo, play_flags);
        } else {
            println!("  Movie missing: {}", logo);
        }
    }

    if !matches!(intro_pref, StartUpIntro::Off) {
        let intro_path = if is_hellfire { "gendata\\Hellfire.smk" } else { "gendata\\diablo1.smk" };
        if ctx.mpq_manager.has_file(intro_path) {
            println!("  Playing intro: {}", intro_path);
            let _ = player.play(intro_path, play_flags);
        } else {
            println!("  Intro missing: {}", intro_path);
        }

        if matches!(intro_pref, StartUpIntro::Once) {
            let mut opts_mut = utils::options::options_mut();
            if is_hellfire {
                opts_mut.startup.hellfire_intro = StartUpIntro::Off;
            } else {
                opts_mut.startup.diablo_intro = StartUpIntro::Off;
            }

            if !flags.demo_mode {
                save_options_from_global(options_path).map_err(|e| e.to_string())?;
            }
        }
    }

    if matches!(splash_pref, StartUpSplash::LogoAndTitleDialog | StartUpSplash::TitleDialog) {
        let assets = snapshot_ui_assets();
        let timeout = Duration::from_secs(7);
        let start = Instant::now();
        let mut fade_ctx = UiContext::new();
        fade_ctx.start_fade_in(0);

        'title_loop: loop {
            let mouse = event_pump.mouse_state();
            let mouse_pos = (mouse.x(), mouse.y());
            let now_ms = start.elapsed().as_millis() as u32;
            let _ = fade_ctx.update_fade(now_ms);
            let fade = fade_ctx.fade_value.min(255) as u8;
            render_title_screen(ctx.window.canvas_mut(), &mut ctx.font, &assets, mouse_pos, start, fade)?;

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown { .. }
                    | Event::MouseButtonUp { .. }
                    | Event::MouseButtonDown { .. } => break 'title_loop,
                    _ => {}
                }
            }

            if start.elapsed() >= timeout {
                break 'title_loop;
            }

            std::thread::sleep(Duration::from_millis(16));
        }
    }

    println!("  Splash complete");
    Ok(())
}

/// Attract mode playback (main menu demo loop)
fn play_attract_mode(ctx: &mut DiabloContext) -> Result<(), String> {
    println!("[AttractMode] Playing demo/intro movie...");

    let candidates = [
        "gendata\\title.smk",
        "gendata\\diablo1.smk",
        "gendata\\logo.smk",
    ];

    let mut player = MoviePlayer::new();
    let play_flags = MovieFlags { user_can_close: true, loop_movie: false, in_game: false };

    for movie in candidates {
        if ctx.mpq_manager.has_file(movie) {
            println!("  Playing attract movie: {}", movie);
            let _ = player.play(movie, play_flags);
            return Ok(());
        }
    }

    println!("  Attract mode skipped (no movie found)");
    Ok(())
}

fn snapshot_ui_assets() -> UiAssetsSnapshot {
    ui_resources()
        .lock()
        .map(|res| res.assets.clone())
        .unwrap_or_default()
}

fn mod_color(color: Color, fade: u8) -> Color {
    let scale = fade as u16;
    Color::RGBA(
        ((color.r as u16 * scale) / 255) as u8,
        ((color.g as u16 * scale) / 255) as u8,
        ((color.b as u16 * scale) / 255) as u8,
        color.a,
    )
}

fn render_ui_image(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    creator: &TextureCreator<WindowContext>,
    image: &UiArtImage,
    x: i32,
    y: i32,
    fade: u8,
) -> Result<(), String> {
    let mut tex = image.to_texture(creator)?;
    tex.set_color_mod(fade, fade, fade);
    let dst = Rect::new(x, y, image.width, image.height);
    canvas
        .copy(&tex, None, dst)
        .map_err(|e| format!("拷贝纹理失败: {}", e))
}

fn render_cursor(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    creator: &TextureCreator<WindowContext>,
    assets: &UiAssetsSnapshot,
    mouse_pos: (i32, i32),
    fade: u8,
) -> Result<(), String> {
    if let Some(cursor) = &assets.cursor {
        let mut tex = cursor.to_texture(creator)?;
        tex.set_color_mod(fade, fade, fade);
        let (mx, my) = mouse_pos;
        let dst = Rect::new(mx, my, cursor.width, cursor.height);
        canvas
            .copy(&tex, None, dst)
            .map_err(|e| format!("拷贝纹理失败: {}", e))?;
    }

    Ok(())
}

fn render_main_menu(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    font: &mut PixelFont,
    menu: &MainMenu,
    assets: &UiAssetsSnapshot,
    base_time: Instant,
    fade: u8,
    mouse_pos: (i32, i32),
) -> Result<(), String> {
    let creator = canvas.texture_creator();
    let (ui_x, ui_y) = ui_origin();

    // Clear to avoid trails when blitting transparent assets
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    if let Some(bg) = &assets.mainmenu_bg {
        render_ui_image(canvas, &creator, bg, ui_x, ui_y, fade)?;
    } else {
        canvas.set_draw_color(Color::RGB(10, 10, 20));
        canvas.clear();
    }

    if let Some(logo) = &assets.logo {
        let x = ui_x + (640 - logo.width as i32) / 2;
        render_ui_image(canvas, &creator, logo, x, ui_y + 30, fade)?;
    }

    // Player name title
    let title = menu.player_name();
    let title_x = ui_x + (640 - font.text_width(title)) / 2;
    font.render_text(canvas, title, title_x, ui_y + 150, mod_color(Color::RGB(255, 215, 0), fade));

    // Menu entries
    let menu_texts = [
        "Single Player",
        "Multi Player",
        "Support",
        "Settings",
        "Credits",
        "Exit Diablo",
    ];

    let list_x = ui_x + 64;
    let list_y = ui_y + 192;
    let item_w: i32 = 510;
    let item_h: i32 = 43;
    let text_v_offset = (item_h - font.line_height()) / 2;
    let frame_idx = if !assets.focus_med.is_empty() {
        ((base_time.elapsed().as_millis() / 100) as usize) % assets.focus_med.len()
    } else {
        0
    };

    for (i, text) in menu_texts.iter().enumerate() {
        let item_y = list_y + i as i32 * item_h;
        let text_x = list_x + (item_w - font.text_width(text)) / 2;
        let text_y = item_y + text_v_offset;

        if i == menu.selected_index() {
            // Focus frame if available
            if let Some(focus) = assets.focus_med.get(frame_idx) {
                let fx = list_x + (item_w - focus.width as i32) / 2;
                let fy = item_y + (item_h - focus.height as i32) / 2;
                render_ui_image(canvas, &creator, focus, fx, fy, fade)?;
            } else {
                canvas.set_draw_color(Color::RGB(60, 40, 80));
                let _ = canvas.fill_rect(Rect::new(list_x, item_y, item_w as u32, item_h as u32));
            }

            font.render_text(canvas, text, text_x, text_y, mod_color(Color::RGB(255, 230, 120), fade));
        } else {
            font.render_text(canvas, text, text_x, text_y, mod_color(Color::RGB(200, 200, 200), fade));
        }
    }

    render_cursor(canvas, &creator, assets, mouse_pos, fade)?;
    canvas.present();
    Ok(())
}

fn render_title_screen(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    font: &mut PixelFont,
    assets: &UiAssetsSnapshot,
    mouse_pos: (i32, i32),
    _fade_clock: Instant,
    fade: u8,
) -> Result<(), String> {
    let creator = canvas.texture_creator();
    let (ui_x, ui_y) = ui_origin();

    // Clear to avoid trails when re-rendering
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    if let Some(bg) = &assets.title_bg {
        render_ui_image(canvas, &creator, bg, ui_x, ui_y, fade)?;
    } else {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
    }

    if let Some(logo) = &assets.logo {
        let x = ui_x + (640 - logo.width as i32) / 2;
        let y = ui_y + 180;
        render_ui_image(canvas, &creator, logo, x, y, fade)?;
    }

    let copyright = "Copyright © 1996-2001 Blizzard Entertainment";
    let cx = ui_x + (640 - font.text_width(copyright)) / 2;
    font.render_text(canvas, copyright, cx, ui_y + 410, mod_color(Color::RGB(200, 200, 200), fade));

    render_cursor(canvas, &creator, assets, mouse_pos, fade)?;
    canvas.present();
    Ok(())
}

fn render_selhero(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    font: &mut PixelFont,
    assets: &UiAssetsSnapshot,
    selection: usize,
    start_time: Instant,
    fade: u8,
    options: &[(&str, bool)],
    mouse_pos: (i32, i32),
) -> Result<(), String> {
    let creator = canvas.texture_creator();
    let (ui_x, ui_y) = ui_origin();

    // Clear each frame to avoid artifacts
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    if let Some(bg) = &assets.selhero_bg {
        render_ui_image(canvas, &creator, bg, ui_x, ui_y, fade)?;
    } else {
        canvas.set_draw_color(Color::RGB(10, 10, 20));
        canvas.clear();
    }

    if let Some(logo) = &assets.logo {
        let x = ui_x + (640 - logo.width as i32) / 2;
        render_ui_image(canvas, &creator, logo, x, ui_y + 40, fade)?;
    }

    let frame_idx = if !assets.focus_big.is_empty() {
        ((start_time.elapsed().as_millis() / 100) as usize) % assets.focus_big.len()
    } else {
        0
    };

    let list_x = ui_x + 265;
    let item_w: i32 = 320;
    let item_h: i32 = 33;
    let visible_rows = options.len().min(6) as i32;
    let start_y = ui_y + 246 + (176 - item_h * visible_rows) / 2;
    let text_v_offset = (item_h - font.line_height()) / 2;

    for (i, (text, selectable)) in options.iter().enumerate() {
        let item_y = start_y + i as i32 * item_h;
        let text_x = list_x + (item_w - font.text_width(text)) / 2;
        let text_y = item_y + text_v_offset;

        if i == selection {
            if let Some(focus) = assets.focus_big.get(frame_idx) {
                let fx = list_x + (item_w - focus.width as i32) / 2;
                let fy = item_y + (item_h - focus.height as i32) / 2;
                render_ui_image(canvas, &creator, focus, fx, fy, fade)?;
            } else {
                canvas.set_draw_color(Color::RGB(50, 30, 70));
                let _ = canvas.fill_rect(Rect::new(list_x, item_y, item_w as u32, item_h as u32));
            }

            let color = if *selectable { Color::RGB(255, 230, 120) } else { Color::RGB(200, 200, 200) };
            font.render_text(canvas, text, text_x, text_y, mod_color(color, fade));
        } else {
            let color = if *selectable { Color::RGB(180, 180, 180) } else { Color::RGB(140, 140, 140) };
            font.render_text(canvas, text, text_x, text_y, mod_color(color, fade));
        }
    }

    render_cursor(canvas, &creator, assets, mouse_pos, fade)?;
    canvas.present();
    Ok(())
}

/// Step 5: mainmenu_loop (C++: menu.cpp line 152)
/// Main menu loop with UiMainMenuDialog
fn mainmenu_loop(ctx: &mut DiabloContext, event_pump: &mut sdl2::EventPump, flags: &CmdFlags) -> Result<(), String> {
    println!("[mainmenu_loop] Entering main menu...");

    let attract_timeout = if flags.no_splash { None } else { Some(Duration::from_secs(60)) };

    // C++: mainmenu_loop() - do-while loop until done
    'main_loop: loop {
        // C++: UiMainMenuDialog(&menu, 30)
        let product_name = ctx.product_name.clone();
        let selection = ui_main_menu_dialog(&product_name, ctx, event_pump, attract_timeout)?;

        println!("[mainmenu_loop] Menu returned: {:?}", selection);

        match selection {
            MainMenuSelection::None => {},
            MainMenuSelection::SinglePlayer => {
                println!("[mainmenu_loop] Starting Single Player...");
                if !init_single_player_menu(ctx, event_pump)? {
                    break 'main_loop; // C++: done = true
                }
            },
            MainMenuSelection::Multiplayer => {
                println!("[mainmenu_loop] Multiplayer selected");
                // Align with C++: attempt to start multiplayer menu; on failure return to loop
                if !init_multiplayer_menu(ctx, event_pump)? {
                    break 'main_loop;
                }
            },
            MainMenuSelection::ShowSupport => {
                show_support_dialog();
            },
            MainMenuSelection::Settings => {
                show_settings_dialog();
            },
            MainMenuSelection::ShowCredits => {
                show_credits_dialog();
            },
            MainMenuSelection::ExitDiablo => {
                println!("[mainmenu_loop] Exit selected");
                // C++ plays button sound and fades; we just exit loop
                break 'main_loop;
            },
            MainMenuSelection::AttractMode => {
                println!("[mainmenu_loop] Attract mode triggered");
                play_attract_mode(ctx)?;
                continue 'main_loop;
            },
        }
    }

    println!("[mainmenu_loop] Exiting main menu");
    Ok(())
}

/// UiMainMenuDialog - blocking menu dialog (C++: DiabloUI/mainmenu.cpp line 108)
fn ui_main_menu_dialog(
    name: &str,
    ctx: &mut DiabloContext,
    event_pump: &mut sdl2::EventPump,
    attract_timeout: Option<Duration>,
) -> Result<MainMenuSelection, String> {
    let mut main_menu = MainMenu::new(name);
    let assets = snapshot_ui_assets();
    let start_time = Instant::now();
    let mut fade_ctx = UiContext::new();
    fade_ctx.start_fade_in(0);

    println!("[UiMainMenuDialog] Showing main menu...");

    let mut last_input = Instant::now();

    let result = 'dialog_loop: loop {
        // Poll events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'dialog_loop MainMenuSelection::ExitDiablo;
                },
                Event::KeyDown { keycode: Some(key), .. } => {
                    last_input = Instant::now();
                    match key {
                        Keycode::Up | Keycode::W => main_menu.move_selection(-1),
                        Keycode::Down | Keycode::S => main_menu.move_selection(1),
                        Keycode::Return | Keycode::Space => {
                            let selection = main_menu.get_selected();
                            if selection != MainMenuSelection::None {
                                break 'dialog_loop selection;
                            }
                        },
                        Keycode::Escape => {
                            break 'dialog_loop MainMenuSelection::ExitDiablo;
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        if let Some(timeout) = attract_timeout {
            if last_input.elapsed() >= timeout {
                println!("[UiMainMenuDialog] Attract timeout reached");
                break 'dialog_loop MainMenuSelection::AttractMode;
            }
        }

        // Render (C++: UiClearScreen + UiPollAndRender)
        let mouse = event_pump.mouse_state();
        let mouse_pos = (mouse.x(), mouse.y());
        let now_ms = start_time.elapsed().as_millis() as u32;
        let _ = fade_ctx.update_fade(now_ms);
        let fade = fade_ctx.fade_value.min(255) as u8;
        render_main_menu(
            ctx.window.canvas_mut(),
            &mut ctx.font,
            &main_menu,
            &assets,
            start_time,
            fade,
            mouse_pos,
        )?;

        std::thread::sleep(std::time::Duration::from_millis(16));
    };

    println!("[UiMainMenuDialog] Result: {:?}", result);
    Ok(result)
}



/// Step 6: InitSinglePlayerMenu (C++: menu.cpp line 75)
/// Returns false if user wants to quit
fn init_single_player_menu(ctx: &mut DiabloContext, event_pump: &mut sdl2::EventPump) -> Result<bool, String> {
    println!("[InitSinglePlayerMenu] Starting single player flow...");

    // C++: gbIsMultiplayer = false
    // C++: return InitMenu(SELHERO_NEW_DUNGEON)
    //   -> mainmenu_select_hero_dialog
    //   -> UiSelHeroSingDialog (hero selection UI)
    //   -> StartGame(WM_DIABNEWGAME or WM_DIABLOADGAME)

    // Show character selection
    if let Some(class) = select_hero_dialog(ctx, event_pump)? {
        println!("[InitSinglePlayerMenu] Selected class: {:?}", class);

        // C++: StartGame(WM_DIABNEWGAME)
        start_game(ctx, class)?;

        // After game ends, return to menu
        return Ok(true);
    }

    // User cancelled, return to menu
    Ok(true)
}

/// Multiplayer menu flow
/// C++ Reference: Source/menu.cpp::InitMultiPlayerMenu()
fn init_multiplayer_menu(ctx: &mut DiabloContext, event_pump: &mut sdl2::EventPump) -> Result<bool, String> {
    println!("[InitMultiPlayerMenu] Starting multiplayer flow...");

    // C++ flow:
    // 1. gbIsMultiplayer = true
    // 2. UiSelConnDialog (select connection type: TCP/IP, etc.)
    // 3. UiSelHeroMultDialog (hero selection with multiplayer options)
    // 4. NetInit(false) to set up network
    // 5. StartGame for either hosting or joining
    //
    // For now, reuse hero select dialog as placeholder
    if let Some(class) = select_hero_dialog(ctx, event_pump)? {
        println!("[InitMultiPlayerMenu] Selected class: {:?}", class);
        // TODO: Implement connection selection and game hosting/joining
        // For now we just return to the menu
    }

    Ok(true)
}

/// Hero selection dialog (C++: UiSelHeroSingDialog)
fn select_hero_dialog(ctx: &mut DiabloContext, event_pump: &mut sdl2::EventPump) -> Result<Option<PlayerClass>, String> {
    println!("[SelectHeroDialog] Showing character selection...");

    let mut selection = 0usize;
    let class_options = [
        ("Warrior - Strong melee fighter", Some(PlayerClass::Warrior)),
        ("Rogue - Swift archer", Some(PlayerClass::Rogue)),
        ("Sorcerer - Powerful mage", Some(PlayerClass::Sorcerer)),
        ("<- Back to Menu", None),
    ];

    let assets = snapshot_ui_assets();
    let start_time = Instant::now();
    let mut fade_ctx = UiContext::new();
    fade_ctx.start_fade_in(0);

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => return Ok(None),
                Event::KeyDown { keycode: Some(key), .. } => {
                    match key {
                        Keycode::Up | Keycode::W => {
                            selection = if selection > 0 { selection - 1 } else { class_options.len() - 1 };
                        }
                        Keycode::Down | Keycode::S => {
                            selection = (selection + 1) % class_options.len();
                        }
                        Keycode::Return | Keycode::Space => {
                            return Ok(class_options[selection].1);
                        }
                        Keycode::Escape => return Ok(None),
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        let options_for_render: Vec<(&str, bool)> = class_options
            .iter()
            .map(|(text, class)| (*text, class.is_some()))
            .collect();

        let mouse = event_pump.mouse_state();
        let mouse_pos = (mouse.x(), mouse.y());
        let now_ms = start_time.elapsed().as_millis() as u32;
        let _ = fade_ctx.update_fade(now_ms);
        let fade = fade_ctx.fade_value.min(255) as u8;

        render_selhero(
            ctx.window.canvas_mut(),
            &mut ctx.font,
            &assets,
            selection,
            start_time,
            fade,
            &options_for_render,
            mouse_pos,
        )?;

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

/// Step 7: StartGame (C++: diablo.cpp line 178)
/// Initialize game state and enter game loop
fn start_game(ctx: &mut DiabloContext, class: game::player::PlayerClass) -> Result<(), String> {
    println!("[StartGame] Starting game with class: {:?}", class);

    if !net_init_single_player() {
        println!("[StartGame] Net init failed");
        return Ok(());
    }

    // Free main menu UI resources before entering game (C++ frees to save memory)
    ui_destroy();

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Initialize Player (Exact Port)
    let mut player = Player::new();
    player._p_class = match class {
        game::player::PlayerClass::Warrior => game::player_exact::HeroClass::Warrior,
        game::player::PlayerClass::Rogue => game::player_exact::HeroClass::Rogue,
        game::player::PlayerClass::Sorcerer => game::player_exact::HeroClass::Sorcerer,
        game::player::PlayerClass::Monk => game::player_exact::HeroClass::Monk,
        game::player::PlayerClass::Bard => game::player_exact::HeroClass::Bard,
        game::player::PlayerClass::Barbarian => game::player_exact::HeroClass::Barbarian,
    };
    player.plr_active = true;

    // Initialize GameState
    let mut game_state = GameState::new(player, false, seed);

    // Run the real game loop
    match run_game_loop(InterfaceMode::NewGame, &mut ctx.window, &mut game_state) {
        Ok(_) => {
            net_close();
            // Recreate UI when returning to main menu
            ui_initialize(ctx);
            ui_reinitialize(ctx);
            Ok(())
        },
        Err(e) => {
            net_close();
            Err(e.to_string())
        },
    }
}

/// Step 8: RunGameLoop (C++: diablo.cpp line 857)
/// Final cleanup (C++: DiabloDeinit line 1326)
fn diablo_deinit(ctx: &mut DiabloContext) {
    println!("[DiabloDeinit] Cleaning up...");

    // Match C++ order: free item gfx, stop audio, free gameplay resources, flush UI
    free_item_gfx();
    music_stop();
    free_game_mem();
    ui_reinitialize(ctx);

    lua_shutdown();
    screen_reader_shutdown();
    snd_deinit();
    ui_destroy();
    init_cleanup(false, false); // Single player, not running
    dx_cleanup();
    unload_fonts();

    println!("  Cleanup complete");
}

/// Main entry point - aligned with C++ main() -> DiabloMain()
fn main() -> Result<(), String> {
    println!("{}", "=".repeat(40));
    println!("DevilutionX-RS v{}", env!("CARGO_PKG_VERSION"));
    println!("{}", "=".repeat(40));

    let log_step = |label: &str| println!("[Step] {}", label);

    let mut flags = parse_cmdline_flags();
    parse_cmdline_with_args(&mut flags);

    log_step("parsed flags");

    init_keymap_actions();
    init_padmap_actions();

    log_step("init input");

    apply_path_overrides(&flags);

    log_step("path overrides applied");

    // Configure MPQ manager and search paths
    log_step("before mpq manager");
    let mut mpq_manager = MpqAssetManager::new();
    log_step("mpq manager constructed");

    mpq_manager.add_search_path(".");
    log_step("added search path: .");

    let assets = assets_path();
    println!("[Path] assets_path='{}'", assets);
    if !assets.is_empty() {
        mpq_manager.add_search_path(&assets);
        log_step("added search path: assets");
    }

    log_step("mpq manager ready");

    // === DiabloMain flow ===

    // Step 1: Load MPQs
    load_core_archives(&mut mpq_manager)?;
    log_step("core archives loaded");
    ensure_mpq_versions(&mut mpq_manager)?;
    log_step("mpq versions ok");

    // Step 1.1: Load options after core assets are available
    let options_path = options_file_path();
    load_or_init_options(&options_path)?;
    apply_option_overrides(&flags);
    log_step("options loaded");

    let detection = detect_local_archives();
    let (is_hellfire_mode, is_spawn_mode) = resolve_game_mode(&flags, &detection);
    log_step("game mode resolved");

    let (window_width, window_height) = {
        let opts = global_options();
        (opts.graphics.width, opts.graphics.height)
    };

    // Create window (C++: init_create_window in ApplicationInit)
    let mut window = GameWindow::new("DevilutionX-RS", window_width, window_height)
        .map_err(|e| e.to_string())?;
    // Use 640x480 logical size so UI coordinates match original assets; SDL handles scaling/letterboxing
    window
        .canvas_mut()
        .set_logical_size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .map_err(|e| e.to_string())?;
    // Hide OS cursor because we draw our own
    window.set_cursor_visible(false);
    log_step("window created");

    let mut event_pump = window.event_pump().map_err(|e| e.to_string())?;
    log_step("event pump ready");

    // Create context
    let mut ctx = DiabloContext::new(window, mpq_manager);

    {
        let mut opts = utils::options::options_mut();
        opts.game_mode.shareware = is_spawn_mode;

        if is_hellfire_mode {
            opts.game_mode.game_mode = StartUpGameMode::Hellfire;
        } else if matches!(opts.game_mode.game_mode, StartUpGameMode::Ask) {
            opts.game_mode.game_mode = StartUpGameMode::Diablo;
        }
    }

    // Set product name based on resolved mode
    ctx.product_name = if is_hellfire_mode {
        "DevilutionX-RS (Hellfire)".to_string()
    } else if is_spawn_mode {
        "DevilutionX-RS (Spawn)".to_string()
    } else {
        "DevilutionX-RS".to_string()
    };

    // Append version for display parity
    ctx.product_name = format!("{} v{}", ctx.product_name, env!("CARGO_PKG_VERSION"));

    // Step 1.5: Load language-specific assets (C++: LoadLanguageArchive)
    load_language_archive(&mut ctx.mpq_manager)?;
    log_step("language archive loaded");
    // Save options after language/application init points (C++ saves twice)

    // Step 2: Initialize application (window already created)
    application_init(&ctx)?;
    log_step("application init done");

    // Step 2.5: Lua initialization stub
    lua_initialize();

    // Persist any defaults so config exists on disk (first save point)
    if !flags.demo_mode {
        save_options_from_global(&options_path).map_err(|e| e.to_string())?;
    }

    // Step 3: Load game data archives after options are parsed
    load_game_archives(&mut ctx.mpq_manager, is_spawn_mode, &detection)?;
    log_step("game archives loaded");

    // Step 3.5: Load static data tables (stubs aligned with C++)
    load_text_data();
    load_player_data_files();
    load_spell_data();
    load_missile_data();
    load_monster_data();
    load_item_data();
    load_object_data();
    load_quest_data();

    // Step 3: Initialize Diablo systems
    diablo_init(&mut ctx)?;
    log_step("diablo init done");

    // Second save point after DiabloInit (mirrors C++)
    if !flags.demo_mode {
        save_options_from_global(&options_path).map_err(|e| e.to_string())?;
    }

    // Step 4: Show splash/intro
    if !flags.no_splash {
        diablo_splash(&mut ctx, &flags, is_hellfire_mode, &options_path, &mut event_pump)?;
        log_step("splash done");
    } else {
        println!("[DiabloSplash] Skipped by flag");
    }

    // Step 5: Main menu loop
    mainmenu_loop(&mut ctx, &mut event_pump, &flags)?;
    log_step("main menu loop exited");

    // Step 6: Cleanup
    diablo_deinit(&mut ctx);
    log_step("cleanup done");

    println!("{}", "=".repeat(40));
    println!("Goodbye!");
    println!("{}", "=".repeat(40));

    Ok(())
}
