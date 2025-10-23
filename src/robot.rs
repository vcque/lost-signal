#![allow(clippy::all)]

use rand::{Rng, rng};
use std::thread::sleep;
use std::time::Duration;

use crate::{
    command::{Command, CommandMessage, CommandQueue},
    world::Direction,
};

const ROBOT_ID: u64 = 1; // Fixed entity ID for the robot

pub struct Robot {
    command_queue: CommandQueue,
    // TOOD: might need to check the world state to know which tick to use
    current_tick: u64,
    spawned: bool,
}

impl Robot {
    pub fn new(command_queue: CommandQueue) -> Self {
        Self {
            command_queue,
            current_tick: 1,
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
        };

        self.command_queue.send_command(spawn_command);
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
        };

        self.command_queue.send_command(move_command);
    }
}
