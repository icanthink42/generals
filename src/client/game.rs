#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[cfg(target_arch = "wasm32")]
use crate::shared::path::Path;
#[cfg(target_arch = "wasm32")]
use crate::shared::{map::MapView, PlayerView};
#[cfg(target_arch = "wasm32")]
use crate::client::websocket::WebSocketClient;

#[cfg(target_arch = "wasm32")]
use parking_lot::Mutex;

#[cfg(target_arch = "wasm32")]
pub struct Game {
    pub map: Mutex<Option<MapView>>,
    pub canvas: Mutex<HtmlCanvasElement>,
    pub context: Mutex<CanvasRenderingContext2d>,
    pub selected_cell: Mutex<Option<usize>>,
    pub players: Mutex<Vec<PlayerView>>,
    pub paths: Mutex<HashMap<u32, Mutex<Path>>>,
    pub websocket: Arc<Mutex<WebSocketClient>>,
}

#[cfg(target_arch = "wasm32")]
impl Game {
    pub fn new(websocket: Arc<Mutex<WebSocketClient>>) -> Result<Game, JsValue> {
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
            paths: Mutex::new(HashMap::new()),
            websocket,
        })
    }

    pub fn canvas(&self) -> &Mutex<HtmlCanvasElement> {
        &self.canvas
    }

    pub fn tick_paths(&self) {
        let mut paths = self.paths.lock();
        // Remove first tile from each path and remove empty paths
        paths.retain(|_, path| {
            let mut path = path.lock();
            path.remove_front(1);
            !path.tile_ids.is_empty()
        });
    }
}