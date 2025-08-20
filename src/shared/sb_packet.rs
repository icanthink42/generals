use std::collections::HashMap;

use crate::shared::{path::Path, Color};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum SBPacket {
    Login(Login),
    GiveMeMap,
    UpdatePaths(UpdatePaths),
}


#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Login {
    pub username: String,
    pub color_bid: Option<Color>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct UpdatePaths {
    pub paths: HashMap<u32, Path>,
}