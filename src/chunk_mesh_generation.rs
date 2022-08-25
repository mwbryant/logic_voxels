use std::f32::consts::PI;

use crate::material::ATTRIBUTE_TEXTURE_INDEX;
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use crate::{Chunk, ChunkDirection, BLOCK_SIZE, CHUNK_SIZE};

#[derive(Default)]
pub struct MeshDescription {
    verts: Vec<Vec3>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
    texture_indices: Vec<u32>,
    vert_indicies: Vec<usize>,
}

pub fn create_chunk_mesh(chunk: &Chunk) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut description = MeshDescription::default();
    create_mesh_faces(chunk, neighbors, &mut description);

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        description
            .verts
            .iter()
            .map(|vec| vec.to_array())
            .collect::<Vec<[f32; 3]>>(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        description
            .normals
            .iter()
            .map(|vec| vec.to_array())
            .collect::<Vec<[f32; 3]>>(),
    );

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        description
            .uvs
            .iter()
            .map(|vec| vec.to_array())
            .collect::<Vec<[f32; 2]>>(),
    );
    mesh.insert_attribute(ATTRIBUTE_TEXTURE_INDEX, description.texture_indices);
    mesh.set_indices(Some(Indices::U32(
        description
            .vert_indicies
            .iter()
            .map(|usized| *usized as u32)
            .collect::<Vec<u32>>(),
    )));
    mesh
}

