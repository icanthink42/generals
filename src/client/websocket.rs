#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use rand::Rng;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use log::info;
#[cfg(target_arch = "wasm32")]
use wasm_sockets::{PollingClient, ConnectionStatus};

#[cfg(target_arch = "wasm32")]
use crate::client::game::Game;
#[cfg(target_arch = "wasm32")]
use crate::shared::{CBPacket, SBPacket, Color};
#[cfg(target_arch = "wasm32")]
use crate::shared::sb_packet::Login;

#[cfg(target_arch = "wasm32")]
pub struct WebSocketClient {
    client: Rc<RefCell<PollingClient>>,
    login_sent: bool,
}

#[cfg(target_arch = "wasm32")]
impl WebSocketClient {
        pub fn new() -> Result<WebSocketClient, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;
        let server_url = window
            .get("SERVER_URL")
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| "ws://127.0.0.1:1812/ws".to_string());

        let client = Rc::new(RefCell::new(
            PollingClient::new(&server_url)
                .map_err(|e| JsValue::from_str(&format!("Failed to create WebSocket: {:?}", e)))?
        ));

        Ok(WebSocketClient {
            client: client.clone(),
            login_sent: false,
        })
    }

    pub fn update(&mut self, game: &Rc<Game>) -> Result<(), JsValue> {

        // Handle any new messages
        let messages = self.client.borrow_mut().receive();
        for msg in messages {
            match msg {
                wasm_sockets::Message::Binary(data) => {
                    if let Ok(pkt) = bincode::deserialize::<CBPacket>(&data) {
                        self.handle_packet(pkt, &game);
                    } else {
                        info!("Failed to deserialize packet: {:?}", data);
                    }
                }
                other => info!("Received non-binary message: {:?}", other),
            }
        }

        Ok(())
    }

    pub fn send_binary(&self, bytes: Vec<u8>) {
        self.client.borrow_mut().send_binary(bytes).ok();
    }

    fn handle_packet(&self, pkt: CBPacket, game: &Rc<Game>) {
        match pkt {
            CBPacket::LoginAccepted(_) => {
                info!("Login accepted");
                if let Ok(bytes) = bincode::serialize(&SBPacket::GiveMeMap) {
                    self.client.borrow_mut().send_binary(bytes)
                        .map_err(|e| JsValue::from_str(&format!("Failed to send GiveMeMap: {:?}", e))).ok();
                }
                *game.connected.lock() = true;
            }
            CBPacket::MapSync(map_sync) => {
                info!("Processing map sync packet");
                game.map.lock().replace(map_sync.map);
            }
            CBPacket::SyncPlayers(sync_players) => {
                info!("Processing sync players packet");
                *game.players.lock() = sync_players.players;
            }

            CBPacket::SetGameState(game_state) => {
                info!("Processing set game state packet");
                *game.game_state.lock() = game_state;
            }
            CBPacket::MovementConfirmed(movement) => {
                info!("Processing movement confirmed packet");
                game.handle_movement_confirmed(movement.path_id, movement.valid_until);
            }
        }
    }
}