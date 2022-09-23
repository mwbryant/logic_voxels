use crate::prelude::*;

pub struct ClickEvent {
    //TODO track held and stuff
    button: MouseButton,
    world_pos: IVec3,
    prev_pos: IVec3,
}

pub(crate) fn click_to_break(
    loaded_chunks: Res<LoadedChunks>,
    comps: Query<&ChunkComp>,
    mut click_reader: EventReader<ClickEvent>,
    mut client: ResMut<RenetClient>,
) {
    for ev in click_reader.iter() {
        if ev.button == MouseButton::Left {
            let (chunk_pos, offset) = Chunk::world_to_chunk(ev.world_pos);
            if let Some(chunk) = loaded_chunks.ent_map.get(&chunk_pos) {
                ClientMessage::BreakBlock(ev.world_pos).send(&mut client);
                let chunk = comps.get(*chunk).unwrap();
                chunk.write_block(offset, Block::Air);
            }
        }
    }
}

pub(crate) fn click_to_place(
    loaded_chunks: Res<LoadedChunks>,
    comps: Query<&ChunkComp>,
    mut click_reader: EventReader<ClickEvent>,
    mut client: ResMut<RenetClient>,
) {
    for ev in click_reader.iter() {
        if ev.button == MouseButton::Right {
            let (chunk_pos, offset) = Chunk::world_to_chunk(ev.prev_pos);
            if let Some(chunk) = loaded_chunks.ent_map.get(&chunk_pos) {
                let chunk = comps.get(*chunk).unwrap();
                if chunk.read_block(offset) == Block::Air {
                    ClientMessage::PlaceBlock(ev.world_pos, Block::Red).send(&mut client);
                    chunk.write_block(offset, Block::Red);
                }
            }
        }
    }
}

pub(crate) fn click_detection(
    mouse: Res<Input<MouseButton>>,
    transform: Query<&Transform, With<Camera3d>>,
    loaded_chunks: Res<LoadedChunks>,
    comps: Query<&ChunkComp>,
    mut click_writer: EventWriter<ClickEvent>,
) {
    let transform = transform.single();
    let range = 9.0;
    if mouse.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        let end = transform.translation + transform.forward() * range;
        let mut current = transform.translation;

        let diff = end - current;

        let steps = diff.abs().max_element().ceil() * 5.0;

        let inc = diff / steps;

        for _i in 0..(steps as usize) {
            let block_pos = current - Vec3::ONE / 2.0;
            let world_pos = block_pos.round().as_ivec3();
            let (chunk_pos, offset) = Chunk::world_to_chunk(world_pos);

            if let Some(chunk) = loaded_chunks.ent_map.get(&chunk_pos) {
                let chunk = comps.get(*chunk).unwrap();

                if chunk.read_block(offset) != Block::Air {
                    //Rewind for placement
                    let block_pos = current - Vec3::ONE / 2.0 - inc;
                    let prev_pos = block_pos.round().as_ivec3();
                    if mouse.just_pressed(MouseButton::Left) {
                        click_writer.send(ClickEvent {
                            button: MouseButton::Left,
                            world_pos,
                            prev_pos,
                        });
                    }
                    //gross
                    if mouse.just_pressed(MouseButton::Right) {
                        click_writer.send(ClickEvent {
                            button: MouseButton::Right,
                            world_pos,
                            prev_pos,
                        });
                    }
                    return;
                }
            }

            current += inc;
        }
    }
}
