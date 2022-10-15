use std::{
    net::{SocketAddr, UdpSocket},
    time::{Duration, SystemTime},
};

use bevy::{app::AppExit, log::LogSettings};
use bevy_inspector_egui::{bevy_egui::EguiContext, egui};
use logic_voxels::{client_chunks::ClientChunkPlugin, *};

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
        .add_system_set(SystemSet::on_update(ClientState::Gameplay).with_system(client_ping_test))
        .run();
}
