use bevy::{ecs::event::ManualEventReader, input::mouse::MouseMotion};
use bevy_flycam::MovementSettings;

use bevy_rapier3d::prelude::*;

use crate::prelude::*;

pub struct PhysicsPlugin;

#[derive(Component)]
pub struct PhysicsObject {
    //This vec3 is meters per second.. hmm
    pub velocity: Vec3,
    pub mass: Kilograms,
}

#[derive(Component)]
pub struct PhysicsCube {
    pub length: Meters,
}

#[derive(Deref, DerefMut)]
pub struct Kilograms(f32);
//Find a better way
// Use dim analysis 7 dim vector?
#[derive(Deref, DerefMut)]
pub struct MetersPerSecond2(f32);

#[derive(Deref, DerefMut)]
pub struct Meters(f32);

impl Default for PhysicsObject {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            mass: Kilograms(100.0),
        }
    }
}

//TODO this should be caused by a gravity generator or something, more dynamic
pub struct Gravity(MetersPerSecond2);
pub struct TerminalVelocity(f32);

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_plugin(NoCameraPlayerPlugin)
            .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
            //.add_plugin(RapierDebugRenderPlugin::default())
            .init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .add_system_set(SystemSet::on_enter(ClientState::Gameplay).with_system(test_physics))
            .add_system_set(
                SystemSet::on_update(ClientState::Gameplay)
                    .with_system(player_move)
                    .with_system(player_look)
                    .with_system(cursor_grab),
            );
    }
}

fn test_physics(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(shape::Cube::default().into()),
            material: mats.add(Color::RED.into()),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(GravityScale(0.1))
        .insert(Restitution::coefficient(0.7))
        .insert(PhysicsCube { length: Meters(1.0) })
        .insert(PhysicsObject::default());
}

//Yoinked from NoCameraPlayerPlugin to allow working with system sets
fn player_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    windows: Res<Windows>,
    settings: Res<MovementSettings>,
    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    if let Some(window) = windows.get_primary() {
        for mut transform in query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            for key in keys.get_pressed() {
                if window.cursor_locked() {
                    match key {
                        KeyCode::W => velocity += forward,
                        KeyCode::S => velocity -= forward,
                        KeyCode::A => velocity -= right,
                        KeyCode::D => velocity += right,
                        KeyCode::Space => velocity += Vec3::Y,
                        KeyCode::LShift => velocity -= Vec3::Y,
                        _ => (),
                    }
                }
            }

            velocity = velocity.normalize_or_zero();

            transform.translation += velocity * time.delta_seconds() * settings.speed
        }
    } else {
        warn!("Primary window not found for `player_move`!");
    }
}
//What is this...
#[derive(Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
    pitch: f32,
    yaw: f32,
}

fn player_look(
    settings: Res<MovementSettings>,
    windows: Res<Windows>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    if let Some(window) = windows.get_primary() {
        let mut delta_state = state.as_mut();
        for mut transform in query.iter_mut() {
            for ev in delta_state.reader_motion.iter(&motion) {
                if window.cursor_locked() {
                    // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                    let window_scale = window.height().min(window.width());
                    delta_state.pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                    delta_state.yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                }

                delta_state.pitch = delta_state.pitch.clamp(-1.54, 1.54);

                // Order is important to prevent unintended roll
                transform.rotation =
                    Quat::from_axis_angle(Vec3::Y, delta_state.yaw) * Quat::from_axis_angle(Vec3::X, delta_state.pitch);
            }
        }
    } else {
        warn!("Primary window not found for `player_look`!");
    }
}

fn toggle_grab_cursor(window: &mut Window) {
    window.set_cursor_lock_mode(!window.cursor_locked());
    window.set_cursor_visibility(!window.cursor_visible());
}

fn cursor_grab(keys: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            toggle_grab_cursor(window);
        }
    } else {
        warn!("Primary window not found for `cursor_grab`!");
    }
}

pub fn add_collider(commands: &mut Commands, entity: Entity, desc: MeshDescription) {
    if let Some(new_collider) = create_collider(desc) {
        commands.entity(entity).insert(new_collider);
    } else {
        commands.entity(entity).remove::<Collider>();
    }
}

pub fn create_collider(desc: MeshDescription) -> Option<Collider> {
    let tri_count = desc.vert_indicies.len() / 3;
    let mut indices = Vec::with_capacity(tri_count);
    for index in 0..tri_count {
        indices.push([
            desc.vert_indicies[index * 3] as u32,
            desc.vert_indicies[index * 3 + 1] as u32,
            desc.vert_indicies[index * 3 + 2] as u32,
        ]);
    }
    if tri_count > 0 {
        Some(Collider::trimesh(desc.verts, indices))
    } else {
        None
    }
}
