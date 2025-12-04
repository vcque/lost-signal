use bounded_integer::BoundedU8;
use losig_core::{
    fov,
    sense::{
        HearingInfo, SelfInfo, SenseStrength, Senses, SensesInfo, SightInfo, SightedAlly,
        SightedAllyStatus, SightedFoe, TouchInfo,
    },
    types::{Avatar, PlayerId, ServerAction, Tile},
};

use crate::stage::{Stage, StagePlayer, StageState};

pub fn gather(senses: &Senses, stage: &Stage, pid: PlayerId) -> SensesInfo {
    let player = &stage.players[&pid];
    let state = &stage.state_for(pid).unwrap();
    let tail_state = stage.tail_state();
    let avatar = &state.avatars[&pid];

    SensesInfo {
        selfi: try_gather(senses.selfs, |_| gather_self(player, avatar, tail_state)),
        touch: try_gather(senses.touch, |_| gather_touch(avatar, stage, state)),
        sight: try_gather(senses.sight, |strength| {
            gather_sight(strength.get(), avatar, stage, state)
        }),
        hearing: try_gather(senses.hearing, |strength| {
            gather_hearing(strength.get(), avatar, stage, state)
        }),
    }
}

fn gather_hearing(
    strength: u8,
    avatar: &Avatar,
    _async_stage: &Stage,
    state: &StageState,
) -> HearingInfo {
    let dist = avatar.position.dist(&state.orb.position) as u8;

    for s in 1..(strength + 1) {
        if let Some(range) = HearingInfo::dist(s)
            && dist <= range
        {
            return HearingInfo {
                range: BoundedU8::new(s),
            };
        }
    }

    HearingInfo { range: None }
}

fn gather_sight(strength: u8, avatar: &Avatar, stage: &Stage, state: &StageState) -> SightInfo {
    let tiles = fov::fov(avatar.position, strength.into(), &stage.template.tiles);
    let mut foes = vec![];

    let center = tiles.center();
    for (i, foe) in state.foes.iter().enumerate() {
        if foe.is_trap() {
            // Traps can't be seen
            continue;
        }
        let offset = foe.position - avatar.position;
        let fov_position = center + offset;

        if tiles.get(fov_position) != Tile::Unknown {
            foes.push(SightedFoe {
                id: i,
                offset,
                foe_type: foe.foe_type,
                alive: foe.alive(),
            });
        }
    }

    let offset = state.orb.position - avatar.position;
    let fov_position = center + offset;
    let orb = match tiles.get(fov_position) {
        Tile::Unknown => None,
        _ => Some(offset),
    };

    let mut allies = vec![];
    for ally in state.avatars.values() {
        let offset = ally.position - avatar.position;
        let fov_position = center + offset;
        if tiles.get(fov_position) == Tile::Unknown {
            // Not in fov
            continue;
        }

        let avatar_tracker = stage.players.get(&ally.player_id);
        let move_offset = if state.turn < stage.head_turn {
            let i = stage.diff_index(state.turn + 1);
            stage
                .diffs
                .get(i)
                .and_then(|d| d.cmd_by_avatar.get(&ally.player_id))
                .and_then(|cmd| match cmd.action {
                    ServerAction::Move(pos) => Some(pos),
                    _ => None,
                })
                .map(|pos| pos - avatar.position)
        } else {
            None
        };
        let status = if let Some(tracker) = avatar_tracker {
            SightedAllyStatus::Controlled {
                turn: tracker.turn,
                name: tracker.player_name.clone(),
            }
        } else {
            SightedAllyStatus::Discarded
        };

        allies.push(SightedAlly {
            name: avatar_tracker.map(|at| at.player_name.clone()),
            offset,
            alive: !avatar.is_dead(),
            status,
            next_move: move_offset,
        });
    }

    SightInfo {
        tiles,
        foes,
        orb,
        allies,
    }
}

fn gather_touch(avatar: &Avatar, async_stage: &Stage, state: &StageState) -> TouchInfo {
    let tiles = fov::fov(avatar.position, 1, &async_stage.template.tiles);

    let mut foes = vec![];
    let mut traps = 0;
    for foe in &state.foes {
        if foe.alive() && foe.position.dist(&avatar.position) <= 1 {
            if foe.is_trap() {
                traps += 1;
            } else {
                foes.push(foe.position - avatar.position);
            }
        }
    }

    TouchInfo {
        tiles,
        foes,
        traps,
        orb: state.orb.position.dist(&avatar.position) <= 1,
    }
}

fn gather_self(player: &StagePlayer, avatar: &Avatar, tail_state: &StageState) -> SelfInfo {
    let hp_max = match tail_state.avatars.get(&avatar.player_id) {
        Some(avatar) => avatar.hp,
        None => 10,
    };

    let hp_max = hp_max.max(avatar.hp);
    SelfInfo {
        focus: player.focus,
        hp: avatar.hp,
        hp_max,
        turn: avatar.turns,
    }
}

fn try_gather<F, Strength: SenseStrength + Eq, Info>(strength: Strength, gather: F) -> Option<Info>
where
    F: FnOnce(Strength) -> Info,
{
    if strength == Strength::min() {
        None
    } else {
        Some(gather(strength))
    }
}
