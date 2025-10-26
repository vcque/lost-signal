use serde_derive::{Deserialize, Serialize};

use crate::common::{
    action::Action,
    sense::{SenseInfo, Senses},
    types::EntityId,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct UdpCommandPacket {
    pub entity_id: EntityId,
    /// If None, means it must use the latest tick
    pub tick: Option<u64>,
    /// Action the entity takes this tick
    pub action: Action,
    /// Info then entity wants to gather this tick
    pub senses: Senses,
}

#[derive(Serialize, Deserialize)]
pub struct UdpSensesPacket {
    pub entity_id: EntityId,
    pub senses: SenseInfo,
}
