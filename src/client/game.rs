#[cfg(target_arch = "wasm32")]
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::shared::map::MapView;
#[cfg(target_arch = "wasm32")]
use crate::shared::PlayerView;

#[cfg(target_arch = "wasm32")]
use parking_lot::Mutex;

#[cfg(target_arch = "wasm32")]
pub struct Game {
    pub map: Mutex<Option<MapView>>,
    pub canvas: Mutex<HtmlCanvasElement>,
    pub context: Mutex<CanvasRenderingContext2d>,
    pub selected_cell: Mutex<Option<usize>>,
    pub players: Mutex<Vec<PlayerView>>,
}

#[cfg(target_arch = "wasm32")]
impl Game {
    pub fn new() -> Result<Game, JsValue> {
        // Get canvas and context
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()?;

        // Set canvas size accounting for device pixel ratio
        let dpr = window.device_pixel_ratio();
        let width = window.inner_width()?.as_f64().unwrap() * dpr;
        let height = window.inner_height()?.as_f64().unwrap() * dpr;

        canvas.set_width(width as u32);
        canvas.set_height(height as u32);

        // Set CSS size
        canvas.style().set_property("width", &format!("{}px", width/dpr))?;
        canvas.style().set_property("height", &format!("{}px", height/dpr))?;

        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;

        // Scale context to match device pixel ratio
        context.scale(dpr, dpr)?;

        Ok(Game {
            map: Mutex::new(None),
            canvas: Mutex::new(canvas),
            context: Mutex::new(context),
            selected_cell: Mutex::new(None),
            players: Mutex::new(vec![]),
        })
    }

    pub fn canvas(&self) -> &Mutex<HtmlCanvasElement> {
        &self.canvas
    }

    fn get_cell_at_position(&self, x: f64, y: f64) -> Option<usize> {
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
        let window = web_sys::window().unwrap();
        let dpr = window.device_pixel_ratio();
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

    pub fn handle_click(&self, x: f64, y: f64) {
        if let Some(cell_id) = self.get_cell_at_position(x, y) {
            self.selected_cell.lock().replace(cell_id);
        }
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
        let window = web_sys::window().unwrap();
        let dpr = window.device_pixel_ratio();
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