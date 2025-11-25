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
        let info = world.add_command(avatar_id, action, senses);

        if let Some(info) = info {
            let msg = TurnResultMessage {
                avatar_id,
                turn,
                stage: 0, // TODO: to update when changing lvls is implemented
                info,
            };
            let msg = ServerMessageWithRecipient {
                recipient: Recipient::Single(avatar_id),
                message: ServerMessage::Turn(msg),
            };
            self.services.sender.send(msg).unwrap();
        }

        Ok(())
    }
}

