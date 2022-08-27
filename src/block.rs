use bevy::prelude::*;

use crate::chunk::ChunkDirection;

#[derive(Default, Clone, Copy, Debug)]
pub enum Block {
    #[default]
    Air,
    Grass,
    Dirt,
}

impl Block {
    pub fn is_filled(&self) -> bool {
        !matches!(self, Block::Air)
    }

    pub fn get_face_index(&self, direction: ChunkDirection) -> u32 {
        match self {
            Block::Air => 0,
            Block::Grass => match direction {
                ChunkDirection::Front | ChunkDirection::Back | ChunkDirection::Left | ChunkDirection::Right => 1,
                ChunkDirection::Top => 0,
                ChunkDirection::Bottom => 2,
            },
            Block::Dirt => 2,
        }
    }
}
