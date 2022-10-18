use bevy::{ecs::event::ManualEventReader, input::mouse::MouseMotion};
use bevy_flycam::MovementSettings;

use bevy_rapier3d::prelude::*;

use crate::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_plugin(NoCameraPlayerPlugin)
            .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugin(RapierDebugRenderPlugin::default())
            .init_resource::<InputState>()
            .insert_resource(MovementSettings {
                sensitivity: 0.00012,
                speed: 4.,
            })
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
        .spawn_bundle(SpatialBundle::default())
        //.spawn_bundle(PbrBundle {
        //mesh: meshes.add(shape::Cube::default().into()),
        //material: mats.add(Color::RED.into()),
        //..default()
        //})
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(RigidBody::Dynamic)
        .insert(Collider::capsule(Vec3::new(0.0, 0.7, 0.0), Vec3::splat(0.0), 0.4))
        .insert(GravityScale(0.1))
        .insert(Restitution::coefficient(0.7));
}

//Yoinked from NoCameraPlayerPlugin to allow working with system sets
fn player_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    windows: Res<Windows>,
    settings: Res<MovementSettings>,
    mut query: Query<(&mut Transform, &mut Velocity), With<FlyCam>>,
) {
    if let Some(window) = windows.get_primary() {
        for (mut transform, mut phys_velocity) in query.iter_mut() {
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
                        //KeyCode::Space => velocity += Vec3::Y,
                        //KeyCode::LShift => velocity -= Vec3::Y,
                        _ => (),
                    }
                }
            }

            velocity = velocity.normalize_or_zero();

            info!("{:?}", velocity);
            phys_velocity.linvel.x = 0.0;
            phys_velocity.linvel.z = 0.0;
            phys_velocity.linvel += velocity * settings.speed * time.delta_seconds() * 100.;
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
    //FIXME this seems to not work if the entity did not already have a collider
    // Is rapier caching something? can I add a disabled collider to work around this
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
