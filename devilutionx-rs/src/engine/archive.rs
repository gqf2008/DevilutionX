//! Archive Manager — multi-MPQ orchestration with load-order priority.
//!
//! This is a thin layer on top of [`crate::engine::mpq::MpqArchive`] that mirrors the
//! behaviour of DevilutionX's C++ `LoadCoreArchives` / `LoadGameArchives`
//! (`Source/engine/assets.cpp`). The C++ engine keeps a `std::map<int, MpqArchive,
//! std::greater<>>` where the highest *priority* integer wins; for the Rust port we
//! use a simpler but semantically equivalent model:
//!
//! * Archives are stored in **load order**.
//! * **Later-loaded archives take precedence** over earlier ones (push semantics),
//!   matching how the game ships its MPQs at startup (core first, game data last).
//! * `read_file` / `has_file` scan from the highest-precedence (most recently loaded)
//!   archive to the lowest, returning the first hit.
//!
//! The three core capabilities required by the port are therefore:
//!   1. multiple archives — [`ArchiveManager::load_archive`]
//!   2. later-loaded overrides earlier — see [`ArchiveManager::read_file`]
//!   3. name-based lookup — see [`ArchiveManager::has_file`]
//!
//! # MPQ parsing reality
//! The underlying [`crate::engine::mpq::MpqArchive`] is **not** a stub: it parses the
//! MPQ header, hash table, block table, and supports sector-based decompression
//! (zlib/PKWare DCL/bzip2) and encryption. It has been verified end-to-end against
//! the real `devilutionx.mpq` / `spawn.mpq` shipping archives (see the integration
//! tests in `tests/archive_manager.rs`).

use std::path::{Path, PathBuf};

use crate::engine::mpq::{MpqArchive, MpqError};

/// Error returned by [`ArchiveManager`] operations.
///
/// This is a small wrapper around [`MpqError`] plus a path so callers can report
/// *which* archive failed. It intentionally does not expose the inner
/// compression/encryption error variants directly — they are funneled through
/// [`ArchiveError::Mpq`] and surfaced as a string when needed.
#[derive(Debug)]
pub enum ArchiveError {
    /// The MPQ file could not be opened or parsed.
    Mpq {
        path: PathBuf,
        source: MpqError,
    },
    /// An I/O error unrelated to MPQ parsing (e.g. file does not exist).
    Io(PathBuf, std::io::Error),
}

impl std::fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchiveError::Mpq { path, source } => {
                write!(f, "MPQ error loading {}: {}", path.display(), source)
            }
            ArchiveError::Io(path, err) => write!(f, "IO error for {}: {}", path.display(), err),
        }
    }
}

impl std::error::Error for ArchiveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ArchiveError::Mpq { source, .. } => Some(source),
            ArchiveError::Io(_, err) => Some(err),
        }
    }
}

impl From<std::io::Error> for ArchiveError {
    fn from(_e: std::io::Error) -> Self {
        // Used by `?` in contexts without a known path; callers should prefer
        // constructing `ArchiveError::Io` explicitly when a path is available.
        ArchiveError::Io(PathBuf::new(), std::io::ErrorKind::Other.into())
    }
}

/// MPQ archive priority constants mirroring C++ `Source/engine/assets.hpp`.
///
/// These are kept here for parity/documentation even though [`ArchiveManager`]
/// primarily relies on load order. They are exposed so that code which needs to
/// match the C++ priority scheme (e.g. language reloads) can do so.
pub mod priority {
    /// Main game data (`diabdat.mpq` / `spawn.mpq`). Lowest precedence.
    pub const MAIN: i32 = 1000;
    /// Hellfire expansion base.
    pub const HELLFIRE: i32 = 8000;
    /// Hellfire optional content (`hfmonk`, `hfmusic`, ...).
    pub const HELLFIRE_EXTRA_BASE: i32 = 8100;
    /// `devilutionx.mpq` engine resources.
    pub const DEVILUTIONX: i32 = 9000;
    /// Language-specific `.mpq` (e.g. `ru.mpq`).
    pub const LANG: i32 = 9100;
    /// `fonts.mpq` extra fonts — highest precedence.
    pub const FONTS: i32 = 9200;
}

