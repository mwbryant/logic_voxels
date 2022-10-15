use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

use crate::{server_chunks::ServerChunkPlugin, *};
use bevy::log::LogSettings;
use bevy_inspector_egui::bevy_egui::EguiContext;
use local_ip_address::local_ip;
use renet_visualizer::RenetServerVisualizer;

pub fn create_renet_server() -> RenetServer {
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

pub fn update_visulizer(
    mut egui_context: ResMut<EguiContext>,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
    lobby: Res<Lobby>,
    server: Res<RenetServer>,
) {
    visualizer.update(&server);
    bevy_inspector_egui::egui::TopBottomPanel::bottom("bottom_panel")
        .min_height(200.)
        .resizable(true)
        .show(egui_context.ctx_mut(), |ui| {
            bevy_inspector_egui::egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Network Info");
                for (id, _) in lobby.players.iter() {
                    ui.label(format!("Client: {}", *id));
                    ui.horizontal(|ui| {
                        visualizer.draw_client_metrics(*id, ui);
                    });
                }
            });
        });
}

pub fn server_connection(
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

//Run before update
pub fn server_recieve_messages(mut server: ResMut<RenetServer>, mut messages: ResMut<CurrentServerMessages>) {
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

pub fn ping_test(messages: Res<CurrentServerMessages>, mut server: ResMut<RenetServer>) {
    for (id, message) in messages.iter() {
        if matches!(message, ClientMessage::Ping) {
            info!("Got ping from {}!", id);
            ServerMessage::Pong.send(&mut server, *id);
        }
    }
}
