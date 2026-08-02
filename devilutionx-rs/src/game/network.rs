// Network module - Stub implementations for network packet handling
// These are placeholders until full multiplayer support is implemented

/// Process network packets (stub)
/// 
/// Equivalent to C++: DvlNet_ProcessNetworkPackets()
/// 
/// In single-player mode, this does nothing.
/// In multiplayer, this would:
/// - Receive packets from other players
/// - Send player state updates
/// - Handle network commands
pub fn process_network_packets() {
    // TODO: Implement when adding multiplayer support
    // For now, single-player mode does nothing here
}

/// Process game message packets
/// 
/// Equivalent to C++: ProcessGameMessagePackets()
/// 
/// Drains the storm receive queue and applies each command through
/// `GameState::handle_command` (walk / attack / spell / doors). No-op when no
/// session is active.
pub fn process_game_message_packets(game_state: &mut crate::game::game_state::GameState) {
    while let Some((_sender, data)) = crate::net::storm::snet_receive_message() {
        if data.is_empty() {
            continue;
        }
        if let Some(cmd) = crate::game::msg::CmdId::from_u8(data[0]) {
            game_state.handle_command(cmd, &data);
        }
    }
}

/// Send the local player's command through the session (C++ `NetSendCmd*`).
/// Loopback queues it; `process_game_message_packets` re-applies it, so local
/// callers that already applied the effect must guard against double-apply.
pub fn send_command(cmd_data: &[u8]) -> bool {
    crate::net::storm::snet_send_message(0, cmd_data)
}

/// Pack + send a walk command (C++ `NetSendCmdLoc(CMD_WALKXY, pos)`).
pub fn send_walk(x: i8, y: i8) -> bool {
    let mut tx = crate::game::msg::MsgHandler::new(0, false);
    if !tx.send_walk(x, y) {
        return false;
    }
    tx.get_send_data().map_or(false, |d| send_command(&d))
}

/// Handle delta/synchronization (stub)
/// 
/// Equivalent to C++: multi_handle_delta()
/// 
/// In multiplayer, this ensures all players are synchronized.
/// Returns false if network timeout occurs.
/// 
/// # Returns
/// * `true` - Network OK (or single-player)
/// * `false` - Network timeout
pub fn handle_delta() -> bool {
    // Single-player always succeeds
    // TODO: Implement network sync when adding multiplayer
    true
}

/// Clear last sent player command (stub)
/// 
/// Equivalent to C++: ClearLastSentPlayerCmd()
/// 
/// In multiplayer, this clears the command buffer after processing.
pub fn clear_last_sent_cmd() {
    // TODO: Implement when adding multiplayer command system
}

/// Check if game is in multiplayer mode (an active storm session exists).
pub fn is_multiplayer() -> bool {
    crate::net::storm::snet_is_active()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_single_player_network() {
        // Single-player should always succeed
        assert!(handle_delta());
        
        // Should not crash
        process_network_packets();
        let mut gs = crate::game::game_state::GameState::new(
            crate::game::player_exact::Player::new(),
            false,
            42,
        );
        process_game_message_packets(&mut gs);
        clear_last_sent_cmd();
        
        // Should report single-player (no active session)
        assert!(!is_multiplayer());
    }

    #[test]
    fn test_loopback_walk_round_trip_applies_movement() {
        use crate::net::storm::{SELCONN_LOOPBACK, snet_create_game, snet_initialize, snet_leave_game};
        assert!(snet_initialize(SELCONN_LOOPBACK));
        snet_create_game("turn_test", "").expect("loopback host");
        assert!(is_multiplayer(), "active session reports multiplayer");

        let mut gs = crate::game::game_state::GameState::new(
            crate::game::player_exact::Player::new(),
            false,
            42,
        );
        gs.player.position.x = 40;
        gs.player.position.y = 40;
        gs.camera.tile_x = 40;
        gs.camera.tile_y = 40;

        // Local input -> send walk -> storm queue -> receive -> handle_command.
        assert!(send_walk(44, 43));
        process_game_message_packets(&mut gs);
        assert!(
            gs.player.position.x > 40 && gs.player.position.y > 40,
            "loopback walk round-trip moved the player"
        );

        snet_leave_game();
        assert!(!is_multiplayer());
    }
}
