use log::*;
use losig_core::{
    network::{CommandMessage, SenseInfoMessage, ServerMessage},
    sense::{SelfSense, SenseInfo, Senses, TerrainSense},
    types::{Action, Avatar, AvatarId, Offset, Position, Tile},
};

use crate::{
    sense,
    services::Services,
    world::{Stage, World},
    ws_server::{Recipient, ServerMessageWithRecipient},
};

pub struct Game {
    services: Services,
}

impl Game {
    pub fn new(services: Services) -> Game {
        Game { services }
    }

    pub fn enact(&self, command: CommandMessage) {
        let world = &mut *self.services.world.lock().unwrap();
        let senses = enact_tick(world, &command);
        let info = senses.and_then(|s| gather_info(world, command.avatar_id, &s));
        if let Some(info) = info {
            let msg = SenseInfoMessage {
                avatar_id: command.avatar_id,
                turn: command.turn,
                senses: info,
            };
            let msg = ServerMessageWithRecipient {
                recipient: Recipient::Single(command.avatar_id),
                message: ServerMessage::Senses(msg),
            };
            self.services.sender.send(msg).unwrap();
        }
    }
}

pub fn enact_tick(world: &mut World, cmd: &CommandMessage) -> Option<Senses> {
    world.tick = world.tick.wrapping_add(1);
    let avatar = world.avatars.remove(&cmd.avatar_id);

    let mut all_senses: Vec<Senses> = vec![];

    match avatar {
        Some(mut avatar) => {
            avatar.turns += 1; // Increment turn count
            let additional_senses = enact_action(world, &cmd.action, &mut avatar);
            all_senses.push(additional_senses);
            let cost = cmd.senses.signal_cost();
            if avatar.signal >= cost {
                avatar.signal -= cost;
                all_senses.push(cmd.senses.clone());
            }

            world.avatars.insert(avatar.id, avatar); // Put it back!
        }
        None => {
            if matches!(cmd.action, Action::Spawn) {
                spawn_avatar(world, cmd.avatar_id);
            }
        }
    }

    enact_foes(world);
    all_senses
        .into_iter()
        .reduce(|acc, senses| acc.merge(senses))
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
            deaths: 0,
            turns: 0,
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
                if foe.position == avatar.position && i == avatar.stage && !avatar.broken {
                    avatar.deaths += 1;
                    avatar.broken = true;
                }
            }
        }
    }
}

fn enact_action(world: &mut World, action: &Action, avatar: &mut Avatar) -> Senses {
    let mut result = Senses::default();
    if avatar.broken && !action.allow_broken() {
        return result;
    }

    // Specific spawn action
    if *action == Action::Spawn {
        let stage = world.stages.get(avatar.stage).unwrap();
        avatar.position = spawn_position(stage, avatar.id);
        avatar.signal = 100;
        avatar.broken = false;
        result.terrain = Some(TerrainSense { radius: 3 });
        return result;
    }

    let Some(stage) = world.stages.get_mut(avatar.stage) else {
        return result;
    };

    match *action {
        Action::Move(dir) => {
            let next_pos = avatar.position.move_once(dir);

            let tile = stage.tiles.at(next_pos);
            if tile.can_travel() {
                avatar.position = next_pos;
            }

            if avatar.position == stage.orb {
                stage.move_orb();
                result.selfs = Some(SelfSense {});
                if avatar.stage == world.stages.len() - 1 {
                    // Player has won all stages
                    avatar.winner = true;
                } else {
                    avatar.stage += 1; // Crashes the server when the player wins
                    if let Some(stage) = world.stages.get(avatar.stage) {
                        avatar.position = spawn_position(stage, avatar.id);
                    }
                }
            }
        }
        Action::Spawn => unreachable!("Spawn case has already been handled."),
        Action::Wait => {
            // NOOP
        }
    }

    if let Some(stage) = world.stages.get(avatar.stage) {
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
    };

    result
}

fn gather_info(world: &World, avatar_id: AvatarId, senses: &Senses) -> Option<SenseInfo> {
    let avatar = world.find_avatar(avatar_id)?;
    if avatar.winner {
        return Some(SenseInfo::win());
    }

    let stage = world.stages.get(avatar.stage)?;
    let senses = sense::gather(senses, avatar, stage);
    Some(senses)
}
