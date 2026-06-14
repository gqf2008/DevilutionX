//! Demo (`.dmo`) file parser — Tier 0 of the timedemo end-to-end harness.
//!
//! Binary format ported from `Source/engine/demomode.cpp`:
//! - `OpenDemoFile()` / `ReadSettings()` — file header
//! - `ReadDemoMessage()` — event stream
//!
//! Supports demo file **version 3** (the current/fixed version used by the
//! `WarriorLevel1to2` fixture). Versions `< 2` used a different (u32) event
//! encoding and are rejected; version 2/3 share the compact u8 encoding.
//!
//! The companion save file (`spawn_0.sv`) is an MPQ archive (magic `MPQ\x1a`).
//! [`load_save_archive`] opens it via the `engine::mpq` reader and exposes the
//! inner entries — that is the Tier 1 save-loading step. Full headless replay
//! (stepping the engine game loop over the demo stream) is Tier 2+ work.

use std::collections::HashMap;

use crate::engine::mpq::{MpqArchive, MpqError};

// ────────────────────────────────────────────────────────────────────────────
// Event types
// ────────────────────────────────────────────────────────────────────────────

/// Raw event type tag as stored in the demo stream (`DemoMsg::EventType` in C++).
///
/// Values mirror `Source/engine/demomode.cpp:78-94`:
/// `GameTick=0, Rendering=1, Quit=8, MouseMotion=9, MouseDown=10, MouseUp=11,
///  MouseWheel=12, KeyDown=13, KeyUp=14`, custom events `>= 64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DemoEventType {
    GameTick,
    Rendering,
    Quit,
    MouseMotion,
    MouseButtonDown,
    MouseButtonUp,
    MouseWheel,
    KeyDown,
    KeyUp,
    /// Interface-mode / custom event (`type >= MinCustomEvent`). The raw tag is
    /// preserved; custom events carry no payload.
    Custom(u8),
}

/// Classify a raw event tag. Mirrors C++ `ReadDemoMessage`: known tags map to
/// their variant, `>= 64` become custom (payload-less), and anything else
/// (e.g. 2..=7, 15..=63) is an error — C++ `app_fatal`s on those.
fn classify_type(raw: u8) -> Result<DemoEventType, DemoParseError> {
    match raw {
        0 => Ok(DemoEventType::GameTick),
        1 => Ok(DemoEventType::Rendering),
        8 => Ok(DemoEventType::Quit),
        9 => Ok(DemoEventType::MouseMotion),
        10 => Ok(DemoEventType::MouseButtonDown),
        11 => Ok(DemoEventType::MouseButtonUp),
        12 => Ok(DemoEventType::MouseWheel),
        13 => Ok(DemoEventType::KeyDown),
        14 => Ok(DemoEventType::KeyUp),
        64..=u8::MAX => Ok(DemoEventType::Custom(raw)),
        other => Err(DemoParseError::UnknownEvent(other)),
    }
}

/// Event-specific payload (`DemoMsg` union in C++).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoPayload {
    /// GameTick / Rendering / Quit / custom — no data.
    None,
    MouseMotion { x: u16, y: u16 },
    MouseButton { button: u8, x: u16, y: u16, mod_state: u16 },
    MouseWheel { x: i16, y: i16, mod_state: u16 },
    Key { sym: u32, mod_state: u16 },
}

/// One decoded demo message.
#[derive(Debug, Clone, Copy)]
pub struct DemoEvent {
    pub event_type: DemoEventType,
    /// Sub-tick progress (C++ `progressToNextGameTick`).
    pub progress_to_next_game_tick: u8,
    pub payload: DemoPayload,
}

// ────────────────────────────────────────────────────────────────────────────
// Header / settings
// ────────────────────────────────────────────────────────────────────────────

