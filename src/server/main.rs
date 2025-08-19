mod player;
mod map;

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures_util::{StreamExt, SinkExt};
use generals::shared::cb_packet::LoginAccepted;
use generals::shared::{CBPacket, Color, MapView, SBPacket};
use parking_lot::RwLock;
use tokio::sync::{Mutex as AsyncMutex};
use uuid::Uuid;

use crate::map::Map;
use crate::player::Player;

async fn ws_handler(ws: WebSocketUpgrade, server: Arc<Server>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, server.clone()))
}

async fn handle_socket(socket: WebSocket, server: Arc<Server>) {
    // Split the socket into read and write parts
    let (write, mut read) = socket.split();
    let write = Arc::new(tokio::sync::Mutex::new(write));


    let mut player: Option<Player> = None;

    while let Some(Ok(msg)) = read.next().await {
        if let Message::Binary(data) = msg {
            match bincode::deserialize::<SBPacket>(&data) {
                Ok(SBPacket::Login(login)) => {
                    let player_id = Uuid::new_v4();
                    let color = login.color_bid.unwrap_or(Color { r: 0, g: 128, b: 255, a: 255 });
                    let new_player = Player::new(player_id, login.username.clone(), color, write.clone());
                    println!("Player with username {} logged in", login.username);
                    let accepted = CBPacket::LoginAccepted(LoginAccepted {
                        player_id,
                        color,
                    });
                    if let Ok(resp) = bincode::serialize(&accepted) {
                        let mut sink = write.lock().await;
                        let _ = sink.send(Message::Binary(resp)).await;
                    }
                    player = Some(new_player);
                }
                Ok(other) => {
                    if let Some(p) = &mut player {
                        p.handle_packet(other, &server).await;
                    }
                }
                Err(err) => eprintln!("bad packet: {}", err),
            }
        }
    }
}

struct Server {
    players: RwLock<HashMap<Uuid, Arc<Player>>>,
    map: Arc<Map>,
}

impl Server {
    fn new(map: Map) -> Self {
        Self { players: RwLock::new(HashMap::new()), map: Arc::new(map) }
    }
}

#[tokio::main]
async fn main() {
    let map = Map::new(50, 50);
    let server = Arc::new(Server::new(map));

    println!("Generals.io server (WS) starting on 127.0.0.1:1812/ws...");
    let app = Router::new().route("/ws", get(move |ws| ws_handler(ws, server.clone())));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1812").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}