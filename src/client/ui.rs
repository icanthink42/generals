#[cfg(target_arch = "wasm32")]
use {
    crate::{
        client::{
            button::Button, game::Game, text_input::TextInput, websocket::WebSocketClient
        },
        shared::{
            sb_packet::Login, Color, SBPacket
        },
    }, parking_lot::Mutex, rand::Rng, std::rc::Rc
};

#[cfg(target_arch = "wasm32")]
pub fn get_buttons(game: Rc<Game>, logical_width: f64, logical_height: f64) -> Vec<Button> {
    let button_width = 200.0;
    let button_height = 50.0;

    vec![
        // Join button (shown before connecting)
        Button::new(
            "Join Game".to_string(),
            (logical_width - button_width) / 2.0,  // center horizontally
            logical_height / 2.0 + 20.0,    // below vertical center
            button_width,
            button_height,
            {
                let game = game.clone();
                Rc::new(move || {
                    // Generate a random color (avoiding too dark colors)
                    let mut rng = rand::thread_rng();
                    let color = Color {
                        r: rng.gen_range(50..=255),
                        g: rng.gen_range(50..=255),
                        b: rng.gen_range(50..=255),
                        a: 255,
                    };

                    // Send login packet with name and color
                    if let Ok(bytes) = bincode::serialize(&SBPacket::Login(Login {
                        username: game.player_name.lock().clone(),
                        color_bid: Some(color),
                    })) {
                        game.websocket.lock().send_binary(bytes);
                    }
                })
            },
        ),
        // Start Game button (shown after connecting)
        Button::new(
            "Start Game".to_string(),
            (logical_width - button_width) / 2.0,  // center horizontally
            logical_height / 2.0 + 20.0,    // below vertical center
            button_width,
            button_height,
            {
                let game = game.clone();
                Rc::new(move || {
                    if let Ok(bytes) = bincode::serialize(&SBPacket::StartGame) {
                        game.websocket.lock().send_binary(bytes);
                    }
                })
            },
        ),
    ]
}

#[cfg(target_arch = "wasm32")]
pub fn get_text_inputs(game: Rc<Game>, logical_width: f64, logical_height: f64) -> Vec<TextInput> {
    let button_width = 200.0;  // Use same width as buttons for consistency

    vec![
        TextInput::new(
            "Enter your name...".to_string(),
            (logical_width - button_width) / 2.0,  // center horizontally
            logical_height / 2.0 - 40.0,    // above start button
            button_width,
            40.0,                          // slightly shorter than buttons
            {
                let game = game.clone();
                Rc::new(move |text| {
                    // Store the name
                    *game.player_name.lock() = text.to_string();
                })
            },
        ),
    ]
}
