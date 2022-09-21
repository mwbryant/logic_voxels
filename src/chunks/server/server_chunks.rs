use std::{
    fs,
    path::Path,
    sync::{Arc, RwLock},
};

use crate::prelude::*;
use noise::{NoiseFn, Perlin};

pub struct ServerChunkPlugin;
impl Plugin for ServerChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(server_create_chunks)
            .add_system(break_blocks)
            .init_resource::<LoadedChunks>();
    }
}

fn break_blocks(
    loaded_chunks: Res<LoadedChunks>,
    comps: Query<&ChunkComp>,
    messages: Res<CurrentServerMessages>,
    mut server: ResMut<RenetServer>,
) {
    for (id, message) in messages.iter() {
        if let ClientMessage::BreakBlock(pos) = message {
            let (chunk_pos, offset) = Chunk::world_to_chunk(*pos);
            if let Some(chunk) = loaded_chunks.ent_map.get(&chunk_pos) {
                let chunk = comps.get(*chunk).unwrap();
                chunk.write_block(offset, Block::Air);
                ServerBlockMessage::Chunk(chunk.read_chunk().compress()).broadcast_except(&mut server, *id);
            } else {
                warn!("Chunk not loaded on server!");
            }
        }
    }
}

//World generation
//TODO kinda gross because the caller sets the actual chunk positions
fn gen_chunk(chunk_x: i32, chunk_y: i32, chunk_z: i32) -> Chunk {
    //Check if file, if not then write
    //FIXME handle windows path encoding
    let filename = format!("saves/chunk_{}_{}_{}.chunk", chunk_x, chunk_y, chunk_z);
    let filename = Path::new(&filename);

    if filename.exists() {
        let chunk_bytes = fs::read(filename).unwrap();
        Chunk::from_compressed(&chunk_bytes)
    } else {
        info!("Creating new chunk {:?}", filename);
        let mut chunk = Chunk::default();
        let perlin = Perlin::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let chunk_x = chunk_x * CHUNK_SIZE as i32;
                    let chunk_y = chunk_y * CHUNK_SIZE as i32;
                    let chunk_z = chunk_z * CHUNK_SIZE as i32;
                    let value = (perlin.get([
                        (x as f64 + chunk_x as f64) / 21.912,
                        (y as f64 + chunk_y as f64) / 29.312,
                        (z as f64 + chunk_z as f64) / 23.253,
                    ]) + 1.0)
                        / 2.0
                        + (0.12
                            * perlin.get([
                                (x as f64 + chunk_x as f64) / 3.912,
                                (y as f64 + chunk_y as f64) / 2.312,
                                (z as f64 + chunk_z as f64) / 3.253,
                            ])
                            + 0.06);
                    //if value >= (y as f32 / CHUNK_SIZE as f32) as f64 || y == 0 {
                    if value >= 0.5 {
                        chunk.cubes[x][y][z] = Block::Grass
                    }
                }
            }
        }
        //Write to file
        fs::write(filename, chunk.compress()).unwrap();
        chunk
    }
}

//FIXME needs to wire up neighbors and stuff..
fn server_load_chunk(
    commands: &mut Commands,
    loaded_chunks: &mut LoadedChunks,
    chunks: &Query<&ChunkComp>,
    chunk_pos: IVec3,
) -> CompressedChunk {
    //Check doesn't already exists!
    match loaded_chunks.ent_map.get(&chunk_pos) {
        Some(ent) => {
            info!("I already have this chunk loaded! {:?}", chunk_pos);
            chunks.get(*ent).unwrap().read_chunk().compress()
        }
        None => {
            info!("Creating new chunk");
            let mut chunk = gen_chunk(chunk_pos.x, chunk_pos.y, chunk_pos.z);
            chunk.pos = chunk_pos;
            let data = chunk.compress();

            let arc = Arc::new(RwLock::new(chunk));
            let comp = ChunkComp::new(arc);
            let ent = commands.spawn().insert(comp).id();
            loaded_chunks.ent_map.insert(chunk_pos, ent);
            data
        }
    }
}

pub fn server_create_chunks(
    mut commands: Commands,
    messages: Res<CurrentServerMessages>,
    mut server: ResMut<RenetServer>,
    mut queued_requests: Local<Vec<(u64, IVec3)>>,
    chunks: Query<&ChunkComp>,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {
    queued_requests.retain(|(id, pos)| {
        if server.can_send_message(*id, Channel::Block.id()) {
            info!("Sending Chunk! {}", *pos);
            let chunk_data = server_load_chunk(&mut commands, &mut loaded_chunks, &chunks, *pos);
            ServerBlockMessage::Chunk(chunk_data).send(&mut server, *id);
            return false;
        }
        true
    });

    for message in messages.iter() {
        //FIXME detect if I've already created this chunk
        if let (id, ClientMessage::RequestChunk(pos)) = message {
            //let chunk_x = pos.x as f32 * CHUNK_SIZE as f32;
            //let chunk_y = pos.y as f32 * CHUNK_SIZE as f32;
            //let chunk_z = pos.z as f32 * CHUNK_SIZE as f32;
            if server.can_send_message(*id, Channel::Block.id()) {
                info!("Sending Chunk! {}", pos);
                let chunk_data = server_load_chunk(&mut commands, &mut loaded_chunks, &chunks, *pos);
                ServerBlockMessage::Chunk(chunk_data).send(&mut server, *id);
            } else {
                queued_requests.push((*id, *pos));
            }
        }
    }
}
