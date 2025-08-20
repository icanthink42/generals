#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Path {
    pub tile_ids: Vec<u32>,
}

impl Path {
    pub fn new(tile_ids: Vec<u32>) -> Self {
        Self { tile_ids }
    }

    pub fn remove_front(&mut self, n: usize) {
        self.tile_ids.drain(0..n.min(self.tile_ids.len()));
    }

    pub fn is_valid(&self, width: usize, height: usize) -> bool {
        // Path must have at least one tile
        if self.tile_ids.is_empty() {
            return false;
        }

        // All tile IDs must be within map bounds
        let max_id = (width * height) as u32;
        if self.tile_ids.iter().any(|&id| id >= max_id) {
            return false;
        }

        // Check each consecutive pair of tiles for adjacency
        for window in self.tile_ids.windows(2) {
            let current = window[0] as usize;
            let next = window[1] as usize;

            // Convert tile IDs to grid coordinates
            let current_x = current % width;
            let current_y = current / width;
            let next_x = next % width;
            let next_y = next / width;

            // Check if tiles are adjacent (share an edge)
            let x_diff = next_x.abs_diff(current_x);
            let y_diff = next_y.abs_diff(current_y);

            // Tiles must differ by exactly 1 in either x or y, but not both
            if !((x_diff == 1 && y_diff == 0) || (x_diff == 0 && y_diff == 1)) {
                return false;
            }
        }

        true
    }


}