use bounded_integer::BoundedU8;
use losig_core::{
    sense::{HearingInfo, SelfInfo, SenseStrength, Senses, SensesInfo, SightInfo, TouchInfo},
    types::{Avatar, Tile},
};

use crate::{fov, world::Stage};

pub fn gather(senses: &Senses, avatar: &Avatar, stage: &Stage) -> SensesInfo {
    SensesInfo {
        selfi: try_gather(senses.selfs, |_| gather_self(avatar)),
        touch: try_gather(senses.touch, |_| gather_touch(avatar, stage)),
        sight: try_gather(senses.sight, |strength| {
            gather_sight(strength.get(), avatar, stage)
        }),
        hearing: try_gather(senses.hearing, |strength| {
            gather_hearing(strength.get(), avatar, stage)
        }),
    }
}

fn gather_hearing(strength: u8, avatar: &Avatar, stage: &Stage) -> HearingInfo {
    let dist = avatar.position.dist(&stage.orb) as u8;

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

fn gather_sight(strength: u8, avatar: &Avatar, stage: &Stage) -> SightInfo {
    let tiles = fov::fov(avatar.position, strength.into(), &stage.tiles);
    let mut foes = vec![];

    let center = tiles.center();
    for foe in &stage.foes {
        let offset = foe.position - avatar.position;
        let fov_position = center + offset;

        if tiles.get(fov_position) == Tile::Empty {
            foes.push(offset);
        }
    }

    let offset = stage.orb - avatar.position;
    let fov_position = center + offset;
    let orb = match tiles.get(fov_position) {
        Tile::Empty => Some(offset),
        _ => None,
    };

    SightInfo { tiles, foes, orb }
}

fn gather_touch(avatar: &Avatar, stage: &Stage) -> TouchInfo {
    // TODO: use a method to copy Tiles into another
    let tiles = fov::fov(avatar.position, 1, &stage.tiles);

    let mut foes = 0;
    for foe in &stage.foes {
        if foe.position.dist(&avatar.position) <= 1 {
            foes += 1;
        }
    }

    TouchInfo {
        tiles,
        foes,
        orb: stage.orb.dist(&avatar.position) <= 1,
    }
}

fn gather_self(avatar: &Avatar) -> SelfInfo {
    SelfInfo {
        signal: avatar.signal,
        stage: avatar.stage as u8,
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
