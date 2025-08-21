#[cfg(target_arch = "wasm32")]
use super::game::Game;
#[cfg(target_arch = "wasm32")]
use crate::shared::game_state::GameState;

#[cfg(target_arch = "wasm32")]
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

    fn render_player_list(&self, context: &web_sys::CanvasRenderingContext2d, x: f64, y: f64) {
        let players = self.players.lock();
        let padding = 10.0;
        let line_height = 25.0;
        let box_width = 200.0;
        let box_height = (players.len() as f64 * line_height) + (padding * 2.0);

        // Draw semi-transparent background
        context.set_fill_style_str("rgba(0, 0, 0, 0.7)");
        context.fill_rect(x, y, box_width, box_height);

        // Draw each player
        context.set_font("16px Arial");
        context.set_text_align("left");
        context.set_text_baseline("middle");

        for (i, player) in players.iter().enumerate() {
            let text_y = y + padding + (i as f64 * line_height) + (line_height / 2.0);

            // Draw color square
            context.set_fill_style_str(&format!("rgba({}, {}, {}, {})",
                player.color.r, player.color.g, player.color.b, player.color.a as f64 / 255.0));
            context.fill_rect(x + padding, text_y - 8.0, 16.0, 16.0);

            // Draw player name
            context.set_fill_style_str("white");
            let _ = context.fill_text(&player.name, x + padding + 25.0, text_y);
        }
    }

    fn render_lobby(&self, context: &web_sys::CanvasRenderingContext2d, width: f64, height: f64) {
        let window = web_sys::window().unwrap();
        let dpr = window.device_pixel_ratio();

        // Convert to logical pixels
        let logical_width = width / dpr;
        let logical_height = height / dpr;

        // Clear canvas
        context.set_fill_style_str("#1a1a1a");
        context.fill_rect(0.0, 0.0, width, height);

        // Draw waiting text
        context.set_font("24px Arial");
        context.set_fill_style_str("white");
        context.set_text_align("center");
        context.set_text_baseline("middle");
        let _ = context.fill_text("Waiting for game to begin...", logical_width / 2.0, logical_height / 2.0 - 40.0);

        // Draw start button
        let button_width = 200.0;
        let button_height = 50.0;
        let button_x = (logical_width - button_width) / 2.0;
        let button_y = logical_height / 2.0 + 20.0;

        // Draw button background
        context.set_fill_style_str("#4CAF50");  // Green color
        context.fill_rect(button_x, button_y, button_width, button_height);

        // Draw button text
        context.set_font("20px Arial");
        context.set_fill_style_str("white");
        let _ = context.fill_text("Start Game", logical_width / 2.0, button_y + button_height / 2.0);

        // Store button coordinates for click handling
        *self.start_button_bounds.lock() = Some((button_x, button_y, button_width, button_height));
    }

    pub fn render_grid(&self) {
        let canvas = self.canvas.lock();
        let width = canvas.width() as f64;
        let height = canvas.height() as f64;
        let context = self.context.lock();

        // Handle different game states
        match *self.game_state.lock() {
            GameState::Lobby => {
                self.render_lobby(&context, width, height);
            }
            GameState::InGame | GameState::GameOver => {
                let map_guard = self.map.lock();
                let Some(map) = map_guard.as_ref() else {
                    return;
                };

                // Clear canvas
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
                            if cell.fog_of_war {
                                // Fog of war cell - show terrain but with darker background
                                context.set_fill_style_str("#2a2a2a");  // Dark gray for fog of war
                            } else if let Some(owner_id) = cell.owner_id {
                                // Fully visible cell with owner
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

                        // Draw terrain emojis
                        if let Some(cell) = map.cells.get(&cell_id) {
                            let emoji = match cell.terrain {
                                crate::shared::terrain::Terrain::Capital => "👑",
                                crate::shared::terrain::Terrain::Mountain => "⛰️",
                                crate::shared::terrain::Terrain::Swamp => "🌿",
                                crate::shared::terrain::Terrain::City => "🏰",
                                _ => "",
                            };

                            if !emoji.is_empty() {
                                context.set_font(&format!("{}px Arial", cell_size * 0.9));
                                context.set_text_align("center");
                                context.set_text_baseline("middle");
                                if cell.fog_of_war {
                                    // Draw terrain emoji with reduced opacity for fog of war
                                    context.set_global_alpha(0.5);
                                    let _ = context.fill_text(
                                        emoji,
                                        x + cell_size / 2.0,
                                        y + cell_size / 2.0,
                                    );
                                    context.set_global_alpha(1.0);
                                } else {
                                    let _ = context.fill_text(
                                        emoji,
                                        x + cell_size / 2.0,
                                        y + cell_size / 2.0,
                                    );
                                }
                            }
                        }

                        // Draw troop count if cell is visible and not in fog of war
                        if let Some(cell) = map.cells.get(&cell_id) {
                            if cell.troops > 0 && !cell.fog_of_war {
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
                            let tile_index = path_guard.tile_ids.iter().position(|&id| id == cell_id as u32);
                            if let Some(index) = tile_index {
                                if index as u32 > path_guard.valid_until {
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

                // Draw player list overlay
                self.render_player_list(&context, 20.0, 20.0);
            }
        }
    }
}