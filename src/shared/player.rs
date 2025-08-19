use uuid::Uuid;
use super::Color;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerView {
    pub id: Uuid,
    pub name: String,
    pub color: Color,
}