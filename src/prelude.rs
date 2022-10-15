pub use bevy::prelude::*;

pub use crate::chunks::direction::Direction;
pub use crate::chunks::*;
pub use crate::client_utils::*;
pub use crate::networking::*;
pub use crate::physics::*;
pub use crate::server_utils::*;
pub use crate::*;

pub const CHUNK_SIZE: usize = 16;
pub const WORLD_SIZE: usize = 2;
pub const MAX_CHUNK_UPDATES_PER_FRAME: usize = 30;

//XXX maybe a memory leak because unloaded chunks are never removed
// Not a very robust design
#[derive(Default)]
pub struct LoadedChunks {
    pub ent_map: HashMap<IVec3, Entity>,
}
