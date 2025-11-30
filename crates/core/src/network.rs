use serde::{Deserialize, Serialize};

use crate::{
    leaderboard::Leaderboard,
    sense::{Senses, SensesInfo},
    types::{
        ClientAction, GameLogEvent, GameOver, PlayerId, ServerAction, StageTurn, Timeline, Turn,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandMessage {
    pub avatar_id: PlayerId,
    /// The avatar's turn. Used to keep track of which response corresponds to which command
    pub turn: Turn,
    /// Action the avatar takes this tick
    pub action: ClientAction,
    /// Info then avatar wants to gather this tick
    pub senses: Senses,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TurnResultMessage {
    pub avatar_id: PlayerId,
    /// The avatar's turn. Used to Keep track of which response corresponds to which command
    pub turn: Turn,
    /// The stage turn, interesting info to know where people are relative to each other
    pub stage_turn: Turn,
    pub stage: u8,
    pub info: SensesInfo,
    pub action: ServerAction,
    pub logs: GameLogsMessage,
}

pub struct LimboMessage {}

pub type GameOverMessage = GameOver;

#[derive(Serialize, Deserialize)]
pub struct ClientMessage {
    pub avatar_id: Option<PlayerId>,
    pub content: ClientMessageContent,
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessageContent {
    Start(PlayerId),
    Leaderboard,
    LeaderboardSubmit(PlayerId, String),
    Command(CommandMessage),
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Leaderboard(Leaderboard),
    Turn(TurnResultMessage),
    GameOver(GameOverMessage),
    Limbo {
        averted: bool,
        senses_info: Option<SensesInfo>,
    },

    /// Sent when someone plays, it updates where the head and tail of the stage is
    Timeline(u8, Timeline),
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GameLogsMessage {
    /// From wich stage turn the game logs has been sent
    pub from: StageTurn,

    /// Logs computed from server. Ordered incr by stage turn
    pub logs: Vec<(StageTurn, GameLogEvent)>,
}
