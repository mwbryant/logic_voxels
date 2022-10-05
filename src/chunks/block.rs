use crate::prelude::*;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Block {
    #[default]
    Air,
    Grass,
    Dirt,
    Metal,
    Glass,
}

impl Block {
    pub fn is_filled(&self) -> bool {
        !matches!(self, Block::Air)
    }

    pub fn get_face_index(&self, direction: Direction) -> u32 {
        match self {
            Block::Air => 0,
            Block::Grass => match direction {
                Direction::Front | Direction::Back | Direction::Left | Direction::Right => 1,
                Direction::Top => 0,
                Direction::Bottom => 2,
            },
            Block::Dirt => 2,
            Block::Metal => match direction {
                Direction::Front | Direction::Back | Direction::Left | Direction::Right => 3,
                Direction::Top => 5,
                Direction::Bottom => 6,
            },
            Block::Glass => 4,
        }
    }
}
