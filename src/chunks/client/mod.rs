mod click_detection;
pub mod client_chunks;

pub mod chunk_mesh_generation;
mod material;

pub use crate::chunks::chunk_mesh_generation::*;
pub use material::{create_array_texture, CustomMaterial};
