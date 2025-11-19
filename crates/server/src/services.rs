use crate::ws_server::ServerMessageWithRecipient;

use std::sync::mpsc::Sender;

use losig_core::leaderboard::Leaderboard;

use crate::world::World;

use std::sync::Mutex;

use std::sync::Arc;

#[derive(Clone)]
pub struct Services {
    pub world: Arc<Mutex<World>>,
    pub leaderboard: Arc<Mutex<Leaderboard>>,
    pub sender: Sender<ServerMessageWithRecipient>,
}

impl Services {
    pub(crate) fn new(
        world: World,
        leaderboard: Leaderboard,
        sender: Sender<ServerMessageWithRecipient>,
    ) -> Self {
        Services {
            world: Arc::new(Mutex::new(world)),
            leaderboard: Arc::new(Mutex::new(leaderboard)),
            sender,
        }
    }
}
