//! Platform-Specific Code
//!
//! C++ Source: Source/platform/
//!
//! Platform abstraction layer for cross-platform support.

pub mod locale;
pub mod sdl;  // SDL2 封装

#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_os = "horizon")]
pub mod ctr;

#[cfg(target_os = "switch")]
pub mod switch;

#[cfg(target_os = "vita")]
pub mod vita;
