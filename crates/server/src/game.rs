use std::{
    sync::{Arc, mpsc::Receiver},
    thread::spawn,
};

use log::*;
use losig_core::types::{Action, Avatar, AvatarId, Tile};

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
        let avatar = world.avatars.remove(&cmd.avatar_id);
        match avatar {
            Some(mut avatar) => {
                if !avatar.broken {
                    enact_command(world, cmd, &mut avatar);
                }
                world.avatars.insert(avatar.id, avatar); // Put it back!
            }
            None => {
                if matches!(cmd.action, Action::Spawn) {
                    spawn_avatar(world, cmd.avatar_id);
                }
            }
        }
    }

    enact_foes(world);
}

fn spawn_avatar(world: &mut World, avatar_id: AvatarId) {
    let spawn_position = world.find_free_spawns();
    let selected_spawn = spawn_position[avatar_id as usize % spawn_position.len()];
    info!("Avatar {} spawned at {:?}", avatar_id, selected_spawn);
    world.avatars.insert(
        avatar_id,
        Avatar {
            id: avatar_id,
            position: selected_spawn,
            broken: false,
        },
    );
}

fn enact_foes(world: &mut World) {
    // Foes are static for now
    for foe in world.foes.iter() {
        for avatar in world.avatars.values_mut() {
            if foe.position == avatar.position {
                avatar.broken = true;
            }
        }
    }
}

fn enact_command(world: &mut World, cmd: &CommandMessage, avatar: &mut Avatar) {
    let avatar_id = cmd.avatar_id;

    match cmd.action {
        Action::Move(dir) => {
            let next_pos = avatar.position.move_once(dir);

            let tile = world.tiles.at(next_pos);
            if !matches!(tile, Tile::Wall) {
                info!(
                    "Avatar {} moved from {:?} to {:?}",
                    avatar_id, avatar.position, next_pos
                );
                avatar.position = next_pos;
            }

            if Some(avatar.position) == world.orb {
                // WIN !
                info!("The game was won by {}!", avatar.id);
                world.orb = None;
                world.winner = Some(avatar.id);
            }
        }
        Action::Spawn => {
            warn!(
                "Cannot execute the following: ({:?} -> {:?}) {:?}",
                avatar_id, avatar, cmd
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
    let avatar_id = cmd.avatar_id;
    let avatar = world.find_avatar(avatar_id);
    let senses = sense::gather(&cmd.senses, avatar, world);

    Some(SensesMessage {
        avatar_id: avatar_id,
        senses,
    })
}
