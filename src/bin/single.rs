use std::net::SocketAddr;

use bevy::log::LogSettings;

use local_ip_address::local_ip;
use logic_voxels::{client_chunks::ClientChunkPlugin, server_chunks::ServerChunkPlugin, *};
use renet_visualizer::RenetServerVisualizer;

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
        .add_stage_after(CoreStage::PreUpdate, ReadMessages, SystemStage::parallel())
        .init_resource::<CurrentServerMessages>()
        .init_resource::<CurrentClientMessages>()
        .insert_resource(RenetServerVisualizer::<200>::default())
        .init_resource::<CurrentClientBlockMessages>()
        .add_state(ClientState::Connecting)
        .add_plugin(RenetClientPlugin)
        .add_plugin(RenetServerPlugin)
        .insert_resource(create_renet_client(SocketAddr::new(local_ip().unwrap(), 5000)))
        .insert_resource(create_renet_server())
        .init_resource::<Lobby>()
        .add_system(server_connection)
        .add_system_to_stage(ReadMessages, server_recieve_messages)
        .add_plugin(ServerChunkPlugin)
        .add_system_to_stage(
            ReadMessages,
            client_recieve_messages.with_run_criteria(run_if_client_connected),
        )
        .add_system_set(SystemSet::on_update(ClientState::MainMenu).with_system(client_connection_system))
        .add_system_set(SystemSet::on_update(ClientState::Connecting).with_system(client_connection_ready))
        .add_plugin(ClientChunkPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin)
        //TODO move
        .add_plugin(MaterialPlugin::<CustomMaterial>::default())
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(WireframePlugin)
        .add_startup_system(spawn_camera)
        .add_system_set(SystemSet::on_update(ClientState::Gameplay).with_system(server_ping_test))
        .add_system_set(SystemSet::on_update(ClientState::Gameplay).with_system(client_ping_test))
        .run();
}