/// Parsed demo header (everything before the event stream).
#[derive(Debug, Clone)]
pub struct DemoHeader {
    /// Demo file format version (currently 3).
    pub version: u8,
    /// Which save slot the demo replays (`gSaveNumber`).
    pub save_number: u32,
    pub graphics_width: u16,
    pub graphics_height: u16,
    /// Raw gameplay-affecting settings bytes (booleans + potion-pickup counts),
    /// parsed verbatim for version > 0. See `ReadSettings()` in C++ for layout.
    pub settings_bytes: Vec<u8>,
}

/// Fully parsed `.dmo` file.
#[derive(Debug, Clone)]
pub struct DemoFile {
    pub header: DemoHeader,
    pub events: Vec<DemoEvent>,
}

// ────────────────────────────────────────────────────────────────────────────
// Errors
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemoParseError {
    /// Demo file version newer than this parser supports (or `< 2`, unsupported).
    UnsupportedVersion(u8),
    /// Unexpected end of file mid-record.
    UnexpectedEof,
    /// Unknown event tag `< MinCustomEvent` (C++ treats this as fatal).
    UnknownEvent(u8),
}

impl std::fmt::Display for DemoParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedVersion(v) => write!(f, "unsupported demo version {v} (need 2..=3)"),
            Self::UnexpectedEof => write!(f, "unexpected EOF parsing demo"),
            Self::UnknownEvent(t) => write!(f, "unknown demo event tag {t}"),
        }
    }
}
impl std::error::Error for DemoParseError {}

// ────────────────────────────────────────────────────────────────────────────
// Parser
// ────────────────────────────────────────────────────────────────────────────

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
    fn eof(&self) -> bool {
        self.pos >= self.data.len()
    }
    fn u8(&mut self) -> Result<u8, DemoParseError> {
        let b = *self.data.get(self.pos).ok_or(DemoParseError::UnexpectedEof)?;
        self.pos += 1;
        Ok(b)
    }
    fn u16le(&mut self) -> Result<u16, DemoParseError> {
        let lo = self.u8()? as u16;
        let hi = self.u8()? as u16;
        Ok(lo | (hi << 8))
    }
    fn i16le(&mut self) -> Result<i16, DemoParseError> {
        Ok(self.u16le()? as i16)
    }
    fn u32le(&mut self) -> Result<u32, DemoParseError> {
        let lo = self.u16le()?;
        let hi = self.u16le()?;
        Ok((lo as u32) | ((hi as u32) << 16))
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], DemoParseError> {
        if self.pos + n > self.data.len() {
            return Err(DemoParseError::UnexpectedEof);
        }
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }
}

/// Parse an entire `.dmo` byte buffer into header + event list.
pub fn parse_demo(data: &[u8]) -> Result<DemoFile, DemoParseError> {
    let mut r = Cursor::new(data);

    // --- Header ---
    let version = r.u8()?;
    // Versions < 2 used u32 event tags + a different mapping; the only fixtures
    // in the repo are version 3, so we require 2..=3.
    if !(2..=3).contains(&version) {
        return Err(DemoParseError::UnsupportedVersion(version));
    }
    let save_number = r.u32le()?;
    let graphics_width = r.u16le()?;
    let graphics_height = r.u16le()?;
    // `ReadSettings`: for version > 0, 17 bool bytes + 6 potion-pickup bytes.
    let settings_bytes = if version > 0 {
        r.take(23)?.to_vec()
    } else {
        Vec::new()
    };
    let header = DemoHeader { version, save_number, graphics_width, graphics_height, settings_bytes };

    // --- Event stream ---
    let mut events = Vec::new();
    while !r.eof() {
        // First byte of a record is the type tag. High bit set = a Rendering
        // event with progress encoded inline in the low 7 bits (no further data).
        let type_num = r.u8()?;
        if type_num & 0x80 != 0 {
            events.push(DemoEvent {
                event_type: DemoEventType::Rendering,
                progress_to_next_game_tick: type_num & 0x7f,
                payload: DemoPayload::None,
            });
            continue;
        }
        let progress = r.u8()?;
        let event_type = classify_type(type_num)?;
        let payload = match event_type {
            DemoEventType::GameTick
            | DemoEventType::Rendering
            | DemoEventType::Quit
            | DemoEventType::Custom(_) => DemoPayload::None,
            DemoEventType::MouseMotion => DemoPayload::MouseMotion {
                x: r.u16le()?,
                y: r.u16le()?,
            },
            DemoEventType::MouseButtonDown | DemoEventType::MouseButtonUp => DemoPayload::MouseButton {
                button: r.u8()?,
                x: r.u16le()?,
                y: r.u16le()?,
                mod_state: r.u16le()?,
            },
            DemoEventType::MouseWheel => DemoPayload::MouseWheel {
                x: r.i16le()?,
                y: r.i16le()?,
                mod_state: r.u16le()?,
            },
            DemoEventType::KeyDown | DemoEventType::KeyUp => DemoPayload::Key {
                sym: r.u32le()?,
                mod_state: r.u16le()?,
            },
        };
        events.push(DemoEvent { event_type, progress_to_next_game_tick: progress, payload });
    }

    Ok(DemoFile { header, events })
}

