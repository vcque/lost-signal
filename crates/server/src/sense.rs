use std::sync::mpsc::Sender;

use log::info;
use losig_core::{
    sense::{
        ProximityInfo, ProximitySense, SelfInfo, SelfSense, SenseInfo, Senses, TerrainInfo,
        TerrainSense, WorldInfo, WorldSense,
    },
    types::{Entity, EntityId, MAP_SIZE, Position, Tile},
};

use crate::world::World;

trait Sense {
    type Output;
    fn gather(&self, entity: &Entity, world: &World) -> Self::Output;

    fn gather_opt(&self, entity: Option<&Entity>, world: &World) -> Option<Self::Output> {
        entity.map(|e| self.gather(e, world))
    }
}

impl Sense for WorldSense {
    type Output = WorldInfo;

    fn gather(&self, _: &Entity, world: &World) -> Self::Output {
        self.gather_opt(None, world).unwrap()
    }

    fn gather_opt(&self, _: Option<&Entity>, world: &World) -> Option<Self::Output> {
        Some(WorldInfo {
            tick: world.tick,
            winner: world.winner,
        })
    }
}

impl Sense for TerrainSense {
    type Output = TerrainInfo;

    fn gather(&self, entity: &Entity, world: &World) -> Self::Output {
        let center_x = entity.position.x as isize;
        let center_y = entity.position.y as isize;
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
    fn gather(&self, entity: &Entity, _world: &World) -> Self::Output {
        SelfInfo {
            broken: entity.broken,
        }
    }
}

impl Sense for ProximitySense {
    type Output = ProximityInfo;
    fn gather(&self, entity: &Entity, world: &World) -> Self::Output {
        let radius = self.radius;
        let pos = entity.position;
        let mut count = 0;

        for foe in world.foes.iter() {
            if pos.dist(&foe.position) <= radius {
                count += 1;
            }
        }
        ProximityInfo { radius, count }
    }
}

impl<T: Sense> Sense for Option<T> {
    type Output = Option<T::Output>;

    fn gather_opt(&self, entity: Option<&Entity>, world: &World) -> Option<Self::Output> {
        self.as_ref().map(|s| s.gather_opt(entity, world))
    }

    fn gather(&self, entity: &Entity, world: &World) -> Self::Output {
        self.as_ref().map(|s| s.gather(entity, world))
    }
}

pub fn gather(senses: &Senses, entity: Option<&Entity>, world: &World) -> SenseInfo {
    SenseInfo {
        world: senses.world.gather_opt(entity, world).flatten(),
        terrain: senses.terrain.gather_opt(entity, world).flatten(),
        selfs: senses.selfs.gather_opt(entity, world).flatten(),
        proximity: senses.proximity.gather_opt(entity, world).flatten(),
    }
}

#[derive(Clone, Debug)]
pub struct SensesMessage {
    pub entity_id: EntityId,
    pub senses: SenseInfo,
}

pub type SensesQueue = Sender<SensesMessage>;
