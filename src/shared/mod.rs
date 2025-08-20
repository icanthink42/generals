pub mod map;
pub mod terrain;
pub mod cb_packet;
pub mod sb_packet;
pub mod packet;
pub mod player;
pub mod path;
pub mod game_state;

pub use map::MapView;
pub use terrain::Terrain;
pub use cb_packet::CBPacket;
pub use sb_packet::SBPacket;
pub use packet::{read_len_prefixed, write_len_prefixed};
pub use player::PlayerView;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
