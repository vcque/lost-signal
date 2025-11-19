use losig_core::{
    sense::{
        DangerInfo, DangerSense, OrbInfo, OrbSense, SelfInfo, SelfSense, Sense, SenseInfo, Senses,
        TerrainInfo, TerrainSense,
    },
    types::Avatar,
};

use crate::{fov, world::Stage};

trait ServerSense: Sense {
    fn gather(&self, avatar: &Avatar, stage: &Stage) -> Self::Info;
}

impl ServerSense for TerrainSense {
    fn gather(&self, avatar: &Avatar, stage: &Stage) -> Self::Info {
        let tiles = fov::fov(avatar.position, self.radius, &stage.tiles);
        TerrainInfo {
            radius: self.radius,
            tiles: tiles.buf,
        }
    }
}

impl ServerSense for SelfSense {
    fn gather(&self, avatar: &Avatar, _stage: &Stage) -> Self::Info {
        SelfInfo {
            broken: avatar.broken,
            signal: avatar.signal,
            winner: avatar.winner,
            stage: avatar.stage,
        }
    }
}

impl ServerSense for DangerSense {
    fn gather(&self, avatar: &Avatar, stage: &Stage) -> Self::Info {
        let radius = self.radius;
        let pos = avatar.position;
        let mut count = 0;

        for foe in stage.foes.iter() {
            if pos.dist(&foe.position) <= radius {
                count += 1;
            }
        }
        DangerInfo { radius, count }
    }
}

impl ServerSense for OrbSense {
    fn gather(&self, avatar: &Avatar, stage: &Stage) -> Self::Info {
        let detected = stage.orb.dist(&avatar.position) <= self.level.range();
        OrbInfo { detected }
    }
}

impl<T: ServerSense> ServerSense for Option<T> {
    fn gather(&self, avatar: &Avatar, stage: &Stage) -> Self::Info {
        self.as_ref().map(|s| s.gather(avatar, stage))
    }
}

pub fn gather(senses: &Senses, avatar: &Avatar, stage: &Stage) -> SenseInfo {
    SenseInfo {
        terrain: senses.terrain.gather(avatar, stage),
        selfi: senses.selfs.gather(avatar, stage),
        danger: senses.danger.gather(avatar, stage),
        orb: senses.orb.gather(avatar, stage),
    }
}
