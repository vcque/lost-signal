use std::{collections::HashMap, net::SocketAddr, sync::Mutex};

use lost_signal::common::{action::Action, sense::Senses};
use serde_derive::{Deserialize, Serialize};

type CommandStorage = HashMap<u64, Vec<CommandMessage>>;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct CommandMessage {
    pub entity_id: u64,
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

pub struct CommandQueue {
    storage: Mutex<CommandStorage>,
}

impl CommandQueue {
    pub fn new() -> CommandQueue {
        CommandQueue {
            storage: Mutex::new(CommandStorage::new()),
        }
    }

    pub fn send_command(&self, cmd: CommandMessage) {
        let tick_id = cmd.tick;

        let lock = self.storage.lock();
        let Ok(mut queue) = lock else { panic!() };
        queue.entry(tick_id).or_insert(vec![]).push(cmd);
    }

    pub fn get_commands(&self, tick_id: u64) -> Vec<CommandMessage> {
        let Ok(mut queue) = self.storage.lock() else {
            panic!()
        };

        queue.remove(&tick_id).unwrap_or_default()
    }
}
