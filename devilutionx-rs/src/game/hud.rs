//! In-game HUD panel (Diablo bottom panel).
//!
//! Draws the iconic Diablo bottom panel over the 640x480 logical viewport:
//!   - panel background (procedurally drawn gradient bar; the real
//!     `ctrlpan\panel8.cel` art can be dropped in later without changing the
//!     layout constants here)
//!   - life sphere (left) and mana sphere (right), filled by hp/max_hp and
//!     mana/max_mana ratio
//!   - experience bar (thin gold strip above the panel)
//!   - central skill slot + belt slots (1-8)
//!   - stat text (level / HP / Mana / gold) via `PixelFont`
//!
//! ## Coordinates
//! All drawing happens in logical 640x480 space (the canvas has
//! `set_logical_size(640, 480)` applied in `main.rs`). The panel occupies the
//! bottom strip: y = PANEL_TOP (336) .. 480, full 640 width.
//!
//! ## Data source
//! Reads only from `GameState::player` (`player_exact::Player`):
//!   - `_p_hit_points` / `_p_max_hp`  (64x fixed-point)
//!   - `_p_mana`      / `_p_max_mana` (64x fixed-point)
//!   - `_p_level`, `_p_experience`, `_p_gold`
//! No mutation of game state is performed.

use crate::engine::font::PixelFont;
use crate::engine::surface::{self as surface_mod, Surface};
use crate::engine::types::{Point, Rectangle, Size};
use crate::engine::window::GameWindow;
use crate::game::game_state::GameState;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Logical render resolution (matches `game_loop::draw_and_blit`).
const SCREEN_W: i32 = 640;
const SCREEN_H: i32 = 480;

/// Panel geometry (Diablo original: 640 wide, 144 tall, at the bottom).
const PANEL_TOP: i32 = SCREEN_H - PANEL_H; // 336
const PANEL_H: i32 = 144;
const PANEL_W: i32 = 640;

/// Sphere (life/mana globe) geometry. Diablo globes are ~56px diameter.
const SPHERE_R: i32 = 29; // radius => ~58px diameter
/// Life sphere centre (lower-left of panel).
const LIFE_CX: i32 = 90;
const LIFE_CY: i32 = PANEL_TOP + 60;
/// Mana sphere centre (lower-right of panel).
const MANA_CX: i32 = SCREEN_W - 90;
const MANA_CY: i32 = PANEL_TOP + 60;

/// Experience bar: thin strip sitting just above the panel, full-ish width.
const XP_BAR_X: i32 = 162;
const XP_BAR_W: i32 = 316; // spans most of the centre
const XP_BAR_Y: i32 = PANEL_TOP - 7;
const XP_BAR_H: i32 = 5;

/// Central skill slot.
const SKILL_SLOT_X: i32 = 304;
const SKILL_SLOT_Y: i32 = PANEL_TOP + 56;
const SKILL_SLOT: i32 = 56;

/// Belt slots row (8 potions), centred under the skill slot.
const BELT_SLOTS: usize = 8;
const BELT_SLOT_W: i32 = 29;
const BELT_SLOT_H: i32 = 29;
const BELT_Y: i32 = PANEL_TOP + PANEL_H - BELT_SLOT_H - 4;
/// Total width of all 8 belt slots; used to centre the row.
const BELT_TOTAL_W: i32 = (BELT_SLOTS as i32) * BELT_SLOT_W;

/// Diablo 1 per-level experience thresholds (XP required to *reach* the next
/// level), lifted from the original `Experience.tsv`. Index `i` = the threshold
/// to go from level `(i+1)` to `(i+2)`; `XP_THRESHOLDS[level-1]` is the XP needed
/// to leave `level`. Level 50 (max) maps to `u32::MAX` sentinel.
///
/// C++ Reference: `Source/playerdat.cpp::GetNextExperienceThresholdForLevel`.
const XP_THRESHOLDS: [u32; 50] = [
    2000, 4620, 8040, 12489, 18258, 25712, 35309, 47622, 63364, 83419, 108879, 141086, 181683,
    231075, 313656, 424067, 571190, 766569, 1025154, 1366227, 1814568, 2401895, 3168651, 4166200,
    5459523, 7130496, 9281874, 12042092, 15571031, 20066900, 25774405, 32994399, 42095202,
    53525811, 67831218, 85670061, 107834823, 135274799, 169122009, 210720231, 261657253,
    323800420, 399335440, 490808349, 601170414, 733825617, 892680222, 1082908612, 1310707109,
    1583495809,
];

