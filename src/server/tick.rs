use axum::extract::ws::Message;
use futures_util::SinkExt;

use crate::Server;

impl Server {
    pub async fn tick(&self) {
        // Increment troops on all capitals
        self.map.increment_capital_troops();

        // Send map updates to all players
        self.sync_map();
    }
}