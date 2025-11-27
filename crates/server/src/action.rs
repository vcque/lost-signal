use log::debug;
use losig_core::types::{Avatar, AvatarId, ClientAction, Direction, Foe, Position, ServerAction};

use crate::world::{Stage, StageState};

/// Execute an action for an avatar
pub fn act(action: &ServerAction, avatar: &mut Avatar, state: &mut StageState, stage: &Stage) {
    match action {
        ServerAction::Spawn => act_spawn(avatar, stage),
        ServerAction::Move(position) => act_move(avatar, *position),
        ServerAction::Attack(target_index) => act_attack(avatar, *target_index, state),
        ServerAction::Wait => {}
    }
}

fn act_spawn(avatar: &mut Avatar, stage: &Stage) {
    let spawn_position = stage.find_spawns();
    avatar.position = spawn_position[avatar.id as usize % spawn_position.len()];
    avatar.hp = 10;
    avatar.focus = 100;
}

fn act_move(avatar: &mut Avatar, position: Position) {
    avatar.position = position;
}

fn act_attack(_avatar: &Avatar, target_index: usize, state: &mut StageState) {
    if let Some(foe) = state.foes.get_mut(target_index)
        && let Foe::Simple(_, hp) = foe
    {
        *hp = hp.saturating_sub(1);
    }
    state.foes.retain(|f| f.alive());
}

pub fn convert_client(action: ClientAction, stage: &mut Stage, aid: AvatarId) -> ServerAction {
    match action {
        ClientAction::Spawn => ServerAction::Spawn,
        ClientAction::MoveOrAttack(direction) => {
            convert_move_or_attack_action(direction, stage, aid).unwrap_or(ServerAction::Wait)
        }
        ClientAction::Wait => ServerAction::Wait,
    }
}

fn convert_move_or_attack_action(dir: Direction, stage: &Stage, aid: u32) -> Option<ServerAction> {
    debug!("convert");
    let state = stage.state_for(aid)?;
    debug!("has state");
    let avatar = state.avatars.get(&aid)?;
    debug!("has avatar");

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
