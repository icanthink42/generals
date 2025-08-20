use crate::Server;
use generals::shared::{game_state::GameState, CBPacket};

impl Server {
    pub async fn tick(&self) {
        if *self.game_state.read() != GameState::InGame {
            return;
        }
        // Process battles first
        let players = self.players.read();
        for player in players.values() {
            // First process all battles
            {
                let paths = player.paths.read();
                for path in paths.values() {
                    if path.tile_ids.len() >= 2 {
                        let attacking_id = path.tile_ids[0] as usize;
                        let defending_id = path.tile_ids[1] as usize;

                        // Only do battle if attacking tile is owned by the player
                        let is_owner = {
                            let cells = self.map.cells.read();
                            cells[attacking_id].owner_id == Some(player.id())
                        };
                        if is_owner {
                            self.map.tile_battle(attacking_id, defending_id, self);
                        }
                    }
                }
            }

            // Then remove all first tiles
            let mut paths = player.paths.write();
            paths.retain(|_, path| {
                if path.tile_ids.len() >= 2 {
                    path.tile_ids.remove(0);
                    path.tile_ids.len() > 1
                } else {
                    false
                }
            });
        }

        // Update troops based on terrain
        self.map.tick_troops();

        // Send map updates to all players
        self.sync_map();

        // Send path tick to all players
        if let Ok(bytes) = bincode::serialize(&CBPacket::TickPaths) {
            for player in players.values() {
                player.send_bytes(bytes.clone());
            }
        }
    }
}