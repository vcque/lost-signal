use std::{sync::mpsc::Receiver, thread::spawn};

use losig_core::network::{ClientMessage, ClientMessageContent};

use crate::{game::Game, services::Services};

pub struct Dispatch {
    services: Services,
    cm_rx: Receiver<ClientMessage>,
}

impl Dispatch {
    pub fn new(services: Services, cm_rx: Receiver<ClientMessage>) -> Self {
        Self { services, cm_rx }
    }

    pub fn run(self) {
        spawn(move || {
            let game = Game::new(self.services.clone());
            while let Ok(msg) = self.cm_rx.recv() {
                match msg.content {
                    ClientMessageContent::Command(cmd) => {
                        game.enact(cmd);
                    }
                    ClientMessageContent::Leaderboard => {
                        // TODO: send leaderboard
                    }
                    ClientMessageContent::LeaderboardSubmit(_id, _name) => {
                        // TODO: update leaderboard and broadcast it
                    }
                }
            }
        });
    }
}
