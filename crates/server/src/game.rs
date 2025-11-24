use bounded_integer::BoundedU8;
use log::*;
use losig_core::{
    network::{CommandMessage, ServerMessage, TurnResultMessage},
    sense::{Senses, SensesInfo},
    types::{Action, Avatar, AvatarId, GameOver, Offset, Position, Tile},
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

    pub fn new_player(&self, avatar_id: AvatarId) {
        let world = &mut *self.services.world.lock().unwrap();
        info!("Avatar {} spawned.", avatar_id);
        world.avatars.insert(
            avatar_id,
            Avatar {
                id: avatar_id,
                stage: 0,
                position: spawn_position(world.stages.first().unwrap(), avatar_id),
                focus: 100,
                turns: 0,
                gameover: None,
            },
        );
    }

    pub fn enact(&self, command: CommandMessage) {
        let avatar_id = command.avatar_id;
        let world = &mut *self.services.world.lock().unwrap();
        if !world.avatars.contains_key(&avatar_id) {
            // TODO: send back an error msg?
            warn!("Couldn't find avatar #{avatar_id} from command");
            return;
        }

        let senses = enact_tick(world, &command);
        let info = senses.and_then(|s| gather_info(world, command.avatar_id, &s));
        let Some(avatar) = world.find_avatar(avatar_id) else {
            warn!("Couldn't find avatar #{avatar_id} after enacting turn");
            return;
        };

        if let Some(info) = info {
            let msg = TurnResultMessage {
                avatar_id,
                turn: command.turn,
                stage: avatar.stage as u8,
                info,
            };
            let msg = ServerMessageWithRecipient {
                recipient: Recipient::Single(command.avatar_id),
                message: ServerMessage::Turn(msg),
            };
            self.services.sender.send(msg).unwrap();
        }

        if let Some(ref gameover) = avatar.gameover {
            let msg = ServerMessageWithRecipient {
                recipient: Recipient::Single(avatar_id),
                message: ServerMessage::GameOver(gameover.clone()),
            };
            self.services.sender.send(msg).unwrap();
        }
    }
}

pub fn enact_tick(world: &mut World, cmd: &CommandMessage) -> Option<Senses> {
    world.tick = world.tick.wrapping_add(1);
    let avatar = world.avatars.remove(&cmd.avatar_id);

    let mut all_senses: Vec<Senses> = vec![];

    if let Some(mut avatar) = avatar {
        avatar.turns += 1; // Increment turn count
        let additional_senses = enact_action(world, &cmd.action, &mut avatar);
        all_senses.push(additional_senses);
        let cost = cmd.senses.cost();
        if avatar.focus >= cost {
            avatar.focus -= cost;
            all_senses.push(cmd.senses.clone());
        }

        world.avatars.insert(avatar.id, avatar); // Put it back!
    } else {
        warn!("Unreachable: {cmd:?}");
    }

    enact_foes(world);
    all_senses
        .into_iter()
        .reduce(|acc, senses| acc.merge(senses))
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
                    avatar.gameover = Some(GameOver::new(avatar, false));
                }
            }
        }
    }
}

fn enact_action(world: &mut World, action: &Action, avatar: &mut Avatar) -> Senses {
    let mut result = Senses::default();

    // Specific spawn action
    if *action == Action::Spawn {
        let stage = world.stages.get(avatar.stage).unwrap();
        avatar.position = spawn_position(stage, avatar.id);
        avatar.focus = 100;
        result.sight = BoundedU8::const_new::<3>();
        return result;
    }

    let Some(stage) = world.stages.get_mut(avatar.stage) else {
        return result;
    };

    match *action {
        Action::Move(dir) => {
            let next_pos = avatar.position.move_once(dir);

            let tile = stage.tiles.grid[next_pos.into()];
            if tile.can_travel() {
                avatar.position = next_pos;
            }

            if avatar.position == stage.orb {
                stage.move_orb();
                result.selfs = true;
                if avatar.stage == world.stages.len() - 1 {
                    // Player has won all stages
                    avatar.gameover = Some(GameOver::new(avatar, true));
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
        // If pylon is adjacent, recharges focus
        for x in -1..2 {
            for y in -1..2 {
                let offset = Offset { x, y };
                let position = avatar.position + offset;
                let tile = stage.tiles.grid[position.into()];
                if matches!(tile, Tile::Pylon) {
                    avatar.focus = 100;
                }
            }
        }
    };

    result
}

fn gather_info(world: &World, avatar_id: AvatarId, senses: &Senses) -> Option<SensesInfo> {
    let avatar = world.find_avatar(avatar_id)?;
    let stage = world.stages.get(avatar.stage)?;
    let senses = sense::gather(senses, avatar, stage);
    Some(senses)
}
