#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use parking_lot::Mutex;

use super::game::Game;

impl Game {
    fn get_path_color(&self, id: u32) -> String {
        // Use golden ratio to get nice spread of colors
        let golden_ratio = 0.618033988749895;
        let mut hue = (id as f64 * golden_ratio) % 1.0;

        // Shift hue to avoid similar colors
        hue = (hue + 0.5) % 1.0;

        // Convert HSV to RGB (using fixed saturation and value)
        let h = hue * 6.0;
        let s = 0.7; // Medium-high saturation
        let v = 0.95; // High value/brightness

        let i = h.floor();
        let f = h - i;
        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));

        let (r, g, b) = match i as i32 % 6 {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };

        format!(
            "rgba({}, {}, {}, 0.8)",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8
        )
    }

    pub fn render_grid(&self) {
        let map_guard = self.map.lock();
        let Some(map) = map_guard.as_ref() else {
            return;
        };

        let canvas = self.canvas.lock();
        let width = canvas.width() as f64;
        let height = canvas.height() as f64;

        // Clear canvas
        let context = self.context.lock();
        context.set_fill_style_str("#1a1a1a");
        context.fill_rect(0.0, 0.0, width, height);

        // Grid config
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

        // Calculate cell size to maintain aspect ratio
        let cell_size = {
            let by_width = (available_width - (cell_gap * (cols as f64 - 1.0))) / cols as f64;
            let by_height = (available_height - (cell_gap * (rows as f64 - 1.0))) / rows as f64;
            by_width.min(by_height).min(desired_cell_size)
        };

        // Calculate actual grid size
        let grid_width = cols as f64 * (cell_size + cell_gap) - cell_gap;
        let grid_height = rows as f64 * (cell_size + cell_gap) - cell_gap;

        // Center the grid
        let x_offset = (logical_width - grid_width) / 2.0;
        let y_offset = (logical_height - grid_height) / 2.0;

        // Draw cells
        for row in 0..rows {
            for col in 0..cols {
                let cell_id = row * cols + col;
                let x = x_offset + col as f64 * (cell_size + cell_gap);
                let y = y_offset + row as f64 * (cell_size + cell_gap);

                // Draw cell background
                if let Some(cell) = map.cells.get(&cell_id) {
                    if let Some(owner_id) = cell.owner_id {
                        // Find the owner's color
                        let players = self.players.lock();
                        if let Some(owner) = players.iter().find(|p| p.id == owner_id) {
                            context.set_fill_style_str(&format!("rgba({}, {}, {}, {})",
                                owner.color.r, owner.color.g, owner.color.b, owner.color.a as f64 / 255.0));
                        } else {
                            context.set_fill_style_str("#4a4a4a");  // Default if owner not found
                        }
                    } else {
                        context.set_fill_style_str("#4a4a4a");  // Unowned but visible cell
                    }
                } else {
                    context.set_fill_style_str("#2a2a2a");  // Dark gray for fog of war
                }
                context.fill_rect(x, y, cell_size, cell_size);

                // Draw troop count if cell is visible and has troops
                if let Some(cell) = map.cells.get(&cell_id) {
                    if cell.troops > 0 {
                        context.set_fill_style_str("white");
                        context.set_text_align("center");
                        context.set_text_baseline("middle");
                        context.set_font(&format!("{}px Arial", cell_size * 0.5));
                        let _ = context.fill_text(
                            &cell.troops.to_string(),
                            x + cell_size / 2.0,
                            y + cell_size / 2.0,
                        );
                    }
                }

                // Draw path if this cell is part of one
                let paths = self.paths.lock();
                for path in paths.values() {
                    let path_guard = path.lock();
                    if path_guard.tile_ids.contains(&(cell_id as u32)) {
                        // Use the first tile in the path for the color
                        if let Some(&first_tile) = path_guard.tile_ids.first() {
                            context.set_stroke_style_str(&self.get_path_color(first_tile));
                            context.set_line_width(2.0);
                            let padding = 1.0;
                            context.stroke_rect(
                                x - padding/2.0,
                                y - padding/2.0,
                                cell_size + padding,
                                cell_size + padding
                            );
                        }
                    }
                }

                // Draw selection border if this is the selected cell
                if let Some(selected) = *self.selected_cell.lock() {
                    if selected == cell_id {
                        context.set_stroke_style_str("#ffd700"); // Gold color
                        context.set_line_width(2.0);
                        let padding = 1.0;
                        context.stroke_rect(
                            x - padding/2.0,
                            y - padding/2.0,
                            cell_size + padding,
                            cell_size + padding
                        );
                    }
                }
            }
        }
    }
}