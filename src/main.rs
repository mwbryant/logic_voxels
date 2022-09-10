use std::sync::{Arc, RwLock, Weak};

use bevy::{
    asset::AssetServerSettings,
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
    render::{
        render_resource::{AddressMode, FilterMode, SamplerDescriptor},
        texture::{ImageSampler, ImageSettings},
    },
    window::PresentMode,
};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_inspector_egui::WorldInspectorPlugin;
use block::Block;
use chunk::{Chunk, ChunkComp, ChunkDirection};
use chunk_mesh_generation::create_chunk_mesh;
use material::{create_array_texture, CustomMaterial};
use noise::{NoiseFn, Perlin};

mod block;
mod chunk;
mod chunk_mesh_generation;
mod material;

#[derive(Component)]
pub struct FollowCamera;

pub const CHUNK_SIZE: usize = 16;
pub const WORLD_SIZE: usize = 15;
pub const MAX_CHUNK_UPDATES_PER_FRAME: usize = 10;
pub const BLOCK_SIZE: f32 = 1.0;

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
            return;
        }
    }
}

fn update_dirt_sys(chunks: Query<&ChunkComp>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Space) {
        chunks.par_for_each(5, |chunk| {
            apply_function_to_blocks(chunk, update_dirt);
        });
    }
}

fn main() {
    App::new()
        .insert_resource(ImageSettings {
            default_sampler: SamplerDescriptor {
                address_mode_u: AddressMode::Repeat,
                /// How to deal with out of bounds accesses in the v (i.e. y) direction
                address_mode_v: AddressMode::Repeat,
                /// How to deal with out of bounds accesses in the w (i.e. z) direction
                address_mode_w: AddressMode::Repeat,
                mag_filter: FilterMode::Nearest,
                min_filter: FilterMode::Nearest,
                ..Default::default()
            },
        })
        .insert_resource(WindowDescriptor {
            width: 1280.,
            height: 720.,
            title: "Voxel Tests".to_string(),
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
        .add_plugin(WireframePlugin)
        .add_plugin(NoCameraPlayerPlugin)
        .add_startup_system(spawn_camera)
        .add_system(create_array_texture)
        .add_startup_system_to_stage(StartupStage::PreStartup, load_chunk_texture)
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

pub struct ChunkTexture(pub Handle<Image>);

fn load_chunk_texture(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(ChunkTexture(server.load("array_test.png")));
}

fn spawn_custom_mesh(
    mut commands: Commands,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    texture: Res<ChunkTexture>,
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

    info!("Part 1");

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

    info!("Part 2");

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

    info!("Part 3");

    for x in 0..chunks_to_spawn {
        for z in 0..chunks_to_spawn {
            let chunk_x = x as f32 * CHUNK_SIZE as f32 * BLOCK_SIZE;
            let chunk_z = z as f32 * CHUNK_SIZE as f32 * BLOCK_SIZE;
            commands
                .spawn_bundle(MaterialMeshBundle {
                    mesh: meshes.add(create_chunk_mesh(&chunks[x][z])),
                    //mesh: meshes.add(shape::Box::default().into()),
                    material: materials.add(CustomMaterial {
                        textures: texture.0.clone(),
                    }),
                    transform: Transform::from_xyz(chunk_x, 0.0, chunk_z),

                    ..default()
                })
                .insert(Wireframe)
                .insert(ChunkComp {
                    chunk: arcs[x][z].clone(),
                });
        }
    }
    info!("Done");
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
