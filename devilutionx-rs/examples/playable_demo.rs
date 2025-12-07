//! Playable Demo - M80 Milestone
//!
//! Interactive demo showing:
//! - WASD/Arrow key movement
//! - Mouse tracking
//! - Real-time rendering
//! - FPS display
//! - Delta time handling
//!
//! Controls:
//! - WASD or Arrow keys: Move player
//! - Mouse: Track cursor
//! - ESC: Quit

use devilutionx_rs::game::playable_demo::PlayableDemo;
use anyhow::Result;

fn main() -> Result<()> {
    println!("=== DevilutionX-RS Playable Demo (M80) ===");
    println!();
    println!("Controls:");
    println!("  WASD / Arrow Keys - Move player");
    println!("  Mouse - Track cursor");
    println!("  ESC - Quit");
    println!();
    println!("Starting demo...");

    let mut demo = PlayableDemo::new("DevilutionX-RS - Playable Demo", 640, 480)?;
    demo.run()?;

    println!("Demo closed.");
    Ok(())
}
