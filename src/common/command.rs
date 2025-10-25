use serde_derive::{Deserialize, Serialize};

use crate::common::types::Direction;

/**
* Lists all possible commands that can be sent by a player to the game.
* A command is an input that (often) leads to a modification of the game state.
*/
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum Command {
    Spawn,
    Move(Direction),
}
