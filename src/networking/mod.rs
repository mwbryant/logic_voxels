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

mod client_utils;
mod message;
mod server_utils;

pub use client_utils::*;
pub use message::*;
pub use server_utils::*;

#[derive(StageLabel)]
pub struct ReadMessages;

#[derive(Debug, Default)]
pub struct Lobby {
    pub players: HashMap<u64, Entity>,
    //TODO theres better structures for this 2 way coupling but sick today
    //Bimap or something
    pub entities: HashMap<Entity, u64>,
}

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