// ────────────────────────────────────────────────────────────────────────────
// Save archive loading — Tier 1
// ────────────────────────────────────────────────────────────────────────────
//
// GOAL (this section): open a Diablo save file (an MPQ archive, e.g.
// `spawn_0.sv`) and enumerate / read its inner entries. This is the save-
// loading half of the timedemo harness. The C++ gold standard
// (`test/timedemo_test.cpp`) loads `spawn_0.sv` this way before replaying the
// demo and byte-comparing against `demo_0_reference_spawn_0.sv`.
//
// WHAT'S HERE: a thin wrapper over `engine::mpq::MpqArchive`. We do NOT parse
// the inner save structure (hero stats, level data, etc.) yet — only prove the
// archive opens and we can reach its contents. Parsing the inner save blob and
// feeding it into engine state is the next step toward Tier 2 (game-loop
// stepping).

/// MPQ-backed view of a Diablo save file.
///
/// Wraps [`MpqArchive`] so callers don't depend on mpq internals. Tier 1 only
/// needs to open the archive and enumerate/reach entries; the engine-side save
/// parser that consumes these bytes is Tier 2+ work.
pub struct SaveArchive {
    archive: MpqArchive,
}

/// Error opening a save archive. Carries the underlying [`MpqError`] so callers
/// can distinguish "not an MPQ" / IO failures from missing inner entries.
#[derive(Debug)]
pub struct SaveLoadError(pub MpqError);

impl std::fmt::Display for SaveLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "save archive load failed: {}", self.0)
    }
}
impl std::error::Error for SaveLoadError {}

impl From<MpqError> for SaveLoadError {
    fn from(e: MpqError) -> Self {
        Self(e)
    }
}

/// One inner entry of a save archive, reached by name lookup.
#[derive(Debug, Clone)]
pub struct SaveEntry {
    pub name: String,
    pub unpacked_size: u32,
}

impl SaveArchive {
    /// Open a save file by path. The file must be a valid MPQ archive
    /// (Diablo saves always are — magic `MPQ\x1a`).
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, SaveLoadError> {
        let archive = MpqArchive::open(path)?;
        Ok(Self { archive })
    }

    /// Number of block-table entries in the archive. This is an upper bound on
    /// the number of inner files; some slots may be empty/deleted. Useful as a
    /// "the archive opened and has structure" sanity check without needing a
    /// listfile (save archives typically ship without one).
    pub fn block_count(&self) -> usize {
        self.archive.get_block_info().len()
    }

    /// Does the archive contain an entry by this name?
    pub fn has_entry(&self, name: &str) -> bool {
        self.archive.has_file(name)
    }

    /// Read an inner entry by name, returning its decoded bytes.
    pub fn read_entry(&mut self, name: &str) -> Result<Vec<u8>, SaveLoadError> {
        let bytes = self.archive.read_file(name)?;
        Ok(bytes)
    }

