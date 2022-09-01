use std::f32::consts::PI;

use crate::{
    block::Block,
    material::{ATTRIBUTE_TEXTURE_INDEX, CUSTOM_NORMAL, CUSTOM_UV},
};
use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};

use crate::{Chunk, ChunkDirection, BLOCK_SIZE, CHUNK_SIZE};

#[derive(Default)]
pub struct MeshDescription {
    verts: Vec<Vec3>,
    normals: Vec<[u8; 2]>,
    uvs: Vec<[u8; 2]>,
    texture_indices: Vec<u32>,
    vert_indicies: Vec<usize>,
}

pub fn create_chunk_mesh(chunk: &Chunk) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut description = MeshDescription::default();
    create_mesh_faces(chunk, &mut description);

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
            .verts
            .iter()
            .map(|vec| vec.to_array())
            .collect::<Vec<[f32; 3]>>(),
    );

    mesh.insert_attribute(CUSTOM_NORMAL, VertexAttributeValues::Uint8x2(description.normals));

    mesh.insert_attribute(CUSTOM_UV, VertexAttributeValues::Uint8x2(description.uvs));

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

#[derive(Default, Copy, Clone)]
pub struct Sheet {
    blocks: [[Block; CHUNK_SIZE]; CHUNK_SIZE],
}

fn create_greedy_face(block: Block, x: usize, y:usize, width:usize, height: usize, desc: &mut MeshDescription) {
    //TODO
}

fn greedy(sheet: &Sheet) -> MeshDescription {
    let mut desc = MeshDescription::default();
    let mut finished = [[false; CHUNK_SIZE]; CHUNK_SIZE];

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            finished[x][y] = !sheet.blocks[x][y].is_filled();
        }
    }

    //gross
    while finished.iter().flatten().collect::<Vec<&bool>>().contains(&&false) {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                if finished[x][y] == false {
                    //Starting point, walk x to get width
                    let start = sheet.blocks[x][y];
                    let mut width = 1;
                    for w in x + 1..CHUNK_SIZE {
                        if start == sheet.blocks[x + w][y] {
                            width += 1;
                        } else {
                            break;
                        }
                    }
                    //Now walk y to get height
                    let mut height = 1;
                    for h in y + 1..CHUNK_SIZE {
                        let mut all_same = true;
                        for w in x + 1..CHUNK_SIZE {
                            if start != sheet.blocks[x + w][y + h] {
                                all_same = false;
                            }
                        }
                        if all_same {
                            height += 1;
                        } else {
                            break;
                        }
                    }
                    //Time to make the rect and mark finished
                    for u in x..x + width {
                        for v in y..y + height {
                            create_greedy_face(start, x, y, width, height, &mut desc);
                            finished[u][v] = true;
                        }
                    }
                }
            }
        }
    }

    desc
}

fn create_mesh_faces(chunk: &Chunk, mesh_description: &mut MeshDescription) {
    let mut top_slices = [Sheet::default(); CHUNK_SIZE];
    for z in 0..CHUNK_SIZE as isize {
        for y in 0..CHUNK_SIZE as isize {
            for x in 0..CHUNK_SIZE as isize {
                let current_block = chunk.get_block(x, y, z).unwrap();
                let [front_block, back_block, left_block, right_block, top_block, bottom_block] =
                    chunk.get_block_neighbors(x, y, z);
                if current_block.is_filled() && (top_block.is_none() || !top_block.unwrap().is_filled()) {
                    //Place top neighbor
                    top_slices[z as usize].blocks[x as usize][y as usize] = current_block;
                }
            }
        }
        let desc = greedy(&top_slices[z as usize]);
    }
}

