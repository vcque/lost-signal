use std::{
    sync::{Arc, mpsc::Receiver},
    thread::spawn,
};

use log::*;
use lost_signal::common::{
    action::Action,
    types::{Entity, Tile},
};

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

                if Some(ent.position) == world.orb {
                    // WIN !
                    info!("The game was won by {}!", ent.id);
                    world.orb = None;
                    world.winner = Some(ent.id);
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

fn gather_infos(world: &World, commands: &[CommandMessage]) -> Vec<SensesMessage> {
    commands
        .iter()
        .filter_map(|cmd| gather_info(world, cmd))
        .collect()
}

fn gather_info(world: &World, cmd: &CommandMessage) -> Option<SensesMessage> {
    let address = cmd.address?;
    let entity_id = cmd.entity_id;
    let entity = world.find_entity(entity_id);
    let senses = sense::gather(&cmd.senses, entity, world);

    Some(SensesMessage {
        address,
        entity_id: entity_id,
        senses,
    })
}