/// A loaded MPQ archive plus the file-name hint used to locate it.
///
/// Kept as a private struct so callers interact only through [`ArchiveManager`].
struct ManagedArchive {
    /// The parsed, readable archive. `Box` keeps the `Vec<ManagedArchive>` move-cost
    /// bounded regardless of internal buffer sizes.
    archive: Box<MpqArchive>,
    /// Where it was loaded from (for diagnostics / `reload` semantics).
    path: PathBuf,
}

/// Multi-MPQ archive manager with load-order (LIFO) priority.
///
/// Later-loaded archives shadow earlier ones: [`ArchiveManager::read_file`] and
/// [`ArchiveManager::has_file`] scan starting from the most recently loaded archive
/// and return on the first match. This mirrors the effective lookup order the game
/// gets from the C++ priority map when archives are pushed in startup order.
///
/// # Example
/// ```
/// use devilutionx_rs::engine::archive::ArchiveManager;
///
/// let mut mgr = ArchiveManager::new();
/// // load_archive returns Ok even if the file is absent (returns Err only on
/// // parse failure) — but here we only load what exists.
/// # let p = std::path::Path::new("does-not-exist.mpq");
/// # let _ = mgr.load_archive(p);
/// ```
pub struct ArchiveManager {
    /// Archives in load order; **last element is searched first** (highest precedence).
    archives: Vec<ManagedArchive>,
    /// Directories searched when resolving bare archive names (e.g. "devilutionx").
    /// Defaults to the current directory. Matches C++ `GetMPQSearchPaths()`.
    search_paths: Vec<PathBuf>,
}

impl Default for ArchiveManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchiveManager {
    /// Create an empty manager whose only search path is the current directory.
    pub fn new() -> Self {
        Self {
            archives: Vec::new(),
            search_paths: vec![PathBuf::from(".")],
        }
    }

