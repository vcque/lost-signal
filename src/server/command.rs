use std::{net::SocketAddr, sync::mpsc::Sender};

use lost_signal::common::{action::Action, sense::Senses, types::EntityId};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct CommandMessage {
    pub entity_id: EntityId,
    pub tick: u64,
    pub address: Option<SocketAddr>,
    pub action: Action,
    pub senses: Senses,
}

impl Ord for CommandMessage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.tick.cmp(&other.tick)
    }
}

impl PartialOrd for CommandMessage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub type CommandQueue = Sender<CommandMessage>;
