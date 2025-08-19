use std::sync::Arc;
use axum::extract::ws::{WebSocket, Message};
use futures_util::{stream::SplitSink, SinkExt};

use generals::shared::{cb_packet::MapSync, CBPacket, Color, SBPacket};
use uuid::Uuid;

use crate::Server;

pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub color: Color,
    pub connection: Arc<tokio::sync::Mutex<SplitSink<WebSocket, Message>>>,
}

impl Player {
    pub fn new(id: Uuid, name: String, color: Color, connection: Arc<tokio::sync::Mutex<SplitSink<WebSocket, Message>>>) -> Self {
        Self { id, name, color, connection }
    }
    pub fn id(&self) -> Uuid { self.id }
    pub async fn handle_packet(&mut self, packet: SBPacket, server: &Arc<Server>) {
        match packet {
            SBPacket::Login(login) => {
                println!("Received login packet from already logged in player {}", self.name);
            }
            SBPacket::GiveMeMap => {
                let map = server.map.to_map_view(self.id());
                let packet = CBPacket::MapSync(MapSync { map });
                let bytes = bincode::serialize(&packet).unwrap();
                let mut sink = self.connection.lock().await;
                let _ = sink.send(Message::Binary(bytes)).await;
            }
        }
    }
}