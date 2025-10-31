use serde::{Deserialize, Serialize};

use crate::types::{EntityId, Tile};

/// Describe information that an entity want retrieved for a given turn
#[derive(Default, Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct Senses {
    /// Retrieve general info about the world
    pub world: Option<WorldSense>,
    pub terrain: Option<TerrainSense>,
}

/// Represents one of the senses of an entity
pub trait Sense {
    /// Make it stronger
    fn incr(&mut self);
    /// Make it weaker
    fn decr(&mut self);
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct SenseInfo {
    pub world: Option<WorldInfo>,
    pub terrain: Option<TerrainInfo>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct WorldInfo {
    pub tick: u64,
    pub winner: Option<EntityId>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct WorldSense {}

impl Sense for Option<WorldSense> {
    fn decr(&mut self) {
        self.take();
    }

    fn incr(&mut self) {
        self.replace(WorldSense {});
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct TerrainSense {
    pub radius: usize,
}

impl Sense for Option<TerrainSense> {
    fn incr(&mut self) {
        match self {
            Some(w) => w.radius += 1,
            None => {
                self.replace(TerrainSense { radius: 1 });
            }
        }
    }

    fn decr(&mut self) {
        match self {
            Some(w) => {
                if w.radius == 1 {
                    self.take();
                } else {
                    w.radius -= 1
                }
            }
            None => {}
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct TerrainInfo {
    pub radius: usize,
    pub tiles: Vec<Tile>,
}
