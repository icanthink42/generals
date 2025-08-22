#[cfg(target_arch = "wasm32")]
use web_sys;
#[cfg(target_arch = "wasm32")]
use parking_lot::Mutex;
#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use crate::shared::path::Path;
#[cfg(target_arch = "wasm32")]
use crate::shared::{SBPacket, sb_packet::UpdatePaths};

#[cfg(target_arch = "wasm32")]
use super::game::Game;

#[cfg(target_arch = "wasm32")]
impl Game {
    pub fn get_cell_at_position(&self, x: f64, y: f64) -> Option<usize> {
        let map_guard = self.map.lock();
        let map = map_guard.as_ref()?;
        let canvas = self.canvas.lock();

        let width = canvas.width() as f64;
        let height = canvas.height() as f64;
        let rows = map.height;
        let cols = map.width;

        let window = web_sys::window().unwrap();
        let dpr = window.device_pixel_ratio();

        let min_padding = 50.0;  // Use consistent padding value
        let cell_gap = 1.0;
        let desired_cell_size = 25.0;

        // Convert to logical pixels for calculations
        let logical_width = width / dpr;
        let logical_height = height / dpr;
        let available_width = logical_width - (2.0 * min_padding);
        let available_height = logical_height - (2.0 * min_padding);

        let cell_size = {
            let by_width = (available_width - (cell_gap * (cols as f64 - 1.0))) / cols as f64;
            let by_height = (available_height - (cell_gap * (rows as f64 - 1.0))) / rows as f64;
            by_width.min(by_height).min(desired_cell_size)
        };

        let grid_width = cols as f64 * (cell_size + cell_gap) - cell_gap;
        let grid_height = rows as f64 * (cell_size + cell_gap) - cell_gap;
        let x_offset = (logical_width - grid_width) / 2.0;
        let y_offset = (logical_height - grid_height) / 2.0;

        // Convert screen coordinates to grid coordinates
        let grid_x = x - x_offset;
        let grid_y = y - y_offset;

        // Check if click is within grid bounds
        if grid_x < 0.0 || grid_y < 0.0 || grid_x > grid_width || grid_y > grid_height {
            return None;
        }

        // Calculate cell coordinates
        let col = (grid_x / (cell_size + cell_gap)).floor() as usize;
        let row = (grid_y / (cell_size + cell_gap)).floor() as usize;

        // Check if within grid bounds
        if col >= cols || row >= rows {
            return None;
        }

        Some(row * cols + col)
    }

    pub fn handle_wasd(&self, key: &str) -> bool {
        let map_guard = self.map.lock();
        let Some(map) = map_guard.as_ref() else {
            return false;
        };

        let Some(current_cell) = *self.selected_cell.lock() else {
            return false;
        };

        let current_x = current_cell % map.width;
        let current_y = current_cell / map.width;

        // Calculate the new position based on WASD input
        let (new_x, new_y) = match key {
            "w" if current_y > 0 => (current_x, current_y - 1),
            "s" if current_y < map.height - 1 => (current_x, current_y + 1),
            "a" if current_x > 0 => (current_x - 1, current_y),
            "d" if current_x < map.width - 1 => (current_x + 1, current_y),
            _ => return false,
        };

        let new_cell = new_y * map.width + new_x;
        self.selected_cell.lock().replace(new_cell);

        // Only extend the selected path with WASD
        let paths = self.paths.lock();
        if let Some(path_id) = *self.selected_path.lock() {
            if let Some(path) = paths.get(&path_id) {
                path.lock().tile_ids.push(new_cell as u32);

                // Send just the updated path to server
                let mut updated_paths = HashMap::new();
                updated_paths.insert(path_id, path.lock().clone());
                if let Ok(bytes) = bincode::serialize(&SBPacket::UpdatePaths(UpdatePaths { paths: updated_paths })) {
                    self.websocket.lock().send_binary(bytes);
                }
            }
        }

        true
    }
}