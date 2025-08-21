use crate::Server;
use generals::shared::{game_state::GameState, CBPacket};

impl Server {
    pub async fn tick(&self) {
        if *self.game_state.read() != GameState::InGame {
            return;
        }

        // Increment tick counter
        let tick_count = {
            let mut counter = self.tick_counter.write();
            *counter += 1;
            *counter
        };

        // Process battles first
        let players = self.players.read();
        for player in players.values() {
            // First process all battles
            {
                let mut paths = player.paths.write();
                for (path_id, path) in paths.iter_mut() {
                    if path.valid_until as usize + 1 < path.tile_ids.len() {
                        let attacking_id = path.tile_ids[path.valid_until as usize] as usize;
                        let defending_id = path.tile_ids[path.valid_until as usize + 1] as usize;

                                                // Only do battle if attacking tile is owned by the player
                        let is_owner = {
                            let cells = self.map.cells.read();
                            cells[attacking_id].owner_id == Some(player.id())
                        };

                        // Do battle if we own the tile
                        if is_owner {
                            self.map.tile_battle(attacking_id, defending_id, self);
                        }

                        // Always progress the path and send confirmation
                        path.valid_until += 1;

                        // Send movement confirmation to client
                        if let Ok(bytes) = bincode::serialize(&CBPacket::MovementConfirmed(
                            generals::shared::cb_packet::MovementConfirmed {
                                path_id: *path_id,
                                valid_until: path.valid_until,
                            }
                        )) {
                            player.send_bytes(bytes);
                        }
                    }
                }
            }
        }

        let config = self.config.read();

        // Check for city and capital growth
        if tick_count % config.city_growth_tick == 0 {
            self.map.tick_troops();
        }

        // Check for regular tile growth
        if tick_count % config.tile_growth_tick == 0 {
            self.map.tick_owned_tiles();
        }

        // Send map updates to all players
        self.sync_map();


    }
}