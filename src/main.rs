#![allow(clippy::too_many_arguments)]

use bevy::{
    asset::AssetServerSettings,
    pbr::wireframe::WireframePlugin,
    render::{
        render_resource::{AddressMode, FilterMode, SamplerDescriptor},
        texture::ImageSettings,
    },
    window::PresentMode,
};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_inspector_egui::WorldInspectorPlugin;

use crate::prelude::*;
use material::{create_array_texture, CustomMaterial};

mod chunks;
mod material;
mod prelude;

#[derive(Component)]
pub struct FollowCamera;

pub struct ClickEvent {
    //TODO track held and stuff
    button: MouseButton,
    world_pos: IVec3,
    prev_pos: IVec3,
}

fn click_to_break(
    loaded_chunks: Res<LoadedChunks>,
    comps: Query<&ChunkComp>,
    mut click_reader: EventReader<ClickEvent>,
) {
    for ev in click_reader.iter() {
        if ev.button == MouseButton::Left {
            let (chunk_pos, offset) = Chunk::world_to_chunk(ev.world_pos);
            if let Some(chunk) = loaded_chunks.ent_map.get(&chunk_pos) {
                let chunk = comps.get(*chunk).unwrap();
                chunk.write_block(offset, Block::Air);
            }
        }
    }
}

fn click_to_place(
    loaded_chunks: Res<LoadedChunks>,
    comps: Query<&ChunkComp>,
    mut click_reader: EventReader<ClickEvent>,
) {
    for ev in click_reader.iter() {
        if ev.button == MouseButton::Right {
            let (chunk_pos, offset) = Chunk::world_to_chunk(ev.prev_pos);
            if let Some(chunk) = loaded_chunks.ent_map.get(&chunk_pos) {
                let chunk = comps.get(*chunk).unwrap();
                if chunk.read_block(offset) == Block::Air {
                    chunk.write_block(offset, Block::Red);
                }
            }
        }
    }
}

fn click_detection(
    mouse: Res<Input<MouseButton>>,
    transform: Query<&Transform, With<Camera3d>>,
    loaded_chunks: Res<LoadedChunks>,
    comps: Query<&ChunkComp>,
    mut click_writer: EventWriter<ClickEvent>,
) {
    let transform = transform.single();
    let range = 9.0;
    if mouse.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        let end = transform.translation + transform.forward() * range;
        let mut current = transform.translation;

        let diff = end - current;

        let steps = diff.abs().max_element().ceil() * 5.0;

        let inc = diff / steps;

        for _i in 0..(steps as usize) {
            let block_pos = current - Vec3::ONE / 2.0;
            let world_pos = block_pos.round().as_ivec3();
            let (chunk_pos, offset) = Chunk::world_to_chunk(world_pos);

            if let Some(chunk) = loaded_chunks.ent_map.get(&chunk_pos) {
                let chunk = comps.get(*chunk).unwrap();

                if chunk.read_block(offset) != Block::Air {
                    //Rewind for placement
                    let block_pos = current - Vec3::ONE / 2.0 - inc;
                    let prev_pos = block_pos.round().as_ivec3();
                    if mouse.just_pressed(MouseButton::Left) {
                        click_writer.send(ClickEvent {
                            button: MouseButton::Left,
                            world_pos,
                            prev_pos,
                        });
                    }
                    //gross
                    if mouse.just_pressed(MouseButton::Right) {
                        click_writer.send(ClickEvent {
                            button: MouseButton::Right,
                            world_pos,
                            prev_pos,
                        });
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
        .add_event::<ClickEvent>()
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<CustomMaterial>::default())
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(WireframePlugin)
        .add_plugin(NoCameraPlayerPlugin)
        .add_startup_system(spawn_camera)
        .add_system(create_array_texture)
        .add_system(spawn_chunk_meshes)
        .add_system(click_detection)
        .add_system(click_to_break)
        .add_system(click_to_place)
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
}