const MAX_PLAYER_LEVEL: u8 = 50;

/// Experience progress within the current level, as a fraction in 0.0..=1.0.
///
/// `level` is 1-based. For `level` L the relevant window is
/// `[XP_THRESHOLDS[L-2], XP_THRESHOLDS[L-1])` (the XP needed to *leave* level L).
/// Level 1 starts at 0; max level is always 1.0.
pub fn xp_progress(level: u8, experience: u32) -> f32 {
    if level >= MAX_PLAYER_LEVEL {
        return 1.0;
    }
    let cur_idx = (level as usize).saturating_sub(1); // threshold to leave current level
    let next = XP_THRESHOLDS.get(cur_idx).copied().unwrap_or(u32::MAX);
    let prev = if level <= 1 {
        0
    } else {
        XP_THRESHOLDS.get(cur_idx - 1).copied().unwrap_or(0)
    };
    if next <= prev {
        // Degenerate table entry; treat as full to avoid div-by-zero.
        return if experience >= next { 1.0 } else { 0.0 };
    }
    if experience <= prev {
        return 0.0;
    }
    if experience >= next {
        return 1.0;
    }
    let span = (next - prev) as f32;
    ((experience - prev) as f32) / span
}

// ---------------------------------------------------------------------------
// Panel palette (procedural; approximates Diablo's stone panel tones)
// ---------------------------------------------------------------------------
mod pal {
    use super::Color;
    pub const PANEL_DARK: Color = Color::RGB(30, 24, 18);
    pub const PANEL_MID: Color = Color::RGB(58, 46, 34);
    pub const PANEL_LIGHT: Color = Color::RGB(96, 78, 56);
    pub const PANEL_EDGE: Color = Color::RGB(140, 116, 84);
    pub const SLOT_FILL: Color = Color::RGB(20, 16, 12);
    pub const SLOT_EDGE: Color = Color::RGB(150, 124, 90);
    pub const LIFE_FILL: Color = Color::RGB(168, 24, 24);
    pub const LIFE_FILL_LO: Color = Color::RGB(96, 12, 12);
    pub const MANA_FILL: Color = Color::RGB(32, 64, 200);
    pub const MANA_FILL_LO: Color = Color::RGB(18, 34, 110);
    pub const SPHERE_RIM: Color = Color::RGB(190, 170, 130);
    pub const SPHERE_GLASS: Color = Color::RGBA(180, 200, 220, 40);
    pub const XP_BACK: Color = Color::RGB(24, 18, 12);
    pub const XP_FILL: Color = Color::RGB(220, 200, 120);
    pub const XP_EDGE: Color = Color::RGB(90, 74, 44);
    pub const TEXT: Color = Color::RGB(240, 230, 200);
    pub const TEXT_DIM: Color = Color::RGB(180, 168, 140);
}

