//! Connection-type selection dialog (C++ `Source/DiabloUI/multi/selconn.cpp`).
//!
//! `UiSelectProvider` shows the list of network providers (ZeroTier,
//! Client-Server (TCP), Offline) with their descriptions and max players.
//! The C++ `provider` global / `SNetInitializeProvider` wiring is the
//! follow-up (multiplayer game loop); this module ports the dialog model.

use crate::net::storm::{ConnType, connection_name};

/// C++ `MAX_PLRS` — maximum players supported by a provider.
pub const MAX_PLRS: usize = 4;

/// C++ `SelconnFocus` per-provider description text.
pub fn connection_description(conn: ConnType) -> &'static str {
    match conn {
        ConnType::ZeroTier => "All computers must be connected to the internet.",
        ConnType::Tcp => "All computers must be connected to a TCP-compatible network.",
        ConnType::Loopback => "Play by yourself with no network exposure.",
    }
}

/// C++ `SelconnFocus`: players supported (loopback is single-player).
pub fn players_supported(conn: ConnType) -> usize {
    match conn {
        ConnType::Loopback => 1,
        _ => MAX_PLRS,
    }
}

/// The connection-selection list (C++ `vecConnItems`): ZT, TCP, Loopback.
pub fn connection_options() -> Vec<ConnType> {
    vec![ConnType::ZeroTier, ConnType::Tcp, ConnType::Loopback]
}

/// `UiSelectProvider` dialog model (C++ selconn.cpp `vecConnItems` + focus).
pub struct SelConnMenu {
    items: Vec<ConnType>,
    selected: usize,
}

impl SelConnMenu {
    pub fn new() -> Self {
        Self {
            items: connection_options(),
            selected: 0,
        }
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn set_selection(&mut self, idx: usize) {
        if idx < self.items.len() {
            self.selected = idx;
        }
    }

    pub fn move_selection(&mut self, delta: i32) {
        let n = self.items.len();
        self.selected = (self.selected as i32 + delta).rem_euclid(n as i32) as usize;
    }

    pub fn row_label(&self, idx: usize) -> String {
        connection_name(self.items[idx]).to_string()
    }

    pub fn selected_connection(&self) -> ConnType {
        self.items[self.selected]
    }

    /// C++ `selconn_Description` for the focused provider.
    pub fn description(&self) -> &'static str {
        connection_description(self.selected_connection())
    }

    /// C++ `selconn_MaxPlayers` ("Players Supported: {:d}").
    pub fn players_supported(&self) -> usize {
        players_supported(self.selected_connection())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selconn_matches_cpp() {
        // C++ selconn.cpp list order: ZT, TCP, Loopback with ConnectionNames.
        assert_eq!(
            connection_options(),
            vec![ConnType::ZeroTier, ConnType::Tcp, ConnType::Loopback]
        );
        let mut menu = SelConnMenu::new();
        assert_eq!(menu.row_label(0), "ZeroTier");
        assert_eq!(menu.row_label(1), "Client-Server (TCP)");
        assert_eq!(menu.row_label(2), "Offline");

        // C++ SelconnFocus descriptions + players.
        menu.set_selection(2);
        assert_eq!(menu.description(), "Play by yourself with no network exposure.");
        assert_eq!(menu.players_supported(), 1);
        menu.set_selection(1);
        assert_eq!(menu.description(), "All computers must be connected to a TCP-compatible network.");
        assert_eq!(menu.players_supported(), MAX_PLRS);
        menu.set_selection(0);
        assert_eq!(menu.description(), "All computers must be connected to the internet.");
        assert_eq!(menu.players_supported(), MAX_PLRS);

        // Navigation wraps.
        menu.move_selection(-1);
        assert_eq!(menu.selected_connection(), ConnType::Loopback);
    }
}
