use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::world::Direction;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CommandMessage {
    pub entity_id: u64,
    pub tick_id: u64,
    pub content: Command,
}

impl Ord for CommandMessage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.tick_id.cmp(&other.tick_id)
    }
}

impl PartialOrd for CommandMessage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/**
* Lists all possible commands that can be sent by a player to the game.
* A command is an input that (often) leads to a modification of the game state.
*/
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Command {
    Spawn,
    Move(Direction),
}

type CommandStorage = HashMap<u64, Vec<CommandMessage>>;

#[derive(Clone)]
pub struct CommandQueue {
    storage: Arc<Mutex<CommandStorage>>,
}

impl CommandQueue {
    pub fn new() -> CommandQueue {
        CommandQueue {
            storage: Arc::new(Mutex::new(CommandStorage::new())),
        }
    }

    pub fn send_command(&self, cmd: CommandMessage) {
        let tick_id = cmd.tick_id;

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