/// Draw the full HUD onto `window`'s canvas. Called at the end of
/// `draw_and_blit`, after the world + player sprite.
///
/// Reads player stats from `game_state.player` (64x fixed-point hp/mana). All
/// drawing is procedural (rectangles + per-pixel sphere fill) so it works with
/// or without the real Diablo panel CEL art.
pub fn draw_hud(window: &mut GameWindow, game_state: &GameState, palette_hud_drawn: bool) {
    let canvas = window.canvas_mut();
    let player = &game_state.player;

    // Snapshot the values we need (avoid holding field borrows while drawing).
    let hp = player._p_hit_points;
    let max_hp = player._p_max_hp;
    let mana = player._p_mana;
    let max_mana = player._p_max_mana;
    let level = player._p_level;
    let experience = player._p_experience;
    let gold = player._p_gold;

    if !palette_hud_drawn {
        // The palette backbuffer did not draw the HUD (fallback render
        // path without faithful art): draw the full panel on the canvas.
        draw_panel_background(canvas);
        draw_sphere(canvas, LIFE_CX, LIFE_CY, SPHERE_R, fill_ratio(hp, max_hp), SphereKind::Life);
        draw_sphere(canvas, MANA_CX, MANA_CY, SPHERE_R, fill_ratio(mana, max_mana), SphereKind::Mana);
        draw_skill_slot(canvas);
        draw_belt(canvas);
        draw_xp_bar(canvas, level, experience);
    }

    // Text overlay. PixelFont renders at scale 2 for readability.
    let font = PixelFont::new(2);
    draw_stats_text(canvas, &font, hp, max_hp, mana, max_mana, level, gold);
}

/// 8-bit palette indices (Diablo's classic 256-colour palette, procedural
/// approximation of the stone panel / red-blue globes / gold XP bar).
mod pal_idx {
    pub const PANEL: u8 = 0x52; // mid brown stone
    pub const PANEL_EDGE: u8 = 0x5E; // lighter stone edge
    pub const SLOT: u8 = 0x12; // dark slot fill
    pub const SPHERE_BG: u8 = 0x0E; // dark glass interior
    pub const LIFE: u8 = 0x20; // red
    pub const LIFE_LO: u8 = 0x18; // dark red
    pub const MANA: u8 = 0x98; // blue
    pub const MANA_LO: u8 = 0x90; // dark blue
    pub const XP_BACK: u8 = 0x10;
    pub const XP_FILL: u8 = 0xE0; // gold
}

/// Draw the HUD panel into the 8-bit palette backbuffer: panel background,
/// life/mana spheres, skill slot, belt slots and XP bar. This is the Rust
/// counterpart of C++ `DrawMain` (scrollrt.cpp) which draws the bottom panel
/// onto the same surface as the world before the palette-converted upload.
/// The text overlay still renders on the canvas via [`draw_hud`].
pub fn draw_hud_palette(surface: &mut Surface, player: &crate::game::player_exact::Player) {
    use pal_idx::*;
    // Panel background strip + top bevel.
    surface_mod::fill_rect(
        surface,
        Rectangle::new(Point::new(0, PANEL_TOP), Size::new(PANEL_W, PANEL_H)),
        PANEL,
    );
    surface_mod::fill_rect(
        surface,
        Rectangle::new(Point::new(0, PANEL_TOP), Size::new(PANEL_W, 2)),
        PANEL_EDGE,
    );
    // Life / mana spheres.
    draw_sphere_palette(
        surface,
        LIFE_CX,
        LIFE_CY,
        SPHERE_R,
        fill_ratio(player._p_hit_points, player._p_max_hp),
        LIFE,
        LIFE_LO,
    );
    draw_sphere_palette(
        surface,
        MANA_CX,
        MANA_CY,
        SPHERE_R,
        fill_ratio(player._p_mana, player._p_max_mana),
        MANA,
        MANA_LO,
    );
    // Skill slot.
    surface_mod::fill_rect(
        surface,
        Rectangle::new(Point::new(SKILL_SLOT_X, SKILL_SLOT_Y), Size::new(SKILL_SLOT, SKILL_SLOT)),
        SLOT,
    );
    // Belt slots.
    for i in 0..BELT_SLOTS {
        let bx = SKILL_SLOT_X + SKILL_SLOT / 2 - BELT_TOTAL_W / 2 + i as i32 * BELT_SLOT_W;
        surface_mod::fill_rect(
            surface,
            Rectangle::new(Point::new(bx, BELT_Y), Size::new(BELT_SLOT_W - 1, BELT_SLOT_H)),
            SLOT,
        );
    }
    // XP bar (thin gold strip above the panel).
    surface_mod::fill_rect(
        surface,
        Rectangle::new(Point::new(XP_BAR_X, XP_BAR_Y), Size::new(XP_BAR_W, XP_BAR_H)),
        XP_BACK,
    );
    let progress = xp_progress(player._p_level, player._p_experience);
    let fill_w = ((XP_BAR_W as f32) * progress) as i32;
    if fill_w > 0 {
        surface_mod::fill_rect(
            surface,
            Rectangle::new(Point::new(XP_BAR_X, XP_BAR_Y), Size::new(fill_w, XP_BAR_H)),
            XP_FILL,
        );
    }
}

