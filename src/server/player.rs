use std::sync::Arc;
use axum::extract::ws::{WebSocket, Message};
use futures_util::{stream::SplitSink, SinkExt};
use tokio::sync::mpsc::{self, UnboundedSender};

use generals::shared::{cb_packet::MapSync, CBPacket, Color, PlayerView, SBPacket};
use uuid::Uuid;

use crate::Server;

use parking_lot::RwLock;

pub struct Player {
    pub id: Uuid,
    pub name: RwLock<String>,
    pub color: RwLock<Color>,
    pub tx: UnboundedSender<Vec<u8>>,
}

impl Player {
        pub fn new(id: Uuid, name: String, color: Color, mut sink: SplitSink<WebSocket, Message>) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Spawn a dedicated task for handling this player's connection
        tokio::spawn(async move {
            while let Some(bytes) = rx.recv().await {
                let _ = sink.send(Message::Binary(bytes)).await;
            }
        });

        Self {
            id,
            name: RwLock::new(name),
            color: RwLock::new(color),
            tx
        }
    }

    pub fn id(&self) -> Uuid { self.id }

    pub async fn handle_packet(&self, packet: SBPacket, server: &Arc<Server>) {
        match packet {
            SBPacket::Login(login) => {
                println!("Received login packet from already logged in player {}", self.name.read());
            }
            SBPacket::GiveMeMap => {
                let map = server.map.to_map_view(self.id());
                let packet = CBPacket::MapSync(MapSync { map });
                if let Ok(bytes) = bincode::serialize(&packet) {
                    let _ = self.tx.send(bytes);
                }
            }
        }
    }

    pub fn send_bytes(&self, bytes: Vec<u8>) {
        let _ = self.tx.send(bytes);
    }

    pub fn to_view(&self) -> PlayerView {
        PlayerView {
            id: self.id,
            name: self.name.read().clone(),
            color: *self.color.read(),
        }
    }
}