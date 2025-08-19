use std::collections::HashMap;

use super::terrain::Terrain;
use parking_lot::RwLock;
use uuid::Uuid;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MapView {
    pub width: usize,
    pub height: usize,
    pub cells: HashMap<usize, Cell>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Cell {
    pub terrain: Terrain,
    pub troops: u32,
    pub owner_id: Option<Uuid>,
}