//Clippy is angry but I am going to add more to the if clauses soon and the suggestions are less clear
#[allow(clippy::nonminimal_bool)]
fn create_mesh_faces(chunk: &Chunk, chunk_neighbors: [Option<&Chunk>; 6], mesh_description: &mut MeshDescription) {
    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                //Front
                if (x != CHUNK_SIZE - 1
                    && chunk.cubes.clone().read().unwrap()[x][y][z].is_filled()
                    && !chunk.cubes.clone().read().unwrap()[x + 1][y][z].is_filled())
                    || (x == CHUNK_SIZE - 1
                        && chunk.cubes.clone().read().unwrap()[x][y][z].is_filled()
                        && (chunk_neighbors[ChunkDirection::Front].is_none()
                            || !chunk_neighbors[ChunkDirection::Front]
                                .unwrap()
                                .cubes
                                .clone()
                                .read()
                                .unwrap()[0][y][z]
                                .is_filled()))
                {
                    add_face(
                        mesh_description,
                        Quat::from_axis_angle(Vec3::Y, PI / 2.0),
                        Vec3::new(
                            (x as f32 + 0.5) * BLOCK_SIZE,
                            y as f32 * BLOCK_SIZE,
                            (z as f32 - 0.5) * BLOCK_SIZE,
                        ),
                        chunk.cubes.clone().read().unwrap()[x][y][z].get_face_index(ChunkDirection::Front),
                    );
                }
                //Back
                if (x != 0
                    && chunk.cubes.clone().read().unwrap()[x][y][z].is_filled()
                    && !chunk.cubes.clone().read().unwrap()[x - 1][y][z].is_filled())
                    || (x == 0 && chunk.cubes.clone().read().unwrap()[x][y][z].is_filled())
                        && (chunk_neighbors[ChunkDirection::Back].is_none()
                            || !chunk_neighbors[ChunkDirection::Back]
                                .unwrap()
                                .cubes
                                .clone()
                                .read()
                                .unwrap()[CHUNK_SIZE - 1][y][z]
                                .is_filled())
                {
                    add_face(
                        mesh_description,
                        Quat::from_axis_angle(Vec3::Y, -PI / 2.0),
                        Vec3::new(
                            (x as f32 - 0.5) * BLOCK_SIZE,
                            y as f32 * BLOCK_SIZE,
                            (z as f32 - 0.5) * BLOCK_SIZE,
                        ),
                        chunk.cubes.clone().read().unwrap()[x][y][z].get_face_index(ChunkDirection::Back),
                    );
                }
                //Top
                //TODO Y neighbors are untested
                if (y != CHUNK_SIZE - 1
                    && chunk.cubes.clone().read().unwrap()[x][y][z].is_filled()
                    && !chunk.cubes.clone().read().unwrap()[x][y + 1][z].is_filled())
                    || (y == CHUNK_SIZE - 1 && chunk.cubes.clone().read().unwrap()[x][y][z].is_filled())
                        && (chunk_neighbors[ChunkDirection::Top].is_none()
                            || !chunk_neighbors[ChunkDirection::Top]
                                .unwrap()
                                .cubes
                                .clone()
                                .read()
                                .unwrap()[x][0][z]
                                .is_filled())
                {
                    add_face(
                        mesh_description,
                        Quat::from_axis_angle(Vec3::X, -PI / 2.0),
                        Vec3::new(
                            x as f32 * BLOCK_SIZE,
                            (y as f32 + 0.5) * BLOCK_SIZE,
                            (z as f32 - 0.5) * BLOCK_SIZE,
                        ),
                        chunk.cubes.clone().read().unwrap()[x][y][z].get_face_index(ChunkDirection::Top),
                    );
                }
                //Bottom
                if (y != 0
                    && chunk.cubes.clone().read().unwrap()[x][y][z].is_filled()
                    && !chunk.cubes.clone().read().unwrap()[x][y - 1][z].is_filled())
                    || (y == 0 && chunk.cubes.clone().read().unwrap()[x][y][z].is_filled())
                        && (chunk_neighbors[ChunkDirection::Bottom].is_none()
                            || !chunk_neighbors[ChunkDirection::Bottom]
                                .unwrap()
                                .cubes
                                .clone()
                                .read()
                                .unwrap()[x][CHUNK_SIZE - 1][z]
                                .is_filled())
                {
                    add_face(
                        mesh_description,
                        Quat::from_axis_angle(Vec3::X, PI / 2.0),
                        Vec3::new(
                            x as f32 * BLOCK_SIZE,
                            (y as f32 - 0.5) * BLOCK_SIZE,
                            (z as f32 - 0.5) * BLOCK_SIZE,
                        ),
                        chunk.cubes.clone().read().unwrap()[x][y][z].get_face_index(ChunkDirection::Bottom),
                    );
                }
                //Left
                if (z != CHUNK_SIZE - 1
                    && chunk.cubes.clone().read().unwrap()[x][y][z].is_filled()
                    && !chunk.cubes.clone().read().unwrap()[x][y][z + 1].is_filled())
                    || (z == CHUNK_SIZE - 1 && chunk.cubes.clone().read().unwrap()[x][y][z].is_filled())
                        && (chunk_neighbors[ChunkDirection::Left].is_none()
                            || !chunk_neighbors[ChunkDirection::Left]
                                .unwrap()
                                .cubes
                                .clone()
                                .read()
                                .unwrap()[x][y][0]
                                .is_filled())
                {
                    add_face(
                        mesh_description,
                        Quat::from_axis_angle(Vec3::Y, 0.0),
                        Vec3::new((x as f32) * BLOCK_SIZE, y as f32 * BLOCK_SIZE, (z as f32) * BLOCK_SIZE),
                        chunk.cubes.clone().read().unwrap()[x][y][z].get_face_index(ChunkDirection::Left),
                    );
                }
                //Right
                if (z != 0
                    && chunk.cubes.clone().read().unwrap()[x][y][z].is_filled()
                    && !chunk.cubes.clone().read().unwrap()[x][y][z - 1].is_filled())
                    || (z == 0 && chunk.cubes.clone().read().unwrap()[x][y][z].is_filled())
                        && (chunk_neighbors[ChunkDirection::Right].is_none()
                            || !chunk_neighbors[ChunkDirection::Right]
                                .unwrap()
                                .cubes
                                .clone()
                                .read()
                                .unwrap()[x][y][CHUNK_SIZE - 1]
                                .is_filled())
                {
                    add_face(
                        mesh_description,
                        Quat::from_axis_angle(Vec3::Y, PI),
                        Vec3::new(
                            (x as f32) * BLOCK_SIZE,
                            y as f32 * BLOCK_SIZE,
                            (z as f32 - 1.0) * BLOCK_SIZE,
                        ),
                        chunk.cubes.clone().read().unwrap()[x][y][z].get_face_index(ChunkDirection::Right),
                    );
                }
            }
        }
    }
}

fn add_face(mesh_description: &mut MeshDescription, rotation: Quat, transform: Vec3, face_index: u32) {
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
        Vec2::new(0.01, 0.99),
        Vec2::new(0.99, 0.99),
        Vec2::new(0.99, 0.01),
        Vec2::new(0.01, 0.01),
    ];

    new_verts
        .iter_mut()
        .for_each(|vec| *vec = (rotation * *vec) + transform);
    new_normals.iter_mut().for_each(|vec| *vec = rotation * *vec);
    //info!("{}", new_normals[1]);

    let vert_start = mesh_description.verts.len();
    mesh_description.verts.extend_from_slice(&new_verts);
    mesh_description.normals.extend_from_slice(&new_normals);
    mesh_description.uvs.extend_from_slice(&new_uvs);
    mesh_description.texture_indices.extend_from_slice(&new_texture_indices);

    mesh_description
        .vert_indicies
        .extend_from_slice(&[vert_start, vert_start + 1, vert_start + 2]);
    mesh_description
        .vert_indicies
        .extend_from_slice(&[vert_start, vert_start + 2, vert_start + 3]);
}
