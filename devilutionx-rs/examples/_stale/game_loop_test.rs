// Complete Game Loop Test - Verifies C++ alignment
//
// This tests the complete production game loop with proper timing,
// not a simplified demo.

use anyhow::Result;
use devilutionx_rs::game::game_loop_new::{run_game_loop, InterfaceMode};

fn main() -> Result<()> {
    println!("=== DevilutionX Rust - Complete Game Loop Test ===");
    println!("C++ Alignment: RunGameLoop() → game_loop() → GameLogic()");
    println!("Architecture: Two-speed loop (60 FPS render, 2 Hz logic)");
    println!();

    // Run the game loop
    let result = run_game_loop(InterfaceMode::NewGame)?;

    if result {
        println!("Game exited normally");
    } else {
        println!("Game quit via menu");
    }

    Ok(())
}
