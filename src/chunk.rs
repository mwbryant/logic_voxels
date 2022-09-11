use std::sync::{Arc, RwLock, Weak};

use bevy::prelude::*;

use crate::{block::Block, direction::Direction, CHUNK_SIZE};

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
    pub pos: IVec3,
    pub cubes: ChunkData,
    pub dirty: bool,
    pub neighbors: [Weak<RwLock<Chunk>>; 6],
}

impl Default for Chunk {
    fn default() -> Chunk {
        Chunk {
            pos: IVec3::ZERO,
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

impl Chunk {
    pub fn get_block(&self, x: isize, y: isize, z: isize) -> Option<Block> {
        if Self::index_inbounds(x) && Self::index_inbounds(y) && Self::index_inbounds(z) {
            Some(self.cubes[x as usize][y as usize][z as usize])
        } else if x < 0 {
            assert!(Self::index_inbounds(y) && Self::index_inbounds(z));
            self.neighbors[Direction::Back]
                .upgrade()
                .map(|back| back.read().unwrap().cubes[CHUNK_SIZE - 1][y as usize][z as usize])
        } else if x >= CHUNK_SIZE as isize {
            assert!(Self::index_inbounds(y) && Self::index_inbounds(z));
            self.neighbors[Direction::Front]
                .upgrade()
                .map(|front| front.read().unwrap().cubes[0][y as usize][z as usize])
        } else if z < 0 {
            assert!(Self::index_inbounds(x) && Self::index_inbounds(y));
            self.neighbors[Direction::Right]
                .upgrade()
                .map(|front| front.read().unwrap().cubes[x as usize][y as usize][CHUNK_SIZE - 1])
        } else if z >= CHUNK_SIZE as isize {
            assert!(Self::index_inbounds(x) && Self::index_inbounds(y));
            self.neighbors[Direction::Left]
                .upgrade()
                .map(|back| back.read().unwrap().cubes[x as usize][y as usize][0])
        } else if y < 0 {
            assert!(Self::index_inbounds(x) && Self::index_inbounds(z));
            self.neighbors[Direction::Bottom]
                .upgrade()
                .map(|bottom| bottom.read().unwrap().cubes[x as usize][CHUNK_SIZE - 1][z as usize])
        } else if y >= CHUNK_SIZE as isize {
            assert!(Self::index_inbounds(x) && Self::index_inbounds(z));
            self.neighbors[Direction::Top]
                .upgrade()
                .map(|top| top.read().unwrap().cubes[x as usize][0][z as usize])
        } else {
            None
        }
    }

    pub fn get_block_neighbors(&self, x: isize, y: isize, z: isize) -> [Option<Block>; 6] {
        let mut block_neighbors = [None; 6];
        //Front
        block_neighbors[Direction::Front] = self.get_block(x + 1, y, z);
        block_neighbors[Direction::Back] = self.get_block(x - 1, y, z);
        block_neighbors[Direction::Left] = self.get_block(x, y, z + 1);
        block_neighbors[Direction::Right] = self.get_block(x, y, z - 1);
        block_neighbors[Direction::Top] = self.get_block(x, y + 1, z);
        block_neighbors[Direction::Bottom] = self.get_block(x, y - 1, z);
        block_neighbors
    }

    fn index_inbounds(index: isize) -> bool {
        index >= 0 && index < CHUNK_SIZE as isize
    }
}
