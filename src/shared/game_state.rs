#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum GameState {
    Lobby,
    InGame,
    GameOver,
}