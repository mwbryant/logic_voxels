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

pub enum SendError {
    CannotSend,
}

impl ServerBlockMessage {
    pub fn send(&self, server: &mut RenetServer, id: u64) -> Result<(), SendError> {
        if server.can_send_message(id, Channel::Block.id()) {
            let message = bincode::serialize(self).unwrap();
            server.send_message(id, Channel::Block.id(), message);
            Ok(())
        } else {
            Err(SendError::CannotSend)
        }
    }

    pub fn broadcast(&self, server: &mut RenetServer) -> Result<(), SendError> {
        for id in server.clients_id() {
            if !server.can_send_message(id, Channel::Block.id()) {
                return Err(SendError::CannotSend);
            }
        }
        let message = bincode::serialize(self).unwrap();
        server.broadcast_message(Channel::Block.id(), message);
        Ok(())
    }

    pub fn broadcast_except(&self, server: &mut RenetServer, id: u64) -> Result<(), SendError> {
        for all_id in server.clients_id() {
            if all_id == id {
                continue;
            }
            if !server.can_send_message(id, Channel::Block.id()) {
                return Err(SendError::CannotSend);
            }
        }
        let message = bincode::serialize(self).unwrap();
        server.broadcast_message_except(id, Channel::Block.id(), message);
        Ok(())
    }
}

impl ServerMessage {
    pub fn send(&self, server: &mut RenetServer, id: u64) -> Result<(), SendError> {
        //TODO only in debug please
        if server.can_send_message(id, Channel::Reliable.id()) {
            return Err(SendError::CannotSend);
        }
        let message = bincode::serialize(self).unwrap();
        match self {
            ServerMessage::Pong => server.send_message(id, Channel::Reliable.id(), message),
        }
        Ok(())
    }

    pub fn broadcast(&self, server: &mut RenetServer) -> Result<(), SendError> {
        //TODO only in debug please
        for id in server.clients_id() {
            if server.can_send_message(id, Channel::Reliable.id()) {
                return Err(SendError::CannotSend);
            }
        }
        let message = bincode::serialize(self).unwrap();
        match self {
            ServerMessage::Pong => server.broadcast_message(Channel::Reliable.id(), message),
        }
        Ok(())
    }

    pub fn broadcast_except(&self, server: &mut RenetServer, id: u64) -> Result<(), SendError> {
        for id in server.clients_id() {
            if server.can_send_message(id, Channel::Reliable.id()) {
                return Err(SendError::CannotSend);
            }
        }
        let message = bincode::serialize(self).unwrap();
        match self {
            ServerMessage::Pong => server.broadcast_message_except(id, Channel::Reliable.id(), message),
        }
        Ok(())
    }
}

impl ClientMessage {
    pub fn send(&self, client: &mut RenetClient) -> Result<(), SendError> {
        let message = bincode::serialize(self).unwrap();
        match self {
            ClientMessage::Ping
            | ClientMessage::RequestChunk(..)
            | ClientMessage::BreakBlock(..)
            | ClientMessage::PlaceBlock(..) => {
                if client.can_send_message(Channel::Reliable.id()) {
                    client.send_message(Channel::Reliable.id(), message);
                    Ok(())
                } else {
                    Err(SendError::CannotSend)
                }
            }
        }
    }
}
