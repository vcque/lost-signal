use std::sync::Mutex;

use crate::{command::CommandQueue, world::World};

/// Where we put all the states used by the server

pub struct States {
    pub world: Mutex<World>,
    pub command_queue: CommandQueue,
}
