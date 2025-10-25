use lost_signal::common::{
    sense::{SenseInfo, Senses, WorldInfo, WorldSense},
    types::Entity,
};

use crate::{game::TICK_DURATION, world::World};

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
            tick_duration: TICK_DURATION,
        })
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
