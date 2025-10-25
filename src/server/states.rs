use std::sync::Mutex;

use crate::{command::CommandQueue, sense::SensesQueue, world::World};

/// Where we put all the states used by the server
pub struct States {
    pub world: Mutex<World>,
    pub commands: CommandQueue,
    pub senses: SensesQueue,
}
