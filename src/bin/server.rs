

use bevy::log::LogSettings;


use logic_voxels::{server_chunks::ServerChunkPlugin, *};
use renet_visualizer::RenetServerVisualizer;

fn main() {
    App::new()
        .insert_resource(LogSettings {
            filter: "info,wgpu_core=warn,wgpu_hal=off,rechannel=warn".into(),
            level: bevy::log::Level::DEBUG,
        })
        .insert_resource(WindowDescriptor {
            width: 1200.,
            height: 640.,
            title: "Voxel Server".to_string(),
            //present_mode: PresentMode::Immediate,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::default())
        // Cpu limiting (I wish I had a better way to make a headless bevy app low power but I can't find one)
        // Poor headless bevy
        //.add_system(janky_cpu_limiting)
        //.add_plugin(LogPlugin::default())
        .add_plugin(RenetServerPlugin)
        .init_resource::<Lobby>()
        .insert_resource(create_renet_server())
        .insert_resource(RenetServerVisualizer::<200>::default())
        .add_system(update_visulizer)
        .add_system(server_connection)
        //XXX is this a bad way to do things...
        .init_resource::<CurrentServerMessages>()
        .add_stage_after(CoreStage::PreUpdate, ReadMessages, SystemStage::parallel())
        .add_system_to_stage(ReadMessages, server_recieve_messages)
        .add_plugin(ServerChunkPlugin)
        .add_system(server_ping_test)
        .run();
}
