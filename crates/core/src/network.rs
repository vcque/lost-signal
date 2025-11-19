use serde::{Deserialize, Serialize};

use crate::{
    leaderboard::Leaderboard,
    sense::{SenseInfo, Senses},
    types::{Action, AvatarId},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct UdpCommandPacket {
    pub avatar_id: AvatarId,
    /// The avatar's turn. Used to keep track of which response corresponds to which command
    pub turn: u64,
    /// Action the avatar takes this tick
    pub action: Action,
    /// Info then avatar wants to gather this tick
    pub senses: Senses,
}

#[derive(Serialize, Deserialize)]
pub struct UdpSensesPacket {
    /// The avatar's turn. Used to Keep track of which resposne corresponds to which command
    pub turn: u64,
    pub avatar_id: AvatarId,
    pub senses: SenseInfo,
}

#[derive(Serialize, Deserialize)]
pub struct ClientMessage {
    pub avatar_id: Option<AvatarId>,
    pub content: ClientMessageContent,
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessageContent {
    Leaderboard,
    LeaderboardSubmit(AvatarId, String),
    Command(UdpCommandPacket),
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    Leaderboard(Leaderboard),
    Senses(UdpSensesPacket),
}
