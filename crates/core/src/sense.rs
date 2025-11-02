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

impl Senses {
    pub fn signal_cost(&self) -> usize {
        let mut cost = 0;
        cost += self.world.signal_cost();
        cost += self.terrain.signal_cost();
        cost += self.selfs.signal_cost();
        cost += self.proximity.signal_cost();
        cost += self.orb.signal_cost();
        cost
    }
}

pub trait Sense {
    type Info;
    fn signal_cost(&self) -> usize;
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

impl Sense for WorldSense {
    type Info = WorldInfo;
    fn signal_cost(&self) -> usize {
        1
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct WorldInfo {
    pub tick: u64,
    pub winner: Option<AvatarId>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct TerrainSense {
    pub radius: usize,
}

impl Sense for TerrainSense {
    type Info = TerrainInfo;
    fn signal_cost(&self) -> usize {
        let cost = 2 * self.radius + 1; // Number of tiles discovered
        cost * cost / 10
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct TerrainInfo {
    pub radius: usize,
    pub tiles: Vec<Tile>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct SelfSense {}

impl Sense for SelfSense {
    type Info = SelfInfo;
    fn signal_cost(&self) -> usize {
        1
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct SelfInfo {
    pub broken: bool,
    pub signal: usize,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct ProximitySense {
    pub radius: usize,
}

impl Sense for ProximitySense {
    type Info = ProximityInfo;
    fn signal_cost(&self) -> usize {
        let cost = self.radius + 1;
        1 + cost * cost / 10
    }
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

impl Sense for OrbSense {
    type Info = OrbInfo;
    fn signal_cost(&self) -> usize {
        match self.level {
            SenseLevel::Minimum => 1,
            SenseLevel::Low => 2,
            SenseLevel::Medium => 3,
            SenseLevel::High => 4,
            SenseLevel::Maximum => 5,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct OrbInfo {
    pub detected: bool,
    pub owned: bool,
}

impl<T: Sense> Sense for Option<T> {
    type Info = Option<T::Info>;
    fn signal_cost(&self) -> usize {
        match self {
            Some(s) => s.signal_cost(),
            None => 0,
        }
    }
}
