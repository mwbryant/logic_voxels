use std::{
    fs,
    path::Path,
    sync::{Arc, RwLock},
};

use crate::prelude::*;
use bevy::{
    tasks::{AsyncComputeTaskPool, Task},
    utils::{FloatOrd, HashMap},
};
use futures_lite::future;
use noise::{NoiseFn, Perlin};

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
fn server_load_chunk(commands: &mut Commands, loaded_chunks: &mut LoadedChunks, chunk: Chunk) {
    let chunk_pos = chunk.pos;

    let arc = Arc::new(RwLock::new(chunk));

    //Check doesn't already exists!
    if let Some(chunk) = loaded_chunks.ent_map.remove(&chunk_pos) {
        error!("I already have this chunk loaded! {:?}", chunk_pos);
        commands.entity(chunk).despawn_recursive();
    }

    let comp = ChunkComp::new(arc);
    let ent = commands.spawn().insert(comp).id();
    loaded_chunks.ent_map.insert(chunk_pos, ent);
}

pub fn server_create_chunks(
    mut commands: Commands,
    messages: Res<CurrentServerMessages>,
    mut server: ResMut<RenetServer>,
    mut queued_requests: Local<Vec<(u64, IVec3)>>,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {
    queued_requests.retain(|(id, pos)| {
        if server.can_send_message(*id, Channel::Block.id()) {
            let mut chunk_data = gen_chunk(pos.x, pos.y, pos.z);
            chunk_data.pos = *pos;
            ServerBlockMessage::Chunk(chunk_data.compress()).send(&mut server, *id);
            info!("Sending Chunk! {}", *pos);
            server_load_chunk(&mut commands, &mut loaded_chunks, chunk_data);
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
                let mut chunk_data = gen_chunk(pos.x, pos.y, pos.z);
                chunk_data.pos = *pos;
                ServerBlockMessage::Chunk(chunk_data.compress()).send(&mut server, *id);
                server_load_chunk(&mut commands, &mut loaded_chunks, chunk_data);
            } else {
                queued_requests.push((*id, *pos));
            }
        }
    }
}
