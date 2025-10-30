use log::debug;
use losig_core::{
    network::{UdpCommandPacket, UdpSensesPacket},
    sense::{SenseInfo, Senses},
    types::{Action, EntityId},
};

use crate::world::WorldView;

pub type CommandMessage = UdpCommandPacket;
pub type SenseMessage = UdpSensesPacket;

type CallbackFn = Box<dyn Fn(CommandMessage) + Send>;

pub struct GameSim {
    entity_id: EntityId,
    world: WorldView,
    on_act: CallbackFn,
}

impl GameSim {
    pub fn new(entity_id: EntityId) -> GameSim {
        GameSim {
            world: WorldView::new(),
            on_act: Box::new(|_| {}),
            entity_id,
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
            self.world.shift(-dir.offset());
        }

        let msg = CommandMessage {
            entity_id: self.entity_id,
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
