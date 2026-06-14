// capture.rs - Screenshot capture functionality
// Ported from Source/capture.cpp (113 lines)

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};
use std::fs::File;
use std::io::Write;

/// Screenshot format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenshotFormat {
    PCX,
    PNG,
}

impl ScreenshotFormat {
    /// Get file extension for format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::PCX => ".pcx",
            Self::PNG => ".png",
        }
    }
}

impl Default for ScreenshotFormat {
    fn default() -> Self {
        Self::PNG
    }
}

/// Screenshot capture result
#[derive(Debug)]
pub enum CaptureError {
    FileOpenError(String),
    WriteError(String),
    EncodingError(String),
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileOpenError(msg) => write!(f, "Failed to open file: {}", msg),
            Self::WriteError(msg) => write!(f, "Failed to write: {}", msg),
            Self::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
        }
    }
}

impl std::error::Error for CaptureError {}

/// Screenshot capture configuration
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub format: ScreenshotFormat,
    pub output_path: PathBuf,
    pub flash_duration_ms: u32,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            format: ScreenshotFormat::default(),
            output_path: PathBuf::from("."),
            flash_duration_ms: 300,
        }
    }
}

/// Generate unique screenshot filename
pub fn generate_screenshot_filename(base_path: &Path, format: ScreenshotFormat) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);

    // Convert to local time components (simplified)
    let secs = now.as_secs();
    let days = secs / 86400;
    let year = 1970 + (days / 365); // Simplified
    let remaining_days = days % 365;
    let month = (remaining_days / 30) + 1;
    let day = (remaining_days % 30) + 1;

    let day_secs = secs % 86400;
    let hour = day_secs / 3600;
    let minute = (day_secs % 3600) / 60;
    let second = day_secs % 60;

    let base_filename = format!(
        "Screenshot from {:04}-{:02}-{:02}-{:02}-{:02}-{:02}",
        year, month, day, hour, minute, second
    );

    let ext = format.extension();
    let mut path = base_path.join(format!("{}{}", base_filename, ext));
    let mut i = 0;

    while path.exists() {
        i += 1;
        path = base_path.join(format!("{}-{}{}", base_filename, i, ext));
    }

    path
}

/// Surface data for screenshots
pub struct SurfaceData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub palette: Vec<(u8, u8, u8)>,
}

/// Screenshot capture manager
pub struct CaptureManager {
    config: CaptureConfig,
}

impl CaptureManager {
    pub fn new(config: CaptureConfig) -> Self {
        Self { config }
    }

    /// Capture screenshot from surface data
    pub fn capture(&self, surface: &SurfaceData) -> Result<PathBuf, CaptureError> {
        let start_time = Instant::now();

        let filename = generate_screenshot_filename(&self.config.output_path, self.config.format);

        let file = File::create(&filename)
            .map_err(|e| CaptureError::FileOpenError(e.to_string()))?;

        match self.config.format {
            ScreenshotFormat::PCX => self.write_pcx(file, surface)?,
            ScreenshotFormat::PNG => self.write_png(file, surface)?,
        }

        // Ensure minimum flash duration
        let elapsed = start_time.elapsed();
        let min_duration = Duration::from_millis(self.config.flash_duration_ms as u64);
        if elapsed < min_duration {
            std::thread::sleep(min_duration - elapsed);
        }

        Ok(filename)
    }

    /// Write PCX format (simplified)
    fn write_pcx(&self, mut file: File, surface: &SurfaceData) -> Result<(), CaptureError> {
        // PCX Header (128 bytes)
        let mut header = [0u8; 128];
        header[0] = 10;  // Manufacturer (ZSoft)
        header[1] = 5;   // Version
        header[2] = 1;   // Encoding (RLE)
        header[3] = 8;   // Bits per pixel

        // Image dimensions (little-endian)
        let xmax = (surface.width - 1) as u16;
        let ymax = (surface.height - 1) as u16;
        header[8..10].copy_from_slice(&xmax.to_le_bytes());
        header[10..12].copy_from_slice(&ymax.to_le_bytes());

        // DPI
        header[12..14].copy_from_slice(&72u16.to_le_bytes());
        header[14..16].copy_from_slice(&72u16.to_le_bytes());

        header[65] = 1;  // Color planes
        let bytes_per_line = surface.width as u16;
        header[66..68].copy_from_slice(&bytes_per_line.to_le_bytes());
        header[68..70].copy_from_slice(&1u16.to_le_bytes()); // Palette type (color)

        file.write_all(&header)
            .map_err(|e| CaptureError::WriteError(e.to_string()))?;

        // Write image data (simplified - no RLE for now)
        for y in 0..surface.height as usize {
            let row_start = y * surface.width as usize;
            let row_end = row_start + surface.width as usize;
            let row = &surface.pixels[row_start..row_end];

            file.write_all(row)
                .map_err(|e| CaptureError::WriteError(e.to_string()))?;
        }

        // Write VGA palette (768 bytes after 0x0C marker)
        file.write_all(&[0x0C])
            .map_err(|e| CaptureError::WriteError(e.to_string()))?;

        for &(r, g, b) in &surface.palette {
            file.write_all(&[r, g, b])
                .map_err(|e| CaptureError::WriteError(e.to_string()))?;
        }

        // Pad to 256 colors if needed
        for _ in surface.palette.len()..256 {
            file.write_all(&[0, 0, 0])
                .map_err(|e| CaptureError::WriteError(e.to_string()))?;
        }

        Ok(())
    }

