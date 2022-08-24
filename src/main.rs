use std::{
    borrow::Borrow,
    cell::RefCell,
    ops::{Index, IndexMut},
};

use bevy::{asset::AssetServerSettings, prelude::*, render::texture::ImageSettings, window::PresentMode};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_inspector_egui::WorldInspectorPlugin;
use chunk_mesh_generation::create_chunk_mesh;
use material::CustomMaterial;
use noise::{NoiseFn, Perlin};

mod chunk_mesh_generation;
mod material;

#[derive(Component)]
pub struct FollowCamera;

pub const CHUNK_SIZE: usize = 24;
pub const BLOCK_SIZE: f32 = 0.3;

#[derive(Clone, Copy)]
pub enum ChunkDirection {
    Front = 0,
    Back = 1,
    Left = 2,
    Right = 3,
    Top = 4,
    Bottom = 5,
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

//TODO serialize?
pub struct Chunk {
    cubes: RefCell<[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
    dirty: RefCell<bool>,
}

impl Block {
    fn is_filled(&self) -> bool {
        !matches!(self, Block::Air)
    }

    fn get_face_index(&self, direction: ChunkDirection) -> u32 {
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

#[derive(Default, Clone, Copy, Debug)]
pub enum Block {
    #[default]
    Air,
    Grass,
    Dirt,
}

impl Default for Chunk {
    fn default() -> Chunk {
        Chunk {
            cubes: RefCell::new([[[Block::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]),
            dirty: RefCell::new(false),
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(WindowDescriptor {
            width: 1280.,
            height: 720.,
            title: "Bevy Template".to_string(),
            present_mode: PresentMode::Immediate,
            resizable: false,
            ..Default::default()
        })
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<CustomMaterial>::default())
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(NoCameraPlayerPlugin)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_custom_mesh)
        .add_system(camera_follow)
        .run();
}

fn apply_function_to_blocks<F>(chunk: &Chunk, neighbors: [Option<&Chunk>; 6], mut function: F)
where
    F: FnMut(&mut Block, [Option<Block>; 6]) -> bool,
{
    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let mut block_neighbors = [None; 6];
                //Front
                if x != CHUNK_SIZE - 1 {
                    block_neighbors[ChunkDirection::Front] = Some(chunk.cubes.borrow()[x + 1][y][z]);
                } else if let Some(front) = neighbors[ChunkDirection::Front] {
                    block_neighbors[ChunkDirection::Front] = Some(front.cubes.borrow()[0][y][z]);
                }
                //Back
                if x != 0 {
                    block_neighbors[ChunkDirection::Back] = Some(chunk.cubes.borrow()[x - 1][y][z]);
                } else if let Some(front) = neighbors[ChunkDirection::Back] {
                    block_neighbors[ChunkDirection::Back] = Some(front.cubes.borrow()[CHUNK_SIZE - 1][y][z]);
                }
                //Top
                if y != CHUNK_SIZE - 1 {
                    block_neighbors[ChunkDirection::Top] = Some(chunk.cubes.borrow()[x][y + 1][z]);
                } else if let Some(front) = neighbors[ChunkDirection::Top] {
                    block_neighbors[ChunkDirection::Top] = Some(front.cubes.borrow()[x][0][z]);
                }
                //warn!("Unfinished cases!");
                if function(&mut chunk.cubes.borrow_mut()[x][y][z], block_neighbors) {
                    *chunk.dirty.borrow_mut() = true;
                }
            }
        }
    }
}

fn update_dirt(block: &mut Block, neighbors: [Option<Block>; 6]) -> bool {
    if matches!(block, Block::Grass) {
        if let Some(top) = neighbors[ChunkDirection::Top] {
            //info!("Dirt {:?}", top);
            if !matches!(top, Block::Air) {
                *block = Block::Dirt;
                return true;
            }
        }
    }
    false
}

fn camera_follow(
    camera: Query<&Transform, With<Camera3d>>,
    mut followers: Query<&mut Transform, (With<FollowCamera>, Without<Camera3d>)>,
) {
    for mut follower in &mut followers {
        follower.translation = camera.single().translation;
    }
}

fn gen_chunk(chunk_x: f32, chunk_z: f32) -> Chunk {
    let mut chunk = Chunk::default();
    let perlin = Perlin::new();

    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let value = (perlin.get([
                    (x as f64 * BLOCK_SIZE as f64 + chunk_x as f64) / 21.912,
                    (z as f64 * BLOCK_SIZE as f64 + chunk_z as f64) / 23.253,
                ]) + 1.0)
                    / 2.0
                    + (0.12
                        * perlin.get([
                            (x as f64 * BLOCK_SIZE as f64 + chunk_x as f64) / 3.912,
                            (z as f64 * BLOCK_SIZE as f64 + chunk_z as f64) / 3.253,
                        ])
                        + 0.06);
                if value >= (y as f32 / CHUNK_SIZE as f32) as f64 || y == 0 {
                    chunk.cubes.borrow_mut()[x][y][z] = Block::Grass
                }
            }
        }
    }
    chunk
}

