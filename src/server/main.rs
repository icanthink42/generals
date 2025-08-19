mod player;
mod map;
mod tick;

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

    // Create a temporary player for the connection
    let player_id = Uuid::new_v4();
    let mut player = Arc::new(Player::new(
        player_id,
        "Connecting...".to_string(),
        Color { r: 0, g: 128, b: 255, a: 255 },
        write
    ));

    while let Some(Ok(msg)) = read.next().await {
        if let Message::Binary(data) = msg {
            match bincode::deserialize::<SBPacket>(&data) {
                Ok(SBPacket::Login(login)) => {
                    // Update player info
                    *player.name.write() = login.username.clone();
                    if let Some(color) = login.color_bid {
                        *player.color.write() = color;
                    }

                    server.players.write().insert(player_id, player.clone());
                    server.map.add_player_capital(player_id);
                    println!("Player with username {} logged in", player.name.read());
                    server.sync_map();

                    // Send login accepted
                    let accepted = CBPacket::LoginAccepted(LoginAccepted {
                        player_id,
                        color: *player.color.read(),
                    });
                    if let Ok(resp) = bincode::serialize(&accepted) {
                        player.send_bytes(resp);
                    }

                    // Send player sync
                    let sync = CBPacket::SyncPlayers(generals::shared::cb_packet::SyncPlayers {
                        players: vec![player.to_view()]
                    });
                    if let Ok(resp) = bincode::serialize(&sync) {
                        player.send_bytes(resp);
                    }
                }
                Ok(other) => {
                    player.handle_packet(other, &server).await;
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

    pub fn sync_map(&self) {
        let players = self.players.read();
        for player in players.values() {
            let map_view = self.map.to_map_view(player.id());
            let packet = generals::shared::CBPacket::MapSync(generals::shared::cb_packet::MapSync { map: map_view });

            if let Ok(bytes) = bincode::serialize(&packet) {
                player.send_bytes(bytes);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let map = Map::new(50, 50);
    let server = Arc::new(Server::new(map));

    println!("Generals.io server (WS) starting on 127.0.0.1:1812/ws...");

    // Start tick loop
    let tick_server = server.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            tick_server.tick().await;
        }
    });

    let app = Router::new().route("/ws", get(move |ws| ws_handler(ws, server.clone())));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1812").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}