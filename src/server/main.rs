mod player;
mod map;
mod tick;
mod generator;

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures_util::StreamExt;
use generals::shared::cb_packet::LoginAccepted;
use generals::shared::game_state::GameState;
use generals::shared::{CBPacket, Color, SBPacket};
use parking_lot::RwLock;

use uuid::Uuid;

use crate::map::Map;
use crate::player::Player;
use generator::{TerrainConfig, generate_map};

async fn ws_handler(ws: WebSocketUpgrade, server: Arc<Server>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, server.clone()))
}

async fn handle_socket(socket: WebSocket, server: Arc<Server>) {
    // Split the socket into read and write parts
    let (write, mut read) = socket.split();

    // Create a temporary player for the connection
    let player_id = Uuid::new_v4();
    let player = Arc::new(Player::new(
        player_id,
        "Connecting...".to_string(),
        Color { r: 0, g: 128, b: 255, a: 255 },
        write
    ));

    // Handle player disconnect when the loop ends
    let _cleanup = CleanupGuard {
        server: server.clone(),
        player_id,
    };

    while let Some(Ok(msg)) = read.next().await {
        if let Message::Binary(data) = msg {
            match bincode::deserialize::<SBPacket>(&data) {
                Ok(SBPacket::Login(login)) => {
                    // Update player info
                    *player.name.write() = login.username.clone();
                    if let Some(color) = login.color_bid {
                        *player.color.write() = color;
                    }

                    // Check if game is already in progress
                    let current_state = *server.game_state.read();
                    if current_state == GameState::InGame {
                        *player.alive.write() = false;
                    } else {
                        server.map.add_player_capital(player_id);
                    }

                    server.players.write().insert(player_id, player.clone());
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

                    // Send player sync with all players
                    let all_players: Vec<_> = server.players.read().values().map(|p| p.to_view()).collect();
                    let sync = CBPacket::SyncPlayers(generals::shared::cb_packet::SyncPlayers {
                        players: all_players
                    });
                    if let Ok(resp) = bincode::serialize(&sync) {
                        // Send to all connected players
                        for p in server.players.read().values() {
                            p.send_bytes(resp.clone());
                        }
                    }

                    // Send current game state to the new player
                    let game_state = (*server.game_state.read()).clone();
                    let state_packet = CBPacket::SetGameState(game_state);
                    if let Ok(resp) = bincode::serialize(&state_packet) {
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

struct CleanupGuard {
    server: Arc<Server>,
    player_id: Uuid,
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        // When a player disconnects, remove them from the game
        self.server.remove_player(self.player_id);
    }
}

struct Server {
    players: RwLock<HashMap<Uuid, Arc<Player>>>,
    map: Arc<Map>,
    game_state: RwLock<GameState>,
    tick_counter: RwLock<u32>,
}

impl Server {
    fn remove_player(&self, player_id: Uuid) {
        // Remove player from the map
        self.map.remove_player(player_id);

        // Remove player from the players list
        self.players.write().remove(&player_id);

        // Notify remaining players about the player list change
        let all_players: Vec<_> = self.players.read().values().map(|p| p.to_view()).collect();
        let sync = CBPacket::SyncPlayers(generals::shared::cb_packet::SyncPlayers {
            players: all_players
        });
        if let Ok(resp) = bincode::serialize(&sync) {
            // Send to all remaining players
            for p in self.players.read().values() {
                p.send_bytes(resp.clone());
            }
        }

        // Sync map to show territory changes
        self.sync_map();
    }

    fn new(map: Map) -> Self {
        Self {
            players: RwLock::new(HashMap::new()),
            map: Arc::new(map),
            game_state: RwLock::new(GameState::Lobby),
            tick_counter: RwLock::new(0),
        }
    }

    pub fn sync_map(&self) {
        let players = self.players.read();
        for player in players.values() {
            let map_view = self.map.to_map_view(player.id(), self);
            let packet = generals::shared::CBPacket::MapSync(generals::shared::cb_packet::MapSync { map: map_view });

            if let Ok(bytes) = bincode::serialize(&packet) {
                player.send_bytes(bytes);
            }
        }
    }

    pub fn set_game_state(&self, new_state: GameState) {
        // Update server's game state
        *self.game_state.write() = new_state;

        // Create packet to notify clients
        let packet = CBPacket::SetGameState(new_state);

        // Serialize and send to all players
        if let Ok(bytes) = bincode::serialize(&packet) {
            for player in self.players.read().values() {
                player.send_bytes(bytes.clone());
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Create a map with interesting terrain
    let config = TerrainConfig {
        mountain_density: 0.12,    // 12% mountains
        desert_density: 0.15,      // 15% desert
        swamp_density: 0.08,       // 8% swamps
        city_density: 0.04,        // 4% cities
        clustering_factor: 0.7,    // High clustering for natural-looking terrain
    };
    let map = generate_map(20, 20, config);
    let server = Arc::new(Server::new(map));

    println!("Generals.io server (WS) starting on 127.0.0.1:1812/ws...");

    // Start tick loop
    let tick_server = server.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));
        loop {
            interval.tick().await;
            tick_server.tick().await;
        }
    });

    let app = Router::new().route("/ws", get(move |ws| ws_handler(ws, server.clone())));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1812").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}