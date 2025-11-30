use anyhow::Result;
use losig_core::{
    network::{CommandMessage, GameLogsMessage, ServerMessage, TurnResultMessage},
    types::{GameOver, GameOverStatus, PlayerId},
};

use crate::{
    services::Services,
    world::Limbo,
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

    pub fn new_player(&mut self, pid: PlayerId) {
        let mut world = self.services.world.lock().unwrap();
        world.new_player(pid);
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

        if let Ok(result) = result {
            // Send turn result with senses info
            let msg = TurnResultMessage {
                player_id,
                turn,
                stage_turn: result.stage_turn,
                stage: result.stage as u8,
                action: result.action,
                info: result.senses_info,
                logs: GameLogsMessage {
                    from: 0,
                    logs: result.logs,
                },
            };
            let msg = ServerMessageWithRecipient {
                recipient: Recipient::Single(player_id),
                message: ServerMessage::Turn(msg),
            };
            self.services.sender.send(msg).unwrap();

            for (stage_id, timeline) in result.timeline_updates {
                let pids = world.get_pids_for_stage(stage_id);
                let msg = ServerMessageWithRecipient {
                    recipient: Recipient::Multi(pids),
                    message: ServerMessage::Timeline(stage_id, timeline),
                };
                self.services.sender.send(msg).unwrap();
            }

            for limbo in result.limbos {
                match limbo {
                    Limbo::Dead(avatar) | Limbo::TooFarBehind(avatar) => {
                        let msg = ServerMessageWithRecipient {
                            recipient: Recipient::Single(avatar.player_id),
                            message: ServerMessage::GameOver(GameOver::new(
                                &avatar,
                                GameOverStatus::Dead,
                                result.stage,
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
        }

        Ok(())
    }
}
