//! DevilutionX-RS Library
//!
//! A Rust port of DevilutionX - the Diablo 1 engine.
//!
//! This crate provides:
//! - C FFI bindings to C++ core (`ffi::*`)
//! - MPQ archive reading (`engine::mpq`)
//! - CLX sprite rendering (`engine::clx`)
//! - Game data structures (`game::*`)
//! - Controls and input handling (`controls::*`)
//! - Game panels and HUD (`panels::*`)
//! - Utility functions (`utils::*`)
//! - And more...

pub mod controls;
pub mod data;
pub mod engine;
pub mod ffi;
pub mod game;
pub mod levels;
pub mod net;
pub mod panels;
pub mod ui;
pub mod utils;
