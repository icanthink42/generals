use generals::shared::{map::Cell as SharedCell, MapView, Terrain, game_state::GameState};

#[derive(Debug, Clone)]
pub struct Cell {
    pub terrain: Terrain,
    pub troops: u32,
    pub owner_id: Option<Uuid>,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            terrain: Terrain::Default,
            troops: 0,
            owner_id: None,
        }
    }
}

impl Cell {
    pub fn to_view(&self, in_vision: bool, terrain_visible: bool) -> Option<SharedCell> {
        if !in_vision && !terrain_visible {
            return None;
        }

        Some(SharedCell {
            terrain: self.terrain,
            troops: if in_vision { self.troops } else { 0 },
            owner_id: if in_vision { self.owner_id } else { None },
            fog_of_war: !in_vision && terrain_visible,
        })
    }
}
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

    pub fn to_map_view(&self, player: Uuid, server: &Server) -> MapView {
        let mut visible_cells = std::collections::HashMap::new();
        let cells = self.cells.read();
        let config = server.config.read();

        // Check if player is alive - dead players can see everything
        if let Some(player_info) = server.players.read().get(&player) {
            if !*player_info.alive.read() {
                // Dead players can see everything
                for (id, cell) in cells.iter().enumerate() {
                    if let Some(view) = cell.to_view(true, true) {
                        visible_cells.insert(id, view);
                    }
                }
                return MapView { width: self.width, height: self.height, cells: visible_cells };
            }
        }

        // First pass: Calculate visible cells based on ownership
        let mut visible_ids = Vec::new();
        for (id, cell) in cells.iter().enumerate() {
            if cell.owner_id == Some(player) {
                let (center_x, center_y) = (id % self.width, id / self.width);
                let radius = match cell.terrain {
                    Terrain::City | Terrain::Capital => config.city_visibility_radius,
                    _ => config.tile_visibility_radius,
                };

                // Calculate visible cell IDs within radius
                let min_x = center_x.saturating_sub(radius);
                let max_x = (center_x + radius + 1).min(self.width);
                let min_y = center_y.saturating_sub(radius);
                let max_y = (center_y + radius + 1).min(self.height);

                for y in min_y..max_y {
                    for x in min_x..max_x {
                        if (x as i32 - center_x as i32).abs() + (y as i32 - center_y as i32).abs() <= radius as i32 {
                            visible_ids.push(self.get_cell_id(x, y));
                        }
                    }
                }
            }
        }

        // Remove duplicates from visible IDs
        visible_ids.sort_unstable();
        visible_ids.dedup();

        // Second pass: Add visible cells and handle fog of war for mountains/swamps
        for (id, cell) in cells.iter().enumerate() {
            let in_vision = visible_ids.contains(&id);
            let terrain_visible = match cell.terrain {
                Terrain::Mountain => !config.fow_mountains,
                Terrain::Swamp => !config.fow_swamps,
                _ => false
            };

            if let Some(view) = cell.to_view(in_vision, terrain_visible) {
                visible_cells.insert(id, view);
            }
        }

        MapView { width: self.width, height: self.height, cells: visible_cells }
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