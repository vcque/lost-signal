use anyhow::Result;
use losig_core::{
    network::{CommandMessage, ServerMessage, TurnResultMessage},
    types::AvatarId,
};

use crate::{
    services::Services,
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

        if let Some(result) = result {
            // Send turn result with senses info
            let msg = TurnResultMessage {
                avatar_id,
                turn,
                stage: 0, // TODO: to update when changing lvls is implemented
                info: result.senses_info,
            };
            let msg = ServerMessageWithRecipient {
                recipient: Recipient::Single(avatar_id),
                message: ServerMessage::Turn(msg),
            };
            self.services.sender.send(msg).unwrap();

            // Send gameover messages
            for (aid, gameover) in result.gameovers {
                let msg = ServerMessageWithRecipient {
                    recipient: Recipient::Single(aid),
                    message: ServerMessage::GameOver(gameover),
                };
                self.services.sender.send(msg).unwrap();
            }

            // Send revert gameover messages
            for aid in result.gameovers_reverted {
                let msg = ServerMessageWithRecipient {
                    recipient: Recipient::Single(aid),
                    message: ServerMessage::RevertGameOver(avatar_id),
                };
                self.services.sender.send(msg).unwrap();
            }
        }

        Ok(())
    }
}
