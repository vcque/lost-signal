use log::debug;
use losig_core::{
    network::CommandMessage,
    sense::{SenseInfo, Senses},
    types::{Action, AvatarId},
};

use crate::world::WorldView;

type CallbackFn = Box<dyn Fn(CommandMessage) + Send>;

pub struct GameSim {
    pub avatar_id: AvatarId,
    world: WorldView,
    on_act: CallbackFn,
}

impl GameSim {
    pub fn new(avatar_id: AvatarId) -> GameSim {
        GameSim {
            world: WorldView::new(avatar_id),
            on_act: Box::new(|_| {}),
            avatar_id,
        }
    }

    pub fn set_callback(&mut self, on_act: CallbackFn) {
        self.on_act = on_act;
    }

    pub fn update(&mut self, turn: u64, senses: SenseInfo) {
        self.world.update(turn, senses);
    }

    pub fn act(&mut self, action: Action, senses: Senses) {
        // Handle each action
        debug!("{action:?}, {senses:?}");
        self.world.act(&action, &senses);
        let msg = CommandMessage {
            avatar_id: self.avatar_id,
            turn: self.world.turn,
            action,
            senses,
        };

        (self.on_act)(msg);
    }

    pub fn world(&self) -> &WorldView {
        &self.world
    }
}
