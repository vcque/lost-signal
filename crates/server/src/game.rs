use std::{
    sync::{Arc, mpsc::Receiver},
    thread::spawn,
};

use log::*;
use losig_core::{
    sense::{SenseInfo, Senses},
    types::{Action, Avatar, AvatarId, Offset, Position, Tile},
};

use crate::{
    command::CommandMessage,
    sense::{self, SensesMessage},
    states::States,
    world::{Stage, World},
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
            loop {
                let commands = self.recv_cmds();
                let msgs: Vec<SensesMessage>;
                {
                    let world = &mut *self.states.world.lock().unwrap();
                    let senses = enact_tick(world, &commands);
                    msgs = gather_infos(world, senses);
                }

                for msg in msgs {
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
            stage: 0,
            position: spawn_position(world.stages.first().unwrap(), avatar_id),
            broken: false,
            signal: 100,
            winner: false,
        },
    );
}

fn spawn_position(stage: &Stage, avatar_id: AvatarId) -> Position {
    let spawn_position = stage.find_spawns();
    spawn_position[avatar_id as usize % spawn_position.len()]
}

fn enact_foes(world: &mut World) {
    // Foes are static for now
    for (i, stage) in world.stages.iter().enumerate() {
        for foe in stage.foes.iter() {
            for avatar in world.avatars.values_mut() {
                if foe.position == avatar.position && i == avatar.stage {
                    avatar.broken = true;
                }
            }
        }
    }
}

fn enact_command(world: &mut World, cmd: &CommandMessage, avatar: &mut Avatar) {
    if avatar.broken && !cmd.action.allow_broken() {
        return;
    }

    // Specific spawn action
    if cmd.action == Action::Spawn {
        avatar.stage = avatar.stage.saturating_sub(1);
        let stage = world.stages.get(avatar.stage).unwrap();
        avatar.position = spawn_position(stage, avatar.id);
        avatar.signal = 100;
        avatar.broken = false;
        return;
    }

    let avatar_id = cmd.avatar_id;
    let Some(stage) = world.stages.get_mut(avatar.stage) else {
        return;
    };

    debug!("cmd received: {cmd:?}");
    match cmd.action {
        Action::Move(dir) => {
            let next_pos = avatar.position.move_once(dir);

            let tile = stage.tiles.at(next_pos);
            if tile.can_travel() {
                avatar.position = next_pos;
            }

            if avatar.position == stage.orb {
                stage.move_orb();
                if avatar.stage == world.stages.len() - 1 {
                    // Player has won all stages
                    avatar.winner = true;
                } else {
                    avatar.stage += 1; // Crashes the server when the player wins
                    if let Some(stage) = world.stages.get(avatar.stage) {
                        avatar.position = spawn_position(stage, avatar_id);
                    }
                }
            }
        }
        Action::Spawn => unreachable!("Spawn case has already been handled."),
        Action::Wait => {
            // NOOP
        }
    }

    let Some(stage) = world.stages.get(avatar.stage) else {
        return;
    };
    // If pylon is adjacent, recharges signal
    for x in -1..2 {
        for y in -1..2 {
            let offset = Offset { x, y };
            let tile = stage.tiles.at(avatar.position + offset);
            if matches!(tile, Tile::Pylon) {
                avatar.signal = 100;
            }
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
    let avatar = world.find_avatar(avatar_id)?;
    if avatar.winner {
        return Some(SensesMessage {
            avatar_id,
            senses: SenseInfo::win(),
        });
    }

    let stage = world.stages.get(avatar.stage)?;
    let senses = sense::gather(&senses, avatar, stage);

    Some(SensesMessage { avatar_id, senses })
}
