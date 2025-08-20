#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use log::info;
#[cfg(target_arch = "wasm32")]
use wasm_sockets::{PollingClient, ConnectionStatus};

#[cfg(target_arch = "wasm32")]
use crate::client::game::Game;
#[cfg(target_arch = "wasm32")]
use crate::shared::{CBPacket, SBPacket};
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
        let client = Rc::new(RefCell::new(
            PollingClient::new("ws://127.0.0.1:1812/ws")
                .map_err(|e| JsValue::from_str(&format!("Failed to create WebSocket: {:?}", e)))?
        ));

        Ok(WebSocketClient {
            client: client.clone(),
            login_sent: false,
        })
    }

    pub fn update(&mut self, game: &Arc<Game>) -> Result<(), JsValue> {
        // Send login once connected
        if !self.login_sent && self.client.borrow().status() == ConnectionStatus::Connected {
            info!("Connected! Sending login packet...");
            let login_bytes = bincode::serialize(&SBPacket::Login(Login {
                username: "guest".to_string(),
                color_bid: None,
            }))
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize login: {:?}", e)))?;

            self.client.borrow_mut().send_binary(login_bytes)
                .map_err(|e| JsValue::from_str(&format!("Failed to send login: {:?}", e)))?;
            self.login_sent = true;
        }

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

    fn handle_packet(&self, pkt: CBPacket, game: &Arc<Game>) {
        match pkt {
            CBPacket::LoginAccepted(_) => {
                info!("Login accepted");
                if let Ok(bytes) = bincode::serialize(&SBPacket::GiveMeMap) {
                    self.client.borrow_mut().send_binary(bytes)
                        .map_err(|e| JsValue::from_str(&format!("Failed to send GiveMeMap: {:?}", e))).ok();
                }
            }
            CBPacket::MapSync(map_sync) => {
                info!("Processing map sync packet");
                game.map.lock().replace(map_sync.map);
            }
            CBPacket::SyncPlayers(sync_players) => {
                info!("Processing sync players packet");
                *game.players.lock() = sync_players.players;
            }
            CBPacket::TickPaths => {
                info!("Processing tick paths packet");
                game.tick_paths();
            }
        }
    }
}