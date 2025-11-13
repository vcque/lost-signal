use std::sync::mpsc::Sender;

use losig_core::{
    sense::{
        OrbInfo, OrbSense, ProximityInfo, ProximitySense, SelfInfo, SelfSense, Sense, SenseInfo,
        Senses, TerrainInfo, TerrainSense, WorldInfo, WorldSense,
    },
    types::{Avatar, AvatarId},
};

use crate::{fov, world::World};

trait ServerSense: Sense {
    fn gather(&self, avatar: &Avatar, world: &World) -> Self::Info;

    fn gather_opt(&self, avatar: Option<&Avatar>, world: &World) -> Option<Self::Info> {
        avatar.map(|e| self.gather(e, world))
    }
}

impl ServerSense for WorldSense {
    fn gather(&self, _: &Avatar, world: &World) -> Self::Info {
        self.gather_opt(None, world).unwrap()
    }

    fn gather_opt(&self, _: Option<&Avatar>, world: &World) -> Option<Self::Info> {
        Some(WorldInfo {
            tick: world.tick,
            winner: world.winner,
        })
    }
}

impl ServerSense for TerrainSense {
    fn gather(&self, avatar: &Avatar, world: &World) -> Self::Info {
        let tiles = fov::fov(avatar.position, self.radius, &world.tiles);
        TerrainInfo {
            radius: self.radius,
            tiles: tiles.buf,
        }
    }
}

impl ServerSense for SelfSense {
    fn gather(&self, avatar: &Avatar, _world: &World) -> Self::Info {
        SelfInfo {
            broken: avatar.broken,
            signal: avatar.signal,
        }
    }
}

impl ServerSense for ProximitySense {
    fn gather(&self, avatar: &Avatar, world: &World) -> Self::Info {
        let radius = self.radius;
        let pos = avatar.position;
        let mut count = 0;

        for foe in world.foes.iter() {
            if pos.dist(&foe.position) <= radius {
                count += 1;
            }
        }
        ProximityInfo { radius, count }
    }
}

impl ServerSense for OrbSense {
    fn gather(&self, avatar: &Avatar, world: &World) -> Self::Info {
        let detected = world
            .orb
            .map(|pos| pos.dist(&avatar.position))
            .map(|d| d <= self.level.range())
            .unwrap_or(false);

        OrbInfo {
            owned: world.winner == Some(avatar.id),
            detected,
        }
    }
}

impl<T: ServerSense> ServerSense for Option<T> {
    fn gather_opt(&self, avatar: Option<&Avatar>, world: &World) -> Option<Self::Info> {
        self.as_ref().map(|s| s.gather_opt(avatar, world))
    }

    fn gather(&self, avatar: &Avatar, world: &World) -> Self::Info {
        self.as_ref().map(|s| s.gather(avatar, world))
    }
}

pub fn gather(senses: &Senses, avatar: Option<&Avatar>, world: &World) -> SenseInfo {
    SenseInfo {
        world: senses.world.gather_opt(avatar, world).flatten(),
        terrain: senses.terrain.gather_opt(avatar, world).flatten(),
        selfs: senses.selfs.gather_opt(avatar, world).flatten(),
        proximity: senses.proximity.gather_opt(avatar, world).flatten(),
        orb: senses.orb.gather_opt(avatar, world).flatten(),
    }
}

#[derive(Clone, Debug)]
pub struct SensesMessage {
    pub avatar_id: AvatarId,
    pub senses: SenseInfo,
}

pub type SensesQueue = Sender<SensesMessage>;
