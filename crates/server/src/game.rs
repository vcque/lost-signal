use anyhow::Result;
use losig_core::{
    network::{CommandMessage, GameLogsMessage, ServerMessage, TurnResultMessage},
    types::{AvatarId, GameOver, GameOverStatus},
};

use crate::{
    services::Services,
    world::Limbo,
    ws_server::{Recipient, ServerMessageWithRecipient},
};

pub struct Game {
    services: Services,
}

impl Game {
    pub fn new(services: Services) -> Self {
        Game { services }
    }

    pub fn new_player(&mut self, aid: AvatarId) {
        let mut world = self.services.world.lock().unwrap();
        world.create_avatar(aid);
    }

    pub fn player_command(
        &mut self,
        CommandMessage {
            avatar_id,
            turn,
            action,
            senses,
        }: CommandMessage,
    ) -> Result<()> {
        let mut world = self.services.world.lock().unwrap();
        let result = world.add_command(avatar_id, action, senses);

        if let Ok(result) = result {
            // Send turn result with senses info
            let msg = TurnResultMessage {
                avatar_id,
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
                recipient: Recipient::Single(avatar_id),
                message: ServerMessage::Turn(msg),
            };
            self.services.sender.send(msg).unwrap();

            for (stage_id, timeline) in result.timeline_updates {
                let aids = world.get_aids_for_stage(stage_id);
                let msg = ServerMessageWithRecipient {
                    recipient: Recipient::Multi(aids),
                    message: ServerMessage::Timeline(stage_id, timeline),
                };
                self.services.sender.send(msg).unwrap();
            }

            for limbo in result.limbos {
                match limbo {
                    Limbo::Dead(avatar) | Limbo::TooFarBehind(avatar) => {
                        let msg = ServerMessageWithRecipient {
                            recipient: Recipient::Single(avatar.id),
                            message: ServerMessage::GameOver(GameOver::new(
                                &avatar,
                                GameOverStatus::Dead,
                                result.stage,
                            )),
                        };
                        self.services.sender.send(msg).unwrap();
                    }
                    Limbo::Averted(aid, senses_info) => {
                        let msg = ServerMessageWithRecipient {
                            recipient: Recipient::Single(aid),
                            message: ServerMessage::Limbo {
                                averted: true,
                                senses_info: Some(senses_info),
                            },
                        };
                        self.services.sender.send(msg).unwrap();
                    }
                    Limbo::MaybeDead(aid) => {
                        let msg = ServerMessageWithRecipient {
                            recipient: Recipient::Single(aid),
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
