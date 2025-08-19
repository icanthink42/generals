#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Terrain {
    Default,
    Mountain,
    Swamp,
    Desert,
}