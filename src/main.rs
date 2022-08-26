use rayon::iter::ParallelIterator;
use std::{
    borrow::Borrow,
    cell::RefCell,
    char::MAX,
    ops::{Index, IndexMut},
    rc::Rc,
    sync::{Arc, Mutex, RwLock, Weak},
};

use bevy::{asset::AssetServerSettings, prelude::*, render::texture::ImageSettings, window::PresentMode};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_inspector_egui::WorldInspectorPlugin;
use chunk_mesh_generation::create_chunk_mesh;
use material::CustomMaterial;
use noise::{NoiseFn, Perlin};
use rayon::prelude::IntoParallelIterator;

mod chunk_mesh_generation;
mod material;

#[derive(Component)]
pub struct FollowCamera;

pub const CHUNK_SIZE: usize = 24;
pub const WORLD_SIZE: usize = 24;
pub const MAX_CHUNK_UPDATES_PER_FRAME: usize = 10;
pub const BLOCK_SIZE: f32 = 0.5;

#[derive(Clone, Copy)]
pub enum ChunkDirection {
    Front = 0,  // x + 1
    Back = 1,   // x - 1
    Left = 2,   // z + 1
    Right = 3,  // z - 1
    Top = 4,    // y + 1
    Bottom = 5, // y - 1
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

type ChunkData = [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
#[derive(Component, Clone)]
pub struct ChunkComp {
    chunk: Arc<RwLock<Chunk>>,
}

//TODO serialize?
//PERF there has to be a more performant way of handling this
#[derive(Clone)]
pub struct Chunk {
    cubes: ChunkData,
    dirty: bool,
    neighbors: [Weak<RwLock<Chunk>>; 6],
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

fn update_dirty_chunks(mut chunks: Query<(&ChunkComp, &mut Handle<Mesh>)>, mut meshes: ResMut<Assets<Mesh>>) {
    //TODO all of this can be done in parallel except for adding mesh to assets
    //FIXME for now I'm just going to cap the number of chunk updates per frame
    let mut updates = 0;
    for (chunk, mut mesh) in &mut chunks {
        if chunk.chunk.read().unwrap().dirty {
            *mesh = meshes.add(create_chunk_mesh(&chunk.chunk.read().unwrap()));
            updates += 1;
            chunk.chunk.write().unwrap().dirty = false;
        }
        if updates > MAX_CHUNK_UPDATES_PER_FRAME {
            info!("too many dirty chunks");
            return;
        }
    }
}

fn update_dirt_sys(chunks: Query<&ChunkComp>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Space) {
        //for chunk in &chunks {
        //apply_function_to_blocks(&mut chunk.chunk.write().unwrap(), update_dirt);
        //}
        chunks.par_for_each(5, |chunk| {
            apply_function_to_blocks(chunk, update_dirt);
        });
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
        .add_system(update_dirt_sys)
        .add_system(update_dirty_chunks)
        .run();
}

fn apply_function_to_blocks<F>(chunk: &ChunkComp, mut function: F)
where
    F: FnMut(&mut Block, [Option<Block>; 6]) -> bool,
{
    for z in 0..CHUNK_SIZE as isize {
        for y in 0..CHUNK_SIZE as isize {
            for x in 0..CHUNK_SIZE as isize {
                let neighbors = chunk.chunk.read().unwrap().get_block_neighbors(x, y, z);
                //No deadlocks because only write when you know you are done reading
                if function(
                    &mut chunk.chunk.write().unwrap().cubes[x as usize][y as usize][z as usize],
                    neighbors,
                ) {
                    chunk.chunk.write().unwrap().dirty = true;
                }
            }
        }
    }
}

fn update_dirt(block: &mut Block, neighbors: [Option<Block>; 6]) -> bool {
    if matches!(block, Block::Grass) {
        if let Some(top) = neighbors[ChunkDirection::Top] {
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
                    chunk.cubes[x][y][z] = Block::Grass
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
    let chunks_to_spawn = WORLD_SIZE;
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

    //let mut arcs: [[Weak<_>; WORLD_SIZE]; WORLD_SIZE] = Default::default();
    let mut arcs = [(); WORLD_SIZE].map(|_| [(); WORLD_SIZE].map(|_| <Arc<_>>::default()));
    let mut weaks = [(); WORLD_SIZE].map(|_| [(); WORLD_SIZE].map(|_| <Weak<_>>::default()));
    //Create arcs
    for x in 0..chunks_to_spawn {
        for z in 0..chunks_to_spawn {
            let arc = Arc::new(RwLock::new(chunks[x][z].clone()));
            arcs[x][z] = arc;
            weaks[x][z] = Arc::downgrade(&arcs[x][z]);
        }
    }

    //link chunk neighbors
    for x in 0..chunks_to_spawn {
        for z in 0..chunks_to_spawn {
            if x != chunks_to_spawn - 1 {
                arcs[x][z].clone().write().unwrap().neighbors[ChunkDirection::Front] = weaks[x + 1][z].clone();
            }
            if x != 0 {
                arcs[x][z].clone().write().unwrap().neighbors[ChunkDirection::Back] = weaks[x - 1][z].clone();
            }
            if z != 0 {
                arcs[x][z].clone().write().unwrap().neighbors[ChunkDirection::Right] = weaks[x][z - 1].clone();
            }
            if z != chunks_to_spawn - 1 {
                arcs[x][z].clone().write().unwrap().neighbors[ChunkDirection::Left] = weaks[x][z + 1].clone();
            }
        }
    }

    for x in 0..chunks_to_spawn {
        for z in 0..chunks_to_spawn {
            let chunk_x = x as f32 * CHUNK_SIZE as f32 * BLOCK_SIZE;
            let chunk_z = z as f32 * CHUNK_SIZE as f32 * BLOCK_SIZE;
            commands
                .spawn_bundle(MaterialMeshBundle {
                    mesh: meshes.add(create_chunk_mesh(&chunks[x][z])),
                    //mesh: meshes.add(shape::Box::default().into()),
                    material: materials.add(CustomMaterial {
                        textures: server.load("test_texture.png"),
                    }),
                    transform: Transform::from_xyz(chunk_x, 0.0, chunk_z),

                    ..default()
                })
                .insert(ChunkComp {
                    chunk: arcs[x][z].clone(),
                });
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