/// Per-pixel liquid sphere fill on the 8-bit surface, matching the canvas
/// `draw_sphere` geometry: liquid grows from the bottom upward.
fn draw_sphere_palette(
    surface: &mut Surface,
    cx: i32,
    cy: i32,
    r: i32,
    ratio: f32,
    fill: u8,
    fill_lo: u8,
) {
    let r = r.max(1);
    // Dark glass interior.
    for dy in -r..=r {
        let hw = circle_half_width(r, dy);
        if hw <= 0 {
            continue;
        }
        for x in (cx - hw)..=(cx + hw) {
            if let Some(px) = surface.at_mut(x, cy + dy) {
                *px = pal_idx::SPHERE_BG;
            }
        }
    }
    // Liquid region (bottom-up fill, same surface_y formula as canvas).
    let surface_y = cy + r - ((r as f32 * 2.0 * ratio).round() as i32);
    for dy in (surface_y - cy)..=r {
        let hw = circle_half_width(r, dy);
        if hw <= 0 {
            continue;
        }
        for x in (cx - hw)..=(cx + hw) {
            if let Some(px) = surface.at_mut(x, cy + dy) {
                *px = if cy + dy >= surface_y { fill } else { fill_lo };
            }
        }
    }
}

/// Convert 64x fixed-point (current, max) into a 0.0..=1.0 fill ratio.
fn fill_ratio(current: i32, max: i32) -> f32 {
    if max <= 0 {
        return 0.0;
    }
    let c = (current >> 6).max(0);
    let m = max >> 6;
    if m <= 0 {
        return 0.0;
    }
    (c as f32 / m as f32).clamp(0.0, 1.0)
}

/// Draw the bottom panel as a vertical gradient with a bevelled top edge.
fn draw_panel_background(canvas: &mut Canvas<Window>) {
    // Top bevel highlight + shadow line for a "stone edge" look.
    canvas.set_draw_color(pal::PANEL_EDGE);
    let _ = canvas.fill_rect(Rect::new(0, PANEL_TOP - 1, PANEL_W as u32, 1));
    canvas.set_draw_color(pal::PANEL_LIGHT);
    let _ = canvas.fill_rect(Rect::new(0, PANEL_TOP, PANEL_W as u32, 2));
    // Gradient body: fade from mid (top) to dark (bottom) in a few bands.
    let bands: i32 = 16;
    let band_h = (PANEL_H - 2) / bands;
    let denom: f32 = ((bands - 1) as f32).max(1.0);
    for i in 0..bands {
        let t = (i as f32) / denom;
        let c = lerp_color(pal::PANEL_MID, pal::PANEL_DARK, t);
        canvas.set_draw_color(c);
        let y = PANEL_TOP + 2 + i * band_h;
        let h = if i == bands - 1 {
            PANEL_H - 2 - i * band_h
        } else {
            band_h
        };
        let _ = canvas.fill_rect(Rect::new(0, y, PANEL_W as u32, h as u32));
    }
}

/// Which liquid a sphere holds (drives the fill gradient + rim tint).
#[derive(Clone, Copy)]
enum SphereKind {
    Life,
    Mana,
}

