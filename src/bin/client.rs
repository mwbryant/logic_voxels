use std::{
    net::{SocketAddr, UdpSocket},
    time::{Duration, SystemTime},
};

use bevy::{ecs::event::ManualEventReader, input::mouse::MouseMotion, log::LogSettings};
use bevy_flycam::MovementSettings;
use bevy_inspector_egui::{bevy_egui::EguiContext, egui};
use logic_voxels::{client_chunks::ClientChunkPlugin, *};

#[derive(Component)]
pub struct FollowCamera;

fn create_renet_client(server_addr: SocketAddr) -> RenetClient {
    //TODO Prompt for server IP
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let connection_config = RenetConnectionConfig::default();
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };
    RenetClient::new(current_time, socket, client_id, connection_config, authentication).unwrap()
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
        .insert_resource(LogSettings {
            filter: "info,wgpu_core=warn,wgpu_hal=off,rechannel=warn".into(),
            level: bevy::log::Level::DEBUG,
        })
        .insert_resource(WindowDescriptor {
            width: 1280.,
            height: 720.,
            title: "Voxel Tests".to_string(),
            //present_mode: PresentMode::Immediate,
            resizable: false,
            ..Default::default()
        })
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .add_state(ClientState::MainMenu)
        .add_plugin(RenetClientPlugin)
        //.insert_resource(create_renet_client())
        .init_resource::<Lobby>()
        //XXX is this a bad way to do things...
        .init_resource::<CurrentClientMessages>()
        .init_resource::<CurrentClientBlockMessages>()
        .add_stage_after(CoreStage::PreUpdate, ReadMessages, SystemStage::parallel())
        .add_system_to_stage(
            ReadMessages,
            client_recieve_messages.with_run_criteria(run_if_client_connected),
        )
        .add_system_set(SystemSet::on_update(ClientState::MainMenu).with_system(client_connection_system))
        .add_plugin(ClientChunkPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<CustomMaterial>::default())
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(WireframePlugin)
        //.add_plugin(NoCameraPlayerPlugin)
        .add_system_set(
            SystemSet::on_update(ClientState::Gameplay)
                .with_system(player_move)
                .with_system(player_look)
                .with_system(cursor_grab),
        )
        .add_system_set(SystemSet::on_update(ClientState::Connecting).with_system(client_connection_ready))
        .init_resource::<InputState>()
        .init_resource::<MovementSettings>()
        //
        .add_startup_system(spawn_camera)
        .add_system_set(SystemSet::on_update(ClientState::Gameplay).with_system(ping_test))
        .add_system(camera_follow)
        .run();
}

pub struct ServerAddr(String);

impl FromWorld for ServerAddr {
    fn from_world(world: &mut World) -> Self {
        ServerAddr("192.168.0.16".to_string())
    }
}

fn client_connection_system(
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    mut state: ResMut<State<ClientState>>,
    mut egui_context: ResMut<EguiContext>,
    mut addr: Local<ServerAddr>,
) {
    egui::Window::new("Connect to server").show(egui_context.ctx_mut(), |ui| {
        ui.label("Address: ");
        ui.add(egui::TextEdit::singleline(&mut addr.0));
        if ui.button("Connect").clicked() || keyboard.just_pressed(KeyCode::Return) {
            info!("Starting Connection!");
            let server_addr = format!("{}:5000", addr.0).parse().unwrap();
            commands.insert_resource(create_renet_client(server_addr));
            let _ = state.set(ClientState::Connecting);
        }
    });
}

fn client_connection_ready(
    mut state: ResMut<State<ClientState>>,
    client: Res<RenetClient>,
    mut timeout_countdown: Local<Timer>,
    mut egui_context: ResMut<EguiContext>,
    time: Res<Time>,
) {
    //This is the default timeout time, could be cleaner and do a from world but eh
    timeout_countdown.set_duration(Duration::from_secs_f32(15.));

    if client.is_connected() {
        info!("Connected!");
        timeout_countdown.reset();
        let _ = state.set(ClientState::Gameplay);
    } else if let Some(reason) = client.disconnected() {
        error!("Failed to connect! {}", reason);
        timeout_countdown.reset();
        let _ = state.set(ClientState::MainMenu);
    } else {
        timeout_countdown.tick(time.delta());
        egui::Window::new("Connect to server").show(egui_context.ctx_mut(), |ui| {
            ui.label(format!(
                "Connecting! Time until timeout: {:.1}",
                timeout_countdown.duration().as_secs_f32() - timeout_countdown.elapsed_secs()
            ));
            if ui.button("Give Up").clicked() {
                timeout_countdown.reset();
                let _ = state.set(ClientState::MainMenu);
            }
        });
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

//Run before update
fn client_recieve_messages(
    mut client: ResMut<RenetClient>,
    mut messages: ResMut<CurrentClientMessages>,
    mut block_messages: ResMut<CurrentClientBlockMessages>,
) {
    messages.clear();
    block_messages.clear();
    for channel in [Channel::Reliable, Channel::Unreliable] {
        while let Some(message) = client.receive_message(channel.id()) {
            let server_message = bincode::deserialize(&message).unwrap();
            messages.push(server_message);
        }
    }
    while let Some(message) = client.receive_message(Channel::Block.id()) {
        let server_message = bincode::deserialize(&message).unwrap();
        block_messages.push(server_message);
    }
}

fn ping_test(mut client: ResMut<RenetClient>, keyboard: Res<Input<KeyCode>>, messages: Res<CurrentClientMessages>) {
    if keyboard.just_pressed(KeyCode::P) {
        info!("Sending ping!");
        ClientMessage::Ping.send(&mut client);
    }
    for message in messages.iter() {
        if matches!(message, ServerMessage::Pong) {
            info!("Pong!");
        }
    }
}

fn camera_follow(
    camera: Query<&Transform, With<Camera3d>>,
    mut followers: Query<&mut Transform, (With<FollowCamera>, Without<Camera3d>)>,
) {
    for mut follower in &mut followers {
        follower.translation = camera.single().translation;
    }
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
