//! MPQ Archive System
//!
//! C++ Source: Source/mpq/
//! 
//! MPQ (Mo'PaQ) archive format support for reading game assets.
//!
//! This module provides MPQ archive reading/writing functionality.

// PKWare DCL decompression (used by MPQ)
pub mod explode;

// MPQ Reader - main implementation
pub mod mpq_reader;

// Re-export main types
pub use mpq_reader::*;

// Sub-modules for C++ alignment
pub mod mpq_common {
    //! MPQ Common Utilities
    //!
    //! C++ Source: Source/mpq/mpq_common.cpp
    pub use super::mpq_reader::{
        MpqHeader, MpqHashEntry, MpqBlockEntry, MpqError,
        hash_string, calculate_file_hash, decrypt_block,
    };
}

pub mod mpq_sdl_rwops {
    //! MPQ SDL RWops Integration
    //!
    //! C++ Source: Source/mpq/mpq_sdl_rwops.cpp
    //!
    //! Note: Rust doesn't use SDL_RWops directly, file streaming
    //! is handled natively through std::io.
    
    // TODO: If SDL integration is needed, implement here
}

pub mod mpq_writer {
    //! MPQ Archive Writer
    //!
    //! C++ Source: Source/mpq/mpq_writer.cpp
    //!
    //! TODO: Port from Source/mpq/mpq_writer.cpp
    //! - MPQ archive creation
    //! - File addition to MPQ archives
    //! - Hash table and block table generation
}
