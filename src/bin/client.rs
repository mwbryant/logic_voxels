use std::{
    net::{SocketAddr, UdpSocket},
    time::{Duration, SystemTime},
};

use bevy::log::LogSettings;
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
// If any error is found we just panic
fn panic_on_error_system(mut renet_error: EventReader<RenetError>) {
    for e in renet_error.iter() {
        panic!("{}", e);
    }
}

fn main() {
    App::new()
        .add_system(panic_on_error_system)
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
        .add_system_set(SystemSet::on_update(ClientState::Connecting).with_system(client_connection_ready))
        .add_plugin(ClientChunkPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugins(DefaultPlugins)
        //TODO move
        .add_plugin(MaterialPlugin::<CustomMaterial>::default())
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(WireframePlugin)
        .add_startup_system(spawn_camera)
        .add_system_set(SystemSet::on_update(ClientState::Gameplay).with_system(ping_test))
        .add_system(camera_follow)
        .run();
}

pub struct ServerAddr(String);

impl FromWorld for ServerAddr {
    fn from_world(_world: &mut World) -> Self {
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
            transform: Transform::from_xyz(-0.0, 0.0, -0.0).looking_at(Vec3::new(100.0, 0.0, 100.0), Vec3::Y),
            ..default()
        })
        .insert_bundle(VisibilityBundle::default())
        //.insert(PhysicsObject::default())
        .insert(FlyCam);
}
