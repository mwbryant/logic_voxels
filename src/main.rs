#![allow(clippy::too_many_arguments)]

use chunk_loading::{initial_chunk_spawning, spawn_chunk_meshes, LoadedChunks};
use chunk_updating::{update_dirt, update_dirt_sys, update_dirty_chunks};
use direction::Direction;

use bevy::{
    asset::AssetServerSettings,
    pbr::wireframe::WireframePlugin,
    prelude::*,
    render::{
        primitives::Aabb,
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
mod chunk_updating;
mod direction;
mod material;

#[derive(Component)]
pub struct FollowCamera;

pub const CHUNK_SIZE: usize = 16;
pub const WORLD_SIZE: usize = 20;
pub const MAX_CHUNK_UPDATES_PER_FRAME: usize = 30;
pub const BLOCK_SIZE: f32 = 1.0;

fn click_detection(
    mouse: Res<Input<MouseButton>>,
    transform: Query<&Transform, With<Camera3d>>,
    loaded_chunks: Res<LoadedChunks>,
    comps: Query<&ChunkComp>,
) {
    let transform = transform.single();
    let range = 9.0;
    if mouse.just_pressed(MouseButton::Left) {
        let end = transform.translation + transform.forward() * range;
        let mut current = transform.translation;

        let diff = end - current;

        let steps = diff.abs().max_element().ceil() * 5.0;

        let inc = diff / steps;

        let size = CHUNK_SIZE as i32;

        for _i in 0..(steps as usize) {
            let block_pos = current - Vec3::ONE / 2.0;
            let pos = block_pos.round().as_ivec3();

            let offset = IVec3::new(pos.x.rem_euclid(size), pos.y.rem_euclid(size), pos.z.rem_euclid(size));
            let x = if pos.x >= 0 { pos.x / size } else { pos.x / size - 1 };
            let y = if pos.y >= 0 { pos.y / size } else { pos.y / size - 1 };
            let z = if pos.z >= 0 { pos.z / size } else { pos.z / size - 1 };
            let chunk_pos = IVec3::new(x, y, z);
            if let Some(chunk) = loaded_chunks.ent_map.get(&chunk_pos) {
                let chunk = comps.get(*chunk).unwrap();

                if chunk.read_block(offset) != Block::Air {
                    chunk.write_block(offset, Block::Air);
                    //Rewind for placement
                    let block_pos = current - Vec3::ONE / 2.0 - inc;
                    let pos = block_pos.round().as_ivec3();

                    let offset = IVec3::new(pos.x.rem_euclid(size), pos.y.rem_euclid(size), pos.z.rem_euclid(size));
                    let x = if pos.x >= 0 { pos.x / size } else { pos.x / size - 1 };
                    let y = if pos.y >= 0 { pos.y / size } else { pos.y / size - 1 };
                    let z = if pos.z >= 0 { pos.z / size } else { pos.z / size - 1 };
                    let chunk_pos = IVec3::new(x, y, z);
                    if let Some(chunk) = loaded_chunks.ent_map.get(&chunk_pos) {
                        let chunk = comps.get(*chunk).unwrap();
                        chunk.write_block(offset, Block::Red);
                    }
                    return;
                }
            }

            current += inc;
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
        .add_plugin(WorldInspectorPlugin::default())
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
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|commands| {
            commands.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(10.0), Val::Px(10.0)),
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                color: Color::RED.into(),
                ..default()
            });
        });

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
