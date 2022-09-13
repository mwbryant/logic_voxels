pub use bevy::prelude::*;

pub use crate::chunks::direction::Direction;
pub use crate::chunks::*;
pub use crate::material::*;
pub use crate::*;

pub const CHUNK_SIZE: usize = 16;
pub const WORLD_SIZE: usize = 20;
pub const MAX_CHUNK_UPDATES_PER_FRAME: usize = 30;
