//! Multiplayer game selection / creation dialog model
//! (C++ `Source/DiabloUI/multi/selgame.cpp`).
//!
//! Ports the `UiSelGameDialog` data model: the game actions
//! (Create / Create Public / Join) with their descriptions and the create-game
//! difficulty list (Normal / Nightmare / Hell) with the C++ descriptions.
//! The network game list / text-input join form are multiplayer-loop
//! follow-ups.

/// C++ `DIFF_NORMAL` / `DIFF_NIGHTMARE` / `DIFF_HELL` (game difficulty).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SelGameDifficulty {
    Normal = 0,
    Nightmare = 1,
    Hell = 2,
}

impl SelGameDifficulty {
    /// C++ difficulty display name.
    pub fn name(self) -> &'static str {
        match self {
            SelGameDifficulty::Normal => "Normal",
            SelGameDifficulty::Nightmare => "Nightmare",
            SelGameDifficulty::Hell => "Hell",
        }
    }

    /// C++ `selgame_Diff_Focus` description.
    pub fn description(self) -> &'static str {
        match self {
            SelGameDifficulty::Normal => "Normal Difficulty\nThis is where a starting character should begin the quest to defeat Diablo.",
            SelGameDifficulty::Nightmare => "Nightmare Difficulty\nThe denizens of the Labyrinth have been bolstered and will prove to be a greater challenge. This is recommended for experienced characters only.",
            SelGameDifficulty::Hell => "Hell Difficulty\nThe most powerful of the underworld's creatures lurk at the gateway into Hell. Only the most experienced characters should venture in this realm.",
        }
    }

    pub fn all() -> [SelGameDifficulty; 3] {
        [
            SelGameDifficulty::Normal,
            SelGameDifficulty::Nightmare,
            SelGameDifficulty::Hell,
        ]
    }
}

/// C++ `selgame_GameSelection_Select` action values (UiListItem m_value).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SelGameAction {
    CreateGame = 0,
    CreatePublicGame = 1,
    JoinGame = 2,
}

impl SelGameAction {
    /// C++ `selgame_GameSelection_Focus` description.
    pub fn description(self) -> &'static str {
        match self {
            SelGameAction::CreateGame => "Create a new game with a difficulty setting of your choice.",
            SelGameAction::CreatePublicGame => "Create a new public game that anyone can join with a difficulty setting of your choice.",
            SelGameAction::JoinGame => "Enter an IP or a hostname to join a game already in progress.",
        }
    }
}

/// Result of the create-game form (C++ `nDifficulty` + provider + action).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelGameForm {
    pub action: SelGameAction,
    pub difficulty: SelGameDifficulty,
}

/// Difficulty-selection menu (C++ `vecSelGameDlgItems` Normal/Nightmare/Hell).
pub struct SelGameMenu {
    items: Vec<SelGameDifficulty>,
    selected: usize,
}

impl SelGameMenu {
    pub fn new() -> Self {
        Self {
            items: SelGameDifficulty::all().to_vec(),
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
        self.items[idx].name().to_string()
    }

    pub fn selected_difficulty(&self) -> SelGameDifficulty {
        self.items[self.selected]
    }

    pub fn selected_description(&self) -> &'static str {
        self.selected_difficulty().description()
    }
}

/// Game-action selection menu (C++ `vecSelGameDlgItems`: Create Game /
/// Create Public Game / Join Game). Skipped for the loopback provider.
pub struct SelGameActionMenu {
    items: Vec<SelGameAction>,
    selected: usize,
}

impl SelGameActionMenu {
    pub fn new() -> Self {
        Self {
            items: vec![
                SelGameAction::CreateGame,
                SelGameAction::CreatePublicGame,
                SelGameAction::JoinGame,
            ],
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
        match self.items[idx] {
            SelGameAction::CreateGame => "Create Game".to_string(),
            SelGameAction::CreatePublicGame => "Create Public Game".to_string(),
            SelGameAction::JoinGame => "Join Game".to_string(),
        }
    }

    pub fn selected_action(&self) -> SelGameAction {
        self.items[self.selected]
    }

    pub fn selected_description(&self) -> &'static str {
        self.selected_action().description()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_names_and_order_match_cpp() {
        // C++ selgame.cpp:339-341 — Normal, Nightmare, Hell.
        assert_eq!(
            SelGameDifficulty::all(),
            [
                SelGameDifficulty::Normal,
                SelGameDifficulty::Nightmare,
                SelGameDifficulty::Hell,
            ]
        );
        assert_eq!(SelGameDifficulty::Normal.name(), "Normal");
        assert_eq!(SelGameDifficulty::Nightmare.name(), "Nightmare");
        assert_eq!(SelGameDifficulty::Hell.name(), "Hell");
        // C++ DIFF_* values.
        assert_eq!(SelGameDifficulty::Normal as u8, 0);
        assert_eq!(SelGameDifficulty::Nightmare as u8, 1);
        assert_eq!(SelGameDifficulty::Hell as u8, 2);
    }

    #[test]
    fn test_action_values_and_descriptions_match_cpp() {
        // C++ selgame.cpp:153-156 action list values + :219-230 descriptions.
        assert_eq!(SelGameAction::CreateGame as u8, 0);
        assert_eq!(SelGameAction::CreatePublicGame as u8, 1);
        assert_eq!(SelGameAction::JoinGame as u8, 2);
        assert!(SelGameAction::CreateGame.description().starts_with("Create a new game"));
        assert!(SelGameAction::JoinGame.description().contains("IP or a hostname"));
    }

    #[test]
    fn test_difficulty_menu_navigation() {
        let mut menu = SelGameMenu::new();
        assert_eq!(menu.item_count(), 3);
        assert_eq!(menu.row_label(0), "Normal");
        assert_eq!(menu.selected_difficulty(), SelGameDifficulty::Normal);
        menu.move_selection(1);
        assert_eq!(menu.selected_difficulty(), SelGameDifficulty::Nightmare);
        assert!(menu.selected_description().contains("Nightmare"));
        menu.move_selection(1);
        assert_eq!(menu.selected_difficulty(), SelGameDifficulty::Hell);
        // Wrap.
        menu.move_selection(1);
        assert_eq!(menu.selected_difficulty(), SelGameDifficulty::Normal);
    }


    #[test]
    fn test_action_menu_order_and_descriptions() {
        let mut menu = SelGameActionMenu::new();
        assert_eq!(menu.item_count(), 3);
        assert_eq!(menu.row_label(0), "Create Game");
        assert_eq!(menu.row_label(1), "Create Public Game");
        assert_eq!(menu.row_label(2), "Join Game");
        menu.set_selection(2);
        assert_eq!(menu.selected_action(), SelGameAction::JoinGame);
        assert!(menu.selected_description().contains("IP or a hostname"));
    }

}
