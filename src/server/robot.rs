#![allow(clippy::all)]
#![allow(dead_code)]

use lost_signal::common::{command::Command, sense::Senses, types::Direction};
use rand::{Rng, rng};
use std::time::Duration;
use std::{sync::Arc, thread::sleep};

use crate::{
    command::{CommandMessage, CommandQueue},
    states::States,
};

const ROBOT_ID: u64 = 1; // Fixed entity ID for the robot

pub struct Robot {
    states: Arc<States>,
    // TOOD: might need to check the world state to know which tick to use
    current_tick: u64,
    spawned: bool,
}

impl Robot {
    pub fn new(states: &Arc<States>) -> Self {
        Self {
            states: states.clone(),
            current_tick: 0,
            spawned: false,
        }
    }

    pub fn run(&mut self) {
        loop {
            if !self.spawned {
                self.spawn_robot();
                self.spawned = true;
            } else if self.current_tick % 2 == 0 {
                // Move every other tick
                self.move_randomly();
            }

            self.current_tick = self.current_tick.wrapping_add(1);

            // Sleep to match game tick duration
            sleep(Duration::from_millis(300));
        }
    }

    fn spawn_robot(&self) {
        let spawn_command = CommandMessage {
            entity_id: ROBOT_ID,
            tick: self.current_tick,
            content: Command::Spawn,
            senses: Senses::default(),
        };

        self.states.command_queue.send_command(spawn_command);
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
            tick: self.current_tick,
            content: Command::Move(random_direction),
            senses: Senses::default(),
        };

        self.states.command_queue.send_command(move_command);
    }
}
