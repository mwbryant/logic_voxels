use std::sync::{Arc, RwLock, Weak};

use bevy::prelude::*;

use crate::{block::Block, direction::Direction, CHUNK_SIZE};

//FIXME make chunk not pub and only expose methods
// That avoid deadlocks
// Ie only write when you know you'll finish
#[derive(Component, Clone)]
pub struct ChunkComp {
    chunk: Arc<RwLock<Chunk>>,
}

impl ChunkComp {
    pub fn new(chunk: Arc<RwLock<Chunk>>) -> Self {
        ChunkComp { chunk }
    }
    //These functions prevent deadlocks, in reality all that matters is writes finish so a pub read, private write would be nice
    pub fn write_block(&self, index: IVec3, block: Block) {
        //There's really no point in bounds checking this index, a logic error trying to write the wrong block should panic
        //Maybe one day there will be a use for a varient that returns a recoverable error
        self.chunk.write().unwrap().cubes[index.x as usize][index.y as usize][index.z as usize] = block;
        //Really only need to dirty if block is different but eh
        self.write_dirty(true);
    }

    pub fn write_block_xyz(&self, x: usize, y: usize, z: usize, block: Block) {
        self.chunk.write().unwrap().cubes[x][y][z] = block;
    }

    pub fn write_dirty(&self, value: bool) {
        self.chunk.write().unwrap().dirty = value;
    }

    pub fn write_neighbor(&self, dir: Direction, neighbor: &ChunkComp) {
        self.chunk.write().unwrap().neighbors[dir] = neighbor.as_neighbor();
    }

    fn as_neighbor(&self) -> Weak<RwLock<Chunk>> {
        Arc::downgrade(&self.chunk)
    }

    pub fn read_block_xyz(&self, x: usize, y: usize, z: usize) -> Block {
        //returns a copy of the block
        self.chunk.read().unwrap().cubes[x][y][z]
    }

    pub fn read_block(&self, index: IVec3) -> Block {
        //returns a copy of the block
        self.chunk.read().unwrap().cubes[index.x as usize][index.y as usize][index.z as usize]
    }

    pub fn read_dirty(&self) -> bool {
        self.chunk.read().unwrap().dirty
    }

    pub fn read_chunk(&self) -> std::sync::RwLockReadGuard<Chunk> {
        self.chunk.read().unwrap()
    }
}

type ChunkData = [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

//TODO serialize?
#[derive(Clone)]
pub struct Chunk {
    pub pos: IVec3,
    pub cubes: ChunkData,
    pub dirty: bool,
    //Cant be pub because then you could write them and cause weird deadlocks
    neighbors: [Weak<RwLock<Chunk>>; 6],
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

    pub fn get_block_neighbors(&self, x: usize, y: usize, z: usize) -> [Option<Block>; 6] {
        let (x, y, z) = (x as isize, y as isize, z as isize);
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
