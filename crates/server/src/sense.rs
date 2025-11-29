use bounded_integer::BoundedU8;
use losig_core::{
    sense::{HearingInfo, SelfInfo, SenseStrength, Senses, SensesInfo, SightInfo, TouchInfo},
    types::{Avatar, Tile},
};

use crate::{
    fov,
    stage::{Stage, StageState},
};

pub fn gather(
    senses: &Senses,
    avatar: &Avatar,
    async_stage: &Stage,
    state: &StageState,
) -> SensesInfo {
    SensesInfo {
        selfi: try_gather(senses.selfs, |_| gather_self(avatar)),
        touch: try_gather(senses.touch, |_| gather_touch(avatar, async_stage, state)),
        sight: try_gather(senses.sight, |strength| {
            gather_sight(strength.get(), avatar, async_stage, state)
        }),
        hearing: try_gather(senses.hearing, |strength| {
            gather_hearing(strength.get(), avatar, async_stage, state)
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
    for foe in &state.foes {
        let offset = foe.position() - avatar.position;
        let fov_position = center + offset;

        if tiles.get(fov_position) == Tile::Empty {
            foes.push(offset);
        }
    }

    let offset = state.orb.position - avatar.position;
    let fov_position = center + offset;
    let orb = match tiles.get(fov_position) {
        Tile::Empty => Some(offset),
        _ => None,
    };

    let mut allies = vec![];
    for ally in state.avatars.values() {
        if ally.is_dead() {
            continue;
        }
        let offset = ally.position - avatar.position;
        let fov_position = center + offset;
        if !tiles.get(fov_position).opaque() {
            allies.push(offset);
        }
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

    let mut foes = 0;
    for foe in &state.foes {
        if foe.position().dist(&avatar.position) <= 1 {
            foes += 1;
        }
    }

    TouchInfo {
        tiles,
        foes,
        orb: state.orb.position.dist(&avatar.position) <= 1,
    }
}

fn gather_self(avatar: &Avatar) -> SelfInfo {
    SelfInfo {
        focus: avatar.focus,
        hp: avatar.hp,
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