    /// Write PNG format (placeholder - would need png crate)
    fn write_png(&self, _file: File, _surface: &SurfaceData) -> Result<(), CaptureError> {
        // In a real implementation, use the `png` crate
        Err(CaptureError::EncodingError("PNG encoding not implemented".to_string()))
    }

    /// Apply red flash effect to palette
    pub fn red_palette_effect(palette: &mut [(u8, u8, u8)]) {
        for color in palette.iter_mut() {
            color.1 = 0; // Green = 0
            color.2 = 0; // Blue = 0
        }
    }
}

impl Default for CaptureManager {
    fn default() -> Self {
        Self::new(CaptureConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_screenshot_format_extension() {
        assert_eq!(ScreenshotFormat::PCX.extension(), ".pcx");
        assert_eq!(ScreenshotFormat::PNG.extension(), ".png");
    }

    #[test]
    fn test_screenshot_format_default() {
        assert_eq!(ScreenshotFormat::default(), ScreenshotFormat::PNG);
    }

    #[test]
    fn test_capture_config_default() {
        let config = CaptureConfig::default();
        assert_eq!(config.format, ScreenshotFormat::PNG);
        assert_eq!(config.flash_duration_ms, 300);
    }

    #[test]
    fn test_generate_filename() {
        let base = PathBuf::from("/tmp");
        let path = generate_screenshot_filename(&base, ScreenshotFormat::PNG);

        // Platform-agnostic: path lives under `base` (avoids `/` vs `\` mismatch on Windows).
        assert!(path.starts_with(&base), "path should be under base dir: {path:?}");
        assert!(path.to_string_lossy().contains("Screenshot from "));
        assert!(path.to_string_lossy().ends_with(".png"));
    }

    #[test]
    fn test_generate_filename_pcx() {
        let base = PathBuf::from("/tmp");
        let path = generate_screenshot_filename(&base, ScreenshotFormat::PCX);

        assert!(path.to_string_lossy().ends_with(".pcx"));
    }

    #[test]
    fn test_capture_error_display() {
        let err = CaptureError::FileOpenError("permission denied".to_string());
        assert!(format!("{}", err).contains("permission denied"));

        let err = CaptureError::WriteError("disk full".to_string());
        assert!(format!("{}", err).contains("disk full"));

        let err = CaptureError::EncodingError("invalid data".to_string());
        assert!(format!("{}", err).contains("invalid data"));
    }

    #[test]
    fn test_capture_manager_new() {
        let config = CaptureConfig {
            format: ScreenshotFormat::PCX,
            output_path: PathBuf::from("/screenshots"),
            flash_duration_ms: 500,
        };

        let manager = CaptureManager::new(config.clone());
        assert_eq!(manager.config.format, ScreenshotFormat::PCX);
        assert_eq!(manager.config.flash_duration_ms, 500);
    }

    #[test]
    fn test_red_palette_effect() {
        let mut palette = vec![
            (255, 128, 64),
            (100, 200, 150),
            (50, 50, 50),
        ];

        CaptureManager::red_palette_effect(&mut palette);

        // Red should stay, green and blue become 0
        assert_eq!(palette[0], (255, 0, 0));
        assert_eq!(palette[1], (100, 0, 0));
        assert_eq!(palette[2], (50, 0, 0));
    }

    #[test]
    fn test_surface_data() {
        let surface = SurfaceData {
            width: 640,
            height: 480,
            pixels: vec![0; 640 * 480],
            palette: vec![(0, 0, 0); 256],
        };

        assert_eq!(surface.width, 640);
        assert_eq!(surface.height, 480);
        assert_eq!(surface.pixels.len(), 640 * 480);
        assert_eq!(surface.palette.len(), 256);
    }

    #[test]
    fn test_pcx_header_constants() {
        // Verify PCX header structure understanding
        let mut header = [0u8; 128];
        header[0] = 10;  // ZSoft
        header[1] = 5;   // Version 5
        header[2] = 1;   // RLE encoding
        header[3] = 8;   // 8 bits per pixel

        assert_eq!(header[0], 10);
        assert_eq!(header[3], 8);
    }
}
