use bevy::{pbr::wireframe::Wireframe, render::primitives::Aabb};
use bevy_rapier3d::prelude::Collider;

use crate::prelude::*;

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
            let data = create_chunk_mesh(&chunk.read_chunk());
            *mesh = meshes.add(data.0);
            add_collider(&mut commands, entity, data.1);
            //TODO move this to physics

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

pub fn apply_buffered_chunk_writes(mut chunks: Query<&mut ChunkComp>) {
    chunks.par_for_each_mut(5, |mut chunk| {
        chunk.apply_buffered_writes();
    });
}

pub fn update_dirt_sys(mut chunks: Query<&mut ChunkComp>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::P) {
        chunks.par_for_each_mut(5, |mut chunk| {
            //let _span = info_span!("Dirt update", name = "Dirt Update").entered();
            chunk.apply_function_to_blocks(update_dirt);
        });
    }
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
