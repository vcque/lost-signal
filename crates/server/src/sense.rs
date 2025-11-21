use losig_core::{
    sense::{SelfInfo, SenseStrength, Senses, SensesInfo, SightInfo, TouchInfo},
    types::Avatar,
};

use crate::{fov, world::Stage};

pub fn gather(senses: &Senses, avatar: &Avatar, stage: &Stage) -> SensesInfo {
    SensesInfo {
        selfi: try_gather(senses.selfs, |_| gather_self(avatar)),
        touch: try_gather(senses.touch, |_| gather_touch(avatar, stage)),
        sight: try_gather(senses.sight, |strength| {
            gather_sight(strength.get(), avatar, stage)
        }),
    }
}

fn gather_sight(strength: u8, avatar: &Avatar, stage: &Stage) -> SightInfo {
    let tiles = fov::fov(avatar.position, strength.into(), &stage.tiles);
    let foes = vec![];

    SightInfo {
        tiles,
        foes,
        orb: None,
    }
}

fn gather_touch(avatar: &Avatar, stage: &Stage) -> TouchInfo {
    // TODO: use a method to copy Tiles into another
    let tiles = fov::fov(avatar.position, 1, &stage.tiles);
    let foes = vec![];

    TouchInfo {
        tiles,
        foes,
        orb: None,
    }
}

fn gather_self(avatar: &Avatar) -> SelfInfo {
    SelfInfo {
        broken: avatar.broken,
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
