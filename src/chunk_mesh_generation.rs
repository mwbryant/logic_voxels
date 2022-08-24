use std::f32::consts::PI;

use crate::material::ATTRIBUTE_TEXTURE_INDEX;
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use crate::{Chunk, ChunkDirection, BLOCK_SIZE, CHUNK_SIZE};

pub fn create_chunk_mesh(chunk: &Chunk, neighbors: [Option<&Chunk>; 6]) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut verts = Vec::default();
    let mut normals = Vec::default();
    let mut uvs = Vec::default();
    let mut texture_indicies = Vec::default();
    let mut indicies = Vec::default();

    create_mesh_faces(
        chunk,
        neighbors,
        &mut verts,
        &mut normals,
        &mut uvs,
        &mut texture_indicies,
        &mut indicies,
    );

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        verts.iter().map(|vec| vec.to_array()).collect::<Vec<[f32; 3]>>(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        normals.iter().map(|vec| vec.to_array()).collect::<Vec<[f32; 3]>>(),
    );

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        uvs.iter().map(|vec| vec.to_array()).collect::<Vec<[f32; 2]>>(),
    );
    mesh.insert_attribute(ATTRIBUTE_TEXTURE_INDEX, texture_indicies);
    mesh.set_indices(Some(Indices::U32(
        indicies.iter().map(|usized| *usized as u32).collect::<Vec<u32>>(),
    )));
    mesh
}

//Clippy is angry but I am going to add more to the if clauses soon and the suggestions are less clear
#[allow(clippy::nonminimal_bool)]
fn create_mesh_faces(
    chunk: &Chunk,
    chunk_neighbors: [Option<&Chunk>; 6],
    verts: &mut Vec<Vec3>,
    normals: &mut Vec<Vec3>,
    uvs: &mut Vec<Vec2>,
    texture_indices: &mut Vec<u32>,
    indicies: &mut Vec<usize>,
) {
    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                //Front
                if (x != CHUNK_SIZE - 1 && chunk.cubes[x][y][z] && !chunk.cubes[x + 1][y][z])
                    || (x == CHUNK_SIZE - 1
                        && chunk.cubes[x][y][z]
                        && (chunk_neighbors[ChunkDirection::Front].is_none()
                            || !chunk_neighbors[ChunkDirection::Front].unwrap().cubes[0][y][z]))
                {
                    add_face(
                        verts,
                        normals,
                        uvs,
                        texture_indices,
                        indicies,
                        Quat::from_axis_angle(Vec3::Y, PI / 2.0),
                        Vec3::new(
                            (x as f32 + 0.5) * BLOCK_SIZE,
                            y as f32 * BLOCK_SIZE,
                            (z as f32 - 0.5) * BLOCK_SIZE,
                        ),
                        1,
                    );
                }
                //Back
                if (x != 0 && chunk.cubes[x][y][z] && !chunk.cubes[x - 1][y][z])
                    || (x == 0 && chunk.cubes[x][y][z])
                        && (chunk_neighbors[ChunkDirection::Back].is_none()
                            || !chunk_neighbors[ChunkDirection::Back].unwrap().cubes[CHUNK_SIZE - 1][y][z])
                {
                    add_face(
                        verts,
                        normals,
                        uvs,
                        texture_indices,
                        indicies,
                        Quat::from_axis_angle(Vec3::Y, -PI / 2.0),
                        Vec3::new(
                            (x as f32 - 0.5) * BLOCK_SIZE,
                            y as f32 * BLOCK_SIZE,
                            (z as f32 - 0.5) * BLOCK_SIZE,
                        ),
                        1,
                    );
                }
                //Top
                //TODO Y neighbors are untested
                if (y != CHUNK_SIZE - 1 && chunk.cubes[x][y][z] && !chunk.cubes[x][y + 1][z])
                    || (y == CHUNK_SIZE - 1 && chunk.cubes[x][y][z])
                        && (chunk_neighbors[ChunkDirection::Top].is_none()
                            || !chunk_neighbors[ChunkDirection::Top].unwrap().cubes[x][0][z])
                {
                    add_face(
                        verts,
                        normals,
                        uvs,
                        texture_indices,
                        indicies,
                        Quat::from_axis_angle(Vec3::X, -PI / 2.0),
                        Vec3::new(
                            x as f32 * BLOCK_SIZE,
                            (y as f32 + 0.5) * BLOCK_SIZE,
                            (z as f32 - 0.5) * BLOCK_SIZE,
                        ),
                        0,
                    );
                }
                //Bottom
                if (y != 0 && chunk.cubes[x][y][z] && !chunk.cubes[x][y - 1][z])
                    || (y == 0 && chunk.cubes[x][y][z])
                        && (chunk_neighbors[ChunkDirection::Bottom].is_none()
                            || !chunk_neighbors[ChunkDirection::Bottom].unwrap().cubes[x][CHUNK_SIZE - 1][z])
                {
                    add_face(
                        verts,
                        normals,
                        uvs,
                        texture_indices,
                        indicies,
                        Quat::from_axis_angle(Vec3::X, PI / 2.0),
                        Vec3::new(
                            x as f32 * BLOCK_SIZE,
                            (y as f32 - 0.5) * BLOCK_SIZE,
                            (z as f32 - 0.5) * BLOCK_SIZE,
                        ),
                        2,
                    );
                }
                //Left
                if (z != CHUNK_SIZE - 1 && chunk.cubes[x][y][z] && !chunk.cubes[x][y][z + 1])
                    || (z == CHUNK_SIZE - 1 && chunk.cubes[x][y][z])
                        && (chunk_neighbors[ChunkDirection::Left].is_none()
                            || !chunk_neighbors[ChunkDirection::Left].unwrap().cubes[x][y][0])
                {
                    add_face(
                        verts,
                        normals,
                        uvs,
                        texture_indices,
                        indicies,
                        Quat::from_axis_angle(Vec3::Y, 0.0),
                        Vec3::new((x as f32) * BLOCK_SIZE, y as f32 * BLOCK_SIZE, (z as f32) * BLOCK_SIZE),
                        1,
                    );
                }
                //Right
                if (z != 0 && chunk.cubes[x][y][z] && !chunk.cubes[x][y][z - 1])
                    || (z == 0 && chunk.cubes[x][y][z])
                        && (chunk_neighbors[ChunkDirection::Right].is_none()
                            || !chunk_neighbors[ChunkDirection::Right].unwrap().cubes[x][y][CHUNK_SIZE - 1])
                {
                    add_face(
                        verts,
                        normals,
                        uvs,
                        texture_indices,
                        indicies,
                        Quat::from_axis_angle(Vec3::Y, PI),
                        Vec3::new(
                            (x as f32) * BLOCK_SIZE,
                            y as f32 * BLOCK_SIZE,
                            (z as f32 - 1.0) * BLOCK_SIZE,
                        ),
                        1,
                    );
                }
            }
        }
    }
}

