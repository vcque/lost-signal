use std::{
    sync::{Arc, mpsc::Receiver},
    thread::spawn,
};

use log::*;
use losig_core::types::{Action, Entity, EntityId, Tile};

use crate::{
    command::CommandMessage,
    sense::{self, SensesMessage},
    states::States,
    world::World,
};

pub struct Game {
    states: Arc<States>,
    commands: Receiver<CommandMessage>,
}

impl Game {
    pub fn new(states: Arc<States>, commands: Receiver<CommandMessage>) -> Game {
        Game { states, commands }
    }

    pub fn run(self) {
        spawn(move || {
            let mut world: World;
            {
                world = self.states.world.lock().unwrap().clone();
            }
            loop {
                let commands = self.recv_cmds();
                enact_tick(&mut world, &commands);
                {
                    *self.states.world.lock().unwrap() = world.clone();
                }
                for msg in gather_infos(&world, &commands) {
                    self.states.senses.send(msg).unwrap();
                }
            }
        });
    }

    fn recv_cmds(&self) -> Vec<CommandMessage> {
        let mut results: Vec<CommandMessage> = self.commands.recv().into_iter().collect();
        results.extend(self.commands.try_iter());
        results
    }
}

pub fn enact_tick(world: &mut World, commands: &[CommandMessage]) {
    world.tick = world.tick.wrapping_add(1);
    for cmd in commands {
        let entity = world.entities.remove(&cmd.entity_id);
        match entity {
            Some(mut entity) => {
                if !entity.broken {
                    enact_command(world, cmd, &mut entity);
                }
                world.entities.insert(entity.id, entity); // Put it back!
            }
            None => {
                if matches!(cmd.action, Action::Spawn) {
                    spawn_entity(world, cmd.entity_id);
                }
            }
        }
    }

    enact_foes(world);
}

fn spawn_entity(world: &mut World, entity_id: EntityId) {
    let spawn_position = world.find_free_spawns();
    let selected_spawn = spawn_position[entity_id as usize % spawn_position.len()];
    info!("Entity {} spawned at {:?}", entity_id, selected_spawn);
    world.entities.insert(
        entity_id,
        Entity {
            id: entity_id,
            position: selected_spawn,
            broken: false,
        },
    );
}

fn enact_foes(world: &mut World) {
    // Foes are static for now
    for foe in world.foes.iter() {
        for entity in world.entities.values_mut() {
            if foe.position == entity.position {
                entity.broken = true;
            }
        }
    }
}

fn enact_command(world: &mut World, cmd: &CommandMessage, entity: &mut Entity) {
    let entity_id = cmd.entity_id;

    match cmd.action {
        Action::Move(dir) => {
            let next_pos = entity.position.move_once(dir);

            let tile = world.tiles.at(next_pos);
            if !matches!(tile, Tile::Wall) {
                info!(
                    "Entity {} moved from {:?} to {:?}",
                    entity_id, entity.position, next_pos
                );
                entity.position = next_pos;
            }

            if Some(entity.position) == world.orb {
                // WIN !
                info!("The game was won by {}!", entity.id);
                world.orb = None;
                world.winner = Some(entity.id);
            }
        }
        Action::Spawn => {
            warn!(
                "Cannot execute the following: ({:?} -> {:?}) {:?}",
                entity_id, entity, cmd
            );
        }
        Action::Wait => {
            // NOOP
        }
    }
}

fn gather_infos(world: &World, commands: &[CommandMessage]) -> Vec<SensesMessage> {
    commands
        .iter()
        .filter_map(|cmd| gather_info(world, cmd))
        .collect()
}

fn gather_info(world: &World, cmd: &CommandMessage) -> Option<SensesMessage> {
    let entity_id = cmd.entity_id;
    let entity = world.find_entity(entity_id);
    let senses = sense::gather(&cmd.senses, entity, world);

    Some(SensesMessage {
        entity_id: entity_id,
        senses,
    })
}