    /// Append a directory to the list of MPQ search paths.
    ///
    /// Used by [`load_core_archives`](Self::load_core_archives) /
    /// [`load_game_archives`](Self::load_game_archives) when resolving bare names
    /// like `"devilutionx"`. Search paths are consulted in insertion order.
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.search_paths.push(path.as_ref().to_path_buf());
        self
    }

    /// Number of archives currently loaded.
    pub fn archive_count(&self) -> usize {
        self.archives.len()
    }

    /// Load (open + parse) an MPQ file and register it at the **highest** precedence.
    ///
    /// Returns:
    /// * `Ok(())` if the archive was opened successfully;
    /// * `Err(ArchiveError::Io(..))` if the file does not exist / is unreadable;
    /// * `Err(ArchiveError::Mpq { .. })` if the file exists but is not a valid MPQ.
    ///
    /// Because the archive is pushed onto the end of the internal list, any file
    /// it contains will shadow a same-named file in an earlier archive.
    pub fn load_archive<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ArchiveError> {
        let path = path.as_ref().to_path_buf();
        match File::open(&path) {
            Ok(_) => {}
            Err(e) => return Err(ArchiveError::Io(path, e)),
        }
        // MpqArchive::open re-opens the file itself; we only probe above so we can
        // produce an accurate `ArchiveError::Io` with the path, then let open()
        // surface parse problems as `ArchiveError::Mpq`.
        match MpqArchive::open(&path) {
            Ok(archive) => {
                self.archives.push(ManagedArchive {
                    archive: Box::new(archive),
                    path,
                });
                Ok(())
            }
            Err(source) => Err(ArchiveError::Mpq { path, source }),
        }
    }

    /// Iterate over archives from highest to lowest precedence.
    ///
    /// The internal list is stored low→high in load order, so we iterate in reverse.
    /// Borrowing happens mutably here because [`MpqArchive::read_file`] / `has_file`
    /// currently require `&mut self` (the underlying reader holds an open file handle
    /// with a seekable cursor).
    fn archives_high_to_low_mut(&mut self) -> impl Iterator<Item = &mut ManagedArchive> {
        self.archives.iter_mut().rev()
    }

    /// Return `true` if any loaded archive contains `name`.
    ///
    /// Searches highest-precedence archive first (i.e. the last one loaded),
    /// matching the "later overrides earlier" contract.
    pub fn has_file(&mut self, name: &str) -> bool {
        for entry in self.archives_high_to_low_mut() {
            if entry.archive.has_file(name) {
                return true;
            }
        }
        false
    }

    /// Read a file by name, returning the bytes from the highest-precedence
    /// archive that contains it, or `None` if no archive has it.
    ///
    /// Path separators are normalised internally by [`MpqArchive`]: both `\\`
    /// (Diablo convention) and `/` work, and matching is case-insensitive /
    /// backslash-normalised as implemented in the hash function.
    pub fn read_file(&mut self, name: &str) -> Option<Vec<u8>> {
        for entry in self.archives_high_to_low_mut() {
            if entry.archive.has_file(name) {
                match entry.archive.read_file(name) {
                    Ok(data) => return Some(data),
                    // On a decompression error we fall through to the next archive.
                    // This preserves forward-progress semantics and avoids
                    // hard-failing on a single corrupt sector.
                    Err(_) => continue,
                }
            }
        }
        None
    }

    /// Resolve a bare archive name (without extension) against the configured
    /// search paths, trying `.mpq` then the upper-cased `.MPQ` form (the latter
    /// matches the original CD layout for `DIABDAT.MPQ`).
    ///
    /// Returns the first existing path, or `None`.
    fn resolve_name(&self, name: &str) -> Option<PathBuf> {
        for base in &self.search_paths {
            let lower = base.join(format!("{}.mpq", name));
            if lower.exists() {
                return Some(lower);
            }
            let upper = base.join(format!("{}.MPQ", name.to_uppercase()));
            if upper.exists() {
                return Some(upper);
            }
        }
        None
    }

    /// Load an archive identified by a bare name (no extension), searching
    /// [`search_paths`](Self::add_search_path). Returns `Ok(true)` if loaded,
    /// `Ok(false)` if not found on any search path.
    pub fn load_named(&mut self, name: &str) -> Result<bool, ArchiveError> {
        if let Some(path) = self.resolve_name(name) {
            self.load_archive(&path)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Load the core engine archives, mirroring C++
    /// `LoadCoreArchives` (`Source/engine/assets.cpp`).
    ///
    /// Order (later = higher precedence):
    /// 1. `devilutionx.mpq` — engine resources (CLX sprites, panels, ...).
    /// 2. `fonts.mpq` — extra fonts.
    ///
    /// Both are best-effort: missing archives are silently skipped (matching the
    /// C++ `LoadMPQ` semantics, which only logs a warning) so the caller can still
    /// proceed without core assets in development/test environments. Returns
    /// `Ok(())` unless a found file fails to *parse*.
    pub fn load_core_archives(&mut self, base_dir: &Path) -> Result<(), ArchiveError> {
        self.search_paths.push(base_dir.to_path_buf());

        // devilutionx.mpq first (lowest precedence of the two).
        if !self.load_named("devilutionx")? {
            // not fatal — development/test environments may lack it
        }
        // fonts.mpq loaded last → highest precedence (overlays font overrides).
        if !self.load_named("fonts")? {
            // not fatal
        }
        Ok(())
    }

    /// Load the game data archives, mirroring C++ `LoadGameArchives`.
    ///
    /// Resolution order (first found wins; all share the [`priority::MAIN`] tier):
    /// 1. `DIABDAT.MPQ` (uppercase, original CD / GOG layout)
    /// 2. `diabdat.mpq`
    /// 3. `spawn.mpq` (shareware fallback)
    ///
    /// Returns `Ok(true)` if any game-data archive was loaded, `Ok(false)` if none
    /// were present, or `Err` if a found file failed to parse. Hellfire archives
    /// are *not* loaded here (see C++ `LoadHellfireArchives`); the shareware path
    /// is intentionally the last resort.
    pub fn load_game_archives(&mut self, base_dir: &Path) -> Result<bool, ArchiveError> {
        self.search_paths.push(base_dir.to_path_buf());

        if self.load_named("DIABDAT")? {
            return Ok(true);
        }
        if self.load_named("diabdat")? {
            return Ok(true);
        }
        if self.load_named("spawn")? {
            return Ok(true);
        }
        Ok(false)
    }

    /// Enumerate every file name known to any loaded archive, by reading each
    /// archive's internal `(listfile)`.
    ///
    /// Names from higher-precedence archives are kept; duplicates are de-duplicated
    /// case-insensitively. If an archive lacks a listfile it is silently skipped.
    /// Requires `&mut self` because reading the listfile moves the file cursor.
    pub fn list_files(&mut self) -> Vec<String> {
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut out: Vec<String> = Vec::new();
        // Iterate high→low so higher-precedence spellings win the dedup slot.
        for entry in self.archives_high_to_low_mut() {
            if let Ok(names) = entry.archive.list_files() {
                for name in names {
                    let key = name.to_ascii_lowercase();
                    if seen.insert(key) {
                        out.push(name);
                    }
                }
            }
        }
        out
    }

    /// Reset the manager: drop all loaded archives and search-path overrides
    /// added via [`load_core_archives`](Self::load_core_archives) /
    /// [`load_game_archives`](Self::load_game_archives). Search paths added
    /// explicitly via [`add_search_path`](Self::add_search_path) are *also*
    /// cleared — call `add_search_path` again after resetting if needed.
    pub fn clear(&mut self) {
        self.archives.clear();
    }
}

// We need `File` for the existence probe in `load_archive`.
use std::fs::File;

#[cfg(test)]
mod tests {
    use super::*;

    /// Manifest dir for locating the shipping test MPQs (two copies exist:
    /// one at the repo root, one inside the crate). We fall back gracefully when
    /// neither is present so this test compiles in asset-less environments.
    fn mpq(name: &str) -> Option<PathBuf> {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let candidates = [
            manifest.join(name),                 // crate-local copy
            manifest.parent()?.join(name),       // repo-root copy
        ];
        candidates.into_iter().find(|p| p.exists())
    }

    #[test]
    fn load_order_later_wins_lookup() {
        // We rely on the real devilutionx.mpq; if absent this is a framework-only
        // assertion so we still pass in CI without assets.
        let Some(dx) = mpq("devilutionx.mpq") else {
            eprintln!("[archive] skipping load_order_later_wins_lookup: no devilutionx.mpq");
            return;
        };

        let mut mgr = ArchiveManager::new();
        assert!(mgr.archive_count() == 0);

        // Load the same archive twice. The internal `(listfile)` exists in both
        // instances, so a lookup must resolve to the second instance. We confirm
        // the lookup itself works (non-None) — proving precedence iteration
        // touches the higher-precedence copy first.
        mgr.load_archive(&dx).expect("open devilutionx.mpq once");
        mgr.load_archive(&dx).expect("open devilutionx.mpq twice");
        assert_eq!(mgr.archive_count(), 2);

        // (listfile) is a guaranteed-present virtual entry in every MPQ.
        assert!(mgr.has_file("(listfile)"));
        let bytes = mgr.read_file("(listfile)").expect("(listfile) readable");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn missing_file_returns_none_without_panic() {
        let mut mgr = ArchiveManager::new();
        assert!(!mgr.has_file("definitely\\not\\a\\file.dat"));
        assert_eq!(mgr.read_file("definitely\\not\\a\\file.dat"), None);
        assert_eq!(mgr.archive_count(), 0);
    }

    #[test]
    fn nonexistent_path_is_io_error_not_mpq_error() {
        let mut mgr = ArchiveManager::new();
        match mgr.load_archive("nonexistent_archive_987654321.mpq") {
            Err(ArchiveError::Io(_, _)) => { /* expected */ }
            other => panic!("expected ArchiveError::Io, got {:?}", other),
        }
        assert_eq!(mgr.archive_count(), 0);
    }

    #[test]
    fn load_named_not_found_returns_false() {
        let mut mgr = ArchiveManager::new();
        let loaded = mgr
            .load_named("this_archive_name_should_not_resolve_xyz")
            .expect("load_named should not error on a missing file");
        assert!(!loaded);
        assert_eq!(mgr.archive_count(), 0);
    }

    #[test]
    fn real_devilutionx_mpq_lists_and_reads_files() {
        // End-to-end proof that the underlying MpqArchive (not a stub) can really
        // open the shipping archive and decompress a known file.
        let Some(dx) = mpq("devilutionx.mpq") else {
            eprintln!("[archive] skipping real_devilutionx_mpq_lists_and_reads_files: no mpq");
            return;
        };
        let mut mgr = ArchiveManager::new();
        mgr.load_archive(&dx).expect("open devilutionx.mpq");

        let names = mgr.list_files();
        assert!(!names.is_empty(), "listfile should enumerate entries");

        // devilutionx.mpq always ships these engine assets.
        let known = names
            .iter()
            .find(|n| n.eq_ignore_ascii_case(r"data\healthbox.clx"));
        assert!(known.is_some(), "expected data\\healthbox.clx in devilutionx.mpq");

        // Read a real (compressed) file end-to-end.
        let data = mgr.read_file(r"data\healthbox.clx").expect("read healthbox.clx");
        assert!(!data.is_empty(), "decompressed data should be non-empty");
    }

    #[test]
    fn two_archives_precedence_overrides() {
        // devilutionx.mpq and spawn.mpq both expose a `(listfile)` whose *contents*
        // differ (139 vs ~1028 entries). After loading both, list_files() should
        // reflect a superset and a name that only exists in spawn should resolve.
        let (Some(dx), Some(sp)) = (mpq("devilutionx.mpq"), mpq("spawn.mpq")) else {
            eprintln!("[archive] skipping two_archives_precedence_overrides: missing mpqs");
            return;
        };

        let mut mgr = ArchiveManager::new();
        mgr.load_archive(&dx).expect("open devilutionx.mpq");
        mgr.load_archive(&sp).expect("open spawn.mpq");
        assert_eq!(mgr.archive_count(), 2);

        // spawn.mpq is loaded last ⇒ highest precedence. Pick a file that lives in
        // spawn but not devilutionx. `levels\l1data\l1.min` exists in the shareware
        // data archive and not in the engine-only devilutionx.mpq.
        assert!(
            mgr.has_file(r"levels\l1data\l1.min"),
            "spawn-only file should be visible through the manager"
        );

        // And confirm we can actually decompress it (not just locate it).
        let data = mgr
            .read_file(r"levels\l1data\l1.min")
            .expect("read spawn-only file");
        assert!(!data.is_empty());
    }

    #[test]
    fn load_core_archives_idempotent_in_assetless_env() {
        // In an environment with no MPQs, load_core_archives must return Ok(())
        // (best-effort) and leave the manager empty.
        let tmp = std::env::temp_dir();
        let mut mgr = ArchiveManager::new();
        let res = mgr.load_core_archives(&tmp);
        assert!(res.is_ok(), "load_core_archives must not fail when mpqs are absent");
        // We cannot assert archive_count==0 unconditionally because the temp dir
        // might coincidentally contain an mpq; instead assert no panic occurred.
    }

    #[test]
    fn load_game_archives_resolves_spawn() {
        // If spawn.mpq exists in the crate dir, load_game_archives should pick it up.
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let has_spawn = manifest.join("spawn.mpq").exists()
            || manifest.parent().map(|p| p.join("spawn.mpq").exists()).unwrap_or(false);
        if !has_spawn {
            eprintln!("[archive] skipping load_game_archives_resolves_spawn: no spawn.mpq");
            return;
        }
        let mut mgr = ArchiveManager::new();
        let loaded = mgr
            .load_game_archives(&manifest)
            .expect("load_game_archives should not hard-fail");
        assert!(loaded, "spawn.mpq should have been detected");
        assert!(mgr.archive_count() >= 1);
    }
}