#[allow(clippy::needless_range_loop)]
fn spawn_custom_mesh(
    mut commands: Commands,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    server: Res<AssetServer>,
) {
    let chunks_to_spawn = 20;
    //FIXME dont use a vec for this
    let mut chunks: Vec<Vec<Chunk>> = Vec::default();

    for x in 0..chunks_to_spawn {
        chunks.push(Vec::default());
        for z in 0..chunks_to_spawn {
            let chunk_x = x as f32 * CHUNK_SIZE as f32 * BLOCK_SIZE;
            let chunk_z = z as f32 * CHUNK_SIZE as f32 * BLOCK_SIZE;
            let chunk = gen_chunk(chunk_x, chunk_z);
            chunks[x].push(chunk);
        }
    }

    for x in 0..chunks_to_spawn {
        for z in 0..chunks_to_spawn {
            let chunk_x = x as f32 * CHUNK_SIZE as f32 * BLOCK_SIZE;
            let chunk_z = z as f32 * CHUNK_SIZE as f32 * BLOCK_SIZE;
            let mut neighbors: [Option<&Chunk>; 6] = Default::default();

            if x != chunks_to_spawn - 1 {
                neighbors[ChunkDirection::Front] = Some(&chunks[x + 1][z]);
            }
            if x != 0 {
                neighbors[ChunkDirection::Back] = Some(&chunks[x - 1][z]);
            }
            if z != 0 {
                neighbors[ChunkDirection::Right] = Some(&chunks[x][z - 1]);
            }
            if z != chunks_to_spawn - 1 {
                neighbors[ChunkDirection::Left] = Some(&chunks[x][z + 1]);
            }
            //TESTING
            {
                let _span = info_span!("span_name", name = "span_name").entered();
                apply_function_to_blocks(&chunks[x][z], neighbors, update_dirt);
            }

            {
                let _span = info_span!("create_mesh", name = "create_mesh").entered();
                let mesh = create_chunk_mesh(&chunks[x][z], neighbors);

                commands.spawn_bundle(MaterialMeshBundle {
                    mesh: meshes.add(mesh),
                    material: materials.add(CustomMaterial {
                        textures: server.load("test_texture.png"),
                    }),
                    transform: Transform::from_xyz(chunk_x, 0.0, chunk_z),
                    ..default()
                });
            }
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-3.0, 15.5, -1.0).looking_at(Vec3::new(100.0, 0.0, 100.0), Vec3::Y),
            ..default()
        })
        .insert(FlyCam)
        .insert_bundle(VisibilityBundle::default())
        .with_children(|commands| {
            commands.spawn_bundle(SpotLightBundle {
                spot_light: SpotLight {
                    color: Color::WHITE,
                    intensity: 3000.0,
                    range: 200.0,
                    shadows_enabled: true,
                    outer_angle: 0.4,
                    ..default()
                },
                transform: Transform::from_xyz(-0.1, -0.0, 0.0),
                ..default()
            });
        });
    //directional 'sun' light
    const HALF_SIZE: f32 = 40.0;
    commands
        .spawn_bundle(DirectionalLightBundle {
            directional_light: DirectionalLight {
                // Configure the projection to better fit the scene
                shadow_projection: OrthographicProjection {
                    left: -HALF_SIZE,
                    right: HALF_SIZE,
                    bottom: -HALF_SIZE,
                    top: HALF_SIZE,
                    near: -10.0 * HALF_SIZE,
                    far: 10.0 * HALF_SIZE,
                    ..default()
                },
                shadows_enabled: false,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(30.0, 2.0, 0.0),
                rotation: Quat::from_euler(EulerRot::XYZ, 0.3, -2.6, 0.0),
                ..default()
            },
            ..default()
        })
        .insert(FollowCamera);
}
