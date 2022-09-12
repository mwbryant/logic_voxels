#![allow(clippy::too_many_arguments)]

use chunk_loading::{initial_chunk_spawning, spawn_chunk_meshes, LoadedChunks};
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
mod direction;
mod material;

#[derive(Component)]
pub struct FollowCamera;

pub const CHUNK_SIZE: usize = 16;
pub const WORLD_SIZE: usize = 4;
pub const MAX_CHUNK_UPDATES_PER_FRAME: usize = 30;
pub const BLOCK_SIZE: f32 = 1.0;

fn update_dirty_chunks(
    mut commands: Commands,
    mut chunks: Query<(Entity, &ChunkComp, &mut Handle<Mesh>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    //TODO all of this can be done in parallel except for adding mesh to assets
    //FIXME for now I'm just going to cap the number of chunk updates per frame
    let mut updates = 0;
    for (entity, chunk, mut mesh) in &mut chunks {
        if chunk.read_dirty() {
            *mesh = meshes.add(create_chunk_mesh(&chunk.read_chunk()));
            //Remove because it needs to be recalculated by bevy
            commands.entity(entity).remove::<Aabb>();
            updates += 1;
            chunk.write_dirty(false);
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

fn click_detection(
    mouse: Res<Input<MouseButton>>,
    transform: Query<&Transform, With<Camera3d>>,
    loaded_chunks: Res<LoadedChunks>,
    comps: Query<&ChunkComp>,
) {
    let transform = transform.single();
    let range = 9.0;
    if mouse.pressed(MouseButton::Left) {
        info!("Looking toward {:?}, {:?}", transform.translation, transform.forward());
        // https://www.geeksforgeeks.org/bresenhams-algorithm-for-3-d-line-drawing/
        let end = transform.translation + transform.forward() * range;
        let mut current = transform.translation;

        let diff = end - current;

        let steps = diff.abs().max_element().ceil() * 5.0;

        let inc = diff / steps;

        let mut to_check = Vec::default();
        for _i in 0..(steps as usize) {
            let block_pos = current - Vec3::ONE / 2.0;
            to_check.push(block_pos.round().as_ivec3());
            current += inc;
        }

        for pos in to_check {
            let size = CHUNK_SIZE as i32;
            let offset = IVec3::new(pos.x.rem_euclid(size), pos.y.rem_euclid(size), pos.z.rem_euclid(size));
            let x = if pos.x >= 0 { pos.x / size } else { pos.x / size - 1 };
            let y = if pos.y >= 0 { pos.y / size } else { pos.y / size - 1 };
            let z = if pos.z >= 0 { pos.z / size } else { pos.z / size - 1 };
            let chunk_pos = IVec3::new(x, y, z);
            if let Some(chunk) = loaded_chunks.ent_map.get(&chunk_pos) {
                let chunk = comps.get(*chunk).unwrap();
                info!(
                    "Block {}, {}, {}, {:?}",
                    pos,
                    chunk_pos,
                    offset,
                    chunk.read_block(offset)
                );
                if chunk.read_block(offset) != Block::Air {
                    //Rewind for placement
                    chunk.write_block(offset, Block::Red);
                    return;
                }
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

fn apply_function_to_blocks<F>(chunk: &ChunkComp, mut function: F)
where
    F: FnMut(&Block, [Option<Block>; 6]) -> Option<Block>,
{
    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let neighbors = chunk.read_chunk().get_block_neighbors(x, y, z);
                //No deadlocks because only write when you know you are done reading
                if let Some(block) = function(&chunk.read_block_xyz(x, y, z), neighbors) {
                    chunk.write_dirty(true);
                    chunk.write_block_xyz(x as usize, y as usize, z as usize, block)
                }
            }
        }
    }
}

fn update_dirt(block: &Block, neighbors: [Option<Block>; 6]) -> Option<Block> {
    if matches!(block, Block::Grass) {
        if let Some(top) = neighbors[Direction::Top] {
            if !matches!(top, Block::Air) {
                return Some(Block::Dirt);
            }
        }
    }
    None
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