fn add_face(
    vertices: &mut Vec<Vec3>,
    normals: &mut Vec<Vec3>,
    uvs: &mut Vec<Vec2>,
    texture_indices: &mut Vec<u32>,
    indicies: &mut Vec<usize>,
    rotation: Quat,
    transform: Vec3,
    face_index: u32,
) {
    let mut new_verts = [
        Vec3::new(-BLOCK_SIZE / 2.0, -BLOCK_SIZE / 2.0, 0.0),
        Vec3::new(BLOCK_SIZE / 2.0, -BLOCK_SIZE / 2.0, 0.0),
        Vec3::new(BLOCK_SIZE / 2.0, BLOCK_SIZE / 2.0, 0.0),
        Vec3::new(-BLOCK_SIZE / 2.0, BLOCK_SIZE / 2.0, 0.0),
    ];
    let mut new_normals = [
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
    ];

    let new_texture_indices = [face_index; 4];

    let new_uvs = [
        Vec2::new(0.01, 0.01),
        Vec2::new(0.99, 0.01),
        Vec2::new(0.99, 0.99),
        Vec2::new(0.01, 0.99),
    ];

    new_verts
        .iter_mut()
        .for_each(|vec| *vec = (rotation * *vec) + transform);
    new_normals.iter_mut().for_each(|vec| *vec = rotation * *vec);
    //info!("{}", new_normals[1]);

    let vert_start = vertices.len();
    vertices.extend_from_slice(&new_verts);
    normals.extend_from_slice(&new_normals);
    uvs.extend_from_slice(&new_uvs);
    texture_indices.extend_from_slice(&new_texture_indices);

    indicies.extend_from_slice(&[vert_start, vert_start + 1, vert_start + 2]);
    indicies.extend_from_slice(&[vert_start, vert_start + 2, vert_start + 3]);
}
