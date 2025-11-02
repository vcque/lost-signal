use log::debug;
use losig_core::{
    network::{UdpCommandPacket, UdpSensesPacket},
    sense::{SenseInfo, Senses},
    types::{Action, AvatarId},
};

use crate::world::WorldView;

pub type CommandMessage = UdpCommandPacket;
pub type SenseMessage = UdpSensesPacket;

type CallbackFn = Box<dyn Fn(CommandMessage) + Send>;

pub struct GameSim {
    pub avatar_id: AvatarId,
    world: WorldView,
    on_act: CallbackFn,
}

impl GameSim {
    pub fn new(avatar_id: AvatarId) -> GameSim {
        GameSim {
            world: WorldView::new(),
            on_act: Box::new(|_| {}),
            avatar_id,
        }
    }

    pub fn set_callback(&mut self, on_act: CallbackFn) {
        self.on_act = on_act;
    }

    pub fn update(&mut self, senses: SenseInfo) {
        self.world.update(senses);
    }

    pub fn act(&mut self, action: Action, senses: Senses) {
        // Handle each action
        debug!("{action:?}, {senses:?}");
        if let Action::Move(dir) = action {
            let new_pos = self.world.viewer + dir.offset();
            let tile = self.world.tile_at(new_pos);
            if tile.can_travel() {
                self.world.viewer = new_pos;
            }
        }

        let msg = CommandMessage {
            avatar_id: self.avatar_id,
            tick: None,
            action,
            senses,
        };

        (self.on_act)(msg);
    }

    pub fn world(&self) -> &WorldView {
        &self.world
    }
}
