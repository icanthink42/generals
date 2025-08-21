use uuid::Uuid;

use crate::shared::{game_state::GameState, Color, PlayerView};

use super::map::MapView;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum CBPacket {
    LoginAccepted(LoginAccepted),
    MapSync(MapSync),
    SyncPlayers(SyncPlayers),

    SetGameState(GameState),
    MovementConfirmed(MovementConfirmed),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MapSync {
    pub map: MapView,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LoginAccepted {
    pub player_id: Uuid,
    pub color: Color,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SyncPlayers {
    pub players: Vec<PlayerView>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MovementConfirmed {
    pub path_id: u32,
    pub valid_until: u32,
}