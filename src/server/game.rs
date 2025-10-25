use std::{
    sync::Arc,
    thread::{sleep, spawn},
    time::Duration,
};

use log::*;
use lost_signal::common::{
    action::Action,
    types::{Entity, Tile},
};

use crate::{command::CommandMessage, states::States, world::World};

pub const TICK_DURATION: Duration = Duration::from_millis(300);

pub struct Game {
    states: Arc<States>,
}

impl Game {
    pub fn new(states: Arc<States>) -> Game {
        Game { states }
    }

    pub fn run(self) {
        spawn(move || {
            let mut world: World;
            {
                world = self.states.world.lock().unwrap().clone();
            }
            loop {
                let tick = world.tick;
                let inputs = self.states.command_queue.get_commands(tick);
                enact_tick(&mut world, inputs);
                {
                    *self.states.world.lock().unwrap() = world.clone();
                }
                world.tick = tick.wrapping_add(1);
                // We advance ticks by fixed duration but it might change in the future.
                sleep(TICK_DURATION);
            }
        });
    }
}

pub fn enact_tick(world: &mut World, commands: Vec<CommandMessage>) {
    for cmd in commands {
        let entity_id = cmd.entity_id;
        let entity = world.entities.get_mut(&entity_id);

        match (cmd.action, entity) {
            (Action::Spawn, None) => {
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
            (Action::Move(dir), Some(ent)) => {
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
