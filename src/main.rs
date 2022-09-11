#![allow(clippy::too_many_arguments)]
use chunk_loading::{initial_chunk_spawning, spawn_chunk_meshes, LoadedChunks};
use direction::Direction;

use bevy::{
    asset::AssetServerSettings,
    pbr::wireframe::WireframePlugin,
    prelude::*,
    render::{
        render_resource::{AddressMode, FilterMode, SamplerDescriptor},
        texture::ImageSettings,
    },
    window::PresentMode,
};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_inspector_egui::WorldInspectorPlugin;
use block::Block;
use chunk::{Chunk, ChunkComp};
use chunk_mesh_generation::create_chunk_mesh;
use material::{create_array_texture, CustomMaterial};

mod block;
mod chunk;
mod chunk_loading;
mod chunk_mesh_generation;
mod direction;
mod material;

#[derive(Component)]
pub struct FollowCamera;

pub const CHUNK_SIZE: usize = 16;
pub const WORLD_SIZE: usize = 20;
pub const MAX_CHUNK_UPDATES_PER_FRAME: usize = 30;
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

fn click_detection(mouse: Res<Input<MouseButton>>, transform: Query<&Transform, With<Camera3d>>) {
    let transform = transform.single();
    let range = 5.0;
    if mouse.just_pressed(MouseButton::Left) {
        info!("Looking toward {:?}, {:?}", transform.translation, transform.forward());
        // https://www.geeksforgeeks.org/bresenhams-algorithm-for-3-d-line-drawing/
        let end = transform.translation + transform.forward() * range;
        let start = transform.translation;
        let mut current = transform.translation;
        let dir = transform.forward();
        let xs = if dir.x > 0.0 { 1.0 } else { -1.0 };
        let ys = if dir.y > 0.0 { 1.0 } else { -1.0 };
        let zs = if dir.z > 0.0 { 1.0 } else { -1.0 };
        let diff = end - current;
        let diff = diff.abs();

        //X is driving
        if diff.x > diff.y && diff.x > diff.z {
            let mut p1 = 2. * diff.y - diff.x;
            let mut p2 = 2. * diff.z - diff.x;
            while (start.x < end.x && current.x < end.x) || (start.x > end.x && current.x > end.x) {
                //FIXME blocksize?
                current.x += xs;
                if p1 >= 0. {
                    current.y += ys;
                    p1 -= 2. * diff.x;
                }
                if p2 >= 0. {
                    current.z += zs;
                    p2 -= 2. * diff.x;
                }
                p1 += 2. * diff.y;
                p2 += 2. * diff.z;
                info!("Checking {}", current.as_ivec3());
            }
        //Y driving
        } else if diff.y > diff.z && diff.y > diff.x {
            let mut p1 = 2. * diff.x - diff.y;
            let mut p2 = 2. * diff.z - diff.y;
            while (start.y < end.y && current.y < end.y) || (start.y > end.y && current.y > end.y) {
                //FIXME blocksize?
                current.y += ys;
                if p1 >= 0. {
                    current.x += xs;
                    p1 -= 2. * diff.y;
                }
                if p2 >= 0. {
                    current.z += zs;
                    p2 -= 2. * diff.y;
                }
                p1 += 2. * diff.x;
                p2 += 2. * diff.z;
                info!("Checking {}", current.as_ivec3());
            }
        //Z driving
        } else {
            let mut p1 = 2. * diff.x - diff.z;
            let mut p2 = 2. * diff.y - diff.z;
            while (start.z < end.z && current.z < end.z) || (start.z > end.z && current.z > end.z) {
                //FIXME blocksize?
                current.z += zs;
                if p1 >= 0. {
                    current.x += xs;
                    p1 -= 2. * diff.z;
                }
                if p2 >= 0. {
                    current.y += ys;
                    p2 -= 2. * diff.z;
                }
                p1 += 2. * diff.x;
                p2 += 2. * diff.y;
                info!("Checking {}", current.as_ivec3());
            }
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ImageSettings {
            default_sampler: SamplerDescriptor {
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
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
        //.add_plugin(WorldInspectorPlugin::default())
        .add_plugin(WireframePlugin)
        .add_plugin(NoCameraPlayerPlugin)
        .add_startup_system(spawn_camera)
        .add_system(create_array_texture)
        .add_system(spawn_chunk_meshes)
        .add_system(click_detection)
        .init_resource::<LoadedChunks>()
        .add_startup_system_to_stage(StartupStage::PreStartup, load_chunk_texture)
        .add_startup_system(initial_chunk_spawning)
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
        if let Some(top) = neighbors[Direction::Top] {
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

pub struct ChunkTexture(pub Handle<Image>);

fn load_chunk_texture(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(ChunkTexture(server.load("array_test.png")));
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
