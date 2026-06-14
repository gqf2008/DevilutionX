/// Network protocol definitions
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMessage {
    PlayerMove { x: i32, y: i32 },
    PlayerAttack { target_id: u32 },
    Chat { message: String },
}

pub struct GameProtocol {
    // TODO: Protocol state
}

impl GameProtocol {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for GameProtocol {
    fn default() -> Self {
        Self::new()
    }
}
