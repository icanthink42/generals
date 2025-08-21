use generals::shared::{map::Cell, MapView, Terrain, game_state::GameState};
use parking_lot::RwLock;
use uuid::Uuid;

use crate::Server;

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub cells: RwLock<Vec<Cell>>,
}

impl Map {
        pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: RwLock::new(vec![Cell { terrain: Terrain::Default, troops: 0, owner_id: None }; width * height])
        }
    }

    fn get_cell_id(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    #[allow(dead_code)]
    pub fn set_cell(&self, x: usize, y: usize, cell: Cell) {
        self.cells.write()[self.get_cell_id(x, y)] = cell;
    }

    pub fn get_visable_cells(&self, player: Uuid, server: &Server) -> Vec<usize> {
        // Check if player is alive
        if let Some(player_info) = server.players.read().get(&player) {
            if !*player_info.alive.read() {
                // Dead players can see everything
                return (0..self.width * self.height).collect();
            }
        }

        let mut visible = Vec::new();
        let cells = self.cells.read();
        let config = server.config.read();

        // Helper function to get cells within a radius
        let get_cells_in_radius = |center_x: usize, center_y: usize, radius: usize| {
            let mut cells = Vec::new();
            let min_x = center_x.saturating_sub(radius);
            let max_x = (center_x + radius + 1).min(self.width);
            let min_y = center_y.saturating_sub(radius);
            let max_y = (center_y + radius + 1).min(self.height);

            for y in min_y..max_y {
                for x in min_x..max_x {
                    if (x as i32 - center_x as i32).abs() + (y as i32 - center_y as i32).abs() <= radius as i32 {
                        cells.push(self.get_cell_id(x, y));
                    }
                }
            }
            cells
        };

        // Find all cells owned by the player and their visibility radius
        for (id, cell) in cells.iter().enumerate() {
            if cell.owner_id == Some(player) {
                let (x, y) = (id % self.width, id / self.width);
                let radius = match cell.terrain {
                    Terrain::City | Terrain::Capital => config.city_visibility_radius,
                    _ => config.tile_visibility_radius,
                };
                visible.extend(get_cells_in_radius(x, y, radius));
            }
        }

        // Remove duplicates
        visible.sort_unstable();
        visible.dedup();
        visible
    }

        pub fn to_map_view(&self, player: Uuid, server: &Server) -> MapView {
        let visible_cell_ids = self.get_visable_cells(player, server);
        let all_cells = self.cells.read();

        let mut cells = std::collections::HashMap::new();
        // Only copy visible cells
        for &id in &visible_cell_ids {
            cells.insert(id, all_cells[id].clone());
        }

        MapView { width: self.width, height: self.height, cells }
    }

    pub fn add_player_capital(&self, player: Uuid) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Choose a random position
        let x = rng.gen_range(0..self.width);
        let y = rng.gen_range(0..self.height);
        let cell_id = self.get_cell_id(x, y);

        // Set the cell as a capital with initial troops
        let mut cells = self.cells.write();
        cells[cell_id] = Cell {
            terrain: Terrain::Capital,
            troops: 1,  // Start with 1 troop
            owner_id: Some(player),
        };
    }

    pub fn remove_player(&self, player_id: Uuid) {
        let mut cells = self.cells.write();
        for cell in cells.iter_mut() {
            // If this cell belongs to the disconnected player
            if cell.owner_id == Some(player_id) {
                // Remove ownership
                cell.owner_id = None;
                // Convert capital to city
                if cell.terrain == Terrain::Capital {
                    cell.terrain = Terrain::City;
                }
            }
        }
    }

    pub fn tick_troops(&self) {
        let mut cells = self.cells.write();
        for cell in cells.iter_mut() {
            match cell.terrain {
                // Increment troops for owned capitals and cities
                Terrain::Capital | Terrain::City if cell.owner_id.is_some() => {
                    cell.troops += 1;
                },
                // Decrease troops in swamps (but not below 0)
                Terrain::Swamp if cell.troops > 0 => {
                    cell.troops -= 1;
                    // Remove ownership if troops hit 0
                    if cell.troops == 0 {
                        cell.owner_id = None;
                    }
                },
                _ => {
                    // For any other case, if troops are 0, remove ownership
                    if cell.troops == 0 {
                        cell.owner_id = None;
                    }
                }
            }
        }
    }

    pub fn tick_owned_tiles(&self) {
        let mut cells = self.cells.write();
        for cell in cells.iter_mut() {
            // Only increment non-capital, non-city tiles that are owned
            if cell.owner_id.is_some() &&
               cell.terrain != Terrain::Capital &&
               cell.terrain != Terrain::City {
                cell.troops += 1;
            }
        }
    }

        pub fn tile_battle(&self, attacking_id: usize, defending_id: usize, server: &Server) {
        let mut cells = self.cells.write();

        // Get the current state
        let attacking_troops = cells[attacking_id].troops;
        let attacking_owner = cells[attacking_id].owner_id;
        let defending_troops = cells[defending_id].troops;
        let defending_owner = cells[defending_id].owner_id;
        let defending_terrain = cells[defending_id].terrain;

        // Don't do anything if attacking tile has 1 or fewer troops
        if attacking_troops <= 1 {
            return;
        }

        // Cannot move onto mountains
        if defending_terrain == Terrain::Mountain {
            return;
        }

        // Calculate the battle outcome
        let moving_troops = attacking_troops - 1;  // Leave 1 troop behind

        // Update the cells based on battle outcome
        cells[attacking_id].troops = 1;  // Always leave 1 behind

        match (attacking_owner, defending_owner) {
            // If same owner, combine troops
            (Some(atk_owner), Some(def_owner)) if atk_owner == def_owner => {
                cells[defending_id].troops += moving_troops;
            }
            // If different owners or defending tile is unowned, battle
            _ => {
                if moving_troops > defending_troops {
                    // Attacker wins
                    cells[defending_id].troops = moving_troops - defending_troops;
                    cells[defending_id].owner_id = attacking_owner;

                    // If this was a capital capture, transfer all territory and convert to city
                    if cells[defending_id].terrain == Terrain::Capital {
                        if let Some(defeated_player) = defending_owner {
                            // Set the defeated player as not alive
                            if let Some(player) = server.players.read().get(&defeated_player) {
                                *player.alive.write() = false;
                            }

                            // Transfer all territory from the defeated player to the attacker
                            for cell in cells.iter_mut() {
                                if cell.owner_id == Some(defeated_player) {
                                    cell.owner_id = attacking_owner;
                                }
                            }
                            // Convert captured capital to a city
                            cells[defending_id].terrain = Terrain::City;

                            // Check if there's only one capital left
                            let remaining_capitals = cells.iter()
                                .filter(|cell| cell.terrain == Terrain::Capital)
                                .count();

                            if remaining_capitals == 1 {
                                server.set_game_state(GameState::GameOver);
                            }
                        }
                    }
                } else {
                    // Defender wins or ties
                    cells[defending_id].troops -= moving_troops;
                    // Remove ownership if troops hit 0
                    if cells[defending_id].troops == 0 {
                        cells[defending_id].owner_id = None;
                    }
                }
            }
        }
    }
}