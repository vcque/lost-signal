use anyhow::Result;
use losig_core::{
    network::{CommandMessage, ServerMessage, TransitionMessage, TurnMessage},
    types::{GameOver, GameOverStatus, PlayerId},
};

use crate::{
    services::Services,
    world::{CommandResult, CommandResultOutcome, Limbo},
    ws_server::{Recipient, ServerMessageWithRecipient},
};

/// More like GameAPI
pub struct Game {
    services: Services,
}

impl Game {
    pub fn new(services: Services) -> Self {
        Game { services }
    }

    pub fn new_player(&mut self, pid: PlayerId, name: Option<String>) {
        let mut world = self.services.world.lock().unwrap();
        world.new_player(pid, name);
    }

    pub fn player_command(
        &mut self,
        CommandMessage {
            player_id,
            turn,
            action,
            senses,
        }: CommandMessage,
    ) -> Result<()> {
        let mut world = self.services.world.lock().unwrap();
        let result = world.add_command(player_id, action, senses);

        let CommandResult {
            timeline_updates,
            limbos,
            outcome,
        } = result?;

        match outcome {
            CommandResultOutcome::Turn {
                stage,
                stage_turn,
                info,
                action,
                events,
                timeline,
            } => {
                // Send turn result with senses info
                let msg = TurnMessage {
                    player_id,
                    turn,
                    stage_turn,
                    stage,
                    action,
                    info,
                    events,
                    timeline,
                };
                let msg = ServerMessageWithRecipient {
                    recipient: Recipient::Single(player_id),
                    message: ServerMessage::Turn(msg),
                };
                self.services.sender.send(msg).unwrap();
            }
            CommandResultOutcome::Transition {
                stage,
                stage_turn,
                info,
                timeline,
            } => {
                let msg = TransitionMessage {
                    player_id,
                    turn,
                    stage_turn,
                    stage,
                    info,
                    timeline,
                };
                let msg = ServerMessageWithRecipient {
                    recipient: Recipient::Single(player_id),
                    message: ServerMessage::Transition(msg),
                };
                self.services.sender.send(msg).unwrap();
            }
            CommandResultOutcome::Gameover(gameover) => {
                let msg = ServerMessageWithRecipient {
                    recipient: Recipient::Single(player_id),
                    message: ServerMessage::GameOver(gameover),
                };
                self.services.sender.send(msg).unwrap();
            }
        }
        for (stage_id, timeline) in timeline_updates {
            let infos = world.get_all_infos_for_stage(stage_id);
            for (pid, stage_turn, senses_info) in infos {
                if pid == player_id {
                    // Don't send timeline update to player
                    continue;
                }
                let msg = ServerMessageWithRecipient {
                    recipient: Recipient::Single(pid),
                    message: ServerMessage::Timeline(
                        stage_id,
                        stage_turn,
                        timeline,
                        Some(senses_info),
                    ),
                };
                self.services.sender.send(msg).unwrap();
            }
        }

        for limbo in limbos {
            match limbo {
                Limbo::Dead(avatar) | Limbo::TooFarBehind(avatar) => {
                    let msg = ServerMessageWithRecipient {
                        recipient: Recipient::Single(avatar.player_id),
                        message: ServerMessage::GameOver(GameOver::new(
                            &avatar,
                            GameOverStatus::Dead,
                            1,
                        )),
                    };
                    self.services.sender.send(msg).unwrap();
                }
                Limbo::Averted(pid, senses_info) => {
                    let msg = ServerMessageWithRecipient {
                        recipient: Recipient::Single(pid),
                        message: ServerMessage::Limbo {
                            averted: true,
                            senses_info: Some(senses_info),
                        },
                    };
                    self.services.sender.send(msg).unwrap();
                }
                Limbo::MaybeDead(pid) => {
                    let msg = ServerMessageWithRecipient {
                        recipient: Recipient::Single(pid),
                        message: ServerMessage::Limbo {
                            averted: false,
                            senses_info: None,
                        },
                    };
                    self.services.sender.send(msg).unwrap();
                }
            }
        }

        Ok(())
    }
}
