use std::sync::{Arc, RwLock};

use bevy::{
    tasks::{AsyncComputeTaskPool, Task},
    utils::{FloatOrd, HashMap},
};
use futures_lite::future;

use crate::client::click_detection::*;
use crate::prelude::*;

pub struct ClientChunkPlugin;

impl Plugin for ClientChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadedChunks>()
            .add_event::<ClickEvent>()
            .add_system(spawn_chunk_meshes)
            .add_system_set(
                SystemSet::on_enter(ClientState::Gameplay)
                    .with_system(initial_chunk_requests)
                    //TODO run on image loaded
                    .with_system(create_array_texture),
            )
            .add_system(load_chunks_from_server)
            .add_system(update_dirt_sys)
            .add_system(update_dirty_chunks)
            .add_system_to_stage(CoreStage::PostUpdate, apply_buffered_chunk_writes)
            .add_startup_system_to_stage(StartupStage::PreStartup, client::material::load_chunk_texture)
            .add_system_set(SystemSet::on_update(ClientState::Gameplay).with_system(click_detection))
            .add_system(click_to_break.with_run_criteria(run_if_client_connected))
            .add_system(click_to_place.with_run_criteria(run_if_client_connected));
    }
}

#[derive(Component)]
pub struct CreateChunkTask(Task<(Chunk, Mesh, MeshDescription)>);

pub fn load_chunks_from_server(mut commands: Commands, messages: Res<CurrentClientBlockMessages>) {
    for message in messages.iter() {
        if let ServerBlockMessage::Chunk(chunk) = message {
            let chunk_data = Chunk::from_compressed(chunk);
            let thread_pool = AsyncComputeTaskPool::get();
            let task = thread_pool.spawn(async move {
                let _span = info_span!("Chunk Generation Task", name = "Chunk Generation Task").entered();
                let (mesh, desc) = create_chunk_mesh(&chunk_data);
                (chunk_data, mesh, desc)
            });
            commands.spawn().insert(CreateChunkTask(task));
        }
    }
}

pub fn initial_chunk_requests(mut client: ResMut<RenetClient>) {
    info!("Init Chunks");
    if client.is_connected() {
        let chunks_to_spawn = (WORLD_SIZE / 2) as i32 + 1;

        let mut request = Vec::default();
        for x in -chunks_to_spawn..chunks_to_spawn {
            for y in -chunks_to_spawn..chunks_to_spawn {
                for z in -chunks_to_spawn..chunks_to_spawn {
                    request.push(IVec3::new(x, y, z));
                }
            }
        }

        //TODO closest to player
        request.sort_by_key(|pos| FloatOrd(Vec3::distance(Vec3::ZERO, pos.as_vec3())));

        request.iter().for_each(|request| {
            info!("Requesting Chunk {:?}", request);
            ClientMessage::RequestChunk(*request).send(&mut client);
        });
    } else {
        error!("Not connected to a server!");
    }
}

pub fn spawn_chunk_meshes(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut CreateChunkTask)>,
    //How to fake mut here for interior mutability parallelism
    chunks: Query<&ChunkComp>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    texture: Res<client::material::ChunkTexture>,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {
    let mut spawned_this_frame = HashMap::default();
    let mut updates = 0;
    for (ent, mut task) in &mut tasks {
        if let Some((chunk, mesh, mesh_data)) = future::block_on(future::poll_once(&mut task.0)) {
            let chunk_pos = chunk.pos;
            let pos = CHUNK_SIZE as i32 * chunk.pos;

            let arc = Arc::new(RwLock::new(chunk));
            //Check doesn't already exists!
            if let Some(chunk) = loaded_chunks.ent_map.remove(&chunk_pos) {
                commands.entity(chunk).despawn_recursive();
            }
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
                        //Dirty neighbors because I have changed
                        neighbor.write_dirty(true);
                    } else {
                        //Spawned this frame
                        let neighbor = &spawned_this_frame[&loaded_chunks.ent_map[&pos]];
                        comp.set_neighbor(dir, neighbor);
                        neighbor.set_neighbor(dir.opposite(), comp);
                        neighbor.write_dirty(true);
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
            add_collider(&mut commands, ent, mesh_data);
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
