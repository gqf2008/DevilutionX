//! Storm Compatibility Layer
//!
//! C++ Source: Source/storm/
//!
//! Provides compatibility with Blizzard's Storm library API.
//! Storm was the internal library used by Blizzard for their games.
//!
//! This module provides:
//! - `storm_net`: Network API (SNet* functions for multiplayer)
//! - `storm_svid`: Video playback (SMK format via libsmacker)

pub mod storm_net;
pub mod storm_svid;
