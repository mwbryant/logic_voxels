use bevy::{ecs::event::ManualEventReader, input::mouse::MouseMotion};
use bevy_flycam::MovementSettings;

use crate::prelude::*;

pub struct PhysicsPlugin;

#[derive(Component)]
pub struct PhysicsObject {
    //This vec3 is meters per second.. hmm
    pub velocity: Vec3,
    pub mass: Kilograms,
}

pub struct Kilograms(f32);
//Find a better way
// Use dim analysis 7 dim vector?
pub struct MetersPerSecond2(f32);

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
            .init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .insert_resource(Gravity(MetersPerSecond2(-9.8)))
            .insert_resource(TerminalVelocity(100.0))
            .add_system_to_stage(CoreStage::PostUpdate, apply_physics_velocity)
            .add_system(apply_physics_gravity)
            .add_system_set(
                SystemSet::on_update(ClientState::Gameplay)
                    .with_system(player_move)
                    .with_system(player_look)
                    .with_system(cursor_grab),
            );
    }
}
fn apply_physics_gravity(mut physics: Query<&mut PhysicsObject>, gravity: Res<Gravity>, time: Res<Time>) {
    for mut physics in &mut physics {
        physics.velocity.y += gravity.0 .0 * time.delta_seconds();
    }
}

fn apply_physics_velocity(
    mut transforms: Query<(&mut Transform, &mut PhysicsObject)>,
    time: Res<Time>,
    terminal_velocity: Res<TerminalVelocity>,
) {
    for (mut transform, mut physics) in &mut transforms {
        if physics.velocity.length() > terminal_velocity.0 {
            physics.velocity = physics.velocity.normalize() * terminal_velocity.0;
        }
        transform.translation += physics.velocity * time.delta_seconds();
    }
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
