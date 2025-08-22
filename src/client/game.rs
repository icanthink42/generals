#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[cfg(target_arch = "wasm32")]
use crate::{
    shared::{
        game_state::GameState,
        path::Path,
        map::MapView,
        PlayerView,
        SBPacket,
        sb_packet::{UpdatePaths, Login},
    },
    client::{
        websocket::WebSocketClient,
        button::Button,
        text_input::TextInput,
    },
};


#[cfg(target_arch = "wasm32")]
use parking_lot::Mutex;

#[cfg(target_arch = "wasm32")]
pub struct Game {
    pub map: Mutex<Option<MapView>>,
    pub canvas: Mutex<HtmlCanvasElement>,
    pub context: Mutex<CanvasRenderingContext2d>,
    pub selected_cell: Mutex<Option<usize>>,
    pub selected_path: Mutex<Option<u32>>,  // ID of the currently selected path
    pub players: Mutex<Vec<PlayerView>>,
    pub paths: Mutex<HashMap<u32, Mutex<Path>>>,
    pub next_path_id: Mutex<u32>,
    pub websocket: Rc<Mutex<WebSocketClient>>,
    pub game_state: Mutex<GameState>,
    pub buttons: Mutex<Vec<Button>>,
    pub text_inputs: Mutex<Vec<TextInput>>,
    pub player_name: Mutex<String>,
    pub connected: Mutex<bool>,
}

#[cfg(target_arch = "wasm32")]
impl Game {
    pub fn new(websocket: Rc<Mutex<WebSocketClient>>) -> Result<Rc<Game>, JsValue> {
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
        let button_width = 200.0;
        let button_height = 50.0;
        let logical_width = width / dpr;
        let logical_height = height / dpr;


        let game = Rc::new(Game {
                map: Mutex::new(None),
                canvas: Mutex::new(canvas),
                context: Mutex::new(context),
                selected_cell: Mutex::new(None),
                selected_path: Mutex::new(None),
                players: Mutex::new(vec![]),
                paths: Mutex::new(HashMap::new()),
                next_path_id: Mutex::new(0),
                websocket: websocket.clone(),
                game_state: Mutex::new(GameState::Lobby),
                buttons: Mutex::new(Vec::new()),
                text_inputs: Mutex::new(Vec::new()),
                player_name: Mutex::new(String::new()),
                connected: Mutex::new(false),
            });

        let buttons = crate::client::ui::get_buttons(game.clone(), logical_width, logical_height);
        let text_inputs = crate::client::ui::get_text_inputs(game.clone(), logical_width, logical_height);

        game.buttons.lock().extend(buttons);
        game.text_inputs.lock().extend(text_inputs);

            // Disable start button until name is entered
            if let Some(start_button) = game.buttons.lock().get_mut(0) {
                start_button.enabled = false;
            }

        Ok(game)
    }

    pub fn canvas(&self) -> &Mutex<HtmlCanvasElement> {
        &self.canvas
    }


    pub fn handle_movement_confirmed(&self, path_id: u32, valid_until: u32) {
        let paths = self.paths.lock();
        if let Some(path) = paths.get(&path_id) {
            let mut path = path.lock();
            path.valid_until = valid_until;
        }
    }

    pub fn handle_key(&self, key: &str) {
        // Only handle text input in lobby
        if *self.game_state.lock() == GameState::Lobby {
            let mut text_inputs = self.text_inputs.lock();
            for input in text_inputs.iter_mut() {
                input.handle_key(key);
                if input.focused {
                    // Enable/disable start button based on name
                    if let Some(start_button) = self.buttons.lock().get_mut(0) {
                        start_button.enabled = !input.text.trim().is_empty();
                    }
                }
            }
        }
    }

    pub fn handle_click(&self, client_x: f64, client_y: f64) {
        // Only check UI elements in lobby
        if *self.game_state.lock() == GameState::Lobby {
            // Check text inputs first
            let mut text_inputs = self.text_inputs.lock();
            for input in text_inputs.iter_mut() {
                if input.handle_click(client_x, client_y) {
                    return;
                }
            }
            drop(text_inputs);

        // Check only the visible button based on connection state
        let buttons = self.buttons.lock();
        if !*self.connected.lock() {
            // Only check Join button (first button) when not connected
            if let Some(button) = buttons.get(0) {
                if button.enabled && button.contains(client_x, client_y) {
                    (button.callback)();
                    return;
                }
            }
        } else {
            // Only check Start button (second button) when connected
            if let Some(button) = buttons.get(1) {
                if button.enabled && button.contains(client_x, client_y) {
                    (button.callback)();
                    return;
                }
            }
        }
        drop(buttons);
        }

        // Handle grid clicks
        if let Some(new_cell_id) = self.get_cell_at_position(client_x, client_y) {
            // Create a new path starting at this cell
            let mut next_id = self.next_path_id.lock();
            let path_id = *next_id;
            *next_id += 1;

            // Create a new path with just this cell
            let mut paths = self.paths.lock();
            paths.insert(path_id, Mutex::new(Path::new(vec![new_cell_id as u32])));

            // Set this as the selected path
            *self.selected_path.lock() = Some(path_id);

            // Send just this new path to the server
            let mut new_paths = HashMap::new();
            new_paths.insert(path_id, Path::new(vec![new_cell_id as u32]));
            if let Ok(bytes) = bincode::serialize(&SBPacket::UpdatePaths(UpdatePaths { paths: new_paths })) {
                self.websocket.lock().send_binary(bytes);
            }

            // Update selected cell
            self.selected_cell.lock().replace(new_cell_id);
        }
    }
}