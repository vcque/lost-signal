use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use log::*;

use crate::{
    command::{Command, CommandMessage, CommandQueue},
    entity::Entity,
    world::{Tile, World},
};

const TICK_DURATION: Duration = Duration::from_millis(300);

pub fn gameloop(world_ref: Arc<Mutex<World>>, command_queue: CommandQueue) {
    let mut tick: u64 = 0;
    let mut world: World;
    {
        world = world_ref.lock().unwrap().clone();
    }
    loop {
        let inputs = command_queue.get_commands(tick);
        enact_tick(&mut world, inputs);
        {
            *world_ref.lock().unwrap() = world.clone();
        }
        tick = tick.wrapping_add(1);
        // We advance ticks by fixed duration but it might change in the future.
        sleep(TICK_DURATION);
    }
}

pub fn enact_tick(world: &mut World, commands: Vec<CommandMessage>) {
    for cmd in commands {
        let entity_id = cmd.entity_id;
        let entity = world.entities.get_mut(&entity_id);

        match (cmd.content, entity) {
            (Command::Spawn, None) => {
                let spawn_position = world.find_free_spawns();
                let selected_spawn = spawn_position[entity_id as usize % spawn_position.len()];

                info!("Entity {} spawned at {:?}", entity_id, selected_spawn);

                world.entities.insert(
                    cmd.entity_id,
                    Entity {
                        id: cmd.entity_id,
                        position: selected_spawn,
                    },
                );
            }
            (Command::Move(dir), Some(ent)) => {
                let next_pos = ent.position.move_once(dir);

                let tile = world.tiles.at(next_pos);
                if !matches!(tile, Tile::Wall) {
                    info!(
                        "Entity {} moved from {:?} to {:?}",
                        entity_id, ent.position, next_pos
                    );
                    ent.position = next_pos;
                }
            }
            (_, ent) => {
                warn!(
                    "Cannot execute the following: ({:?} -> {:?}) {:?}",
                    entity_id, ent, cmd
                );
            }
        }
    }
}
