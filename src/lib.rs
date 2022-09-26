#![allow(clippy::too_many_arguments)]

pub use bevy::{
    asset::AssetServerSettings,
    pbr::wireframe::WireframePlugin,
    render::{
        render_resource::{AddressMode, FilterMode, SamplerDescriptor},
        texture::ImageSettings,
    },
    window::PresentMode,
};
use lz4::block::{compress, CompressionMode};

pub use crate::prelude::*;
pub use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
pub use bevy_inspector_egui::WorldInspectorPlugin;

pub use bevy::utils::HashMap;
pub use bevy_renet::renet::*;
pub use bevy_renet::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub struct Lobby {
    pub players: HashMap<u64, Entity>,
    //TODO theres better structures for this 2 way coupling but sick today
    //Bimap or something
    pub entities: HashMap<Entity, u64>,
}

pub const PROTOCOL_ID: u64 = 1000;

mod chunks;
mod networking;
mod physics;
mod prelude;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ClientState {
    MainMenu,
    Connecting,
    Gameplay,
}
