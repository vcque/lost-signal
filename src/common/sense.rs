use std::time::Duration;

use serde_derive::{Deserialize, Serialize};

/// Describe information that an entity want retrieved for a given turn
#[derive(Default, Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct Senses {
    /// Retrieve info about the general world
    pub world: Option<WorldSense>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct SenseInfo {
    pub world: Option<WorldInfo>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct WorldInfo {
    pub tick: u64,
    pub tick_duration: Duration,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct WorldSense {}
