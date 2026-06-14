/// Network client
use anyhow::Result;
use super::protocol::GameMessage;

pub struct GameClient {
    // TODO: Client state
}

impl GameClient {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn connect(&mut self, _addr: &str) -> Result<()> {
        // TODO: Connect to server
        Ok(())
    }

    pub async fn send(&mut self, _msg: GameMessage) -> Result<()> {
        // TODO: Send message
        Ok(())
    }
}

impl Default for GameClient {
    fn default() -> Self {
        Self::new()
    }
}
