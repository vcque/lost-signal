use std::{sync::mpsc::Receiver, thread::spawn};

use log::error;
use losig_core::{
    leaderboard::LeaderboardEntry,
    network::{ClientMessage, ClientMessageContent, ServerMessage},
};

use crate::{
    game::Game,
    services::Services,
    ws_server::{Recipient, ServerMessageWithRecipient},
};

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
            let mut game = Game::new(self.services.clone());

            while let Ok(msg) = self.cm_rx.recv() {
                match msg.content {
                    ClientMessageContent::Start(pid) => {
                        game.new_player(pid);
                    }
                    ClientMessageContent::Command(cmd) => {
                        if let Err(e) = game.player_command(cmd) {
                            error!("Error while using command: {e}");
                        }
                    }
                    ClientMessageContent::Leaderboard => {
                        // Send current leaderboard to requesting client
                        if let Some(player_id) = msg.player_id {
                            let leaderboard = self.services.leaderboard.lock().unwrap();
                            let message = ServerMessageWithRecipient {
                                recipient: Recipient::Single(player_id),
                                message: ServerMessage::Leaderboard((*leaderboard).clone()),
                            };

                            if let Err(e) = self.services.sender.send(message) {
                                eprintln!("Failed to send leaderboard: {}", e);
                            }
                        }
                    }
                    ClientMessageContent::LeaderboardSubmit(player_id, name) => {
                        // Get avatar stats
                        let mut world = self.services.world.lock().unwrap();
                        if let Some(gameover) = world.retire_player(player_id) {
                            let entry = LeaderboardEntry::new(name, &gameover);
                            {
                                let mut leaderboard = self.services.leaderboard.lock().unwrap();
                                leaderboard.add(entry);
                            }

                            let leaderboard = self.services.leaderboard.lock().unwrap();
                            let message = ServerMessageWithRecipient {
                                recipient: Recipient::Broadcast,
                                message: ServerMessage::Leaderboard((*leaderboard).clone()),
                            };

                            if let Err(e) = self.services.sender.send(message) {
                                error!("Failed to broadcast leaderboard update: {}", e);
                            }
                        }
                    }
                }
            }
        });
    }
}
