use losig_core::{
    events::{GameEvent, Target},
    types::{Avatar, ClientAction, Direction, PlayerId, Position, ServerAction},
};

use crate::{
    events::{EventSenses, EventSource, GameEventSource},
    stage::{Stage, StageState},
};

/// Execute an action for an avatar
pub fn act(action: &ServerAction, avatar: &mut Avatar, state: &mut StageState, _stage: &Stage) {
    match action {
        ServerAction::Move(position) => act_move(avatar, *position),
        ServerAction::Attack(target_index) => act_attack(avatar, *target_index, state),
        ServerAction::Wait | ServerAction::Enter => {}
    }
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

        let event = if foe.alive() {
            GameEvent::Attack {
                subject: Target::Foe(foe.foe_type),
                source: Target::Avatar(avatar.player_id),
            }
        } else {
            GameEvent::Kill {
                subject: Target::Foe(foe.foe_type),
                source: Target::Avatar(avatar.player_id),
            }
        };

        state.events.add(GameEventSource {
            senses: EventSenses::All,
            source: EventSource::Position(foe.position),
            event,
        });
    } else {
        state.events.add(GameEventSource {
            senses: EventSenses::All,
            source: EventSource::Position(avatar.position),
            event: GameEvent::Fumble(Target::Avatar(avatar.player_id)),
        });
    }
}

pub fn convert_client(action: ClientAction, stage: &mut Stage, pid: PlayerId) -> ServerAction {
    match action {
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
