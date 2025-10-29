use std::{
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    thread::spawn,
};

use losig_core::{
    sense::Senses,
    types::{Action, EntityId},
};

use crate::{world::WorldView, CommandMessage, SenseMessage};

pub struct GameSim {
    entity_id: EntityId,
    world: Arc<Mutex<WorldView>>,
    commands: Sender<CommandMessage>,
    senses: Option<Receiver<SenseMessage>>,
}

impl GameSim {
    pub fn new(
        entity_id: EntityId,
        commands: Sender<CommandMessage>,
        senses: Receiver<SenseMessage>,
    ) -> GameSim {
        let world = WorldView::new();
        let world = Arc::new(Mutex::new(world));
        let senses = Some(senses);
        GameSim {
            entity_id,
            world,
            commands,
            senses,
        }
    }

    pub fn run(&mut self) {
        let senses = self.senses.take().unwrap();
        let world = self.world.clone();
        spawn(move || {
            loop {
                let sense_info = senses.recv().unwrap().senses;

                // Handle each sense
                {
                    let mut world = world.lock().unwrap();
                    world.apply(sense_info);
                }
            }
        });
    }

    pub fn act(&self, action: Action, senses: Senses) {
        // Handle each action
        if let Action::Move(dir) = action {
            let mut world = self.world.lock().unwrap();
            world.shift(-dir.offset());
        }

        self.commands
            .send(CommandMessage {
                entity_id: self.entity_id,
                tick: None,
                action,
                senses,
            })
            .unwrap();
    }

    pub fn world(&self) -> WorldView {
        self.world.lock().unwrap().clone()
    }
}
