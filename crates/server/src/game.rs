use std::{
    sync::{Arc, mpsc::Receiver},
    thread::spawn,
};

use log::*;
use losig_core::{
    sense::Senses,
    types::{Action, Avatar, AvatarId, Position, Tile},
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
                let senses = enact_tick(&mut world, &commands);
                {
                    *self.states.world.lock().unwrap() = world.clone();
                }
                for msg in gather_infos(&mut world, senses) {
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

/// Returns accepted information gathers
pub fn enact_tick(world: &mut World, commands: &[CommandMessage]) -> Vec<(AvatarId, Senses)> {
    let mut accepted_senses = vec![];
    world.tick = world.tick.wrapping_add(1);
    for cmd in commands {
        let avatar = world.avatars.remove(&cmd.avatar_id);
        match avatar {
            Some(mut avatar) => {
                enact_command(world, cmd, &mut avatar);
                let cost = cmd.senses.signal_cost();
                if avatar.signal >= cost {
                    avatar.signal -= cost;
                    accepted_senses.push((avatar.id, cmd.senses.clone()));
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

    accepted_senses
}

fn spawn_avatar(world: &mut World, avatar_id: AvatarId) {
    info!("Avatar {} spawned.", avatar_id);
    world.avatars.insert(
        avatar_id,
        Avatar {
            id: avatar_id,
            position: spawn_position(world, avatar_id),
            broken: false,
            signal: 100,
        },
    );
}

fn spawn_position(world: &World, avatar_id: AvatarId) -> Position {
    let spawn_position = world.find_free_spawns();
    spawn_position[avatar_id as usize % spawn_position.len()]
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
    if avatar.broken && !cmd.action.allow_broken() {
        return;
    }

    let avatar_id = cmd.avatar_id;

    debug!("cmd received: {cmd:?}");
    match cmd.action {
        Action::Move(dir) => {
            let next_pos = avatar.position.move_once(dir);

            let tile = world.tiles.at(next_pos);
            if !matches!(tile, Tile::Wall) {
                avatar.position = next_pos;
            }

            // Spawn tiles recharge signal
            if matches!(tile, Tile::Spawn) {
                avatar.signal = 100;
            }

            if Some(avatar.position) == world.orb {
                // WIN !
                info!("The game was won by {}!", avatar.id);
                world.orb = None;
                world.winner = Some(avatar.id);
            }
        }
        Action::Spawn => {
            avatar.position = spawn_position(world, avatar_id);
            avatar.signal = 100;
            avatar.broken = false;
        }
        Action::Wait => {
            // NOOP
        }
    }
}

fn gather_infos(world: &World, senses: Vec<(AvatarId, Senses)>) -> Vec<SensesMessage> {
    senses
        .into_iter()
        .filter_map(|sns| gather_info(world, sns))
        .collect()
}

fn gather_info(world: &World, senses: (AvatarId, Senses)) -> Option<SensesMessage> {
    let (avatar_id, senses) = senses;
    let avatar = world.find_avatar(avatar_id);
    let senses = sense::gather(&senses, avatar, world);

    Some(SensesMessage {
        avatar_id: avatar_id,
        senses,
    })
}
