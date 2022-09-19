use std::{
    net::{SocketAddr, UdpSocket},
    time::{Duration, SystemTime},
};

use bevy::log::LogPlugin;
use local_ip_address::local_ip;
use logic_voxels::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

fn create_renet_server() -> RenetServer {
    //TODO prompt for lan or external?
    //I have a weak understanding here
    let server_addr = SocketAddr::new(local_ip().unwrap(), 5000);
    println!("Creating Server! {:?}", server_addr);

    let socket = UdpSocket::bind(server_addr).unwrap();
    let connection_config = RenetConnectionConfig::default();
    let server_config = ServerConfig::new(64, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure);
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        // Cpu limiting (I wish I had a better way to make a headless bevy app low power but I can't find one)
        // Poor headless bevy
        .add_system(janky_cpu_limiting)
        .add_plugin(LogPlugin::default())
        .add_plugin(RenetServerPlugin)
        .insert_resource(create_renet_server())
        //XXX is this a bad way to do things...
        .init_resource::<CurrentServerMessages>()
        .add_stage_after(CoreStage::PreUpdate, ReadMessages, SystemStage::parallel())
        .add_system_to_stage(ReadMessages, server_recieve_messages)
        //FIXME fix this because its generating useless meshes on the server
        //.add_system(initial_chunk_spawning)
        .add_system(server_create_chunks)
        .add_system(ping_test)
        .run();
}

fn janky_cpu_limiting() {
    std::thread::sleep(Duration::from_millis(10));
}

//Run before update
fn server_recieve_messages(mut server: ResMut<RenetServer>, mut messages: ResMut<CurrentServerMessages>) {
    messages.clear();
    for client_id in server.clients_id().into_iter() {
        for channel in [Channel::Reliable, Channel::Unreliable] {
            while let Some(message) = server.receive_message(client_id, channel.id()) {
                let client_message = bincode::deserialize(&message).unwrap();
                messages.push((client_id, client_message));
            }
        }
    }
}

fn ping_test(messages: Res<CurrentServerMessages>, mut server: ResMut<RenetServer>) {
    for (id, message) in messages.iter() {
        if matches!(message, ClientMessage::Ping) {
            info!("Got ping from {}!", id);
            ServerMessage::Pong.send(&mut server, *id);
        }
    }
}