    /// Enumerate inner entries via the MPQ `(listfile)` pseudo-entry.
    ///
    /// Caveat: most Diablo save archives do **not** ship a `(listfile)`, so this
    /// will commonly return `Err`. Callers that need the canonical entry names
    /// for a Diablo save (`hero`, level blobs, etc.) should probe known names
    /// with [`has_entry`](Self::has_entry) instead.
    pub fn list_entries(&mut self) -> Result<Vec<String>, SaveLoadError> {
        let names = self.archive.list_files()?;
        Ok(names)
    }

    /// Borrowed access to the underlying archive, for callers that need an API
    /// not yet surfaced here.
    pub fn inner(&self) -> &MpqArchive {
        &self.archive
    }
}

/// Convenience: open a save archive and report whether it has structure
/// (block table is non-empty). This is the Tier 1 acceptance predicate —
/// "the save file opened as an MPQ and we can see its entries."
pub fn load_save_archive<P: AsRef<std::path::Path>>(path: P) -> Result<SaveArchive, SaveLoadError> {
    let archive = SaveArchive::open(path)?;
    if archive.block_count() == 0 {
        // Not strictly an error, but every real Diablo save has entries.
        return Err(SaveLoadError(MpqError::InvalidHash));
    }
    Ok(archive)
}

// ────────────────────────────────────────────────────────────────────────────
// Headless replay driver — Tier 2+ scaffold
// ────────────────────────────────────────────────────────────────────────────
//
// GOAL: load `spawn_0.sv`, feed the demo event stream into the engine game loop
// headlessly, and (Tier 3) compare the final save state byte-for-byte against
// `demo_0_reference_spawn_0.sv`.
//
// CURRENT STATE: the demo parser is real (Tier 0). The engine integration
// (MPQ extraction of the save, headless `game::game_loop` stepping, state
// capture) is NOT wired — those systems are still being ported. The driver
// exposes the API the workflow will fill in so each porting unit has a
// concrete target.

/// Aggregate counts of what a replay processed.
#[derive(Debug, Clone, Default)]
pub struct ReplaySummary {
    pub events_processed: usize,
    pub by_type: HashMap<DemoEventType, usize>,
}

/// Headless timedemo driver.
pub struct ReplayDriver {
    demo: DemoFile,
    event_index: usize,
}

impl ReplayDriver {
    /// Build a driver from a parsed demo file.
    pub fn new(demo: DemoFile) -> Self {
        Self { demo, event_index: 0 }
    }

    /// Parse `.dmo` bytes and build a driver.
    pub fn from_bytes(data: &[u8]) -> Result<Self, DemoParseError> {
        Ok(Self::new(parse_demo(data)?))
    }

    pub fn header(&self) -> &DemoHeader {
        &self.demo.header
    }

    pub fn event_count(&self) -> usize {
        self.demo.events.len()
    }

    /// Next event to be processed, if any.
    pub fn peek(&self) -> Option<&DemoEvent> {
        self.demo.events.get(self.event_index)
    }

    /// Advance one demo event. Returns the event that was consumed.
    ///
    /// Tier 1 TODO: actually drive the engine — translate the event into an
    /// input/game-tick and step `game::game_loop` (headless). For now this only
    /// advances the cursor so the harness can validate event-stream handling.
    pub fn step(&mut self) -> Option<&DemoEvent> {
        let ev = self.demo.events.get(self.event_index)?;
        self.event_index += 1;
        Some(ev)
    }

    /// Run through every demo event, returning an aggregate summary.
    ///
    /// Tier 1 TODO: between events, run the engine logic ticks implied by
    /// `progress_to_next_game_tick`; after the last event, capture final state.
    pub fn run_to_completion(&mut self) -> ReplaySummary {
        let mut summary = ReplaySummary::default();
        while let Some(ev) = self.step() {
            summary.events_processed += 1;
            *summary.by_type.entry(ev.event_type).or_insert(0) += 1;
        }
        summary
    }

    /// Reset the cursor to the first event.
    pub fn reset(&mut self) {
        self.event_index = 0;
    }
}