fn create_mesh_faces_old(chunk: &Chunk, mesh_description: &mut MeshDescription) {
    for z in 0..CHUNK_SIZE as isize {
        for y in 0..CHUNK_SIZE as isize {
            for x in 0..CHUNK_SIZE as isize {
                let current_block = chunk.get_block(x, y, z).unwrap();
                let [front_block, back_block, left_block, right_block, top_block, bottom_block] =
                    chunk.get_block_neighbors(x, y, z);
                if current_block.is_filled() && (front_block.is_none() || !front_block.unwrap().is_filled()) {
                    add_face(
                        mesh_description,
                        Quat::from_axis_angle(Vec3::Y, PI / 2.0),
                        Vec3::new(
                            (x as f32 + 0.5) * BLOCK_SIZE,
                            y as f32 * BLOCK_SIZE,
                            (z as f32 - 0.5) * BLOCK_SIZE,
                        ),
                        current_block.get_face_index(ChunkDirection::Front),
                    );
                }
                //Back
                if current_block.is_filled() && (back_block.is_none() || !back_block.unwrap().is_filled()) {
                    add_face(
                        mesh_description,
                        Quat::from_axis_angle(Vec3::Y, -PI / 2.0),
                        Vec3::new(
                            (x as f32 - 0.5) * BLOCK_SIZE,
                            y as f32 * BLOCK_SIZE,
                            (z as f32 - 0.5) * BLOCK_SIZE,
                        ),
                        current_block.get_face_index(ChunkDirection::Back),
                    );
                }
                //Top
                //TODO Y neighbors are untested
                if current_block.is_filled() && (top_block.is_none() || !top_block.unwrap().is_filled()) {
                    add_face(
                        mesh_description,
                        Quat::from_axis_angle(Vec3::X, -PI / 2.0),
                        Vec3::new(
                            x as f32 * BLOCK_SIZE,
                            (y as f32 + 0.5) * BLOCK_SIZE,
                            (z as f32 - 0.5) * BLOCK_SIZE,
                        ),
                        current_block.get_face_index(ChunkDirection::Top),
                    );
                }
                //Bottom
                if current_block.is_filled() && (bottom_block.is_none() || !bottom_block.unwrap().is_filled()) {
                    add_face(
                        mesh_description,
                        Quat::from_axis_angle(Vec3::X, PI / 2.0),
                        Vec3::new(
                            x as f32 * BLOCK_SIZE,
                            (y as f32 - 0.5) * BLOCK_SIZE,
                            (z as f32 - 0.5) * BLOCK_SIZE,
                        ),
                        current_block.get_face_index(ChunkDirection::Bottom),
                    );
                }
                //Left
                if current_block.is_filled() && (left_block.is_none() || !left_block.unwrap().is_filled()) {
                    add_face(
                        mesh_description,
                        Quat::from_axis_angle(Vec3::Y, 0.0),
                        Vec3::new((x as f32) * BLOCK_SIZE, y as f32 * BLOCK_SIZE, (z as f32) * BLOCK_SIZE),
                        current_block.get_face_index(ChunkDirection::Left),
                    );
                }
                //Right
                if current_block.is_filled() && (right_block.is_none() || !right_block.unwrap().is_filled()) {
                    add_face(
                        mesh_description,
                        Quat::from_axis_angle(Vec3::Y, PI),
                        Vec3::new(
                            (x as f32) * BLOCK_SIZE,
                            y as f32 * BLOCK_SIZE,
                            (z as f32 - 1.0) * BLOCK_SIZE,
                        ),
                        current_block.get_face_index(ChunkDirection::Right),
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

    //let new_uvs = [
    //Vec2::new(0.01, 0.99),
    //Vec2::new(0.99, 0.99),
    //Vec2::new(0.99, 0.01),
    //Vec2::new(0.01, 0.01),
    //];
    let new_uvs = [[0, 1], [1, 1], [1, 0], [0, 0]];

    new_verts
        .iter_mut()
        .for_each(|vec| *vec = (rotation * *vec) + transform);
    new_normals.iter_mut().for_each(|vec| *vec = rotation * *vec);

    let normals = new_normals
        .iter()
        .map(|norm| {
            if norm.x > 0.5 {
                [0, 1]
            } else if norm.x < 0.5 {
                [0, 2]
            } else if norm.y > 0.5 {
                [0, 3]
            } else if norm.y < 0.5 {
                [0, 4]
            } else if norm.z > 0.5 {
                [0, 5]
            } else if norm.z < 0.5 {
                [0, 6]
            } else {
                [0, 0]
            }
        })
        .collect::<Vec<[u8; 2]>>();

    let vert_start = mesh_description.verts.len();
    mesh_description.verts.extend_from_slice(&new_verts);
    mesh_description.normals.extend_from_slice(&normals);
    mesh_description.uvs.extend_from_slice(&new_uvs);

    mesh_description.texture_indices.extend_from_slice(&new_texture_indices);

    mesh_description
        .vert_indicies
        .extend_from_slice(&[vert_start, vert_start + 1, vert_start + 2]);
    mesh_description
        .vert_indicies
        .extend_from_slice(&[vert_start, vert_start + 2, vert_start + 3]);
}