/// Draw a globe centred at (cx, cy) with the given radius, filled from the
/// bottom up to `ratio` of its height.
///
/// The fill is a circle clipped to a horizontal band: for each scanline row we
/// compute the circle's half-width at that y and fill either the empty (glass)
/// span or the liquid span depending on whether the row is below the liquid
/// surface. The liquid uses a vertical gradient (bright at the top of the
/// liquid, darker at the bottom).
fn draw_sphere(canvas: &mut Canvas<Window>, cx: i32, cy: i32, r: i32, ratio: f32, kind: SphereKind) {
    let r = r.max(1);
    // Outer rim (stone setting).
    canvas.set_draw_color(pal::PANEL_DARK);
    for dy in -r - 2..=r + 2 {
        let hw = circle_half_width(r + 2, dy);
        if hw > 0 {
            let _ = canvas.fill_rect(Rect::new(cx - hw, cy + dy, (hw * 2) as u32, 1));
        }
    }
    // Glass interior background (dark).
    canvas.set_draw_color(pal::SLOT_FILL);
    fill_disc(canvas, cx, cy, r);

    // Liquid surface y. ratio=1 => fill the whole disc; ratio=0 => none.
    // Liquid grows from the bottom upward.
    let surface_y = cy + r - ((r as f32 * 2.0 * ratio).round() as i32);

    let (top, bot) = match kind {
        SphereKind::Life => (pal::LIFE_FILL, pal::LIFE_FILL_LO),
        SphereKind::Mana => (pal::MANA_FILL, pal::MANA_FILL_LO),
    };

    // Fill the liquid region: rows from surface_y down to cy + r.
    for dy in (surface_y - cy)..=r {
        let hw = circle_half_width(r, dy);
        if hw <= 0 {
            continue;
        }
        // Gradient: top of liquid is bright, bottom is darker.
        let depth = if r == 0 {
            0.0
        } else {
            ((dy - (surface_y - cy)) as f32 / (r as f32)).clamp(0.0, 1.0)
        };
        canvas.set_draw_color(lerp_color(top, bot, depth));
        let _ = canvas.fill_rect(Rect::new(cx - hw, cy + dy, (hw * 2) as u32, 1));
    }

    // Glass highlight + rim.
    canvas.set_draw_color(pal::SPHERE_GLASS);
    fill_disc(canvas, cx, cy, r);
    canvas.set_draw_color(pal::SPHERE_RIM);
    draw_circle_outline(canvas, cx, cy, r);
}

/// Half-width of a circle (radius r) at vertical offset dy from centre.
/// Returns 0 for rows outside the disc. Used to fill discs scanline-by-scanline.
fn circle_half_width(r: i32, dy: i32) -> i32 {
    if dy.abs() >= r {
        return 0;
    }
    // hw = floor(sqrt(r^2 - dy^2))
    let rr = (r as i64) * (r as i64);
    let dd = (dy as i64) * (dy as i64);
    let v = ((rr - dd).max(0) as f64).sqrt().floor() as i64;
    v as i32
}

/// Fill a solid disc (1-pixel-resolution) by scanlines.
fn fill_disc(canvas: &mut Canvas<Window>, cx: i32, cy: i32, r: i32) {
    for dy in -r..=r {
        let hw = circle_half_width(r, dy);
        if hw > 0 {
            let _ = canvas.fill_rect(Rect::new(cx - hw, cy + dy, (hw * 2) as u32, 1));
        }
    }
}

/// Draw a 1px circle outline (approximation via the mid-point scanline ends).
fn draw_circle_outline(canvas: &mut Canvas<Window>, cx: i32, cy: i32, r: i32) {
    for dy in -r..=r {
        let hw = circle_half_width(r, dy);
        if hw > 0 {
            let _ = canvas.draw_point((cx - hw, cy + dy));
            let _ = canvas.draw_point((cx + hw - 1, cy + dy));
        }
    }
}

/// The central skill slot (empty for now; shows the frame outline).
fn draw_skill_slot(canvas: &mut Canvas<Window>) {
    let x = SKILL_SLOT_X;
    let y = SKILL_SLOT_Y;
    let s = SKILL_SLOT;
    // Recessed look: dark fill, light top-left bevel, dark bottom-right.
    canvas.set_draw_color(pal::SLOT_FILL);
    let _ = canvas.fill_rect(Rect::new(x, y, s as u32, s as u32));
    canvas.set_draw_color(pal::SLOT_EDGE);
    let _ = canvas.draw_rect(Rect::new(x, y, s as u32, s as u32));
    canvas.set_draw_color(pal::PANEL_DARK);
    let _ = canvas.draw_rect(Rect::new(x + 1, y + 1, (s - 2) as u32, (s - 2) as u32));
}

