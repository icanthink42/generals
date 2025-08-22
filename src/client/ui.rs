#[cfg(target_arch = "wasm32")]
use {
    std::rc::Rc,
    std::sync::Arc,
    parking_lot::Mutex,
    web_sys::CanvasRenderingContext2d,
    crate::{
        client::{
            button::Button,
            websocket::WebSocketClient,
        },
        shared::SBPacket,
    },
};

#[cfg(target_arch = "wasm32")]
pub struct UI {
    pub buttons: Vec<Button>,
}

#[cfg(target_arch = "wasm32")]
impl UI {
    pub fn new(websocket: Rc<Mutex<WebSocketClient>>) -> Self {
        let window = web_sys::window().unwrap();
        let dpr = window.device_pixel_ratio();
        let width = window.inner_width().unwrap().as_f64().unwrap() * dpr;
        let height = window.inner_height().unwrap().as_f64().unwrap() * dpr;
        let logical_width = width / dpr;
        let logical_height = height / dpr;

        // Define button dimensions
        let button_width = 200.0;
        let button_height = 50.0;

        Self {
            buttons: vec![
                Button::new(
                    "Start Game".to_string(),
                    (logical_width - button_width) / 2.0,  // center horizontally
                    logical_height / 2.0 + 20.0,    // below vertical center
                    button_width,
                    button_height,
                    Rc::new(move || {
                        if let Ok(bytes) = bincode::serialize(&SBPacket::StartGame) {
                            websocket.lock().send_binary(bytes);
                        }
                    }),
                ),
                // Add more buttons here as needed
            ],
        }
    }

    pub fn handle_click(&self, x: f64, y: f64) -> bool {
        for button in &self.buttons {
            if button.enabled && button.contains(x, y) {
                (button.callback)();
                return true;
            }
        }
        false
    }

    pub fn render(&self, context: &CanvasRenderingContext2d) {
        for button in &self.buttons {
            button.render(context);
        }
    }

    pub fn resize(&mut self, width: f64, height: f64) {
        let window = web_sys::window().unwrap();
        let dpr = window.device_pixel_ratio();
        let logical_width = width / dpr;
        let logical_height = height / dpr;

        // Update button positions
        if let Some(start_button) = self.buttons.get_mut(0) {
            start_button.x = (logical_width - start_button.width) / 2.0;
            start_button.y = logical_height / 2.0 + 20.0;
        }
        // Add more button position updates as needed
    }
}
