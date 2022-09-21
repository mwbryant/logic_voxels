use std::{
    net::{SocketAddr, UdpSocket},
    time::{Duration, SystemTime},
};

use bevy::log::{LogPlugin, LogSettings};
use bevy_inspector_egui::bevy_egui::EguiContext;
use local_ip_address::local_ip;
use logic_voxels::{server_chunks::ServerChunkPlugin, *};
use renet_visualizer::RenetServerVisualizer;

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
        .add_system(ping_test)
        .run();
}

fn update_visulizer(
    mut egui_context: ResMut<EguiContext>,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
    server: Res<RenetServer>,
) {
    visualizer.update(&server);
    visualizer.show_window(egui_context.ctx_mut());
}

fn server_connection(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                visualizer.add_client(*id);
                // Spawn new player
                let transform = Transform::from_xyz(0.0, 0.51, 0.0);
                let player_entity = commands.spawn().insert(Name::new(format!("Player {}", id))).id();

                lobby.players.insert(*id, player_entity);
            }
            ServerEvent::ClientDisconnected(id) => {
                visualizer.remove_client(*id);
                if let Some(player_entity) = lobby.players.remove(id) {
                    commands.entity(player_entity).despawn();
                }
            }
        }
    }
}
fn janky_cpu_limiting() {
    std::thread::sleep(Duration::from_millis(5));
}

//Run before update
fn server_recieve_messages(mut server: ResMut<RenetServer>, mut messages: ResMut<CurrentServerMessages>) {
    messages.clear();
    for client_id in server.clients_id().into_iter() {
        for channel in [Channel::Reliable, Channel::Unreliable] {
            while let Some(message) = server.receive_message(client_id, channel.id()) {
                let client_message = bincode::deserialize(&message).unwrap();
                info!("Got message {:?}", client_message);
                messages.push((client_id, client_message));
            }
        }
    }
}

/*
fn place_blocks(
    loaded_chunks: Res<LoadedChunks>,
    comps: Query<&ChunkComp>,
) {
    for ev in click_reader.iter() {
        if ev.button == MouseButton::Right {
            let (chunk_pos, offset) = Chunk::world_to_chunk(ev.prev_pos);
            if let Some(chunk) = loaded_chunks.ent_map.get(&chunk_pos) {
                let chunk = comps.get(*chunk).unwrap();
                if chunk.read_block(offset) == Block::Air {
                    ClientMessage::PlaceBlock(offset, Block::Red).send(&mut client);
                    chunk.write_block(offset, Block::Red);
                }
            }
        }
    }
}
*/
fn ping_test(messages: Res<CurrentServerMessages>, mut server: ResMut<RenetServer>) {
    for (id, message) in messages.iter() {
        if matches!(message, ClientMessage::Ping) {
            info!("Got ping from {}!", id);
            ServerMessage::Pong.send(&mut server, *id);
        }
    }
}
