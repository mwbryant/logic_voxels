use std::{
    net::{SocketAddr, UdpSocket},
    time::{Duration, SystemTime},
};

use bevy::{app::AppExit, log::LogSettings};
use bevy_inspector_egui::{bevy_egui::EguiContext, egui};
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
        .add_state(ClientState::Connecting)
        .add_plugin(RenetClientPlugin)
        .insert_resource(create_renet_client("192.168.0.16:5000".parse().unwrap()))
        .init_resource::<Lobby>()
        .insert_resource(create_renet_server())
        .add_system(server_connection)
        .insert_resource(RenetServerVisualizer::<200>::default())
        .init_resource::<CurrentServerMessages>()
        .add_system_to_stage(ReadMessages, server_recieve_messages)
        .add_plugin(ServerChunkPlugin)
        .add_system(ping_test)
        .add_plugin(RenetServerPlugin)
        .init_resource::<CurrentClientMessages>()
        .init_resource::<CurrentClientBlockMessages>()
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
        .run();
}
