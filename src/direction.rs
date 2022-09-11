use std::{
    f32::consts::PI,
    ops::{Index, IndexMut},
};

use bevy::prelude::*;

#[derive(Clone, Copy)]
pub enum Direction {
    Front = 0,  // x + 1
    Back = 1,   // x - 1
    Left = 2,   // z + 1
    Right = 3,  // z - 1
    Top = 4,    // y + 1
    Bottom = 5, // y - 1
}

impl<T> Index<Direction> for [T; 6] {
    type Output = T;

    fn index(&self, index: Direction) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> IndexMut<Direction> for [T; 6] {
    fn index_mut(&mut self, index: Direction) -> &mut Self::Output {
        &mut self[index as usize]
    }
}
impl Direction {
    pub fn get_face_rotation(&self) -> Quat {
        match self {
            Direction::Front => Quat::from_axis_angle(Vec3::Y, PI / 2.0),
            Direction::Back => Quat::from_axis_angle(Vec3::Y, -PI / 2.0),
            Direction::Top => Quat::from_axis_angle(Vec3::X, -PI / 2.0),
            Direction::Bottom => Quat::from_axis_angle(Vec3::X, PI / 2.0),
            Direction::Left => Quat::from_axis_angle(Vec3::Y, 0.0),
            Direction::Right => Quat::from_axis_angle(Vec3::Y, PI),
        }
    }

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Front => Direction::Back,
            Direction::Back => Direction::Front,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Top => Direction::Bottom,
            Direction::Bottom => Direction::Top,
        }
    }
}
