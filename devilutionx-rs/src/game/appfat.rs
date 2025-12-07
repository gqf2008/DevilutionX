// appfat.rs - Fatal error handling and dialogs
// Ported from Source/appfat.cpp (97 lines)

use std::sync::atomic::{AtomicBool, Ordering};

/// Set to true when a fatal error is encountered and the application should shut down.
static TERMINATING: AtomicBool = AtomicBool::new(false);

/// Fatal error type
#[derive(Debug, Clone)]
pub struct FatalError {
    pub title: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

impl FatalError {
    pub fn new(title: &str, message: &str) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            file: None,
            line: None,
        }
    }

    pub fn with_location(mut self, file: &str, line: u32) -> Self {
        self.file = Some(file.to_string());
        self.line = Some(line);
        self
    }
}

impl std::fmt::Display for FatalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.title, self.message)?;
        if let (Some(file), Some(line)) = (&self.file, self.line) {
            write!(f, "\nThe error occurred at: {} line {}", file, line)?;
        }
        Ok(())
    }
}

impl std::error::Error for FatalError {}

/// Result type for fatal operations
pub type FatalResult<T> = Result<T, FatalError>;

/// Check if we are in terminating state
pub fn is_terminating() -> bool {
    TERMINATING.load(Ordering::SeqCst)
}

/// Set terminating state
fn set_terminating(value: bool) {
    TERMINATING.store(value, Ordering::SeqCst);
}

/// Clean up and prepare for exit
fn free_dlg(_is_multiplayer: bool) {
    if is_terminating() {
        // Another thread is already cleaning up
        std::thread::sleep(std::time::Duration::from_millis(20000));
    }

    set_terminating(true);

    // In multiplayer, would leave game and wait
    // if is_multiplayer {
    //     SNetLeaveGame();
    //     std::thread::sleep(std::time::Duration::from_millis(2000));
    // }
    // SNetDestroy();
}

/// Display a fatal error and exit the application
pub fn display_fatal_error_and_exit(title: &str, body: &str) -> ! {
    free_dlg(false);

    eprintln!("=== FATAL ERROR ===");
    eprintln!("Title: {}", title);
    eprintln!("Message: {}", body);
    eprintln!("==================");

    // In real implementation, would show UI dialog
    // UiErrorOkDialog(title, body);

    std::process::exit(1);
}

/// Terminate with a generic error message
pub fn app_fatal(message: &str) -> ! {
    display_fatal_error_and_exit("Error", message)
}

/// Assert failure handler (debug only)
#[cfg(debug_assertions)]
pub fn assert_fail(line_no: u32, file: &str, fail_msg: &str) -> ! {
    let message = format!("assertion failed ({}:{})\n{}", file, line_no, fail_msg);
    app_fatal(&message)
}

/// Display error dialog with location information
pub fn err_dlg(title: &str, error: &str, log_file_path: &str, log_line_nr: u32) -> ! {
    let body = format!(
        "{}\n\nThe error occurred at: {} line {}",
        error, log_file_path, log_line_nr
    );
    display_fatal_error_and_exit(title, &body)
}

/// Display insert CD error dialog
pub fn insert_cd_dlg(archive_name: &str) -> ! {
    let body = format!(
        "Unable to open main data archive ({}).\n\nMake sure that it is in the game folder.",
        archive_name
    );
    display_fatal_error_and_exit("Data File Error", &body)
}

/// Display read-only directory error dialog
pub fn dir_error_dlg(error: &str) -> ! {
    let body = format!("Unable to write to location:\n{}", error);
    display_fatal_error_and_exit("Read-Only Directory Error", &body)
}

/// SDL error macro equivalent
#[macro_export]
macro_rules! err_sdl {
    ($msg:expr) => {
        $crate::game::appfat::err_dlg("SDL Error", $msg, file!(), line!())
    };
}

/// Assert macro for debug builds
#[macro_export]
macro_rules! dvl_assert {
    ($expr:expr) => {
        if cfg!(debug_assertions) && !($expr) {
            $crate::game::appfat::assert_fail(line!(), file!(), stringify!($expr))
        }
    };
    ($expr:expr, $msg:expr) => {
        if cfg!(debug_assertions) && !($expr) {
            $crate::game::appfat::assert_fail(line!(), file!(), $msg)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fatal_error_new() {
        let error = FatalError::new("Test Title", "Test message");
        assert_eq!(error.title, "Test Title");
        assert_eq!(error.message, "Test message");
        assert!(error.file.is_none());
        assert!(error.line.is_none());
    }

    #[test]
    fn test_fatal_error_with_location() {
        let error = FatalError::new("Title", "Message")
            .with_location("test.rs", 42);

        assert_eq!(error.file, Some("test.rs".to_string()));
        assert_eq!(error.line, Some(42));
    }

    #[test]
    fn test_fatal_error_display() {
        let error = FatalError::new("Error", "Something went wrong")
            .with_location("main.rs", 100);

        let display = format!("{}", error);
        assert!(display.contains("Error"));
        assert!(display.contains("Something went wrong"));
        assert!(display.contains("main.rs"));
        assert!(display.contains("100"));
    }

    #[test]
    fn test_fatal_error_display_no_location() {
        let error = FatalError::new("Error", "Simple message");
        let display = format!("{}", error);

        assert_eq!(display, "Error: Simple message");
    }

    #[test]
    fn test_is_terminating_default() {
        // Note: This test might fail if other tests set TERMINATING
        // In a real test suite, you'd want test isolation
        // Reset for predictable testing
        set_terminating(false);
        assert!(!is_terminating());
    }

    #[test]
    fn test_set_terminating() {
        set_terminating(true);
        assert!(is_terminating());
        set_terminating(false);
        assert!(!is_terminating());
    }

    #[test]
    fn test_fatal_result_ok() {
        let result: FatalResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_fatal_result_err() {
        let error = FatalError::new("Test", "Error message");
        let result: FatalResult<i32> = Err(error);
        assert!(result.is_err());
    }

    // Note: We can't test app_fatal, display_fatal_error_and_exit, etc.
    // because they call std::process::exit() which would terminate the test runner
    // In a real implementation, you might use a different approach for testability
}
