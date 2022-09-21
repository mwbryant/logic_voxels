use std::{net::UdpSocket, time::SystemTime};

use bevy::log::LogSettings;
use logic_voxels::{client_chunks::ClientChunkPlugin, *};

#[derive(Component)]
pub struct FollowCamera;

fn create_renet_client() -> RenetClient {
    //TODO Prompt for server IP
    let server_addr = "192.168.0.16:5000".parse().unwrap();
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
        .add_plugin(RenetClientPlugin)
        .insert_resource(create_renet_client())
        .init_resource::<Lobby>()
        //XXX is this a bad way to do things...
        .init_resource::<CurrentClientMessages>()
        .init_resource::<CurrentClientBlockMessages>()
        .add_stage_after(CoreStage::PreUpdate, ReadMessages, SystemStage::parallel())
        .add_system_to_stage(ReadMessages, client_recieve_messages)
        //.add_system(client_connection_system)
        //
        .add_plugin(ClientChunkPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<CustomMaterial>::default())
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(WireframePlugin)
        .add_plugin(NoCameraPlayerPlugin)
        .add_startup_system(spawn_camera)
        .add_system(ping_test)
        .add_system(camera_follow)
        .run();
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
