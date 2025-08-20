use generals::shared::{map::Cell, MapView, Terrain};
use parking_lot::RwLock;
use uuid::Uuid;

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub cells: RwLock<Vec<Cell>>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, cells: RwLock::new(vec![Cell { terrain: Terrain::Default, troops: 0, owner_id: None }; width * height]) }
    }

    fn get_cell_id(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    #[allow(dead_code)]
    pub fn set_cell(&self, x: usize, y: usize, cell: Cell) {
        self.cells.write()[self.get_cell_id(x, y)] = cell;
    }

    pub fn get_adjacent_cells(&self, x: usize, y: usize) -> Vec<usize> {
        let mut cells = Vec::new();
        if x > 0 {
            cells.push(self.get_cell_id(x - 1, y));
        }
        if x < self.width - 1 {
            cells.push(self.get_cell_id(x + 1, y));
        }
        if y > 0 {
            cells.push(self.get_cell_id(x, y - 1));
        }
        if y < self.height - 1 {
            cells.push(self.get_cell_id(x, y + 1));
        }
        cells
    }

    pub fn get_visable_cells(&self, player: Uuid) -> Vec<usize> {
        let mut visible = Vec::new();
        let cells = self.cells.read();

        // First find all cells owned by the player
        for (id, cell) in cells.iter().enumerate() {
            if cell.owner_id == Some(player) {
                visible.push(id);
                // Add all adjacent cells
                let (x, y) = (id % self.width, id / self.width);
                visible.extend(self.get_adjacent_cells(x, y));
            }
        }

        // Remove duplicates
        visible.sort_unstable();
        visible.dedup();
        visible
    }

        pub fn to_map_view(&self, player: Uuid) -> MapView {
        let visible_cell_ids = self.get_visable_cells(player);
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

    pub fn increment_capital_troops(&self) {
        let mut cells = self.cells.write();
        for cell in cells.iter_mut() {
            if cell.terrain == Terrain::Capital {
                cell.troops += 1;
            }
        }
    }

        pub fn tile_battle(&self, attacking_id: usize, defending_id: usize) {
        let mut cells = self.cells.write();

        // Get the current state
        let attacking_troops = cells[attacking_id].troops;
        let attacking_owner = cells[attacking_id].owner_id;
        let defending_troops = cells[defending_id].troops;
        let defending_owner = cells[defending_id].owner_id;

                // Don't do anything if attacking tile has 1 or fewer troops
        if attacking_troops <= 1 {
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
                } else {
                    // Defender wins or ties
                    cells[defending_id].troops -= moving_troops;
                }
            }
        }
    }
}