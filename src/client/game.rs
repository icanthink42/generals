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
use parking_lot::Mutex;

#[cfg(target_arch = "wasm32")]
pub struct Game {
    pub map: Mutex<Option<MapView>>,
    pub canvas: Mutex<HtmlCanvasElement>,
    pub context: Mutex<CanvasRenderingContext2d>,
}

#[cfg(target_arch = "wasm32")]
impl Game {
    pub fn new() -> Result<Game, JsValue> {
        // Get canvas and context
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()?;

        // Set canvas size to window inner size
        canvas.set_width(window.inner_width()?.as_f64().unwrap() as u32);
        canvas.set_height(window.inner_height()?.as_f64().unwrap() as u32);

        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;

        Ok(Game {
            map: Mutex::new(None),
            canvas: Mutex::new(canvas),
            context: Mutex::new(context),
        })
    }

    pub fn canvas(&self) -> &Mutex<HtmlCanvasElement> {
        &self.canvas
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
        let min_padding = 20.0;
        let cell_gap = 2.0;
        let desired_cell_size = 40.0;

        // Calculate available space
        let available_width = width - (2.0 * min_padding);
        let available_height = height - (2.0 * min_padding);

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
        let x_offset = (width - grid_width) / 2.0;
        let y_offset = (height - grid_height) / 2.0;

        // Draw cells
        context.set_fill_style_str("#2a2a2a");
        for row in 0..rows {
            for col in 0..cols {
                let x = x_offset + col as f64 * (cell_size + cell_gap);
                let y = y_offset + row as f64 * (cell_size + cell_gap);
                context.fill_rect(x, y, cell_size, cell_size);
            }
        }
    }
}