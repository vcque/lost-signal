use std::time::Duration;

use serde_derive::{Deserialize, Serialize};

use crate::game::TICK_DURATION;
use crate::{entity::Entity, world::World};

/// Describe information that an entity want retrieved for a given turn
#[derive(Default, Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct Senses {
    /// Retrieve info about the general world
    pub world: Option<WorldSense>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct SenseInfo {
    world: Option<WorldInfo>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct WorldInfo {
    tick: u64,
    tick_duration: Duration,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct WorldSense {}

impl Sense for WorldSense {
    type Output = WorldInfo;

    fn gather(&self, _: &Entity, world: &World) -> Self::Output {
        self.gather_opt(None, world).unwrap()
    }

    fn gather_opt(&self, _: Option<&Entity>, world: &World) -> Option<Self::Output> {
        Some(WorldInfo {
            tick: world.tick,
            tick_duration: TICK_DURATION,
        })
    }
}

trait Sense {
    type Output;
    fn gather(&self, entity: &Entity, world: &World) -> Self::Output;

    fn gather_opt(&self, entity: Option<&Entity>, world: &World) -> Option<Self::Output> {
        entity.map(|e| self.gather(e, world))
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
    }
}
