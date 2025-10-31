use serde::{Deserialize, Serialize};

use crate::types::{EntityId, Tile};

/// Describe information that an entity want retrieved for a given turn
#[derive(Default, Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct Senses {
    /// Retrieve general info about the world
    pub world: Option<WorldSense>,
    pub terrain: Option<TerrainSense>,
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

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct TerrainSense {
    pub radius: usize,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct TerrainInfo {
    pub radius: usize,
    pub tiles: Vec<Tile>,
}
