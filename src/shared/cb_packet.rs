use uuid::Uuid;

use crate::shared::Color;

use super::map::MapView;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum CBPacket {
    LoginAccepted(LoginAccepted),
    MapSync(MapSync),
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