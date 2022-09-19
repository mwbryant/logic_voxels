use std::sync::{Arc, RwLock};

use bevy::{
    tasks::{AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use futures_lite::future;
use noise::{NoiseFn, Perlin};

use crate::prelude::*;

//XXX maybe a memory leak because unloaded chunks are never removed
// Not a very robust design
#[derive(Default)]
pub struct LoadedChunks {
    pub ent_map: HashMap<IVec3, Entity>,
}

//World generation
fn gen_chunk(chunk_x: f32, chunk_y: f32, chunk_z: f32) -> Chunk {
    let mut chunk = Chunk::default();
    let perlin = Perlin::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
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
                if value >= 0.7 {
                    chunk.cubes[x][y][z] = Block::Grass
                }
            }
        }
    }
    chunk
}

#[derive(Component)]
pub struct CreateChunkTask(Task<(Chunk, Mesh)>);

pub fn server_send_chunks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut CreateChunkTask)>,
    //How to fake mut here for interior mutability parallelism
    chunks: Query<&ChunkComp>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    texture: Res<ChunkTexture>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    mut server: ResMut<RenetServer>,
) {
    let mut spawned_this_frame = HashMap::default();
    let mut updates = 0;
    for (ent, mut task) in &mut tasks {
        if let Some((chunk, mesh)) = future::block_on(future::poll_once(&mut task.0)) {
            let chunk_pos = chunk.pos;
            let pos = CHUNK_SIZE as i32 * chunk.pos;

            let arc = Arc::new(RwLock::new(chunk));
            loaded_chunks.ent_map.insert(chunk_pos, ent);
            let arc = ChunkComp::new(arc);

            fn connect_neighbor(
                pos: IVec3,
                dir: Direction,
                loaded_chunks: &ResMut<LoadedChunks>,
                chunks: &Query<&ChunkComp>,
                comp: &ChunkComp,
                spawned_this_frame: &HashMap<Entity, ChunkComp>,
            ) {
                if loaded_chunks.ent_map.contains_key(&pos) {
                    //Set this chunks top neighbor
                    //Set the top neighbors bottom to this chunk
                    if let Ok(neighbor) = chunks.get(loaded_chunks.ent_map[&pos]) {
                        comp.set_neighbor(dir, neighbor);
                        neighbor.set_neighbor(dir.opposite(), comp);
                    } else {
                        //Spawned this frame
                        let neighbor = &spawned_this_frame[&loaded_chunks.ent_map[&pos]];
                        comp.set_neighbor(dir, neighbor);
                        neighbor.set_neighbor(dir.opposite(), comp);
                    }
                }
            }

            connect_neighbor(
                chunk_pos + IVec3::Y,
                Direction::Top,
                &loaded_chunks,
                &chunks,
                &arc,
                &spawned_this_frame,
            );
            connect_neighbor(
                chunk_pos - IVec3::Y,
                Direction::Bottom,
                &loaded_chunks,
                &chunks,
                &arc,
                &spawned_this_frame,
            );

            connect_neighbor(
                chunk_pos + IVec3::X,
                Direction::Front,
                &loaded_chunks,
                &chunks,
                &arc,
                &spawned_this_frame,
            );
            connect_neighbor(
                chunk_pos - IVec3::X,
                Direction::Back,
                &loaded_chunks,
                &chunks,
                &arc,
                &spawned_this_frame,
            );

            connect_neighbor(
                chunk_pos + IVec3::Z,
                Direction::Left,
                &loaded_chunks,
                &chunks,
                &arc,
                &spawned_this_frame,
            );
            connect_neighbor(
                chunk_pos - IVec3::Z,
                Direction::Right,
                &loaded_chunks,
                &chunks,
                &arc,
                &spawned_this_frame,
            );

            //FIXME servers don't need mesh bundles
            commands.entity(ent).insert_bundle(MaterialMeshBundle {
                mesh: meshes.add(mesh),
                //mesh: meshes.add(shape::Box::default().into()),
                material: materials.add(CustomMaterial {
                    textures: texture.0.clone(),
                }),
                transform: Transform::from_xyz(pos.x as f32, pos.y as f32, pos.z as f32),

                ..default()
            });
            //.insert(Wireframe);
            spawned_this_frame.insert(ent, arc);

            commands.entity(ent).remove::<CreateChunkTask>();
            updates += 1;
            if updates > MAX_CHUNK_UPDATES_PER_FRAME {
                break;
            }
        }
    }
    for (ent, comp) in spawned_this_frame.into_iter() {
        commands.entity(ent).insert(comp);
    }
}

