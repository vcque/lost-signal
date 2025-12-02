use losig_core::types::{
    Avatar, ClientAction, Direction, FOCUS_MAX, FoeType, GameLogEvent, HP_MAX, PlayerId, Position,
    ServerAction, Target,
};

use crate::stage::{Stage, StageState};

/// Execute an action for an avatar
pub fn act(action: &ServerAction, avatar: &mut Avatar, state: &mut StageState, stage: &Stage) {
    match action {
        ServerAction::Spawn => act_spawn(avatar, stage, state),
        ServerAction::Move(position) => act_move(avatar, *position),
        ServerAction::Attack(target_index) => act_attack(avatar, *target_index, state),
        ServerAction::Wait => {}
    }
}

fn act_spawn(avatar: &mut Avatar, stage: &Stage, state: &StageState) {
    let spawn_position = stage.find_spawns();
    avatar.position = spawn_position[avatar.player_id as usize % spawn_position.len()];
    avatar.hp = HP_MAX;
    avatar.focus = FOCUS_MAX;
    avatar.logs.push((state.turn, GameLogEvent::Spawn));
}

fn act_move(avatar: &mut Avatar, position: Position) {
    avatar.position = position;
}

fn act_attack(avatar: &mut Avatar, target_index: usize, state: &mut StageState) {
    if let Some(foe) = state.foes.get_mut(target_index)
        && foe.can_be_attacked()
        && foe.position.dist(&avatar.position) <= 1
    {
        foe.hp = foe.hp.saturating_sub(1);
        avatar.logs.push((
            state.turn,
            GameLogEvent::Attack {
                from: Target::You,
                to: Target::Foe(FoeType::Simple),
            },
        ));
    }
}

pub fn convert_client(action: ClientAction, stage: &mut Stage, pid: PlayerId) -> ServerAction {
    match action {
        ClientAction::Spawn => ServerAction::Spawn,
        ClientAction::MoveOrAttack(direction) => {
            convert_move_or_attack_action(direction, stage, pid).unwrap_or(ServerAction::Wait)
        }
        ClientAction::Wait => ServerAction::Wait,
    }
}

fn convert_move_or_attack_action(dir: Direction, stage: &Stage, aid: u32) -> Option<ServerAction> {
    let state = stage.state_for(aid)?;
    let avatar = state.avatars.get(&aid)?;

    let next_pos = avatar.position + dir.offset();

    if let Some((id, foe)) = state.find_foe(next_pos)
        && foe.can_be_attacked()
    {
        return Some(ServerAction::Attack(id));
    }

    let tile = stage.template.tiles.get(next_pos);
    if tile.can_travel() {
        Some(ServerAction::Move(next_pos))
    } else {
        None
    }
}
