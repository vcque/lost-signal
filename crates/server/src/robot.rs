#![allow(clippy::all)]
#![allow(dead_code)]

use losig_core::types::{Action, EntityId};
use losig_core::{sense::Senses, types::Direction};
use rand::{rng, Rng};
use std::time::Duration;
use std::{sync::Arc, thread::sleep};

use crate::{command::CommandMessage, states::States};

const ROBOT_ID: EntityId = 1; // Fixed entity ID for the robot

pub struct Robot {
    states: Arc<States>,
}

impl Robot {
    pub fn new(states: &Arc<States>) -> Self {
        Self {
            states: states.clone(),
        }
    }

    pub fn run(&mut self) {
        self.spawn_robot();

        loop {
            self.move_randomly();
            // Sleep to match game tick duration
            sleep(Duration::from_millis(300));
        }
    }

    fn spawn_robot(&self) {
        let spawn_command = CommandMessage {
            entity_id: ROBOT_ID,
            tick: None,
            action: Action::Spawn,
            senses: Senses::default(),
        };

        self.states.commands.send(spawn_command).unwrap();
    }

    fn move_randomly(&self) {
        let directions = [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
            Direction::UpLeft,
            Direction::UpRight,
            Direction::DownLeft,
            Direction::DownRight,
        ];

        let mut rng = rng();
        let random_direction = directions[rng.random_range(0..directions.len())];

        let move_command = CommandMessage {
            entity_id: ROBOT_ID,
            tick: None,
            action: Action::Move(random_direction),
            senses: Senses::default(),
        };

        self.states.commands.send(move_command).unwrap();
    }
}
