pub use bevy::{
    asset::AssetServerSettings,
    pbr::wireframe::WireframePlugin,
    render::{
        render_resource::{AddressMode, FilterMode, SamplerDescriptor},
        texture::ImageSettings,
    },
    window::PresentMode,
};

pub use crate::prelude::*;
pub use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
pub use bevy_inspector_egui::WorldInspectorPlugin;

pub use bevy::utils::HashMap;
pub use bevy_renet::renet::*;
pub use bevy_renet::*;
use serde::{Deserialize, Serialize};

#[derive(StageLabel)]
pub struct ReadMessages;

pub enum Channel {
    Reliable,
    Unreliable,
    Block,
}

impl Channel {
    pub fn id(&self) -> u8 {
        match self {
            Channel::Reliable => ReliableChannelConfig::default().channel_id,
            Channel::Unreliable => UnreliableChannelConfig::default().channel_id,
            Channel::Block => BlockChannelConfig::default().channel_id,
        }
    }
}

#[derive(Debug, Default, Deref, DerefMut)]
pub struct CurrentServerMessages(Vec<(u64, ClientMessage)>);

#[derive(Debug, Default, Deref, DerefMut)]
pub struct CurrentClientMessages(Vec<ServerMessage>);
#[derive(Default, Deref, DerefMut)]
pub struct CurrentClientBlockMessages(Vec<ServerBlockMessage>);

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Pong,
}

//Enum size is the max message size, so big messages need to be handled seperate
#[derive(Serialize, Deserialize)]
pub enum ServerBlockMessage {
    Chunk(CompressedChunk),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Ping,
    BreakBlock(IVec3),
    PlaceBlock(IVec3, Block),
    RequestChunk(IVec3),
}

impl ServerBlockMessage {
    pub fn send(&self, server: &mut RenetServer, id: u64) {
        let message = bincode::serialize(self).unwrap();
        server.send_message(id, Channel::Block.id(), message);
    }

    pub fn broadcast(&self, server: &mut RenetServer) {
        let message = bincode::serialize(self).unwrap();
        server.broadcast_message(Channel::Block.id(), message);
    }

    pub fn broadcast_except(&self, server: &mut RenetServer, id: u64) {
        let message = bincode::serialize(self).unwrap();
        server.broadcast_message_except(id, Channel::Block.id(), message);
    }
}

impl ServerMessage {
    pub fn send(&self, server: &mut RenetServer, id: u64) {
        let message = bincode::serialize(self).unwrap();
        match self {
            ServerMessage::Pong => server.send_message(id, Channel::Reliable.id(), message),
        }
    }

    pub fn broadcast(&self, server: &mut RenetServer) {
        let message = bincode::serialize(self).unwrap();
        match self {
            ServerMessage::Pong => server.broadcast_message(Channel::Reliable.id(), message),
        }
    }

    pub fn broadcast_except(&self, server: &mut RenetServer, id: u64) {
        let message = bincode::serialize(self).unwrap();
        match self {
            ServerMessage::Pong => server.broadcast_message_except(id, Channel::Reliable.id(), message),
        }
    }
}

impl ClientMessage {
    pub fn send(&self, client: &mut RenetClient) {
        let message = bincode::serialize(self).unwrap();
        match self {
            ClientMessage::Ping
            | ClientMessage::RequestChunk(..)
            | ClientMessage::BreakBlock(..)
            | ClientMessage::PlaceBlock(..) => {
                if client.can_send_message(Channel::Reliable.id()) {
                    client.send_message(Channel::Reliable.id(), message)
                } else {
                    error!("Cannot send message! {:?}", self);
                }
            }
        }
    }
}
