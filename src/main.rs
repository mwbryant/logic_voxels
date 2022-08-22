use std::ops::{Index, IndexMut};

use bevy::{prelude::*, window::PresentMode};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_inspector_egui::WorldInspectorPlugin;
use chunk_mesh_generation::create_chunk_mesh;
use noise::{NoiseFn, Perlin};

mod chunk_mesh_generation;

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
    cubes: [[[bool; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Default for Chunk {
    fn default() -> Chunk {
        Chunk {
            cubes: [[[false; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
        }
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1280.,
            height: 720.,
            title: "Bevy Template".to_string(),
            present_mode: PresentMode::Immediate,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(NoCameraPlayerPlugin)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_custom_mesh)
        .add_system(camera_follow)
        .run();
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
                chunk.cubes[x][y][z] = value >= (y as f32 / CHUNK_SIZE as f32) as f64 || y == 0;
            }
        }
    }
    chunk
}

#[allow(clippy::needless_range_loop)]
fn spawn_custom_mesh(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
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

            let mesh = create_chunk_mesh(&chunks[x][z], neighbors);

            commands.spawn_bundle(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.53, 0.53, 0.67).into()),
                transform: Transform::from_xyz(chunk_x, 0.0, chunk_z),
                ..default()
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
