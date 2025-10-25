use serde_derive::{Deserialize, Serialize};

use crate::common::{action::Action, sense::Senses};

#[derive(Debug, Serialize, Deserialize)]
pub struct UdpPacket {
    pub entity_id: u64,
    /// If None, means it must use the latest tick
    pub tick: Option<u64>,
    /// Action the entity takes this tick
    pub action: Action,
    /// Info then entity wants to gather this tick
    pub senses: Senses,
}
