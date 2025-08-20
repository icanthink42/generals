mod game;
mod movement;
mod rendering;
mod websocket;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use log::Level;

#[cfg(target_arch = "wasm32")]
use self::game::Game;
#[cfg(target_arch = "wasm32")]
use self::websocket::WebSocketClient;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Set up better panic messages and logging

    use std::sync::Arc;

    use parking_lot::Mutex;

    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Info).expect("Failed to initialize logging");

    // Initialize game and websocket
    let websocket = Arc::new(Mutex::new(WebSocketClient::new().expect("Failed to create websocket")));
    let game = Arc::new(Game::new(websocket.clone()).expect("Failed to create game"));

    // Set up resize handler
    let resize_game = game.clone();
    let resize_handler = Closure::wrap(Box::new(move || {
        let window = web_sys::window().unwrap();
        let canvas = resize_game.canvas().lock();
        canvas.set_width(window.inner_width().unwrap().as_f64().unwrap() as u32);
        canvas.set_height(window.inner_height().unwrap().as_f64().unwrap() as u32);
    }) as Box<dyn FnMut()>);

    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback(
            "resize",
            resize_handler.as_ref().unchecked_ref(),
        )?;
    resize_handler.forget();

    // Set up click handler
    let click_game = game.clone();
    let click_handler = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        let rect = click_game.canvas().lock().get_bounding_client_rect();
        let x = event.client_x() as f64 - rect.left();
        let y = event.client_y() as f64 - rect.top();
        click_game.handle_click(x, y);
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);

    game.canvas().lock()
        .add_event_listener_with_callback(
            "click",
            click_handler.as_ref().unchecked_ref(),
        )?;
    click_handler.forget();

    // Set up keyboard handler
    let keyboard_game = game.clone();
    let keyboard_handler = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        let key = event.key().to_lowercase();
        match key.as_str() {
            "w" | "a" | "s" | "d" => {
                event.prevent_default();
                keyboard_game.handle_wasd(key.as_str());
            }
            _ => {}
        }
    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback(
            "keydown",
            keyboard_handler.as_ref().unchecked_ref(),
        )?;
    keyboard_handler.forget();

    // Set up game loop
    let game_loop = game.clone();
    let websocket_loop = websocket.clone();
    let f = Closure::wrap(Box::new(move || {
        if let Err(e) = websocket_loop.lock().update(&game_loop) {
            web_sys::console::error_1(&e);
        }
        game_loop.render_grid();
    }) as Box<dyn FnMut()>);

    web_sys::window()
        .unwrap()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            f.as_ref().unchecked_ref(),
            16, // ~60 FPS
        )?;
    f.forget();

    Ok(())
}