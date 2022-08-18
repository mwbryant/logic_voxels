use std::f32::consts::PI;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    window::PresentMode,
};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin, PlayerPlugin};
use bevy_inspector_egui::WorldInspectorPlugin;
use noise::{NoiseFn, Perlin};
use rand::{thread_rng, Rng};

#[derive(Component)]
pub struct FollowCamera;

pub const CHUNK_SIZE: usize = 24;
pub const BLOCK_SIZE: f32 = 0.3;

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
        .add_system(rotator)
        .add_system(camera_follow)
        .run();
}

#[derive(Component)]
struct Rotator;

fn camera_follow(
    camera: Query<&Transform, With<Camera3d>>,
    mut followers: Query<&mut Transform, (With<FollowCamera>, Without<Camera3d>)>,
) {
    for mut follower in &mut followers {
        follower.translation = camera.single().translation;
    }
}

fn rotator(mut to_rotate: Query<&mut Transform, With<Rotator>>, time: Res<Time>) {
    for mut transform in &mut to_rotate {
        transform.rotate_axis(Vec3::Y, time.delta_seconds());
    }
}

fn add_face(
    vertices: &mut Vec<Vec3>,
    normals: &mut Vec<Vec3>,
    indicies: &mut Vec<usize>,
    rotation: Quat,
    transform: Vec3,
) {
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

    new_verts
        .iter_mut()
        .for_each(|vec| *vec = (rotation * *vec) + transform);
    new_normals
        .iter_mut()
        .for_each(|vec| *vec = (rotation * *vec));
    //info!("{}", new_normals[1]);

    let vert_start = vertices.len();
    vertices.extend_from_slice(&new_verts);
    normals.extend_from_slice(&new_normals);

    indicies.extend_from_slice(&[vert_start, vert_start + 1, vert_start + 2]);
    indicies.extend_from_slice(&[vert_start, vert_start + 2, vert_start + 3]);
}

//Clippy is angry but I am going to add more to the if clauses soon and the suggestions are less clear
#[allow(clippy::nonminimal_bool)]
fn create_mesh_faces(
    chunk: &Chunk,
    verts: &mut Vec<Vec3>,
    normals: &mut Vec<Vec3>,
    indicies: &mut Vec<usize>,
) {
    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                //Front
                //TODO check neighbor chunks
                if (x != CHUNK_SIZE - 1 && chunk.cubes[x][y][z] && !chunk.cubes[x + 1][y][z])
                    || (x == CHUNK_SIZE - 1 && chunk.cubes[x][y][z])
                {
                    add_face(
                        verts,
                        normals,
                        indicies,
                        Quat::from_axis_angle(Vec3::Y, PI / 2.0),
                        Vec3::new(
                            (x as f32 + 0.5) * BLOCK_SIZE,
                            y as f32 * BLOCK_SIZE,
                            (z as f32 - 0.5) * BLOCK_SIZE,
                        ),
                    );
                }
                //Back
                //TODO check neighbor chunks
                if (x != 0 && chunk.cubes[x][y][z] && !chunk.cubes[x - 1][y][z])
                    || (x == 0 && chunk.cubes[x][y][z])
                {
                    add_face(
                        verts,
                        normals,
                        indicies,
                        Quat::from_axis_angle(Vec3::Y, -PI / 2.0),
                        Vec3::new(
                            (x as f32 - 0.5) * BLOCK_SIZE,
                            y as f32 * BLOCK_SIZE,
                            (z as f32 - 0.5) * BLOCK_SIZE,
                        ),
                    );
                }
                //Top
                //TODO check neighbor chunks
                if (y != CHUNK_SIZE - 1 && chunk.cubes[x][y][z] && !chunk.cubes[x][y + 1][z])
                    || (y == CHUNK_SIZE - 1 && chunk.cubes[x][y][z])
                {
                    add_face(
                        verts,
                        normals,
                        indicies,
                        Quat::from_axis_angle(Vec3::X, -PI / 2.0),
                        Vec3::new(
                            x as f32 * BLOCK_SIZE,
                            (y as f32 + 0.5) * BLOCK_SIZE,
                            (z as f32 - 0.5) * BLOCK_SIZE,
                        ),
                    );
                }
                //Bottom
                //TODO check neighbor chunks
                if (y != 0 && chunk.cubes[x][y][z] && !chunk.cubes[x][y - 1][z])
                    || (y == 0 && chunk.cubes[x][y][z])
                {
                    add_face(
                        verts,
                        normals,
                        indicies,
                        Quat::from_axis_angle(Vec3::X, PI / 2.0),
                        Vec3::new(
                            x as f32 * BLOCK_SIZE,
                            (y as f32 - 0.5) * BLOCK_SIZE,
                            (z as f32 - 0.5) * BLOCK_SIZE,
                        ),
                    );
                }
                //Left
                //TODO check neighbor chunks
                if (z != CHUNK_SIZE - 1 && chunk.cubes[x][y][z] && !chunk.cubes[x][y][z + 1])
                    || (z == CHUNK_SIZE - 1 && chunk.cubes[x][y][z])
                {
                    add_face(
                        verts,
                        normals,
                        indicies,
                        Quat::from_axis_angle(Vec3::Y, 0.0),
                        Vec3::new(
                            (x as f32) * BLOCK_SIZE,
                            y as f32 * BLOCK_SIZE,
                            (z as f32) * BLOCK_SIZE,
                        ),
                    );
                }
                //Back
                //TODO check neighbor chunks
                if (z != 0 && chunk.cubes[x][y][z] && !chunk.cubes[x][y][z - 1])
                    || (z == 0 && chunk.cubes[x][y][z])
                {
                    add_face(
                        verts,
                        normals,
                        indicies,
                        Quat::from_axis_angle(Vec3::Y, PI),
                        Vec3::new(
                            (x as f32) * BLOCK_SIZE,
                            y as f32 * BLOCK_SIZE,
                            (z as f32 - 1.0) * BLOCK_SIZE,
                        ),
                    );
                }
            }
        }
    }
}

fn create_chunk_mesh(chunk: &Chunk) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut verts = Vec::default();
    let mut normals = Vec::default();
    let mut indicies = Vec::default();

    create_mesh_faces(chunk, &mut verts, &mut normals, &mut indicies);

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        verts
            .iter()
            .map(|vec| vec.to_array())
            .collect::<Vec<[f32; 3]>>(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        normals
            .iter()
            .map(|vec| vec.to_array())
            .collect::<Vec<[f32; 3]>>(),
    );
    mesh.set_indices(Some(Indices::U32(
        indicies
            .iter()
            .map(|usized| *usized as u32)
            .collect::<Vec<u32>>(),
    )));
    mesh
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

fn spawn_custom_mesh(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for x in 0..20 {
        for z in 0..20 {
            let chunk_x = x as f32 * CHUNK_SIZE as f32 * BLOCK_SIZE;
            let chunk_z = z as f32 * CHUNK_SIZE as f32 * BLOCK_SIZE;
            let chunk = gen_chunk(chunk_x, chunk_z);
            let mesh = create_chunk_mesh(&chunk);

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
            transform: Transform::from_xyz(-3.0, 15.5, -1.0)
                .looking_at(Vec3::new(100.0, 0.0, 100.0), Vec3::Y),
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
