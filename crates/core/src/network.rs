use serde::{Deserialize, Serialize};

use crate::{
    sense::{SenseInfo, Senses},
    types::{Action, AvatarId},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct UdpCommandPacket {
    pub avatar_id: AvatarId,
    /// If None, means it must use the latest tick
    pub tick: Option<u64>,
    /// Action the avatar takes this tick
    pub action: Action,
    /// Info then avatar wants to gather this tick
    pub senses: Senses,
}

#[derive(Serialize, Deserialize)]
pub struct UdpSensesPacket {
    pub avatar_id: AvatarId,
    pub senses: SenseInfo,
}