/// Draw the 8-slot belt row, centred horizontally below the skill area.
fn draw_belt(canvas: &mut Canvas<Window>) {
    let total_w = BELT_TOTAL_W;
    let start_x = (SCREEN_W - total_w) / 2;
    for i in 0..BELT_SLOTS {
        let x = start_x + (i as i32) * BELT_SLOT_W;
        // Recessed empty slot.
        canvas.set_draw_color(pal::SLOT_FILL);
        let _ = canvas.fill_rect(Rect::new(x, BELT_Y, BELT_SLOT_W as u32, BELT_SLOT_H as u32));
        canvas.set_draw_color(pal::SLOT_EDGE);
        let _ = canvas.draw_rect(Rect::new(x, BELT_Y, BELT_SLOT_W as u32, BELT_SLOT_H as u32));
    }
}

/// Draw the experience bar above the panel, filled to the level's progress.
fn draw_xp_bar(canvas: &mut Canvas<Window>, level: u8, experience: u32) {
    // Background.
    canvas.set_draw_color(pal::XP_BACK);
    let _ = canvas.fill_rect(Rect::new(XP_BAR_X, XP_BAR_Y, XP_BAR_W as u32, XP_BAR_H as u32));
    // Fill.
    let ratio = xp_progress(level, experience);
    let fill_w = ((XP_BAR_W as f32 * ratio).round() as i32).clamp(0, XP_BAR_W);
    if fill_w > 0 {
        canvas.set_draw_color(pal::XP_FILL);
        let _ = canvas.fill_rect(Rect::new(XP_BAR_X, XP_BAR_Y, fill_w as u32, XP_BAR_H as u32));
    }
    // Edge.
    canvas.set_draw_color(pal::XP_EDGE);
    let _ = canvas.draw_rect(Rect::new(XP_BAR_X, XP_BAR_Y, XP_BAR_W as u32, XP_BAR_H as u32));
}

/// Render the stat text: Level, HP, Mana, Gold.
///
/// `hp`/`max_hp`/`mana`/`max_mana` are 64x fixed-point (raw player fields); we
/// shift back to display units here.
fn draw_stats_text(
    canvas: &mut Canvas<Window>,
    font: &PixelFont,
    hp: i32,
    max_hp: i32,
    mana: i32,
    max_mana: i32,
    level: u8,
    gold: i32,
) {
    // ---- Life sphere label (below the globe) ----
    let hp_txt = format!("{}/{}", hp >> 6, max_hp >> 6);
    let tw = font.text_width(&hp_txt);
    font.render_text_centered(canvas, &hp_txt, LIFE_CX, LIFE_CY + SPHERE_R + 6, pal::TEXT);

    // ---- Mana sphere label ----
    let mana_txt = format!("{}/{}", mana >> 6, max_mana >> 6);
    font.render_text_centered(canvas, &mana_txt, MANA_CX, MANA_CY + SPHERE_R + 6, pal::TEXT);

    // ---- Level + gold, centred over the skill/belt area ----
    let level_txt = format!("LV {}", level);
    font.render_text_centered(canvas, &level_txt, SCREEN_W / 2, PANEL_TOP + 14, pal::TEXT);

    let gold_txt = format!("GOLD {}", gold);
    let gw = font.text_width(&gold_txt);
    font.render_text(canvas, &gold_txt, SCREEN_W / 2 - gw / 2, PANEL_TOP + 30, pal::TEXT_DIM);

    // Suppress unused warning for `tw` (kept for symmetry / future centre calc).
    let _ = tw;
}

/// Linear-interpolate two colors. `t` in 0.0..=1.0.
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color::RGBA(lerp(a.r, b.r), lerp(a.g, b.g), lerp(a.b, b.b), 255)
}

