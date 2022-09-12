use bevy::prelude::*;
use bevy::render::primitives::Aabb;

use crate::block::Block;
use crate::chunk::ChunkComp;
use crate::chunk_mesh_generation::create_chunk_mesh;
use crate::direction::Direction;
use crate::MAX_CHUNK_UPDATES_PER_FRAME;

pub fn update_dirty_chunks(
    mut commands: Commands,
    mut chunks: Query<(Entity, &ChunkComp, &mut Handle<Mesh>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    //TODO all of this can be done in parallel except for adding mesh to assets
    //FIXME for now I'm just going to cap the number of chunk updates per frame
    let mut updates = 0;
    for (entity, chunk, mut mesh) in &mut chunks {
        if chunk.read_dirty() {
            *mesh = meshes.add(create_chunk_mesh(&chunk.read_chunk()));
            //Remove because it needs to be recalculated by bevy
            commands.entity(entity).remove::<Aabb>();
            updates += 1;
            chunk.write_dirty(false);
        }
        if updates > MAX_CHUNK_UPDATES_PER_FRAME {
            return;
        }
    }
}

pub fn update_dirt_sys(chunks: Query<&ChunkComp>, input: Res<Input<KeyCode>>) {
    /*
    if input.just_pressed(KeyCode::Space) {
        chunks.par_for_each(5, |chunk| {
            let _span = info_span!("Dirt update", name = "Dirt Update").entered();
            chunk.apply_function_to_blocks(update_dirt);
        });
    }
    */
}

pub fn update_dirt(block: &Block, neighbors: [Option<Block>; 6]) -> Option<Block> {
    if matches!(block, Block::Grass) {
        if let Some(top) = neighbors[Direction::Top] {
            if !matches!(top, Block::Air) {
                return Some(Block::Dirt);
            }
        }
    }
    None
}
