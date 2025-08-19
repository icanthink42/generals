use std::sync::Arc;

use generals::shared::{map::Cell, MapView, Terrain};
use parking_lot::RwLock;
use uuid::Uuid;

use crate::player::Player;

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
}