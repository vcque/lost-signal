use serde::{Deserialize, Serialize};

use crate::{
    leaderboard::Leaderboard,
    sense::{Senses, SensesInfo},
    types::{Action, AvatarId, GameOver, Turn},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandMessage {
    pub avatar_id: AvatarId,
    /// The avatar's turn. Used to keep track of which response corresponds to which command
    pub turn: Turn,
    /// Action the avatar takes this tick
    pub action: Action,
    /// Info then avatar wants to gather this tick
    pub senses: Senses,
}

#[derive(Serialize, Deserialize)]
pub struct TurnResultMessage {
    /// The avatar's turn. Used to Keep track of which resposne corresponds to which command
    pub avatar_id: AvatarId,
    pub turn: Turn,
    pub stage: u8,
    pub info: SensesInfo,
}

pub type GameOverMessage = GameOver;

#[derive(Serialize, Deserialize)]
pub struct ClientMessage {
    pub avatar_id: Option<AvatarId>,
    pub content: ClientMessageContent,
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessageContent {
    Start(AvatarId),
    Leaderboard,
    LeaderboardSubmit(AvatarId, String),
    Command(CommandMessage),
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    Leaderboard(Leaderboard),
    Turn(TurnResultMessage),
    GameOver(GameOverMessage),
}
