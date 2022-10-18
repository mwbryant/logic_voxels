use std::{
    net::{SocketAddr, UdpSocket},
    time::{Duration, SystemTime},
};

use crate::*;
use bevy::app::AppExit;
use bevy_inspector_egui::{bevy_egui::EguiContext, egui};
use bevy_rapier3d::prelude::{
    Ccd, Collider, Damping, Dominance, ExternalForce, GravityScale, LockedAxes, RigidBody, Velocity,
};

pub fn create_renet_client(server_addr: SocketAddr) -> RenetClient {
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

// If any error is found we just panic
pub fn panic_on_error_system(mut renet_error: EventReader<RenetError>, mut exit: EventWriter<AppExit>) {
    for e in renet_error.iter() {
        if matches!(e, RenetError::Rechannel(RechannelError::ClientDisconnected(..)))
            || matches!(e, RenetError::Netcode(NetcodeError::Disconnected(..)))
        {
            warn!("Server disconnected! Shutting down");
            exit.send(AppExit);
        } else {
            panic!("{}", e);
        }
    }
}

pub struct DefaultServerAddr(String);

impl FromWorld for DefaultServerAddr {
    fn from_world(_world: &mut World) -> Self {
        //TODO load from file of last used
        DefaultServerAddr("192.168.0.16".to_string())
    }
}

pub fn client_connection_system(
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    mut state: ResMut<State<ClientState>>,
    mut egui_context: ResMut<EguiContext>,
    mut addr: Local<DefaultServerAddr>,
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

pub fn client_connection_ready(
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
//Run before update
pub fn client_recieve_messages(
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

pub fn client_ping_test(
    mut client: ResMut<RenetClient>,
    keyboard: Res<Input<KeyCode>>,
    messages: Res<CurrentClientMessages>,
) {
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

pub fn spawn_camera(mut commands: Commands) {
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
            transform: Transform::from_xyz(-0.0, 1.1, -0.0).looking_at(Vec3::new(100.0, 0.0, 100.0), Vec3::Y),
            ..default()
        })
        .insert_bundle(VisibilityBundle::default())
        //.insert(PhysicsObject::default())
        .insert(RigidBody::Dynamic)
        //.insert(Collider::cuboid(0.5, 0.5, 0.5))
        //.insert(Collider::capsule(Vec3::splat(0.0), Vec3::splat(0.5), 0.3))
        .insert(Collider::capsule(
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            0.4,
        ))
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Dominance::group(10))
        .insert(GravityScale(0.1))
        .insert(Damping {
            linear_damping: 0.5,

            angular_damping: 1.0,
        })
        .insert(Velocity {
            linvel: Vec3::ZERO,
            angvel: Vec3::ZERO,
        })
        .insert(ExternalForce {
            force: Vec3::ZERO,
            torque: Vec3::ZERO,
        })
        .insert(Ccd::enabled())
        .insert(FlyCam);
}