pub fn spawn_chunk_meshes(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut CreateChunkTask)>,
    //How to fake mut here for interior mutability parallelism
    chunks: Query<&ChunkComp>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    texture: Res<ChunkTexture>,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {
    let mut spawned_this_frame = HashMap::default();
    let mut updates = 0;
    for (ent, mut task) in &mut tasks {
        if let Some((chunk, mesh)) = future::block_on(future::poll_once(&mut task.0)) {
            let chunk_pos = chunk.pos;
            let pos = CHUNK_SIZE as i32 * chunk.pos;

            let arc = Arc::new(RwLock::new(chunk));
            loaded_chunks.ent_map.insert(chunk_pos, ent);
            let arc = ChunkComp::new(arc);

            fn connect_neighbor(
                pos: IVec3,
                dir: Direction,
                loaded_chunks: &ResMut<LoadedChunks>,
                chunks: &Query<&ChunkComp>,
                comp: &ChunkComp,
                spawned_this_frame: &HashMap<Entity, ChunkComp>,
            ) {
                if loaded_chunks.ent_map.contains_key(&pos) {
                    //Set this chunks top neighbor
                    //Set the top neighbors bottom to this chunk
                    if let Ok(neighbor) = chunks.get(loaded_chunks.ent_map[&pos]) {
                        comp.set_neighbor(dir, neighbor);
                        neighbor.set_neighbor(dir.opposite(), comp);
                    } else {
                        //Spawned this frame
                        let neighbor = &spawned_this_frame[&loaded_chunks.ent_map[&pos]];
                        comp.set_neighbor(dir, neighbor);
                        neighbor.set_neighbor(dir.opposite(), comp);
                    }
                }
            }

            connect_neighbor(
                chunk_pos + IVec3::Y,
                Direction::Top,
                &loaded_chunks,
                &chunks,
                &arc,
                &spawned_this_frame,
            );
            connect_neighbor(
                chunk_pos - IVec3::Y,
                Direction::Bottom,
                &loaded_chunks,
                &chunks,
                &arc,
                &spawned_this_frame,
            );

            connect_neighbor(
                chunk_pos + IVec3::X,
                Direction::Front,
                &loaded_chunks,
                &chunks,
                &arc,
                &spawned_this_frame,
            );
            connect_neighbor(
                chunk_pos - IVec3::X,
                Direction::Back,
                &loaded_chunks,
                &chunks,
                &arc,
                &spawned_this_frame,
            );

            connect_neighbor(
                chunk_pos + IVec3::Z,
                Direction::Left,
                &loaded_chunks,
                &chunks,
                &arc,
                &spawned_this_frame,
            );
            connect_neighbor(
                chunk_pos - IVec3::Z,
                Direction::Right,
                &loaded_chunks,
                &chunks,
                &arc,
                &spawned_this_frame,
            );

            commands.entity(ent).insert_bundle(MaterialMeshBundle {
                mesh: meshes.add(mesh),
                //mesh: meshes.add(shape::Box::default().into()),
                material: materials.add(CustomMaterial {
                    textures: texture.0.clone(),
                }),
                transform: Transform::from_xyz(pos.x as f32, pos.y as f32, pos.z as f32),

                ..default()
            });
            //.insert(Wireframe);
            spawned_this_frame.insert(ent, arc);

            commands.entity(ent).remove::<CreateChunkTask>();
            updates += 1;
            if updates > MAX_CHUNK_UPDATES_PER_FRAME {
                break;
            }
        }
    }
    for (ent, comp) in spawned_this_frame.into_iter() {
        commands.entity(ent).insert(comp);
    }
}

pub fn load_server_chunks(mut commands: Commands, messages: Res<CurrentClientBlockMessages>) {
    for message in messages.iter() {
        if let ServerBlockMessage::Chunk(chunk) = message {
            //Ugh but I guess this makes ownership happy;
            let chunk_data = chunk.clone();
            let thread_pool = AsyncComputeTaskPool::get();
            let task = thread_pool.spawn(async move {
                let _span = info_span!("Chunk Generation Task", name = "Chunk Generation Task").entered();
                let mesh = create_chunk_mesh(&chunk_data);
                (chunk_data, mesh)
            });
            commands.spawn().insert(CreateChunkTask(task));
        }
    }
}
pub fn server_create_chunks(
    mut commands: Commands,
    messages: Res<CurrentServerMessages>,
    mut server: ResMut<RenetServer>,
) {
    for message in messages.iter() {
        //FIXME detect if I've already created this chunk
        if let (id, ClientMessage::RequestChunk(pos)) = message {
            let chunk_x = pos.x as f32 * CHUNK_SIZE as f32;
            let chunk_y = pos.y as f32 * CHUNK_SIZE as f32;
            let chunk_z = pos.z as f32 * CHUNK_SIZE as f32;
            let mut chunk_data = gen_chunk(chunk_x, chunk_y, chunk_z);
            chunk_data.pos = *pos;
            info!("Sending Chunk! {}", pos);
            ServerBlockMessage::Chunk(chunk_data).send(&mut server, *id);
        }
    }
}

pub fn initial_chunk_spawning(mut commands: Commands, mut client: ResMut<RenetClient>) {
    let chunks_to_spawn = (WORLD_SIZE / 2) as i32 + 1;

    for x in -chunks_to_spawn..chunks_to_spawn {
        for y in -chunks_to_spawn..chunks_to_spawn {
            for z in -chunks_to_spawn..chunks_to_spawn {
                info!("Requesting Chunk {:?}", IVec3::new(x, y, z));
                ClientMessage::RequestChunk(IVec3::new(x, y, z)).send(&mut client);
            }
        }
    }
}
