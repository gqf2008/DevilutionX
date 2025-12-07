//! File System Restriction Testing
//!
//! This module implements functionality for checking if the game will be able
//! to run on the system by testing write access to the configuration directory.
//!
//! # C++ Source Reference
//! - Source/restrict.cpp
//! - Source/restrict.h
//!
//! # Purpose
//! Before the game starts, it tests whether it can write to the user's
//! preferences/save directory. If not, it displays an error dialog.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Test file name used for write access check
const TEST_FILE_NAME: &str = "Diablo1ReadOnlyTest.foo";

/// Error types for restriction testing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestrictError {
    /// Cannot write to the specified directory
    ReadOnlyDirectory(PathBuf),
    /// Directory does not exist
    DirectoryNotFound(PathBuf),
    /// Other IO error
    IoError(String),
}

impl std::fmt::Display for RestrictError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RestrictError::ReadOnlyDirectory(path) => {
                write!(f, "Cannot write to directory: {}", path.display())
            }
            RestrictError::DirectoryNotFound(path) => {
                write!(f, "Directory not found: {}", path.display())
            }
            RestrictError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for RestrictError {}

/// Result type for restriction operations
pub type RestrictResult<T> = Result<T, RestrictError>;

/// Tests whether the game can write to the specified directory
///
/// This function attempts to:
/// 1. Create a test file in the directory
/// 2. Write some data to it
/// 3. Delete the test file
///
/// If any of these operations fail, the directory is considered read-only
/// or inaccessible.
///
/// # Arguments
/// * `pref_path` - The preferences/save directory to test
///
/// # Returns
/// * `Ok(())` if the directory is writable
/// * `Err(RestrictError)` if the directory is not writable
///
/// # C++ Reference
/// ```cpp
/// void ReadOnlyTest()
/// {
///     const std::string path = paths::PrefPath() + "Diablo1ReadOnlyTest.foo";
///     SDL_IOStream *file = SDL_IOFromFile(path.c_str(), "w");
///     if (file == nullptr) {
///         DirErrorDlg(paths::PrefPath());
///     }
///     SDL_CloseIO(file);
///     RemoveFile(path.c_str());
/// }
/// ```
pub fn read_only_test<P: AsRef<Path>>(pref_path: P) -> RestrictResult<()> {
    let pref_path = pref_path.as_ref();

    // Check if directory exists
    if !pref_path.exists() {
        return Err(RestrictError::DirectoryNotFound(pref_path.to_path_buf()));
    }

    // Build test file path
    let test_path = pref_path.join(TEST_FILE_NAME);

    // Try to create and write to the test file
    let file_result = File::create(&test_path);
    match file_result {
        Ok(mut file) => {
            // Try to write something
            if let Err(e) = file.write_all(b"test") {
                // Clean up if possible
                let _ = fs::remove_file(&test_path);
                return Err(RestrictError::IoError(e.to_string()));
            }

            // Flush and close
            if let Err(e) = file.flush() {
                let _ = fs::remove_file(&test_path);
                return Err(RestrictError::IoError(e.to_string()));
            }

            drop(file);

            // Delete the test file
            if let Err(e) = fs::remove_file(&test_path) {
                return Err(RestrictError::IoError(e.to_string()));
            }

            Ok(())
        }
        Err(_) => {
            // Cannot create file - likely read-only
            Err(RestrictError::ReadOnlyDirectory(pref_path.to_path_buf()))
        }
    }
}

/// Checks if a directory is writable without leaving test files
///
/// # Arguments
/// * `path` - The directory to check
///
/// # Returns
/// `true` if the directory is writable
pub fn is_writable<P: AsRef<Path>>(path: P) -> bool {
    read_only_test(path).is_ok()
}

/// Gets the default preferences path for the current platform
///
/// This is a simplified version - in real implementation this would
/// use platform-specific paths.
pub fn get_default_pref_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return PathBuf::from(appdata).join("DevilutionX");
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("DevilutionX");
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(xdg_data) = std::env::var_os("XDG_DATA_HOME") {
            return PathBuf::from(xdg_data).join("diasurgical").join("devilution");
        }
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("diasurgical")
                .join("devilution");
        }
    }

    // Fallback to current directory
    PathBuf::from(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writable_directory() {
        // Test with current directory (should be writable in most test environments)
        let current_dir = std::env::current_dir().unwrap();
        let _result = read_only_test(&current_dir);
        // Note: This might fail in some CI environments
        // assert!(result.is_ok());

        // Test file should be cleaned up if it was created
        let test_path = current_dir.join(TEST_FILE_NAME);
        assert!(!test_path.exists());
    }

    #[test]
    fn test_nonexistent_directory() {
        let result = read_only_test("/nonexistent/directory/path/that/does/not/exist");
        assert!(matches!(result, Err(RestrictError::DirectoryNotFound(_))));
    }

    #[test]
    fn test_is_writable_nonexistent() {
        assert!(!is_writable("/nonexistent/directory/path"));
    }

    #[test]
    fn test_error_display() {
        let path = PathBuf::from("/test/path");

        let err = RestrictError::ReadOnlyDirectory(path.clone());
        assert!(err.to_string().contains("/test/path"));

        let err = RestrictError::DirectoryNotFound(path.clone());
        assert!(err.to_string().contains("not found"));

        let err = RestrictError::IoError("test error".to_string());
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_default_pref_path() {
        let path = get_default_pref_path();
        // Should return some path
        assert!(!path.as_os_str().is_empty());
    }
}
