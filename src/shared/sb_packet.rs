use crate::shared::Color;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum SBPacket {
    Login(Login),
    GiveMeMap,
}


#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Login {
    pub username: String,
    pub color_bid: Option<Color>,
}
