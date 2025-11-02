use std::sync::mpsc::Sender;

use log::info;
use losig_core::{
    sense::{
        OrbInfo, OrbSense, ProximityInfo, ProximitySense, SelfInfo, SelfSense, SenseInfo, Senses,
        TerrainInfo, TerrainSense, WorldInfo, WorldSense,
    },
    types::{Avatar, AvatarId, MAP_SIZE, Position, Tile},
};

use crate::world::World;

trait Sense {
    type Output;
    fn gather(&self, avatar: &Avatar, world: &World) -> Self::Output;

    fn gather_opt(&self, avatar: Option<&Avatar>, world: &World) -> Option<Self::Output> {
        avatar.map(|e| self.gather(e, world))
    }
}

impl Sense for WorldSense {
    type Output = WorldInfo;

    fn gather(&self, _: &Avatar, world: &World) -> Self::Output {
        self.gather_opt(None, world).unwrap()
    }

    fn gather_opt(&self, _: Option<&Avatar>, world: &World) -> Option<Self::Output> {
        Some(WorldInfo {
            tick: world.tick,
            winner: world.winner,
        })
    }
}

impl Sense for TerrainSense {
    type Output = TerrainInfo;

    fn gather(&self, avatar: &Avatar, world: &World) -> Self::Output {
        let center_x = avatar.position.x as isize;
        let center_y = avatar.position.y as isize;
        let radius = self.radius as isize;

        let mut results = vec![];
        for y in (center_y - radius)..(center_y + radius + 1) {
            for x in (center_x - radius)..(center_x + radius + 1) {
                if x < 0 || x >= MAP_SIZE as isize || y < 0 || y >= MAP_SIZE as isize {
                    results.push(Tile::Unknown);
                } else {
                    let tile = world.tiles.at(Position {
                        x: x as usize,
                        y: y as usize,
                    });
                    if matches!(tile, Tile::Spawn) {
                        info!("Found an S!");
                    }
                    results.push(tile);
                }
            }
        }

        TerrainInfo {
            radius: self.radius,
            tiles: results,
        }
    }
}

impl Sense for SelfSense {
    type Output = SelfInfo;
    fn gather(&self, avatar: &Avatar, _world: &World) -> Self::Output {
        SelfInfo {
            broken: avatar.broken,
            signal: avatar.signal,
        }
    }
}

impl Sense for ProximitySense {
    type Output = ProximityInfo;
    fn gather(&self, avatar: &Avatar, world: &World) -> Self::Output {
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

impl Sense for OrbSense {
    type Output = OrbInfo;
    fn gather(&self, avatar: &Avatar, world: &World) -> Self::Output {
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

impl<T: Sense> Sense for Option<T> {
    type Output = Option<T::Output>;

    fn gather_opt(&self, avatar: Option<&Avatar>, world: &World) -> Option<Self::Output> {
        self.as_ref().map(|s| s.gather_opt(avatar, world))
    }

    fn gather(&self, avatar: &Avatar, world: &World) -> Self::Output {
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
