use std::{
    f32::consts::PI,
    ops::{Index, IndexMut},
    sync::{Arc, RwLock, Weak},
};

use bevy::prelude::*;

use crate::{block::Block, CHUNK_SIZE};

//FIXME make chunk not pub and only expose methods
// That avoid deadlocks
// Ie only write when you know you'll finish
#[derive(Component, Clone)]
pub struct ChunkComp {
    pub chunk: Arc<RwLock<Chunk>>,
}

type ChunkData = [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

//TODO serialize?
#[derive(Clone)]
pub struct Chunk {
    pub cubes: ChunkData,
    pub dirty: bool,
    pub neighbors: [Weak<RwLock<Chunk>>; 6],
}

#[derive(Clone, Copy)]
pub enum ChunkDirection {
    Front = 0,  // x + 1
    Back = 1,   // x - 1
    Left = 2,   // z + 1
    Right = 3,  // z - 1
    Top = 4,    // y + 1
    Bottom = 5, // y - 1
}

impl Default for Chunk {
    fn default() -> Chunk {
        Chunk {
            cubes: [[[Block::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            dirty: false,
            neighbors: [
                Weak::new(),
                Weak::new(),
                Weak::new(),
                Weak::new(),
                Weak::new(),
                Weak::new(),
            ],
        }
    }
}

impl<T> Index<ChunkDirection> for [T; 6] {
    type Output = T;

    fn index(&self, index: ChunkDirection) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> IndexMut<ChunkDirection> for [T; 6] {
    fn index_mut(&mut self, index: ChunkDirection) -> &mut Self::Output {
        &mut self[index as usize]
    }
}
impl ChunkDirection {
    pub fn get_face_rotation(&self) -> Quat {
        match self {
            ChunkDirection::Front => Quat::from_axis_angle(Vec3::Y, PI / 2.0),
            ChunkDirection::Back => Quat::from_axis_angle(Vec3::Y, -PI / 2.0),
            ChunkDirection::Top => Quat::from_axis_angle(Vec3::X, -PI / 2.0),
            ChunkDirection::Bottom => Quat::from_axis_angle(Vec3::X, PI / 2.0),
            ChunkDirection::Left => Quat::from_axis_angle(Vec3::Y, 0.0),
            ChunkDirection::Right => Quat::from_axis_angle(Vec3::Y, PI),
        }
    }
}

impl Chunk {
    pub fn get_block(&self, x: isize, y: isize, z: isize) -> Option<Block> {
        if Self::index_inbounds(x) && Self::index_inbounds(y) && Self::index_inbounds(z) {
            Some(self.cubes[x as usize][y as usize][z as usize])
        } else if x < 0 {
            assert!(Self::index_inbounds(y) && Self::index_inbounds(z));
            self.neighbors[ChunkDirection::Back]
                .upgrade()
                .map(|back| back.read().unwrap().cubes[CHUNK_SIZE - 1][y as usize][z as usize])
        } else if x >= CHUNK_SIZE as isize {
            assert!(Self::index_inbounds(y) && Self::index_inbounds(z));
            self.neighbors[ChunkDirection::Front]
                .upgrade()
                .map(|front| front.read().unwrap().cubes[0][y as usize][z as usize])
        } else if z < 0 {
            assert!(Self::index_inbounds(x) && Self::index_inbounds(y));
            self.neighbors[ChunkDirection::Right]
                .upgrade()
                .map(|front| front.read().unwrap().cubes[x as usize][y as usize][CHUNK_SIZE - 1])
        } else if z >= CHUNK_SIZE as isize {
            assert!(Self::index_inbounds(x) && Self::index_inbounds(y));
            self.neighbors[ChunkDirection::Left]
                .upgrade()
                .map(|back| back.read().unwrap().cubes[x as usize][y as usize][0])
        } else if y < 0 {
            assert!(Self::index_inbounds(x) && Self::index_inbounds(z));
            self.neighbors[ChunkDirection::Bottom]
                .upgrade()
                .map(|bottom| bottom.read().unwrap().cubes[x as usize][CHUNK_SIZE - 1][z as usize])
        } else if y >= CHUNK_SIZE as isize {
            assert!(Self::index_inbounds(x) && Self::index_inbounds(z));
            self.neighbors[ChunkDirection::Top]
                .upgrade()
                .map(|top| top.read().unwrap().cubes[x as usize][0][z as usize])
        } else {
            None
        }
    }

    pub fn get_block_neighbors(&self, x: isize, y: isize, z: isize) -> [Option<Block>; 6] {
        let mut block_neighbors = [None; 6];
        //Front
        block_neighbors[ChunkDirection::Front] = self.get_block(x + 1, y, z);
        block_neighbors[ChunkDirection::Back] = self.get_block(x - 1, y, z);
        block_neighbors[ChunkDirection::Left] = self.get_block(x, y, z + 1);
        block_neighbors[ChunkDirection::Right] = self.get_block(x, y, z - 1);
        block_neighbors[ChunkDirection::Top] = self.get_block(x, y + 1, z);
        block_neighbors[ChunkDirection::Bottom] = self.get_block(x, y - 1, z);
        block_neighbors
    }

    fn index_inbounds(index: isize) -> bool {
        index >= 0 && index < CHUNK_SIZE as isize
    }
}
