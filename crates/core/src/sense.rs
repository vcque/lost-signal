use serde::{Deserialize, Serialize};

use crate::types::{AvatarId, Tile};

/// Describe information that an avatar want retrieved for a given turn
#[derive(Default, Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct Senses {
    /// Retrieve general info about the world
    pub world: Option<WorldSense>,
    pub terrain: Option<TerrainSense>,
    pub selfs: Option<SelfSense>,
    pub proximity: Option<ProximitySense>,
    pub orb: Option<OrbSense>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct SenseInfo {
    pub world: Option<WorldInfo>,
    pub terrain: Option<TerrainInfo>,
    pub selfs: Option<SelfInfo>,
    pub proximity: Option<ProximityInfo>,
    pub orb: Option<OrbInfo>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy, PartialOrd, Ord)]
pub enum SenseLevel {
    Minimum,
    Low,
    Medium,
    High,
    Maximum,
}

impl SenseLevel {
    pub fn range(&self) -> usize {
        match self {
            Self::Minimum => 3,
            Self::Low => 6,
            Self::Medium => 9,
            Self::High => 12,
            Self::Maximum => 15,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct WorldSense {}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct WorldInfo {
    pub tick: u64,
    pub winner: Option<AvatarId>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct TerrainSense {
    pub radius: usize,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct TerrainInfo {
    pub radius: usize,
    pub tiles: Vec<Tile>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct SelfSense {}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct SelfInfo {
    pub broken: bool,
    pub signal: usize,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct ProximitySense {
    pub radius: usize,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct ProximityInfo {
    pub radius: usize,
    pub count: usize,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct OrbSense {
    pub level: SenseLevel,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct OrbInfo {
    pub detected: bool,
    pub owned: bool,
}