// ---------------------------------------------------------------------------
// Tests (pure logic only; no SDL canvas needed)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_hud_palette_paints_panel_and_spheres() {
        let mut player = crate::game::player_exact::Player::new();
        player._p_max_hp = 100 * 64;
        player._p_hit_points = 50 * 64;
        player._p_max_mana = 100 * 64;
        player._p_mana = 100 * 64;
        let mut buf = vec![0u8; (SCREEN_W * SCREEN_H) as usize];
        let (w, h) = (SCREEN_W as usize, SCREEN_H as usize);
        {
            let mut surface = Surface::new(&mut buf, w as u32, w as i32, h as i32);
            draw_hud_palette(&mut surface, &player);
        }
        // Panel background painted in the bottom strip.
        assert_ne!(buf[400 * w + 10], 0, "panel background painted");
        // Life sphere area painted (centre-left, y = PANEL_TOP + 60).
        assert_ne!(buf[(PANEL_TOP + 60) as usize * w + LIFE_CX as usize], 0, "life sphere painted");
        // Mana sphere area painted.
        assert_ne!(buf[(PANEL_TOP + 60) as usize * w + MANA_CX as usize], 0, "mana sphere painted");
        // XP bar painted (above the panel).
        assert_ne!(buf[XP_BAR_Y as usize * w + (XP_BAR_X + 40) as usize], 0, "XP bar painted");
        // World area (upper screen) untouched.
        assert_eq!(buf[50 * w + 50], 0, "world area untouched");
    }

    #[test]
    fn test_xp_progress_level_1_start() {
        // Level 1 at 0 XP -> 0.0 (window is [0, 2000)).
        assert!((xp_progress(1, 0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_xp_progress_level_1_mid() {
        // Level 1 at 1000 XP -> halfway (1000/2000).
        let p = xp_progress(1, 1000);
        assert!((p - 0.5).abs() < 1e-3, "got {}", p);
    }

    #[test]
    fn test_xp_progress_level_2_window() {
        // Level 2 window is [2000, 4620). At 3310 (midpoint) -> ~0.5.
        let p = xp_progress(2, 3310);
        assert!((p - 0.5).abs() < 0.02, "got {}", p);
    }

    #[test]
    fn test_xp_progress_at_threshold_is_full() {
        // Exactly at the threshold to leave the level -> 1.0 (about to level).
        assert!((xp_progress(1, 2000) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_xp_progress_max_level_is_full() {
        assert!((xp_progress(MAX_PLAYER_LEVEL, u32::MAX) - 1.0).abs() < 1e-6);
        assert!((xp_progress(MAX_PLAYER_LEVEL, 0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_fill_ratio_basic() {
        // 64x: 6400/12800 -> 0.5.
        assert!((fill_ratio(6400, 12800) - 0.5).abs() < 1e-3);
    }

    #[test]
    fn test_fill_ratio_zero_max_is_zero() {
        assert_eq!(fill_ratio(0, 0), 0.0);
        assert_eq!(fill_ratio(6400, 0), 0.0);
    }

    #[test]
    fn test_fill_ratio_clamps_overfull() {
        // current > max clamps to 1.0.
        assert!((fill_ratio(12800, 6400) - 1.0).abs() < 1e-3);
    }

    #[test]
    fn test_fill_ratio_negative_hp_is_zero() {
        // Dead player (negative hp) -> 0.
        assert_eq!(fill_ratio(-640, 6400), 0.0);
    }

    #[test]
    fn test_circle_half_width() {
        // Centre row of a radius-29 disc is widest.
        assert!(circle_half_width(29, 0) == 29);
        // Just past the radius -> 0.
        assert_eq!(circle_half_width(29, 29), 0);
        assert_eq!(circle_half_width(29, 30), 0);
        // Symmetric.
        assert_eq!(circle_half_width(29, 5), circle_half_width(29, -5));
    }

    #[test]
    fn test_xp_threshold_table_size() {
        assert_eq!(XP_THRESHOLDS.len(), 50);
        // Monotonically increasing.
        for w in XP_THRESHOLDS.windows(2) {
            assert!(w[1] > w[0], "XP table not monotonic at {:?}", w);
        }
    }
}